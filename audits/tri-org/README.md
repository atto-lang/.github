# 2026-11-08 private AttoLang / Oreslang / AttoCo repository identity inventory

**Scope: READ-ONLY IDENTITY OBSERVATION. Not a migration, ownership assignment, security approval, or release authorization.** This snapshot lives in the legacy organization's **private** `.github` repository because it enumerates private repository names and numeric IDs. Do **not** publish or mirror this file into public `atto-lang/.github` or any unreviewed organization.

## Inputs and reconciliation
- `observed-repository-identity-20261108.json`: a live GitHub search snapshot of **165 distinct repository numeric IDs and names** across three organizations, with visibility, active/archive flag and default branch; 80 legacy Oreslang, 68 AttoLang, 17 AttoCo. Of these, 153 are private and 12 are public at observation.
- All rows carry `codeowner_verified:false`, `codeowner_identity:null`, `release_authority:null`, and `transfer_status:not_authorized`. **These are unknown/unverified authorities, not claims that the actual repository lacks an owner.** Current organization membership is not equivalent to independent CODEOWNERS or release authority.
- `migration-identity-reconciliation-20261108.json`: compares each of the **80** observed original-Oreslang names/IDs/visibilities/default branches to the existing, independently governed migration draft [PR #25](https://github.com/ores-truffle-oreslang/.github/pull/25), exact head `0d8856f6eee48d0f7c1a54cca950e788d6ab95f3`, CSV blob `f955ef0c14ceebd6821d5466e691d1b1c9a701b5`. **80/80 identities matched; 0 mismatches**. Current proposal remains **74 transfers to AttoLang, 6 legacy holds, 0 direct transfers to AttoCo**, every row `not_started`. These are not executed transfers.

## Follow-up review gates
1. Security/privacy review: do not attach raw private inventories to public issue comments or public workflows.
2. Independent owner/release authority: collect CODEOWNERS, effective rulesets, explicit release authority and denied/403 visibility cases under authorized repository administration. Do not silently change `codeowner_verified` when no admin readback exists.
3. Require reviewed source manifest and destination mapping, protected source and target, explicit numeric repository ID, exact SHA/tree and approval receipts before **each** transfer. Separate migration ownership from private CI carrier approval.
4. Reconcile repo identity against live GitHub immediately before any migration attempt. A dated inventory or green source-free workflow is not freshness proof.
5. After independently reviewed migration PR #25 is merged and its runner-backed tests execute, attach exact transfer receipts per repo. Do not overwrite historical status from this snapshot.

## Existing authoritative work
- [Oreslang migration issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26) and draft [PR #25](https://github.com/ores-truffle-oreslang/.github/pull/25) own transfer decisions.
- [Oreslang source export #962](https://github.com/ores-truffle-oreslang/oreslang-source.java/issues/962), [governance #37](https://github.com/ores-truffle-oreslang/.github/issues/37), and [private runner admission #315](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/315) remain independent blockers.
- The public additive synthetic-only governance tests are in [atto-lang/.github PR #3](https://github.com/atto-lang/.github/pull/3); they do not ingest this private roster or authorize live execution.
