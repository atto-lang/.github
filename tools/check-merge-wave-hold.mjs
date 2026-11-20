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
// Independent defense in depth: immutable historical identities. A caller may
// use checkSnapshot() directly without passing through readSnapshot()'s pinned
// blob digest; a plausible replacement batch must still be refused.
const PINNED_ROWS = Object.freeze([
  Object.freeze(["ores-truffle-oreslang/oreslang-source.forge",198,"51e2369d5e3d9f9fc9368c20a982e54e7e6c2ff1","7902eb2e3610babe1004a3e752b12079737992e6",42294518377,128156170772,0,"not_executed_sample"]),
  Object.freeze(["ores-truffle-oreslang/oreslang-source.js",215,"5a580d7ed69c6c6c02afdd6683a199246b7cf421","b437edff0ab136d83f9aedae652a5f368b4cc5ee",42294569932,128156348036,0,"not_executed_sample"]),
  Object.freeze(["ores-truffle-oreslang/oreslang-source.hdl",27,"a12245f3f1ce16bfe354b33f8ef806c5f43e733e","05b2a5899ca2b4f38ee27827a509dbfd1d19a3ef",42294592084,128156423214,0,"not_executed_sample"]),
  Object.freeze(["ores-truffle-oreslang/oreslang-source.rs",26,"6968d620957a6eb069df71440463beb43b9321c2","e516b09622f6d1b9d61330aff88e6c86bb6d97f1",42294604675,128156465081,0,"not_executed_sample"]),
  Object.freeze(["ores-truffle-oreslang/oreslang-source.net",374,"d570052ed6ec2a5f3ce54483733dcb7664a01181","5d8246bca843309ee3f66538f1385fefb5690858",42294558773,128156310276,0,"not_executed_sample"]),
  Object.freeze(["ores-truffle-oreslang/sublime-text-plugins",130,"83980c85999f3c1631d7629cf9069d96c907fa02","6c54e1042596b3274c0d921fcc41636da5f82f76",42297029167,128164835552,15,"success_stepful"]),
  Object.freeze(["ores-truffle-oreslang/sublime-text-plugins",131,"82bd6bb6376e84739d17e49324e422abc68ebf04","8320a29a2fbaccc5d5f296a1a1712000c0a6d229",42297059629,128164940257,15,"success_stepful"]),
  Object.freeze(["ores-truffle-oreslang/oreslang-common-source",243,"dab5f37f1e49ebf3efd2ab2793f8d2a15d1ef2a4","3c39e798f2b6d4f8d4f0760821329321c4433355",42297083612,128165025185,0,"not_executed_sample"]),
  Object.freeze(["ores-truffle-oreslang/oreslang-ssr",36,"d8c0dceeb9ce1a80d3bb17864914793e2fd0f5c4","eeb0535f34af15b0773585d42259595b67eec996",42300219068,128175746624,0,"not_executed_sample"]),
  Object.freeze(["ores-truffle-oreslang/oreslang-linked-queue",23,"6c4e7b713d22da20a69ba157983dca7ec991f96e","09ea6364794cc050639b669a783a169875c5f50a",42297102590,128165095940,0,"not_executed_sample"]),
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
    const pinned = PINNED_ROWS[i];
    if (!pinned || repo !== pinned[0] || pr.pr !== pinned[1] ||
        pr.pr_head !== pinned[2] || pr.landed_sha !== pinned[3] ||
        ci.run_id !== pinned[4] || ci.job_id !== pinned[5] ||
        ci.executed_steps !== pinned[6] || ci.status !== pinned[7]) {
      fail('historical PR/commit/run/job identity differs from pinned evidence');
    }
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
