import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { decideRecovery, formatRecoveryHandoff } from './ci-recovery-gate.mjs';

const HEAD = 'a'.repeat(40), TREE = 'b'.repeat(40), CARRIER_HEAD = 'c'.repeat(40);
const source = {
  repository: 'atto-lang/atto-source.rs', repo_id: 13579, visibility: 'private',
  head_sha: HEAD, tree_sha: TREE, run_id: 123,
};
const observed = { status: 'NOT_EXECUTED' };
const carrier = {
  repository: 'file-tunnel-test/atto-ci', repo_id: 24680, visibility: 'private',
  probe: { conclusion: 'success', run_id: 321, runner_id: 456,
    executed_steps: 6, head_sha: CARRIER_HEAD, carrier_commit_sha: CARRIER_HEAD, job_status: 'completed' },
};
const check = overrides => decideRecovery({ source, observed, carrier, ...overrides });

test('zero-step without candidate requests source-free probe', () => {
  assert.equal(check({ carrier: undefined }).status, 'PROBE_CARRIER_CAPACITY');
});
test('a successful source-free canary never grants private-source export', () => {
  const p = check();
  assert.equal(p.status, 'PRIVATE_EXPORT_AUTHORIZATION_REQUIRED');
  assert.equal(p.authorized_private_copy, false);
  assert.equal(p.mirror_performed, false);
  assert.equal(p.material_tests_verified, false);
});
test('self-claimed approval true never overrides independent security boundary', () => {
  for (const value of [
    { approved: true }, { state: 'APPROVED', approver: 'the1mills' },
    { export_decision: 'ALLOW', guard_enforced: true, carrier_repo_id: 24680 },
  ]) {
    assert.equal(check({ authorization: value }).status, 'PRIVATE_EXPORT_AUTHORIZATION_REQUIRED');
  }
});
test('explicit DENIED export claim is not treated as permission', () => {
  assert.equal(check({ carrier: { ...carrier, export_decision: 'DENIED' } }).status,
    'PRIVATE_EXPORT_AUTHORIZATION_REQUIRED');
});
test('mixed stepful failure and zero-step admission preserve both blockers without export', () => {
  const p = check({ observed: { status: 'MIXED_ADMISSION_AND_TEST_FAILURE',
    failed_job_ids: [31], job_ids: [32] } });
  assert.equal(p.status, 'DIAGNOSE_MIXED_ADMISSION_AND_TEST_FAILURE');
  assert.match(p.action, /executed test failure/);
  assert.match(p.action, /runner admission/);
  assert.equal(p.authorized_private_copy, false);
  assert.equal(p.mirror_performed, false);
  assert.equal(p.material_tests_verified, false);
  assert.equal(p.carrier_repository, undefined);
});
test('mixed failed job without carrier cannot be quietly treated as a capacity-only probe', () => {
  const p = check({ carrier: undefined, observed: { status: 'MIXED_ADMISSION_AND_TEST_FAILURE' } });
  assert.equal(p.status, 'DIAGNOSE_MIXED_ADMISSION_AND_TEST_FAILURE');
  assert.equal(p.authorized_private_copy, false);
});

test('real stepful compiler failure is not classified as missing capacity', () => {
  const p = check({ observed: { status: 'TEST_FAILED' } });
  assert.equal(p.status, 'DIAGNOSE_ACTUAL_TEST_FAILURE');
  assert.equal(p.carrier_repository, undefined);
});
test('steps reported successful do not become material test proof', () => {
  const p = check({ observed: { status: 'STEPFUL_RUN_REPORTED_SUCCESS' } });
  assert.equal(p.status, 'TEST_COVERAGE_UNVERIFIED');
});
test('incomplete, pending, and contradictory snapshots cause re-fetch, not transfer', () => {
  for (const status of ['INCOMPLETE_JOB_EVIDENCE', 'INCONSISTENT_JOB_EVIDENCE', 'PENDING_OR_INCONCLUSIVE']) {
    const p = check({ observed: { status } });
    assert.equal(p.status, 'RECHECK_RUN_EVIDENCE');
    assert.equal(p.authorized_private_copy, false);
  }
});
test('stale head requires exact new source head', () => {
  assert.equal(check({ observed: { status: 'STALE_HEAD' } }).status, 'RERUN_CURRENT_HEAD');
});
test('candidate must belong to another GitHub organization', () => {
  const p = check({ carrier: { ...carrier, repository: 'atto-lang/alternate-ci' } });
  assert.equal(p.status, 'CARRIER_REJECTED');
});
test('candidate must be private, not merely public source visible', () => {
  assert.equal(check({ carrier: { ...carrier, visibility: 'public' } }).status, 'CARRIER_REJECTED');
});
test('source and carrier cannot be same repository', () => {
  assert.equal(check({ carrier: { ...carrier, repository: source.repository } }).status, 'CARRIER_REJECTED');
});
test('carrier runner absent cannot be considered a budget proof', () => {
  const p = check({ carrier: { ...carrier, probe: { ...carrier.probe, runner_id: 0 } } });
  assert.equal(p.status, 'PROBE_CARRIER_CAPACITY');
});
test('carrier zero executed steps cannot be considered a budget proof', () => {
  const p = check({ carrier: { ...carrier, probe: { ...carrier.probe, executed_steps: 0 } } });
  assert.equal(p.status, 'PROBE_CARRIER_CAPACITY');
});
test('carrier success on a different Git commit cannot be used as proof', () => {
  const p = check({ carrier: { ...carrier, probe: { ...carrier.probe, carrier_commit_sha: HEAD } } });
  assert.equal(p.status, 'PROBE_CARRIER_CAPACITY');
});
test('cancelled carrier run cannot be considered a capacity proof', () => {
  const p = check({ carrier: { ...carrier, probe: { ...carrier.probe, job_status: 'cancelled' } } });
  assert.equal(p.status, 'PROBE_CARRIER_CAPACITY');
});
test('public source does not grant private export permission', () => {
  const p = check({ source: { ...source, visibility: 'public' } });
  assert.equal(p.status, 'PUBLIC_MIRROR_PLAN_ONLY');
  assert.equal(p.authorized_private_copy, false);
  assert.equal(p.mirror_performed, false);
});
test('unverified source visibility fails closed', () => {
  assert.equal(check({ source: { ...source, visibility: 'secret' } }).status, 'SOURCE_VISIBILITY_UNKNOWN');
});
test('invalid commit SHA fails closed before any carrier step', () => {
  assert.equal(check({ source: { ...source, head_sha: 'main' } }).status, 'EVIDENCE_INVALID');
});
test('invalid source repo ID fails closed', () => {
  assert.equal(check({ source: { ...source, repo_id: 0 } }).status, 'EVIDENCE_INVALID');
});
test('unrecognized diagnosis is not treated as zero steps', () => {
  assert.equal(check({ observed: { status: 'all-green' } }).status, 'EVIDENCE_INVALID');
});
test('plain-text handoff never alleges export or testing took place', () => {
  const handoff = formatRecoveryHandoff({ source, observed, carrier });
  assert.match(handoff, /approved_private_export=NO/);
  assert.match(handoff, /source_mirror_done=NO/);
  assert.match(handoff, /material_tests_verified=NO/);
  assert.match(handoff, /source_tree=b{40}/);
});

test('CI recovery instructions cannot erase canonical persistent-human Ready-cycle gates', () => {
  const agents = readFileSync(new URL('../AGENTS.md', import.meta.url), 'utf8');
  for (const required of [
    '0 ≤ ready_age < 72h',
    '72h ≤ ready_age < 240h',
    'ready_age ≥ 240h',
    'OpenAI',
    'Anthropic',
    'Human approval is required once per PR',
    'keep-as-draft',
    'keep-draft',
    'not-ready',
    'Mandatory zero-step CI recovery:',
    'governance/ci-zero-step-recovery.md',
  ]) {
    assert.ok(agents.includes(required), 'canonical policy requirement missing: ' + required);
  }
  assert.ok(!agents.includes('Single trusted-owner approval, any Ready age'));
  assert.ok(!agents.includes('or reapproval after 72h, 240h, 10 days, or another age boundary'));
});

test('private callers never fetch mutable public diagnostic scripts using their read token', () => {
  const watcher = readFileSync(new URL('../.github/workflows/ci-zero-step-watcher.yml', import.meta.url), 'utf8');
  const pin = '82f050ee5d4043d98c0150458bca743970b390ed';
  assert.match(watcher, new RegExp('^\\s+ref: ' + pin + '$', 'm'));
  assert.ok(watcher.includes('persist-credentials: false'));
  assert.ok(watcher.includes('actions: read'));
  assert.ok(watcher.includes('contents: read'));
  assert.ok(!watcher.includes("github.ref_name"));
  assert.ok(!watcher.includes('ref: main'));
});

test('required recovery CI is unfiltered and verifies exact PR head before tests', () => {
  const workflow = readFileSync(new URL('../.github/workflows/ci-zero-step-policy.yml', import.meta.url), 'utf8');
  for (const contract of [
    'pull_request:',
    'workflow_dispatch:',
    'EXPECTED_HEAD: ${{ github.event.pull_request.head.sha || github.sha }}',
    'ref: ${{ github.event.pull_request.head.sha || github.sha }}',
    'persist-credentials: false',
    'test "$(git rev-parse HEAD)" = "$EXPECTED_HEAD"',
    'git diff --quiet --exit-code',
    'node --test governance/ci-zero-step-classifier.test.mjs',
  ]) {
    assert.ok(workflow.includes(contract), 'required head-pinned CI contract missing: ' + contract);
  }
  assert.doesNotMatch(workflow, /^\s+paths:/m);
  assert.doesNotMatch(workflow, /pull_request_target:/);
  assert.doesNotMatch(workflow, /ref: \$\{\{ github\.ref \}\}/);
});

test('pull-request-controlled CI scripts never receive the Actions REST token', () => {
  const workflow = readFileSync(new URL('../.github/workflows/ci-zero-step-policy.yml', import.meta.url), 'utf8');
  assert.match(workflow, /- name: Smoke-test live same-repository GitHub Actions REST reader\n\s+if: github\.event_name != 'pull_request'\n\s+env:\n\s+GITHUB_TOKEN: /);
  assert.match(workflow, /permissions:\n\s+contents: read\n\s+actions: read/);
  assert.ok(workflow.includes('persist-credentials: false'));
  assert.doesNotMatch(workflow, /pull_request_target:/);
});
