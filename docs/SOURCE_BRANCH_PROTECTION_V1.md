# Oreslang source branch protection contract v1

This is the organization-level release gate for the nine Oreslang / AttoLang source backends:

- `oreslang-source.java`
- `oreslang-source.llvm`
- `oreslang-source.net`
- `oreslang-source.wasm`
- `oreslang-source.forge`
- `oreslang-source.cranelift`
- `oreslang-source.js`
- `oreslang-source.rs`
- `oreslang-source.hdl`

It is a policy target for repository administrators. It does not claim the settings are currently enabled. For **reviewer identity and sufficiency**, the organization-wide [trusted merge-approval policy](ORG_TRUSTED_MERGE_APPROVALS.md) controls. A valid approval from either `@oresoftware` or `@the1mills` is sufficient; no additional procedural reviewer may be demanded.

**Current unresolved runner-admission incident:** [source-local smoke, post-merge
verification debt, and administrator remediation runbook](ACTIONS_RUNNER_ADMISSION_RECOVERY_20261026.md).
Do not require zero-step failing checks, count unexecuted jobs as passes, or
export private source to an unapproved alternate organization.

## Required `main` protections

Each repository's default `main` branch MUST:

1. report `protected=true` through GitHub's authenticated branch API;
2. require pull requests and at least one **current approval from `@oresoftware` or `@the1mills`**, with the approving account distinct from the PR author;
3. ensure CODEOWNER requirements for compiler/runtime/workflow/security-policy changes **can be satisfied by either trusted account**; do not require a separate CODEOWNER or second independent reviewer after a valid approval by either account;
4. retain the one valid approval across ordinary new commits (`dismiss_stale_reviews_on_push=false`; `require_last_push_approval=false`), while still testing each changed SHA;
5. require resolution of review conversations;
6. apply protections to administrators and expose no user/team/app pull-request bypass allowance;
7. reject direct/force pushes and branch deletion except through an explicitly reviewed incident procedure;
8. require strict status checks bound to the issuing GitHub App, never legacy name-only contexts;
9. require the `exact-run-evidence-contract` check from the reviewed workflow plus the backend's real native/compiler/runtime checks;
10. treat missing checks, startup failures, zero-step jobs, cancelled jobs, and source-free verifier-only runs as NOT EXECUTED, never as passing native evidence.

For GitHub Actions checks, policy code currently pins GitHub Actions App ID `15368`. If GitHub changes the issuer identity, that must be independently reviewed and updated deliberately rather than accepted dynamically.

## Exact-SHA rule

Required execution evidence MUST identify the exact PR head or merge-queue commit and its root Git tree. Evidence from a different commit/tree, a synthetic source-free carrier, or a rerun whose jobs never execute is insufficient.

## Current evidence PRs

- Java: #1112
- LLVM: #495
- Forge: #50
- .NET: #56
- Wasm: #52

These PRs are supporting enforcement code, not substitutes for branch protection itself.

## Activation gate

Do not activate required status contexts until source-org runners can actually start those jobs. Before activation, an administrator must demonstrate on each backend:

- a deliberately failing synthetic PR is blocked;
- missing valid approval from `@oresoftware` or `@the1mills` is blocked, and an unrelated approver alone does not satisfy this review policy;
- a routine new commit does not automatically invalidate the already accepted one-time trusted-owner approval once the target repository rules have been changed;
- a zero-step/startup-failure check is blocked;
- the exact required checks plus valid approval from either trusted account permit merge (subject to all other applicable GitHub restrictions).

Record repository ID, protected branch SHA, ruleset/protection readback, check context + issuer App ID, PR SHA/tree, and run/job IDs in the security tracker.

As of 2026-11-06, authenticated API readbacks on all nine source backends report `main.protected=false` and no required checks; this remains an **unapplied policy target**, not a certification of effective controls. GitHub's connected integration cannot change administrative rules. Verify fresh branch/ruleset state before acting.

## Administrator-first activation sequence (P0)

1. **Private-runner diagnosis before any required-check activation.** Read the exact failed-check **annotation** in GitHub's UI and record the message. Inspect organization **Billing and licensing → Actions** for private repository entitlements, budgets, spending-stop flags, usage and payment state. Inspect organization/repository **Settings → Actions → General** for permitted actions, hosted runners and runner-group eligibility. The API-visible `steps=[]` finding alone does **not** prove a billing problem.
2. **Differential check without source export.** Public same-org [Sublime run 42288890488](https://github.com/ores-truffle-oreslang/sublime-text-plugins/actions/runs/42288890488) executed 3/3 jobs successfully, but private [SSR setup run 42288841249](https://github.com/ores-truffle-oreslang/oreslang-ssr/actions/runs/42288841249) and [native audit/render run 42288841279](https://github.com/ores-truffle-oreslang/oreslang-ssr/actions/runs/42288841279) failed with no steps. This distinguishes repository eligibility/policy from a total org-wide runner outage, but does not identify the root cause. Execute a private **source-free** canary in the source org: require a named runner, non-zero completed steps and authenticated exact-head linkage. Do not copy private compiler trees to another organization or make repos public.
3. **Restore native source-owned CI.** After admission recovery, test exact original PR HEAD and root tree for each backend in its original repository. Include the project's substantive compiler/runtime tests, negative fixtures, job identity, attempt and executed-step evidence. A host-only or source-free canary is not compiled `.atto` guest parity.
4. **Confirm plan and administrator permission.** For private repositories, ruleset queries previously returned an HTTP 403 upgrade/visibility notice; branch-protection details returned HTTP 403 `Resource not accessible by integration`. Only an authorized administrator can verify entitlements or configure protection. Never infer that an inaccessible ruleset is active, and do not purchase a plan without owner authorization.
5. **Enable and verify enforced protection.** Require PRs, independent code-owner approval and stale-review dismissal; disable unauthorized bypass/force push/deletion; attach the actual source-owned required checks by issuer identity, after those checks are able to run. Record API/UI proof of effective rules on every `main`. A checked-in CODEOWNERS file alone does not enforce review.
6. **Exercise five fail-closed mutations.** Demonstrate denial for a zero-step job, absent approval, stale approval after a new commit, wrong head/tree, and missing/failed required check. Demonstrate acceptance only for the valid exact head with a genuinely completed native run and independent review. Verify protection applies to administrators.
7. **Post-merge replay and receipt.** Re-read the merge SHA and `main` Git tree; run real native tests on the landed commit, persist immutable run/job/check/step receipts, and classify `NOT_EXECUTED` separately from tests that executed and failed. Keep unvalidated merges in an explicit debt queue.

Owner: administrator assigned on [central security incident #2](https://github.com/ores-truffle-oreslang/.github/issues/2). Cross-repo coordination: [platform conformance issue #359](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/359). Until both effective runner admission and enforced review/branch controls are proven, GitHub's `mergeable=true` is not sufficient release authorization.

