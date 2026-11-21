import test from 'node:test';
import assert from 'node:assert/strict';
import { classifyJobEvidence } from './ci-zero-step-classifier.mjs';

const job = (id, steps, conclusion = 'success', runner_id = 1001) => ({
  id, name: `job-${id}`, steps, conclusion, runner_id,
});
const done = (name) => ({ name, status: 'completed', conclusion: 'success' });

test('null steps means runner never executed, not compiler failure', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, null, 'failure', 0)] }).status, 'NOT_EXECUTED');
});
test('empty steps means runner never executed even when marked success', () => {
  assert.equal(classifyJobEvidence({ jobs: [job(1, [], 'success')] }).status, 'NOT_EXECUTED');
});
test('no jobs fail closed', () => {
  assert.equal(classifyJobEvidence({ jobs: [] }).status, 'NOT_EXECUTED');
});
test('mixed pass and zero-step is not a green build', () => {
  const r = classifyJobEvidence({ jobs: [job(1, [done('cargo test')]), job(2, null, 'failure')] });
  assert.equal(r.status, 'NOT_EXECUTED');
  assert.deepEqual(r.job_ids, [2]);
});
test('executed test failure is genuine rather than runner admission', () => {
  const r = classifyJobEvidence({ jobs: [job(1, [done('Checkout'), { name: 'cargo test', status: 'completed', conclusion: 'failure' }], 'failure')] });
  assert.equal(r.status, 'TEST_FAILED');
  assert.equal(r.total_steps, 2);
});
test('pending or cancelled job does not pass', () => {
  const r = classifyJobEvidence({ jobs: [job(1, [done('Checkout')], 'cancelled')] });
  assert.equal(r.status, 'PENDING_OR_INCONCLUSIVE');
});
test('actual success indicates only stepful result, not export authorization', () => {
  const r = classifyJobEvidence({ jobs: [job(1, [done('Checkout'), done('cargo test')])] });
  assert.equal(r.status, 'STEPFUL_RUN_REPORTED_SUCCESS');
  assert.equal(r.total_steps, 2);
  assert.notEqual(r.status, 'PRIVATE_EXPORT_APPROVED');
});
