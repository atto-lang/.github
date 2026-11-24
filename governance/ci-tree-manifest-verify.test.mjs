import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { compareExactGitTrees } from './ci-tree-manifest-verify.mjs';

const ROOT = 'a'.repeat(40), OTHER = 'b'.repeat(40), BLOB = 'c'.repeat(40);
const file = { path: 'docs/a.md', type: 'blob', mode: '100644', sha: BLOB };
const tree = { path: 'docs', type: 'tree', mode: '040000', sha: OTHER };
const fixture = () => ({
  expectedSourceRoot: ROOT, expectedCarrierRoot: ROOT,
  sourceTree: { sha: ROOT, tree: [tree, file], truncated: false },
  carrierTree: { sha: ROOT, tree: [tree, file], truncated: false },
});
const check = x => compareExactGitTrees(x);

test('identical source and carrier trees give integrity-only result', () => {
  const result = check(fixture());
  assert.equal(result.status, 'TREE_IDENTICAL_ONLY');
  assert.equal(result.verified_blob_count, 1);
  assert.equal(result.private_export_approved, false);
  assert.equal(result.tests_executed, false);
  assert.equal(result.lfs_payloads_verified, false);
});
test('different root tree SHA is rejected', () => {
  const x = fixture(); x.expectedCarrierRoot = OTHER; x.carrierTree.sha = OTHER;
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('missing expected immutable root SHA is rejected', () => {
  const x = fixture(); delete x.expectedSourceRoot;
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('reported source root must match authenticated expected source root', () => {
  const x = fixture(); x.sourceTree.sha = OTHER;
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('truncated source or destination trees fail closed', () => {
  for (const role of ['sourceTree', 'carrierTree']) {
    const x = fixture(); x[role].truncated = true;
    assert.equal(check(x).status, 'MIRROR_INVALID');
  }
});
test('missing truncated flag is insufficient evidence', () => {
  const x = fixture(); delete x.carrierTree.truncated;
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('missing path cannot pass even with matching claimed root SHA', () => {
  const x = fixture(); x.carrierTree.tree = [tree];
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('different blob object OID is rejected', () => {
  const x = fixture(); x.carrierTree.tree = [tree, { ...file, sha: OTHER }];
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('mode mismatch is rejected even if blob OID same', () => {
  const x = fixture(); x.carrierTree.tree = [tree, { ...file, mode: '100755' }];
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('same paths in different listing order are equivalent', () => {
  const x = fixture(); x.carrierTree.tree = [file, tree];
  assert.equal(check(x).status, 'TREE_IDENTICAL_ONLY');
});
test('duplicate paths are fail-closed', () => {
  const x = fixture(); x.sourceTree.tree.push(file);
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('directory mode and type are checked strictly', () => {
  const x = fixture(); x.carrierTree.tree = [{ ...tree, mode: '100644' }, file];
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('symlinks and submodules cannot silently transfer', () => {
  for (const [type, mode] of [['blob', '120000'], ['commit', '160000']]) {
    const x = fixture(); x.sourceTree.tree = [tree, { ...file, type, mode }];
    assert.equal(check(x).status, 'MIRROR_INVALID');
  }
});
test('invalid or traversing file names are rejected', () => {
  for (const path of ['../outside', 'docs//a.md', '/abs', 'docs/./a.md', 'docs/x\nbad', 'docs\\bad']) {
    const x = fixture(); x.sourceTree.tree = [tree, { ...file, path }];
    assert.equal(check(x).status, 'MIRROR_INVALID', path);
  }
});
test('parent directories must be present in recursive tree', () => {
  const x = fixture(); x.sourceTree.tree = [file];
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('file pretending to be parent of nested file is rejected', () => {
  const x = fixture(); x.sourceTree.tree = [
    { path: 'docs', type: 'blob', mode: '100644', sha: OTHER }, file,
  ];
  assert.equal(check(x).status, 'MIRROR_INVALID');
});
test('invalid SHA or unsupported Git type fail closed', () => {
  for (const change of [{ sha: 'xyz' }, { type: 'tag' }]) {
    const x = fixture(); x.carrierTree.tree = [tree, { ...file, ...change }];
    assert.equal(check(x).status, 'MIRROR_INVALID');
  }
});
test('nonobject or absent tree evidence is never valid', () => {
  for (const bad of [null, [], {}, { sha: ROOT, truncated: false }]) {
    const x = fixture(); x.carrierTree = bad;
    assert.equal(check(x).status, 'MIRROR_INVALID');
  }
});
test('formatting is stable for all-lowercase OIDs', () => {
  const x = fixture();
  x.sourceTree.sha = ROOT.toUpperCase();
  x.carrierTree.sha = ROOT.toUpperCase();
  assert.equal(check(x).status, 'TREE_IDENTICAL_ONLY');
});
test('empty cryptographically equal trees remain integrity-only', () => {
  const x = fixture(); x.sourceTree.tree = []; x.carrierTree.tree = [];
  assert.equal(check(x).status, 'TREE_IDENTICAL_ONLY');
  assert.equal(check(x).verified_blob_count, 0);
});

const script = fileURLToPath(new URL('./ci-tree-manifest-verify.mjs', import.meta.url));
const relativeScript = relative(process.cwd(), script);
function execute(data, raw = false) {
  const dir = mkdtempSync(join(tmpdir(), 'atto-tree-verify-'));
  const path = join(dir, 'trees.json');
  try {
    writeFileSync(path, raw ? data : JSON.stringify(data));
    return spawnSync(process.execPath, [relativeScript, path], {
      cwd: process.cwd(), encoding: 'utf8',
    });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}
test('real CLI on normal relative path prints evidence but never export approval', () => {
  const result = execute(fixture());
  assert.equal(result.status, 0, result.stderr);
  const response = JSON.parse(result.stdout);
  assert.equal(response.status, 'TREE_IDENTICAL_ONLY');
  assert.equal(response.private_export_approved, false);
});
test('real CLI exits nonzero when a destination blob differs', () => {
  const x = fixture(); x.carrierTree.tree = [tree, { ...file, sha: OTHER }];
  const result = execute(x);
  assert.equal(result.status, 2);
  assert.equal(JSON.parse(result.stdout).status, 'MIRROR_INVALID');
});
test('invalid CLI JSON fails closed without silent zero exit', () => {
  const result = execute('{broken', true);
  assert.equal(result.status, 2);
  assert.match(result.stderr, /MIRROR_INVALID/);
});
