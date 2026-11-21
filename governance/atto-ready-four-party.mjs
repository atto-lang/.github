// Read-only advisory evaluator for AttoLang's one-time human + time-window AI review policy.
// This validates supplied authenticated evidence, not GitHub live state. NEVER
// authorize merges or treat untrusted PR text as reviewer attestation.
const SHA = /^[a-f0-9]{40}$/;
const HOUR = 60 * 60 * 1000;
const HUMAN = new Map([['oresoftware','human-oresoftware'],['the1mills','human-the1mills']]);
const AI = new Map([['openai','ai-openai'],['anthropic','ai-anthropic']]);
const requiredAi = [...AI.values()];
const date = x => typeof x === 'string' && Number.isFinite(Date.parse(x)) ? Date.parse(x) : NaN;
const int = x => Number.isSafeInteger(x) && x > 0;
export function assessFourPartyReadyCycle(x) {
  const failures = [];
  const bad = (condition, why) => { if (!condition) failures.push(why); };
  const checked = date(x?.checked_at), started = date(x?.ready_cycle_started_at);
  const pushed = date(x?.last_push_at);
  bad(SHA.test(x?.head_sha ?? ''), 'missing immutable head SHA');
  bad(x?.ready_cycle_verified === true && !x?.draft, 'ready lifecycle not independently verified');
  bad(Number.isFinite(checked) && Number.isFinite(started) && checked >= started,
      'ready-cycle clock missing or invalid');
  bad(Number.isFinite(pushed) && pushed <= checked, 'last-push timestamp invalid');
  bad(int(x?.author_github_id), 'author numeric GitHub identity absent');
  bad(Array.isArray(x?.reviews), 'authenticated review history absent');
  bad(x?.labels_verified === true && Array.isArray(x?.labels), 'current PR labels not independently verified');
  if (Array.isArray(x?.labels)) {
    bad(!x.labels.some(label => ['keep-as-draft','keep-draft','not-ready'].includes(String(label).toLowerCase())),
        'explicit draft/readiness hold label present');
  }
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
  // The one authorized human approval is PR-scoped: it persists across all
  // Ready windows and merges/rebases from main, even if the HEAD SHA changes.
  // An explicitly dismissed or superseded review remains invalid.
  const human = [...latest.values()].filter(r => r.state === 'APPROVED' &&
    r.source === 'github-human' &&
    r.github_id !== x?.author_github_id && date(r.submitted_at) <= checked);
  const currentAi = [...latest.values()].filter(r => r.state === 'APPROVED' &&
    r.source === 'independent-provider-attestation' &&
    r.commit_sha === x?.head_sha && date(r.submitted_at) >= boundary &&
    date(r.submitted_at) > pushed && date(r.submitted_at) <= checked);
  const humanRoles = new Set(human.map(r => r.role));
  const aiRoles = new Set(currentAi.map(r => r.role));
  bad(humanRoles.size >= 1, 'one independent the1mills OR ORESoftware approval missing');
  if (lane !== 'fast-one-independent') {
    bad(requiredAi.every(role => aiRoles.has(role)),
        'both independently verified OpenAI and Anthropic current-head attestations missing');
  }
  const principals = new Set([...human,...currentAi].map(r => r.key));
  return {status: failures.length ? 'HOLD' : 'REPORTED_SATISFIED_NOT_AUTHORIZED',
          advisory_only: true, lane, unique_reviewers: principals.size, failures};
}
