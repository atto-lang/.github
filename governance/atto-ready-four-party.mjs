// Read-only advisory evaluator for a single independent trusted-owner review.
// Review evidence, labels and timeline MUST be externally authenticated.
// This cannot authorize a merge: current source CI, overlap and GitHub protection
// remain separate, mandatory checks.
const SHA=/^[0-9a-f]{40}$/;
const HUMAN=new Map([['oresoftware',11139560],['the1mills',2148448]]);
const TRUSTED_IDS=new Set(HUMAN.values());
const HOLD_LABELS=new Set(['keep-as-draft','keep-draft','not-ready']);
const millis=x=>typeof x==='string'&&Number.isFinite(Date.parse(x))?Date.parse(x):NaN;
const positive=n=>Number.isSafeInteger(n)&&n>0;
const named=s=>typeof s==='string'&&s.trim().length>0;

export function assessFourPartyReadyCycle(x) {
  // Historical export name retained for existing callers; four-party voting
  // is NOT a policy requirement.
  const failures=[];
  const require=(ok,why)=>{if(!ok)failures.push(why);};
  const checked=millis(x?.checked_at), started=millis(x?.ready_cycle_started_at);
  const pushed=millis(x?.last_push_at);
  require(SHA.test(x?.head_sha??''),'missing immutable current head SHA');
  require(x?.ready_cycle_verified===true&&x?.draft===false,
    'Ready lifecycle not independently verified or still Draft');
  require(Number.isFinite(started)&&Number.isFinite(checked)&&checked>=started,
    'Ready lifecycle timestamps missing or invalid');
  require(Number.isFinite(pushed)&&pushed<=checked,'last-push timestamp invalid');
  require(positive(x?.author_github_id),'PR author numeric GitHub identity absent');
  require(x?.reviews_verified===true&&Array.isArray(x?.reviews)&&x.reviews.length<=5000,
    'authenticated bounded GitHub review history absent');
  require(x?.labels_verified===true&&Array.isArray(x?.labels),
    'current PR labels not independently verified');
  for(const raw of Array.isArray(x?.labels)?x.labels:[]) {
    const label=typeof raw==='string'?raw:
      raw&&typeof raw==='object'&&!Array.isArray(raw)?raw.name:null;
    if(!named(label)){failures.push('malformed current PR label');continue;}
    if(HOLD_LABELS.has(label.trim().toLowerCase()))
      failures.push('explicit draft/readiness hold label present');
  }
  const latest=new Map();
  for(const r of Array.isArray(x?.reviews)?x.reviews.slice(0,5000):[]) {
    if(!r||typeof r!=='object'||!['APPROVED','DISMISSED',
      'CHANGES_REQUESTED','COMMENTED'].includes(r.state)){
      failures.push('invalid review record');continue;
    }
    const at=millis(r.submitted_at);
    if(!Number.isFinite(at)||at>checked){
      failures.push('review timestamp missing or future');continue;
    }
    // Provider claims neither help nor hurt the one independent human slot;
    // these are not GitHub human review events or quorum requirements.
    if(r.source==='independent-provider-attestation')continue;
    if(r.source!=='github-human'){
      failures.push('unrecognized or unauthenticated review source');continue;
    }
    if(!positive(r.github_id)||!named(r.login)){
      failures.push('unauthenticated GitHub reviewer identity');continue;
    }
    const login=r.login.trim().toLowerCase(), expected=HUMAN.get(login);
    if((expected&&r.github_id!==expected)||
      (!expected&&TRUSTED_IDS.has(r.github_id))){
      failures.push('trusted reviewer login / immutable ID mismatch');continue;
    }
    // A COMMENTED review does not dismiss an earlier actual approval or
    // clear an earlier CHANGES_REQUESTED event.
    if(r.state==='COMMENTED')continue;
    const prior=latest.get(r.github_id);
    if(prior?.at===at)failures.push('ambiguous equal-time review events');
    else if(!prior||prior.at<at)latest.set(r.github_id,{review:r,at});
  }
  const current=[...latest.values()].map(v=>v.review);
  require(!current.some(r=>r.state==='CHANGES_REQUESTED'),
    'outstanding blocking GitHub CHANGES_REQUESTED review');
  const eligible=current.filter(r=>r.state==='APPROVED'&&
    HUMAN.get(r.login.trim().toLowerCase())===r.github_id&&
    r.github_id!==x?.author_github_id);
  require(eligible.length>=1,
    'one independent the1mills OR ORESoftware approval missing');
  return {status:failures.length?'HOLD':'REPORTED_SATISFIED_NOT_AUTHORIZED',
    advisory_only:true,lane:'single-trusted-human',
    unique_reviewers:eligible.length,human_approvals:eligible.length,
    required_ai:[],failures};
}
