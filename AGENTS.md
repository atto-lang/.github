# AGENTS.md — Single Trusted-Owner Approval & Draft Recovery Policy

> **Scope:** All repositories in this GitHub organization, including existing and legacy repositories.
> **Mode:** Stealth development, merge-first integration with auditable checks.
> **Priority:** Finish and integrate existing, ready PRs rather than create redundant PRs. Do not weaken required tests, security controls, CODEOWNERS, or branch protection.

## Single trusted-owner approval, any Ready age

**Exactly one** independently authenticated GitHub `APPROVED` review from **either** `the1mills` (GitHub ID `2148448`) **or** `ORESoftware` (GitHub ID `11139560`) satisfies this organization's human and AI review policy for a Ready pull request. Do **not** require both humans, a second CODEOWNER, ChatGPT/Codex or Claude/Anthropic approval, or reapproval after 72h, 240h, 10 days, or another age boundary. One eligible approval suffices for the PR.

The reviewer cannot be the PR author. For a `the1mills`-authored PR, `ORESoftware` must approve; for an `ORESoftware`-authored PR, `the1mills` must approve. Check verified GitHub identity, actual PR review state and author; comments, requested reviews, AI-generated prose, self-approvals, withdrawn or dismissed approvals never count. Never impersonate a trusted account.

**Approve only once per PR**: retain the one approval after routine new commits and do not request fresh approval based on time or push count. Administrator deployment requires an effective GitHub protection/ruleset requiring 1 approval, accepting either trusted user via correctly ordered CODEOWNERS or trusted identity enforcement, with `dismiss_stale_reviews_on_push=false` and `require_last_push_approval=false`. A review explicitly revoked/dismissed by GitHub is not active. Until those settings actually change, GitHub's stronger existing restrictions remain mandatory and must not be bypassed. Material CI on the **current head or proposed merge-group** still must pass after code changes. A critical security issue, blocking changes request, or HOLD label still blocks merging.

This file is an agent policy, **not** evidence that GitHub settings or review integrations have been deployed. There is no automatic self-approval workaround; user-facing task automation connected as `the1mills` cannot independently approve a `the1mills` PR. A separately authenticated `ORESoftware` account may provide the single required review.

## Draft lifecycle: recover, integrate, then make ready

A draft is a work queue, **not a way to bypass governance or wait out the clock**. An agent working the backlog must actively review draft PRs as well as ready PRs:

1. **Triage each draft:** inspect its head, changed files, open blockers, test evidence, related issues, and current compatibility with the base branch. Identify obsolete or abandoned work and its owner.
2. **Search for overlaps before more coding:** inspect other open PRs in the repo and related org repositories by changed paths, modules, exported APIs, issue/feature intent and dependency relationships. Compare implementations semantically, not merely by filename. Choose which implementation should remain canonical; incorporate missing behavior/tests into the chosen PR, update dependent/competing PRs, or mark a superseded PR for closure with a reason. Never discard unique improvements silently.
3. **Make the draft reviewable:** finish outstanding implementation, resolve conflicts, update docs/specs/tests, record runnable test results (or explicit missing test infrastructure), and write a concise overlap/disposition summary linked to competing PRs.
4. **Automatically promote at 72 hours in Draft:** if the draft has spent **at least 72 consecutive hours in its CURRENT draft cycle**, its required/material tests have genuinely **passed on the exact current head SHA**, labels are freshly verified and **neither `keep-as-draft` nor `not-ready` nor legacy `keep-draft`** is applied, and no hard safety or GitHub restriction prevents a ready transition, an authorized agent **MUST mark the PR Ready for review automatically**. Do not require a human review or wait for the new Ready cycle to age. Perform overlap/competing-PR triage in parallel and record candidates; **unresolved overlap or a base-branch merge conflict alone must NOT postpone the promised green, unlabelled Draft→Ready transition**. Reconcile those issues before merging (before merging), updating the current or competing PR as needed. Explicit critical safety holds and GitHub permission failures still take precedence.
5. **Draft clock definition:** if a PR is opened as Draft, the first draft cycle begins at `pull_request.created_at`. If a Ready PR is legitimately converted back to Draft, the **new** draft cycle begins at that latest authenticated `converted_to_draft` event. Draft age is measured from this start, NOT original PR creation after re-drafting; a new push or test rerun does not restart it. At the first scheduled evaluation at/after **72h**, check passing exact-head tests and labels and promote. If tests fail, remain Draft, repair/test and retry promptly when green (including after 72h); if a hold label exists, remain Draft regardless of test results.
6. **If still blocked:** leave Draft, record exact missing tests/holds/overlap and a next owner/action; revisit every backlog run. Draft time is not a path around the single-approval rule. Following promotion, exactly one eligible approval is required regardless of Ready age.

### Label contract for draft automation

Canonical label names, descriptions, and intended behavior are recorded in [`governance/merge-first-labels.json`](governance/merge-first-labels.json). This file is a provisionable catalog, **not** evidence that labels have been created in every repository or that automation is deployed.

- **`keep-as-draft`** — **canonical persistent human hold**. NEVER automatically convert a Draft to Ready while present, even after 72 hours and successful tests; only an authorized owner/human removes this label.
- **`keep-draft`** — backwards-compatible alias treated exactly like `keep-as-draft`; do not automatically remove existing labels.
- **`not-ready`** — work/readiness blocker. **Never automatically convert a draft to ready** while present. Resolve the stated blocker and remove the label only by an authorized action.
- **All three labels (`keep-as-draft`, `not-ready`, and `keep-draft`) also prohibit automated merging**, even after a manual transition to Ready.
- `overlap-review-needed`, `needs-tests`, or `blocked` are useful optional triage labels, but are advisory unless configured by an enforcing workflow. Labels are case-normalized when checked; do not infer approval from the absence of labels.

Label names alone do **not** create enforcement in GitHub: the authorized automation must read current labels **again immediately before** `markPullRequestReadyForReview` and before merge, and fail closed if labels cannot be read. Do not auto-clear `keep-as-draft`, `keep-draft`, or `not-ready`. Manual transition with a hold label must not be interpreted as permission to merge.

## Mandatory competing/overlapping PR audit — before every merge

**Before authorizing any merge**, the reviewing+merging agent MUST:

1. Search **all currently open ready and draft PRs** that compete with or depend on the target, including other relevant repos, matching changed files, symbols, feature goals, linked issues, APIs, tests, and branch history.
2. Read the relevant diffs and determine whether they duplicate, conflict conceptually, introduce incompatible designs, or contain complementary functionality. A clean Git merge is **not** evidence of semantic independence.
3. **Update the current PR or the competing PR(s) before merging** where overlap exists. Prefer one canonical solution with all needed tests/specs. Rebase/retest impacted code and handle dependencies in the correct order. Record a concrete disposition for each candidate (`incorporated`, `updated`, `superseded`, or `independent with rationale`); leave **no unresolved competing PR**.
4. Re-run required CI on the resulting exact SHA. Verify that GitHub still recognizes the one trusted approval; request reapproval only when the approval is genuinely dismissed or revoked by enforced settings. Re-run overlap discovery if the head or relevant open-PR landscape changes.
5. Record scan time, current SHA, scope of repo/org searches, competing PR URLs and dispositions in the PR or authenticated policy-check evidence. If a relevant diff or overlap search is unavailable, block the **automated merge** pending inspection; never assert `no overlaps` without evidence.

Apply the same overlap integration habit **when recovering drafts and during reviews**; the pre-merge audit applies to all Ready PRs.

## Common merge gates — all windows

- PR is currently **ready**, not draft; no `keep-as-draft`, `keep-draft`, `not-ready`, explicit hold or critical risk.
- GitHub confirms **no merge conflicts**, no outstanding blocking change requests, no critical credential exposure, known critical security flaw, or irreversible data loss.
- All applicable required CI/material tests actually **pass** on the current head/integration SHA. Failed, pending, skipped, cancelled, timed out, absent or unrun checks do **not** count as passing. If Actions minutes are exhausted, use an authorized **token-free Git-tree mirror** to an alternate CI org, with per-blob SHA verification; never leak source secrets.
- Comply with actual GitHub branch rules, CODEOWNERS, checks, and existing stronger protections. Do not unilaterally remove rules to facilitate a merge.
- Authenticate reviewer evidence, lifecycle events, overlap audit evidence and check provenance via an external trusted integration. Status checks and labels cannot be satisfied using attacker-controlled PR text or untrusted workflows.

## Merge-first operating cadence

1. Devote about 80% of effort to finishing existing PRs, resolving conflicts and overlaps, and proving tests on the exact current head.
2. For each Ready PR, check whether exactly **one** `APPROVED` review from eligible `the1mills` or `ORESoftware` already satisfies the requirement; avoid duplicate review requests and discard any historic four-party/age-tier logic.
3. Merge immediately after one valid independent trusted approval, complete material CI, security/overlap audit, clear hold labels, conflict resolution and GitHub-enforced branch restrictions. No artificial age windows or AI-provider quorum.
4. Promote eligible Draft PRs after 72h in the current Draft cycle when material CI is green and no protected hold or hard restriction exists; their subsequent Ready approval requirement is still one.
5. Before merging re-read GitHub reviews, PR head, checks, labels, protection and conflicts. If a reviewer approval was dismissed by old branch settings, seek a replacement once; do not forge it or bypass GitHub. Use `expected_head_sha` for merge operations.

## Deployment, reporting, and precedence

This `AGENTS.md` is the organization's canonical instruction for AI agents; it is **not itself an enforced GitHub rule**. Repository README pointers remain useful but are not a substitute for independently loading the policy. A trusted installed GitHub App/check, accurate Draft/Ready lifecycle timeline, indexed open PRs, authorized 72-hour draft-promotion service, and compatible rulesets are needed for automatic enforcement. The existing Rust evaluator at `gha-indie-worker/.github#52` is standalone unless separately deployed.

Report merged PR URLs/SHAs, readiness-cycle age, draft-to-ready promotions, holds, overlap candidates and dispositions, test evidence, and unresolved blockers. Prefer accurate partial progress over unverified assertions. Preserve repo-specific stronger safety and technical requirements.

<!-- incoming-oreslang-migration:begin -->
## Incoming migration from the legacy Oreslang organization

`atto-lang` is the primary organization for new language and implementation work. Before creating/merging a substantial PR, inspect relevant open and draft work in the corresponding legacy `ores-truffle-oreslang` repo, using the [legacy-to-AttoLang repository crosswalk](https://github.com/ores-truffle-oreslang/.github/blob/main/governance/atto-migration-map.json). This applies to readiness triage and especially to the mandatory competing/overlapping PR audit before any merge. Consolidate independent fixes and tests into the canonical AttoLang PR rather than leaving parallel features to diverge. Similar names are not evidence of parity or completed migration; verify exact commits, contracts and executed tests. Continue to respect labels, current ready-cycle timestamps, and GitHub protections. Do not auto-close, force-update, or discard unique legacy work without verified disposition.
<!-- incoming-oreslang-migration:end -->
