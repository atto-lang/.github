# AGENTS.md — Readiness-Cycled Merge-First Review & Draft Recovery Policy

> **Scope:** All repositories in this GitHub organization, including existing and legacy repositories.
> **Mode:** Stealth development, merge-first integration with auditable checks.
> **Priority:** Finish and integrate existing, ready PRs rather than create redundant PRs. Do not weaken required tests, security controls, CODEOWNERS, or branch protection.

## Authoritative review clock: READY, not PR creation

The **current ready-for-review cycle starts at the most recent authoritative transition to Ready for review**. For a PR opened directly as ready, it starts at `pull_request.created_at`. A draft's time in draft **never accrues review age**.

If a ready PR is converted **back to draft**, its ready cycle ends immediately; no merger is permitted. On the next **draft → ready** transition, the 72-hour/240-hour clock **restarts at zero**, whether that happens after an hour or many days. Previous-cycle approvals do **not** satisfy the new cycle. Commit pushes, rebases, labels, reviewer requests, and reopenings do not restart an active ready cycle.

Source the timeline from authenticated GitHub PR lifecycle events (`ready_for_review`, `converted_to_draft`, and initial draft state), not editable PR-body timestamps or labels. Reconcile GitHub's timeline and current draft flag; if event history, timestamp, or state is ambiguous or missing, **fail closed**, not `created_at`-fallback for formerly drafted PRs. Record UTC ready-cycle start and event IDs in the audit. **Do not toggle draft/ready just to reset age or avoid renewed review**: a draft conversion must represent actual incomplete work, documented blockers, or material rework.

| Time since start of **current ready cycle** | Required independent reviews | Action |
| --- | --- | --- |
| `0 ≤ ready_age < 72h` | OpenAI Codex/ChatGPT + Anthropic Claude/Opus + authenticated `ORESoftware` + authenticated `the1mills` | Merge immediately as soon as all four and the common gates pass; never wait for day 3. |
| `72h ≤ ready_age < 240h` | **One** independently authenticated, qualified human or verified AI/agent approval | **Fast lane:** perform the competing/overlapping-PR integration audit described below; update the current PR or a competing PR where needed; then merge immediately once all gates pass. |
| `ready_age ≥ 240h` | **Fresh, post-boundary approvals from all four** named parties | Re-review on the current SHA and merge as soon as all gates pass. |

At **exactly** 72 hours the fast lane opens. At **exactly** 240 hours it closes. Recalculate immediately before merging and at every relevant transition.

All counted reviews must cover the current head SHA, must be submitted in the **current ready cycle**, and must be newer than the applicable 240-hour re-review boundary. Human identities must be verified by immutable GitHub account ID; AI evidence needs independently authenticated provider provenance and separate family identities. No author self-approval, unverified AI comment, repeated reviewer, revoked review, dismissed review, or stale SHA may count. If a designated human authored a PR, require explicit independent authorized substitute approval rather than silently waive the requirement.

## Draft lifecycle: recover, integrate, then make ready

A draft is a work queue, **not a way to bypass governance or wait out the clock**. An agent working the backlog must actively review draft PRs as well as ready PRs:

1. **Triage each draft:** inspect its head, changed files, open blockers, test evidence, related issues, and current compatibility with the base branch. Identify obsolete or abandoned work and its owner.
2. **Search for overlaps before more coding:** inspect other open PRs in the repo and related org repositories by changed paths, modules, exported APIs, issue/feature intent and dependency relationships. Compare implementations semantically, not merely by filename. Choose which implementation should remain canonical; incorporate missing behavior/tests into the chosen PR, update dependent/competing PRs, or mark a superseded PR for closure with a reason. Never discard unique improvements silently.
3. **Make the draft reviewable:** finish outstanding implementation, resolve conflicts, update docs/specs/tests, record runnable test results (or explicit missing test infrastructure), and write a concise overlap/disposition summary linked to competing PRs.
4. **Automatically promote at 72 hours in Draft:** if the draft has spent **at least 72 consecutive hours in its CURRENT draft cycle**, its required/material tests have genuinely **passed on the exact current head SHA**, labels are freshly verified and **neither `keep-as-draft` nor `not-ready` nor legacy `keep-draft`** is applied, and no hard safety or GitHub restriction prevents a ready transition, an authorized agent **MUST mark the PR Ready for review automatically**. Do not require a human review or wait for the new Ready cycle to age. Perform overlap/competing-PR triage in parallel and record candidates; **unresolved overlap or a base-branch merge conflict alone must NOT postpone the promised green, unlabelled Draft→Ready transition**. Reconcile those issues before merging (especially during the 72–240h Ready fast lane), updating the current or competing PR as needed. Explicit critical safety holds and GitHub permission failures still take precedence.
5. **Draft clock definition:** if a PR is opened as Draft, the first draft cycle begins at `pull_request.created_at`. If a Ready PR is legitimately converted back to Draft, the **new** draft cycle begins at that latest authenticated `converted_to_draft` event. Draft age is measured from this start, NOT original PR creation after re-drafting; a new push or test rerun does not restart it. At the first scheduled evaluation at/after **72h**, check passing exact-head tests and labels and promote. If tests fail, remain Draft, repair/test and retry promptly when green (including after 72h); if a hold label exists, remain Draft regardless of test results.
6. **If still blocked:** leave Draft, record exact missing tests/holds/overlap and a next owner/action; revisit every backlog run. Draft time **never** counts toward the Ready 3–10-day merge window. On successful promotion, start a **fresh 0h Ready review clock**; require four reviews until Ready age reaches 72h.

### Label contract for draft automation

Canonical label names, descriptions, and intended behavior are recorded in [`governance/merge-first-labels.json`](governance/merge-first-labels.json). This file is a provisionable catalog, **not** evidence that labels have been created in every repository or that automation is deployed.

- **`keep-as-draft`** — **canonical persistent human hold**. NEVER automatically convert a Draft to Ready while present, even after 72 hours and successful tests; only an authorized owner/human removes this label.
- **`keep-draft`** — backwards-compatible alias treated exactly like `keep-as-draft`; do not automatically remove existing labels.
- **`not-ready`** — work/readiness blocker. **Never automatically convert a draft to ready** while present. Resolve the stated blocker and remove the label only by an authorized action.
- **All three labels (`keep-as-draft`, `not-ready`, and `keep-draft`) also prohibit automated merging**, even after a manual transition to Ready.
- `overlap-review-needed`, `needs-tests`, or `blocked` are useful optional triage labels, but are advisory unless configured by an enforcing workflow. Labels are case-normalized when checked; do not infer approval from the absence of labels.

Label names alone do **not** create enforcement in GitHub: the authorized automation must read current labels **again immediately before** `markPullRequestReadyForReview` and before merge, and fail closed if labels cannot be read. Do not auto-clear `keep-as-draft`, `keep-draft`, or `not-ready`. Manual transition with a hold label must not be interpreted as permission to merge.

## Mandatory competing/overlapping PR audit — especially fast lane

**Before authorizing any merge in the 72h–240h fast lane**, the reviewing+merging agent MUST:

1. Search **all currently open ready and draft PRs** that compete with or depend on the target, including other relevant repos, matching changed files, symbols, feature goals, linked issues, APIs, tests, and branch history.
2. Read the relevant diffs and determine whether they duplicate, conflict conceptually, introduce incompatible designs, or contain complementary functionality. A clean Git merge is **not** evidence of semantic independence.
3. **Update the current PR or the competing PR(s) before merging** where overlap exists. Prefer one canonical solution with all needed tests/specs. Rebase/retest impacted code and handle dependencies in the correct order. Record a concrete disposition for each candidate (`incorporated`, `updated`, `superseded`, or `independent with rationale`); leave **no unresolved competing PR**.
4. Re-run required CI on the resulting exact SHA and collect a fresh independent review if SHA changes. Re-run overlap discovery if the head or relevant open-PR landscape changes.
5. Record scan time, current SHA, scope of repo/org searches, competing PR URLs and dispositions in the PR or authenticated policy-check evidence. If a relevant diff or overlap search is unavailable, block the **automated fast-lane merge** pending inspection; never assert `no overlaps` without evidence.

Apply the same overlap integration habit **when recovering drafts and during initial/re-renewed reviews**; the explicitly mandatory *pre-merge audit gate* is the fast lane.

## Common merge gates — all windows

- PR is currently **ready**, not draft; current ready-cycle history verified; no `keep-as-draft`, `keep-draft`, `not-ready`, explicit hold or critical risk.
- GitHub confirms **no merge conflicts**, no outstanding blocking change requests, no critical credential exposure, known critical security flaw, or irreversible data loss.
- All applicable required CI/material tests actually **pass** on the current head/integration SHA. Failed, pending, skipped, cancelled, timed out, absent or unrun checks do **not** count as passing. If Actions minutes are exhausted, use an authorized **token-free Git-tree mirror** to an alternate CI org, with per-blob SHA verification; never leak source secrets.
- Comply with actual GitHub branch rules, CODEOWNERS, checks, and existing stronger protections. Do not unilaterally remove rules to facilitate a merge.
- Authenticate reviewer evidence, lifecycle events, overlap audit evidence and check provenance via an external trusted integration. Status checks and labels cannot be satisfied using attacker-controlled PR text or untrusted workflows.

## Merge-first operating cadence

1. Allocate about **80%** of PR-management effort to finishing existing PRs; prefer completing, merging, or properly consolidating existing work before opening more.
2. **Every pass:** assess drafts for promotion and ready PRs for immediate merge; search and reconcile duplicate work. Prefer foundational dependencies and unblock downstream PRs.
3. **Ready + 72h:** examine all fast-lane-eligible PRs and merge qualifying, green, conflict-free work **after overlap audit**.
4. **Ready + 7d:** escalate tests, semantic conflicts, competing PR decisions and missing review evidence.
5. **Ready + 9d:** prioritize executable integrations before the ten-day cutoff.
6. **Ready + 10d:** demand fresh four-party reviews; never reuse earlier approvals, and merge immediately once renewed gate passes.
7. **Draft backlog:** at/after **72h of the current Draft cycle**, automatically mark Ready when current-head material tests pass, authenticated labels have none of `keep-as-draft`, `not-ready` or legacy `keep-draft`, and no hard restrictions apply. If tests are not green, repair and rerun; promote promptly when green. Never confuse Draft age with Ready review age.
8. Before any action, refresh lifecycle, head SHA, draft status, labels, related PR landscape, reviews, checks, protections and mergeability. Use expected-head-SHA on merge. Never treat policy docs, stale checks or an isolated unit test as live enforcement.

## Deployment, reporting, and precedence

This `AGENTS.md` is the organization's canonical instruction for AI agents; it is **not itself an enforced GitHub rule**. Repository README pointers remain useful but are not a substitute for independently loading the policy. A trusted installed GitHub App/check, accurate Draft/Ready lifecycle timeline, indexed open PRs, authorized 72-hour draft-promotion service, and compatible rulesets are needed for automatic enforcement. The existing Rust evaluator at `gha-indie-worker/.github#52` is standalone unless separately deployed.

Report merged PR URLs/SHAs, readiness-cycle age, draft-to-ready promotions, holds, overlap candidates and dispositions, test evidence, and unresolved blockers. Prefer accurate partial progress over unverified assertions. Preserve repo-specific stronger safety and technical requirements.

<!-- legacy-to-atto-migration:begin -->
## AttoLang is the primary development organization — migration routing

**AttoLang is the canonical successor organization:** https://github.com/atto-lang . This `ores-truffle-oreslang` organization contains legacy repositories, transitional work, and historical PRs. Before implementing, reviewing, consolidating, promoting a draft, or merging a legacy PR, agents MUST consult [the repository-by-repository AttoLang crosswalk](governance/atto-migration-map.json) and inspect its linked `atto-lang` counterpart (if mapped). Prefer new feature implementation, authoritative language/spec work, and the canonical PR in the AttoLang repository. For substantial legacy changes, assess relevant open/draft PRs in **both** organizations, preserve unique work, and port/migrate it to the corresponding `atto-lang` PR or default branch when authorized. Avoid creating a second independent implementation that will diverge.

A crosswalk match means the destination **repository exists**; it is NOT evidence of migrated history, semantic parity, tested execution, or approval to close a PR. Validate exact source/target SHAs, dependencies, CI, and feature semantics before treating anything as migrated. When the crosswalk marks a repository unmapped, route agents to https://github.com/atto-lang and explicitly record that the destination requires a confirmed decision; never invent a counterpart. Do not automatically close old PRs or force-push solely because a similarly named Atto repository exists. Apply urgent legacy-only fixes where necessary, but plan corresponding AttoLang fixes and keep the review clock and hold-label rules intact.
<!-- legacy-to-atto-migration:end -->
