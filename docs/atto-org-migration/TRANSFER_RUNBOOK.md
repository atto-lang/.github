# Transfer runbook (human-approved, fail closed)

## Preflight checklist — every repository
- [ ] Destination GitHub organization exists and is owned by authorized maintainers; name is available and target repo name has no collision.
- [ ] Source and target admins approve transfer; document intended privacy; review least-privilege, outside collaborators, membership, private fork policy, and any legal/retention restrictions.
- [ ] Identify private consumers, packages, cross-org actions, internal references, webhooks, environments, deploy keys, Pages, branch protection/rulesets, CODEOWNERS, GitHub App installations, releases, and billing entitlements.
- [ ] Freeze or coordinate active PRs and automations. Snapshot source repo **ID, exact default head SHA, Git tree, PR count/open PR numbers, rulesets**.
- [ ] CI evidence: ensure non-zero executed job steps on exact head; hosted runner admission and actual guest execution are distinct from host/static tests.
- [ ] Confirm source repository has a rollback plan and no secret exposure.
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
Choose `attolang-language-spec.md` only after target org creation and successful security/ownership preflight. Treat it as a pilot, not an authorization to move compiler repositories. The `oreslang-source.*` family moves only after validating pilot redirects and dependent automation.
