#!/usr/bin/env node
// Historical source-free audit integrity check. NEVER a merge/release authorization.
import { readFileSync, lstatSync, constants, openSync, closeSync, fstatSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(fileURLToPath(new URL('..', import.meta.url)));
const DEFAULT_FILE = resolve(ROOT, 'audits/merge-waves/2026-11-06-retroactive.json');
// This hash is the exact Git blob ID independently read back from GitHub for
// the ten-PR evidence snapshot. The structural parser is *not* an authenticity
// verifier; only the source-pinned historical file is accepted by the CLI.
// Do not update the blob anchor when editing history without independent review.
const PINNED_SNAPSHOT_GIT_BLOB_SHA = '8ba3658779c6a836d24f2321451be5474ecbbc5e';

const EXACT_KEYS = Object.freeze({
  root: ['schema', 'snapshot_date_utc', 'source', 'classification', 'state', 'merge_count',
    'repository_count', 'approved_merge_count', 'protected_main_repository_count',
    'sampled_success_stepful_merge_count', 'sampled_zero_step_failure_merge_count',
    'merges', 'release_gates', 'notes'],
  pr: ['repo', 'pr', 'pr_head', 'landed_sha', 'main_contains_land_sha',
    'main_compare_behind_by', 'submitted_review_count', 'main_protected',
    'post_merge_ci_sample', 'release_certification'],
  ci: ['run_id', 'job_id', 'executed_steps', 'status'],
});
const GATES = Object.freeze([
  'live_required_main_protection', 'independent_review_for_release',
  'exact_landed_commit_source_owned_ci', 'source_guest_tests_where_applicable',
  'admin_runner_admission_resolution', 'no_unapproved_private_source_export',
]);
const sha = s => typeof s === 'string' && /^[0-9a-f]{40}$/.test(s) && /[1-9a-f]/.test(s);
const natural = n => Number.isSafeInteger(n) && n > 0;
const nonnegative = n => Number.isSafeInteger(n) && n >= 0;
const fail = message => { throw new Error(message); };
const keys = (obj, expected, context) => {
  if (obj === null || typeof obj !== 'object' || Array.isArray(obj) ||
      Object.keys(obj).length !== expected.length ||
      expected.some(key => !Object.hasOwn(obj, key))) fail(`invalid ${context} fields`);
};

export function checkSnapshot(snapshot) {
  keys(snapshot, EXACT_KEYS.root, 'root');
  if (snapshot.schema !== 'atto.merge-wave-retroactive-snapshot.v1' ||
      snapshot.snapshot_date_utc !== '2026-11-06' ||
      snapshot.source !== 'authenticated read-only GitHub PR/review/branch/compare/Actions API observations' ||
      snapshot.classification !== 'historical_snapshot_not_live_release_authorization' ||
      snapshot.state !== 'BLOCKED') fail('snapshot cannot certify a release');
  if (snapshot.merge_count !== 10 || snapshot.repository_count !== 9 ||
      snapshot.approved_merge_count !== 0 || snapshot.protected_main_repository_count !== 0 ||
      snapshot.sampled_success_stepful_merge_count !== 2 ||
      snapshot.sampled_zero_step_failure_merge_count !== 8 ||
      !Array.isArray(snapshot.merges) || snapshot.merges.length !== 10) {
    fail('historical counts do not match expected blocked audit');
  }
  if (!Array.isArray(snapshot.release_gates) ||
      snapshot.release_gates.length !== GATES.length ||
      GATES.some((gate, i) => snapshot.release_gates[i] !== gate) ||
      !Array.isArray(snapshot.notes) || snapshot.notes.length < 4 ||
      snapshot.notes.some(s => typeof s !== 'string' || s.length > 500)) {
    fail('missing or modified release hold gates');
  }
  const identities = new Set(), commits = new Set(), jobs = new Set(), runs = new Set();
  let passed = 0, notExecuted = 0;
  for (const [i, pr] of snapshot.merges.entries()) {
    keys(pr, EXACT_KEYS.pr, 'PR');
    const {repo, post_merge_ci_sample: ci} = pr;
    keys(ci, EXACT_KEYS.ci, 'CI');
    if (typeof repo !== 'string' || !/^ores-truffle-oreslang\/[A-Za-z0-9._-]{1,100}$/.test(repo) ||
        !natural(pr.pr) || !sha(pr.pr_head) || !sha(pr.landed_sha) ||
        pr.pr_head === pr.landed_sha || pr.main_contains_land_sha !== true ||
        pr.main_compare_behind_by !== 0 || pr.submitted_review_count !== 0 ||
        pr.main_protected !== false || pr.release_certification !== 'BLOCKED' ||
        !natural(ci.run_id) || !natural(ci.job_id) || !nonnegative(ci.executed_steps)) {
      fail(`unverified or altered PR record ${i + 1}`);
    }
    const id = `${repo}#${pr.pr}`;
    if (identities.has(id) || commits.has(`${repo}@${pr.landed_sha}`) ||
        jobs.has(`${repo}:${ci.job_id}`) || runs.has(`${repo}:${ci.run_id}`)) {
      fail('duplicate PR, landed SHA or workflow identity');
    }
    identities.add(id); commits.add(`${repo}@${pr.landed_sha}`);
    jobs.add(`${repo}:${ci.job_id}`); runs.add(`${repo}:${ci.run_id}`);
    if (ci.status === 'success_stepful' && ci.executed_steps > 0) passed++;
    else if (ci.status === 'not_executed_sample' && ci.executed_steps === 0) notExecuted++;
    else fail('workflow step/status mismatch or forged status');
  }
  if (new Set(snapshot.merges.map(v => v.repo)).size !== 9 ||
      new Set(snapshot.merges.slice(0, 5).map(v => v.repo)).size < 3 ||
      new Set(snapshot.merges.slice(5).map(v => v.repo)).size < 3 ||
      passed !== 2 || notExecuted !== 8) fail('merge wave quorum or sampled CI mismatch');
  return Object.freeze({state:'BLOCKED', merges:10, repositories:9, executedSamples:2,
    notExecutedSamples:8, releaseAuthorized:false});
}

export function parseCanonicalSnapshot(raw) {
  if (typeof raw !== 'string' || Buffer.byteLength(raw, 'utf8') > 65536 || !raw.endsWith('\n')) {
    fail('snapshot too large, malformed or noncanonical');
  }
  let parsed;
  try { parsed = JSON.parse(raw); } catch { fail('invalid JSON'); }
  // Our checked-in audit uses this exact serialisation. This also rejects
  // duplicate JSON keys (which JSON.parse otherwise silently normalizes).
  if (JSON.stringify(parsed, null, 2) + '\n' !== raw) fail('noncanonical JSON or duplicate authority');
  return checkSnapshot(parsed);
}

export function readSnapshot(path) {
  const stats = lstatSync(path);
  if (!stats.isFile() || stats.isSymbolicLink() || stats.nlink !== 1 || stats.size > 65536) {
    fail('unsafe source snapshot filesystem entry');
  }
  const fd = openSync(path, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0));
  try {
    const opened = fstatSync(fd);
    if (!opened.isFile() || opened.nlink !== 1 || opened.size > 65536 ||
        opened.ino !== stats.ino || opened.dev !== stats.dev) fail('snapshot changed during open');
    const raw = readFileSync(fd, {encoding:'utf8'});
    const byteCount = Buffer.byteLength(raw, 'utf8');
    // Authenticity of the historical snapshot is pinned to the original
    // Git object, not to user-supplied PR fields or self-reported counts.
    const actualBlob = createHash('sha1').update(`blob ${byteCount}\0`)
      .update(raw, 'utf8').digest('hex');
    if (actualBlob !== PINNED_SNAPSHOT_GIT_BLOB_SHA) {
      fail('snapshot does not match pinned historical Git blob');
    }
    return parseCanonicalSnapshot(raw);
  } finally { closeSync(fd); }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length > 1 || (args.length === 1 && args[0] !== '--release')) {
    console.error('INVALID_INPUT: usage check-merge-wave-hold.mjs [--release]');
    process.exitCode = 2;
  } else {
    try {
      // Deliberately use the one exact pinned repository audit document.
      const result = readSnapshot(DEFAULT_FILE);
      console.log(`AUDIT_INTEGRITY_ONLY: HOLD_ACTIVE ${result.merges} merges; ${result.notExecutedSamples} not executed; RELEASE_BLOCKED`);
      if (args.includes('--release')) process.exitCode = 1; // NEVER authorizes release.
    } catch (err) {
      console.error(`INVALID_AUDIT: ${err.message}`);
      process.exitCode = 2;
    }
  }
}
