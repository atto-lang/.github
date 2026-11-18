# .github
Canonical atto-lang organization repository for the org profile, default community health files, and shared GitHub configuration.

## Canonical naming and documentation migration

[AttoLang](https://github.com/atto-lang) is the sole canonical project name; configure repository remotes as `https://github.com/atto-lang/.github.git` and refer to `.atto` in new examples. [atto-spec](https://github.com/atto-lang/atto-spec) owns normative grammar, ownership, lifetime, effects and actor contracts; [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance) tracks executed verification. Distinguish design proposals from tested compiler support and do not advertise zero-step CI as guest conformance.

## Repository ownership and contribution routing

**Belongs in this repository:** Organization profile, repository-routing policy, shared community health defaults, CODEOWNERS/review standards, and security/CI evidence guidance.

**Belongs elsewhere:** Compiler source, application features, normative syntax, release binaries or tenant secrets; use the responsible code repository instead.

**Safety and verification boundary:** This repository is public: do not publish private repository inventories, credentials, internal incident details or unreleased source here.



Changes asserting compiler/runtime support must identify the exact source head, target, toolchain, executed positive and negative guest tests, and failure/unsupported status; an empty CI step list is **not executed**. Documentation records responsibility, **not** a claim of feature completion.

## CI/testing language

**Rust-first policy:** new test harnesses and CI support scripts must use Rust, not Python. Legacy Python tests and pilot compilers must be migrated with verified fixture and negative-test parity before removal. See [CONTRIBUTING.md](CONTRIBUTING.md) for required exact-source Actions evidence and private alternate-runner mirroring.
