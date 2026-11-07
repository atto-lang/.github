# GitHub PR review governance — scoped audit and controlled threshold repair

The requested policy for `ores-truffle-oreslang` is that **one valid, independent approval from either `oresoftware` or `the1mills` should be sufficient**. This tool deals ONLY with an existing **numeric** required-approval count. It does not create a GitHub ruleset, allowlist specific reviewers, alter CODEOWNERS, grant bypass permissions, or merge PRs. An author cannot approve their own pull request.

## Verified platform blockers (November 7, 2026)

The connected GitHub integration verified metadata access and admin permissions for the two named accounts in Java, LLVM, and platform-conformance repositories, but **does not have permission to read or write branch protection**. Direct integration GET requests returned:

- `oreslang-source.java/branches/main/protection`: HTTP **403**, `Resource not accessible by integration`.
- `oreslang-source.java/rulesets?includes_parents=true`: HTTP **403**, `Upgrade to GitHub Pro or make this repository public to enable this feature`.
- `.github/branches/main/protection`: HTTP **403**, `Resource not accessible by integration`.
- `.github/rulesets?includes_parents=true`: same GitHub plan-related **403**.
- Public `oreslang-cli`, `ores-truffle-oreslang.github.io`, and `vscode-plugins` repository ruleset lists returned `[]`; branch protection GET still returned integration-permission **403**.

Consequently, **organization-wide policy has NOT been applied or verified**. The org's GitHub plan prevents a reliable private-repo ruleset solution until plan/visibility constraints are resolved; a separate admin-authenticated GitHub CLI may read branch protection where the connector cannot, but cannot bypass plan restrictions.

## Usage — no write on default invocation

With administrator-authenticated GitHub CLI, installed Rust, and `ATTO_GH_BIN` set to an **absolute** path when `gh` is not at `/usr/bin/gh`:

```sh
gh auth status
cargo test --manifest-path tools/merge-review-policy/Cargo.toml --all-targets
cargo clippy --manifest-path tools/merge-review-policy/Cargo.toml --all-targets -- -D warnings

# Read-only organization audit (default)
cargo run --manifest-path tools/merge-review-policy/Cargo.toml --

# Read-only single repo
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- --repo oreslang-source.java

# Deliberate one-repo update after review of all effective rules
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- \
  --repo oreslang-source.java --apply --confirm-org ores-truffle-oreslang

# ONLY AFTER an audit, review and a successful single-repo pilot:
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- \
  --all --apply --confirm-org ores-truffle-oreslang
```

For Homebrew on macOS, pass `ATTO_GH_BIN=/opt/homebrew/bin/gh` (when that is the actual trusted binary path). The CLI does **not** search `PATH` or invoke shell scripts for API operations, and targets `github.com` explicitly.

**No `--apply` invocation should be used until a trusted administrator has reviewed the effective settings and checked repository eligibility.** The program requires the repository scope AND the explicit organization confirmation before any write.

## What the tool hardens

1. Validates that all repository metadata belongs to exactly `ores-truffle-oreslang`, rejects unsafe API path components, and percent-encodes the default branch.
2. Paginates the repository list and inherited/local ruleset list, treating incomplete or malformed results as errors.
3. Treats **any active ruleset** as requiring manual inspection rather than assuming a numeric branch protection threshold controls the actual merge.
4. Treats missing protections, missing review counts, unexpected counts, CODEOWNERS approval requirements, existing bypass allowances, inaccessible endpoints, archived repos and changed snapshots as `MANUAL`.
5. On explicit `--apply` only, re-fetches effective ruleset listings and the complete branch protection snapshot immediately before a single `PATCH` to `required_approving_review_count: 1`.
6. Re-reads the **complete** protection snapshot after the write; any change beyond the single count fails verification. Re-reads active rulesets again before reporting a successful receipt.
7. Gives **non-zero exit** for any incomplete organization run, manual review or API failure. Never reports a partial rollout as total success.
8. Preserves existing status checks, stale-review dismissal, last-push review, CODEOWNERS, bypass/dismissal settings, and administrator protection. No `PUT` overwriting full protection.

## Review scope versus reviewer identity

GitHub's native numeric approval count permits any qualified reviewer, **not only** `oresoftware` and `the1mills`. If those two must be the ONLY authorized approving reviewers, a separately enforceable **CODEOWNERS** or **required-reviewer team ruleset** is necessary. A `merge-approvers` team with both accounts, with one required team review on all changed files, is the preferred policy where GitHub Team/enterprise ruleset enforcement is available. Existing independent-review and security tickets should not be retired just because the count is one.

The tool does **not** dismiss any `CHANGES_REQUESTED` reviews, remove draft status, bypass required checks, or suppress existing protections.

## Evidence and promotion

- Source and negative-path unit tests are checked into this PR; they are **not an executed test receipt**.
- A SHA-pinned `actions/checkout` workflow was added at `.github/workflows/merge-review-policy-rust.yml` to test this Rust crate (and print exact checkout/tree identity) once runner admission executes steps.
- Existing hosted runs on the original PR head had `steps=null`; these are **runner-admission failures**, not compiler-backed evidence of passing/failing Rust code.
- Before promoting or using `--apply`, require executed Rust test results at exact head, an authorized independent reviewer, a dry-run receipt including inaccessible repositories, a one-repo pilot with full policy readback, and verified plan-supported enforcement.

**Not a merge-rule bypass.** The important outcome is accurate, fail-closed visibility: no change to private-repo merge protections can be claimed from this PR alone.
