import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {assessFourPartyReadyCycle} from '../governance/atto-ready-four-party.mjs';

const HEAD = 'a'.repeat(40), NEW_HEAD = 'b'.repeat(40);
const person = (who = 'ore', when = '2026-11-19T15:20:00Z') => ({
  source: 'github-human', state: 'APPROVED',
  login: who === 'ore' ? 'ORESoftware' : 'the1mills',
  github_id: who === 'ore' ? 11139560 : 2148448,
  submitted_at: when, commit_sha: HEAD
});
const ai = (provider, when = '2026-11-19T15:45:00Z', commit_sha = HEAD) => ({
  source: 'independent-provider-attestation', state: 'APPROVED',
  provider, provider_verified: true, attestation_verified: true,
  agent_identity: provider === 'OpenAI' ? 'codex-001' : 'claude-002',
  submitted_at: when, commit_sha
});
const ready = () => ({
  head_sha: HEAD, author_github_id: 33333333,
  checked_at: '2026-11-19T16:00:00Z',
  ready_cycle_started_at: '2026-11-19T15:00:00Z',
  last_push_at: '2026-11-19T15:30:00Z',
  ready_cycle_verified: true, draft: false,
  labels_verified: true, labels: [],
  reviews_verified: true, reviews: [person(), ai('OpenAI'), ai('Anthropic')]
});
const audit = x => assessFourPartyReadyCycle(x);
const pass = x => assert.equal(audit(x).status, 'REPORTED_SATISFIED_NOT_AUTHORIZED',
  JSON.stringify(audit(x).failures));
const hold = x => assert.equal(audit(x).status, 'HOLD');

test('initial 0-72h lane needs one owner and both AI providers; advisory only', () => {
  const x = ready(); pass(x);
  assert.equal(audit(x).lane, 'initial-two-ai');
  assert.equal(audit(x).advisory_only, true);
  assert.deepEqual(audit(x).required_ai, ['openai','anthropic']);
  x.reviews = [person(), ai('OpenAI')]; hold(x);
  x.reviews = [person()]; hold(x);
  x.reviews = [ai('OpenAI'), ai('Anthropic')]; hold(x);
});
test('either trusted human alone fills one human slot even in initial lane', () => {
  const x = ready();
  x.reviews = [person('mills'), ai('OpenAI'), ai('Anthropic')]; pass(x);
  x.reviews = [person('ore'), ai('OpenAI'), ai('Anthropic')]; pass(x);
  assert.equal(audit(x).human_approvals, 1);
});
test('fast lane opens exactly at 72 hours and requires no new AI review', () => {
  const x = ready(); x.reviews = [person()];
  x.ready_cycle_started_at = '2026-11-16T16:00:00.001Z'; hold(x);
  x.ready_cycle_started_at = '2026-11-16T16:00:00Z'; pass(x);
  assert.equal(audit(x).lane, 'fast-one-human');
  assert.deepEqual(audit(x).required_ai, []);
  x.ready_cycle_started_at = '2026-11-15T16:00:00Z'; pass(x);
});
test('fast-lane review still fails without the named independent human', () => {
  const x = ready(); x.ready_cycle_started_at = '2026-11-16T16:00:00Z';
  x.reviews = []; hold(x);
  x.reviews = [{...person(), login: 'somebody', github_id: 4444}]; hold(x);
});
test('renewed lane starts exactly at 240 hours; human approval persists', () => {
  const x = ready(); x.reviews = [person('ore', '2026-11-10T12:00:00Z')];
  x.ready_cycle_started_at = '2026-11-09T16:00:00.001Z'; pass(x);
  x.ready_cycle_started_at = '2026-11-09T16:00:00Z'; hold(x);
  assert.equal(audit(x).lane, 'renewed-two-ai');
  x.checked_at = '2026-11-19T16:10:00Z';
  x.reviews.push(ai('OpenAI', '2026-11-19T16:05:00Z'));
  x.reviews.push(ai('Anthropic', '2026-11-19T16:06:00Z')); pass(x);
});
test('renewal requires post-240h current-head attestations by both providers', () => {
  const x = ready(); x.ready_cycle_started_at = '2026-11-09T16:00:00Z';
  hold(x);
  x.checked_at = '2026-11-19T16:10:00Z';
  x.reviews = [person(), ai('OpenAI', '2026-11-19T16:03:00Z'),
    ai('Anthropic', '2026-11-19T16:04:00Z')]; pass(x);
  x.reviews[2].commit_sha = NEW_HEAD; hold(x);
});
test('sync with main preserves a single human approval but refreshes both AIs', () => {
  const x = ready(); x.head_sha = NEW_HEAD;
  x.last_push_at = '2026-11-19T15:49:00Z';
  x.reviews = [person('ore', '2026-11-19T15:10:00Z'),
    ai('OpenAI','2026-11-19T15:50:00Z',NEW_HEAD),
    ai('Anthropic','2026-11-19T15:51:00Z',NEW_HEAD)];
  pass(x);
  x.reviews[2].commit_sha = HEAD; hold(x);
  x.reviews[2].commit_sha = NEW_HEAD; x.reviews[2].submitted_at = '2026-11-19T15:48:00Z'; hold(x);
});
test('two provider slots cannot be filled by the same claimed agent identity', () => {
  const x = ready();
  x.reviews[2].agent_identity = x.reviews[1].agent_identity; hold(x);
});
test('latest authenticated AI dismissal or changes request revokes prior attestation', () => {
  const x = ready();
  pass(x);
  x.reviews.push({...ai('OpenAI','2026-11-19T15:55:00Z'), state:'DISMISSED'});
  hold(x);
  x.reviews.push(ai('OpenAI','2026-11-19T15:56:00Z'));
  pass(x);
  x.reviews.push({...ai('Anthropic','2026-11-19T15:57:00Z'), state:'CHANGES_REQUESTED'});
  hold(x);
  x.reviews.push(ai('Anthropic','2026-11-19T15:58:00Z'));
  pass(x);
});
test('a late AI COMMENTED event does not revoke a valid attestation', () => {
  const x=ready();
  x.reviews.push({...ai('OpenAI','2026-11-19T15:59:00Z'),state:'COMMENTED'});
  pass(x);
});
test('a new Ready cycle keeps human review but invalidates old-cycle AI evidence', () => {
  const x = ready(); x.ready_cycle_started_at = '2026-11-19T15:50:00Z';
  x.reviews = [person('ore','2026-11-18T12:00:00Z'), ai('OpenAI'), ai('Anthropic')]; hold(x);
  x.reviews[1].submitted_at = '2026-11-19T15:55:00Z';
  x.reviews[2].submitted_at = '2026-11-19T15:56:00Z'; pass(x);
});
test('real immutable GitHub IDs, not usernames alone, establish owner identity', () => {
  const x = ready(); x.reviews[0] = {...person(), github_id: 2148448}; hold(x);
  x.reviews[0] = {...person('mills'), github_id: 11139560}; hold(x);
  x.reviews[0] = person('mills'); pass(x);
});
test('PR author cannot count their own approval', () => {
  const x = ready(); x.author_github_id = 11139560; hold(x);
  x.reviews[0] = person('mills'); pass(x);
  x.author_github_id = 2148448; hold(x);
});
test('latest dismissed human review is not active', () => {
  const x = ready();
  x.reviews.push({...person('ore','2026-11-19T15:59:00Z'), state:'DISMISSED'}); hold(x);
  x.reviews.push(person('ore','2026-11-19T15:59:30Z')); pass(x);
});
test('outstanding changes request blocks merge, later resolution clears it', () => {
  const x = ready();
  x.reviews.push({...person('mills','2026-11-19T15:50:00Z'), state:'CHANGES_REQUESTED'});
  hold(x);
  x.reviews.push(person('mills','2026-11-19T15:55:00Z')); pass(x);
  x.reviews.push({...person('ore','2026-11-19T15:59:00Z'), state:'CHANGES_REQUESTED'}); hold(x);
});
test('unknown reviewer CHANGES_REQUESTED is also a real block', () => {
  const x = ready();
  x.reviews.push({source:'github-human',state:'CHANGES_REQUESTED',
    login:'reviewer',github_id:55555,submitted_at:'2026-11-19T15:55:00Z'}); hold(x);
});
test('later COMMENTED review does not revoke an APPROVED review', () => {
  const x = ready();
  x.reviews.push({...person('ore','2026-11-19T15:59:00Z'),state:'COMMENTED'}); pass(x);
});
test('all draft hold labels fail closed, including GitHub object labels', () => {
  for(const label of ['keep-as-draft','keep-draft','not-ready']) {
    const x = ready(); x.labels = [label.toUpperCase()]; hold(x);
    x.labels = [{name:label}]; hold(x);
  }
  const x=ready(); x.labels=[{bad:'not-ready'}]; hold(x);
  x.labels=[];x.labels_verified=false;hold(x);
});
test('missing lifecycle, provenance, author, or review history is a HOLD', () => {
  const x=ready();x.ready_cycle_verified=false;hold(x);
  x.ready_cycle_verified=true;x.draft=true;hold(x);
  x.draft=false;x.reviews_verified=false;hold(x);
  x.reviews_verified=true;x.author_github_id=null;hold(x);
  x.author_github_id=33333333;x.ready_cycle_started_at='invalid';hold(x);
});
test('forged, malformed, or ambiguous review evidence fails closed', () => {
  const x=ready();
  x.reviews[1].attestation_verified=false;hold(x);
  x.reviews=ready().reviews;
  x.reviews[2]={...person('ore'),provider:'Anthropic'};hold(x);
  x.reviews=ready().reviews;
  x.reviews.push(person('ore'));hold(x);
  x.reviews=[{source:'forged',state:'APPROVED',submitted_at:'2026-11-19T15:40:00Z'},...ready().reviews];hold(x);
});
test('future reviews and invalid head timestamps cannot satisfy', () => {
  const x=ready();x.reviews[0].submitted_at='2026-12-01T00:00:00Z';hold(x);
  x.reviews=ready().reviews; x.head_sha='bad'; hold(x);
  x.head_sha=HEAD;x.last_push_at='2026-11-19T17:00:00Z';hold(x);
});

test('canonical policy documentation still contains both boundaries and draft holds', () => {
  const canonical=readFileSync(new URL('../AGENTS.md',import.meta.url),'utf8');
  const humans=readFileSync(new URL('../docs/ORG_TRUSTED_MERGE_APPROVALS.md',import.meta.url),'utf8');
  for(const exact of ['0 ≤ ready_age < 72h', '72h ≤ ready_age < 240h',
                       'ready_age ≥ 240h', 'OpenAI', 'Anthropic',
                       'keep-as-draft', 'keep-draft', 'not-ready']) {
    assert.ok(canonical.includes(exact), 'missing canonical policy requirement: '+exact);
  }
  assert.match(canonical,/one[^.\n]{0,100}human approval from \`the1mills\`[^.\n]{0,50}OR[^.\n]{0,50}\`ORESoftware\`/i);
  assert.match(humans,/human-approval slot/i);
  assert.match(humans,/72 hours/);
  assert.match(humans,/240 hours/);
});

test('central policy regression workflow pins immutable head and cannot be path-skipped', () => {
  const workflow=readFileSync(new URL('../.github/workflows/atto-ready-policy-tests.yml', import.meta.url),'utf8');
  for(const required of [
    'pull_request:', 'push:', 'workflow_dispatch:',
    'EXPECTED_HEAD: ${{ github.event.pull_request.head.sha || github.sha }}',
    'ref: ${{ github.event.pull_request.head.sha || github.sha }}',
    'persist-credentials: false',
    'test "$(git rev-parse HEAD)" = "$EXPECTED_HEAD"',
    'node --test tests/atto-ready-four-party.test.mjs'
  ]) assert.ok(workflow.includes(required), 'missing head-pinned test requirement: '+required);
  assert.doesNotMatch(workflow,/^\\s+paths:/m);
});
