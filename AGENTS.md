# Oreslang coding-agent instructions

These instructions govern work in this organization policy repository. They
also describe the organization-wide policy that contributor and automation
PRs should follow. The canonical policy is
[docs/RUST_ONLY_POLICY.md](docs/RUST_ONLY_POLICY.md); see
[CONTRIBUTING.md](CONTRIBUTING.md) for contribution rules.

## Rust, not Python, for first-party scripts and tests

- **Do not create, extend, invoke, or depend on Python scripts or tests.**
  This includes Python test runners, code generators, build scripts, CI
  helpers, notebooks, package managers, and Python setup Actions.
- **Implement new first-party scripts, test harnesses, security checks,
  migration tools, and CI test tooling in Rust.** Prefer small `rustc --test`
  suites for dependency-free guards; use Cargo when dependencies or a
  multi-crate project justify it.
- Existing Python code is migration debt. Port its behavior and regression
  coverage to Rust, update all callers, and preserve fail-closed checks
  before retiring old entry points. Do not add Python as a temporary
  workaround for missing CI minutes.
- Keep compiler/runtime logic and *native* tests in the implementation
  language required to exercise it (C++/LLVM, Java, JavaScript, C#, etc.).
  This is not permission to introduce Python tooling.
- Test Rust changes with `cargo fmt --check`, `cargo clippy -- -D warnings`,
  and `cargo test --all-targets`, or with `rustc -D warnings --test` where
  there is no Cargo project. Report any checks that could not execute.
- Use the existing Rust no-Python gate in `tools/check_no_python.rs` and
  `.github/workflows/no-python.yml`. Do not weaken the gate to accommodate
  new Python dependencies.

## CI and evidence

- Distinguish source inspection, host-model tests, real compiler builds,
  executable language guests, and AOT verification. A CI job with zero
  executed steps is **NOT_EXECUTED**, never a passing test.
- If GitHub Actions capacity is unavailable, only use a reviewed,
  authorized carrier for **private source**. Never copy private Git objects
  across organizations using connector/API writes as a shortcut around
  protected-source, approved-destination, or independent-review controls.
- Bind cross-backend evidence to exact source commits, Git root trees,
  fixture blobs and independently verified executable digests. Leave PRs
  in draft when required approvals or executed tests are missing.
