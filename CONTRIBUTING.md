# Contributing to Oreslang

## Rust-only tooling policy (no Python)

**Python is prohibited in this organization.** Do not add, maintain, invoke, generate, or require Python code or Python-based scripts, helpers, tests, code generators, build utilities, automation, CI steps, notebooks, or runtime dependencies. This includes `.py`, `.pyi`, `.ipynb`, `python` / `python3` commands, `pip`, `uv`, `poetry`, `pytest`, and Python GitHub Actions setup steps.

Implement tooling previously written in Python in **Rust** instead. Preserve behavior and test coverage while migrating; use `cargo fmt`, `cargo clippy`, and `cargo test` for Rust code. Existing Python must be inventoried and ported, not merely deleted if it implements required behavior. Changes introducing Python must be rejected in review and CI. References discussing the prohibition are allowed in documentation; they are not an authorization to execute Python.

This rule applies across all repositories, including compiler backends, examples, benchmarks, workflows, and documentation. Language implementations written in Java, LLVM/other native languages, .NET, etc. remain in their existing implementation languages; **the Python-to-Rust mandate applies to Python tooling and Python implementations**.

## Full-AOT and zero-GC review gate

For any compiler/language/runtime change, follow [controlled dynamism, static proxies, ownership and AOT/GC evidence requirements](docs/AOT_ZERO_GC_DYNAMISM_POLICY.md). State the execution/memory profile, build-time reachability plan, ownership and cleanup path, positive and negative `.atto` tests, and exact-head stepful execution evidence. Do not silently add runtime `eval`, reflection or JIT fallback to a sealed full-AOT path. This is additive to the Rust-only tooling policy above.
