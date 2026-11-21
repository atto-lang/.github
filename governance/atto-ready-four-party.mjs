// Read-only advisory evaluation of AttoLang's persistent one-human + ready-cycle policy.
// The caller MUST independently authenticate GitHub lifecycle, labels, reviews and
// provider attestations. This function is NOT a GitHub branch protection or merge gate.
// Never use its output alone to authorize a merge or to bypass required current-head CI.
const SHA = /^[0-9a-f]{40}$/;
const HOUR = 60 * 60 * 1000;
const HUMAN = new Map([['oresoftware', 11139560], ['the1mills', 2148448]]);
const TRUSTED_IDS = new Set(HUMAN.values());
const AI = ['openai', 'anthropic'];
const HOLD_LABELS = new Set(['keep-as-draft', 'keep-draft', 'not-ready']);
const PERSISTENT_DRAFT_HOLDS = new Set(['keep-as-draft', 'keep-draft']);
const millis = s => typeof s === 'string' && Number.isFinite(Date.parse(s)) ? Date.parse(s) : NaN;
const validId = n => Number.isSafeInteger(n) && n > 0;
const text = s => typeof s === 'string' && s.trim().length > 0;

export function assessFourPartyReadyCycle(x) { // Retained export for existing callers.
  const failures = [];
  const require = (ok, why) => { if (!ok) failures.push(why); };
  const checked = millis(x?.checked_at), started = millis(x?.ready_cycle_started_at);
  const pushed = millis(x?.last_push_at);
  const age = checked - started;
  const lane = !Number.isFinite(age) || age < 0 ? 'unknown' :
    age < 72 * HOUR ? 'initial-two-ai' :
    age < 240 * HOUR ? 'fast-one-human' : 'renewed-two-ai';
  const aiRequired = lane !== 'fast-one-human';
  const aiBoundary = lane === 'renewed-two-ai' ? started + 240 * HOUR : started;
  require(SHA.test(x?.head_sha ?? ''), 'missing immutable current head SHA');
  require(x?.ready_cycle_verified === true && x?.draft === false,
    'ready lifecycle not independently verified');
  require(Number.isFinite(age) && age >= 0, 'ready-cycle timestamps missing or invalid');
  require(Number.isFinite(pushed) && pushed <= checked, 'last-push timestamp invalid');
  require(validId(x?.author_github_id), 'PR author numeric GitHub ID absent');
  require(x?.reviews_verified === true && Array.isArray(x?.reviews) &&
    x.reviews.length <= 5000, 'authenticated bounded review history absent');
  require(x?.labels_verified === true && Array.isArray(x?.labels),
    'current PR labels not independently verified');

  let persistentDraftHold = false;
  for (const raw of Array.isArray(x?.labels) ? x.labels : []) {
    const name = typeof raw === 'string' ? raw :
      raw && typeof raw === 'object' && !Array.isArray(raw) ? raw.name : null;
    if (!text(name)) { failures.push('malformed current PR label'); continue; }
    const label = name.trim().toLowerCase();
    if (HOLD_LABELS.has(label))
      failures.push('explicit draft/readiness hold label present');
    if (PERSISTENT_DRAFT_HOLDS.has(label)) persistentDraftHold = true;
  }
  // This is advisory only; a separately authorized controller must re-fetch
  // live lifecycle and labels before a Draft-state correction. "not-ready"
  // blocks merge but is not itself a permanent Draft-state invariant.
  const draftHoldAction = x?.labels_verified === true &&
    x?.ready_cycle_verified === true && persistentDraftHold
      ? x?.draft === false ? 'CONVERT_TO_DRAFT' :
        x?.draft === true ? 'REMAIN_DRAFT' : 'RECHECK_LIFECYCLE_AND_LABELS'
      : 'NONE';

  const humanLatest = new Map();
  const aiLatest = new Map();
  const putLatest = (map, key, r, at) => {
    const old = map.get(key);
    if (old?.at === at) failures.push('ambiguous equal-time review events');
    else if (!old || old.at < at) map.set(key, {review: r, at});
  };

  for (const r of Array.isArray(x?.reviews) ? x.reviews.slice(0, 5000) : []) {
    if (!r || typeof r !== 'object' || !['APPROVED', 'DISMISSED',
      'CHANGES_REQUESTED', 'COMMENTED'].includes(r.state)) {
      failures.push('invalid review record'); continue;
    }
    const at = millis(r.submitted_at);
    if (!Number.isFinite(at) || at > checked) {
      failures.push('review timestamp missing or future'); continue;
    }
    if (r.source === 'github-human') {
      if (!validId(r.github_id) || !text(r.login)) {
        failures.push('unauthenticated GitHub reviewer identity'); continue;
      }
      const login = r.login.trim().toLowerCase();
      const expected = HUMAN.get(login);
      if ((expected && r.github_id !== expected) ||
          (!expected && TRUSTED_IDS.has(r.github_id))) {
        failures.push('trusted reviewer login / immutable ID mismatch'); continue;
      }
      // COMMENTED reviews do not withdraw an earlier APPROVED review.
      if (r.state !== 'COMMENTED') putLatest(humanLatest, r.github_id, r, at);
      continue;
    }
    if (r.source === 'independent-provider-attestation') {
      // Non-current attestations are historical evidence, not an approval on HEAD.
      // Only provenance-verified current-cycle attestations can fill an AI slot.
      const provider = String(r.provider ?? '').toLowerCase();
      if (!AI.includes(provider) || r.provider_verified !== true ||
          r.attestation_verified !== true || !text(r.agent_identity)) {
        failures.push('unverified provider attestation'); continue;
      }
      // COMMENTED does not revoke an earlier attestation. Dismissal and
      // changed-requests must supersede a prior APPROVED on this head.
      if (r.state === 'COMMENTED') continue;
      if (r.commit_sha !== x?.head_sha || at <= pushed ||
          at < started || at < aiBoundary) continue;
      putLatest(aiLatest, provider, r, at);
      continue;
    }
    failures.push('unrecognized or unauthenticated review source');
  }

  const active = [...humanLatest.values()].map(v => v.review);
  require(!active.some(r => r.state === 'CHANGES_REQUESTED'),
    'outstanding blocking GitHub CHANGES_REQUESTED review');
  const acceptedHumans = active.filter(r => r.state === 'APPROVED' &&
    HUMAN.get(r.login.trim().toLowerCase()) === r.github_id &&
    r.github_id !== x?.author_github_id);
  require(acceptedHumans.length >= 1,
    'one independent the1mills OR ORESoftware approval missing');
  if (aiRequired) {
    require(AI.every(provider => aiLatest.get(provider)?.review.state === 'APPROVED'),
      'fresh independently verified and unrevoked OpenAI AND Anthropic attestations missing');
    const identities = [...aiLatest.values()].map(v => v.review.agent_identity);
    require(new Set(identities).size === identities.length,
      'AI provider attestations must have distinct verified agent identities');
  }

  return {
    status: failures.length ? 'HOLD' : 'REPORTED_SATISFIED_NOT_AUTHORIZED',
    advisory_only: true,
    draft_hold_action: draftHoldAction,
    lane,
    ready_age_hours: Number.isFinite(age) ? age / HOUR : null,
    unique_reviewers: acceptedHumans.length + (aiRequired ? aiLatest.size : 0),
    human_approvals: acceptedHumans.length,
    required_ai: aiRequired ? [...AI] : [],
    failures
  };
}
