# Trusted merge approvals for atto-lang and its legacy Oreslang counterpart

Applies to all active repositories in `github.com/atto-lang`. Legacy Oreslang repositories must explicitly adopt this rule in their own active GitHub settings; this document does not install protections across organizations.

## Human-review criterion

An approving GitHub pull-request review from **either** `@oresoftware` **or** `@the1mills` satisfies the **single human-approval slot** at every Ready age. Do not require both humans or another CODEOWNER approval. The independent OpenAI and Anthropic AI attestations required during the first 72 hours and from 240 hours onward remain mandatory; from 72 hours up to 240 hours the fast-lane overlap audit applies instead. See [`AGENTS.md`](../AGENTS.md) for the authoritative lifecycle and gates.

- The reviewer must be different from the PR author. GitHub does not allow a PR author to approve their own PR. A PR authored by `@the1mills` can be approved by `@oresoftware`, and vice versa.
- Only an actual `APPROVED` pull-request review counts. A comment, requested review, team membership, admin role, or presumed consent is not an approval.
- An explicitly dismissed or revoked review, or an outstanding changes-requested state, does not satisfy the requirement. Configure GitHub so an ordinary new commit does not automatically dismiss the previously accepted one-time review; re-read its *actual* state before merging.
- If neither account has a valid approval, the human-review requirement remains unmet. An approval by some other account does not substitute for one of the two trusted accounts.
- Do not assert a procedural **"Merge Without Human Review"** block once either eligible account has approved; separately enforce applicable AI attestations, fast-lane overlap audits, CI, and draft holds.

## Keep technical gates separate

This review-policy simplification is **not** permission to bypass applicable security and CI controls, merge conflicts, failed or unexecuted required tests, missing exact-head evidence, draft status, or GitHub's actual access controls. Private-source publication/export authorization and release certification remain separate decisions; trusted PR approval does not itself authorize source export.

## Enforcement and rollout

Prefer repository settings that require a PR and exactly one approval with `CODEOWNERS` configured so either trusted account can satisfy the relevant ownership requirement. To avoid repeated reviews, set `dismiss_stale_reviews_on_push=false` and `require_last_push_approval=false` if the administrator has permission; retain mandatory CI and other safety gates. GitHub's one-approval count **alone** does not constrain reviewer identity: check the approving login through enforced CODEOWNERS or an independently verifiable required check. Verify CODEOWNERS matching precedence and make sure more-specific rules do not accidentally require a different reviewer.

Where repository settings are unavailable, this document defines how maintainers and automation **evaluate** review evidence; it does not pretend to install branch protection or rulesets. Keep separate reporting for (1) trusted-review satisfaction, (2) GitHub-enforced branch restrictions, and (3) real required-check execution. Do not mark a repository protected merely because this policy document exists.
