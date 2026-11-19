// Source-free, read-only diagnostics. Input is untrusted: this does not certify a release.
import { lstatSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SHA = /^[a-f0-9]{40}$/;
const SOURCE_REPOS = ['java', 'llvm', 'net', 'wasm', 'forge'].map(
  (backend) => `ores-truffle-oreslang/oreslang-source.${backend}`,
);
const isRecord = (x) => x !== null && typeof x === 'object' && !Array.isArray(x);
const sameSha = (a, b) => typeof a === 'string' && SHA.test(a) && a === b;

function control(name, rules) {
  const failures = rules.filter((r) => !r.ok).map((r) => r.reason);
  return { name, status: failures.length ? 'HOLD' : 'REPORTED_READY', failures };
}

function branchRules(b, repo) {
  return [
    { ok: isRecord(b) && b.repository === repo, reason: `${repo}: branch readback absent or repository mismatch` },
    { ok: b?.branch === 'main' && b?.protected === true, reason: `${repo}: main not confirmed protected` },
    { ok: Number.isSafeInteger(b?.required_independent_reviews) && b.required_independent_reviews >= 1 && b?.require_codeowners === true, reason: `${repo}: independent CODEOWNERS review not enforced` },
    { ok: b?.dismiss_stale_reviews === true && b?.require_last_pusher_review === true, reason: `${repo}: stale/last-pusher review gates absent` },
    { ok: b?.allow_force_pushes === false && b?.allow_deletions === false && Array.isArray(b?.bypass_actors) && b.bypass_actors.length === 0, reason: `${repo}: force/delete or bypass controls not proven disabled` },
    { ok: Array.isArray(b?.required_trusted_checks) && b.required_trusted_checks.length > 0 && b.required_trusted_checks.every((v) => isRecord(v) && typeof v.name === 'string' && v.name.length > 0 && Number.isSafeInteger(v.app_id) && v.app_id > 0), reason: `${repo}: trusted required CI check provenance absent` },
  ];
}

export function assess(snapshot) {
  if (!isRecord(snapshot) || snapshot.schema !== 1) {
    throw new Error('schema=1 object required');
  }
  const ci = snapshot.source_ci;
  const carrier = snapshot.carrier;
  const compiler = snapshot.compiler;
  const pages = snapshot.pages;
  const branches = snapshot.source_branches;
  const allSourceBranches = SOURCE_REPOS.flatMap((repo) =>
    branchRules(Array.isArray(branches) ? branches.find((b) => b?.repository === repo) : undefined, repo),
  );
  const review = compiler?.review;
  const compilerHead = review?.head_sha;
  const approved = Array.isArray(review?.submissions) && review.submissions.some((r) =>
    r?.state === 'APPROVED' &&
    typeof r?.reviewer === 'string' &&
    ['oresoftware', 'the1mills'].includes(r.reviewer.toLowerCase()) &&
    r.reviewer.toLowerCase() !== (typeof review.author === "string" ? review.author.toLowerCase() : "") &&
    sameSha(r.commit_sha, compilerHead),
  );
  const controls = [
    control('1.private_ci_admission', [
      { ok: typeof ci?.repository === 'string' && ci.repository.startsWith('ores-truffle-oreslang/') && ci?.repository_private === true, reason: 'not a private original-source Oreslang job' },
      { ok: sameSha(ci?.head_sha, ci?.expected_sha), reason: 'current original-source head SHA not pinned' },
      { ok: ci?.status === 'completed' && ci?.conclusion === 'success' && Number.isSafeInteger(ci?.run_attempt) && ci.run_attempt > 0, reason: 'job has no successful completed run attempt' },
      { ok: Number.isSafeInteger(ci?.runner_id) && ci.runner_id > 0, reason: 'no assigned runner (zero-step admission failure)' },
      { ok: Array.isArray(ci?.steps) && ci.steps.length >= 2 && ci.steps.every((s) => s?.status === 'completed' && s?.conclusion === 'success'), reason: 'nonempty successfully executed steps absent' },
      { ok: Array.isArray(ci?.verified_log_markers) && ci.verified_log_markers.includes('SOURCE_FREE_START') && ci.verified_log_markers.includes('SOURCE_FREE_END'), reason: 'retrieved canary log markers absent' },
    ]),
    control('2.private_runner_carrier_authorization', [
      { ok: carrier?.source_org === 'ores-truffle-oreslang' && carrier?.source_private === true && carrier?.destination_private === true, reason: 'private-to-private transfer boundary not established' },
      { ok: Number.isSafeInteger(carrier?.destination_repo_id) && carrier.destination_repo_id > 0 && Array.isArray(carrier?.approved_repo_ids) && carrier.approved_repo_ids.includes(carrier.destination_repo_id), reason: 'destination numeric repository ID not explicitly allowlisted' },
      { ok: sameSha(carrier?.source_sha, carrier?.approved_sha) && typeof carrier?.source_tree_sha === 'string' && SHA.test(carrier.source_tree_sha), reason: 'immutable source commit/tree authorization absent' },
      { ok: carrier?.independent_security_approval === true && carrier?.transfer_incidents_resolved === true && carrier?.workflow_approved === true, reason: 'independent authorization, incident closure, or workflow approval absent' },
      { ok: carrier?.source_free_canary_only === false && carrier?.authorized_private_transfer === true, reason: 'source-free canary is NOT private-source transfer authorization' },
    ]),
    control('3.oreslang_main_protection', allSourceBranches),
    control('4.attolang_compiler_review', [
      ...branchRules(compiler?.branch_rules, 'atto-lang/atto-source.rs'),
      { ok: typeof compilerHead === 'string' && SHA.test(compilerHead), reason: 'compiler PR current head SHA absent' },
      { ok: approved === true, reason: 'no independent authorized approval bound to compiler PR current head' },
      { ok: review?.merge_tree_checked === true && review?.executed_head_ci === true, reason: 'executed exact-head and proposed-merge validation absent' },
    ]),
    control('5.attolang_pages_release', [
      ...branchRules(pages?.branch_rules, 'atto-lang/atto-lang.github.io'),
      { ok: pages?.deployment_source === 'github_actions' && pages?.legacy_branch_publishing === false, reason: 'legacy branch-based Pages publisher not confirmed disabled' },
      { ok: pages?.environment?.name === 'github-pages' && pages.environment.protected_branches_only === true && pages.environment.prevent_self_review === true && pages.environment.admin_bypass === false && Array.isArray(pages.environment.required_reviewers) && pages.environment.required_reviewers.length > 0, reason: 'protected environment reviewer/no-bypass controls absent' },
      { ok: pages?.manual_release_reviewed === true && pages?.compiler_digest_independently_approved === true, reason: 'manual release and compiler digest authorization absent' },
      { ok: pages?.live_audit?.manifest_http === 200 && pages.live_audit.forbidden_paths_denied === true && pages.live_audit.compiler_digest_matches === true && pages.live_audit.wasm_mime_correct === true, reason: 'live HTTPS manifest/digest/MIME/forbidden-path evidence absent' },
    ]),
  ];
  return { diagnostic_only: true, certified: false, status: controls.every((c) => c.status === 'REPORTED_READY') ? 'REPORTED_READY_NOT_CERTIFIED' : 'HOLD', controls };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const filename = process.argv[2];
  if (!filename || process.argv.length !== 3) {
    console.error('Usage: node security/recovery-gates.mjs snapshot.json');
    process.exitCode = 2;
  } else {
    try {
      const file = lstatSync(filename);
      if (!file.isFile() || file.isSymbolicLink() || file.size > 131072) throw new Error('input must be a regular JSON file <=128 KiB');
      const report = assess(JSON.parse(readFileSync(filename, 'utf8')));
      console.log(JSON.stringify(report, null, 2));
      if (report.status !== 'REPORTED_READY_NOT_CERTIFIED') process.exitCode = 1;
    } catch (error) {
      console.error(`HOLD: ${error.message}`);
      process.exitCode = 2;
    }
  }
}
