# Compiler, runtime and conformance testing policy

## Proof levels: do not substitute one for another

| Level | What it proves | What it does **not** prove |
| --- | --- | --- |
| Source/schema/formatter checks | Static source/contracts are internally well-formed | Executable behavior |
| Host Rust/JUnit/C++ unit or model checks | An algorithm or host-side component meets its tested properties | Guest compiler/runtime correctness |
| Compiler diagnostics | Source is accepted or rejected on a named exact target | Generated code ran |
| Compiled guest execution | Real emitted executable/Wasm/JVM guest produced expected results | Other backends or security properties |
| Cross-backend conformance | Same portable contract passed across individually evidenced targets | Untested profiles, platforms or unrun cases |
| Isolation/security/real-time qualification | A bounded adversarial or measured property passed for a specific deployment profile | A general security or hard-real-time guarantee |

The only allowed result labels are **PASSED**, **FAILED**, **UNSUPPORTED** and **NOT_EXECUTED**. A skipped, cancelled, queued, zero-step or runner-admission-failed workflow is NOT_EXECUTED for the affected tests, never PASSED.

## Required test matrix for compiler/runtime changes

- **Positive:** supported syntax to typed IR, emitted artifact, actual execution with expected values/output/exit code and target ABI.
- **Negative:** invalid syntax/types/ownership/effects/permissions must fail with the expected diagnostic and **no success-shaped artifact**; unsupported features must fail closed.
- **Ownership:** use-after-take/drop, outstanding loans, mutable/shared conflicts, escaped borrows, cycles/aliases in deep copy, frozen/shared publication, suspension/cancellation boundaries.
- **Actors:** concurrent producers, one-at-a-time serial receive when required, FIFO admission, mailbox closure, supervisor notices, panic isolation, torn-down actor state, state access and stop/restart races.
- **Security:** hostile source, resource limits, malformed package metadata, path traversal, symlink/hardlink abuse, integer/offset overflow, tenant separation and capability-denied host operations.
- **Backend:** source-to-executable proof on the actual target; release/AOT if claimed; memory/lifetime sanitizer passes on qualifying native builds; browser/worker tests for Wasm; exact platform on .NET/Native Image claims.
- **Cross-backend:** same versioned fixtures and byte/value/diagnostic oracles; explicitly mark absent capabilities UNSUPPORTED instead of silently dropping tests.

Prefer Rust for **all new** test drivers, property/model harnesses and CI utilities in atto-lang. Shell can launch tools minimally; do not add new Python tests, pilots or generators. Legacy Python migration must retain equivalent fixtures and negative-test coverage before deletion; see [CONTRIBUTING.md](CONTRIBUTING.md).

## Reproducible evidence receipt

Each feature/backend result must specify:

- repo, branch, **full immutable source commit SHA**, **full root Git-tree SHA**, PR head and exact build inputs;
- versioned specification/conformance fixture IDs and fixture hashes;
- OS/architecture/target triple, compiler/SDK/runtime versions, build flags and execution command;
- executed tests (positive, negative, guest vs host), count, failures/skips, output/exit evidence;
- generated executable or Wasm SHA-256, provenance/artifact URL and relevant job/step logs;
- status **per backend and per capability**, including NOT_EXECUTED when the runner never performed the action;
- named reviewer and unresolved qualification limitations.

Prefer immutable, independent exact-head CI. A green workflow for a different SHA, a static model, an IR schema-only test, or a source-free canary cannot promote guest support.

## Private CI capacity and carrier safeguards

If a private source organization lacks Actions runner budget, an approved **private** carrier may run an exact mirror, but only after authorization of both source and destination. Do not use a public test repository, cross-organization checkout secrets, or an arbitrary branch-copy script.

Require a guarded publisher which rechecks source/destination visibility and an explicit carrier allowlist *inside* the Git object write path; copy the exact committed source Git tree via the GitHub object API to a new isolated carrier branch; verify every blob and full root tree SHA; never alter source main or merge the carrier branch. Build against that identical tree using carrier-local GITHUB_TOKEN/Actions minutes. Record the carrier run, source tree identity and limits; delete temporary refs per approved retention policy. If the checks cannot be satisfied, report NOT_EXECUTED.

Refer to per-repository approved CI security tooling and [SECURITY.md](SECURITY.md). A mirror's Git SHA proves content integrity, **not** permission to disclose private source.
