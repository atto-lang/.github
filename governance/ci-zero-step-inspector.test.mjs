import test from 'node:test';
import assert from 'node:assert/strict';
import {
  inspectRun, inspectRecent, summarize, formatSummary, createGitHubReader, MONITOR_NAME,
} from './ci-zero-step-inspector.mjs';

const repository = 'atto-lang/.github';
const sha = 'a'.repeat(40);
const makeRun = (props = {}) => ({
  id: 42, run_attempt: 2, name: 'compiler', head_sha: sha,
  status: 'completed', conclusion: 'failure',
  repository: { full_name: repository }, html_url: 'https://github.com/atto-lang/.github/actions/runs/42',
  ...props,
});
const success = id => ({
  id, name: 'test', runner_id: 100, status: 'completed', conclusion: 'success',
  steps: [{ name: 'test', status: 'completed', conclusion: 'success' }],
});
const failed = id => ({
  ...success(id), conclusion: 'failure',
  steps: [{ name: 'test', status: 'completed', conclusion: 'failure' }],
});
const zerostep = id => ({ ...success(id), runner_id: 0, conclusion: 'failure', steps: null });

function fakeApi(run, pages, expected = []) {
  const calls = [];
  const read = async route => {
    calls.push(route);
    if (route === '/repos/atto-lang/.github/actions/runs/42') return run;
    const match = route.match(/\/attempts\/(\d+)\/jobs\?per_page=100&page=(\d+)$/);
    if (match) return pages[Number(match[2]) - 1];
    throw new Error('unexpected endpoint: ' + route);
  };
  return { read, calls, expected };
}
test('zero-step run is detected using exact attempt 2 and never classified compiler failure', async () => {
  const f = fakeApi(makeRun(), [{ total_count: 1, jobs: [zerostep(1)] }]);
  const result = await inspectRun({ repository, runId: 42, getJson: f.read });
  assert.equal(result.status, 'NOT_EXECUTED');
  assert.equal(result.reported_job_count, 1);
  assert.equal(result.attempt, 2);
  assert.ok(f.calls[1].includes('/attempts/2/jobs?per_page=100&page=1'));
  assert.ok(!f.calls.some(x => /attempts\/1\//.test(x)));
});
test('genuine stepful failure is classified as TEST_FAILED', async () => {
  const f = fakeApi(makeRun(), [{ total_count: 1, jobs: [failed(1)] }]);
  const result = await inspectRun({ repository, runId: 42, getJson: f.read });
  assert.equal(result.status, 'TEST_FAILED');
  assert.equal(result.total_steps, 1);
});
test('runner absence with no jobs cannot be called a passing build', async () => {
  const f = fakeApi(makeRun(), [{ total_count: 0, jobs: [] }]);
  const result = await inspectRun({ repository, runId: 42, getJson: f.read });
  assert.equal(result.status, 'NOT_EXECUTED');
});
test('complete successful head and job metadata gives only a scoped metadata outcome', async () => {
  const f = fakeApi(makeRun({ conclusion: 'success' }),
    [{ total_count: 1, jobs: [success(1)] }]);
  const r = await inspectRun({ repository, runId: 42, expectedSha: sha, getJson: f.read });
  assert.equal(r.status, 'STEPFUL_RUN_REPORTED_SUCCESS');
  assert.equal(r.head_sha, sha);
});
test('run success contradicting failed job cannot be treated green', async () => {
  const f = fakeApi(makeRun({ conclusion: 'success' }), [{ total_count: 1, jobs: [failed(1)] }]);
  assert.equal((await inspectRun({ repository, runId: 42, getJson: f.read })).status, 'INCONSISTENT_JOB_EVIDENCE');
});
test('run failure contradicting successful jobs cannot be treated green', async () => {
  const f = fakeApi(makeRun(), [{ total_count: 1, jobs: [success(1)] }]);
  assert.equal((await inspectRun({ repository, runId: 42, getJson: f.read })).status, 'INCONSISTENT_JOB_EVIDENCE');
});
test('stale head stops before fetching any job pages', async () => {
  const f = fakeApi(makeRun(), [{ total_count: 1, jobs: [success(1)] }]);
  const result = await inspectRun({ repository, runId: 42, expectedSha: 'b'.repeat(40), getJson: f.read });
  assert.equal(result.status, 'STALE_HEAD');
  assert.equal(f.calls.length, 1);
});
test('in-progress run is inconclusive and never triggers no-runner remediation', async () => {
  const f = fakeApi(makeRun({ status: 'in_progress', conclusion: null }), []);
  const result = await inspectRun({ repository, runId: 42, getJson: f.read });
  assert.equal(result.status, 'PENDING_OR_INCONCLUSIVE');
  assert.equal(f.calls.length, 1);
});
test('actual 101-job matrix is fetched across two exact-attempt pages', async () => {
  const first = Array.from({ length: 100 }, (_, i) => success(i + 1));
  const second = [zerostep(101)];
  const f = fakeApi(makeRun(), [
    { total_count: 101, jobs: first }, { total_count: 101, jobs: second },
  ]);
  const result = await inspectRun({ repository, runId: 42, getJson: f.read });
  assert.equal(result.status, 'NOT_EXECUTED');
  assert.equal(result.reported_job_count, 101);
  assert.equal(result.job_ids[0], 101);
  assert.equal(f.calls.length, 3);
});
test('truncated page fails closed', async () => {
  const f = fakeApi(makeRun(), [{ total_count: 101, jobs: [success(1)] }]);
  await assert.rejects(inspectRun({ repository, runId: 42, getJson: f.read }), /incomplete REST job page/);
});
test('changed count between pages fails closed', async () => {
  const f = fakeApi(makeRun(), [
    { total_count: 101, jobs: Array.from({ length: 100 }, (_, i) => success(i + 1)) },
    { total_count: 100, jobs: [success(101)] },
  ]);
  await assert.rejects(inspectRun({ repository, runId: 42, getJson: f.read }), /pagination changed/);
});
test('duplicate job IDs across REST pages cannot be green', async () => {
  const f = fakeApi(makeRun({ conclusion: 'success' }), [
    { total_count: 101, jobs: Array.from({ length: 100 }, (_, i) => success(i + 1)) },
    { total_count: 101, jobs: [success(100)] },
  ]);
  assert.equal((await inspectRun({ repository, runId: 42, getJson: f.read })).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('run identity must agree with local repository', async () => {
  const f = fakeApi(makeRun({ repository: { full_name: 'other/source' } }), []);
  await assert.rejects(inspectRun({ repository, runId: 42, getJson: f.read }), /mismatched immutable/);
});
test('incomplete attempt metadata cannot silently fall back to latest job attempt', async () => {
  const f = fakeApi(makeRun({ run_attempt: null }), []);
  await assert.rejects(inspectRun({ repository, runId: 42, getJson: f.read }), /mismatched immutable/);
  assert.equal(f.calls.length, 1);
});
test('invalid arbitrary repository/run IDs are rejected before API requests', async () => {
  const f = fakeApi(makeRun(), []);
  await assert.rejects(inspectRun({ repository: '../attacker', runId: 42, getJson: f.read }), /invalid/);
  await assert.rejects(inspectRun({ repository, runId: NaN, getJson: f.read }), /invalid/);
  assert.equal(f.calls.length, 0);
});
test('recent watcher skips itself, bounds scan, and reports uncertain API failures', async () => {
  const calls = [];
  const reader = async route => {
    calls.push(route);
    if (route.endsWith('/actions/runs?per_page=3')) return {
      workflow_runs: [{ id: 43, name: MONITOR_NAME }, { id: 42, name: 'compiler' }],
    };
    if (route.endsWith('/runs/42')) return makeRun();
    throw new Error('permission unavailable');
  };
  const r = await inspectRecent({ repository, limit: 3, getJson: reader });
  assert.equal(r.exhaustive, false);
  assert.equal(r.results.length, 1);
  assert.equal(r.results[0].status, 'INCOMPLETE_JOB_EVIDENCE');
  assert.equal(calls.filter(x => x.includes('/runs/43')).length, 0);
});
test('Markdown summary escapes workflow-controlled characters to block injection', () => {
  const formatted = formatSummary([{ run_id: 42, workflow_name: 'a|b\n# header', status: 'NOT_EXECUTED', head_sha: sha }]);
  assert.ok(formatted.includes('a b # header'));
  assert.ok(!formatted.includes('a|b\n#'));
  assert.ok(formatted.includes('not an exhaustive audit'));
  assert.equal(summarize([{ status: 'NOT_EXECUTED' }]).statuses.NOT_EXECUTED, 1);
});
test('GitHub fetch reader only performs no-redirect GET with no token leakage', async () => {
  let seen;
  const reader = createGitHubReader({
    token: 'test-do-not-output', fetchFn: async (url, options) => {
      seen = { url, options };
      return { ok: true, json: async () => ({ total_count: 0, jobs: [] }) };
    },
  });
  const result = await reader('/repos/atto-lang/.github/actions/runs/42/attempts/2/jobs?per_page=100&page=1');
  assert.equal(result.total_count, 0);
  assert.equal(seen.options.method, 'GET');
  assert.equal(seen.options.redirect, 'error');
  assert.equal(seen.options.headers.Authorization, 'Bearer test-do-not-output');
  await assert.rejects(reader('/repos/../bad'), /invalid REST route/);
});
test('HTTP failures report status only and never reveal token or response content', async () => {
  const reader = createGitHubReader({
    token: 'top-secret-value', fetchFn: async () => ({ ok: false, status: 403 }),
  });
  await assert.rejects(reader('/repos/atto-lang/.github/actions/runs/42'), e =>
    /HTTP 403/.test(e.message) && !e.message.includes('top-secret-value'));
});
