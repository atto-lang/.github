// Source-free diagnostic classifier for GitHub Actions job snapshots.
//
// Usage: node governance/ci-zero-step-classifier.mjs jobs.json
// Receives GitHub REST /actions/runs/{run_id}/jobs JSON, or a jobs[] array.
// Does NOT authorize private export, attest provenance, or grant merge approval.
import fs from 'node:fs';

export function classifyJobEvidence(input) {
  const jobs = Array.isArray(input) ? input : input?.jobs;
  if (!Array.isArray(jobs) || jobs.length === 0) {
    return { status: 'NOT_EXECUTED', reason: 'no job records', total_steps: 0 };
  }

  let totalSteps = 0;
  const failed = [];
  const zeroStep = [];
  let incomplete = false;
  for (const job of jobs) {
    const identifier = job?.id ?? job?.name ?? 'unknown';
    if (!Array.isArray(job?.steps) || job.steps.length === 0 || job.runner_id === 0) {
      zeroStep.push(identifier);
      continue;
    }
    const actual = job.steps.filter((s) => s?.status !== 'pending' && s?.conclusion !== 'skipped');
    totalSteps += actual.length;
    if (job.conclusion === 'failure' || job.conclusion === 'timed_out') {
      failed.push(identifier);
    } else if (job.conclusion !== 'success') {
      incomplete = true;
    }
  }

  // Mixed work is never a complete pass when any required job did not execute.
  if (zeroStep.length) {
    return { status: 'NOT_EXECUTED', reason: 'at least one job has no runner/steps', job_ids: zeroStep, total_steps: totalSteps };
  }
  if (failed.length) {
    return { status: 'TEST_FAILED', reason: 'stepful job(s) failed', job_ids: failed, total_steps: totalSteps };
  }
  if (incomplete) {
    return { status: 'PENDING_OR_INCONCLUSIVE', reason: 'not every job concluded success', total_steps: totalSteps };
  }
  // Execution is evidenced, but the tool intentionally never claims test
  // correctness or exact source SHA; an authenticated workflow is needed.
  return { status: 'STEPFUL_RUN_REPORTED_SUCCESS', total_steps: totalSteps };
}

if (process.argv[1] && import.meta.url === new URL(`file://${process.argv[1]}`).href) {
  try {
    if (process.argv.length !== 3) throw new Error('usage: node ci-zero-step-classifier.mjs <jobs.json>');
    const data = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
    const verdict = classifyJobEvidence(data);
    console.log(JSON.stringify(verdict));
    process.exitCode = verdict.status === 'STEPFUL_RUN_REPORTED_SUCCESS' ? 0 : verdict.status === 'TEST_FAILED' ? 1 : 2;
  } catch (error) {
    console.error('INCONCLUSIVE: ' + error.message);
    process.exitCode = 2;
  }
}
