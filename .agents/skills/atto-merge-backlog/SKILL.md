---
name: atto-merge-backlog
description: Drain AttoLang pull request backlogs or run an authorized merge-when-green queue across repositories. Use for local-first exact-tree testing, dependency stacks, semantic conflict resolution, throughput measurement, and agent intake limits.
---

# Merge the AttoLang PR backlog

Use local testing first when it can execute the required behavior. Keep repositories moving independently and merge verified work as soon as it clears the applicable gates. Aim for three confirmed merges per minute across the organization during a backlog run. Work within the requested scope and permissions.

Load the organization policy and the target repository's instructions before any external write. Follow the current trusted-owner review policy, draft lifecycle, overlap audit, explicit holds, and effective branch protections. Use this procedure with existing governance. [Organization AGENTS.md](https://github.com/atto-lang/.github/blob/main/AGENTS.md)

## Triage and schedule

1. Inventory open Ready and draft PRs in scope. Capture current head and base, changed paths and symbols, linked issues, dependencies, and the specific blocker for each candidate. For a percentage target, keep the opening PR count per repo and track verified merges against that baseline. Count superseded closures separately.
2. Check competing open PRs and drafts across relevant AttoLang repos and the matching legacy repo. Stack genuine dependencies and land the prerequisite that unlocks the most work. For competing implementations, choose a canonical PR, carry forward unique behavior and regression tests, and record each disposition. Resolve conflicts against current main by preserving both intended behaviors where compatible. Retest the resolution.
3. Prepare and test several repos while the merge queue lands green PRs. Start with five local test workers and adjust for CPU, memory, and observed suite time. Rotate after one to three PRs per repo, favor repos below the requested target, and keep an independent queue for each target branch. Move a blocked stack aside with its owner and next action so unrelated repos keep moving.
4. For authorized merge-on-ready operation, start validation when an eligible PR opens Ready or becomes Ready. Requeue on head changes and respect new holds immediately. Use a native merge queue where available. Where PRs land individually, validate each resulting tree. Where a supported queue can land a group atomically, validate the combined tree. When a group fails, split it in dependency order and retest.

## Test locally first

- Fetch pinned head and base SHAs. Compute the proposed integration tree with Git from those commits and any earlier queued changes. Use an isolated checkout or Linux VM with reviewed toolchains and authenticated Git operations outside the test environment. Keep credentials and writable host mounts away from PR code.
- Run the repository's material suites for the changed behavior. Keep test selection in trusted configuration. Use the actual native, browser, Wasm, or compiled guest target when its behavior is part of the change. For compiler changes, run the relevant positive and negative regressions. Use a permitted platform-specific runner when the local machine cannot reproduce a required check. [Testing and evidence guidance](https://github.com/atto-lang/.github/blob/main/TESTING.md)
- Keep dependency and compiler caches warm. Reuse a passing receipt only when the exact Git tree, required suite, toolchain, platform, and relevant test inputs still match. Record repository, PR, source head, base, tested root tree, commands, executed tests, outcome, and log location. Keep private receipts out of public repositories.
- Diagnose billing, runner admission, and code failures separately. Classify zero executed steps as `NOT_EXECUTED`. Use the authorized private carrier fallback for budget or admission failures when local testing cannot cover the job. Verify live authorization and destination privacy, prove capacity with a source-free run, and mirror the exact tree through the GitHub connector. Check every path, mode, blob SHA, and the full root tree. Run against the carrier's local snapshot with its own `GITHUB_TOKEN`. Record executed evidence and clean up temporary refs under the retention policy. Preserve required origin checks. [Zero-step recovery runbook](https://github.com/atto-lang/.github/blob/main/governance/ci-zero-step-recovery.md)

## Land the tested result

Use one coordinated merge writer per target branch and pace GitHub mutations. Immediately before landing, recheck Ready state, labels, review, overlap dispositions, branch rules, required checks, and both head and base. Use GitHub's expected-head guard and a queue or enforced base-freshness gate to prevent a concurrent merge from invalidating evidence. An expected-head guard alone leaves the base free to move. If the tested candidate changes, recompute and revalidate before landing. [GitHub merge queues](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue) · [GitHub API best practices](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api)

Confirm GitHub reports the PR merged and verify the landed commit and root tree against the receipt. If a merge call times out, read the live PR state before retrying. Record the PR URL, landed SHA, and test evidence. Pause the affected branch and investigate if the landed tree differs from the tested tree.

## Keep the backlog from refilling

- Start with one active PR or dependency stack per subsystem, including drafts. Tie each to an issue, owner, acceptance criteria, and a focused test command. When a slot is occupied, have agents finish, repair, or resolve conflicts on existing work. Keep unstarted requests in issues.
- Measure newly opened PRs, confirmed merges, superseded closures, queue age, and time spent preparing, testing, and waiting to merge. An end-to-end benchmark for three per minute is 90 confirmed merges in 30 minutes, including preparation and retries. Keep test replay throughput separate from actual merge throughput.
- If arrivals outpace merges, reduce new assignments and send agents to the oldest actionable blockers. Keep advancing unblocked repos. Report which checks, reviews, or dependencies still prevent the remaining merges.
