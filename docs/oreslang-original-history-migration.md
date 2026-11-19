# Full-history Oreslang compiler migration (nine backends)

Tracker: https://github.com/atto-lang/.github/issues/11

**Goal:** make each original `ores-truffle-oreslang/oreslang-source.<backend>` Git commit reachable from the matching `atto-lang/atto-source.<backend>` history. **Do not equate same file tree, cherry-picks, or copied blobs with original commit ancestry.** Retain the independent Atto branch's commits, and never force-push `main`.

## Pinned source tips at audit (2026-11-19)

| Backend | Legacy `main` expected SHA |
|---|---|
| cranelift | `3d0c297dffc29b8beb8fac2b365cfe197e428ea9` |
| llvm | `fe222285a4113783ed555079037faf11db7e1d3a` |
| js | `5c8cef088ee6734cd43589becd31a9b95ce75222` |
| wasm | `f122361f1a64be77fa741e48bc9f31671c3af7bf` |
| rs | `4d57a7927d9d7d8e67083f64e9b7c80611fcfa72` |
| net | `99d5f6183f5b10d473fa24e456ac4fb6354c5f8f` |
| forge | `3fbd03a342e50c4b4583a5f0bfaac296044a66a8` |
| java | `b4f6c7e11a7295e1ae46ac89a316bdaf93e54fea` |
| hdl | `a9fd278965d08b58378ff41237ccd19ab2e766c5` |

Before using a pinned tip, recheck whether upstream moved. If it did, choose the newer exact SHA intentionally, record the changed range and redo the diff/PR overlap audit.

## Prepare, without pushing

The operator must have legitimate read credentials for **both private repositories** in the normal Git credential helper/SSH agent; this procedure does not create or display tokens. From a clean local clone of each canonical Atto repository:

```bash
# Example: in the root of a local atto-lang/atto-source.hdl clone.
bash /path/to/atto-lang-.github/scripts/prepare-oreslang-history-merge.sh \
  hdl a9fd278965d08b58378ff41237ccd19ab2e766c5
```

The script verifies the expected origin; fetches **all legacy branch tips and tags**, plus current Atto main; checks the legacy main pin exactly; prepares a local migration branch and a real, **uncommitted, no-fast-forward merge with `--allow-unrelated-histories`**. On conflicts it stops without overriding either side. Nothing is pushed. **Only the legacy `main` tip is merged**; other branches and PRs must be reconciled with separate reviewed work.

If the script reports conflicts, resolve them semantically, especially overlapping runtime, API, CI and naming changes, while retaining original parentage. Keep the Atto main branch's current features and original security rules. For source-only legacy Python scripts, copied policy files, CI secrets assumptions or stale `.ores` paths, make explicit reviewed adaptations after import. Do not use `git merge -s ours`, `-X ours`, `-X theirs`, or destructive fast-forward/reset shortcuts to make the report pass.

## Completion gates

1. Every source main tip remains an original Git object and becomes an ancestor of the **new two-parent merge commit**, and subsequently of Atto main: `git merge-base --is-ancestor <pinned-source-sha> <atto-main-sha>` exits zero.
2. Inspect source branch refs, tags, open/draft PRs and recently merged source PRs. Track source-only commits not reachable from source main and their reviewed disposition. Do not automatically merge drafts or held PRs. Re-run against new source main tips until caught up.
3. Compare original and destination Git tree/blob SHA inventories. Preserve current Atto-only files/code while reviewing renamed or incompatible contracts. In-place source snapshots are **not** proof of preserved history.
4. Run applicable real compiler/guest tests, negative admission/security tests, Native AOT/runtime tests and exact-head CI on the resulting **Atto merge commit**. Do not count empty/failed/skipped Actions jobs as passes.
5. If source org Actions are unallocated, an alternate authorized CI carrier may receive a **token-free byte-verified** Git-tree mirror; retain source-private constraints and never insert access tokens into workflows, branches, archives or logs. A cross-org private checkout with an unrelated `GITHUB_TOKEN` is not an acceptable assumption.
6. Confirm actual current GitHub PR readiness-cycle age, independently authenticated reviewer approvals, labels, CODEOWNERS and branch protections. Reconcile overlapping Atto and Oreslang PRs before merging. No policy bypasses.
7. When approved, merge by **merge commit**, not squash/rebase, then independently re-fetch `atto-lang/main` and verify ancestral coverage at the exact published SHA.

## Known partial prior migration

- Wasm [PR #16](https://github.com/atto-lang/atto-source.wasm/pull/16) merged original legacy history through `088c637d72` (1,461 commits). Its imported cutoff is reachable from current Atto main, but 199 subsequent legacy source commits existed at the audited tip and the latter tip was not available from the destination repository at audit time.
- HDL [PR #3](https://github.com/atto-lang/atto-source.hdl/pull/3) is a **draft file-level import**, 39/39 exact blob SHAs, not an original history import. Initial destination GHA runs concluded failure without recorded executed steps; no passing test claim is available.
- Cranelift [PR #4](https://github.com/atto-lang/atto-source.cranelift/pull/4) copied 17 exact files, but did not establish complete source ancestry.
- Rust [PR #232](https://github.com/atto-lang/atto-source.rs/pull/232) imported a legacy compiler under `native/`; review its integration separately from preserving the complete Oreslang source Git graph.

No other pair should be marked fully migrated without an independent source-ancestry verification.


## Operator workspaces on macOS (2026-11-19 clarification)

The migration must use both existing local checkouts **without overwriting either**:

- Legacy: `~/codes/ores-truffle-oreslang/oreslang-source.<backend>`
- Canonical: `~/codes/atto-lang/atto-source.<backend>`

A local runner must retain **four** independent classes of original commits: published legacy main/branches, legacy local-only branches, published Atto main/branches, and Atto local-only branches. It must also fetch GitHub source PR head refs (`refs/pull/*/head`) and inventory source/destination PR metadata with pagination, including open/draft/closed/merged statuses. Preserve source PR URLs, discussions, and approvals as original provenance: GitHub does not support transferring their native identities to a different repository.

### Backup before touching branch integration

Before each fetch: create and verify a local `git bundle --all` for the two checkouts, and a **local-only** backup of any dirty/uncommitted worktree. After fetching both remotes, create additional Git bundles (remote history onto local). Protect **local committed-only** heads on the destination private remote with non-overwriting backup refs (local onto remote). For a large set of PRs, a remote archival branch may use a synthetic merge-parent chain whose **tree is exactly Atto main** and whose parents preserve original Git object IDs. This avoids checking out/pushing untrusted source workflows as the archive tree, but the archive is **backup only**, not semantic integration or proof of completion on Atto main. Verify every original ref OID is an ancestor of the fetched-back backup branch SHA. Do not publish uncommitted files, ignored data, tokens, or secrets to the remote.

Stage the **real** original-history merge in a separate Atto worktree with `git merge --no-commit --no-ff --allow-unrelated-histories`. On conflicts, preserve the worktree and inspect their meaning; do not automatically select `ours` or `theirs` or rewrite original Oreslang SHAs with rebase/squash. The existing Atto checkout, current branch, and `main` remain unchanged during staging.

An offline Bash fixture suite (separate local kit provided to the operator) passed for nine synthetic repository pairs, bidirectional local/remote backup preservation, exact original commit ancestry, unrelated-history conflicts, dirty working trees, and one source PR worktree preparation. This fixture evidence is **not** a claim of live private-repo Git execution or actual backend CI. The operator's local checkout is not mounted in the GitHub connector or ChatGPT cloud runtime.

### Review and destination PR handling

Source PR head objects can be archived without duplicating unreviewed PRs. Only create a destination Atto draft PR when the latest source main is an ancestor of canonical Atto main and the particular PR is semantically reconciled. Each destination PR should backlink to the original source PR, original head SHA and state, overlap disposition, exact-head CI evidence and independently authenticated reviews. A source PR with merge conflicts is **blocked**, not lost or silently skipped. Do not automatically merge old draft PRs or bypass readiness labels.

Follow completion tracking in [issue #11](https://github.com/atto-lang/.github/issues/11), and never equate a remote backup branch or file-level snapshot with fully migrated source history.
