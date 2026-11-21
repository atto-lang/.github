// Read-only advisory evaluator for AttoLang's one-human-approval policy.
// This validates supplied authenticated evidence, NOT live GitHub branch policy.
// It cannot authorize a merge. Current-head CI and GitHub protection are separate.
const SHA = /^[a-f0-9]{40}$/;
const HUMAN = new Map([['oresoftware',11139560], ['the1mills',2148448]]);
const date = x => typeof x === 'string' && Number.isFinite(Date.parse(x)) ? Date.parse(x) : NaN;
const int = x => Number.isSafeInteger(x) && x > 0;
export function assessFourPartyReadyCycle(x) {
  const failures = [];
  const bad = (condition, reason) => { if (!condition) failures.push(reason); };
  const checked = date(x?.checked_at), started = date(x?.ready_cycle_started_at);
  const pushed = date(x?.last_push_at);
  bad(SHA.test(x?.head_sha ?? ''), 'missing immutable head SHA');
  bad(x?.ready_cycle_verified === true && x?.draft === false, 'ready lifecycle not independently verified');
  bad(Number.isFinite(checked) && Number.isFinite(started) && checked >= started,
      'ready-cycle timestamps missing or invalid');
  bad(Number.isFinite(pushed) && pushed <= checked, 'last-push timestamp invalid');
  bad(int(x?.author_github_id), 'author numeric GitHub identity absent');
  bad(Array.isArray(x?.reviews), 'authenticated review history absent');
  bad(x?.labels_verified === true && Array.isArray(x?.labels), 'current PR labels not independently verified');
  if (Array.isArray(x?.labels)) {
    bad(!x.labels.some(v => ['keep-as-draft','keep-draft','not-ready'].includes(String(v).toLowerCase())),
        'explicit draft/readiness hold label present');
  }
  const latest = new Map();
  for (const r of Array.isArray(x?.reviews) ? x.reviews : []) {
    if (!r || !['APPROVED','DISMISSED','CHANGES_REQUESTED'].includes(r.state)) {
      failures.push('invalid review record'); continue;
    }
    const at = date(r.submitted_at);
    bad(Number.isFinite(at) && at <= checked, 'review timestamp missing or future');
    if (r.state === 'CHANGES_REQUESTED') failures.push('blocking changes requested');
    // AI/provider attestations are NOT required and NEVER count as named human approval.
    if (r.source === 'independent-provider-attestation') continue;
    if (r.source !== 'github-human') {
      failures.push('unverified review source'); continue;
    }
    const login = String(r.login ?? '').toLowerCase();
    const expected = HUMAN.get(login);
    // Other GitHub humans may comment/review, but only either designated owner counts.
    if (!expected) continue;
    if (!int(r.github_id) || r.github_id !== expected) {
      failures.push('named reviewer GitHub ID mismatch'); continue;
    }
    if (r.github_id === x?.author_github_id) {
      failures.push('PR author self-review is not independent'); continue;
    }
    if (!Number.isFinite(at)) continue;
    const key = 'gh:' + expected, prev = latest.get(key);
    if (!prev || date(prev.submitted_at) < at) latest.set(key, r);
    else if (date(prev.submitted_at) === at) failures.push('ambiguous equal-time reviewer event');
  }
  // A PR-scoped human approval survives normal head changes, ready-cycle changes,
  // and the original review timestamp, unless GitHub explicitly dismissed/revoked it.
  const approvers = [...latest.values()].filter(r => r.state === 'APPROVED');
  bad(approvers.length >= 1, 'one independent the1mills OR ORESoftware approval missing');
  return {status: failures.length ? 'HOLD' : 'REPORTED_SATISFIED_NOT_AUTHORIZED',
          advisory_only: true, lane: 'single-trusted-human',
          unique_reviewers: approvers.length, failures};
}
