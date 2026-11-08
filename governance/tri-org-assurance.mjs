// Read-only security/advisory contract for Tasks 6–10.
// No writes, network, source checkout, transfers, approvals, merges, or release authority.
import { readFileSync, lstatSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const HEX = /^[a-f0-9]{40}$/;
const ORGS = ['ores-truffle-oreslang', 'atto-lang', 'atto-co'];
const APPROVERS = new Set(['oresoftware','the1mills']);
const record = x => x !== null && typeof x === 'object' && !Array.isArray(x);
const sha = x => typeof x === 'string' && HEX.test(x);
const id = x => Number.isSafeInteger(x) && x > 0;
const iso = x => typeof x === 'string' && /^\d{4}-\d\d-\d\dT/.test(x) && Number.isFinite(Date.parse(x));
const fail = (a, pass, reason) => { if (!pass) a.push(reason); };
const outcome = (name, errors) => ({ control: name, status: errors.length ? 'HOLD' : 'REPORTED_SATISFIED', failures: errors });

export function reviewGate(input, namespace = 'review') {
  const e = [];
  const p = input?.protection;
  fail(e, sha(input?.head_sha), `${namespace}: current 40-hex head SHA absent`);
  fail(e, typeof input?.author === 'string' && input.author.length > 0, `${namespace}: PR author absent`);
  fail(e, record(p) && p.enforced === true && p.main_protected === true, `${namespace}: effective protected main not confirmed`);
  fail(e, p?.required_approvals === 1 && p?.codeowners_required === true, `${namespace}: one independent CODEOWNER approval not enforced`);
  fail(e, p?.stale_dismissal === true && p?.last_push_approval === true, `${namespace}: stale / last-pusher review controls missing`);
  fail(e, p?.force_push === false && p?.deletion === false && Array.isArray(p?.bypass_actors) && p.bypass_actors.length === 0, `${namespace}: force push, deletion or bypass not proven disabled`);
  fail(e, typeof p?.trusted_check === 'string' && p.trusted_check.length > 0 && id(p?.trusted_app_id), `${namespace}: issuer-bound required CI check absent`);
  const reviews = input?.reviews;
  fail(e, Array.isArray(reviews), `${namespace}: review history absent`);
  const latest = new Map();
  if (Array.isArray(reviews)) {
    for (const r of reviews) {
      if (!record(r) || typeof r.login !== 'string' || !iso(r.submitted_at) ||
          !['APPROVED','CHANGES_REQUESTED','COMMENTED','DISMISSED'].includes(r.state) ||
          !sha(r.commit_sha)) {
        e.push(`${namespace}: malformed review history`);
        continue;
      }
      const key = r.login.toLowerCase();
      const prev = latest.get(key);
      if (!prev || Date.parse(prev.submitted_at) < Date.parse(r.submitted_at)) latest.set(key, r);
      else if (Date.parse(prev.submitted_at) === Date.parse(r.submitted_at)) e.push(`${namespace}: ambiguous equal-timestamp reviews`);
    }
  }
  const lastPush = input?.last_push_at;
  fail(e, iso(lastPush), `${namespace}: last push timestamp missing`);
  const author = typeof input?.author === 'string' ? input.author.toLowerCase() : '';
  const approved = [...latest.values()].filter(r => r.state === 'APPROVED' &&
    APPROVERS.has(r.login.toLowerCase()) && r.login.toLowerCase() !== author &&
    r.commit_sha === input?.head_sha && iso(lastPush) &&
    Date.parse(r.submitted_at) > Date.parse(lastPush));
  fail(e, approved.length >= 1, `${namespace}: independent authorized current-head APPROVED review after last push absent`);
  fail(e, ![...latest.values()].some(r => r.state === 'CHANGES_REQUESTED'),
    `${namespace}: unresolved CHANGES_REQUESTED review`);
  return outcome(namespace, e);
}

function runGate(run, head, namespace) {
  const e = [];
  fail(e, record(run) && id(run.run_id) && id(run.workflow_id) && run.event === 'pull_request', `${namespace}: trusted PR workflow/run IDs absent`);
  fail(e, sha(head) && run?.head_sha === head, `${namespace}: run head differs from PR head`);
  fail(e, run?.status === 'completed' && run?.conclusion === 'success' && id(run?.attempt), `${namespace}: successful exact attempt missing`);
  fail(e, Array.isArray(run?.jobs) && run.jobs.length > 0, `${namespace}: jobs absent`);
  const seen = new Set();
  if (Array.isArray(run?.jobs)) for (const job of run.jobs) {
    fail(e, id(job?.job_id) && !seen.has(job.job_id), `${namespace}: job ID invalid or duplicated`);
    if (id(job?.job_id)) seen.add(job.job_id);
    fail(e, job?.run_id === run.run_id && job?.attempt === run.attempt && id(job?.runner_id), `${namespace}: job identity or assigned runner absent`);
    fail(e, job?.conclusion === 'success' && Array.isArray(job?.steps) && job.steps.length > 0 &&
      job.steps.some(s => s?.name === 'Execute mandatory tests' && s?.status === 'completed' && s?.conclusion === 'success') &&
      job.steps.every(s => s?.status === 'completed' && s?.conclusion === 'success'),
      `${namespace}: real successful named mandatory test steps absent`);
  }
  return e;
}

export function mergeWaveGate(wave) {
  const e = [];
  const entries = wave?.prs;
  fail(e, Array.isArray(entries) && entries.length === 5, 'merge wave must contain exactly five PRs');
  const names = new Set(), repos = new Set();
  if (Array.isArray(entries)) for (const [idx, pr] of entries.entries()) {
    const label = `merge PR ${idx+1}`;
    fail(e, typeof pr?.repository === 'string' && /^[-\w]+\/[-.\w]+$/.test(pr.repository) && pr.repository.startsWith('ores-truffle-oreslang/'), `${label}: source repository invalid`);
    fail(e, id(pr?.number) && !names.has(`${pr.repository}#${pr.number}`), `${label}: PR number absent or duplicated`);
    names.add(`${pr?.repository}#${pr?.number}`);
    if (typeof pr?.repository === 'string') repos.add(pr.repository);
    fail(e, pr?.base === 'main' && sha(pr?.base_sha) && sha(pr?.head_sha) && pr?.draft === false && pr?.merged === false,
      `${label}: must be open, non-draft, against pinned main`);
    fail(e, typeof pr?.author === 'string' && pr.author.length > 0 &&
      pr?.review?.author === pr.author && pr?.review?.head_sha === pr.head_sha,
      `${label}: approval author's PR identity or reviewed head diverges from PR`);
    e.push(...reviewGate(pr?.review, label).failures);
    e.push(...runGate(pr?.run, pr?.head_sha, label));
  }
  fail(e, repos.size >= 3, 'five-PR batch must span >=3 repositories');
  fail(e, wave?.all_dependencies_reconciled === true && wave?.conflicts_resolved === true && wave?.agent_lease_verified === true,
    'dependency/conflict/concurrent-agent review missing');
  return outcome('7.merge_wave_exact_head', e);
}

export function privateCarrierGate(c) {
  const e = [];
  fail(e, c?.source_org === 'ores-truffle-oreslang' && c?.source_private === true && id(c?.source_repo_id),
    'private authenticated source identity missing');
  fail(e, c?.destination_private === true && id(c?.destination_repo_id) &&
    Array.isArray(c?.reviewed_destination_ids) && c.reviewed_destination_ids.includes(c.destination_repo_id),
    'reviewed immutable numeric private destination ID absent');
  fail(e, c?.source_repo_id === c?.approved_source_repo_id &&
    typeof c?.destination_full_name === 'string' &&
    /^[-\\w]+\\/[-.\\w]+$/.test(c.destination_full_name) &&
    c.destination_full_name === c?.approved_destination_full_name,
    'source numeric ID or destination name not bound to an independent approved identity');
  fail(e, sha(c?.source_commit) && sha(c?.source_tree) && c.source_commit === c?.approved_source_commit &&
    c.source_tree === c?.approved_source_tree, 'source immutable commit/tree mismatch');
  fail(e, c?.policy_origin === 'reviewed_protected_main' && c?.source_branch_protected === true &&
    c?.source_codeowner_approval === true, 'source policy not independently protected and reviewed');
  fail(e, c?.source_preflight_verified === true && c?.destination_preflight_verified === true &&
    c?.prewrite_revalidation === true && c?.postwrite_tree_matches === true,
    'source+destination preflight, atomic recheck or verified destination tree absent');
  fail(e, c?.security_incidents_closed === true && c?.private_transfer_explicitly_approved === true &&
    c?.workflow_reviewed === true, 'private source transfer security authority missing');
  fail(e, c?.source_free_canary_only === false, 'source-free runner capacity is not source-export authorization');
  return outcome('8.private_carrier', e);
}

export function retrospectiveGate(batch) {
  const e = [];
  const records = batch?.landed_prs;
  fail(e, Array.isArray(records) && records.length === 10, 'exactly ten landed PR records required');
  const pairs = new Set();
  if (Array.isArray(records)) for (const [idx,r] of records.entries()) {
    const label = `landed PR ${idx+1}`;
    fail(e, typeof r?.repository === 'string' && r.repository.startsWith('ores-truffle-oreslang/') &&
      id(r?.number) && !pairs.has(`${r.repository}#${r.number}`), `${label}: duplicate or invalid repository/PR`);
    pairs.add(`${r?.repository}#${r?.number}`);
    fail(e, sha(r?.landed_commit) && sha(r?.landed_tree) && r?.merged === true && r?.current_main_contains_landed === true,
      `${label}: exact landed commit/tree ancestry missing`);
    fail(e, r?.current_main_protected === true && r?.issuer_bound_checks_required === true,
      `${label}: currently protected main/issuer-bound checks missing`);
    const s = r?.security_review;
    fail(e, s?.kind === 'POST_MERGE' && s?.state === 'APPROVED' && sha(s?.reviewed_commit) &&
      s.reviewed_commit === r?.landed_commit && typeof s?.reviewer === 'string' &&
      s.reviewer.toLowerCase() !== (r?.author||'').toLowerCase() &&
      APPROVERS.has(s.reviewer.toLowerCase()), `${label}: independent labeled post-merge security review missing`);
    fail(e, r?.landed_ci?.commit_sha === r?.landed_commit && r?.landed_ci?.status === 'completed' &&
      r?.landed_ci?.conclusion === 'success' && id(r?.landed_ci?.runner_id) &&
      id(r?.landed_ci?.run_id) && id(r?.landed_ci?.workflow_id) && id(r?.landed_ci?.attempt) &&
      Array.isArray(r?.landed_ci?.executed_test_steps) &&
      r.landed_ci.executed_test_steps.includes('Execute mandatory tests'),
      `${label}: executed exact-landed CI missing`);
    fail(e, r?.current_integration_reviewed === true && r?.blocking_findings_closed === true,
      `${label}: current integration review/findings incomplete`);
  }
  return outcome('9.post_merge_release_hold', e);
}

export function ownershipRegistryGate(registry) {
  const e = [];
  fail(e, registry?.schema === 'tri-org-identity-v1' && iso(registry?.observed_at),
    'dated registry schema and observation absent');
  fail(e, registry?.migration_authorized === false && registry?.private_transfer_authorized === false,
    'inventory is not a migration or source-transfer grant');
  const entries = registry?.repositories;
  fail(e, Array.isArray(entries) && entries.length === 165, 'full 165-repository snapshot absent');
  const counts = Object.fromEntries(ORGS.map(x=>[x,0]));
  const ids = new Set(), names = new Set();
  if (Array.isArray(entries)) for (const [idx,r] of entries.entries()) {
    const k = `row ${idx+1}`;
    fail(e, record(r) && id(r.repo_id), `${k}: stable numeric GitHub ID absent`);
    fail(e, typeof r?.full_name === 'string' && /^[-\w]+\/[-.\w]+$/.test(r.full_name), `${k}: full name absent`);
    fail(e, r?.visibility === 'private' || r?.visibility === 'public', `${k}: visibility unknown`);
    fail(e, r?.default_branch === 'main' && r?.archived === false, `${k}: default branch or archival ambiguity`);
    fail(e, r?.codeowner_verified === false && r?.transfer_status === 'not_authorized', `${k}: invented ownership or migration decision`);
    if(id(r?.repo_id)) {
      fail(e,!ids.has(r.repo_id),`${k}: duplicate numeric repo identity`);
      ids.add(r.repo_id);
    }
    if (typeof r?.full_name === 'string') {
      fail(e,!names.has(r.full_name),`${k}: duplicate repository name`);
      names.add(r.full_name);
      const org = r.full_name.split('/')[0];
      fail(e, ORGS.includes(org), `${k}: unexpected organization`);
      fail(e, r?.current_organization === org, `${k}: declared owner organization disagrees with repository identity`);
      if(ORGS.includes(org)) counts[org]++;
    }
  }
  for(const [org,n] of [['ores-truffle-oreslang',80],['atto-lang',68],['atto-co',17]]) {
    fail(e, counts[org] === n, `${org}: expected ${n} observed entries, found ${counts[org]}`);
  }
  fail(e, registry?.migration_policy_reference === 'ores-truffle-oreslang/.github#25',
    'migration decisions must refer to the independently reviewed private source manifest');
  return outcome('10.repository_ownership_inventory',e);
}

export function assess(snapshot) {
  if (!record(snapshot) || snapshot.schema !== 'tri-org-controls-v1') throw new TypeError('tri-org-controls-v1 object required');
  const controls = [
    reviewGate(snapshot.review_policy,'6.independent_review'),
    mergeWaveGate(snapshot.merge_wave),
    privateCarrierGate(snapshot.private_carrier),
    retrospectiveGate(snapshot.retrospective),
    ownershipRegistryGate(snapshot.registry),
  ];
  return {
    advisory_only:true, authorized:false, certified:false,
    status: controls.every(x=>x.status === 'REPORTED_SATISFIED') ? 'REPORTED_SATISFIED_NOT_AUTHORIZED' : 'HOLD',
    controls,
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const f = process.argv[2];
  if (!f || process.argv.length !== 3) {
    console.error('Usage: node governance/tri-org-assurance.mjs snapshot.json');
    process.exitCode = 2;
  } else try {
    const stat = lstatSync(f);
    if (!stat.isFile() || stat.isSymbolicLink() || stat.size > 2097152) throw Error('input must be non-symlink regular JSON <=2MiB');
    const report = assess(JSON.parse(readFileSync(f,'utf8')));
    console.log(JSON.stringify(report,null,2));
    process.exitCode = report.status === 'HOLD' ? 1 : 0;
  } catch(e) {
    console.error('HOLD: ' + e.message);
    process.exitCode = 2;
  }
}
