> **AI agent instructions (organization-wide):** Before working on code, issues, or pull requests, read and follow the [organization `AGENTS.md`](https://github.com/atto-lang/.github/blob/main/AGENTS.md), including its **merge-first stealth-development policy**. Honor any repository-local `AGENTS.md`, explicit holds, and GitHub-enforced rules.

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

## Engineering handbook and onboarding

For new compiler, runtime and tooling engineers, start with [ONBOARDING.md](ONBOARDING.md) and the [engineering roadmap](ROADMAP.md). These documents are guidance for the AttoLang organization and compatibility work, **not** a declaration that any backend has completed the whole language.

| Guide | Focus |
| --- | --- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Compiler phases, actors, IR and backend boundaries |
| [BORROW_CHECKER.md](BORROW_CHECKER.md) | Ownership, implicit borrowing, take/share/shallow/deep copy, pointer safety, and comparisons with Go, Erlang, Rust, Java and C++ |
| [BUILDING.md](BUILDING.md) | Verified repository toolchain references and build examples |
| [TESTING.md](TESTING.md) | Host vs guest proof, conformance, AOT, CI and sanitizers |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Rust-first tooling, PR process and independent reviews |
| [CODEOWNERS](CODEOWNERS) | Maintainers of **this** .github repository |
| [RFC_PROCESS.md](RFC_PROCESS.md) | Language-change decision and ratification process |
| [ROADMAP.md](ROADMAP.md) | P0–P3 priorities and milestone gates |
| [ONBOARDING.md](ONBOARDING.md) | First-month engineering checklist |
| [SECURITY.md](SECURITY.md) | Vulnerability reporting, trusted CI and source protection |

Per-backend CODEOWNERS, access control, issues, releases and branch protection must be installed and maintained **in each implementation repository**.
