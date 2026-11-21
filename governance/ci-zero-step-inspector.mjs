// Read-only automatic GitHub Actions job inspector. No source export, tokens
// in logs, status writes, workflow dispatch, or merge authority.
import fs from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { classifyJobEvidence } from './ci-zero-step-classifier.mjs';

const SHA40 = /^[0-9a-f]{40}$/i;
const REPO = /^[A-Za-z0-9][A-Za-z0-9-]{0,38}\/[A-Za-z0-9_.-]{1,100}$/;
const MAX_PAGES = 20;
export const MONITOR_NAME = 'Zero-step CI admission watcher';
const validRepo = value => REPO.test(value ?? '') && !value.includes('..');

export async function inspectRun({ repository, runId, expectedSha, getJson }) {
  if (!validRepo(repository) || !Number.isSafeInteger(runId) || runId <= 0 ||
      (expectedSha !== undefined && !SHA40.test(expectedSha)) ||
      typeof getJson !== 'function') throw new Error('invalid repository/run/head or reader');
  const prefix = '/repos/' + repository + '/actions/runs/';
  const run = await getJson(prefix + runId);
  if (run?.id !== runId || !SHA40.test(run?.head_sha) ||
      !Number.isSafeInteger(run?.run_attempt) || run.run_attempt <= 0 ||
      run.repository?.full_name?.toLowerCase() !== repository.toLowerCase()) {
    throw new Error('incomplete or mismatched immutable workflow run metadata');
  }
  const base = {
    repository, run_id: runId, attempt: run.run_attempt,
    head_sha: run.head_sha, workflow_name: run.name ?? '',
    run_url: run.html_url ?? '', run_status: run.status, run_conclusion: run.conclusion,
  };
  if (expectedSha && expectedSha.toLowerCase() !== run.head_sha.toLowerCase()) {
    return { ...base, status: 'STALE_HEAD', reason: 'run does not match the expected head' };
  }
  if (run.status !== 'completed') {
    return { ...base, status: 'PENDING_OR_INCONCLUSIVE', reason: 'run has not completed' };
  }
  const route = prefix + runId + '/attempts/' + run.run_attempt + '/jobs';
  const jobs = [];
  let count;
  for (let page = 1; page <= MAX_PAGES; page++) {
    const response = await getJson(route + '?per_page=100&page=' + page);
    if (!Number.isSafeInteger(response?.total_count) || response.total_count < 0 ||
        !Array.isArray(response.jobs)) throw new Error('missing REST job-page metadata');
    if (count === undefined) count = response.total_count;
    if (response.total_count !== count || count > 100 * MAX_PAGES) {
      throw new Error('job pagination changed or exceeds bounded scan');
    }
    const expectedLength = Math.min(100, Math.max(0, count - jobs.length));
    if (response.jobs.length !== expectedLength) throw new Error('incomplete REST job page');
    jobs.push(...response.jobs);
    if (jobs.length === count) break;
    if (page === MAX_PAGES) throw new Error('REST job pagination exceeds safe bound');
  }
  const verdict = classifyJobEvidence({ total_count: count, jobs });
  let status = verdict.status;
  let reason = verdict.reason;
  if (status === 'STEPFUL_RUN_REPORTED_SUCCESS' && run.conclusion !== 'success') {
    status = 'INCONSISTENT_JOB_EVIDENCE';
    reason = 'successful jobs contradict overall run conclusion';
  } else if (status === 'TEST_FAILED' && run.conclusion === 'success') {
    status = 'INCONSISTENT_JOB_EVIDENCE';
    reason = 'failed jobs contradict overall run conclusion';
  }
  return { ...base, ...verdict, status, reason, reported_job_count: count };
}

export async function inspectRecent({ repository, limit = 20, getJson }) {
  if (!validRepo(repository) || !Number.isSafeInteger(limit) ||
      limit < 1 || limit > 30 || typeof getJson !== 'function') {
    throw new Error('invalid local repository/scan limit');
  }
  const response = await getJson('/repos/' + repository + '/actions/runs?per_page=' + limit);
  if (!Array.isArray(response?.workflow_runs)) throw new Error('missing runs listing');
  const runs = response.workflow_runs.slice(0, limit)
    .filter(run => run?.name !== MONITOR_NAME &&
      Number.isSafeInteger(run?.id) && run.id > 0);
  const results = [];
  for (const run of runs) {
    try {
      results.push(await inspectRun({ repository, runId: run.id, getJson }));
    } catch (error) {
      results.push({ repository, run_id: run.id, status: 'INCOMPLETE_JOB_EVIDENCE',
        reason: 'REST metadata unavailable: ' + error.message });
    }
  }
  return { repository, limit, exhaustive: false, results };
}

export function summarize(inspected) {
  const results = Array.isArray(inspected) ? inspected : inspected?.results;
  if (!Array.isArray(results)) throw new Error('missing inspected results');
  const statuses = {};
  for (const row of results) statuses[row.status] = (statuses[row.status] ?? 0) + 1;
  return { checked: results.length, statuses, results };
}

const safeCell = value => String(value ?? '').replace(/[\r\n|<>]/g, ' ').slice(0, 140);
export function formatSummary(inspected) {
  const data = summarize(inspected);
  const lines = [
    '## AttoLang zero-step CI admission watch', '',
    'This read-only job is not a private-source export approval or merge gate.', '',
    '| Run ID | Workflow | Head | Diagnosis |',
    '| --- | --- | --- | --- |',
  ];
  for (const row of data.results) {
    lines.push('| ' + safeCell(row.run_id) + ' | ' + safeCell(row.workflow_name) + ' | ' +
      safeCell(row.head_sha) + ' | ' + safeCell(row.status) + ' |');
  }
  lines.push('', 'Checked ' + data.checked + ' recent runs; this is not an exhaustive audit.',
    'NOT_EXECUTED requires runner-admission investigation; it does not prove minutes were exhausted.',
    'Probe alternate runners source-free; require independent authorization before mirroring private code.', '');
  return lines.join('\n');
}

export function createGitHubReader({ token, apiUrl = 'https://api.github.com', fetchFn = fetch }) {
  const base = new URL(apiUrl);
  // Never allow an attacker-controlled HTTPS origin to receive GITHUB_TOKEN.
  // GitHub.com only: enterprise support requires a separate reviewed trust
  // policy and explicit hostname pinning rather than arbitrary env override.
  if (base.protocol !== 'https:' || base.hostname !== 'api.github.com' ||
      base.port || base.username || base.password || base.search ||
      base.hash || base.pathname !== '/') throw new Error('GitHub API origin must be https://api.github.com/');
  return async route => {
    if (!route.startsWith('/repos/') || route.includes('..') ||
        route.includes('#') || route.includes('@') || route.includes('\\')) {
      throw new Error('invalid REST route');
    }
    const response = await fetchFn(base.origin + route, {
      method: 'GET', redirect: 'error',
      headers: {
        Accept: 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
        ...(token ? { Authorization: 'Bearer ' + token } : {}),
      },
    });
    if (!response.ok) throw new Error('GitHub REST returned HTTP ' + response.status);
    return response.json();
  };
}

const invoked = process.argv[1];
if (invoked && import.meta.url === pathToFileURL(resolve(invoked)).href) {
  try {
    const mode = process.argv[2] ?? '--recent';
    const val = process.argv[3];
    if (process.argv.length > 4 || !['--recent', '--run-id'].includes(mode)) {
      throw new Error('usage: ci-zero-step-inspector.mjs [--recent [1-30] | --run-id <run ID>]');
    }
    const getJson = createGitHubReader({
      token: process.env.GITHUB_TOKEN, apiUrl: process.env.GITHUB_API_URL ?? 'https://api.github.com',
    });
    const repository = process.env.GITHUB_REPOSITORY;
    const inspected = mode === '--run-id'
      ? [await inspectRun({ repository, runId: Number(val), getJson,
          expectedSha: process.env.EXPECTED_HEAD_SHA || undefined })]
      : await inspectRecent({ repository, limit: val === undefined ? 20 : Number(val), getJson });
    const summary = summarize(inspected);
    console.log(JSON.stringify(summary));
    if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, formatSummary(inspected));
    if (summary.statuses.NOT_EXECUTED) {
      console.log('::warning::Actions NOT_EXECUTED: investigate admission and capacity. Private-source copying requires separate approval.');
    }
    // Diagnosing genuine failed runs must not create a false CI failure.
    process.exitCode = summary.statuses.INCOMPLETE_JOB_EVIDENCE ||
      summary.statuses.INCONSISTENT_JOB_EVIDENCE ? 2 : 0;
  } catch (error) {
    console.error('INCONCLUSIVE: ' + error.message);
    process.exitCode = 2;
  }
}
