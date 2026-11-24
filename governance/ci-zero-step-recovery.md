# AttoLang: mandatory zero-step GitHub Actions recovery

**Applies to every `atto-lang` repository, all `atto-source.*` backends, shared libraries,
`atto-cli`, spec/formatter, conformance, and public artifacts.** Inherits
[canonical AGENTS governance](../AGENTS.md). Repository-specific security rules,
export denials, authorizations, and branch protections take precedence.

## Standing instruction (carry into every CI triage, PR review, and agent run)

> **Unblock CI when a source organization has no runnable Actions capacity.**
> If a GitHub Actions run fails before runner admission (zero actual steps), check
> an **authorized private runner in a different GitHub organization** with
> available capacity. For private sources, that destination's `GITHUB_TOKEN`
> cannot checkout the source repository across organization boundaries. **Never
> supply a PAT, cross-org token, or a source-org secret.** After independent
> source-owner/security authorization for the *exact* source and destination,
> use the GitHub connector to reproduce the source **Git tree** on an isolated,
> temporary branch in that authorized private carrier. The carrier runs against
> its *own local branch/tree*, with only its local `GITHUB_TOKEN` and its
> own runner budget. Verify source tree identity and every blob SHA before tests.

This is an **automatic recovery workflow**, not automatic private-source
export permission or permission to merge.

## Detection and escalation: ALWAYS, not on request

1. For each requested PR/head, inspect its **exact head SHA**, workflow run
   attempt, conclusion and full job records. Check whether a runner was allocated
   and whether **actual steps** executed. A job with `steps: []`, `steps: null`,
   or equivalent empty execution evidence is **`NOT_EXECUTED`**. With
   populated executed steps but missing runner metadata, classify
   **`INCOMPLETE_JOB_EVIDENCE`**; with explicit `runner_id: 0` plus
   purported executed steps, classify **`INCONSISTENT_JOB_EVIDENCE`**.
   The REST jobs response must include **all pages** (`total_count`
   exactly matches the assembled jobs); otherwise **`INCOMPLETE_JOB_EVIDENCE`**.
   Neither missing nor contradictory evidence proves a compiler failure
   or a successful test. A failed *stepful* job is a genuine test/infrastructure
   result; collect logs and fix the actual failure instead.
2. Check source-org Actions budget/admission. If unavailable or repeatedly
   zero-step, **immediately** discover a different org with candidate runner
   capacity. Do not wait for the user to repeat the mirroring trick.
3. Prove capacity using a tiny **source-free** workflow on the prospective
   carrier: real allocated runner, actual steps, real toolchain/test command,
   successful conclusion. Never infer budget from stale runs or merely
   queued jobs. This probe **does not** authorize private source transfer.
4. For a private source, consult its current security policy,
   `.github/private-ci-approved-repositories.txt`,
   `scripts/private-ci-export-policy-guard.sh` where present,
   open incident/denial issues, and permission boundary. **DENIED or
   unapproved means NO MIRROR**—even if a historical transfer succeeded.
   Request authenticated independent source-owner/security approval with
   exact source repo ID, commit/tree, carrier **owner/name + numeric repo ID**,
   readers, purpose, permitted source scope, retention and cleanup. Never
   edit an allowlist or bypass its enforcing guard without the required
   independent policy review; do not use a public carrier for private code.
   Existing denied examples: `atto-source.rs#447`, `#495`; LLVM `#233`.
5. **Only after an effective, reviewed allowlist/approval** construct the
   approved mirror with the GitHub connector using Git objects, no source
   checkout tokens and no secrets:
   - Pin exact immutable source **commit SHA and root Git tree SHA**.
   - Enumerate recursive tree, reject `truncated` results, unknown modes,
     gitlinks/submodules, symlinks and unresolved Git LFS pointers unless
     explicitly understood and authorized. Preserve each path/mode/blob OID.
   - Copy the *contents* of each authorized blob into the private carrier via
     connector Git tree/blob APIs, in bounded batches. An unattached SHA
     reference from another repo does **not** copy the blob.
   - Verify **every** destination blob OID, mode and path, then require the
     final **root tree OID to equal the source tree OID**. Failure or missing
     blob = `MIRROR_INVALID`, no testing or pass claim. A composite job may
     nest multiple exact source root trees under separate prefixes; record
     each independent subtree SHA as well as the composite carrier SHA.
   - Create a temporary, distinct branch/commit in the private carrier;
     never push back to the source branch. Never replace the carrier's
     default branch or reuse unsafe public workflows.
6. The carrier's workflow must check out **its own immutable head** using
   its local `GITHUB_TOKEN` (or credentials disabled), verify the exact
   Git tree(s) and pinned source SHAs, execute real test commands and capture
   actual result codes. Use `permissions: contents: read` or less; disable
   `pull_request_target`, unreviewed source-controlled steps, secrets, artifact
   uploads, caching and external egress where practical. Never run untrusted
   PR code with privileged credentials. A passing source-free probe is not
   a source test.
7. Prove the exact **source revision and actual executed test coverage**:
   formatting, strict static diagnostics, positive and negative conformance,
   native/guest execution (where applicable), sanitizer/AOT/target-specific
   jobs. For compound source trees, ensure *all* dependency versions are
   pinned. Do not claim missing tests or guest execution passed.
8. In the PR/issue, report an immutable receipt: source org/repo ID and commit,
   source tree SHA; authorized destination owner/repo ID; carrier branch/commit
   and verified tree SHA; **blob count**; workflow run ID and attempt;
   runner/job step count; exact test commands/results; remaining
   `NOT_EXECUTED`/unsupported cells; authorization reference; retention plan.
   Cleanup temporary refs per approved retention, remembering that deleting
   a branch does **not** immediately remove private Git objects or caches.
   Never publish a private repository's source contents/logs in a public issue.
9. Run the normal **independent review, conflict, overlap and branch protection**
   gates before any merge. An external green CI receipt cannot by itself bypass
   required on-source status checks; if a required status needs a trusted bridge,
   configure it through authorized governance, not forged checks.

## Status table (fail closed)

| Observation | Status | Next action |
| --- | --- | --- |
| Zero steps or null steps | `NOT_EXECUTED` | Investigate source runner admission; then authorized capacity probe |
| Missing REST job pages, IDs or runner metadata | `INCOMPLETE_JOB_EVIDENCE` | Fetch full real job records; never claim execution or failure |
| `runner_id: 0` but populated executed steps, or success with failed step | `INCONSISTENT_JOB_EVIDENCE` | Verify original run metadata, attempt and provenance; do not certify |
| Stepful checks still running, cancelled or all skipped | `PENDING_OR_INCONCLUSIVE` | Wait or rerun; no pass claim |
| Private source export forbidden/unapproved | `TRANSFER_DENIED` | Security decision; source-free tests only |
| Source-free runner capacity test succeeds | `CARRIER_CAPABLE_ONLY` | Authorization, then exact mirror if permitted |
| Missing/mismatched paths, blob OIDs or root tree | `MIRROR_INVALID` | Stop; reconcile all objects and modes |
| Executed test step fails | `TEST_FAILED` | Inspect logs, patch source, re-pin and rerun |
| Both zero-step jobs and actual failed test steps | `MIXED_ADMISSION_AND_TEST_FAILURE` | Fix the executed test failure and recover the non-admitted jobs independently; neither is green |
| Exact source/head, stepful required test suite succeeds | `TESTS_PASSED_SCOPE` | Report exact covered profile; normal review gate |
| Guest/AOT/backends not executed | `NOT_VERIFIED` for those cells | Keep them blocked; never claim parity |

## Operational notes

- A change to **any** source PR head invalidates that head's test claim.
  Reconstruct and verify its new exact tree, then rerun tests.
- An earlier approval for one destination, source snapshot, issue or carrier
  org cannot be transferred to another destination or source commit.
- If no private carrier is approved, **do not export**. Escalate the incident
  and report `NOT_EXECUTED / TRANSFER_DENIED` prominently; do not silently
  defer or invent a result.
- Prefer **one authorized, scope-limited temporary carrier workflow** for
  related PRs; avoid spraying private source across unrelated organizations.
- Compiler parity means independent compiler checks plus *real generated
  guest binaries*, not just Python snapshots, a synthetic probe, or shared
  host-side tests.

## Automatic inspection (read only)

The canonical tooling includes:

- governance/ci-zero-step-classifier.mjs: classifies *complete* REST job snapshots.
- governance/ci-tree-manifest-verify.mjs: compares full recursive source and carrier Git tree metadata (root OID, every path, object type, mode and OID). Fails closed on truncated GitHub responses, invalid paths, symlinks, gitlinks, missing/duplicate entries and unsupported modes. It outputs `TREE_IDENTICAL_ONLY`, **not** export authorization or executed-test proof; Git LFS payloads still require separate validation.
- governance/ci-zero-step-inspector.mjs: retrieves the immutable run head and attempt,
  fetches every job page scoped to that attempt, verifies IDs and pagination,
  and produces bounded per-run diagnostics. It uses only read-only Actions REST.
- .github/workflows/ci-zero-step-watcher.yml: a scheduled/manual
  reusable workflow. Call it from a repository-owned workflow with
  permissions actions: read and contents: read; the caller's own GITHUB_TOKEN
  can inspect the caller's own run records. Its only checkout is the PUBLIC,
  reviewed atto-lang/.github diagnostic scripts, not private compiler source.
- Each repository's draft .github/workflows/ci-zero-step-admission-watch.yml
  schedules a daily local metadata scan and offers workflow_dispatch for an
  exact run_id. **Call the reusable workflow at a full, immutable,
  independently reviewed commit SHA rather than a mutable `@main`.**
  Do not activate private callers until this governance PR is reviewed and
  merged, the reviewed workflow commit has been pinned by each caller, and
  applicable source-private approval/branch protections have been checked.
  Inside the watcher, the public diagnostic script checkout is separately
  pinned to tested commit `82f050ee5d4043d98c0150458bca743970b390ed`.
  Later source updates require an independently reviewed SHA re-pin.

The watch is *diagnostic*, not a CI test replacement, permission to export
private source, an independent PR review, or a certified test pass. It never
modifies GitHub checks, posts credentials, requests a source checkout, dispatches
a compiler build, or transfers any private code. Run-id and repository inputs
are checked. The scan defaults to the 20 most recent runs, is deliberately
non-exhaustive, and excludes the watcher itself to avoid recursion.

**Important capacity limit:** a scheduled workflow inside the source organization
may itself be unable to start when that org is out of GitHub Actions minutes.
It is not sufficient as the *only* detector. The existing off-host GitHub
connector-based PR/CI admission automations must continue to inspect zero-step
runs and probe alternative private runner capacity using **source-free** tasks
without depending on Actions admission in the blocked org. External approved
carriers may use the public diagnosis scripts; they must not receive private
compiler source until independent source/destination transfer approval and an
enforcing reviewed allowlist are in place.

The inspector distinguishes NOT_EXECUTED from PENDING_OR_INCONCLUSIVE,
INCOMPLETE_JOB_EVIDENCE, INCONSISTENT_JOB_EVIDENCE, STALE_HEAD, and genuinely
failed stepful jobs. A diagnosis is successful when the read-only observation
completed, even when the observed workflow failed. Test certificates still
require independent exact-head provenance and actual required test evidence.


## Off-host monitor and fail-closed recovery handoff

An existing **six-hour CI Admission Watch automation** also uses the connected
GitHub app directly outside source-hosted Actions to inspect AttoLang CI. This
matters because an in-repository scheduled workflow cannot execute during
source-organization runner admission or billing lockouts. The off-host monitor
may diagnose and request source-free capacity probes without any private-source
checkout. GitHub connector rate limits must be respected, and a 403 means STOP,
not aggressive retries.

`governance/ci-recovery-gate.mjs` is a pure **non-authorizing decision engine**
used for the next handoff. It accepts immutable source commit/tree/run identity,
a classified CI diagnosis, and optional source-free private carrier probe
receipts. It distinguishes actual stepful test failures from zero-step
admission failures; rejects public or same-org carrier candidates; and rejects
zero-step, cancelled, mismatched-head, or non-running capacity probes.

**Security boundary:** even when supplied JSON claims export approval,
`ci-recovery-gate.mjs` always reports
`PRIVATE_EXPORT_AUTHORIZATION_REQUIRED` for private/internal source.
An external actor with independent authenticated read access to source export
rules and owner/security approvals must verify these *live* before any
connector private blob transfer. No plan output is an authorization capability.
For public source it may produce `PUBLIC_MIRROR_PLAN_ONLY` but cannot perform
mirroring or certify actual test results. It never executes Actions, writes
GitHub statuses, exports source, or bypasses branch protection.

The off-host watch and in-repository watcher are complementary; neither can
create an authenticated approval, fix GitHub billing automatically, or replace
an actual executed exact-head compiler/conformance test receipt.

## Reaffirmed standing instruction and live capacity receipt (2026-11-21)

The user explicitly reaffirmed: **When the source organization has insufficient GitHub Actions minutes, proactively locate a different GitHub organization with a working runner.** For a private source, the destination's `GITHUB_TOKEN` cannot directly check out the source across organization boundaries. **Only when the exact source-to-private-destination transfer has independent security authorization and an effective allowlist**, construct a temporary destination-local Git-tree mirror through the GitHub connector, using no cross-organization source checkout credential, and verify every path, mode, blob SHA and complete source root-tree SHA before executing real tests. This is a **standing instruction**, not a one-time prompt or exception to private-source security.

**Live source-free proof:** A newly created synthetic-only Rust workflow on private `file-tunnel-test/atto-ci` (repository ID `1458317355`) [run 44949778398](https://github.com/file-tunnel-test/atto-ci/actions/runs/44949778398) executed **three job steps** and **two passing Rust unit tests**. No private compiler source was transferred. This proves **carrier capacity at that moment only**, not permission to copy any private source, ongoing available minutes, or material compiler CI. Newly tested other private carrier candidates `ores-data-platform/atto-ci` and `benefactor-cc/atto-ci` failed with **zero executed steps** and are `NOT_EXECUTED`, not code-test failures. See [independent private transfer approval/incident issue #548](https://github.com/atto-lang/atto-platform-conformance/issues/548) for the authorization/cleanup queue and mutable live evidence.

The enabled **Review, test and merge PRs** recurring automation holds the same fallback instruction for future CI triage. An unmerged PR or an automation prompt is not a GitHub branch protection setting; the independent source/destination security decision, real source CI admission and normal merge gates remain mandatory.
