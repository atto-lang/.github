import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { parseSnapshotJSON } from '../tools/strict-receipt-json-v1.mjs';
import { main } from '../tools/verify-merge-receipt.mjs';

const accepted = (name, raw) => test(name, () =>
  assert.deepEqual(parseSnapshotJSON(raw), JSON.parse(raw)));
const rejected = (name, raw) => test(name, () =>
  assert.throws(() => parseSnapshotJSON(raw)));
accepted('valid nested objects and array retain JSON semantics', '{"checkout":{"sha":"abc"},"reviews":[{"author":"a"},{"author":"b"}]}');
accepted('same key under different objects is not a duplicate', '{"a":{"n":1},"b":{"n":2}}');
accepted('escaped Unicode property names decode correctly', '{"\\u0061":1,"b":2}');
accepted('primitive and escaped text values do not become property keys', '{"p":[null,false,123,"quoted \\n text"]}');
rejected('duplicated root key is unsafe', '{"schema":"wrong","schema":"atto.merge-receipt/v1"}');
rejected('escaped-equivalent duplicate key is unsafe', '{"main_protected":false,"main_\\u0070rotected":true}');
rejected('nested protection contradiction is unsafe', '{"protection":{"main_protected":false,"main_protected":true}}');
rejected('nested review author contradiction is unsafe', '{"reviews":[{"author":"author","author":"other"}]}');
rejected('prototype-shaped property is unsafe', '{"__proto__":{"admin":true}}');
rejected('constructor property is unsafe', '{"constructor":null}');
rejected('prototype property is unsafe', '{"prototype":null}');
rejected('trailing object comma is unsafe', '{"a":1,}');
rejected('trailing array comma is unsafe', '["a",]');
rejected('more than 64 nested levels is unsafe', '['.repeat(65) + '0' + ']'.repeat(65));
rejected('invalid escaped key is unsafe', '{"bad\\z":2}');
rejected('truncated object is unsafe', '{"a":');
rejected('two root values are unsafe', '{"a":1} {"b":2}');
rejected('more than 8192 keys is unsafe', '{' + Array.from({length:8193},(_,i)=>'"k'+i+'":0').join(',') + '}');

// For regressions, the exact same physical fd path enters main() as in normal CLI use.
const a='a'.repeat(40), b='b'.repeat(40), c='c'.repeat(40), d='d'.repeat(40);
const validReceipt=()=>({
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
function runCLI(snapshot) {
  const dir=mkdtempSync(join(tmpdir(), 'atto-strict-json-'));
  try {
    const file=join(dir,'snapshot.json');
    writeFileSync(file,snapshot);
    let printed;
    const exit=main([file], msg=>{printed=msg;});
    return {exit, outcome:JSON.parse(printed)};
  } finally { rmSync(dir,{recursive:true,force:true}); }
}
test('normal snapshot stays advisory with exit code 3, never 0',()=>{
  const r=runCLI(JSON.stringify(validReceipt()));
  assert.equal(r.exit,3);
  assert.equal(r.outcome.status,'SNAPSHOT_VALID_RECHECK_LIVE');
});
test('CLI refuses contradictory protected-main values in one snapshot',()=>{
  const raw=JSON.stringify(validReceipt()).replace('"main_protected":true','"main_protected":false,"main_protected":true');
  const r=runCLI(raw);
  assert.equal(r.exit,2);
  assert.equal(r.outcome.status,'BLOCKED');
});
test('CLI refuses escaped-duplicate protected-main values',()=>{
  const raw=JSON.stringify(validReceipt()).replace('"main_protected":true','"main_protected":false,"main_\\u0070rotected":true');
  const r=runCLI(raw);
  assert.equal(r.exit,2);
  assert.equal(r.outcome.status,'BLOCKED');
});
test('CLI refuses prototype-shaped key anywhere in receipt',()=>{
  const raw=JSON.stringify(validReceipt()).replace('"main_protected":true','"__proto__":{},"main_protected":true');
  const r=runCLI(raw);
  assert.equal(r.exit,2);
  assert.equal(r.outcome.status,'BLOCKED');
});

// Differential: valid JSON remains equivalent to Node's parser on 2,000 fixed-seed cases.
test('2,000 seeded valid JSON inputs remain equivalent',()=>{
  let seed=0x7a531202;
  const rand=n=>{seed=(seed*1664525+1013904223)>>>0;return seed%n;};
  const names=['a','a b','k\\u002f','x','quote"one','日本語','emoji☃','author','main_protected'];
  const key=()=>names[rand(names.length)];
  const value=(depth=0)=>{
    const z=rand(depth>7?4:7);
    if(z===0)return null;
    if(z===1)return rand(2)===0;
    if(z===2)return rand(5)-3;
    if(z===3)return key();
    if(z===4)return Array.from({length:rand(6)},()=>value(depth+1));
    const object={};
    for(let i=0,n=rand(8);i<n;i++)object[key()]=value(depth+1);
    return object;
  };
  for(let i=0;i<2000;i++){
    const raw=JSON.stringify(value());
    assert.deepEqual(parseSnapshotJSON(raw),JSON.parse(raw),'case '+i);
  }
});
