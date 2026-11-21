import test from 'node:test';
import assert from 'node:assert/strict';
import {assessFourPartyReadyCycle} from '../governance/atto-ready-four-party.mjs';
const HEAD='a'.repeat(40), OTHER='b'.repeat(40);
const approval=(who, when='2026-11-19T15:45:00Z') => ({
  source:'github-human',state:'APPROVED',login:who==='ore'?'ORESoftware':'the1mills',
  github_id:who==='ore'?11139560:2148448,commit_sha:HEAD,submitted_at:when
});
const ai={source:'independent-provider-attestation',state:'APPROVED',provider:'OpenAI',
  agent_identity:'unneeded',submitted_at:'2026-11-19T15:45:00Z',commit_sha:HEAD};
const valid=()=>({head_sha:HEAD,author_github_id:33333333,
  checked_at:'2026-11-19T16:00:00Z',ready_cycle_started_at:'2026-11-19T15:00:00Z',
  last_push_at:'2026-11-19T15:30:00Z',ready_cycle_verified:true,draft:false,
  labels_verified:true,labels:[],reviews:[approval('ore')]});
const check=x=>assessFourPartyReadyCycle(x);
const passed=x=>assert.equal(check(x).status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
const blocked=x=>assert.equal(check(x).status,'HOLD');
test('a single eligible independent ORESoftware approval suffices, advisory only',()=>{
  const r=check(valid());assert.equal(r.status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  assert.equal(r.unique_reviewers,1);assert.equal(r.advisory_only,true);
  assert.equal(r.lane,'single-trusted-human');
});
test('one the1mills approval is equally valid without ORESoftware or AI',()=>{
  const x=valid();x.reviews=[approval('mills')];passed(x);
});
test('no approval cannot be replaced by AI, GitHub comments, or untrusted humans',()=>{
  const x=valid();x.reviews=[];blocked(x);
  x.reviews=[ai];blocked(x);
  x.reviews=[{...approval('ore'),login:'someoneElse',github_id:999}];blocked(x);
});
test('the author cannot self-review, but the other named owner can',()=>{
  const x=valid();x.author_github_id=11139560;blocked(x);
  x.reviews=[approval('mills')];passed(x);
  x.author_github_id=2148448;blocked(x);
  x.reviews=[approval('ore')];passed(x);
});
test('trusted names must match immutable GitHub account IDs',()=>{
  const x=valid();x.reviews=[{...approval('ore'),github_id:2148448}];blocked(x);
  x.reviews=[{...approval('mills'),github_id:11139560}];blocked(x);
  x.reviews=[{...approval('ore'),github_id:11139560}];passed(x);
});
test('one approval remains sufficient after head/last-push changes',()=>{
  const x=valid();x.head_sha=OTHER;x.last_push_at='2026-11-19T15:59:00Z';
  x.reviews=[approval('ore','2026-11-19T15:10:00Z')];passed(x);
});
test('one earlier approval remains sufficient after new Ready cycle and after 240 hours',()=>{
  const x=valid();x.ready_cycle_started_at='2026-11-19T15:50:00Z';
  x.reviews=[approval('ore','2026-11-18T16:00:00Z')];passed(x);
  x.ready_cycle_started_at='2026-11-08T16:00:00Z';passed(x);
  x.ready_cycle_started_at='2026-11-16T16:00:00Z';passed(x);
});
test('multiple reviews by the same account do not create a second approval requirement',()=>{
  const x=valid();x.reviews=[approval('ore'),approval('ore','2026-11-19T15:55:00Z')];
  const r=check(x);assert.equal(r.unique_reviewers,1);passed(x);
});
test('explicitly dismissed or revoked latest approval must not count',()=>{
  const x=valid();x.reviews=[{...approval('ore'),state:'DISMISSED'}];blocked(x);
  x.reviews=[approval('ore'),{...approval('ore','2026-11-19T15:55:00Z'),state:'DISMISSED'}];blocked(x);
});
test('unresolved changes requests remain blockers even if an owner approved',()=>{
  const x=valid();x.reviews.push({...approval('mills','2026-11-19T15:55:00Z'),state:'CHANGES_REQUESTED'});
  blocked(x);
  x.reviews=[approval('ore'),{...approval('ore','2026-11-19T15:57:00Z'),state:'CHANGES_REQUESTED'}];blocked(x);
});
test('AI reviews never count and cannot veto the one-human approval merely by absence',()=>{
  const x=valid();passed(x);x.reviews.push(ai);passed(x);
  x.reviews.push({...ai,provider:'Anthropic',agent_identity:'claude'});passed(x);
});
test('review history is authenticated, timestamps valid, and policy remains advisory',()=>{
  const x=valid();x.ready_cycle_verified=false;blocked(x);
  x.ready_cycle_verified=true;x.draft=true;blocked(x);
  x.draft=false;x.checked_at='bad';blocked(x);
  x.checked_at='2026-11-19T16:00:00Z';x.reviews=[{...approval('ore'),submitted_at:'2027-01-01T00:00:00Z'}];blocked(x);
  x.reviews=[];blocked(x);x.reviews=null;blocked(x);
});
test('protected hold labels and missing label attestation fail closed',()=>{
  for(const label of ['keep-as-draft','keep-draft','not-ready']) {
    const x=valid();x.labels=[label];blocked(x);
  }
  const x=valid();x.labels_verified=false;blocked(x);
});
test('ambiguous equal-time, malformed event, and unverified source fail closed',()=>{
  const x=valid();x.reviews=[approval('ore'),approval('ore')];blocked(x);
  x.reviews=[{state:'APPROVED',source:'forged',submitted_at:'2026-11-19T15:45:00Z'}];blocked(x);
  x.reviews=[{...approval('ore'),state:'PENDING'}];blocked(x);
});
