# Atto organization migration: reviewable plan

**Inventory baseline:** 2026-11-06, connected GitHub repository search; **80** repositories in `ores-truffle-oreslang`, reconciled by exact repo name. This is a **proposal, not an executed transfer**. The destination organizations `atto-lang` and `atto-co` have not been verified as provisioned, and no private source is authorized to be copied or transferred by this file alone.

## Ownership rule
- `atto-lang` — canonical language semantics, core/std, compiler frontends and backends, target ABI, tooling, conformance, specifications.
- `atto-co` — standalone userland packages, web/frameworks, application services, stateful hosting, commercial infrastructure and products.
- `ores-truffle-oreslang` — historical namespace/compatibility and 6 explicit holds pending owner review.
- Product-branded **sibling organizations** remain independent by default: Orestra (11 repos), Volucro (14), Bretelle (10). Do not move them into `atto-co` merely to make the org chart tidy. Vertilt could not be inventoried with this GitHub installation.

## Machine-readable source of truth
[`repo-manifest.csv`](./repo-manifest.csv) has a proposed destination for **every one of the 80 source repositories**.
Counts: **41 to atto-lang**, **33 to atto-co**, **6 hold**.
`status=not_started` intentionally applies to all: no transfer has been executed.

## Transfer order and gates
1. **Provision:** destination organizations created by an authorized organization owner; 2+ admins where possible; SSO/team/security/ruleset design; paid-plan and private-repo features verified.
2. **Validate destination:** exact handle, owner permissions, collision-free names, intended visibility and transfer consent. Do not assume a 404 means the handle is claimable.
3. **A/B:** specification, docs, examples, and low-risk dev tooling after external URL/package/automation consumers audited.
4. **C/D/E:** compilers, targets, and conformance, one dependency-aware wave at a time. Freeze or coordinate conflicting PR/CI jobs. No claim of guest parity based on host-only or zero-step workflows.
5. **F:** application/infrastructure/userland packages after upstream API/URL consumers and credentials are verified.
6. **H:** adjudicate six holds independently, maintain GitHub Pages/history and package namespace compatibility.

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
