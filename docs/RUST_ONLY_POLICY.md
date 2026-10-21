# Rust-only tooling; do not use Python

**Canonical Oreslang organization policy. Do not write, invoke, or add Python for first-party code, scripts, tests, build/CI orchestration, code generation, migration utilities, or repo maintenance. Write tooling in Rust.** Use shell and YAML only as thin wrappers where the platform requires them.

- No new `.py`, `.pyi`, `.ipynb`, Python dependency manifests, `python3`, `pip`, `pytest`, or Python setup steps in first-party code or workflows.
- Existing Python is migration debt, not an exception for writing more. Port functionality and tests to Rust before retiring the Python entry point; update every caller in workflows, hooks, docs, and build tooling.
- Rust replacements must retain CLI arguments, output contracts, security checks, failure behavior, and positive/negative regression tests. Run appropriate `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and end-to-end checks before merging.
- Enforce this rule in CI, preserve generated or third-party sources as appropriate, and document genuinely unavoidable external Python dependencies with owner and retirement plan. Do not silently weaken existing policy gates.
- Inventory and track work across repositories and organizations. **Count a migration only when code, callers, and evidence exist; an audit or policy-only PR does not count as a migrated script.**

This repository's original `tests/test_sync_gh_org.py` on the default branch is currently migration debt. PR #12 proposes its Rust replacement and a Rust no-Python gate; those changes are not in the default branch until merged and verified. Do not represent policy text as proof of complete migration.
