# Oreslang GitHub organization policy

**Do not use Python for first-party scripts, automation, tests, or CI tooling. Use Rust.** Existing Python is migration debt and should be replaced with tested Rust implementations, not extended.

See [the canonical Rust-only tooling policy](docs/RUST_ONLY_POLICY.md) for scope, migration requirements, and CI enforcement. Refer to [organization sync](docs/org-sync.md) for the existing safe repository synchronization process.

Migration rollout: [50-organization Python-to-Rust PR inventory and CI status](docs/RUST_MIGRATION_ROLLOUT_2026-10-21.md).

## AOT and zero-GC language design

**New compiler/runtime feature reviews must follow [the controlled-dynamism policy](docs/AOT_ZERO_GC_DYNAMISM_POLICY.md)** and [agent directives](AGENTS.md). Full *guest* AOT is not the same as an AOT host runtime; guest zero tracing-GC is not the same as whole-process zero-GC. Reject uncontrolled dynamic code generation in sealed AOT while preserving statically generated proxies, virtual dispatch and ownership-safe actors.
