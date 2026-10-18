# Safe organization Git sync

From the parent of your repositories, run `.github/scripts/sync-gh-org.sh` after pulling the `ores-truffle-oreslang/.github` repository.

```bash
# Nothing changes locally or remotely, including log creation.
./.github/scripts/sync-gh-org.sh ores-truffle-oreslang --dry-run

# Commit pending nonignored code, back up local and remote refs, sync one repo.
./.github/scripts/sync-gh-org.sh ores-truffle-oreslang --repo oreslang-source.java \
  --checkpoint --backup-remote --verify --report

# Full organization only after reviewing the plan.
./.github/scripts/sync-gh-org.sh ores-truffle-oreslang --all \
  --checkpoint --backup-remote --verify --report

# Pass a different parent directory explicitly:
./.github/scripts/sync-gh-org.sh ores-truffle-oreslang --root ~/src/ores-truffle-oreslang --dry-run
```

## Preservation rules

- No `git reset`, `git clean`, `git push --force`, `git stash drop`, or remote ref deletion.
- `--dry-run` invokes read-only Git commands and GitHub queries; it does not clone or fetch.
- Dirty worktrees require `--checkpoint`. The script commits tracked and unignored untracked changes (does not add ignored files), and refuses to proceed when an unfinished Git operation or suspect untracked secret file path is present. **Review staged/untracked content for secrets before using `--checkpoint`.**
- Local branches receive timestamped `sync/local/<run>/bN` snapshots, existing stashes receive `sync/stash/<SHA>` branches, and detached worktrees receive named `sync/detached/<run>/wN` branches. These refs are pushed and verified before integration.
- Local tags, including annotated tags, are snapshotted to `refs/tags/sync-local/<run>/tN` before ordinary tag pushes.
- `--backup-remote` snapshots current remote branches to `refs/heads/sync/remote/<run>/rN` and tags to `refs/tags/sync-remote/<run>/rN`, including annotated tag objects. The report lists their original refs and SHAs.
- Remote backups are confirmed before merging. Existing local branch histories are never rewritten. Divergent branches use regular merge commits. Non-checked-out local branches are integrated in temporary worktrees.
- Conflicts remain in place for *semantic* review; the script cannot safely resolve arbitrary conflicting implementations. Temporary worktrees with conflicts stay under `.org-sync/worktrees/<run>`.
- Rejected pushes, failed backups or mismatched verification make the overall command exit nonzero. Other repositories continue independently. All results are reported in `.org-sync/reports/` when `--report` is enabled.

## Important limitations

This is a preservation-first synchronizer, **not an infallible backup product**. Files ignored by Git, unreferenced dangling objects, inaccessible organizations, submodule payloads, uncommitted changes in nested submodules, GitHub pull-request refs, and repositories outside the scanned root/depth cannot be guaranteed remotely preserved. It uses `find -maxdepth 5`; clone locations deeper than that should be moved or handled separately. The target must be a GitHub organization on github.com, and `gh` must be logged in. Clone discovery requires existing `origin` URLs to refer to `github.com/<org>/<repo>`; Git URL rewrites still work because discovery reads configured URLs.

Commits may run project hooks and can fail. Use your usual compiler/test suite after automatically clean merges. Pushing code can trigger CI, branch restrictions, and secret scanning. `--verify` verifies Git reference object IDs, not source correctness or that CI passed. If a branch is protected or a tag name collides, preserve both versions and resolve the policy conflict manually; never force a push. Running this script may create a large number of `sync/` backup branches and tags, which are intentionally retained until reviewed.
