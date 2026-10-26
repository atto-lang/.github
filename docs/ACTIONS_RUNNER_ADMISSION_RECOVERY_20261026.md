# Oreslang Actions runner-admission recovery — 2026-10-26

**Status: unresolved / administrator action required.** This is an incident
runbook and evidence ledger, not a claim that GitHub Actions or branch rules
have been fixed. Follow [central incident #2](https://github.com/ores-truffle-oreslang/.github/issues/2)
and the [branch-protection contract](SOURCE_BRANCH_PROTECTION_V1.md).

## Minimal source-local reproduction

Private repository: `ores-truffle-oreslang/oreslang-source.js`.

- Branch: `ci/runner-admission-smoke-20261026`
- Exact commit: `900c4d9e6acf48ccedbb1c070809beeebb47edba`
- Workflow: `.github/workflows/source-local-runner-admission-smoke.yml`
- GitHub-hosted runner label: `ubuntu-24.04`
- Trigger: branch-limited push; `permissions: {}`; **no checkout,
  repository source execution, artifacts, caches, privileged secrets, or
  cross-organization copies**.
- [Run 40411364952](https://github.com/ores-truffle-oreslang/oreslang-source.js/actions/runs/40411364952);
  job `121882588267`: completed/failure, **no recorded steps** (`steps: null`).
  The job log download was unavailable (404).

This failure occurred before the trivial Bash verification step. It does not
identify the root cause: **billing, spending cap, workflow policy, runner
eligibility, and GitHub service failure remain hypotheses**. Do not treat
`failure` with zero steps as a failed unit test or a successful test.

## Source-free independent runner and negative-test evidence (NOT backend CI)

On 2026-10-26 an alternate **private** test repository,
`sonus-auris-test/clients-api-contract` (repository ID `1323599063`),
executed [synthetic Actions run 40422046692](https://github.com/sonus-auris-test/clients-api-contract/actions/runs/40422046692)
at test-branch head `c92e4f99eac89d4b56755e062ec655706c123de9`.
Job `121918589780` recorded completed/success with actual checkout, compile
and test steps. It compiled standalone **source-free** Rust using `rustc
--edition=2021 --test`, then reported **12 passed / 0 failed**.
No Oreslang source, Git tree or cross-repository checkout was transferred.

The exact test source has also been retained in this draft recovery PR as
[`tools/source_free_ci_evidence_model.rs`](../tools/source_free_ci_evidence_model.rs).
Its Git blob SHA is `4992708c6396c146746377cde8b3533873f967a4`
in **both** repositories. This proves content identity of the standalone
synthetic fixture, not execution of the Oreslang compiler. To rerun without
Python or external Rust dependencies, use:
```sh
rustc --edition=2021 --test tools/source_free_ci_evidence_model.rs -o /tmp/ores-source-free-evidence-tests
/tmp/ores-source-free-evidence-tests --nocapture
```


Those fixture tests model rejection of missing runner IDs, zero steps,
skipped/failed/queued steps, failed or incomplete jobs, wrong or malformed
commit/tree identities, untrusted workflows, and unapproved cross-repository
destinations. The illustrative positive case is **NOT a release attestation,
not destination approval, and not a substitute for real compiler tests**.

### Mandatory classification of real evidence

- **NOT_EXECUTED** when no runner is allocated, job steps are absent or
  empty, the run/job is skipped/cancelled/queued, or logs are unavailable
  without corroborating executed steps. An accepted rerun API request does
  not count.
- **EXECUTED_FAILED** only when steps actually ran and native tests failed;
  do not label admission failures as regressions.
- **EXECUTED_PASSED** requires a trusted workflow selected independently
  from the PR code; exact source commit AND root tree identity; authoritative
  run and job IDs, assigned runner, nonempty completed successful steps,
  successful native check conclusion, and independent review/branch gates.
  A self-reported fixture bool or synthetic model success proves none of this.
- **CROSS_ORG_BLOCKED** whenever private source would leave the source
  organization without both a reviewed immutable destination ID on protected
  source `main` **and** explicit clearance of the open exposure incidents.
  A private destination, working runner, or user-requested mirroring technique
  alone never authorizes the export.

The synthetic external run is useful only as proof of **independent runner
capacity and fail-closed model tests**. Keep source-local and backend-native
CI conclusions separate in dashboards and release gates.

## Five already-merged PRs still need executable post-merge checks

The PRs below are **already merged**, and representative native or compiler
workflow jobs on their exact merge commits completed with failure and *no
recorded executed steps*. A merged PR is not certified merely by being merged.

| Backend / merged PR | Exact merge commit | Representative failed post-merge run |
| --- | --- | --- |
| [JS #57](https://github.com/ores-truffle-oreslang/oreslang-source.js/pull/57) | `83603743a3980b39b309753be7f6024104ffa2c2` | [40369762437](https://github.com/ores-truffle-oreslang/oreslang-source.js/actions/runs/40369762437) — compiler bootstrap |
| [HDL #19](https://github.com/ores-truffle-oreslang/oreslang-source.hdl/pull/19) | `5fed67f0cba3fc0ad8de49d75778eafde4586ec3` | [40369771830](https://github.com/ores-truffle-oreslang/oreslang-source.hdl/actions/runs/40369771830) — width proof |
| [WASM #126](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/pull/126) | `f72b095b1cb29154417115b2d7348056c87b8f7d` | [40369799462](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/actions/runs/40369799462) — scalar backend |
| [.NET #151](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/151) | `0ee385cda7a2414f5a6befcc1d8f9393f908b356` | [40369815927](https://github.com/ores-truffle-oreslang/oreslang-source.net/actions/runs/40369815927) — Windows Native AOT |
| [Forge #107](https://github.com/ores-truffle-oreslang/oreslang-source.forge/pull/107) | `e0513d495f0b0b30bd6999f402b4dec2f2f1f4b4` | [40369784465](https://github.com/ores-truffle-oreslang/oreslang-source.forge/actions/runs/40369784465) — compiler/runtime |

At the time of this audit the authenticated GitHub branch read reported
`protected: false` on `main` for all five. Submitted review lists for these
five PRs were empty. These facts need explicit security/release follow-up.

## Administrator remediation (do not publish raw credentials)

1. **Confirm org plan and controls.** The connected GitHub App's branch
   protection read returned `403 Resource not accessible by integration`;
   ruleset listing returned `403 Upgrade to GitHub Pro or make this repository
   public to enable this feature` for the private JS repository. Neither an
   App's `admin: true` repo permission in metadata nor a `403` is proof
   that branch protection exists. The organization administrator should
   confirm a plan supporting **private** branch protection (typically Team or
   Enterprise), plus an identity authorized to manage the rules. **Do not
   make the source repositories public** just to obtain free-tier rules.
2. **Check Actions billing in the authenticated UI.** Review organization
   Billing and licensing, Actions usage, payment problems, and the metered
   Actions budget's *stop-usage* policy. Request a budget/payment change
   explicitly from the authorized billing owner; do not silently raise cost
   limits or assume zero-step failures prove exhausted minutes.
3. **Check policy and runner eligibility.** Review organization/repository
   Settings → Actions (workflow access and GitHub-hosted runners), eligible
   runner labels, repository restrictions and any enterprise policies.
   Inspect the failed run's annotations and GitHub Status. If policies
   and billing look valid, escalate the source-local smoke run and job IDs
   to GitHub Support, asking why runner admission failed before steps.
   Provide IDs and sanitized metadata, not raw tokens or private sources.
4. **Verify admission before expensive CI.** In a deliberately isolated,
   reviewed source-local diagnostic ref, rerun the existing *no-checkout*
   smoke test. Require an allocated runner, one or more actual executed
   steps, and a successful result. Do not interpret an accepted rerun API
   call, queued job, or zero-step failure as recovery.
5. **Establish effective `main` protection** before treating any future
   PR as mergeable. Follow `SOURCE_BRANCH_PROTECTION_V1.md`: independent
   approval, CODEOWNER review, stale-review dismissal, no direct pushes or
   hidden administrator bypass, trusted native checks with pinned issuing
   App identity. Activate required status checks only after those checks
   demonstrably execute; test negative/positive protection cases and read
   back `protected=true`.
6. **Rerun real post-merge backends** at *current* immutable `main`
   commit and root tree identities, capturing exact job IDs and nonzero
   steps. Validate JS Node/browser, HDL proofs, WASM AOT/scalar/structural
   parity, .NET Native AOT/serialization, and Forge builds/concurrency/
   sanitizers. Diagnose *actual* test failures separately.
7. **Record release evidence.** Keep reviewed exact-source CI receipts,
   workflow/test provenance, observed check conclusions and protection
   readback in incident #2. Invalidate evidence after any head/tree change.
   No synthetic/source-free carrier result establishes native correctness.

## Cross-organization private source: DO NOT EXPORT

Open P0 incidents [#22](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/22)
and [#77](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/77)
explicitly prohibit further private source exports while exposure/remediation
and independent review are pending. A private alternate runner and working
Actions capacity are **not** sufficient authorization. Do not mirror the
Git tree to a public *or private* unrelated organization, work around the
source destination allowlists, or use tokens to bypass these controls.

## External reference

- [GitHub: troubleshooting Actions](https://docs.github.com/en/actions/how-tos/troubleshoot-workflows)
- [GitHub: budgets and alerts](https://docs.github.com/en/billing/concepts/budgets-and-alerts)
- [GitHub: branch protection rule eligibility](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule)
- [GitHub: private-repository ruleset eligibility](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets)
