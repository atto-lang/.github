# Oreslang organization coding guidance

This repository maintains the Oreslang organization policy. Consult the
[canonical Rust-only policy](../docs/RUST_ONLY_POLICY.md),
[contribution rules](../CONTRIBUTING.md), and
[root agent instructions](../AGENTS.md). These rules must be copied or
referred to explicitly in individual repositories; a policy file here is
not automatically enforced in every project.

## Rust scripts and tests; no Python

- **Do not add, modify, run, or depend on Python scripts, Python-based tests,
  Python code generators, migration helpers, CI runners, notebooks or package
  managers.** Do not hide Python behind shell commands or workflow wrappers.
- **Write first-party verification, fuzzing, fixtures, test runners, security
  guards, migration tools, and helper automation in Rust**. Use `rustc --test`
  for standalone standard-library checks, or `cargo test --all-targets` for
  crates. Run `cargo fmt --all -- --check` and
  `cargo clippy --all-targets -- -D warnings` where applicable.
- Existing Python tooling is migration debt: port **all** CLI behavior, output
  contracts, error paths, positive cases, negative cases, and adversarial
  regressions to Rust. Update every caller and workflow; only then retire
  the old entry point. Never drop working test coverage to make a scanner pass.
- Leave compiler/runtime implementation code and authentic backend tests in
  their appropriate languages (LLVM/C++, Java/JUnit, .NET, Wasm/JavaScript,
  etc.). Rust orchestration does not replace real compiled guest/native tests.

## Verification and private-source safety

- Run the Rust no-Python scanner in
  `tools/check_no_python.rs` and the `Rust-only / no Python` workflow.
  A check that was not actually executed is not a pass.
- Pin the tested source commit, Git root tree, and relevant fixture artifacts.
  Record actual Rust, native AOT, and sanitizer commands and receipts separately.
- If Actions jobs have zero executed steps, report **NOT_EXECUTED** and keep
  affected PRs in draft. Do not substitute synthetic host tests for production
  compiler or guest evidence.
- Do not copy private source trees or Git blobs into another organization for
  CI until the originating repository's independent security approval,
  protected provenance, destination approval, retention and cleanup policy
  explicitly permit that transfer. Available runner minutes are not approval.

For project-specific requirements, the local repository's tests and stricter
security gates also apply.
