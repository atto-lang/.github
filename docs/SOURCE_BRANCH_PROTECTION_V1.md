# Oreslang source branch protection contract v1

This is the organization-level release gate for the five private compiler backends:

- `oreslang-source.java`
- `oreslang-source.llvm`
- `oreslang-source.net`
- `oreslang-source.wasm`
- `oreslang-source.forge`

It is a policy target for repository administrators. It does not claim the settings are currently enabled.

**Current unresolved runner-admission incident:** [source-local smoke, post-merge
verification debt, and administrator remediation runbook](ACTIONS_RUNNER_ADMISSION_RECOVERY_20261026.md).
Do not require zero-step failing checks, count unexecuted jobs as passes, or
export private source to an unapproved alternate organization.

## Required `main` protections

Each repository's default `main` branch MUST:

1. report `protected=true` through GitHub's authenticated branch API;
2. require pull requests and at least one approving reviewer distinct from the PR author;
3. require CODEOWNER review for compiler/runtime/workflow/security-policy changes;
4. dismiss stale approvals after new commits;
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
- missing independent approval is blocked;
- a new commit invalidates a stale approval;
- a zero-step/startup-failure check is blocked;
- the exact required checks plus independent approval permit merge.

Record repository ID, protected branch SHA, ruleset/protection readback, check context + issuer App ID, PR SHA/tree, and run/job IDs in the security tracker.

As of 2026-10-17, authenticated API reads report all five backend `main` branches as `protected=false`. This contract therefore remains an unapplied release gate.
