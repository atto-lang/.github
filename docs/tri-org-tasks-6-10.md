# Tasks 6–10: cross-organization audit controls (advisory only)

This code introduces **offline, source-free, synthetic regression contracts** for five governance/security workstreams. It does not supersede their established private PRs, do live API reads, update GitHub rulesets, approve reviews, transfer private source, merge, or authorize deployment.

| Task | Existing authoritative implementation | Additional fail-closed property |
| --- | --- | --- |
| 6. One independent approval | ores-truffle-oreslang/.github PR #36 and issue #37 | Require the reviewer's identity, independence from author, current exact SHA, post-push time, CODEOWNERS, issuer-bound status, no bypass/force-push |
| 7. Five-PR/three-repo merge wave | oreslang-platform-conformance PR #584, issue #582 | Reject fewer than five PRs, fewer than three repos, stacked branch, duplicate PR, stale CI SHA, no runner, no named executed tests |
| 8. Private carrier source publication | oreslang-source.java PR #954, issue #962 | Reject name-to-numeric-ID swap, source-commit/tree drift, caller-owned policy, absent protected source and independent approval, prewrite revocation, no destination tree proof |
| 9. Ten landed PRs release audit | ores-truffle-oreslang/.github PR #28, issue #29 | Require ten distinct landed commits with current-main ancestry, an explicitly labeled **post-merge** review, live exact-landed CI with runner/steps, effective protected main, closed findings |
| 10. Cross-org identity inventory | ores-truffle-oreslang/.github PR #25, issue #26 | Require 80 legacy, 68 AttoLang and 17 AttoCo entries with unique immutable numeric IDs, and explicitly no inferred owner / transfer authorization |

**Important:** The actual 165-repository inventory includes private repository names, IDs, and visibility. It must stay within a private, independently controlled repository; the public tests here use **only synthetic names**. Do not copy the private migration manifest into this public repository. The private manifest in PR #25 is the source of truth for planned moves: 74 proposed direct transfers to AttoLang, 6 unresolved holds, and none directly to AttoCo; **no transfer is authorized** by a plan.

## Run

```sh
node --test tests/tri-org-assurance.test.mjs
node governance/tri-org-assurance.mjs snapshot.json
```

The CLI is an **offline advisory** parser of a local regular, non-symlink JSON file (<=2 MiB). Exit 0 means the *supplied, untrusted* fields reported satisfaction; exit 1 means a HOLD; exit 2 means invalid input. Output explicitly has `advisory_only: true`, `certified: false`, `authorized: false`. **Never use this CLI exit code or output to enable merges, CI source exports, payments, ruleset changes, or production releases.**

The public workflow pins the checkout action by commit, asserts the exact proposed PR head and clean tree, and runs adversarial Node tests without network access in the test code. A separate source-free canary job does not check out anything or access secrets. A green **public** canary never clears Oreslang's **private** Actions billing/admission gate.

## Next responsible operational actions

1. Have an authorized owner verify billing and private Actions admission in the original source organization; preserve a real private runner + executed-step receipt.
2. Obtain independent security/admin review of private governance PR #36, merge-wave PR #584, publisher PR #954 and retroactive audit PR #28 only after exact-head hosted CI, effective protected branches and provenance are verified. None is promoted by this supplemental PR.
3. Have the repo migration coordinator review PR #25's existing private manifest and verify unique repository IDs from live GitHub; do not move repos until both source and destination governance/CI are ready.
4. Perform a fresh GET-only administrative readback before any branch-protection or Pages status assertions. Never infer historical approvals from later reviews.

These controls are deliberately separate from release-authorizing code. They provide regression tests for *classification errors*, not proof of external security settings.
