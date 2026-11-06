# Transfer runbook (human-approved, fail closed)

## Non-negotiable scope
A transfer is **one repository at a time** from `ores-truffle-oreslang` to its reviewed final owner in the manifest. `atto-lang` is the canonical language successor. Proposed AttoCo destinations are subject to owner/security signoff, not a mandate to perform two successive transfers. If the owner instead wants every repository to land first in `atto-lang`, revise the manifest and review access/URL impacts before execution. Existing branded sibling organizations remain unchanged.

**No destination organization, transfer API, private-source security approval, destination plan parity, or successful hosted CI has been verified by this planning PR.** The [audit](AUDIT_FINDINGS_20261106.md) records branch protection and runner-admission blockers. Never treat a 403, a zero-step job, a source-only test, or an unexecuted checklist as evidence of a passing gate.

## Preflight checklist — every repository
- [ ] Destination GitHub organization exists and is owned by authorized maintainers; name is available and target repo name has no collision.
- [ ] Source and target admins approve transfer; document intended privacy; review least-privilege, outside collaborators, membership, private fork policy, and any legal/retention restrictions.
- [ ] Identify private consumers, packages, cross-org actions, internal references, webhooks, environments, deploy keys, Pages, branch protection/rulesets, CODEOWNERS, GitHub App installations, releases, and billing entitlements.
- [ ] Freeze or coordinate active PRs and automations. Snapshot source repo **ID, exact default head SHA, Git tree, PR count/open PR numbers, rulesets**.
- [ ] CI evidence: ensure non-zero executed job steps on exact head; hosted runner admission and actual guest execution are distinct from host/static tests.
- [ ] Confirm source repository has a rollback plan and no secret exposure.
- [ ] Require a per-repository [transfer receipt](TRANSFER_RECEIPT_TEMPLATE.md). Its stable ID, exact pre-transfer head SHA/tree, access, protections, Actions executed-step receipts, approval signatures and dependencies must be populated and checked before setting manifest status to `approved`.
- [ ] If the source GitHub branch reports `protected=false`, or protection/ruleset API returns a forbidden/plan error, hold until independently verified protection is configured/enforced at destination. Never count a private branch's unverified ruleset as protection.
- [ ] Record the coordinator/agent ownership claim in [issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26), including precise source repo and frozen SHA, avoiding duplicate work.
- [ ] Have both security reviewer and migration owner mark this specific manifest entry approved.

## Transfer execution (not automated by this plan)
1. Authorized owner uses GitHub UI Settings > General > Danger Zone > Transfer ownership, or an approved scoped API client to POST /repos/{owner}/{repo}/transfer, after confirming owner/repo and transfer approval.
2. Do **not** manually clone private code into a destination as a shortcut. Do **not** cross org checkout with privileged secrets. If funded test carriers are required, their separate exact-tree mirror process must be approved independently.
3. Preserve existing repository name; do not simultaneously rename package IDs or force-update protected refs.
4. Confirm destination repository ID matches the source ID and exact source commit/tree remain accessible. Recheck retained PRs/issues, reviewers, and branch/ruleset protections.
5. Rebind GitHub App installations, Actions permissions, branch/status checks, secrets and environments, deploy keys, webhooks, registry/package identity, signed releases, pages, and external integrations with independent review.
6. Re-run exact-head CI and canaries at the destination; verify executed steps and negative evidence. Ensure no forced-green/skip/receipt substitutions.
7. Replace hard-coded `ores-truffle-oreslang` URLs in a dependency-ordered code/README/workflow PR wave; preserve compatibility aliases where possible.
8. Record transfer completion timestamp, reviewer, source/target URLs, repo ID and final exact commit/tree in change log and update manifest status.

## Abort / hold conditions
- Target org inaccessible, handle not provisioned, plan feature missing, permissions not equivalent, or target repo collision.
- Missing security approval for private source, source/target admin gap, outstanding required branch protection, active release freeze.
- CI zero-step, missing real compiled-guest receipts for parity-sensitive work, unverified exact SHA or submodule references.
- Secrets, webhooks, package registries, Pages, or production deployment cutover cannot be safely reconfigured.

## First pilot
Choose **`oreslang-demos`** (public and zero open PRs at 2026-11-06 audit) only after target org creation and successful security/ownership preflight. Its branch also reported **unprotected**, so this must be remediated before transfer. Check release/download/Pages integrations even for public demos. The earlier `attolang-language-spec.md` pilot is postponed because it is private and has active PRs. The `oreslang-source.*` family moves only after validating pilot redirects, CI evidence, destination rules and dependent automation.
