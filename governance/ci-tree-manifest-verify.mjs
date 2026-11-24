// Source-free, read-only Git tree integrity comparator.
// The caller must independently authenticate source/destination repository,
// immutable commit IDs, access permissions, Git object contents, and transfer
// policy. Identical trees are NOT permission to copy private source or proof
// that any compiler tests executed.
import fs from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const OID = /^[a-f0-9]{40}$/i;
const SAFE_BLOB_MODES = new Set(['100644', '100755']);
const MAX_ENTRIES = 50000;
const result = (status, reason, extra = {}) => ({
  status, reason, exact_tree_only: status === 'TREE_IDENTICAL_ONLY',
  private_export_approved: false, tests_executed: false, ...extra,
});
const sha = s => typeof s === 'string' && OID.test(s);
const validPath = p =>
  typeof p === 'string' && p.length > 0 && p.length <= 4096 &&
  !p.startsWith('/') && !p.endsWith('/') &&
  !p.includes('\\') && !p.includes('\0') &&
  !/[\x00-\x1f\x7f]/.test(p) &&
  p.split('/').every(x => x !== '' && x !== '.' && x !== '..');

function indexedTree(raw, expected, role) {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw) ||
      !Array.isArray(raw.tree) || raw.truncated !== false ||
      !sha(raw.sha) || raw.sha.toLowerCase() !== expected.toLowerCase() ||
      raw.tree.length > MAX_ENTRIES) {
    throw new Error(role + ': incomplete, truncated, oversized, or wrong-root Git tree');
  }
  const entries = new Map();
  const dirs = new Set();
  const blobs = new Set();
  for (const entry of raw.tree) {
    if (!entry || typeof entry !== 'object' || Array.isArray(entry) ||
        !validPath(entry.path) || !sha(entry.sha)) {
      throw new Error(role + ': malformed Git tree entry');
    }
    const type = entry.type, mode = entry.mode;
    if (type === 'tree') {
      if (mode !== '040000') throw new Error(role + ': invalid directory mode');
      dirs.add(entry.path);
    } else if (type === 'blob') {
      if (!SAFE_BLOB_MODES.has(mode)) {
        throw new Error(role + ': symlink, gitlink, or unsupported blob mode');
      }
      blobs.add(entry.path);
    } else {
      throw new Error(role + ': unsupported Git object type');
    }
    if (entries.has(entry.path)) throw new Error(role + ': duplicate path');
    entries.set(entry.path, {
      type, mode, oid: entry.sha.toLowerCase(),
    });
  }

  for (const [path, entry] of entries) {
    const parts = path.split('/');
    for (let i = 1; i < parts.length; i++) {
      const parent = parts.slice(0, i).join('/');
      if (blobs.has(parent)) throw new Error(role + ': file contains descendant path');
      // The recursive tree listing must include the parent directory itself.
      if (!dirs.has(parent)) throw new Error(role + ': missing parent tree entry');
    }
    if (entry.type === 'tree') {
      // Directory has to be real, but Git empty trees are legitimate and
      // carry their own cryptographic OID.
      if (!sha(entry.oid)) throw new Error(role + ': invalid directory object');
    }
  }
  return { entries, blobCount: blobs.size, directoryCount: dirs.size };
}

export function compareExactGitTrees({ sourceTree, carrierTree, expectedSourceRoot, expectedCarrierRoot }) {
  if (!sha(expectedSourceRoot) || !sha(expectedCarrierRoot)) {
    return result('MIRROR_INVALID', 'missing immutable source or carrier root SHA');
  }
  try {
    const source = indexedTree(sourceTree, expectedSourceRoot, 'source');
    const carrier = indexedTree(carrierTree, expectedCarrierRoot, 'carrier');
    const stats = { source_root: expectedSourceRoot.toLowerCase(),
      carrier_root: expectedCarrierRoot.toLowerCase(),
      source_blob_count: source.blobCount,
      carrier_blob_count: carrier.blobCount,
      source_directory_count: source.directoryCount,
      carrier_directory_count: carrier.directoryCount,
    };
    if (expectedSourceRoot.toLowerCase() !== expectedCarrierRoot.toLowerCase()) {
      return result('MIRROR_INVALID', 'root tree OID mismatch', stats);
    }
    if (source.entries.size !== carrier.entries.size) {
      return result('MIRROR_INVALID', 'path counts differ', stats);
    }
    for (const [path, entry] of source.entries) {
      const other = carrier.entries.get(path);
      if (!other || entry.type !== other.type ||
          entry.mode !== other.mode || entry.oid !== other.oid) {
        return result('MIRROR_INVALID', 'path/mode/blob OID mismatch', stats);
      }
    }
    // This is an exact-object integrity check only. It does not verify
    // GitHub API authenticity, export permission, LFS payloads, executed
    // tests or valid independent security approval.
    return result('TREE_IDENTICAL_ONLY', 'all path/mode/object OIDs and root tree identical; no export or test authorization', {
      ...stats, verified_blob_count: source.blobCount, lfs_payloads_verified: false,
    });
  } catch (error) {
    return result('MIRROR_INVALID', error.message);
  }
}

const invoked = process.argv[1];
if (invoked && import.meta.url === pathToFileURL(resolve(invoked)).href) {
  try {
    if (process.argv.length !== 3) throw new Error('usage: ci-tree-manifest-verify.mjs <comparison.json>');
    const data = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
    const verdict = compareExactGitTrees(data);
    console.log(JSON.stringify(verdict));
    process.exitCode = verdict.status === 'TREE_IDENTICAL_ONLY' ? 0 : 2;
  } catch (error) {
    console.error('MIRROR_INVALID: ' + error.message);
    process.exitCode = 2;
  }
}
