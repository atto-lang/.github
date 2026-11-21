import test from 'node:test';
import assert from 'node:assert/strict';
import {assessFourPartyReadyCycle, assessDraftHold} from '../governance/atto-ready-four-party.mjs';
const HEAD='a'.repeat(40), OTHER='b'.repeat(40);
const r=(label, time='2026-11-19T15:45:00Z') => {
  const common={state:'APPROVED',commit_sha:HEAD,submitted_at:time};
  if (label==='openai') return {...common,source:'independent-provider-attestation',
    provider:'OpenAI',provider_verified:true,attestation_verified:true,agent_identity:'codex-run-1'};
  if (label==='anthropic') return {...common,source:'independent-provider-attestation',
    provider:'Anthropic',provider_verified:true,attestation_verified:true,agent_identity:'claude-run-1'};
  const login=label==='ore'?'ORESoftware':'the1mills';
  return {...common,source:'github-human',login,github_id:label==='ore'?11139560:2148448};
};
function valid() {return {head_sha:HEAD,author_github_id:33333333,
  checked_at:'2026-11-19T16:00:00Z',ready_cycle_started_at:'2026-11-19T15:00:00Z',
  last_push_at:'2026-11-19T15:30:00Z',ready_cycle_verified:true,draft:false,
  labels_verified:true,labels:[],
  reviews:[r('openai'),r('anthropic'),r('ore'),r('mills')]};}
const gate=v=>assessFourPartyReadyCycle(v);
test('synthetic four independent approvals report advisory satisfaction only',()=>{
  const v=gate(valid());assert.equal(v.status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  assert.equal(v.unique_reviewers,4);assert.equal(v.advisory_only,true);
});
test('real PR #385 failure class: four approvals by one immutable ORESoftware ID are NOT four reviewers',()=>{
  const x=valid();x.reviews=[r('ore'),r('ore','2026-11-19T15:46:00Z'),
    r('ore','2026-11-19T15:47:00Z'),r('ore','2026-11-19T15:48:00Z')];
  const out=gate(x);assert.equal(out.status,'HOLD');assert.equal(out.unique_reviewers,1);
});
test('human account must not self-review even if named as a required party',()=>{
  const x=valid();x.author_github_id=11139560;assert.equal(gate(x).status,'HOLD');
});
test('review on stale head is not an approval',()=>{
  const x=valid();x.reviews[0].commit_sha=OTHER;assert.equal(gate(x).status,'HOLD');
});
test('AI review without independent provider authentication is rejected',()=>{
  const x=valid();x.reviews[0].attestation_verified=false;assert.equal(gate(x).status,'HOLD');
});
test('GitHub human comment cannot masquerade as Claude',()=>{
  const x=valid();x.reviews[1]={...r('ore'),provider:'Anthropic'};
  assert.equal(gate(x).status,'HOLD');
});
test('an otherwise valid four-party review fails when ready lifecycle provenance is missing',()=>{
  const x=valid();x.ready_cycle_verified=false;assert.equal(gate(x).status,'HOLD');
});
test('review before the last push cannot be counted',()=>{
  const x=valid();x.reviews[2].submitted_at='2026-11-19T15:20:00Z';
  assert.equal(gate(x).status,'HOLD');
});
test('fast window opens at 72 hours with a human plus AI',()=>{
  const x=valid();x.ready_cycle_started_at='2026-11-16T16:00:00Z';
  x.reviews=[r('ore'),r('openai')];assert.equal(gate(x).lane,'fast-one-independent');
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
});
test('240 hours renews four-party review and requires new evidence after cutoff',()=>{
  const x=valid();x.ready_cycle_started_at='2026-11-09T16:00:00Z';
  const out=gate(x);assert.equal(out.lane,'renewed-four-party');
  assert.equal(out.status,'HOLD');
  x.checked_at='2026-11-19T16:10:00Z';
  x.reviews=x.reviews.map(v=>({...v,submitted_at:'2026-11-19T16:05:00Z'}));
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
});
test('dismissal and blocking change requests do not count as approval',()=>{
  const x=valid();x.reviews[0].state='DISMISSED';assert.equal(gate(x).status,'HOLD');
  const y=valid();y.reviews.push({...r('ore','2026-11-19T15:50:00Z'),state:'CHANGES_REQUESTED'});
  assert.equal(gate(y).status,'HOLD');
});

test('72h fast lane rejects human-only or AI-only and requires one of each',()=>{
  const x=valid();x.ready_cycle_started_at='2026-11-16T16:00:00Z';
  x.reviews=[r('ore')];assert.equal(gate(x).status,'HOLD');
  x.reviews=[r('openai')];assert.equal(gate(x).status,'HOLD');
  x.reviews=[r('ore'),r('mills')];assert.equal(gate(x).status,'HOLD');
  x.reviews=[r('openai'),r('anthropic')];assert.equal(gate(x).status,'HOLD');
  x.reviews=[r('mills'),r('anthropic')];
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
});
test('keep-as-draft always blocks ready PR and orders conversion back to Draft',()=>{
  const x=valid();x.labels=[{name:'KEEP-AS-DRAFT'}];
  const report=gate(x);
  assert.equal(report.status,'HOLD');
  assert.equal(report.draft_action,'CONVERT_TO_DRAFT');
  assert.deepEqual(report.blocking_labels,['keep-as-draft']);
  assert.equal(assessDraftHold({...x,draft:true}).draft_action,'REMAIN_DRAFT');
  assert.equal(assessDraftHold({...x,draft:true}).labels_allow_auto_ready,false);
});
test('legacy keep-draft is the same hard Draft invariant; not-ready blocks merge',()=>{
  const x=valid();x.labels=['keep-draft'];
  assert.equal(gate(x).draft_action,'CONVERT_TO_DRAFT');
  assert.equal(gate(x).status,'HOLD');
  x.labels=['not-ready'];
  assert.equal(gate(x).draft_action,'NONE');
  assert.equal(gate(x).status,'HOLD');
});
test('unknown, malformed or unverified labels fail closed; PR text is irrelevant',()=>{
  const x=valid();
  x.labels_verified=false;assert.equal(gate(x).status,'HOLD');
  assert.equal(gate(x).draft_action,'HOLD_UNVERIFIED_LABELS');
  x.labels_verified=true;x.labels=undefined;assert.equal(gate(x).status,'HOLD');
  x.labels=[{name:''}];assert.equal(gate(x).status,'HOLD');
  x.labels=[];x.draft=undefined;assert.equal(gate(x).status,'HOLD');
  assert.equal(assessDraftHold(x).draft_action,'HOLD_UNVERIFIED_DRAFT_STATE');
  const clean=valid();clean.body='remove keep-as-draft please';
  assert.equal(gate(clean).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
});
test('hold overrides positive four-party approval in every ready-age window',()=>{
  for (const started of ['2026-11-19T15:00:00Z','2026-11-16T16:00:00Z','2026-11-09T16:00:00Z']) {
    const x=valid();x.ready_cycle_started_at=started;
    x.labels=['keep-as-draft'];x.reviews=x.reviews.map(v=>({...v,
      submitted_at:'2026-11-19T15:45:00Z'}));
    assert.equal(gate(x).status,'HOLD');
    assert.equal(gate(x).draft_action,'CONVERT_TO_DRAFT');
  }
});

test('human approval must match immutable authenticated GitHub account ID',()=>{
  const x=valid();
  x.reviews[2].github_id=999999;
  assert.equal(gate(x).status,'HOLD');
  assert.match(gate(x).failures.join(' '),/unverified or unqualified reviewer identity/);
  x.reviews[2]=r('ore');
  x.reviews[3].login='ORESoftware';
  assert.equal(gate(x).status,'HOLD');
});
test('a READY PR cannot be auto-promoted, even when unlabelled',()=>{
  const x=valid();
  assert.equal(assessDraftHold(x).labels_allow_auto_ready,false);
  assert.equal(assessDraftHold({...x,draft:true}).labels_allow_auto_ready,true);
  assert.equal(assessDraftHold({...x,draft:true,labels:['not-ready']}).labels_allow_auto_ready,false);
});
test('review clock is strict UTC; a re-draft and new Ready cycle reject old approvals',()=>{
  const x=valid();
  x.ready_cycle_started_at='2026-11-19T15:50:00Z';
  assert.equal(gate(x).status,'HOLD');
  x.ready_cycle_started_at='2026-11-19T15:00:00-00:00';
  assert.equal(gate(x).status,'HOLD');
  x.ready_cycle_started_at='2026-11-19T15:00:00Z';
  x.checked_at='yesterday';
  assert.equal(gate(x).status,'HOLD');
});
test('AI attestations require canonical provider identity and a nonblank agent identity',()=>{
  const x=valid();
  x.reviews[0].agent_identity=' ';
  assert.equal(gate(x).status,'HOLD');
  x.reviews[0]=r('openai');
  x.reviews[0].provider='oPeNaI';
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  x.reviews[0].provider='unknown-model';
  assert.equal(gate(x).status,'HOLD');
});
