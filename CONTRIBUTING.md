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
