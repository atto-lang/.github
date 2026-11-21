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
