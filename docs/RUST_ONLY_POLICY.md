# Rust-only tooling; do not use Python

**Canonical Oreslang organization policy. Do not write, invoke, or add Python for first-party code, scripts, tests, build/CI orchestration, code generation, migration utilities, or repo maintenance. Write tooling in Rust.** Use shell and YAML only as thin wrappers for platform-specific entry points.

- No new `.py`, `.pyi`, `.ipynb`, Python package manifests, `python3`, `pip`, `pytest`, or Python setup steps in first-party code or workflows.
- Existing Python is migration debt, not an exception for writing more. Port functionality and tests to Rust before retiring the Python entry point; update every caller in workflows, hooks, docs, and build tooling.
- Rust replacements must retain CLI arguments, output contracts, security checks, failure behavior, and positive/negative regression tests. Run appropriate `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and end-to-end checks before merging.
- Enforce this rule in CI, preserve generated or third-party sources as appropriate, and document genuinely unavoidable external Python dependencies with owner and retirement plan. Do not silently weaken existing policy gates.
- Inventory and track work across repositories and organizations. **Count a migration only when code, callers, and evidence exist; an audit or policy-only PR does not count as a migrated script.**

PR #12 supplies the Rust org-sync regression and no-Python gate on the default branch. Policy text alone does not establish migration of any other repository or retained tooling.

## Required test-runner practice for new work

Write first-party testing infrastructure in Rust: use `rustc --test` for
dependency-free runner probes and `cargo test` (with `cargo fmt --check` and
`cargo clippy -- -D warnings`) for maintained test crates. A shell or YAML
wrapper may invoke the Rust binary, but may not implement a Python substitute
in another scripting language. Prefer reproducible, pinned toolchains.

When migrating old tests, retain positive, negative, property/stress and
security regression behavior (including exact exit and output contracts).
Do not simply remove the legacy test or mark an unexecuted GitHub Actions job
as a passing test. Runtime-specific host tests and compiled guest tests remain
necessary where Rust alone cannot exercise the actual implementation boundary.

Agents should also follow the [root AGENTS.md](../AGENTS.md) note. Do not
reinterpret this policy as authorization to mirror private repositories to
unapproved CI organizations.
