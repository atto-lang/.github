// Source-free, fail-closed GitHub Actions job-evidence classifier.
//
// Usage: node governance/ci-zero-step-classifier.mjs jobs.json
// Input: REST /actions/runs/{id}/jobs (complete page set) or jobs[].
// This does NOT authorize private export, validate source SHA or approve merges.
import fs from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const verdict = (status, reason, details = {}) => ({ status, reason, ...details });
const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);

export function classifyJobEvidence(input) {
  const jobs = Array.isArray(input) ? input : isObject(input) ? input.jobs : undefined;
  if (!Array.isArray(jobs) || jobs.length === 0) {
    return verdict('NOT_EXECUTED', 'no job records', { total_steps: 0 });
  }

  // GitHub paginates job listings. The first 100 jobs are not a full verdict
  // when total_count > jobs.length; never silently green an incomplete page.
  if (isObject(input) && 'total_count' in input) {
    if (!Number.isSafeInteger(input.total_count) ||
        input.total_count !== jobs.length) {
      return verdict('INCOMPLETE_JOB_EVIDENCE', 'job count/pagination mismatch', {
        reported_total: input.total_count, returned_jobs: jobs.length, total_steps: 0,
      });
    }
  }

  let totalSteps = 0;
  let skippedSteps = 0;
  const failed = [];
  const zeroStep = [];
  const missingRunner = [];
  const inconsistent = [];
  let incomplete = false;
  const seenIds = new Set();

  for (const job of jobs) {
    if (!isObject(job)) {
      return verdict('INCOMPLETE_JOB_EVIDENCE', 'invalid job record', { total_steps: totalSteps });
    }
    const identifier = job.id ?? job.name ?? 'unknown';
    if (!Number.isSafeInteger(job.id) || job.id <= 0) {
      return verdict('INCOMPLETE_JOB_EVIDENCE', 'missing or invalid job ID', { total_steps: totalSteps });
    }
    if (seenIds.has(job.id)) {
      return verdict('INCOMPLETE_JOB_EVIDENCE', 'duplicate job id', { job_ids: [job.id], total_steps: totalSteps });
    }
    seenIds.add(job.id);

    // No step records means no verified execution. Missing runner metadata
    // alongside completed steps is INCOMPLETE evidence, not proof of
    // an actual zero-step admission failure. An explicit runner_id=0 with
    // populated steps is contradictory evidence.
    if (!Array.isArray(job.steps) || job.steps.length === 0) {
      zeroStep.push(identifier);
      continue;
    }
    if (!Number.isSafeInteger(job.runner_id)) {
      missingRunner.push(identifier);
      continue;
    }
    if (job.runner_id <= 0) {
      inconsistent.push(identifier);
      continue;
    }

    let jobFailedStep = false;
    let jobIncompleteStep = false;
    let jobExecutedSteps = 0;
    for (const step of job.steps) {
      if (!isObject(step)) {
        jobIncompleteStep = true;
        continue;
      }
      if (step.conclusion === 'skipped') {
        skippedSteps++;
        continue;
      }
      if (step.status === 'completed' && step.conclusion === 'success') {
        totalSteps++;
        jobExecutedSteps++;
      } else if (step.status === 'completed' &&
                 (step.conclusion === 'failure' || step.conclusion === 'timed_out')) {
        totalSteps++;
        jobExecutedSteps++;
        jobFailedStep = true;
      } else {
        jobIncompleteStep = true;
      }
    }

    if (job.conclusion === 'success' && jobFailedStep) {
      inconsistent.push(identifier);
    } else if (job.conclusion === 'failure' || job.conclusion === 'timed_out') {
      failed.push(identifier);
    } else if (job.conclusion !== 'success' || jobIncompleteStep ||
               jobExecutedSteps === 0 || job.status !== 'completed') {
      incomplete = true;
    }
  }

  // A missing runner anywhere in a required jobs set blocks a complete pass.
  if (zeroStep.length) {
    return verdict('NOT_EXECUTED', 'one or more jobs have no admitted runner or executed steps', {
      job_ids: zeroStep, total_steps: totalSteps, skipped_steps: skippedSteps,
    });
  }
  if (missingRunner.length) {
    return verdict('INCOMPLETE_JOB_EVIDENCE', 'job steps exist but runner metadata is missing', {
      job_ids: missingRunner, total_steps: totalSteps, skipped_steps: skippedSteps,
    });
  }
  if (inconsistent.length) {
    return verdict('INCONSISTENT_JOB_EVIDENCE', 'job result contradicts executed steps', {
      job_ids: inconsistent, total_steps: totalSteps, skipped_steps: skippedSteps,
    });
  }
  if (failed.length) {
    return verdict('TEST_FAILED', 'stepful job(s) failed', {
      job_ids: failed, total_steps: totalSteps, skipped_steps: skippedSteps,
    });
  }
  if (incomplete || totalSteps === 0) {
    return verdict('PENDING_OR_INCONCLUSIVE', 'not every job/step has sufficient completed evidence', {
      total_steps: totalSteps, skipped_steps: skippedSteps,
    });
  }

  // The GitHub REST jobs response supplies total_count. Without it, an
  // object may be only the first page; a standalone array is explicitly a
  // caller-supplied complete snapshot and still not a source attestation.
  if (!Array.isArray(input) && !('total_count' in input)) {
    return verdict('INCOMPLETE_JOB_EVIDENCE', 'missing REST pagination total_count', {
      total_steps: totalSteps, skipped_steps: skippedSteps,
    });
  }

  // A successful job report is not proof of material tests or source revision.
  // The run's required-check list, exact SHA, and individual test logs must
  // be independently authenticated before making a TESTS_PASSED_SCOPE claim.
  return verdict('STEPFUL_RUN_REPORTED_SUCCESS', 'reported successful steps only; source and coverage unverified', {
    total_steps: totalSteps, skipped_steps: skippedSteps,
  });
}

const invokedPath = process.argv[1];
if (invokedPath && import.meta.url === pathToFileURL(resolve(invokedPath)).href) {
  try {
    if (process.argv.length !== 3) throw new Error('usage: node ci-zero-step-classifier.mjs <jobs.json>');
    const data = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
    const result = classifyJobEvidence(data);
    console.log(JSON.stringify(result));
    process.exitCode = result.status === 'STEPFUL_RUN_REPORTED_SUCCESS' ? 0 :
      result.status === 'TEST_FAILED' ? 1 : 2;
  } catch (error) {
    console.error('INCONCLUSIVE: ' + error.message);
    process.exitCode = 2;
  }
}
