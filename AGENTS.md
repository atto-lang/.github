# Agent instructions — Rust for tooling and tests

These instructions govern changes to this `.github` repository. For all Oreslang organization repositories, the canonical policy is [docs/RUST_ONLY_POLICY.md](docs/RUST_ONLY_POLICY.md); this file is **not automatically inherited by other repositories**. Reviewers and agents working across repositories must follow the canonical organization policy there too.

## No Python scripts or tests

- **Do not add or run Python** scripts, test frameworks, code generators, lint helpers, migration programs, CI steps, notebooks, or build orchestration. Do not introduce new Python dependencies, setup steps, or language-specific manifests.
- **Implement new first-party test utilities, fixtures, source-tree/CI verifiers, automation and helper tools in Rust**, using `rustc` or `cargo`. Prefer the Rust standard library for security-sensitive, source-free checks where practical; document any required crates and lockfiles.
- Existing Python tooling is migration debt, **not permission to extend it**. Port the feature and its positive, negative, adversarial, and failure-mode tests into Rust, update callers, then remove obsolete Python only after the replacement has real execution evidence. Never delete needed behavior simply to make a scan green.
- **Do not rewrite native compiler tests solely to switch languages.** Keep Java compiler/runtime tests in JUnit/Maven, LLVM/Forge tests in C++/CTest, .NET tests in `dotnet test`, and Wasm-specific runtime/bridge tests in their required framework. Add Rust for independent reference models, test orchestration, fuzzing, integration evidence and portable verification. New AttoLang guest features and fixtures remain in `.atto`; Rust tooling is not a substitute for guest compilation/execution.

## Verification required before merging

1. Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all-targets` on Rust crates (with `--manifest-path` if appropriate). Use `rustc --edition=2021 -D warnings --test` for dependency-free single-file verifiers. Capture the exact commands and real results.
2. Run the backend's native compiler/runtime test suite for any affected backend. Confirm positive and negative cases, not just that a manifest exists or a runner checked out code.
3. Pin the tested commit SHA and Git tree (and relevant source/fixture blob SHAs). An isolated reference test is **not** a complete compiler, runtime, AOT, sanitizer or cross-backend parity result.
4. A GitHub Actions job with no runner assignment or no executed steps is `NOT_EXECUTED` even if the workflow was triggered. Do not fabricate green checks, waive branch protection or merge unreviewed draft PRs.
5. Keep private source in its originating approved security boundary. **Do not export/mirror it across organizations** while the [private-source incident](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/22) and associated security gates remain unresolved. Fix source-local runner admission first.

## Related policies

- [Canonical Rust-only tooling policy](docs/RUST_ONLY_POLICY.md)
- [Contributing guide](CONTRIBUTING.md)
- [Python-to-Rust migration inventory](docs/PYTHON_TO_RUST_MIGRATION.md)
- [Private Actions runner recovery](docs/ACTIONS_RUNNER_ADMISSION_RECOVERY_20261026.md)

Policy text is not itself a migration or a passing test; require executed Rust and native backend checks before declaring either.
