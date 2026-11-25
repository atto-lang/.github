#!/usr/bin/env bash
# Prepare (do not push or merge) a history-preserving Oreslang -> Atto Git merge.
# Requires a local Git client whose existing credentials can READ BOTH private repos.
# No PAT, secret, or installation token should be placed in this script or its output.
set -euo pipefail

fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

if (( $# != 2 )); then
  printf 'Usage: %s <backend> <expected-40-char-legacy-main-SHA>\n' "$0" >&2
  exit 2
fi

backend="$1"
expected="$2"
case "$backend" in
  cranelift|llvm|js|wasm|rs|net|forge|java|hdl) ;;
  *) fail "Unrecognized backend suffix: $backend" ;;
esac
[[ "$expected" =~ ^[0-9a-f]{40}$ ]] || fail 'Expected source SHA must be exactly 40 lowercase hex characters.'

dest_repo="atto-lang/atto-source.$backend"
legacy_repo="ores-truffle-oreslang/oreslang-source.$backend"
legacy_url="https://github.com/$legacy_repo.git"
legacy_ref="refs/remotes/migration-oreslang/main"
target_ref="refs/remotes/origin/main"
branch="migration/legacy-history-$backend-$(printf '%s' "$expected" | cut -c1-12)"

git rev-parse --is-inside-work-tree >/dev/null || fail 'Run in a clone of the target Atto repo.'
root="$(git rev-parse --show-toplevel)"
[[ "$PWD" == "$root" ]] || fail 'Run at the root of the Atto repository.'
[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || fail 'Working tree is not clean. Nothing has been fetched or changed.'
origin="$(git remote get-url origin)"
case "$origin" in
  "https://github.com/$dest_repo"|"https://github.com/$dest_repo.git"|"git@github.com:$dest_repo"|"git@github.com:$dest_repo.git") ;;
  *) fail "Origin must be the matching canonical destination ($dest_repo); got $origin" ;;
esac

# No permissions are elevated, and no refs are ever pushed. Credentials come
# solely from the operator's existing Git credential helper/SSH configuration.
git fetch --no-tags origin '+refs/heads/main:refs/remotes/origin/main'
git fetch --no-tags "$legacy_url" '+refs/heads/*:refs/remotes/migration-oreslang/*'
git fetch --no-tags "$legacy_url" '+refs/tags/*:refs/tags/migration-oreslang/*'

actual="$(git rev-parse "$legacy_ref^{commit}")"
[[ "$actual" == "$expected" ]] || fail "Legacy main moved: expected $expected, found $actual; inspect changes and re-pin before proceeding."
target="$(git rev-parse "$target_ref^{commit}")"

printf 'Destination: %s @ %s\n' "$dest_repo" "$target"
printf 'Legacy:      %s @ %s\n' "$legacy_repo" "$actual"
printf 'Imported ancestor count remaining: '
git rev-list --count "$legacy_ref" --not "$target_ref"
printf 'Other legacy branch tips (must be triaged separately):\n'
git for-each-ref --format='%(refname:short) %(objectname)' refs/remotes/migration-oreslang

if git merge-base --is-ancestor "$expected" "$target"; then
  printf 'ALREADY AN ANCESTOR of destination main at the pinned source SHA.\n'
  printf 'Still audit open legacy PR branches, tags, and current source advances.\n'
  exit 0
fi

if git show-ref --verify --quiet "refs/heads/$branch"; then
  fail "Migration branch already exists: $branch. Inspect it rather than overwriting it."
fi
git switch --create "$branch" "$target"

printf 'Preparing ordinary two-parent merge; source history will remain intact.\n'
set +e
git -c merge.conflictStyle=zdiff3 merge --no-ff --no-commit --allow-unrelated-histories "$legacy_ref"
merge_status=$?
set -e

if (( merge_status != 0 )); then
  printf '\nMERGE CONFLICTS: resolve semantically; do NOT blindly choose ours/theirs.\n' >&2
  git diff --name-only --diff-filter=U >&2 || true
  printf '\nThe local migration branch is intentionally left in merge state.\n' >&2
  printf 'No changes were pushed; source and destination Git histories were not rewritten.\n' >&2
  exit 3
fi

merge_head_path="$(git rev-parse --git-path MERGE_HEAD)"
if [[ ! -f "$merge_head_path" ]]; then
  fail 'No merge in progress; inspect Git state before claiming migration.'
fi
[[ "$(git rev-parse MERGE_HEAD)" == "$expected" ]] || fail 'MERGE_HEAD does not match pinned legacy source SHA.'

printf '\nPrepared UNCOMMITTED history-preserving merge on branch %s.\n' "$branch"
printf 'Review the full staged diff, including additions from legacy source and security boundaries.\n'
printf 'Treat Python tooling, .github workflows, CODEOWNERS, secrets, naming and incompatible runtimes as blockers until reconciled.\n'
printf 'Never rewrite the imported history or use -s ours to feign a completed port.\n'
printf 'Run destination exact-head material tests and authenticated review before any PR merge.\n'
printf 'A human/review agent must finish conflict handling, create the merge commit, and push a review branch.\n'
printf 'Verify after commit and on final main: git merge-base --is-ancestor %s HEAD\n' "$expected"
printf 'Source branch tips outside main, including open draft PRs, require separate triage.\n'
