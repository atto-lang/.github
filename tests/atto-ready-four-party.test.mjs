import test from 'node:test';
import assert from 'node:assert/strict';
import {assessFourPartyReadyCycle} from '../governance/atto-ready-four-party.mjs';
const HEAD='a'.repeat(40), OTHER='b'.repeat(40);
const r=(label, time='2026-11-19T15:45:00Z') => {
  const common={state:'APPROVED',commit_sha:HEAD,submitted_at:time};
  if (label==='openai') return {...common,source:'independent-provider-attestation',
    provider:'OpenAI',provider_verified:true,attestation_verified:true,agent_identity:'codex-run-1'};
  if (label==='anthropic') return {...common,source:'independent-provider-attestation',
    provider:'Anthropic',provider_verified:true,attestation_verified:true,agent_identity:'claude-run-1'};
  const login=label==='ore'?'ORESoftware':'the1mills';
  return {...common,source:'github-human',login,github_id:label==='ore'?11139560:22222222};
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
test('human account cannot self-review; only the nonauthor named human can approve',()=>{
  const x=valid();x.author_github_id=11139560;x.reviews=x.reviews.filter(r=>r.login!=='the1mills');
  assert.equal(gate(x).status,'HOLD');
  x.reviews=x.reviews.filter(r=>r.login!=='ORESoftware');
  x.reviews.push(r('mills'));assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
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
test('human approval predating a base sync is reusable; AI attestation is not',()=>{
  const x=valid();x.reviews=[r('openai'),r('anthropic'),r('ore','2026-11-19T15:20:00Z')];
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  x.reviews[0].submitted_at='2026-11-19T15:20:00Z';
  assert.equal(gate(x).status,'HOLD');
});
test('fast window opens at 72 hours with exactly one independent reviewer',()=>{
  const x=valid();x.ready_cycle_started_at='2026-11-16T16:00:00Z';
  x.reviews=[r('ore')];assert.equal(gate(x).lane,'fast-one-independent');
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
});
test('240h requires fresh AI only, while existing one human remains valid',()=>{
  const x=valid();x.ready_cycle_started_at='2026-11-09T16:00:00Z';
  x.reviews=[r('ore','2026-11-10T17:00:00Z'),r('openai','2026-11-19T15:45:00Z'),r('anthropic','2026-11-19T15:45:00Z')];
  assert.equal(gate(x).lane,'renewed-four-party');
  assert.equal(gate(x).status,'HOLD');
  x.checked_at='2026-11-19T16:10:00Z';
  x.reviews=x.reviews.map(v=>v.source==='github-human'?v:{...v,submitted_at:'2026-11-19T16:05:00Z'});
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
});
test('dismissal and blocking change requests do not count as approval',()=>{
  const x=valid();x.reviews[0].state='DISMISSED';assert.equal(gate(x).status,'HOLD');
  const y=valid();y.reviews.push({...r('ore','2026-11-19T15:50:00Z'),state:'CHANGES_REQUESTED'});
  assert.equal(gate(y).status,'HOLD');
});

test('initial window needs only either ONE named human plus both AIs',()=> {
  const x=valid();x.reviews=[r('ore'),r('openai'),r('anthropic')];
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  x.reviews=[r('mills'),r('openai'),r('anthropic')];
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  x.reviews=[r('openai'),r('anthropic')];assert.equal(gate(x).status,'HOLD');
});
test('approval stays valid when main sync moves HEAD, but AIs must refresh',()=>{
  const x=valid();x.head_sha=OTHER;x.last_push_at='2026-11-19T15:47:00Z';
  x.reviews=[r('ore','2026-11-19T15:40:00Z'),
    {...r('openai','2026-11-19T15:50:00Z'),commit_sha:OTHER},
    {...r('anthropic','2026-11-19T15:51:00Z'),commit_sha:OTHER}];
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  x.reviews[2].commit_sha=HEAD;assert.equal(gate(x).status,'HOLD');
});
test('ready-cycle reset does not erase a previously eligible human approval',()=>{
  const x=valid();x.ready_cycle_started_at='2026-11-19T15:40:00Z';
  x.reviews=[r('ore','2026-11-18T12:00:00Z'),r('openai'),r('anthropic')];
  assert.equal(gate(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
});
test('dismissed human approval and explicit draft hold labels fail closed',()=>{
  const x=valid();x.reviews=[r('openai'),r('anthropic'),r('ore')];
  x.reviews[2].state='DISMISSED';assert.equal(gate(x).status,'HOLD');
  x.reviews[2].state='APPROVED';x.labels=['keep-as-draft'];assert.equal(gate(x).status,'HOLD');
  x.labels=['keep-draft'];assert.equal(gate(x).status,'HOLD');
  x.labels=['not-ready'];assert.equal(gate(x).status,'HOLD');
  x.labels=[];x.labels_verified=false;assert.equal(gate(x).status,'HOLD');
});
