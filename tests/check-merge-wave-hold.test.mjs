import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { mkdtempSync, writeFileSync, symlinkSync, linkSync, rmSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkSnapshot, parseCanonicalSnapshot, readSnapshot } from '../tools/check-merge-wave-hold.mjs';

const repos=['oreslang-source.forge','oreslang-source.js','oreslang-source.hdl',
  'oreslang-source.rs','oreslang-source.net','sublime-text-plugins',
  'sublime-text-plugins','oreslang-common-source','oreslang-ssr','oreslang-linked-queue'];
const gates=['live_required_main_protection','independent_review_for_release',
  'exact_landed_commit_source_owned_ci','source_guest_tests_where_applicable',
  'admin_runner_admission_resolution','no_unapproved_private_source_export'];
function valid() {
  return {
    schema:'atto.merge-wave-retroactive-snapshot.v1', snapshot_date_utc:'2026-11-06',
    source:'authenticated read-only GitHub PR/review/branch/compare/Actions API observations',
    classification:'historical_snapshot_not_live_release_authorization', state:'BLOCKED',
    merge_count:10, repository_count:9, approved_merge_count:0,
    protected_main_repository_count:0, sampled_success_stepful_merge_count:2,
    sampled_zero_step_failure_merge_count:8,
    merges:repos.map((r,i)=>({repo:`ores-truffle-oreslang/${r}`,pr:100+i,
      pr_head:'a'.repeat(39)+(i+1).toString(16),landed_sha:'f'.repeat(38)+(i+11).toString(16).padStart(2,'0'),
      main_contains_land_sha:true,main_compare_behind_by:0,
      submitted_review_count:0,main_protected:false,
      post_merge_ci_sample:{run_id:1000+i,job_id:2000+i,
        executed_steps:i===5||i===6?15:0,status:i===5||i===6?'success_stepful':'not_executed_sample'},
      release_certification:'BLOCKED'})),
    release_gates:[...gates], notes:['premerge approval cannot be recreated',
      'zero steps is not executed','host tests do not prove guest execution',
      'this is never an authorization to release'],
  };
}
const canonical=x=>JSON.stringify(x,null,2)+'\n';

test('baseline: exactly ten merges, nine repos and eight unexecuted samples remain blocked',()=>{
  assert.deepEqual(checkSnapshot(valid()),{
    state:'BLOCKED',merges:10,repositories:9,executedSamples:2,
    notExecutedSamples:8,releaseAuthorized:false});
  assert.equal(parseCanonicalSnapshot(canonical(valid())).releaseAuthorized,false);
});
const negatives=[
  ['forged approved state', x=>{x.state='CERTIFIED'}],
  ['forged authority classification',x=>{x.classification='trusted_release'}],
  ['version downgrade',x=>{x.schema='v0'}],
  ['extra release_ready field',x=>{x.release_ready=true}],
  ['wrong date',x=>{x.snapshot_date_utc='2026-11-07'}],
  ['missing source provenance',x=>{x.source='unknown'}],
  ['source prefix spoofing',x=>{x.source+=' fabricated by untrusted source'}],
  ['count mismatch',x=>{x.merge_count=9}],
  ['nine-PR short wave',x=>{x.merges.pop()}],
  ['incorrect repository count',x=>{x.repository_count=10}],
  ['author-reported independent approval',x=>{x.approved_merge_count=1}],
  ['fabricated effective protection',x=>{x.protected_main_repository_count=1}],
  ['modified passing-samples count',x=>{x.sampled_success_stepful_merge_count=10}],
  ['modified zero-step count',x=>{x.sampled_zero_step_failure_merge_count=0}],
  ['missing required release gate',x=>{x.release_gates.pop()}],
  ['reordered release gates',x=>{x.release_gates.reverse()}],
  ['duplicate PR identity',x=>{x.merges[1].repo=x.merges[0].repo;x.merges[1].pr=x.merges[0].pr}],
  ['duplicate landed commit',x=>{x.merges[1].repo=x.merges[0].repo;x.merges[1].landed_sha=x.merges[0].landed_sha}],
  ['duplicate job ID',x=>{x.merges[1].repo=x.merges[0].repo;x.merges[1].post_merge_ci_sample.job_id=x.merges[0].post_merge_ci_sample.job_id}],
  ['invalid commit OID',x=>{x.merges[0].landed_sha='0'.repeat(40)}],
  ['wrong PR OID',x=>{x.merges[0].pr_head='A'.repeat(40)}],
  ['non-ancestor landed commit',x=>{x.merges[0].main_compare_behind_by=1}],
  ['invented review',x=>{x.merges[0].submitted_review_count=1}],
  ['invented branch protection',x=>{x.merges[0].main_protected=true}],
  ['per-PR false certification',x=>{x.merges[0].release_certification='PASS'}],
  ['zero-step success',x=>{x.merges[0].post_merge_ci_sample.status='success_stepful'}],
  ['stepful but skipped',x=>{x.merges[5].post_merge_ci_sample.executed_steps=0}],
  ['fake success without steps',x=>{x.merges[5].post_merge_ci_sample.status='not_executed_sample'}],
  ['unknown CI status',x=>{x.merges[0].post_merge_ci_sample.status='skipped'}],
  ['missing job identity',x=>{x.merges[0].post_merge_ci_sample.job_id=0}],
  ['wrong numeric type',x=>{x.merges[0].post_merge_ci_sample.run_id='1000'}],
  ['extra CI authority',x=>{x.merges[0].post_merge_ci_sample.approved=true}],
  ['repo impersonation',x=>{x.merges[0].repo='attacker/private'}],
  ['fewer than three repos in wave',x=>{for(let i=0;i<5;i++)x.merges[i].repo='ores-truffle-oreslang/oreslang-source.forge'}],
  ['unexpected mutable root field',x=>{x.can_merge=true}],
];
for(const [name,mutate] of negatives){
  test(`fail closed: ${name}`,()=>{const x=valid();mutate(x);assert.throws(()=>checkSnapshot(x))});
}
test('reject duplicate JSON keys even when parser would take the last value',()=>{
  const raw=canonical(valid()).replace('"state": "BLOCKED",', '"state": "CERTIFIED",\n  "state": "BLOCKED",');
  assert.throws(()=>parseCanonicalSnapshot(raw),/noncanonical/);
});
test('reject alternate whitespace, trailing payload and corrupted JSON',()=>{
  const raw=canonical(valid());
  assert.throws(()=>parseCanonicalSnapshot(JSON.stringify(valid())),/noncanonical/);
  assert.throws(()=>parseCanonicalSnapshot(raw+'garbage\n'));
  assert.throws(()=>parseCanonicalSnapshot('{"missing":true}\n'));
  assert.throws(()=>parseCanonicalSnapshot('x'.repeat(65537)+'\n'),/too large/);
});
test('accept regular immutable local file, deny symlink and hardlink',()=>{
  const dir=mkdtempSync(join(tmpdir(),'atto-release-hold-'));
  try{
    const p=join(dir,'snap.json');writeFileSync(p,canonical(valid()));
    // A plausible *synthetic* JSON snapshot must never pass the anchored
    // historical filesystem read, regardless of its structural validity.
    assert.throws(()=>readSnapshot(p),/pinned historical Git blob/);
    const alias=join(dir,'alias.json');symlinkSync(p,alias);
    assert.throws(()=>readSnapshot(alias),/unsafe/);
    const hard=join(dir,'hard.json');linkSync(p,hard);
    assert.throws(()=>readSnapshot(p),/unsafe/);
  }finally{rmSync(dir,{recursive:true,force:true})}
});

test('exact committed historical file is accepted only as release-BLOCKED',()=>{
  const original=fileURLToPath(new URL('../audits/merge-waves/2026-11-06-retroactive.json',import.meta.url));
  const r=readSnapshot(original);
  assert.equal(r.releaseAuthorized,false);
  assert.equal(r.state,'BLOCKED');
});
test('CLI keeps integrity (exit 0) separate from release permission (exit 1)',()=>{
  const cli=fileURLToPath(new URL('../tools/check-merge-wave-hold.mjs',import.meta.url));
  const normal=spawnSync(process.execPath,[cli],{encoding:'utf8',timeout:10000});
  assert.equal(normal.error,undefined);
  assert.equal(normal.status,0,normal.stderr);
  assert.match(normal.stdout,/AUDIT_INTEGRITY_ONLY: HOLD_ACTIVE/);
  assert.match(normal.stdout,/RELEASE_BLOCKED/);
  const release=spawnSync(process.execPath,[cli,'--release'],{encoding:'utf8',timeout:10000});
  assert.equal(release.error,undefined);
  assert.equal(release.status,1,release.stderr);
  assert.match(release.stdout,/RELEASE_BLOCKED/);
  const bad=spawnSync(process.execPath,[cli,'--unsafe-release'],{encoding:'utf8',timeout:10000});
  assert.equal(bad.error,undefined);
  assert.equal(bad.status,2);
  assert.match(bad.stderr,/INVALID_INPUT/);
});
test('whitespace-only editing of pinned historical snapshot is rejected',()=>{
  const dir=mkdtempSync(join(tmpdir(),'atto-release-anchored-'));
  try {
    const original=fileURLToPath(new URL('../audits/merge-waves/2026-11-06-retroactive.json',import.meta.url));
    const source=readFileSync(original,'utf8');
    const p=join(dir,'modified.json');
    writeFileSync(p,source.replace('"state": "BLOCKED"','"state": "CERTIFIED"'));
    assert.throws(()=>readSnapshot(p),/pinned historical Git blob/);
    writeFileSync(p,source.replace('"merge_count": 10','"merge_count": 10 '));
    assert.throws(()=>readSnapshot(p),/pinned historical Git blob/);
  } finally { rmSync(dir,{recursive:true,force:true}); }
});
