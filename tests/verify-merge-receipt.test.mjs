import test from 'node:test';
import assert from 'node:assert/strict';
import { verifyMergeReceipt, main } from '../tools/verify-merge-receipt.mjs';

const a='a'.repeat(40), b='b'.repeat(40), c='c'.repeat(40), d='d'.repeat(40);
const good=()=>({
 schema:'atto.merge-receipt/v1',
 pr:{repository:'ores-truffle-oreslang/example',number:42,author:'author',head_sha:a,head_tree_sha:b,base_sha:c},
 checkout:{sha:a,tree_sha:b,mode:'physical-original-head',dirty:false,synthetic_merge:false},
 protection:{main_protected:true,requires_pr:true,required_approvals:1,codeowners_required:true,dismiss_stale:true,include_admins:true,bypass_actors:0},
 workflow:{head_sha:a,head_tree_sha:b,run_id:123,attempt:2,event:'pull_request',status:'completed',conclusion:'success',completed_at:'2026-11-06T20:00:00Z',jobs:[
   {id:456,name:'native test',run_id:123,attempt:2,head_sha:a,head_tree_sha:b,runner_id:999,status:'completed',conclusion:'success',steps:[{name:'compile guest',kind:'test',status:'completed',conclusion:'success'}]}
 ]},
 reviews:[{author:'independent',commit_sha:a,state:'APPROVED',submitted_at:'2026-11-06T20:01:00Z',dismissed:false,superseded:false}],
 merge:{method:'squash',commit_sha:c,main_sha:d,main_tree_sha:b,pr_head_sha:a,github_merged:true,on_main_ancestry:true,main_protected_at_merge:true,merged_at:'2026-11-06T20:02:00Z'}
});
const blocked=(name,mutate,reason)=>test(name,()=>{const x=good();mutate(x);const v=verifyMergeReceipt(x);assert.equal(v.status,'BLOCKED');assert.ok(v.reasons.some(r=>r.includes(reason)),JSON.stringify(v));});
test('well-formed snapshot is still not a live authorization',()=>assert.deepEqual(verifyMergeReceipt(good()),{status:'SNAPSHOT_VALID_RECHECK_LIVE',reasons:[]}));
blocked('missing PR tree',x=>x.pr.head_tree_sha=null,'PR_INVALID');
blocked('synthetic checkout',x=>x.checkout.synthetic_merge=true,'CHECKOUT_INVALID');
blocked('wrong checkout tree',x=>x.checkout.tree_sha=c,'CHECKOUT_INVALID');
blocked('zero-step runner admission',x=>x.workflow.jobs[0].steps=[],'UNEXECUTED');
blocked('missing runner',x=>x.workflow.jobs[0].runner_id=0,'UNEXECUTED');
blocked('skipped test',x=>x.workflow.jobs[0].steps[0].conclusion='skipped','NO_EXECUTED_TEST');
blocked('running workflow',x=>x.workflow.status='in_progress','WORKFLOW_INVALID');
blocked('wrong attempt',x=>x.workflow.jobs[0].attempt=1,'IDENTITY');
blocked('wrong job source head',x=>x.workflow.jobs[0].head_sha=c,'IDENTITY');
blocked('wrong job source tree',x=>x.workflow.jobs[0].head_tree_sha=d,'IDENTITY');
blocked('duplicate job names',x=>x.workflow.jobs.push({...x.workflow.jobs[0],id:457}),'IDENTITY');
blocked('self approval',x=>x.reviews[0].author='author','INDEPENDENT_APPROVAL_MISSING');
blocked('stale review head',x=>x.reviews[0].commit_sha=c,'INDEPENDENT_APPROVAL_MISSING');
blocked('dismissed review',x=>x.reviews[0].dismissed=true,'INDEPENDENT_APPROVAL_MISSING');
blocked('active change request',x=>x.reviews.push({state:'CHANGES_REQUESTED',author:'reviewer',dismissed:false}),'CONFLICTING_REVIEW');
blocked('unprotected main',x=>x.protection.main_protected=false,'PROTECTION_UNENFORCED');
blocked('admin bypass',x=>x.protection.bypass_actors=1,'PROTECTION_UNENFORCED');
blocked('missing CODEOWNERS',x=>x.protection.codeowners_required=false,'PROTECTION_UNENFORCED');
blocked('merge head drift',x=>x.merge.pr_head_sha=c,'MERGE_INVALID');
blocked('not main ancestor',x=>x.merge.on_main_ancestry=false,'MERGE_INVALID');
blocked('approval after merge',x=>x.reviews[0].submitted_at='2026-11-06T20:03:00Z','REVIEW_AFTER_MERGE');
blocked('run after merge',x=>x.workflow.completed_at='2026-11-06T20:03:00Z','RUN_AFTER_MERGE');
blocked('invalid calendar date',x=>x.merge.merged_at='2026-02-30T20:02:00Z','MERGE_INVALID');
blocked('duplicate same reviewer not distinct',x=>{x.protection.required_approvals=2;x.reviews.push({...x.reviews[0]});},'INDEPENDENT_APPROVAL_MISSING');
test('CLI with no path cannot succeed',()=>{let out;assert.equal(main([],s=>out=s),2);assert.equal(JSON.parse(out).status,'BLOCKED');});
test('untrusted input denied',()=>assert.equal(verifyMergeReceipt(null).status,'BLOCKED'));
