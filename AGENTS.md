# AGENTS.md — Merge-First Stealth Development Policy

> **Scope:** All repositories belonging to this GitHub organization.
> **Operating mode:** Stealth development / rapid integration.
> **Primary objective:** Reduce the open pull-request backlog by integrating eligible work. Do not optimize for the number of PRs created.

## Default behavior: merge first

**A conflict-free, otherwise eligible PR should be merged by default, not left open awaiting perfect test coverage, repeated audits, or extra hardening.** Agents carrying out authorized PR maintenance may merge eligible PRs without asking for approval on each one.

1. **Merge before creating.** Search for existing PRs addressing the task before opening another. Prefer completing and merging those PRs over duplicating work.
2. **Merge eligible PRs promptly.** If the PR is not on hold, has no merge conflicts, and GitHub allows the merge, merge it. Do not stop after the first success; continue through the eligible backlog.
3. **Tests are advisory in stealth mode.** Pending, skipped, unavailable, or failing noncritical CI checks do not by themselves justify delaying a merge. Record known failures and handle them after integration.
4. **Respect enforced protections.** Never bypass required reviews, branch protections, repository rulesets, or other GitHub-enforced restrictions. Report them as blockers instead of attempting to evade them.
5. **Avoid CI-capacity deadlocks.** Exhausted GitHub Actions minutes are not a reason to leave otherwise mergeable PRs idle. If running tests is important, an authorized alternate runner/organization may be used with a token-free mirror of the exact Git tree; verify source blob SHAs, avoid cross-org secrets, and do not delay safe integration solely to obtain a run.
6. **Resolve conflicts semantically.** Reconcile both sides' intended behavior rather than blindly choosing one branch. Validate conflict resolution sufficiently to detect obvious breakage, then merge.
7. **Don't over-harden before integration.** Avoid repetitive review, refactoring, and test loops that keep otherwise eligible PRs open. Open follow-up issues or small post-merge PRs for nonblocking improvements.
8. **Merge in dependency order.** Prioritize shared contracts, language specifications, parsers/compilers, runtime foundations, and prerequisites before dependent repositories or documentation that assumes them.
9. **Use batch processing.** Work across multiple repositories and target 15–25 successful merges per maintenance run when the eligible backlog and execution budget allow. Spend approximately 80% of PR-management effort integrating existing work and 20% on new PRs.
10. **Verify outcomes.** After each merge, confirm GitHub reports it as merged. Never describe a queued, attempted, or blocked merge as successful.

## Safety and hold exceptions

Do **not** automatically merge:
- PRs explicitly labeled or described as **HOLD**, **DO NOT MERGE**, or otherwise intentionally deferred.
- PRs containing exposed secrets or credentials, known critical security vulnerabilities, or credible risks of irreversible data loss.
- Changes that would knowingly break essential production/infrastructure controls or violate an explicit repository-specific safety requirement.
- PRs blocked by GitHub-enforced rules or genuinely conflicting with their target branch.

Document the precise blocker and route exceptions for focused review. Do not treat every ordinary test failure as a safety exception. This policy does not grant permission to alter protected-branch settings or disable required checks; that requires separate authorization.

## PR maintenance algorithm

1. Inventory open PRs across the organization, including age, mergeability, dependencies, labels, and GitHub-enforced blockers.
2. Merge immediately eligible PRs, with priority to the oldest ready PRs and foundational dependencies.
3. Resolve straightforward merge conflicts conceptually, then reassess mergeability.
4. For uncertain or genuinely blocked PRs, record a concrete next action and move to the next eligible PR instead of stalling the entire batch.
5. Create new PRs only when an existing one cannot reasonably be extended or the work is genuinely distinct.
6. Repeat until the available batch budget is used or no eligible PRs remain.

## Report after each run

Provide:
- **Merged:** exact repository names, PR numbers/links, and confirmed merge results.
- **Conflicts resolved:** PRs and the semantic resolution.
- **Blocked:** PRs with specific blockers (policy, conflict, security, dependency, etc.).
- **Backlog:** open PR count before and after, net change, and oldest outstanding PR when available.
- **Follow-ups:** actionable post-merge defects, CI failures, or hardening tasks.

**Success is measured by merged PRs and shrinking backlog, not PR creation, test perfection, or length of review.**

## Precedence

Follow higher-priority instructions, explicit user requests, and applicable repository policies. If a repository-specific `AGENTS.md` imposes stronger safety or compatibility requirements, honor those requirements; this organization-level policy is the default where no stricter local rule applies.
