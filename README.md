# Oreslang GitHub organization policy

**Do not use Python for first-party scripts, automation, tests, or CI tooling. Use Rust.** Existing Python is migration debt and should be replaced with tested Rust implementations, not extended.

See [the canonical Rust-only tooling policy](docs/RUST_ONLY_POLICY.md) for scope, migration requirements, and CI enforcement. Refer to [organization sync](docs/org-sync.md) for the existing safe repository synchronization process.

Migration rollout: [50-organization Python-to-Rust PR inventory and CI status](docs/RUST_MIGRATION_ROLLOUT_2026-10-21.md).

## Repository boundaries and AttoLang migration (2026-11-08)

This private `.github` repository governs the **legacy `ores-truffle-oreslang` organization**: Rust-only first-party tooling, review/security policy, CI evidence and repo-by-repo migration coordination. It is **not** an AttoLang compiler or an AttoCo hosting product. The public language is **AttoLang**; existing `oreslang-*` names, `.ores` compatibility and package/CLI identities must not be broken by documentation-only branding changes.

See [all 80 repository responsibilities and migration checklist](docs/repository-ownership-and-migration.md), [migration draft #25](https://github.com/ores-truffle-oreslang/.github/pull/25), and [coordination issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26). **Current plan: 74 first-stage transfers to `atto-lang`, six temporary holds, zero direct transfers to `atto-co`; no transfer is authorized by these docs.** `atto-lang` owns language/compiler/tooling/ecosystem source; `atto-co` owns separately governed hosted commercial infrastructure. Existing similarly named `atto-*` repos are not presumed to be transferred Git identities.

Do not transfer/mirror private sources or change branch policies without named owner, independent security review, target access/protection equivalence and per-repo exact-SHA/tree receipts. Actions jobs with zero executed steps are `NOT_EXECUTED`, not evidence. Preserve GitHub Pages, PRs, releases, secrets, apps, package references and remote redirects explicitly at cutover.
