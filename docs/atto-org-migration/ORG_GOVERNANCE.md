# Organization governance baseline

## atto-lang (language foundation)
- **Scope:** canonical AttoLang specification, language-owned core/alloc/actor/std `.atto`, compilers, runtimes, target profiles, editor/tooling, reproducible conformance and formal specifications.
- **Owners:** organization admins and approved compiler maintainers; separate teams for spec, backends, platform, security/release, devex.
- **Protections:** default branch protected; no force pushes or branch deletion; reviewed PRs; independent approval on sensitive semantics/ownership/sandbox changes; required successful *executed* CI (not empty steps).
- **Evidence:** exact head SHA and tree, command/case inventory, native guest execution for parity claims, explicit negative tests, repeatable artifacts.
- **Security:** compiler supply-chain access, GitHub Apps, runners and release publishing credentials are least privilege and scoped. Private mirrors require separate owner/security approval.
- **Releases:** immutable tags, signed/pinned artifacts where supported; backward-compatible command `oreslang` may coexist with `atto`. Renaming organization must not silently change package/import semantics.

## atto-co (infrastructure/products)
- **Scope:** hosted products, deploy/orchestration tooling, networking/observability services, separately owned userland libraries, paid/cloud infrastructure. **The current `ores-truffle-oreslang` migration sends its repositories to `atto-lang` only**; any later AttoCo product carve-out requires a separate approved transfer.
- **Owners:** infrastructure/platform security, product maintainers, release engineers; per-product repositories or sibling GitHub organizations with explicit integration contracts.
- **Protections:** independently reviewed application/IaC PRs, environment approvals for production, secret/credential separation, deploy rollback, tenant-isolation/fail-closed tests.
- **Cross-org dependencies:** consume versioned AttoLang interfaces rather than importing unstable private compiler branches directly. Pin compiler revision and guest test receipts for generated/runtime code.
- **Product orgs:** preserve independent Orestra/Volucro/Bretelle repositories, CI permissions, credentials and release histories. Vertilt needs separate inventory/owner confirmation.

## Migration coordination
- Open a transfer-specific issue per batch, assign a responsible human owner and independent security reviewer.
- Record exact source repo ID, SHA/tree, target account/visibility, transfer authority, GitHub billing/runner readiness, and dependency review **before** action.
- Declare maintenance window or coordinated freeze for active PRs only when necessary.
- After transfer, verify same repository ID and exact SHA/tree; PR/issues and URL behavior; then update consumers atomically.
- Keep historical organization as a redirect/compatibility layer; do not delete it during migration.
