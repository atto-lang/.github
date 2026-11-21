import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { classifyJobEvidence } from './ci-zero-step-classifier.mjs';

const job = (id, steps, conclusion = 'success', runner_id = 1001) => ({
  id, name: `job-${id}`, steps, status: 'completed', conclusion, runner_id,
});
const done = (name) => ({ name, status: 'completed', conclusion: 'success' });
const failed = (name) => ({ name, status: 'completed', conclusion: 'failure' });

test('zero-step failure is NOT_EXECUTED, never source failure', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, null, 'failure', 0)] }).status, 'NOT_EXECUTED');
});
test('empty successful job is not a passing test', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [], 'success')] }).status, 'NOT_EXECUTED');
});
test('no job records fail closed', () => {
  assert.equal(classifyJobEvidence({ jobs: [] }).status, 'NOT_EXECUTED');
});
test('a missing runner ID with completed steps is INCOMPLETE evidence, not zero-step', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [done('checkout')], 'success', null)] }).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('missing runner metadata never authorizes a pass', () => {
  const j = job(1, [done('checkout')]);
  delete j.runner_id;
  assert.equal(classifyJobEvidence({ jobs: [j] }).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('runner_id zero with purported executed steps is inconsistent', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [done('cargo test')], 'success', 0)] }).status, 'INCONSISTENT_JOB_EVIDENCE');
});
test('one zero-step job blocks an otherwise green matrix', () => {
  const result = classifyJobEvidence({ jobs: [job(1, [done('test')]), job(2, null, 'failure')] });
  assert.equal(result.status, 'NOT_EXECUTED');
  assert.deepEqual(result.job_ids, [2]);
});
test('a real stepful test failure is a failure', () => {
  const result = classifyJobEvidence({ jobs: [job(1, [done('checkout'), failed('cargo test')], 'failure')] });
  assert.equal(result.status, 'TEST_FAILED');
  assert.equal(result.total_steps, 2);
});
test('success job containing a failed step is inconsistent, not passing', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [done('checkout'), failed('test')])] }).status, 'INCONSISTENT_JOB_EVIDENCE');
});
test('a missing step result blocks pass even if the job claims success', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [done('checkout'), { name: 'test', status: 'in_progress', conclusion: null }])] }).status, 'PENDING_OR_INCONCLUSIVE');
});
test('a cancelled stepful job cannot pass', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [done('checkout')], 'cancelled')] }).status, 'PENDING_OR_INCONCLUSIVE');
});
test('a job with non-completed status and success conclusion is inconclusive', () => {
  const j = job(1, [done('test')]);
  j.status = 'in_progress';
  assert.equal(classifyJobEvidence({ jobs: [j] }).status, 'PENDING_OR_INCONCLUSIVE');
});
test('a job with only skipped steps never passes', () => {
  const j = job(1, [{ name: 'test', status: 'completed', conclusion: 'skipped' }]);
  const result = classifyJobEvidence({ jobs: [j] });
  assert.equal(result.status, 'PENDING_OR_INCONCLUSIVE');
  assert.equal(result.skipped_steps, 1);
});
test('an all-skipped second job cannot be hidden behind a successful first job', () => {
  const jobs = [job(1, [done('real test')]), job(2, [{ name: 'skipped test', status: 'completed', conclusion: 'skipped' }])];
  assert.equal(classifyJobEvidence({ jobs }).status, 'PENDING_OR_INCONCLUSIVE');
});
test('a partial first page cannot report success', () => {
  const input = { total_count: 2, jobs: [job(1, [done('test')])] };
  assert.equal(classifyJobEvidence(input).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('malformed or negative count cannot report success', () => {
  assert.equal(classifyJobEvidence({ total_count: -1, jobs: [job(1, [done('test')])] }).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('duplicate job ID means incomplete data', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [done('test')]), job(1, [done('test')])] }).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('malformed nested job cannot report success', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [done('test')]), null] }).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('fully completed credible metadata means stepful report only, not source proof', () => {
  const result = classifyJobEvidence({ total_count: 1, jobs: [job(1, [done('checkout'), done('test')])] });
  assert.equal(result.status, 'STEPFUL_RUN_REPORTED_SUCCESS');
  assert.equal(result.total_steps, 2);
  assert.match(result.reason, /source and coverage unverified/);
});
test('successful job with optional skips surfaces those skipped steps', () => {
  const result = classifyJobEvidence({ total_count: 1, jobs: [job(1, [done('test'), { status: 'completed', conclusion: 'skipped' }])] });
  assert.equal(result.status, 'STEPFUL_RUN_REPORTED_SUCCESS');
  assert.equal(result.skipped_steps, 1);
});

test('missing REST total_count cannot be green', () => {
  const result = classifyJobEvidence({ jobs: [job(1, [done('cargo test')])] });
  assert.equal(result.status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('caller-supplied complete jobs array is reported as unverified stepful metadata', () => {
  assert.equal(classifyJobEvidence([job(1, [done('test')])]).status, 'STEPFUL_RUN_REPORTED_SUCCESS');
});
test('missing job ID cannot be green', () => {
  const j = job(1, [done('test')]);
  delete j.id;
  assert.equal(classifyJobEvidence({ total_count: 1, jobs: [j] }).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('invalid job ID cannot be green', () => {
  const j = job(1, [done('test')]);
  j.id = -1;
  assert.equal(classifyJobEvidence({ total_count: 1, jobs: [j] }).status, 'INCOMPLETE_JOB_EVIDENCE');
});

const script = fileURLToPath(new URL('./ci-zero-step-classifier.mjs', import.meta.url));
const relativeScript = relative(process.cwd(), script);

function executeWith(input, raw = false) {
  const dir = mkdtempSync(join(tmpdir(), 'atto-zero-step-policy-'));
  const path = join(dir, 'jobs.json');
  try {
    writeFileSync(path, raw ? input : JSON.stringify(input));
    // Critical regression: relative process.argv[1] must execute the CLI,
    // not silently skip the main block and return exit code zero.
    return spawnSync(process.execPath, [relativeScript, path], { cwd: process.cwd(), encoding: 'utf8' });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}
test('actual CLI invoked by relative path produces a verdict, not silent success', () => {
  const output = executeWith({ total_count: 1, jobs: [job(1, [done('cargo test')])] });
  assert.equal(output.status, 0, output.stderr);
  assert.equal(JSON.parse(output.stdout).status, 'STEPFUL_RUN_REPORTED_SUCCESS');
});
test('actual CLI exits 2 for zero-step evidence', () => {
  const output = executeWith({ total_count: 1, jobs: [job(1, null, 'failure', 0)] });
  assert.equal(output.status, 2);
  assert.equal(JSON.parse(output.stdout).status, 'NOT_EXECUTED');
});
test('actual CLI exits 1 on a real stepful failure', () => {
  const output = executeWith({ jobs: [job(1, [failed('cargo test')], 'failure')] });
  assert.equal(output.status, 1);
  assert.equal(JSON.parse(output.stdout).status, 'TEST_FAILED');
});
test('actual CLI exits 2 on truncated data', () => {
  const output = executeWith({ total_count: 2, jobs: [job(1, [done('cargo test')])] });
  assert.equal(output.status, 2);
  assert.equal(JSON.parse(output.stdout).status, 'INCOMPLETE_JOB_EVIDENCE');
});
test('invalid JSON is never a pass', () => {
  const output = executeWith('{broken json}', true);
  assert.equal(output.status, 2);
  assert.match(output.stderr, /INCONCLUSIVE/);
});
