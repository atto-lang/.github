# Safe organization Git sync

Run `scripts/sync-gh-org.sh` from the parent of your repositories; it is maintained in `ores-truffle-oreslang/.github`.

```bash
# Nothing changes locally or remotely, including log creation.
./scripts/sync-gh-org.sh ores-truffle-oreslang --dry-run

# Commit pending nonignored code, back up local and remote refs, sync one repo.
./scripts/sync-gh-org.sh ores-truffle-oreslang --repo oreslang-source.java \
  --checkpoint --backup-remote --verify --report

# Full organization only after reviewing the plan.
./scripts/sync-gh-org.sh ores-truffle-oreslang --all \
  --checkpoint --backup-remote --verify --report

# Pass a different parent directory explicitly:
./scripts/sync-gh-org.sh ores-truffle-oreslang --root ~/src/ores-truffle-oreslang --dry-run
```

## Preservation rules

- No `git reset`, `git clean`, `git push --force`, `git stash drop`, or remote ref deletion.
- `--dry-run` invokes read-only Git commands and GitHub queries; it does not clone or fetch.
- Dirty worktrees require `--checkpoint`. The script commits tracked and unignored untracked changes (does not add ignored files), and refuses to proceed when an unfinished Git operation or suspect untracked secret file path is present. **Review staged/untracked content for secrets before using `--checkpoint`.**
- Freshly fetched `origin/main` is the ancestry reference: local branches already contained in `main`, or already published at the same SHA on their corresponding remote, do **not** receive redundant `sync/local/<run>/bN` snapshots. Local branches with unpublished commits do. Existing stashes receive `sync/stash/<SHA>` branches, and detached worktrees receive named `sync/detached/<run>/wN` branches. Newly selected safety refs are pushed and verified before integration.
- Local tags, including annotated tags, are snapshotted to `refs/tags/sync-local/<run>/tN` **only if their exact tag-object SHA is not already published at the same remote tag name**. All tag pushes remain non-force.
- `--backup-remote` backs up only remote branch tips that have an existing local branch which could advance or merge them **and** that are not already ancestors of the freshly fetched remote `main`. Remote branches with no corresponding local branch and remote tags are not changed by this tool, so they need no duplicate safety refs. Existing historical `sync/*` backups are never removed. The report lists selected backups and skipped-ref counts.
- Changes to **unrelated remote branches or tags do not abort** a sync. The tool logs `CHURN` and adopts the latest remote snapshot. Only branches the sync may actually advance get individual fetch-and-live-tip checks (with three bounded attempts); if a selected target keeps moving, that target fails closed rather than risking code loss. Non-force Git pushes still reject late non-fast-forward updates.
- `origin/*` is a cache, not proof of live state: a branch is skipped as already synchronized only after checking the current remote SHA.
- Selected remote backups are confirmed before merging. Existing local branch histories are never rewritten. Divergent branches use regular merge commits. Non-checked-out local branches are integrated in temporary worktrees. Squash-/rebase-merged commits are **not** assumed fully merged by name or PR status; only verified Git ancestry counts as merged.
- Conflicts remain in place for *semantic* review; the script cannot safely resolve arbitrary conflicting implementations. Temporary worktrees with conflicts stay under `.org-sync/worktrees/<run>`.
- Rejected pushes, failed backups or mismatched verification make the overall command exit nonzero. Other repositories continue independently. All results are reported in `.org-sync/reports/` when `--report` is enabled.

## Important limitations

This is a preservation-first synchronizer, **not an infallible backup product**. Files ignored by Git, unreferenced dangling objects, inaccessible organizations, submodule payloads, uncommitted changes in nested submodules, GitHub pull-request refs, and repositories outside the scanned root/depth cannot be guaranteed remotely preserved. It uses `find -maxdepth 5`; clone locations deeper than that should be moved or handled separately. The target must be a GitHub organization on github.com, and `gh` must be logged in. Clone discovery requires existing `origin` URLs to refer to `github.com/<org>/<repo>`; Git URL rewrites still work because discovery reads configured URLs.

Commits may run project hooks and can fail. Use your usual compiler/test suite after automatically clean merges. Pushing code can trigger CI, branch restrictions, and secret scanning. `--verify` verifies Git reference object IDs, not source correctness or that CI passed. If a branch is protected or a tag name collides, preserve both versions and resolve the policy conflict manually; never force a push. Earlier script versions may have created many `sync/` backup branches and tags; this version reduces **new** redundant backups but intentionally leaves historical safety refs in place. Do not delete them until every source SHA is confirmed reachable from another retained ref. The command does not test project CI or fully resolve semantic conflicts.
