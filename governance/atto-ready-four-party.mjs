// Read-only advisory evaluator for AttoLang's ready-cycle four-party policy.
// A verified keep-as-draft label is a hard Draft-state invariant; this module
// reports required actions but does not itself operate on GitHub.
// This validates supplied authenticated evidence, not GitHub live state. NEVER
// authorize merges or treat untrusted PR text as reviewer attestation.
const SHA = /^[a-f0-9]{40}$/;
const HOUR = 60 * 60 * 1000;
const HUMAN = new Map([['oresoftware','human-oresoftware'],['the1mills','human-the1mills']]);
const AI = new Map([['openai','ai-openai'],['anthropic','ai-anthropic']]);
const roles = [...HUMAN.values(),...AI.values()];
const date = x => typeof x === 'string' && Number.isFinite(Date.parse(x)) ? Date.parse(x) : NaN;
const int = x => Number.isSafeInteger(x) && x > 0;

// Never infer label absence from an unavailable or untrusted label listing.
// Caller MUST obtain current labels and draft state from authenticated GitHub
// APIs at the action boundary; PR bodies or workflow inputs are not evidence.
const DRAFT_HOLDS = new Set(['keep-as-draft', 'keep-draft']);
const MERGE_HOLDS = new Set([...DRAFT_HOLDS, 'not-ready']);
export function assessDraftHold(x) {
  const names = [];
  let verified = x?.labels_verified === true && Array.isArray(x?.labels);
  if (verified) for (const label of x.labels) {
    const name = typeof label === 'string' ? label
      : label && !Array.isArray(label) && typeof label.name === 'string' ? label.name : null;
    if (!name || name.trim() !== name) { verified = false; break; }
    names.push(name.toLowerCase());
  }
  const blocking_labels = verified
    ? [...new Set(names.filter(name => MERGE_HOLDS.has(name)))].sort() : [];
  const draft_required = verified && blocking_labels.some(name => DRAFT_HOLDS.has(name));
  const draftKnown = typeof x?.draft === 'boolean';
  return {
    advisory_only: true, labels_verified: verified, blocking_labels, draft_required,
    draft_action: !verified ? 'HOLD_UNVERIFIED_LABELS'
      : !draftKnown ? 'HOLD_UNVERIFIED_DRAFT_STATE'
      : draft_required ? (x.draft ? 'REMAIN_DRAFT' : 'CONVERT_TO_DRAFT') : 'NONE',
    labels_allow_auto_ready: verified && draftKnown && blocking_labels.length === 0,
  };
}
export function assessFourPartyReadyCycle(x) {
  const failures = [];
  const bad = (condition, why) => { if (!condition) failures.push(why); };
  const draftHold = assessDraftHold(x);
  bad(draftHold.labels_verified, 'current PR labels not independently verified');
  bad(typeof x?.draft === 'boolean', 'current PR draft state absent');
  bad(draftHold.blocking_labels.length === 0,
      'blocking hold label: ' + draftHold.blocking_labels.join(', '));
  const checked = date(x?.checked_at), started = date(x?.ready_cycle_started_at);
  const pushed = date(x?.last_push_at);
  bad(SHA.test(x?.head_sha ?? ''), 'missing immutable head SHA');
  bad(x?.ready_cycle_verified === true && !x?.draft, 'ready lifecycle not independently verified');
  bad(Number.isFinite(checked) && Number.isFinite(started) && checked >= started,
      'ready-cycle clock missing or invalid');
  bad(Number.isFinite(pushed) && pushed <= checked, 'last-push timestamp invalid');
  bad(int(x?.author_github_id), 'author numeric GitHub identity absent');
  bad(Array.isArray(x?.reviews), 'authenticated review history absent');
  const age = Number.isFinite(checked) && Number.isFinite(started) ? checked - started : NaN;
  const lane = age >= 240 * HOUR ? 'renewed-four-party'
             : age >= 72 * HOUR ? 'fast-one-independent' : 'initial-four-party';
  const boundary = lane === 'renewed-four-party' ? started + 240 * HOUR : started;
  const latest = new Map();
  for (const r of Array.isArray(x?.reviews) ? x.reviews : []) {
    if (!r || !['APPROVED','DISMISSED','CHANGES_REQUESTED'].includes(r.state)) {
      failures.push('invalid review record'); continue;
    }
    const at = date(r.submitted_at);
    bad(Number.isFinite(at) && at <= checked, 'review timestamp missing or future');
    // A GitHub account cannot become two independent reviewers by using two
    // role strings, and a human GitHub APPROVED event is never an AI attestation.
    let key, role;
    if (r.source === 'github-human' && int(r.github_id) &&
        HUMAN.has(String(r.login).toLowerCase())) {
      key = 'gh:' + r.github_id;
      role = HUMAN.get(String(r.login).toLowerCase());
      bad(r.github_id !== x.author_github_id, 'self-review not independent');
    } else if (r.source === 'independent-provider-attestation' &&
               AI.has(String(r.provider).toLowerCase()) &&
               r.provider_verified === true && r.attestation_verified === true &&
               typeof r.agent_identity === 'string' && r.agent_identity.length > 0) {
      key = 'ai:' + r.provider + ':' + r.agent_identity;
      role = AI.get(String(r.provider).toLowerCase());
    } else {
      failures.push('unverified or unqualified reviewer identity'); continue;
    }
    if (r.state === 'CHANGES_REQUESTED') failures.push('blocking changes requested');
    if (!Number.isFinite(at)) continue;
    const prev = latest.get(key);
    if (!prev || date(prev.submitted_at) < at) latest.set(key, {...r, role, key});
    else if (date(prev.submitted_at) === at) failures.push('ambiguous equal-time reviewer event');
  }
  const qualified = [...latest.values()].filter(r => r.state === 'APPROVED' &&
    r.commit_sha === x?.head_sha && date(r.submitted_at) >= boundary &&
    date(r.submitted_at) > pushed && date(r.submitted_at) <= checked &&
    !(r.source === 'github-human' && r.github_id === x.author_github_id));
  const principals = new Set(qualified.map(r => r.key));
  const byRole = new Map();
  for (const r of qualified) {
    if (byRole.has(r.role) && byRole.get(r.role) !== r.key)
      failures.push('multiple principals claim one required role');
    byRole.set(r.role, r.key);
  }
  if (lane === 'fast-one-independent') {
    const humanCount = qualified.filter(r => r.role.startsWith('human-')).length;
    const aiCount = qualified.filter(r => r.role.startsWith('ai-')).length;
    bad(humanCount >= 1 && aiCount >= 1 && principals.size >= 2,
        'fast lane requires one qualified human AND one verified AI approval');
  } else {
    bad(roles.every(role => byRole.has(role)) && principals.size >= 4 &&
        new Set([...byRole.values()]).size === 4,
        'four distinct OpenAI, Anthropic, ORESoftware and the1mills approvals missing');
  }
  return {status: failures.length ? 'HOLD' : 'REPORTED_SATISFIED_NOT_AUTHORIZED',
          advisory_only: true, lane, unique_reviewers: principals.size,
          draft_action: draftHold.draft_action,
          blocking_labels: draftHold.blocking_labels, failures};
}
