# AttoLang Draft-hold controller: staged rollout

**Status:** source implementation for review. Not yet an installed GitHub App or an organization-wide required check. The controller is intentionally narrower than the complete day-0/day-3/day-10 four-party merge policy in AGENTS.md and governance/atto-ready-four-party.mjs (introduced in PR #49).

## What this Rust component does

- Reads the **current PR resource directly from authenticated GitHub API** using GitHub CLI. The jq query produces a fixed schema; all expected organization/repository/PR identifiers, immutable head SHA, draft/merged state and label-array availability are validated.
- If either canonical keep-as-draft or legacy keep-draft exists: publishes a failing status on current head; if PR is Ready and open, re-reads state immediately before issuing gh pr ready --undo, then verifies that the PR is Draft. It never removes labels.
- If not-ready exists: publishes failure without forcing a Draft conversion.
- If PR is unlabelled and Ready: publishes **only** the narrow atto-governance/draft-hold success status. This means "no draft hold", **not** permission to merge. A full review gate remains independently required.
- Fail-closed on missing labels, API/CLI failures, unexpected state, malformed SHA, repository mismatch, PR/head drift, unverified restoration, or invalid caller activation. An error may leave an earlier commit-status unchanged if GitHub itself is unavailable: this is not a fail-safe merge lock unless the required check, issuer restriction, and merge controls are installed and working.

By default the tool is read-only. Writes require **all**: --apply, ATTO_DRAFT_HOLD_APPLY=1, GITHUB_REPOSITORY equal to --repo, an approved triggering event, a valid GH_TOKEN, and live GitHub re-reads. No shell expansion of PR-body or label text occurs. This does not accept a source-PR branch as trusted executable policy.

## Reviewable test path

PR workflows use .github/workflows/atto-draft-hold-contract.yml: exact-head source checkout, no write permissions, Rust unit/negative tests, jq projection fixtures, and Rust formatting. The workflow never converts a real PR to Draft or posts a production status.

## Example deployment (do not activate with an unreviewed ref)

See governance/examples/draft-hold-caller.yml. For each **approved repository**:

1. Merge and independently review #49 (policy) and this controller PR. Publish the approved immutable commit SHA of the controller on the **protected** default branch. Pin the example checkout to that exact SHA (never main, a floating tag, or an unreviewed PR head).
2. Review the caller workflow and merge it on the repository's protected default branch. The privileged pull_request_target workflow must **never** check out PR head code or execute PR-supplied shell strings. Use a repository-scoped GITHUB_TOKEN with only contents:read, pull-requests:write, statuses:write. Require Actions policy to permit the trusted event.
3. Initially run **read-only** and review logs/permissions. After code-owner/security approval, opt in to --apply and the explicit activation variable in the protected caller only. Avoid starting a PR by making keep-as-draft appear or disappear during rollout.
4. Configure a main-branch ruleset/branch protection requiring this context, with the expected **trusted issuing app**, and separately require a full, independently authenticated atto-governance/reviews-verified gate or equivalent. Do **not** assume atto-governance/draft-hold alone satisfies the 2-human/2-AI -> 1-human+1-AI -> 2-human/2-AI schedule, reviewer freshness, CODEOWNERS, CI, overlap audit, or mergeability. Do not disable stronger protections to add these.
5. Audit bypass actors and third-party apps. Lock down direct and API merges to authorized flows where supported; a repository admin with bypass rights or an untrusted publisher of the same commit-status context can defeat the check. Bind expected source to the trusted App rather than relying on a bare status name.
6. Reconcile every **open PR** periodically from a trusted installation, not only on webhooks: a missed event, permission failure or out-of-order event can leave a previously successful status on the **same SHA**. Serialize evaluations per repo+PR and recheck state before any authorized merge. For absolute transaction-level authorization, use a privileged merge broker/required app policy that re-evaluates under its own authority and disables ordinary UI merge bypass.
7. Exercise label add/remove, Ready-with-hold, Draft-with-hold, legacy alias, not-ready, manual Ready overrides, duplicate/out-of-order events, pushes, API denial, reviewer identity substitution, exact 72h/240h boundaries, and every Draft -> Ready timer reset. Keep a rollback procedure that restores an explicitly *blocking* required check until the replacement is ready.
8. Roll out progressively: atto-lang/.github then atto-lang/atto-lang.github.io then each active atto-source.* repo and the remaining organization. Inventory protected branches and record per-repo issuer, ruleset IDs, workflow installed SHA, and tested guard response.

### Important limits

- No GitHub App installation, ruleset mutation, cross-repository workflow installation, or human/AI attestation integration is performed by this PR.
- An asynchronous webhook handler **cannot guarantee zero race window** between label changes and manually initiated merge, regardless of re-fetching. Required issuer-bound protection plus controlled merge authority is necessary to make "no matter what" a strong claim.
- GitHub Actions source CI is available for the policy repository. Do not mirror a private tree to a different org just to add CI minutes. If admission is genuinely blocked, use **only** a pre-authorized private carrier with exact blob and root-tree SHA verification.
