# Trusted merge approvals for ores-truffle-oreslang

Applies to all active repositories in `github.com/ores-truffle-oreslang`, including all compiler backends, CLI, libraries, platforms, documentation, and organization tooling.

## Human-review criterion

An approving GitHub pull-request review from **either** `@oresoftware` **or** `@the1mills` is sufficient to meet the human-review requirement. Do not impose an additional independent reviewer, second approval, or separate CODEOWNER approval after one of these accounts has supplied a valid approval.

- The reviewer must be different from the PR author. GitHub does not allow a PR author to approve their own PR. A PR authored by `@the1mills` can be approved by `@oresoftware`, and vice versa.
- Only an actual `APPROVED` pull-request review counts. A comment, requested review, team membership, admin role, or presumed consent is not an approval.
- A dismissed review, outstanding changes-requested state, or review made stale by later code changes does not satisfy the requirement. Re-evaluate approval against the current PR head before merging.
- If neither account has a valid approval, the human-review requirement remains unmet. An approval by some other account does not substitute for one of the two trusted accounts.
- No procedural or agent-authored **"Merge Without Review"** block should be asserted when a valid review by either trusted account is present.

## Keep technical gates separate

This review-policy simplification is **not** permission to bypass applicable security and CI controls, merge conflicts, failed or unexecuted required tests, missing exact-head evidence, draft status, or GitHub's actual access controls. Private-source publication/export authorization and release certification remain separate decisions; trusted PR approval does not itself authorize source export.

## Enforcement and rollout

Prefer repository settings that require a PR and at least one approval with `CODEOWNERS` configured so either trusted account can satisfy the relevant ownership requirement. GitHub's one-approval count **alone** does not constrain reviewer identity: check the approving login through enforced CODEOWNERS or an independently verifiable required check. Verify CODEOWNERS matching precedence and make sure more-specific rules do not accidentally require a different reviewer.

Where repository settings are unavailable, this document defines how maintainers and automation **evaluate** review evidence; it does not pretend to install branch protection or rulesets. Keep separate reporting for (1) trusted-review satisfaction, (2) GitHub-enforced branch restrictions, and (3) real required-check execution. Do not mark a repository protected merely because this policy document exists.
