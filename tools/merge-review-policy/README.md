# GitHub PR review governance — read-only audit with fail-closed write hold

The requested policy for `ores-truffle-oreslang` is that **one valid, independent approval from either `oresoftware` or `the1mills` should be sufficient**. This tool deals ONLY with an existing **numeric** required-approval count. It does not create a GitHub ruleset, allowlist specific reviewers, alter CODEOWNERS, grant bypass permissions, or merge PRs. An author cannot approve their own pull request.

## Verified platform blockers (November 7, 2026)

The connected GitHub integration verified metadata access and admin permissions for the two named accounts in Java, LLVM, and platform-conformance repositories, but **does not have permission to read or write branch protection**. Direct integration GET requests returned:

- `oreslang-source.java/branches/main/protection`: HTTP **403**, `Resource not accessible by integration`.
- `oreslang-source.java/rulesets?includes_parents=true`: HTTP **403**, `Upgrade to GitHub Pro or make this repository public to enable this feature`.
- `.github/branches/main/protection`: HTTP **403**, `Resource not accessible by integration`.
- `.github/rulesets?includes_parents=true`: same GitHub plan-related **403**.
- Public `oreslang-cli`, `ores-truffle-oreslang.github.io`, and `vscode-plugins` repository ruleset lists returned `[]`; branch protection GET still returned integration-permission **403**.

Consequently, **organization-wide policy has NOT been applied or verified**. The org's GitHub plan prevents a reliable private-repo ruleset solution until plan/visibility constraints are resolved; a separate admin-authenticated GitHub CLI may read branch protection where the connector cannot, but cannot bypass plan restrictions.

## Critical enforcement hold — all `--apply` calls currently reject

An independent audit found a dangerous mismatch: the historical numeric-only `patch()` path was eligible when `require_code_owner_reviews=false` but rejected `require_code_owner_reviews=true`. Such a patch could lower the number of reviews **without** enforcing the designated `oresoftware`/`the1mills` reviewers. The `.github/CODEOWNERS` entry in this **draft** PR does not supply protection on the base branch, and the organization has not verified required code-owner review on a private repository.

**Therefore the production CLI now rejects every `--apply` invocation before `gh` is initialized, before GitHub API discovery, and before any PATCH, even if `--ack-numeric-only` and an approved manifest are supplied.** Dry-run audits remain supported. The underlying library patch functions are retained as isolated planning/test fixtures, **not** a live implementation authorization. Future work must introduce an independently tested GitHub-effective named-reviewer gate before intentionally lifting this hold. Four source tests ensure the hold covers single-repository and bulk writes. This is not evidence that the tests executed.
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

# Currently BLOCKED; preserved only to demonstrate rejected arguments
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- \
  --repo oreslang-source.java --apply --confirm-org ores-truffle-oreslang \
  --ack-numeric-only

# Also BLOCKED until GitHub-enforced named reviewers are verified:
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- \
  --all --apply --confirm-org ores-truffle-oreslang \
  --expect-repos 80 --ack-numeric-only
```

For Homebrew on macOS, pass `ATTO_GH_BIN=/opt/homebrew/bin/gh` (when that is the actual trusted binary path). The CLI does **not** search `PATH` or invoke shell scripts for API operations, and targets `github.com` explicitly.

**Do not attempt `--apply` while this enforcement hold is in place.** It now fails closed even with scope confirmation, and no live write is permitted. A future reviewed change must verify effective named-reviewer rules rather than merely rely on numeric acknowledgement.

## Two critical safety acknowledgements

**1. Numeric-only is NOT named-reviewer enforcement.** A branch-review API PATCH with `required_approving_review_count: 1` can count approval from any eligible reviewer, not exclusively `oresoftware` or `the1mills`. The argument parser retains `--ack-numeric-only`, but the CLI **currently denies all writes**, including acknowledged ones. It does **not** claim the user's named-reviewer requirement is fully enforced: that requires separate GitHub-side required-reviewer team or CODEOWNERS policy and live merge-eligibility testing.

**2. A connected account may not see all private repositories.** Bulk writes now require `--expect-repos COUNT` from an independently verified organization inventory and stop without writes on a cardinality mismatch. The example count of **80** came from one accessible inventory on 2026-11-07, not a permanent guarantee. Before applying, refresh it using an organization-owner view, check archived repositories and newly created/renamed repositories, and adjust the flag based on a signed-off list. A count alone cannot rule out substitutions; compare names and branch identity too. All operations remain gated by read-only preflight, so inaccessible protections/rulesets produce an abort.

The `review_evidence` Rust module provides a **read-only review-history witness**, not an enforceable status check. It rejects self-approval, unrelated approvals, stale head, conflicting changes requests, dismissed approval, and malformed histories. An actual GitHub ruleset plus required CI remains essential.

## Mandatory bulk identity manifest

Before any `--all --apply`, supply an **independently approved complete repository/default-branch inventory** in a local UTF-8 TSV file. Set `ATTO_APPROVED_REPO_INVENTORY` to its **absolute path**. The file must be a regular file, not a symlink. Every active repository must have exactly one row of the format `repo<TAB>default_branch`, with LF line endings. Blank/comment lines, duplicate names, unsafe names, or a changed default branch cause an immediate nonzero exit **before any API PATCH**.

Example rows (these are only an illustration, **not** a complete org inventory):

```text
.github	main
oreslang-source.java	main
oreslang-source.llvm	main
```

Use an owner-reviewed full inventory export, not the example above. One manifest path can be supplied as an environment variable:

```sh
export ATTO_APPROVED_REPO_INVENTORY=/absolute/path/to/approved-ores-truffle-oreslang.tsv
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- \
  --all --apply --confirm-org ores-truffle-oreslang \
  --expect-repos 80 --ack-numeric-only
```

The example count **80** was one observed snapshot, not a guaranteed current size. It must match the reviewed manifest and the live API inventory. An access-limited caller, a same-count repo substitution, or a default-branch change fails closed. The tool rechecks the entire list after preflight and each target immediately before mutation. The file is an operator approval artifact, **not a cryptographic signature**. This tool still cannot enforce named-reviewer identity.

## What the tool hardens

1. Validates that all repository metadata belongs to exactly `ores-truffle-oreslang`, rejects unsafe API path components, and percent-encodes the default branch.
2. Paginates the repository list and inherited/local ruleset list, treating incomplete or malformed results as errors.
3. Treats **any active ruleset** as requiring manual inspection rather than assuming a numeric branch protection threshold controls the actual merge.
4. Treats missing protections, missing review counts, unexpected counts, CODEOWNERS approval requirements, existing bypass allowances, inaccessible endpoints, archived repos and changed snapshots as `MANUAL`.
5. A future, separately reviewed writer would re-fetch effective rulesets and the full protection snapshot immediately before any PATCH. **This write path is currently unreachable from the CLI while named-reviewer enforcement is unverified.**
6. Re-reads the **complete** protection snapshot after the write; any change beyond the single count fails verification. Re-reads active rulesets again before reporting a successful receipt.
7. Gives **non-zero exit** for any incomplete organization run, manual review or API failure. Never reports a partial rollout as total success.
8. Preserves existing status checks, stale-review dismissal, last-push review, CODEOWNERS, bypass/dismissal settings, and administrator protection. No `PUT` overwriting full protection.

## Review scope versus reviewer identity

GitHub's native numeric approval count permits any qualified reviewer, **not only** `oresoftware` and `the1mills`. If those two must be the ONLY authorized approving reviewers, a separately enforceable **CODEOWNERS** or **required-reviewer team ruleset** is necessary. A `merge-approvers` team with both accounts, with one required team review on all changed files, is the preferred policy where GitHub Team/enterprise ruleset enforcement is available. Existing independent-review and security tickets should not be retired just because the count is one.

The tool does **not** dismiss any `CHANGES_REQUESTED` reviews, remove draft status, bypass required checks, or suppress existing protections.

## Two-phase bulk rollout protocol

The organization-wide `--all --apply` path performs an initial **read-only preflight of every eligible repository**. Each preflight captures the complete branch-protection JSON snapshot and confirms there are no active inherited/local rulesets. If **one** repository is inaccessible, unsupported, ambiguous, or missing review protection, **no write begins**; it reports a failed preflight and requires manual handling. The bulk application then verifies each repository against its original snapshot and checks again immediately before PATCH.

GitHub does not provide an atomic transaction across repositories. If a write fails or the post-write readback differs, the command **halts the bulk rollout at the first failure**. Some earlier repositories may already have changed. Reconcile using captured before/after policy receipts and administrator review; do not assume automatic rollback or blindly rerun all repositories. A rollback must restore exactly the previous numeric count using an authorized administrator, subject to the same live protection/ruleset checks, and verify that other settings remain unchanged.

Repository list pagination, per-repository ruleset pagination, active-ruleset detection, protected-branch availability, and a permission/plan denial are explicit fail-closed stages. Avoid blanket claims that one approval is enforced until each repository's effective GitHub rules have been verified.

## Privileged GitHub CLI binary selection

`--apply` can change repository settings. The policy auditor calls an executable directly (not through a shell or PATH search), using `/usr/bin/gh` by default. When an administrator uses Homebrew on macOS, they may explicitly select a trusted absolute binary path such as `ATTO_GH_BIN=/opt/homebrew/bin/gh`. Paths with relative lookup, dot traversal, control characters, or a final executable name other than `gh` are rejected. A valid path is **not** proof that the selected binary is trustworthy: verify the installation independently before running any write.

Three new Rust source tests check accepted Linux/macOS paths and negative traversal/shell-like inputs. These tests are **not** hosted execution evidence until admitted exact-head CI runs them.
## Proposed `.github` repository CODEOWNERS pilot

This PR adds `.github/CODEOWNERS` to the **organization's `.github` repository only**, with one full-coverage rule:

```text
* @oresoftware @the1mills
```

Four source tests pin the exact two-owner, single-rule coverage and prevent accidental overrides. **This file is not automatically inherited by other repositories, and merely adding it does not enforce approvals.** It takes effect for review requests after it lands on the protected base branch; blocking merges on code-owner approval additionally requires an administrator-enabled effective `require_code_owner_reviews` rule or equivalent ruleset on an eligible GitHub plan. The two accounts must have write permission. Neither author can independently approve their own PR under normal GitHub review rules. <https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners>

**Do not replace other repositories' CODEOWNERS wholesale.** For example, the private Java compiler's `.github/CODEOWNERS` includes security-sensitive CI/source-export paths owned exclusively by `ORESoftware`. A blanket ownership line inserted after those path-specific rules could override that stricter policy because GitHub uses the last matching entry. Preserve all existing security-specific patterns and their precedence. Each repository needs an independently audited ownership proposal, plan/access verification, and live merge-eligibility pilot.

**Important release hold:** the policy auditor's numeric PATCH by itself cannot enforce this designated owner set. Do not run `--apply` to decrease any threshold unless the target repository's *effective GitHub* code-owner/required-reviewer enforcement has been independently verified and existing CI/security restrictions preserved. No administrator enforcement or global rollout is implied by this PR.
## Read-only named-reviewer pilot evidence (source-level candidate checker)

`src/named_owner_preflight.rs` adds `assess_pilot(PilotEvidence)` and **24 adversarial/source tests**. Its result is deliberately called `ReadOnlyCandidate`, not `Enforced`, `ReadyToMerge`, or authorization to PATCH. It is scoped to this `.github` repository's **pinned `main`** only and requires all of:

- A matching full 40-hex base-branch SHA and CODEOWNERS content ref; the exact, sole effective `* @oresoftware @the1mills` ownership rule; and an empty GitHub CODEOWNERS error list.
- Explicit write/maintain/admin access for **both** named accounts, from administrative collaborator-permission readbacks.
- A complete, inspectable ruleset inventory with **no active local or inherited rulesets**. An active ruleset is `MANUAL`, not ignored or automatically accepted.
- Effective branch protection requiring **exactly one code-owner approval**, stale-review dismissal, independent approval of the last push, enforced admin protection, no ambiguous review bypass allowances, and at least one required CI status context/check.

These checks deliberately fail closed if inputs are absent, malformed, non-pinned, or externally inaccessible. A candidate still needs verified check-run publisher identity, independently reviewed source/test receipts, exact-head PR review behavior, and a **live GitHub merge-eligibility pilot**. Tests using constructed JSON fixtures are not live GitHub evidence. The checker is intentionally not a way to lift the executable's `--apply` hold, and no production automation calls this checker as a decision authorizing writes.

**Administrator evidence capture:** Run `gh auth status` from a trusted admin workstation, then read the exact `main` commit SHA and retrieve `.github/CODEOWNERS` at **that SHA** via `GET /repos/ores-truffle-oreslang/.github/contents/.github/CODEOWNERS?ref=SHA`; verify the returned file path, blob and decoded text. Separately use `GET /repos/ores-truffle-oreslang/.github/codeowners/errors`, `GET /repos/ores-truffle-oreslang/.github/collaborators/oresoftware/permission`, the same for `the1mills`, `GET /repos/ores-truffle-oreslang/.github/branches/main/protection`, and `GET /repos/ores-truffle-oreslang/.github/rulesets?includes_parents=true`. Each call is read-only; 403/404 or changed base HEAD is a **failed witness**, not proof of absence. Recheck the base HEAD afterward. Redact access metadata and do not publish raw private admin responses.

**Release sequence:** First recover the source-free private runner, verify the named-reviewer candidate readback, run positive/negative live merge tests, and independently approve the code. Only then create and review a **separate deliberate change** to enable writing under the verified mechanism.
## Required CI evidence

`merge-review-policy-rust.yml`: SHA-pinned source checkout, exact tree metadata, executed Rust tests, and Clippy. `merge-review-runner-canary.yml`: **source-free admission only** (does not check out or run repository code). For any Actions job, `steps=null` or `steps=[]` means runner admission failure, not passing CI, even if the workflow status is completed.

The pull request's current source contains 41 core policy tests, 24 CI receipt/workflow-integrity tests, 20 named-reviewer history tests, 24 named-owner pilot preflight tests, 18 exact-inventory tests, 7 CLI path/write-hold tests, and 74 integration tests (208 test cases in source). **No passing executed test receipt is claimed** until the exact-head hosted run contains non-zero executed steps and passing Cargo output. The connected runtime lacks a Rust toolchain, so local Cargo execution was not performed during authoring.

A required end-to-end pilot must also exercise the actual GitHub effective PR merge state for: `oresoftware` approving a `the1mills` PR; the reverse; no approval; unrelated user approval; failing required CI; and stale approvals after changes. These are **not** proven by source-only/mocked tests.


### Exact PR head versus GitHub's synthetic merge commit

For `pull_request` events, Actions normally checks out a synthetic `refs/pull/N/merge` commit. This is **not** the PR's exact submitted head. The Rust policy workflow now explicitly checks out `${{ github.event.pull_request.head.sha || github.sha }}`, asserts `git rev-parse HEAD == EXPECTED_HEAD`, and prints immutable tree identity and run/attempt. Four negative/source tests detect missing head checkout, missing SHA equality, and missing tree receipts. Historic pre-fix runs must not be promoted as *exact-head* compiler or tool execution, even if they eventually had executed steps.

### Independent exact-head gate receipts

The Rust workflow keeps **Cargo tests, Clippy, and rustfmt all required**. Tests run first, then Clippy and formatting are marked `if: success() || failure()` so a failing earlier gate cannot conceal whether other gates passed or failed. This does not bypass any required gate, and canceled workflows remain non-authorizing. Since the connected environment has no Rust compiler, the formatting and test statuses remain **unverified until an admitted runner executes them**.

## Evidence and promotion

- Source and negative-path unit tests are checked into this PR; they are **not an executed test receipt**.
- A SHA-pinned `actions/checkout` workflow was added at `.github/workflows/merge-review-policy-rust.yml` to test this Rust crate (and print exact checkout/tree identity) once runner admission executes steps.
- Existing hosted runs on the original PR head had `steps=null`; these are **runner-admission failures**, not compiler-backed evidence of passing/failing Rust code.
- Before promoting or using `--apply`, require executed Rust test results at exact head, an authorized independent reviewer, a dry-run receipt including inaccessible repositories, a one-repo pilot with full policy readback, and verified plan-supported enforcement.

**Not a merge-rule bypass.** The important outcome is accurate, fail-closed visibility: no change to private-repo merge protections can be claimed from this PR alone.
