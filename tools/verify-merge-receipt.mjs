#!/usr/bin/env node
// An offline untrusted snapshot checker, NOT GitHub authentication or release approval.
import { openSync, fstatSync, readSync, closeSync, constants as FS } from 'node:fs';
import { TextDecoder } from 'node:util';
import { pathToFileURL } from 'node:url';

const SHA = /^[a-f0-9]{40}$/;
const ISO = /^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?Z$/;
const CTX = /^[\w./ :()-]{1,180}$/;
const obj = x => x !== null && typeof x === 'object' && !Array.isArray(x);
const str = x => typeof x === 'string' && x.trim().length > 0;
// GitHub user logins are ASCII and case-insensitive. Reject aliases or
// malformed identities before counting an independent review.
const LOGIN = /^[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?$/;
const login = x => typeof x === 'string' && LOGIN.test(x);
const loginKey = x => x.toLowerCase();
const sha = x => typeof x === 'string' && SHA.test(x);
const positive = x => Number.isSafeInteger(x) && x > 0;
const when = x => typeof x === 'string' && ISO.test(x) && !Number.isNaN(Date.parse(x)) && new Date(x).toISOString().slice(0,19) === x.slice(0,19);

export function verifyMergeReceipt(receipt) {
  const problems = [];
  const deny = (code, condition) => { if (!condition) problems.push(code); };
  if (!obj(receipt)) return {status:'BLOCKED',reasons:['RECEIPT_INVALID']};
  const {pr,checkout,workflow,reviews,protection,merge} = receipt;
  deny('SCHEMA_INVALID',receipt.schema === 'atto.merge-receipt/v1');
  deny('PR_INVALID',obj(pr) && str(pr.repository) && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(pr.repository) && positive(pr.number) && login(pr.author) && sha(pr.head_sha) && sha(pr.head_tree_sha) && sha(pr.base_sha));
  deny('CHECKOUT_INVALID',obj(checkout) && sha(checkout.sha) && sha(checkout.tree_sha) && checkout.mode === 'physical-original-head' && checkout.dirty === false && checkout.synthetic_merge === false && checkout.sha === pr?.head_sha && checkout.tree_sha === pr?.head_tree_sha);
  deny('PROTECTION_UNENFORCED',obj(protection) && protection.main_protected === true && protection.requires_pr === true && Number.isSafeInteger(protection.required_approvals) && protection.required_approvals >= 1 && protection.codeowners_required === true && protection.dismiss_stale === true && protection.include_admins === true && protection.bypass_actors === 0);
  deny('WORKFLOW_INVALID',obj(workflow) && sha(workflow.head_sha) && sha(workflow.head_tree_sha) && workflow.head_sha === pr?.head_sha && workflow.head_tree_sha === pr?.head_tree_sha && positive(workflow.run_id) && positive(workflow.attempt) && workflow.event === 'pull_request' && workflow.status === 'completed' && workflow.conclusion === 'success' && Array.isArray(workflow.jobs) && workflow.jobs.length > 0);
  const unique = new Set();
  let tested = false;
  for (const [i,job] of (Array.isArray(workflow?.jobs) ? workflow.jobs : []).entries()) {
    if (!obj(job)) {problems.push(`JOB_${i}_INVALID`);continue;}
    deny(`JOB_${i}_IDENTITY`,positive(job.id) && str(job.name) && CTX.test(job.name) && !unique.has(job.name) && job.run_id === workflow.run_id && job.attempt === workflow.attempt && job.head_sha === pr?.head_sha && job.head_tree_sha === pr?.head_tree_sha);
    unique.add(job.name);
    deny(`JOB_${i}_UNEXECUTED`,positive(job.runner_id) && job.status === 'completed' && job.conclusion === 'success' && Array.isArray(job.steps) && job.steps.length > 0);
    tested ||= Array.isArray(job.steps) && job.steps.some(s => obj(s) && s.kind === 'test' && s.status === 'completed' && s.conclusion === 'success');
    for (const [k,step] of (Array.isArray(job.steps) ? job.steps : []).entries()) {
      deny(`JOB_${i}_STEP_${k}`,obj(step) && str(step.name) && step.status === 'completed' && (step.conclusion === 'success' || (step.kind === 'cleanup' && step.conclusion === 'skipped')));
    }
  }
  deny('NO_EXECUTED_TEST',tested);
  deny('REVIEWS_INVALID',Array.isArray(reviews) && reviews.length > 0);
  const valid = (Array.isArray(reviews) ? reviews : []).filter(r => obj(r) && r.state === 'APPROVED' && login(r.author) && login(pr?.author) && loginKey(r.author) !== loginKey(pr.author) && r.commit_sha === pr?.head_sha && when(r.submitted_at) && r.dismissed !== true && r.superseded !== true);
  deny('INDEPENDENT_APPROVAL_MISSING',new Set(valid.map(r=>loginKey(r.author))).size >= (protection?.required_approvals ?? Infinity));
  deny('CONFLICTING_REVIEW',Array.isArray(reviews) && !reviews.some(r => obj(r) && r.state === 'CHANGES_REQUESTED' && r.dismissed !== true && r.superseded !== true));
  deny('MERGE_INVALID',obj(merge) && merge.method === 'squash' && sha(merge.commit_sha) && sha(merge.main_sha) && sha(merge.main_tree_sha) && merge.pr_head_sha === pr?.head_sha && merge.github_merged === true && merge.on_main_ancestry === true && merge.main_protected_at_merge === true && when(merge.merged_at));
  deny('REVIEW_AFTER_MERGE',!when(merge?.merged_at) || valid.every(r => Date.parse(r.submitted_at) <= Date.parse(merge.merged_at)));
  deny('RUN_AFTER_MERGE',!when(merge?.merged_at) || !when(workflow?.completed_at) || Date.parse(workflow.completed_at) <= Date.parse(merge.merged_at));
  deny('RUN_COMPLETED_AT_INVALID',when(workflow?.completed_at));
  return {status:problems.length ? 'BLOCKED' : 'SNAPSHOT_VALID_RECHECK_LIVE',reasons:[...new Set(problems)]};
}

// Treat the CLI input path as untrusted. This is source-free, offline snapshot
// validation, not an authenticated merge or release receipt.
const MAX_SNAPSHOT_BYTES = 256 * 1024;
function readSnapshot(path) {
  if (typeof path !== 'string' || path.length === 0 ||
      typeof FS.O_NOFOLLOW !== 'number' || typeof FS.O_NONBLOCK !== 'number')
    throw Error('UNSAFE_SNAPSHOT_PATH');
  const fd = openSync(path, FS.O_RDONLY | FS.O_NOFOLLOW | FS.O_NONBLOCK);
  try {
    const before = fstatSync(fd, {bigint:true});
    if (!before.isFile() || before.nlink !== 1n ||
        before.size < 1n || before.size > BigInt(MAX_SNAPSHOT_BYTES))
      throw Error('UNSAFE_SNAPSHOT_FILE');
    const bytes = Buffer.alloc(Number(before.size));
    let offset = 0;
    while (offset < bytes.length) {
      const n = readSync(fd, bytes, offset, bytes.length - offset, offset);
      if (n <= 0) throw Error('INCOMPLETE_SNAPSHOT_READ');
      offset += n;
    }
    const after = fstatSync(fd, {bigint:true});
    if (after.dev !== before.dev || after.ino !== before.ino ||
        after.size !== before.size || after.nlink !== 1n ||
        after.mtimeNs !== before.mtimeNs || after.ctimeNs !== before.ctimeNs)
      throw Error('CHANGED_SNAPSHOT_FILE');
    return new TextDecoder('utf-8', {fatal:true}).decode(bytes);
  } finally { closeSync(fd); }
}

export function main(argv=process.argv.slice(2),write=console.log) {
  if(argv.length!==1){write(JSON.stringify({status:'BLOCKED',reasons:['USAGE']}));return 2;}
  try {
    const result=verifyMergeReceipt(JSON.parse(readSnapshot(argv[0])));
    write(JSON.stringify(result));
    return result.status==='BLOCKED'?1:3; // NEVER 0: live revalidation is mandatory.
  } catch {
    write(JSON.stringify({status:'BLOCKED',reasons:['READ_OR_PARSE_FAILED']}));
    return 2;
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) process.exitCode=main();
