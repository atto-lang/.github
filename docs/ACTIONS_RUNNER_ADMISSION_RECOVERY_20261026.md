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
Its latest hardened Git blob SHA is `ddedc86f5e3ebcbb7db94b352dec5acefcb0671d`
in **both** the Oreslang draft PR and the independently executed private
`usa-acc/usa-acc-e2e` test repository. This proves content identity of the
standalone synthetic fixture, not execution of the Oreslang compiler. To rerun without
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

### Second runner, hardened API-shaped synthetic fixtures

The first rerun of the expanded fixture on `sonus-auris-test` failed **before
steps were recorded**: [run 40423475206](https://github.com/sonus-auris-test/clients-api-contract/actions/runs/40423475206),
both initial attempt and failed-jobs rerun. This is NOT_EXECUTED; it
does not indicate a Rust compiler regression.

A separate private source-free test carrier,
`usa-acc/usa-acc-e2e`, executed [run 40423684313](https://github.com/usa-acc/usa-acc-e2e/actions/runs/40423684313)
at exact carrier HEAD `0114f6c5041d88b9590d7da792e47e2e69e9a2a0`,
job `121924020094`. GitHub reported completed/success with actual
checkout, `rustc --test` compile and test execution steps. The log
reports **23 passed / 0 failed**. The test fixture's Git blob identity
(`ff6eea952de8814208b13061a07ca5dfb8cc124f`) matches this draft
PR. No Oreslang compiler sources were exported.

The additional 11 cases exercise **API-shaped fixtures** for run/job/check
linkage, expected trusted workflow and check-issuer App ID, exact commit,
protected base branch, independent exact-head approval and change requests,
and fail-closed admission classification. Their success is **only a
unit-test result for mock data**. The Rust file still does NOT fetch or
authenticate GitHub API payloads and does NOT verify a real checkout tree,
trusted workflow content, native compiler test semantics, or effective
production branch protections.

**Production adapter follow-up (release-blocking):**
1. Obtain source repository ID, current base-branch protection/ruleset,
   reviews and reviewer collaborator permissions through an **authenticated,
   read-only trusted GitHub installation**, not from PR-supplied fields;
   require that the collector/reviewer identity cannot be controlled by the
   candidate source changes.
2. Read actual workflow run/job/check identities and issuer from the same
   authenticated trust boundary; require checks bound to an immutable
   source head and a separately reviewed workflow definition, not just
   workflow names or a self-asserted `trusted` flag.
3. Verify root Git-tree identity using a **real executed checkout** and
   authenticated immutable result receipt. The synthetic `GithubRun`
   fixture does not include this proof.
4. Verify an independently pinned required *native test* workflow/job
   and actual positive/negative compiler tests. A job named `test`,
   a successful shell, or `Step { conclusion: "success" }` alone is
   insufficient, even if all synthetic fixtures pass.
5. Evaluate each reviewer's latest effective review after the final head,
   detect stale approvals, and independently validate CODEOWNER status.
   The synthetic review model is deliberately conservative and does not
   implement the full GitHub review state machine.
6. Run production negative cases proving missing steps, stale head/tree,
   disabled protections, wrong issuer, missing approval and revoked access
   fail closed before enabling any required checks.

### Additional P0 export safety hardening — 24 verified tests

A follow-up audit found the original synthetic `Receipt` validator could
return `Ok(())` for cross-repository source execution if model-only
`independent_export_approval` / `disclosure_incidents_cleared` Booleans
were set. **That positive path is now removed**: it returns
`cross-organization source export not authorized` for every nonlocal
repository ID, regardless of modeled approval or private visibility.
This deliberately prevents accidental use of editable flags as export
authorization while P0 exposure incidents are still open.

The exact independently compiled and executed revised fixture is
[`usa-acc/usa-acc-e2e` run 40424169727](https://github.com/usa-acc/usa-acc-e2e/actions/runs/40424169727)
at commit `dc94f236783cf3a6351720b2241ad5dc337cb366`,
job `121925686274`: **24 passed / 0 failed** with actual runner steps.
The Rust test blob `45a3d348499ae4f0167dbc32d09732b0653e8f34`
is identical in the carrier and this draft PR. Neither a synthetic
positive test nor private runner availability reauthorizes private-source
transfers. A future genuinely reviewed export system requires a **separate,
explicit security design and approval**, not toggling the retired mock flags.

### Third hardening pass — 25 executed Rust tests; checkout-tree receipt required

A follow-up review removed four obsolete simulated private-source export-approval
fields from the source-local `Receipt` model altogether. Cross-repository
execution remains unconditionally denied. No combination of self-declared
approval flags can transform a foreign repository into an authorized
source-local CI target.

The API-shaped `verify_native_release` fixture previously checked only the
40-character commit, but **did not require the source root Git-tree identity**.
It now requires a separate `GithubCheckoutReceipt` containing the repository,
run and job identifiers, checked commit, and checked root tree. Missing receipts,
wrong trees, wrong checked commits, foreign repository IDs, and mismatched
run/job IDs fail closed. This prevents success in *the unit model* without
an exact-tree receipt. It does **not** make fixture data authentic.

The precise validated version of `tools/source_free_ci_evidence_model.rs`
(Git blob `ddedc86f5e3ebcbb7db94b352dec5acefcb0671d`) was compiled and
executed on an actual private runner at
[`usa-acc/usa-acc-e2e` run 40429213864](https://github.com/usa-acc/usa-acc-e2e/actions/runs/40429213864)
at exact test-carrier commit `2889474d52758313df2bbef87ee9c65b47e0d070`,
job `121942515183`: **25 passed / 0 failed, no Rust compiler warnings**,
with recorded checkout, compile and test steps. The file's Git blob is
identical in the test carrier and this Oreslang PR; no private Oreslang
compiler source crossed an organization boundary.

**Production blocker:** `GithubCheckoutReceipt` is currently a deliberately
synthetic structure. Real authorization requires independently controlled,
authenticated checkout/tree provenance tied to the exact source revision and
trusted workflow identity; neither a JSON `tree_sha` field nor an asserted
receipt from candidate PR code is acceptable. Also obtain effective protected
branch state, independent review history and native backend test checks. These
are not implemented by the fixture and cannot be inferred from 25 passing
source-free tests.

### Source-local authenticated metadata collector — added, not certified

A new Rust/std-only collector,
[`tools/ci_evidence_live.rs`](../tools/ci_evidence_live.rs), performs
**GET-only** authenticated GitHub REST reads via the installed `gh api` CLI.
It accepts only a repository of the form `owner/repo`, a protected `main`
push context, immutable SHA/tree and numeric run/job/check/workflow/issuer
identifiers. It never executes remote code or transfers Git objects.
It fails closed on any inaccessible/malformed REST response.

When invoked **inside a trusted, protected source-local GitHub Actions job**
with a read-only `GITHUB_TOKEN`/GH CLI token and
`GITHUB_REF_PROTECTED=true`, it requests:

- `GET /repos/{owner}/{repo}`: immutable repo ID and private visibility
- `GET /repos/{owner}/{repo}/branches/main`: protected state/current commit
- `GET /repos/{owner}/{repo}/actions/runs/{run_id}`: run, workflow,
  repository, SHA, event, completion and check-suite linkage
- `GET /repos/{owner}/{repo}/actions/jobs/{job_id}`: assigned runner and
  actual completed/successful steps; zero-step or skipped steps fail
- `GET /repos/{owner}/{repo}/check-runs/{check_id}`: GitHub App issuer,
  check SHA/status and suite identity; must match the expected job
- `GET /repos/{owner}/{repo}/git/commits/{commit}`: Git object root tree
  must match the independently configured expected tree

The fixed CLI argument contract is:

```sh
rustc --edition=2021 tools/ci_evidence_live.rs -o /tmp/ci-evidence-readonly
/tmp/ci-evidence-readonly OWNER/PRIVATE_REPO main EXPECTED_COMMIT_SHA EXPECTED_TREE_SHA RUN_ID JOB_ID CHECK_ID WORKFLOW_ID ISSUER_APP_ID
```

**Important:** The expected IDs and commit/tree must be configured in a
separately protected workflow, not passed through untrusted PR parameters.
The GitHub token must have the minimum necessary read permissions
(`contents: read`, `actions: read`, `checks: read` and appropriate
repository metadata); permission errors fail closed. This code does not
create or modify branch rules, workflows, checks or reviews. Neither
the token nor unfiltered GitHub API error messages are printed.

**Deliberate release-block:** even if metadata checks pass, the binary prints
`METADATA_VERIFIED_ONLY` and exits nonzero. It does **not** yet verify
independent author/approver/CODEOWNER review history or obtain a trusted
*executed checkout* tree receipt. Reading the commit Git object's `tree.sha`
cannot prove what a runner built. In addition, `protected=true` alone
does not prove all required review/check rules are effective. This is
not a full release evidence collector until these trusted inputs are
added and cross-checked. Do **not** mark it as a required passing CI check
or promote a result to certified status.

**Independent source-free validation:** the exact collector Rust file has
Git blob SHA `6381db8b2a2fcefc4964be35ad00db2e15d3f540`,
identical to the one compiled in private
[`usa-acc/usa-acc-e2e` Actions run #40431039574](https://github.com/usa-acc/usa-acc-e2e/actions/runs/40431039574)
at test-carrier head `4d1eb81a334cec762db1e5d6eeef958f71da8ea3`,
job `121948473858`. There were **37 passing Rust tests**:
25 existing receipt-model cases and 12 read-only collector
API-response/mock adversarial tests. The job also compiled the real CLI
binary and verified that it exits nonzero with `NOT_CERTIFIED` in an
unprotected Actions environment. All checkout, compile, test and
guard steps actually executed.

No authenticated GitHub metadata reads from an Oreslang source-local job
have been executed; source-org runner admission remains broken. No Oreslang
compiler source or Git tree was exported to the private test carrier.

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
