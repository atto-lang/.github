import test from 'node:test';
import assert from 'node:assert/strict';
import { assess } from '../security/recovery-gates.mjs';
const SHA = 'a'.repeat(40);
function branch(repository) { return { repository, branch: 'main', protected: true, required_independent_reviews: 1, require_codeowners: true, dismiss_stale_reviews: true, require_last_pusher_review: true, allow_force_pushes: false, allow_deletions: false, bypass_actors: [], required_trusted_checks: [{ name: 'trusted-ci', app_id: 15368 }] }; }
function valid() {
  return {
    schema: 1,
    source_ci: { repository: 'ores-truffle-oreslang/oreslang-source.java', repository_private: true, head_sha: SHA, expected_sha: SHA, status: 'completed', conclusion: 'success', run_attempt: 1, runner_id: 42, steps: [{status:'completed',conclusion:'success'}, {status:'completed',conclusion:'success'}], verified_log_markers: ['SOURCE_FREE_START','SOURCE_FREE_END'] },
    carrier: { source_org: 'ores-truffle-oreslang', source_private: true, destination_private: true, destination_repo_id: 42, approved_repo_ids: [42], source_sha: SHA, approved_sha: SHA, source_tree_sha: SHA, independent_security_approval: true, transfer_incidents_resolved: true, workflow_approved: true, source_free_canary_only: false, authorized_private_transfer: true },
    source_branches: ['java','llvm','net','wasm','forge'].map(b => branch(`ores-truffle-oreslang/oreslang-source.${b}`)),
    compiler: { branch_rules: branch('atto-lang/atto-source.rs'), review: { head_sha: SHA, author: 'the1mills', submissions: [{ state: 'APPROVED', reviewer: 'ORESoftware', commit_sha: SHA }], merge_tree_checked: true, executed_head_ci: true } },
    pages: { branch_rules: branch('atto-lang/atto-lang.github.io'), deployment_source:'github_actions', legacy_branch_publishing:false, environment: { name:'github-pages', protected_branches_only:true, prevent_self_review:true, admin_bypass:false, required_reviewers:['ORESoftware'] }, manual_release_reviewed:true, compiler_digest_independently_approved:true, live_audit: { manifest_http:200, forbidden_paths_denied:true, compiler_digest_matches:true, wasm_mime_correct:true } },
  };
}
const status = (r,i) => assess(r).controls[i-1].status;
test('no live administrative evidence means five HOLDS', () => {
  const r = assess({schema:1});
  assert.equal(r.status,'HOLD'); assert.equal(r.certified,false);
  assert.deepEqual(r.controls.map(c=>c.status),Array(5).fill('HOLD'));
});
test('synthetic all-true input is never certified', () => {
  const r = assess(valid()); assert.equal(r.status,'REPORTED_READY_NOT_CERTIFIED'); assert.equal(r.certified,false);
});
test('zero-step job cannot pass source CI', () => { const r=valid();r.source_ci.steps=[];assert.equal(status(r,1),'HOLD'); });
test('zero-runner job cannot pass source CI', () => { const r=valid();r.source_ci.runner_id=0;assert.equal(status(r,1),'HOLD'); });
test('no log markers cannot pass source CI', () => { const r=valid();r.source_ci.verified_log_markers=[];assert.equal(status(r,1),'HOLD'); });
test('stale head cannot pass source CI', () => { const r=valid();r.source_ci.expected_sha='b'.repeat(40);assert.equal(status(r,1),'HOLD'); });
test('unallowlisted carrier numeric ID fails closed', () => { const r=valid();r.carrier.approved_repo_ids=[43];assert.equal(status(r,2),'HOLD'); });
test('source-free runner pass cannot authorize private source export', () => { const r=valid();r.carrier.source_free_canary_only=true;r.carrier.authorized_private_transfer=false;assert.equal(status(r,2),'HOLD'); });
test('unresolved private-source incidents block carrier', () => { const r=valid();r.carrier.transfer_incidents_resolved=false;assert.equal(status(r,2),'HOLD'); });
test('all five source branches are required', () => { const r=valid();r.source_branches.pop();assert.equal(status(r,3),'HOLD'); });
test('bypass actors block protection', () => { const r=valid();r.source_branches[0].bypass_actors=['admin'];assert.equal(status(r,3),'HOLD'); });
test('status checks require trusted app ID', () => { const r=valid();r.source_branches[0].required_trusted_checks=[{name:'trusted-ci'}];assert.equal(status(r,3),'HOLD'); });
test('compiler author cannot approve own PR', () => { const r=valid();r.compiler.review.submissions[0].reviewer='the1mills';assert.equal(status(r,4),'HOLD'); });
test('approval on obsolete SHA not enough', () => { const r=valid();r.compiler.review.submissions[0].commit_sha='b'.repeat(40);assert.equal(status(r,4),'HOLD'); });
test('compiler source branch protection is mandatory', () => { const r=valid();r.compiler.branch_rules.protected=false;assert.equal(status(r,4),'HOLD'); });
test('legacy branch publisher remains a blocker', () => { const r=valid();r.pages.legacy_branch_publishing=true;assert.equal(status(r,5),'HOLD'); });
test('deployment environment must prevent self review', () => { const r=valid();r.pages.environment.prevent_self_review=false;assert.equal(status(r,5),'HOLD'); });
test('public test-path exposure blocks Pages certification', () => { const r=valid();r.pages.live_audit.forbidden_paths_denied=false;assert.equal(status(r,5),'HOLD'); });
test('unknown schema rejected', () => assert.throws(()=>assess({schema:2}),/schema=1/));
test('wrong types fail closed, not as implicit policy booleans', () => { const r=valid(); r.source_ci.repository=9; r.compiler.branch_rules.required_independent_reviews='1'; const a=assess(r); assert.equal(a.controls[0].status,'HOLD'); assert.equal(a.controls[3].status,'HOLD'); });
