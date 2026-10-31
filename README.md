# Oreslang GitHub organization policy

**Do not use Python for first-party scripts, automation, tests, or CI tooling. Use Rust.** Existing Python is migration debt and should be replaced with tested Rust implementations, not extended.

See [the canonical Rust-only tooling policy](docs/RUST_ONLY_POLICY.md) for scope, migration requirements, and CI enforcement.

For zero-step GitHub Actions runner failures, see the [read-only Rust admission diagnostic and independent-administrator workflow](docs/GITHUB_ACTIONS_RUNNER_ADMISSION_DIAGNOSTICS.md). This tool does not grant private-source export authorization or fix billing. Refer to [organization sync](docs/org-sync.md) for the existing safe repository synchronization process.

Migration rollout: [50-organization Python-to-Rust PR inventory and CI status](docs/RUST_MIGRATION_ROLLOUT_2026-10-21.md).
