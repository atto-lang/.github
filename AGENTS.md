# AGENTS.md — Time-Windowed Merge-First Review Policy

> **Scope:** All repositories in this GitHub organization, including legacy repositories.
> **Mode:** Stealth development with rapid, auditable integration.
> **Primary objective:** Merge eligible PRs promptly, keep the backlog small, and preserve mandatory tests and time-bounded independent review.

## Mandatory PR review windows

All durations are measured from the immutable GitHub `pull_request.created_at` timestamp, in UTC. Reopening, rebasing, changing labels, updating commits, or requesting review must **not** reset the PR age.

| Elapsed age | Review gate | Merge action |
| --- | --- | --- |
| **0 ≤ age < 72 hours** | Verified reviews from **both designated AI families** (OpenAI Codex/ChatGPT AND Anthropic Claude/Opus) **and both designated humans** (`ORESoftware` AND `the1mills`) | Merge immediately once all four reviews and common merge gates pass. Do **not** wait until day 3. |
| **72 ≤ age < 240 hours** | **One** qualified, independent, positively recorded reviewer is sufficient: either a designated human or a verified AI reviewer/agent. | Merge immediately when one valid review, passing tests, and the common merge gates are satisfied. This is the **7-day accelerated merge window**. |
| **Age ≥ 240 hours** | **Fresh, post-240-hour reviews from all four designated parties**: both AI families and both humans | Return to strict review; merge immediately after the refreshed reviews and common merge gates pass. |

At exactly 72 hours the one-review window opens. At exactly 240 hours it closes. Determine eligibility **at merge time**, not when a job began. Previously collected approvals may remain recorded for audit, but approvals predating the 240-hour boundary do **not** satisfy the final strict re-review requirement. Reviews and AI evidence must pertain to the **current PR head commit SHA**; stale or superseded approvals never count.

**Verified independent review:** A reviewer must not be the author; different AI families must each provide their own authenticated review evidence. An AI-generated PR comment, unverifiable label, self-declared reviewer, or untrusted workflow output is not an approval. A human approval must come from the correct authenticated GitHub identity. If a designated human is the PR author, use a separately authorized independent substitution; do not count self-approval or silently waive the review.

## Common merge gates — every window

- **No merge conflicts.** Resolve conflicts semantically, with the surrounding implementation and history; recheck the actual mergeability of the current head.
- **Tests pass.** All applicable, configured required checks and material tests must have an actual **successful** result on the exact current PR head/integration SHA. Failed, pending, skipped, timed-out, cancelled, or never-run tests are **not passes**. A missing CI budget is not passing evidence. Where necessary, use an authorized alternative CI organization with a **token-free exact-tree mirror and per-blob SHA verification**. Never expose source-repository secrets across organizations.
- **No unresolved blocking review, explicit HOLD, critical credential exposure, known critical security flaw, irreversible data loss, or other high-severity safety risk.**
- **Respect GitHub rulesets, CODEOWNERS, branch protections, required checks, and permissions.** Never bypass or weaken them to claim a merge. Record mismatches between this time-window policy and native GitHub rules as blockers requiring separately authorized policy-enforcement changes.
- **Auditability.** Decisions must bind immutable repo/PR identity, `created_at`, exact head SHA, verified reviewer identity/provenance, CI results, and current GitHub mergeability; fail closed if those are unavailable. No self-approval, duplicate AI-family counting, recycled approval across PRs, or replayed exception permits.

## Merge-first working instructions

1. **Merge before creating.** Search the relevant organizations for open PRs solving the task and prefer completing those over creating duplicates.
2. **Prioritize ready PRs and day-9 urgency.** Merge immediately once a PR passes its currently applicable gate. In the 72–240-hour fast lane, actively resolve blockers before the 10-day deadline. Do not artificially age a PR to bypass four reviews.
3. **Use batches.** Target 15–25 confirmed merges per maintenance run as backlog allows; allocate approximately **80% of PR-management effort to integrating existing work** and 20% to newly proposed work.
4. **Prefer foundational dependencies first** and avoid overly long review/hardening loops. Follow-up defects may go into new issues or PRs, but never misclassify a failing required test as nonblocking.
5. **Do not stop at one PR.** Continue through all eligible PRs in each batch, and move on from genuinely blocked PRs after recording exact next steps.
6. **Do not force, rewrite protected history, or bypass hooks/rulesets.** Preserve all repository-specific technical, language, runtime, security, and conformance requirements.

## PR scheduling and alerts

- **At creation:** Request the two named human reviewers and both AI-family reviews in parallel.
- **At 72 hours:** Re-evaluate; if one verified independent approval and common gates are satisfied, merge now.
- **At 7 days:** Escalate outstanding tests, conflicts, or missing approval evidence.
- **At 9 days:** Make eligible PRs top priority, before the fast lane closes.
- **At 10 days:** Require four **new** post-240-hour approvals. Notify all four immediately, and keep the PR blocked until satisfied.
- **At every update/merge attempt:** Re-read `created_at`, current head SHA, reviews, CI checks, labels, protections, and mergeability. Recheck the time boundary at the moment of merge.

## Reporting

Report exact repositories and PR URLs, confirmed merges and merge SHAs, conflicts resolved, CI/evidence status, the active age window, reviewer coverage, pending code-owner/security blockers, and open-backlog before/after. Never claim a queued attempt, doc-only policy, or standalone evaluator is an enforced rule or a successful merge.

## Policy deployment and precedence

This `AGENTS.md` is the canonical **human/agent instruction** for this organization. Repository `README.md` and root `AGENTS.md` files should link to it. A README link alone **does not** automatically activate agent instructions in all AI tools. Repo-specific instructions remain in effect; any stronger safety requirement is preserved. A live time-dependent merge gate requires a trusted, installed GitHub App/check with verified external AI review provenance and compatible repository rulesets; documentation alone is **not** enforcement.

Follow explicit user instructions, applicable law and GitHub-enforced restrictions; do not lower repository protections unilaterally.
