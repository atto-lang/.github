# AttoLang contribution and testing policy

## Rust-first scripts and tests (effective 2026-11-15)

**Use Rust, not Python, for all new test harnesses, CI utilities, conformance tools, code-generation support utilities, source verification, and regression scripts in the `atto-lang` repositories.** Prefer `cargo test --locked` for a Rust crate and `rustc --edition=2021 --test path/to/test.rs -o "$RUNNER_TEMP/test-name"` for a small dependency-free test. Shell is allowed as a minimal CI launcher; substantive checks must be written in Rust.

Do **not** add `.py` test scripts, `python -m unittest`, or Python-based new test utilities. When modifying a legacy Python-based compiler pilot or test, prioritize a separate migration to Rust, retain parity fixtures, and remove Python only after the replacement is demonstrably equivalent. Existing legacy Python tests are transitional technical debt, **not evidence of Rust migration**. Do not silently delete tests or claim an unported Python compiler is Rust.

## Requirements for each compiler/runtime PR

1. Compile and execute the **actual** target (native Rust executable, WebAssembly guest, JVM/GraalVM Native Image, or other platform binary); use Rust test drivers where feasible.
2. Include positive, negative, concurrency, lifecycle, and malicious-input regressions. Maintain a versioned, shared cross-backend conformance contract and test Rust/JVM/Wasm against it, where supported.
3. Record exact source commit and Git-tree SHA, toolchain versions, executed test counts, failed/skipped steps, and artifact digests. A workflow queued, cancelled, skipped or failed before jobs start is **not** a pass.
4. If the source GitHub organization has no Actions budget, use an authorized, private alternative carrier: transfer exact Git blobs and tree through the GitHub Git-object API, independently verify every changed blob SHA **and the identical full root Git tree SHA**, and run Actions against the carrier's local tree using only its own `GITHUB_TOKEN`. Do not use cross-organization checkout credentials, publish private source or merge the temporary carrier. Respect API limits.
5. Keep work as a draft PR until successful immutable-head executable evidence and independent approval. Never weaken protection, required reviews or negative safety checks to obtain a green status.

## Naming and scope

Use `snake_case` for AttoLang callable names and Rust helpers. Document unsupported language features, single-worker limitations, and backend differences explicitly. A static IR JSON schema test is not a compiled guest execution, and guest error state is not proof of supervisor CONTROL delivery.

## Legacy Python retirement checklist

Identify every Python test/tool in a touched repository; create a Rust equivalent with the **same source fixtures, negative mutation coverage, exit codes and deterministic byte output**, exercise both on an immutable baseline until equivalent, then switch CI to the Rust version and remove the legacy Python dependency in a separately reviewed change. If a pilot compiler still uses Python, label it as such and do not represent its tests as Rust-native.


## Issue, branch, pull request and review workflow

- Create or link a scoped issue with the owning repo, normative contract, acceptance oracle, dependency/stacked-PR relationships and directly responsible engineer. Avoid duplicate cross-backend tickets.
- Work on a feature/fix branch; do not commit directly to protected main. Keep stacked branches explicit and review their base/head dependencies before retargeting or merging.
- Keep PRs small enough to validate. The description must link the issue, enumerate changed/unsupported behavior, show real positive and negative tests, source/head/tree, target/toolchain, artifacts, and exact CI receipts.
- Require an independent qualified review for compiler semantics, ownership, actor isolation, security-critical code, package provenance and CI trust boundaries. The author's self-approval is insufficient.
- Changes to observable language semantics require the accepted specification/RFC and a portable regression oracle; an implementation is not a language specification.
- Resolve conflicts semantically (never drop assertions or safety checks to make Git merge clean), rerun immutable-head CI after rebasing, and merge only after required checks and approval.
- A completed task requires merged code, documented supported profiles, actual test execution, regression and negative cases, and updated fixtures/docs when applicable. Pending/skipped tests remain visible as NOT_EXECUTED.

See [RFC_PROCESS.md](RFC_PROCESS.md), [TESTING.md](TESTING.md), [ROADMAP.md](ROADMAP.md), [SECURITY.md](SECURITY.md) and [CODEOWNERS](CODEOWNERS). These are organization defaults; the assigned repository's more specific controls take precedence where they are stricter. The CODEOWNERS file here is **not** automatically inherited by other repositories.

## Sealed AOT and controlled dynamism

Use the [AOT/zero-guest-GC review policy](docs/AOT_ZERO_GC_DYNAMISM_POLICY.md) when changing compiler profiles, dynamic code paths, actor heaps, instrumented AOT/JIT, runtime capabilities or examples. State exact target and distinguish tested host behavior from guest execution. Preserve the Rust-first, exact-head tests and review policies above.
