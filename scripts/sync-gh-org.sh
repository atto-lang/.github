#!/usr/bin/env bash
# Organization-wide, no-force, preservation-first Git synchronizer.
# macOS Bash 3.2+; requires git, gh (authenticated), and standard POSIX utilities.
# Run --dry-run first. Unresolved semantic conflicts are deliberately left for review.
set -Eeuo pipefail

usage() {
  cat <<'EOF'
Usage: sync-gh-org.sh ORG [--root DIRECTORY] [--repo NAME | --all]
                      [--dry-run] [--checkpoint] [--backup-remote]
                      [--verify] [--report] [--strict]

  --dry-run        Read-only plan; no clones, fetch, files, refs, logs or commits.
  --repo NAME      Only a specific org repository (including linked worktrees).
  --all            Include all accessible, non-archived org repositories.
  --root DIRECTORY Parent of the org clones (default: current directory).
  --checkpoint     Commit all pending tracked/untracked nonignored changes.
  --backup-remote  Preserve remote branch refs that may change, not merged/untouched refs.
  --verify         Compare remote refs with intended pushed refs by SHA.
  --report         Write Markdown and tab-separated reports in .org-sync/reports/.
  --strict         Fail closed (default); retained for CLI compatibility.

Mutating runs require --repo or --all. Backups must push and verify BEFORE
merging. Never uses git reset, clean, stash drop, branch deletion, or force push.
EOF
}
(( $# >= 1 )) || { usage >&2; exit 2; }
ORG=$1; shift
[[ "$ORG" =~ ^[a-zA-Z0-9][a-zA-Z0-9-]*$ ]] || { echo 'Invalid org' >&2; exit 2; }
ROOT=$PWD; REPO=""; ALL=0; DRY=0; CHECKPOINT=0; BACKUP_REMOTE=0; VERIFY=0; REPORT=0
while (( $# )); do
  case "$1" in
    --root) (( $# >= 2 )) || { usage >&2; exit 2; }; ROOT=$2; shift 2 ;;
    --repo) (( $# >= 2 )) || { usage >&2; exit 2; }; REPO=$2; shift 2 ;;
    --all) ALL=1; shift ;;
    --dry-run) DRY=1; shift ;;
    --checkpoint) CHECKPOINT=1; shift ;;
    --backup-remote) BACKUP_REMOTE=1; shift ;;
    --verify) VERIFY=1; shift ;;
    --report) REPORT=1; shift ;;
    --strict) shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done
[[ -z "$REPO" || "$ALL" == 0 ]] || { echo '--repo and --all are exclusive' >&2; exit 2; }
[[ -z "$REPO" || "$REPO" =~ ^[a-zA-Z0-9_.-]+$ ]] || { echo 'Invalid repo name' >&2; exit 2; }
if (( DRY == 0 && ALL == 0 )) && [[ -z "$REPO" ]]; then
  echo 'Specify --all or --repo for mutation. --dry-run alone inspects all.' >&2; exit 2
fi
for cmd in git gh awk find grep date; do
  command -v "$cmd" >/dev/null || { echo "Missing $cmd" >&2; exit 2; }
done
[[ -d "$ROOT" ]] || { echo "Root directory does not exist: $ROOT" >&2; exit 2; }
ROOT=$(cd "$ROOT" && pwd -P)
# Nothing under ROOT is created in dry-run mode.
RUN="$(date -u +%Y%m%dT%H%M%SZ)-$$"
RUNDIR="$ROOT/.org-sync/reports"
TMPBASE="$ROOT/.org-sync/worktrees/$RUN"
ERRORS=0; REPO_ERRORS=0
EXPECTED=""; REPORT_TSV=""; REPORT_MD=""; LOG=""
if (( ! DRY )); then
  mkdir -p "$RUNDIR" "$TMPBASE"
  EXPECTED="$RUNDIR/$RUN-expected.tsv"
  REPORT_TSV="$RUNDIR/$RUN.tsv"
  REPORT_MD="$RUNDIR/$RUN.md"
  LOG="$RUNDIR/$RUN.log"
  : > "$EXPECTED"; : > "$LOG"; : > "$REPORT_TSV"
  printf 'repository\tstatus\tdetails\n' >> "$REPORT_TSV"
fi
say() { printf '%s\n' "$*"; if (( ! DRY )); then printf '%s\n' "$*" >> "$LOG"; fi; }
record() {
  local name=$1 status=$2 detail=$3
  say "[$name] $status: $detail"
  if (( ! DRY )); then
    # Single-line fields for a simple TSV export.
    detail=${detail//$'\t'/ }; detail=${detail//$'\n'/ }
    printf '%s\t%s\t%s\n' "$name" "$status" "$detail" >> "$REPORT_TSV"
  fi
}
problem() { ERRORS=$((ERRORS+1)); REPO_ERRORS=$((REPO_ERRORS+1)); record "$1" 'BLOCKED' "$2"; }
expected() { (( DRY )) || printf '%s\t%s\n' "$1" "$2" >> "$EXPECTED"; }
# git ls-remote output is SHA<TAB>ref. Verify exact values with no regex interpolation.
remote_sha() {
  awk -F '\t' -v k="$2" '$2 == k {print $1; exit}' "$1"
}
fetch_remote_listing() { git -C "$1" ls-remote origin 'refs/heads/*' 'refs/tags/*'; }
# Compare only mutable source refs, never safety refs intentionally created.
ordinary_remote_listing() {
  fetch_remote_listing "$1" | awk -F '\t' '$2 !~ /^refs\/heads\/sync\// && $2 !~ /^refs\/tags\/sync-(local|remote)\//'
}
# A branch tip already reachable from freshly fetched origin/main needs no
# duplicate backup. This deliberately uses ancestry, not PR merge status:
# squash/rebase merges cannot be proved redundant by ancestry alone.
contained_in_main() {
  git -C "$1" show-ref --verify -q refs/remotes/origin/main &&
    git -C "$1" merge-base --is-ancestor "$2" refs/remotes/origin/main
}
# An ordinary remote ref is at risk only if this run may update its target.
# No code path force-pushes, deletes refs, or overwrites existing tags.
remote_branch_needs_backup() {
  local repo=$1 remote_branch=$2 remote_tip=$3 local_tip
  git -C "$repo" show-ref --verify -q "refs/heads/$remote_branch" || return 1
  local_tip=$(git -C "$repo" rev-parse "refs/heads/$remote_branch") || return 1
  [[ "$local_tip" != "$remote_tip" ]] || return 1
  # Local tip behind remote will only fast-forward locally; push is unchanged.
  git -C "$repo" merge-base --is-ancestor "$local_tip" "$remote_tip" && return 1
  return 0
}
verify_expected() {
  local name=$1 repo=$2 label=$3 expected_file=$4 live=$5 sha ref got failed=0
  if ! fetch_remote_listing "$repo" > "$live"; then
    problem "$name" "Cannot verify remote refs ($label)"; return 1
  fi
  while IFS=$'\t' read -r sha ref; do
    [[ -n "$ref" ]] || continue
    got=$(remote_sha "$live" "$ref")
    if [[ "$got" != "$sha" ]]; then
      failed=1; record "$name" 'MISMATCH' "$label $ref expected=$sha remote=${got:-MISSING}"
    fi
  done < "$expected_file"
  if (( failed )); then problem "$name" "Remote verification failed ($label)"; return 1; fi
  record "$name" 'VERIFIED' "$label refs match"; return 0
}
# Filter only GitHub origin URLs belonging to the requested organization.
org_repo_from_url() {
  local url=$1 slug
  case "$url" in
    git@github.com:*) slug=${url#git@github.com:} ;;
    ssh://git@github.com/*) slug=${url#ssh://git@github.com/} ;;
    https://github.com/*) slug=${url#https://github.com/} ;;
    *) return 1 ;;
  esac
  slug=${slug%.git}
  case "$slug" in
    "$ORG"/*) printf '%s' "${slug#"$ORG"/}" ;;
    *) return 1 ;;
  esac
}
# For each repository, discover all standalone clones and linked worktrees.
# .git directories are pruned so find does not traverse Git object databases.
ROOT_REPOS=(); ROOT_NAMES=(); COMMON_DIRS=()
discover_local() {
  local marker dir url name common prior found
  while IFS= read -r marker; do
    [[ "${marker##*/}" == '.git' ]] || continue
    dir=${marker%/.git}
    url=$(git -C "$dir" config --get remote.origin.url 2>/dev/null || true)
    name=$(org_repo_from_url "$url" 2>/dev/null || true)
    [[ -n "$name" && "$name" != */* ]] || continue
    [[ -z "$REPO" || "$REPO" == "$name" ]] || continue
    common=$(git -C "$dir" rev-parse --path-format=absolute --git-common-dir 2>/dev/null || git -C "$dir" rev-parse --git-common-dir 2>/dev/null || true)
    [[ -n "$common" ]] || continue
    if [[ "$common" != /* ]]; then common=$(cd "$dir" && cd "$common" && pwd -P) || continue; fi
    found=0
    for prior in "${COMMON_DIRS[@]+"${COMMON_DIRS[@]}"}"; do
      if [[ "$prior" == "$common" ]]; then found=1; break; fi
    done
    (( found )) && continue
    COMMON_DIRS+=("$common"); ROOT_REPOS+=("$dir"); ROOT_NAMES+=("$name")
  done < <(find "$ROOT" -maxdepth 5 \( -name .git -o -name .org-sync \) -prune -print)
}
# Read names from gh without creating temporary files for dry-run.
remote_names() {
  gh api --paginate "/orgs/$ORG/repos?type=all&per_page=100" \
    --jq '.[] | select(.archived == false) | .name'
}
remote_exists() { gh repo view "$ORG/$1" --json name --jq .name >/dev/null 2>&1; }

# A read-only plan deliberately does NOT call git fetch, pull, add, commit,
# checkout, stash, push, clone, or mkdir.
plan_repo() {
  local name=$1 dir=$2 wt b dirty count=0 sha ref kind localsha relation
  local remotes="" branch tag
  record "$name" 'PLAN' "location=$dir"
  if [[ ! -e "$dir/.git" ]]; then record "$name" 'PLAN' 'clone missing remote repository'; return; fi
  while IFS= read -r wt; do
    [[ "$wt" == worktree\ * ]] || continue
    wt=${wt#worktree }
    dirty=$(GIT_OPTIONAL_LOCKS=0 git -C "$wt" status --porcelain=v1 --untracked-files=all 2>/dev/null || true)
    b=$(git -C "$wt" branch --show-current 2>/dev/null || true)
    record "$name" 'PLAN' "worktree=$wt branch=${b:-DETACHED} dirty=$([[ -n "$dirty" ]] && echo yes || echo no)"
    if has_unfinished_operation "$wt"; then record "$name" 'BLOCKER' "unfinished Git operation: $wt"; fi
  done < <(git -C "$dir" worktree list --porcelain)
  count=$(git -C "$dir" stash list --format='%H' | awk 'END{print NR+0}')
  record "$name" 'PLAN' "stashes=$count; will back up each stash and branch tip before integration"
  # This reads live remote refs but does not download Git objects or mutate refs.
  remotes=$(git -C "$dir" ls-remote --heads --tags origin) || {
    record "$name" 'UNKNOWN' 'remote access failed'; ERRORS=$((ERRORS+1)); return;
  }
  while IFS=$'\t' read -r branch localsha; do
    [[ -n "$branch" && "$branch" != sync/* ]] || continue
    sha=$(printf '%s\n' "$remotes" | awk -F '\t' -v k="refs/heads/$branch" '$2==k{print $1;exit}')
    if [[ -z "$sha" ]]; then relation='new remote branch'
    elif [[ "$sha" == "$localsha" ]]; then relation='identical'
    elif git -C "$dir" cat-file -e "$sha^{commit}" 2>/dev/null; then
      if git -C "$dir" merge-base --is-ancestor "$localsha" "$sha"; then relation='fast-forward possible'
      elif git -C "$dir" merge-base --is-ancestor "$sha" "$localsha"; then relation='push local commits'
      else relation='diverged; merge/conflict possible'; fi
    else relation='remote commit not fetched; integration outcome unknown'; fi
    record "$name" 'PLAN' "branch=$branch local=$localsha remote=${sha:-MISSING} action=$relation"
  done < <(git -C "$dir" for-each-ref --format='%(refname:short)%09%(objectname)' refs/heads)
  while IFS=$'\t' read -r tag localsha; do
    [[ -n "$tag" ]] || continue
    sha=$(printf '%s\n' "$remotes" | awk -F '\t' -v k="refs/tags/$tag" '$2==k{print $1;exit}')
    if [[ -n "$sha" && "$sha" != "$localsha" ]]; then
      record "$name" 'PLAN' "tag collision: $tag local=$localsha remote=$sha (preserve both, no overwrite)"
    fi
  done < <(git -C "$dir" for-each-ref --format='%(refname:strip=2)%09%(objectname)' refs/tags)
  if (( BACKUP_REMOTE )); then record "$name" 'PLAN' 'backup remote refs before integration'; fi
  if (( VERIFY )); then record "$name" 'PLAN' 'verify published refs by SHA'; fi
}

has_unfinished_operation() {
  local wt=$1 p
  [[ -n "$(git -C "$wt" ls-files -u)" ]] && return 0
  for p in MERGE_HEAD CHERRY_PICK_HEAD REVERT_HEAD REBASE_HEAD; do
    [[ -f "$(git -C "$wt" rev-parse --git-path "$p")" ]] && return 0
  done
  for p in rebase-merge rebase-apply sequencer; do
    [[ -d "$(git -C "$wt" rev-parse --git-path "$p")" ]] && return 0
  done
  return 1
}
# Detect recognizable private-key/config paths before staging untracked files.
# This is not a substitute for secret-scanning content and repository policy.
sensitive_untracked() {
  local wt=$1 f
  while IFS= read -r f; do
    case "${f##*/}" in
      .env|.env.*|*.pem|*.p12|*.pfx|*.key|id_rsa|id_ed25519|credentials.json|service-account.json)
        return 0 ;;
    esac
  done < <(git -C "$wt" ls-files --others --exclude-standard)
  return 1
}
checkpoint_worktrees() {
  local name=$1 dir=$2 line wt state current i=0
  while IFS= read -r line; do
    [[ "$line" == worktree\ * ]] || continue
    wt=${line#worktree }; i=$((i+1))
    [[ -d "$wt" ]] || { problem "$name" "missing worktree $wt"; return 1; }
    has_unfinished_operation "$wt" && { problem "$name" "unfinished Git operation in $wt"; return 1; }
    state=$(GIT_OPTIONAL_LOCKS=0 git -C "$wt" status --porcelain=v1 --untracked-files=all) || { problem "$name" "status failed for $wt"; return 1; }
    if [[ -n "$state" ]]; then
      if (( ! CHECKPOINT )); then problem "$name" "dirty worktree; rerun with --checkpoint: $wt"; return 1; fi
      if sensitive_untracked "$wt"; then problem "$name" "potential secret in untracked paths: $wt (review before checkpoint)"; return 1; fi
      if ! git -C "$wt" add -A; then problem "$name" "git add failed: $wt"; return 1; fi
      if ! git -C "$wt" diff --cached --quiet; then
        if ! git -C "$wt" commit -m "sync: checkpoint $RUN"; then
          problem "$name" "commit failed: $wt"; return 1
        fi
      fi
    fi
    current=$(git -C "$wt" symbolic-ref -q --short HEAD || true)
    if [[ -z "$current" ]]; then
      if ! git -C "$wt" switch -c "sync/detached/$RUN/w$i"; then
        problem "$name" "cannot name detached worktree $wt"; return 1
      fi
      record "$name" 'PRESERVED' "detached worktree $wt assigned sync/detached/$RUN/w$i"
    fi
  done < <(git -C "$dir" worktree list --porcelain)
}
# Save commit-bearing refs and stashes in reproducible immutable safety refs.
create_local_backups() {
  local name=$1 dir=$2 sha branch i=0 dest skipped_merged=0 skipped_remote=0 skipped_tags=0
  while IFS= read -r sha; do
    [[ -n "$sha" ]] || continue
    dest="sync/stash/$sha"
    if ! git -C "$dir" show-ref -q --verify "refs/heads/$dest"; then
      git -C "$dir" branch "$dest" "$sha" || { problem "$name" "cannot back up stash $sha"; return 1; }
    fi
    expected "$sha" "refs/heads/$dest"
  done < <(git -C "$dir" stash list --format='%H')
  while IFS=$'\t' read -r branch sha; do
    [[ -n "$sha" && "$branch" != sync/* ]] || continue
    if contained_in_main "$dir" "$sha"; then
      skipped_merged=$((skipped_merged + 1))
      continue
    fi
    # A commit already published as the tip of an identical remote branch
    # needs no additional remote checkpoint, even if not merged into main.
    if git -C "$dir" show-ref --verify -q "refs/remotes/origin/$branch" &&
       [[ "$sha" == "$(git -C "$dir" rev-parse "refs/remotes/origin/$branch")" ]]; then
      skipped_remote=$((skipped_remote + 1))
      continue
    fi
    i=$((i+1)); dest="sync/local/$RUN/b$i"
    git -C "$dir" branch "$dest" "$sha" || { problem "$name" "cannot snapshot $branch"; return 1; }
    record "$name" 'CHECKPOINT' "$branch => $dest ($sha)"
    expected "$sha" "refs/heads/$dest"
  done < <(git -C "$dir" for-each-ref --format='%(refname:short)%09%(objectname)' refs/heads)
  record "$name" 'SKIPPED' "local branch checkpoints: merged-into-main=$skipped_merged already-on-origin=$skipped_remote"
  # Back up every local tag, including annotated and orphaned tag objects,
  # before attempting any push of a potentially colliding tag name.
  i=0
  while IFS=$'\t' read -r branch sha; do
    [[ -n "$branch" && "$branch" != sync-local/* && "$branch" != sync-remote/* ]] || continue
    if git -C "$dir" show-ref --verify -q "refs/org-sync/import/$RUN/$branch" &&
       [[ "$sha" == "$(git -C "$dir" rev-parse "refs/org-sync/import/$RUN/$branch")" ]]; then
      skipped_tags=$((skipped_tags+1))
      continue
    fi
    i=$((i+1)); dest="refs/tags/sync-local/$RUN/t$i"
    if ! git -C "$dir" update-ref "$dest" "$sha"; then
      problem "$name" "cannot back up local tag $branch"; return 1
    fi
    if ! git -C "$dir" push origin "$dest:$dest" >> "$LOG" 2>&1; then
      problem "$name" "cannot push local tag backup $branch"; return 1
    fi
    expected "$sha" "$dest"
  done < <(git -C "$dir" for-each-ref --format='%(refname:strip=2)%09%(objectname)' refs/tags)
  # Upload only new safety refs for THIS run, plus stable stash backups.
  # Re-pushing every historical sync/* branch multiplies network work.
  local push_specs=() ref
  local testfile="$RUNDIR/$RUN-$name-local-check.tsv" live="$RUNDIR/$RUN-$name-local-live.tsv"
  : > "$testfile"
  while IFS=$'\t' read -r ref sha; do
    [[ -n "$ref" ]] || continue
    push_specs+=("$ref:$ref")
    printf '%s\t%s\n' "$sha" "$ref" >> "$testfile"
  done < <(git -C "$dir" for-each-ref --format='%(refname)%09%(objectname)' \
    "refs/heads/sync/local/$RUN" "refs/heads/sync/detached/$RUN" refs/heads/sync/stash)
  if (( ${#push_specs[@]} )); then
    if ! git -C "$dir" push origin "${push_specs[@]}" >> "$LOG" 2>&1; then
      problem "$name" 'local safety-ref push rejected (aborting integration)'; return 1
    fi
  fi
  record "$name" 'SKIPPED' "unchanged remote tags=$skipped_tags; historical sync/* refs not re-pushed"
  # Ensure the selected safety refs actually arrived before fetch/merge.
  while IFS=$'\t' read -r ref sha; do
    [[ "$ref" == refs/tags/sync-local/"$RUN"/* ]] || continue
    printf '%s\t%s\n' "$sha" "$ref" >> "$testfile"
  done < <(git -C "$dir" for-each-ref --format='%(refname)%09%(objectname)' refs/tags)
  verify_expected "$name" "$dir" local-backups "$testfile" "$live"
}
# Fetch remote branches and tags into non-destructive refs. Import tags into
# a private local ref namespace instead of overwriting colliding local tags.
fetch_safely() {
  local name=$1 dir=$2
  git -C "$dir" fetch --no-tags origin \
    '+refs/heads/*:refs/remotes/origin/*' >> "$LOG" 2>&1 || {
    problem "$name" 'remote branch fetch failed'; return 1;
  }
  git -C "$dir" fetch --no-tags origin \
    "+refs/tags/*:refs/org-sync/import/$RUN/*" >> "$LOG" 2>&1 || {
    problem "$name" 'isolated remote tag fetch failed'; return 1;
  }
}
backup_remote_refs() {
  local name=$1 dir=$2 before=$3 sha ref i=0 dest branch
  local skipped_merged=0 skipped_untouched=0 skipped_tags=0
  local after="$RUNDIR/$RUN-$name-prebackup-live.tsv" expected_file="$RUNDIR/$RUN-$name-remote-check.tsv" live="$RUNDIR/$RUN-$name-remote-live.tsv"
  : > "$expected_file"
  ordinary_remote_listing "$dir" > "$after" || { problem "$name" 'remote disappeared during backup'; return 1; }
  # Unrelated branch/tag updates are ordinary activity in a busy org.
  # Keep the newer snapshot, then independently validate only target refs.
  if ! cmp -s "$before" "$after"; then
    record "$name" 'CHURN' 'remote refs changed since fetch; using current per-ref state (unrelated changes do not block)'
  fi
  while IFS=$'\t' read -r sha ref; do
    [[ "$ref" != *'^{}' ]] || continue
    case "$ref" in
      refs/heads/sync/*|refs/tags/sync-remote/*|refs/tags/sync-local/*) continue ;;
      refs/heads/*)
        branch=${ref#refs/heads/}
        if contained_in_main "$dir" "$sha"; then
          skipped_merged=$((skipped_merged + 1))
          continue
        fi
        if ! remote_branch_needs_backup "$dir" "$branch" "$sha"; then
          skipped_untouched=$((skipped_untouched + 1))
          continue
        fi
        # For an affected branch, fetch exactly that ref and confirm the
        # fetched tip matches the live remote. Retry if it races again.
        local tip="" current="" attempt=0 matched=0
        while (( attempt < 3 )); do
          attempt=$((attempt + 1))
          if ! git -C "$dir" fetch --no-tags origin \
              "+$ref:refs/remotes/origin/$branch" >> "$LOG" 2>&1; then
            problem "$name" "cannot refresh changing target $ref"; return 1
          fi
          tip=$(git -C "$dir" rev-parse "refs/remotes/origin/$branch") || {
            problem "$name" "target vanished after fetch: $ref"; return 1;
          }
          if ! current=$(git -C "$dir" ls-remote --heads origin "$ref"); then
            problem "$name" "cannot check changing target $ref"; return 1
          fi
          current=$(printf '%s\n' "$current" | awk -F '\t' -v k="$ref" '$2 == k {print $1; exit}')
          if [[ -n "$current" && "$tip" == "$current" ]]; then
            matched=1
            if [[ "$sha" != "$tip" ]]; then
              record "$name" 'CHURN' "$ref advanced $sha -> $tip (refreshed)"
            fi
            sha=$tip
            break
          fi
          record "$name" 'CHURN' "$ref moved during targeted fetch (attempt $attempt/3)"
        done
        if (( ! matched )); then
          problem "$name" "target ref cannot stabilize after 3 checks: $ref"; return 1
        fi
        if ! remote_branch_needs_backup "$dir" "$branch" "$sha"; then
          skipped_untouched=$((skipped_untouched + 1))
          continue
        fi
        dest="refs/heads/sync/remote/$RUN/r$((i+1))"
        ;;
      refs/tags/*)
        # Tags are only created if absent, never changed/deleted. An existing
        # remote tag cannot be endangered by our non-force-pushing sync.
        skipped_tags=$((skipped_tags + 1))
        continue
        ;;
      *) continue ;;
    esac
    i=$((i+1))
    if ! git -C "$dir" cat-file -e "$sha^{object}" 2>/dev/null; then
      problem "$name" "unfetched remote object $ref@$sha"; return 1
    fi
    if ! git -C "$dir" push origin "$sha:$dest" >> "$LOG" 2>&1; then
      problem "$name" "remote backup rejected for $ref"; return 1
    fi
    printf '%s\t%s\n' "$sha" "$dest" >> "$expected_file"
    record "$name" 'BACKED_UP' "$ref => $dest ($sha)"
  done < "$after"
  record "$name" 'SKIPPED' "remote backups: merged-into-main=$skipped_merged untouched=$skipped_untouched tags-not-modified=$skipped_tags selected=$i"
  verify_expected "$name" "$dir" remote-backups "$expected_file" "$live"
}

# Integrate safely in any worktree. No -X ours/theirs; conflict stays intact.
integrate_one() {
  local name=$1 wt=$2 branch=$3 remote_branch=$4 target="refs/remotes/origin/$remote_branch"
  git -C "$wt" show-ref -q --verify "$target" || return 0
  if git -C "$wt" merge --ff-only "$target" >> "$LOG" 2>&1; then
    record "$name" 'INTEGRATED' "$branch fast-forward/already up-to-date"
    return 0
  fi
  if git -C "$wt" merge --no-ff --no-edit "$target" >> "$LOG" 2>&1; then
    record "$name" 'MERGED' "$branch (run your project tests)"
    return 0
  fi
  problem "$name" "merge conflict left intact: $wt ($branch); resolve and rerun"
  return 1
}
# Existing worktrees are integrated; temporarily check out other local branches
# in separate worktrees. Successful temp worktrees are removed only if clean.
integrate_all_branches() {
  local name=$1 dir=$2 branch wt remote_branch upstream onwt i=0 tmp
  while IFS= read -r branch; do
    [[ -n "$branch" && "$branch" != sync/* ]] || continue
    upstream=$(git -C "$dir" for-each-ref --format='%(upstream:short)' "refs/heads/$branch")
    if [[ "$upstream" == origin/* ]]; then remote_branch=${upstream#origin/}; else remote_branch=$branch; fi
    git -C "$dir" show-ref --verify -q "refs/remotes/origin/$remote_branch" || continue
    onwt=""
    # A branch can only be attached to a single worktree.
    while IFS= read -r wt; do
      [[ "$wt" == 'worktree '* ]] || continue
      tmp=${wt#worktree }
      if [[ "$(git -C "$tmp" symbolic-ref -q --short HEAD 2>/dev/null || true)" == "$branch" ]]; then
        onwt=$tmp; break
      fi
    done < <(git -C "$dir" worktree list --porcelain)
    if [[ -n "$onwt" ]]; then
      if has_unfinished_operation "$onwt" || [[ -n "$(GIT_OPTIONAL_LOCKS=0 git -C "$onwt" status --porcelain=v1)" ]]; then
        problem "$name" "cannot merge unclean worktree $onwt ($branch)"; continue
      fi
      integrate_one "$name" "$onwt" "$branch" "$remote_branch" || true
    else
      i=$((i+1)); tmp="$TMPBASE/$name-$i"
      if ! git -C "$dir" worktree add "$tmp" "$branch" >> "$LOG" 2>&1; then
        problem "$name" "cannot create temp worktree for $branch"; continue
      fi
      if integrate_one "$name" "$tmp" "$branch" "$remote_branch"; then
        if ! git -C "$dir" worktree remove "$tmp" >> "$LOG" 2>&1; then
          problem "$name" "clean temporary worktree remains: $tmp"
        fi
      else
        record "$name" 'RECOVERY' "conflicted temporary worktree retained at $tmp"
      fi
    fi
  done < <(git -C "$dir" for-each-ref --format='%(refname:short)' refs/heads)
}
# Publish ordinary branches individually: a rejected branch cannot prevent
# unrelated branches from being pushed. Never use --force or --mirror.
publish_all() {
  local name=$1 dir=$2 branch sha ref local_expected="$RUNDIR/$RUN-$name-final-check.tsv" live="$RUNDIR/$RUN-$name-final-live.tsv" failed=0
  : > "$local_expected"
  while IFS=$'\t' read -r branch sha; do
    [[ -n "$sha" ]] || continue
    ref="refs/heads/$branch"
    # Safety refs have already been pushed and verified at the safety gate.
    [[ "$branch" != sync/* ]] || continue
    # A local branch that has been fully incorporated into main, and has no
    # remote peer, does not warrant creating a new stale remote branch.
    if ! git -C "$dir" show-ref --verify -q "refs/remotes/origin/$branch" &&
       contained_in_main "$dir" "$sha"; then
      record "$name" 'SKIPPED' "not publishing merged-only local branch: $branch"
      continue
    fi
    if git -C "$dir" show-ref --verify -q "refs/remotes/origin/$branch" &&
       [[ "$sha" == "$(git -C "$dir" rev-parse "refs/remotes/origin/$branch")" ]]; then
      # Cached origin/* is not proof that GitHub is unchanged. Never
      # report 'already synced' based solely on a stale tracking ref.
      local observed=""
      if ! observed=$(git -C "$dir" ls-remote --heads origin "$ref"); then
        problem "$name" "cannot verify live branch tip: $branch"; failed=1; continue
      fi
      observed=$(printf '%s\n' "$observed" | awk -F '\t' -v k="$ref" '$2 == k {print $1; exit }')
      if [[ "$sha" == "$observed" ]]; then continue; fi
      record "$name" 'CHURN' "$branch moved after fetch; attempting safe non-force push"
    fi
    if git -C "$dir" push origin "$ref:$ref" >> "$LOG" 2>&1; then
      printf '%s\t%s\n' "$sha" "$ref" >> "$local_expected"
    else
      problem "$name" "push rejected: $branch (remote retained; local preserved)"; failed=1
    fi
  done < <(git -C "$dir" for-each-ref --format='%(refname:short)%09%(objectname)' refs/heads)
  # Publish all locally known tags. Colliding tags are never overwritten.
  while IFS=$'\t' read -r ref sha; do
    [[ -n "$ref" ]] || continue
    if git -C "$dir" push origin "$ref:$ref" >> "$LOG" 2>&1; then
      printf '%s\t%s\n' "$sha" "$ref" >> "$local_expected"
    else
      problem "$name" "tag push rejected: $ref (inspect conflict; no overwrite)"; failed=1
    fi
  done < <(git -C "$dir" for-each-ref --format='%(refname)%09%(objectname)' refs/tags)
  while IFS=$'\t' read -r ref sha; do
    [[ -n "$ref" ]] || continue
    if git -C "$dir" push origin "$ref:$ref" >> "$LOG" 2>&1; then
      printf '%s\t%s\n' "$sha" "$ref" >> "$local_expected"
    else
      problem "$name" "notes push rejected: $ref"; failed=1
    fi
  done < <(git -C "$dir" for-each-ref --format='%(refname)%09%(objectname)' refs/notes)
  if (( VERIFY )); then verify_expected "$name" "$dir" final-push "$local_expected" "$live" || failed=1; fi
  (( failed == 0 ))
}

sync_one() {
  local name=$1 dir=$2 before status_ok=1
  REPO_ERRORS=0
  record "$name" 'START' "$dir"
  if (( DRY )); then plan_repo "$name" "$dir"; return; fi
  [[ -d "$dir" ]] || { problem "$name" "missing path $dir"; return; }
  # Never let unverified local backup failures proceed to integration.
  checkpoint_worktrees "$name" "$dir" || return
  # Fetch updates only tracking/private refs, never worktree contents.
  # The local branch checkpoint decision must use a FRESH origin/main.
  before="$RUNDIR/$RUN-$name-before.tsv"
  if ! ordinary_remote_listing "$dir" > "$before"; then problem "$name" 'ls-remote failed'; return; fi
  fetch_safely "$name" "$dir" || return
  create_local_backups "$name" "$dir" || return
  if (( BACKUP_REMOTE )); then backup_remote_refs "$name" "$dir" "$before" || return; fi
  integrate_all_branches "$name" "$dir"
  publish_all "$name" "$dir" || status_ok=0
  if (( REPO_ERRORS > 0 || status_ok == 0 )); then
    record "$name" 'INCOMPLETE' 'local commits and backups preserved; inspect failures, run tests, rerun'
  else
    record "$name" 'COMPLETE' 'all attempted refs published (and verified if --verify)'
  fi
}

say "Org=$ORG Root=$ROOT Run=$RUN Mode=$([[ $DRY == 1 ]] && echo read-only || echo mutating)"
discover_local
NAMES=()
if [[ -n "$REPO" ]]; then
  remote_exists "$REPO" || { say "Cannot access $ORG/$REPO"; exit 1; }
  NAMES+=("$REPO")
else
  while IFS= read -r name; do [[ -n "$name" ]] && NAMES+=("$name"); done < <(remote_names)
  (( ${#NAMES[@]} > 0 )) || { say 'No accessible repositories returned; aborting'; exit 1; }
fi
# Process every standalone local clone of each repo; clone only when not found.
for name in "${NAMES[@]}"; do
  found=0
  for (( idx=0; idx<${#ROOT_NAMES[@]}; idx++ )); do
    if [[ "${ROOT_NAMES[idx]}" == "$name" ]]; then
      found=1
      sync_one "$name" "${ROOT_REPOS[idx]}"
    fi
  done
  if (( ! found )); then
    dir="$ROOT/$name"
    if [[ -e "$dir" ]]; then
      problem "$name" "destination exists but isn't recognized as an org clone: $dir"
    elif (( DRY )); then
      record "$name" 'PLAN' "would clone $ORG/$name into $dir"
    elif gh repo clone "$ORG/$name" "$dir" >> "$LOG" 2>&1; then
      record "$name" 'CLONED' "$dir"
      sync_one "$name" "$dir"
    else
      problem "$name" "clone failed: $dir"
    fi
  fi
done
if (( ! DRY && REPORT )); then
  {
    printf '# Organization sync report\n\n**Organization:** `%s`  \n**Run:** `%s`  \n**Issues:** %s\n\n' "$ORG" "$RUN" "$ERRORS"
    printf '| Repository | Status | Detail |\n|---|---|---|\n'
    awk -F '\t' 'NR>1 {gsub(/\|/, "\\|", $3); printf "| `%s` | %s | %s |\n", $1, $2, $3}' "$REPORT_TSV"
    printf '\nFull log: `%s`\n' "$LOG"
  } > "$REPORT_MD"
  say "Report: $REPORT_MD"
fi
say "Finished. Issues=$ERRORS"
(( ERRORS == 0 ))
