# Atto organization migration: reviewable plan

**Inventory baseline:** 2026-11-06, connected GitHub repository search; **80** repositories in `ores-truffle-oreslang`, reconciled by exact repo name and stable GitHub repository ID. **Goal:** retire this old GitHub organization as a source-code home, moving repositories one by one into the Atto successor ecosystem (direct to `atto-lang` for language-owned code; direct to `atto-co` for product-owned code if approved). The old namespace remains solely for explicit legacy compatibility while it is being emptied. This is a **proposal, not an executed transfer**. The destination organizations `atto-lang` and `atto-co` have not been verified as provisioned, and no private source is authorized to be copied or transferred by this file alone.

## Ownership rule
- `atto-lang` — canonical language semantics, core/std, compiler frontends and backends, target ABI, tooling, conformance, specifications.
- `atto-co` — standalone userland packages, web/frameworks, application services, stateful hosting, commercial infrastructure and products.
- `ores-truffle-oreslang` — historical namespace/compatibility and 6 explicit holds pending owner review.
- Product-branded **sibling organizations** remain independent by default: Orestra (11 repos), Volucro (14), Bretelle (10). Do not move them into `atto-co` merely to make the org chart tidy. Vertilt could not be inventoried with this GitHub installation.

## Machine-readable source of truth
[`repo-manifest.csv`](./repo-manifest.csv) has a proposed destination for **every one of the 80 source repositories**.
Counts: **41 proposed directly to atto-lang**, **33 proposed directly to atto-co**, **6 temporary holds**. `atto-lang` is the canonical successor to the former language organization; all 80 existing repositories require an explicit legacy-exit disposition. A product-owned repository should be transferred **once** to the approved final home, not redundantly moved through both organizations. Changes in the 33 provisional product destinations require a separate reviewed manifest update. The six holds are **not** permanent legacy homes by default.
`status=not_started` intentionally applies to all: no transfer has been executed.

The [point-in-time audit](AUDIT_FINDINGS_20261106.md) documents eight pilot branches reporting `protected=false`, the private ruleset/branch-permission 403s, and an Actions runner-admission failure with zero executed steps. The [transfer receipt template](TRANSFER_RECEIPT_TEMPLATE.md) must be filled for **each individual repository**. The [Rust validator](../../tools/validate_migration_manifest.rs) and [read-only CI workflow](../../.github/workflows/atto-org-migration-manifest.yml) enforce manifest invariants but **do not** authorize a transfer.

## Transfer order and gates
1. **Provision:** destination organizations created by an authorized organization owner; 2+ admins where possible; SSO/team/security/ruleset design; paid-plan and private-repo features verified.
2. **Validate destination:** exact handle, owner permissions, collision-free names, intended visibility and transfer consent. Do not assume a 404 means the handle is claimable.
3. **Pilot:** start with `oreslang-demos` (public, no open PRs in the 2026-11-06 snapshot) *after* destination branch protection, billable Actions, security and consumer audit pass. This supersedes the earlier spec-first pilot. Then transfer lower-risk docs/examples/tooling, one repository per receipt.
4. **C/D/E:** compilers, targets, and conformance, one dependency-aware wave at a time. Freeze or coordinate conflicting PR/CI jobs. No claim of guest parity based on host-only or zero-step workflows.
5. **F:** application/infrastructure/userland packages after upstream API/URL consumers and credentials are verified.
6. **H:** adjudicate six holds independently, maintain GitHub Pages/history and package namespace compatibility. For each hold, choose move, archive, or time-bounded compatibility exception with a named owner and review date. Do not delete the old GitHub organization while redirects, private code or live integrations depend on it.

Each transfer requires an owner-approved entry in the manifest; verified before/after repository ID, exact default-branch SHA/tree, protected branch/rulesets, secrets/environments, Actions permissions and runner billing, deploy keys, webhooks, package registries, release tags, pages, external submodules, GitHub App installation, automation, CODEOWNERS, PR/status retention and dependency URLs. See [runbook](./TRANSFER_RUNBOOK.md).

## Security
- **No mirror or transfer of private source into a different organization** without explicit source-owner security approval, permission-equivalence review, and destination authorization.
- Never publish this private manifest to a public repository without a redaction review.
- A workflow with zero executed job steps is **not** test evidence; verify exact-head executed build/test receipts before any high-risk compiler merge or cutover.
- Preserve the original repo name on first transfer; rename public branding / CLI / package paths separately in follow-up PRs.
- Do not create GitHub orgs, purchase plans, transfer repositories or rewrite protected branches as an incidental effect of accepting this proposal.

## Open decisions
1. Confirm administrators and legal/business ownership of `atto-lang` and `atto-co`, including whether each should be GitHub Enterprise governed.
2. Confirm whether a product-branded sibling org should ever be consolidated; current default is **no**.
3. Review Zorde/serialization and durable-object boundaries for userland vs compiler ownership.
4. Decide whether `oreslana.ores` becomes a separate product organization.
5. Determine how to preserve the historical GitHub Pages site; GitHub Pages URLs are not guaranteed to redirect with repository transfer.
6. Decide whether the 33 proposed `atto-co` destinations remain direct final homes; if the intended policy is that **every** original repository must physically enter `atto-lang` first, that is a distinct two-hop plan requiring revised approvals, URL/permission impact review, and an amended manifest. The current default is one direct transfer to each repo's approved final owner.
7. Resolve private branch-protection/ruleset entitlements and GitHub Actions runner admission before any pilot.
