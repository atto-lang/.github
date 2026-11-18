# Oreslang GitHub organization policy

## Oreslang → AttoLang project migration (2026-11-18)

The public language name is **AttoLang** (formerly **Oreslang**). This is the legacy `ores-truffle-oreslang/.github` repository. New corresponding AttoLang work belongs in **[atto-lang/.github](https://github.com/atto-lang/.github)**. Do not infer identical Git histories, issues, PRs, execution evidence or completed ownership transfer from matching project names. This is a legacy organization profile and routing notice; the new public language organization is atto-lang.

The **canonical language specification** is [atto-lang/atto-spec](https://github.com/atto-lang/atto-spec), not an older Oreslang README or historical specification copy. [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance) owns portable executed guest tests, [atto-compilers-parity-testing](https://github.com/atto-lang/atto-compilers-parity-testing) owns differential checks, and [atto-common-source](https://github.com/atto-lang/atto-common-source) owns portable `.atto` libraries. Current design targets include `snake_case` names, `borrow!`/`take!`/`copy!`/`copy!!`/`share!`, [implicit borrowing](https://github.com/atto-lang/atto-spec/blob/main/language/implicit-argument-borrow-v1.md), [file/module constants including one-time non-throwing `os.read_env`](https://github.com/atto-lang/atto-spec/blob/main/language/file-module-const-v1.md), and [`<<<`/`>>>`/`ashr` shifts distinct from stream `<<`/`>>`](https://github.com/atto-lang/atto-spec/blob/main/language/bitwise-and-stream-operators-v1.md). These are normative/proposed targets; unsupported syntax must fail closed.

Preserve legacy `.ores` fixtures, historical CLI/package names and ABI/wire contracts until replacements are verified. Do not use Python for new AttoLang scripts/tests/CI; prefer Rust for support tooling. Report exact SHA/tree, toolchain, positive/negative compiled guest runs and explicit failed/unsupported/not-executed states; host-only tests and zero executed Actions steps do not prove guest parity.

**Do not use Python for first-party scripts, automation, tests, or CI tooling. Use Rust.** Existing Python is migration debt and should be replaced with tested Rust implementations, not extended.

See [the canonical Rust-only tooling policy](docs/RUST_ONLY_POLICY.md) for scope, migration requirements, and CI enforcement. Refer to [organization sync](docs/org-sync.md) for the existing safe repository synchronization process.

Migration rollout: [50-organization Python-to-Rust PR inventory and CI status](docs/RUST_MIGRATION_ROLLOUT_2026-10-21.md).
