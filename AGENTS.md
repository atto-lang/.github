# Oreslang organization agent guidance

This repository publishes the **organization-wide** engineering policy. Follow
[docs/RUST_ONLY_POLICY.md](docs/RUST_ONLY_POLICY.md) and
[profile/README.md](profile/README.md); these rules apply across the
`ores-truffle-oreslang` organization, including all compiler backends.

## Rust-first testing and automation

- **Do not create, execute, expand or depend upon Python scripts or tests.**
  This includes Python in CI YAML, Makefiles, generated test fixtures, build
  helpers, verification tools, and temporary validation scripts.
- New first-party test harnesses, source/contract validators, fuzzing tools,
  CLI utilities, and workflow automation should be written in **Rust**.
  Prefer `cargo test` and `rustc --test`; require `cargo fmt --check`,
  `cargo clippy -- -D warnings`, and tests wherever the relevant toolchain
  and checks are available.
- Migrate legacy Python **behavior for behavior**, including argument
  handling, exit codes, security checks, invalid inputs, byte budgets, and
  positive/negative regression cases. Update callers only when the Rust
  replacement is functional; do not delete coverage to satisfy a no-Python
  gate.
- Existing backend implementation languages and executable guest languages
  remain valid for their actual runtime/compiler work. In particular, Wasm
  host/worker runtime tests may still need JavaScript to exercise real
  boundaries; **never replace those tests with Rust-only simulations and then
  claim guest/compiler parity**. The Python-to-Rust restriction applies to
  tooling and Python-based tests.
- Record exact source/test Git SHAs and the commands actually executed. A
  zero-step CI job is **NOT_EXECUTED**. Never present host-model success as
  compiled `.atto` guest, wasm32 binary, or native AOT evidence.
- Do not transfer private source to another GitHub organization for CI without
  source-owner approval, approved destination, and independent security review.

If a repository still contains legacy Python, open a scoped **draft migration
PR** with a Rust replacement and explicit validation evidence. The existence
of this policy note is not evidence that the migration is complete.
