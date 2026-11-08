import test from 'node:test';
import assert from 'node:assert/strict';
import {
  assess, reviewGate, mergeWaveGate, privateCarrierGate,
  retrospectiveGate, ownershipRegistryGate,
} from '../governance/tri-org-assurance.mjs';

const SHA = 'a'.repeat(40), TREE = 'b'.repeat(40);
const BEFORE = '2026-11-08T09:00:00Z', AFTER = '2026-11-08T10:00:00Z';
function protection() {
  return {enforced:true, main_protected:true, required_approvals:1, codeowners_required:true,
    stale_dismissal:true, last_push_approval:true, force_push:false, deletion:false,
    bypass_actors:[], trusted_check:'mandatory-tests', trusted_app_id:15368};
}
function review(author='the1mills', head=SHA) {
  return {head_sha:head, author, last_push_at:BEFORE, protection:protection(),
    reviews:[{login:'ORESoftware',state:'APPROVED',commit_sha:head,submitted_at:AFTER}]};
}
function run(head=SHA, number=1) {
  return {run_id:100+number, workflow_id:200+number, event:'pull_request', head_sha:head,
    status:'completed',conclusion:'success',attempt:1,jobs:[{job_id:300+number,
      run_id:100+number,attempt:1,runner_id:400+number,conclusion:'success',
      steps:[{name:'Set up job',status:'completed',conclusion:'success'},
        {name:'Execute mandatory tests',status:'completed',conclusion:'success'}]}]};
}
function mergeWave() {
  return {all_dependencies_reconciled:true,conflicts_resolved:true,agent_lease_verified:true,
    prs:Array.from({length:5},(_,i)=>({repository:`ores-truffle-oreslang/synthetic-${i%3}`,
      number:100+i,base:'main',base_sha:TREE,head_sha:SHA,draft:false,merged:false,
      review:review(),run:run(SHA,i+1)}))};
}
function carrier() {
  return {source_org:'ores-truffle-oreslang',source_private:true,source_repo_id:12,
    destination_private:true,destination_repo_id:34,reviewed_destination_ids:[34],
    source_commit:SHA,source_tree:TREE,approved_source_commit:SHA,approved_source_tree:TREE,
    policy_origin:'reviewed_protected_main',source_branch_protected:true,
    source_codeowner_approval:true,source_preflight_verified:true,
    destination_preflight_verified:true,prewrite_revalidation:true,postwrite_tree_matches:true,
    security_incidents_closed:true,private_transfer_explicitly_approved:true,
    workflow_reviewed:true,source_free_canary_only:false};
}
function retrospect() {
  return {landed_prs:Array.from({length:10},(_,i)=>({
    repository:`ores-truffle-oreslang/synthetic-${i%9}`,number:400+i,landed_commit:SHA,
    landed_tree:TREE,author:'the1mills',merged:true,current_main_contains_landed:true,
    current_main_protected:true,issuer_bound_checks_required:true,
    security_review:{kind:'POST_MERGE',state:'APPROVED',reviewer:'ORESoftware',reviewed_commit:SHA},
    landed_ci:{commit_sha:SHA,status:'completed',conclusion:'success',runner_id:123,
      executed_test_steps:['Execute mandatory tests']},
    current_integration_reviewed:true,blocking_findings_closed:true
  }))};
}
function registry() {
  let next=1; const repositories=[];
  for (const [org,n] of [['ores-truffle-oreslang',80],['atto-lang',68],['atto-co',17]]) {
    for(let i=0;i<n;i++) repositories.push({
      repo_id:next++,full_name:`${org}/synthetic-${i}`,visibility:'private',
      default_branch:'main',archived:false,codeowner_verified:false,transfer_status:'not_authorized'
    });
  }
  return {schema:'tri-org-identity-v1', observed_at:'2026-11-08T12:00:00Z',
    migration_authorized:false,private_transfer_authorized:false,
    migration_policy_reference:'ores-truffle-oreslang/.github#25',repositories};
}
function valid() {return {schema:'tri-org-controls-v1',review_policy:review(),
  merge_wave:mergeWave(),private_carrier:carrier(),retrospective:retrospect(),registry:registry()};}
function reportGate(value,idx) { return assess(value).controls[idx].status; }

test('all absent evidence remains an explicit five-control HOLD',()=>{
  const s=assess({schema:'tri-org-controls-v1'});
  assert.equal(s.status,'HOLD');assert.equal(s.authorized,false);
  assert.deepEqual(s.controls.map(x=>x.status),Array(5).fill('HOLD'));
});
test('synthetic all-positive evidence NEVER becomes merge or release authority',()=>{
  const s=assess(valid());assert.equal(s.status,'REPORTED_SATISFIED_NOT_AUTHORIZED');
  assert.equal(s.certified,false);assert.equal(s.authorized,false);assert.equal(s.advisory_only,true);
});
test('wrong top-level schema rejected',()=>assert.throws(()=>assess({schema:'v0'}),/tri-org-controls-v1/));

const tests=[
 ['author self-approval denied',s=>s.review_policy.reviews[0].login='the1mills',0],
 ['unapproved reviewer denied',s=>s.review_policy.reviews[0].login='unrelated-bot',0],
 ['stale SHA approval denied',s=>s.review_policy.reviews[0].commit_sha=TREE,0],
 ['pre-push approval denied',s=>s.review_policy.reviews[0].submitted_at='2026-11-08T08:00:00Z',0],
 ['dismissed approval denied',s=>s.review_policy.reviews[0].state='DISMISSED',0],
 ['changes requested denies otherwise approved',s=>s.review_policy.reviews.push({login:'auditor',state:'CHANGES_REQUESTED',commit_sha:SHA,submitted_at:AFTER}),0],
 ['unprotected main denied',s=>s.review_policy.protection.main_protected=false,0],
 ['two required approvals do not implement one approval policy',s=>s.review_policy.protection.required_approvals=2,0],
 ['missing CODEOWNERS denied',s=>s.review_policy.protection.codeowners_required=false,0],
 ['missing last push review denied',s=>s.review_policy.protection.last_push_approval=false,0],
 ['bypass actor denied',s=>s.review_policy.protection.bypass_actors=['admin'],0],
 ['untrusted required check denied',s=>s.review_policy.protection.trusted_app_id=null,0],
 ['four PR wave denied',s=>s.merge_wave.prs.pop(),1],
 ['five PR wave spanning two repos denied',s=>s.merge_wave.prs.forEach((p,i)=>p.repository=`ores-truffle-oreslang/synthetic-${i%2}`),1],
 ['duplicate PR denied',s=>{s.merge_wave.prs[1].repository=s.merge_wave.prs[0].repository;s.merge_wave.prs[1].number=s.merge_wave.prs[0].number;},1],
 ['draft PR denied',s=>s.merge_wave.prs[0].draft=true,1],
 ['non-main stacked PR denied',s=>s.merge_wave.prs[0].base='feature-branch',1],
 ['CI run for another head denied',s=>s.merge_wave.prs[0].run.head_sha=TREE,1],
 ['zero-step CI denied',s=>s.merge_wave.prs[0].run.jobs[0].steps=[],1],
 ['zero-runner CI denied',s=>s.merge_wave.prs[0].run.jobs[0].runner_id=0,1],
 ['renamed test step denied',s=>s.merge_wave.prs[0].run.jobs[0].steps[1].name='No tests',1],
 ['stale or malformed attempt denied',s=>s.merge_wave.prs[0].run.attempt=0,1],
 ['agent lease missing denied',s=>s.merge_wave.agent_lease_verified=false,1],
 ['source private identity absent denied',s=>s.private_carrier.source_private=false,2],
 ['numeric destination swap denied',s=>s.private_carrier.destination_repo_id=35,2],
 ['source tree differs denied',s=>s.private_carrier.approved_source_tree=SHA,2],
 ['caller-controlled source policy denied',s=>s.private_carrier.policy_origin='untrusted_pr',2],
 ['unprotected source denied',s=>s.private_carrier.source_branch_protected=false,2],
 ['missing source preflight denied',s=>s.private_carrier.source_preflight_verified=false,2],
 ['policy recheck omitted denied',s=>s.private_carrier.prewrite_revalidation=false,2],
 ['destination tree mismatch denied',s=>s.private_carrier.postwrite_tree_matches=false,2],
 ['unresolved security transfer incidents denied',s=>s.private_carrier.security_incidents_closed=false,2],
 ['source free canary not transfer permission',s=>s.private_carrier.source_free_canary_only=true,2],
 ['private transfer not explicitly approved',s=>s.private_carrier.private_transfer_explicitly_approved=false,2],
 ['nine retrospective records denied',s=>s.retrospective.landed_prs.pop(),3],
 ['duplicate landed PR denied',s=>{s.retrospective.landed_prs[1].number=s.retrospective.landed_prs[0].number;s.retrospective.landed_prs[1].repository=s.retrospective.landed_prs[0].repository;},3],
 ['landed SHA not on main denied',s=>s.retrospective.landed_prs[0].current_main_contains_landed=false,3],
 ['pre-merge review falsely labeled as post-merge denied',s=>s.retrospective.landed_prs[0].security_review.kind='PRE_MERGE',3],
 ['postmerge self-review denied',s=>s.retrospective.landed_prs[0].security_review.reviewer='the1mills',3],
 ['unprotected landed main denied',s=>s.retrospective.landed_prs[0].current_main_protected=false,3],
 ['zero-runner landed CI denied',s=>s.retrospective.landed_prs[0].landed_ci.runner_id=0,3],
 ['missing executed landed tests denied',s=>s.retrospective.landed_prs[0].landed_ci.executed_test_steps=[],3],
 ['unresolved postmerge finding denied',s=>s.retrospective.landed_prs[0].blocking_findings_closed=false,3],
 ['incomplete 164 repository roster denied',s=>s.registry.repositories.pop(),4],
 ['repo numeric ID duplicate denied',s=>s.registry.repositories[1].repo_id=1,4],
 ['repo name duplicate denied',s=>s.registry.repositories[1].full_name=s.registry.repositories[0].full_name,4],
 ['wrong target org count denied',s=>s.registry.repositories[0].full_name='atto-lang/fake-extra',4],
 ['invented CODEOWNERS verification denied',s=>s.registry.repositories[0].codeowner_verified=true,4],
 ['inventory cannot authorize transfer',s=>s.registry.private_transfer_authorized=true,4],
 ['migration policy citation absent denied',s=>s.registry.migration_policy_reference='',4],
 ['unknown visibility denied',s=>s.registry.repositories[0].visibility='unknown',4],
 ['archived repo cannot satisfy active inventory',s=>s.registry.repositories[0].archived=true,4],
];
for(const [name,mutate,idx] of tests) test(name,()=>{
  const s=valid();mutate(s);assert.equal(reportGate(s,idx),'HOLD');
});
test('each positive synthetic control validates individually without certifying source evidence',()=>{
  const v=valid();
  assert.equal(reviewGate(v.review_policy).status,'REPORTED_SATISFIED');
  assert.equal(mergeWaveGate(v.merge_wave).status,'REPORTED_SATISFIED');
  assert.equal(privateCarrierGate(v.private_carrier).status,'REPORTED_SATISFIED');
  assert.equal(retrospectiveGate(v.retrospective).status,'REPORTED_SATISFIED');
  assert.equal(ownershipRegistryGate(v.registry).status,'REPORTED_SATISFIED');
});
