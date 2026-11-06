# Security and dependency audit — 2026-11-06

This is a **point-in-time inspection**, not proof that a destination organization exists or a repository is safe to transfer. The canonical transfer plan is [README](README.md), with [manifest](repo-manifest.csv) and [runbook](TRANSFER_RUNBOOK.md). Revalidate every observation at transfer time.

## Findings

| Priority | Finding | Evidence | Required remediation |
|---|---|---|---|
| BLOCKER | Destination organizations `atto-lang` and `atto-co` cannot be confirmed provisioned or writable through the connected GitHub installation. | Org-scoped search gave no accessible repositories, and the connector has **no** create-org or transfer action. This does **not** prove name availability. | Source and target owners provision/verify orgs and transfer authority before pilot. |
| BLOCKER | Branch protection on sampled existing repositories is absent according to the GitHub branch API. | `branches/main.protected=false` on eight sampled candidates. | Define destination rulesets/protection before transfer, test enforcement with an authorized negative push, capture evidence and independently review. |
| BLOCKER | Fine-grained protection visibility and plan entitlement are not verified. | `.github/branches/main/protection` returned 403 `Resource not accessible by integration`; `.github/rulesets` returned 403 `Upgrade to GitHub Pro or make this repository public`. | Resolve GitHub plan/permissions; do not claim protection from a 403. |
| BLOCKER | GitHub Actions failed before admitting any steps. | PR #25 head `7ca93c0...`: manifest workflow run `42286113924`, job `128127244563`, `steps=null`; Rust-only and ownership workflows likewise. | Obtain runner budget/admission, rerun exact head; capture non-zero executed steps. A job failure here does not demonstrate code/test failure. |
| HIGH | Transfer manifest is a planned mapping, not an approved transfer roster. | All 80 rows marked `not_started`, head `capture_at_transfer_preflight`, receipt `pending`. | Keep these pending until exact source/target proof and three-party approval exist. |
| HIGH | Private-source exposure risk. | 72 private repositories in baseline; source and target owner/team visibility may differ. | Require written approval of private-source transfer and target access equivalence. No improvised cross-org mirror. |
| HIGH | Compilers and standard library carry large open PR and dependency surfaces. | E.g. `oreslang-common-source` had 13 open PRs; `oreslang-format` had 7; language spec had 2 when sampled. | Coordinate per repo with maintainer and other agents. Preserve active PRs and release gates. |
| HIGH | Dependency mapping remains partial. | Search index returned limited hits; manifest records unknown hints for many entries. | Build a full dependency/consumer inventory from workflow files, manifests, submodules, release automation, registry scopes, all branches, and package metadata, not only indexed code search. |
| MEDIUM | GitHub Pages and packages may not follow normal repository URL redirects. | Historical org owns `ores-truffle-oreslang.github.io`; editor marketplace and package publishing may have owner-bound identity. | Keep existing Pages live until a separately tested redirect/domain transition. Inventory release and marketplace credentials, package scopes, badges. |
| MEDIUM | Initial hold list could accidentally become permanent. | Six rows target the old org and have disposition `hold`. | Assign resolution owner for each; exit choices are relocate, retain with an explicit compatibility exception, or archive/retire with documented reason. |
| MEDIUM | Manifest corruption or silent gate weakening. | Static CSV is editable; initial PR lacked automated schema and invariant checks. | Rust-only validator and negative mutation tests added in this PR; hosted test still awaits executed-step receipt. |

## Pilot snapshots (read-only, NOT preflight approvals)

| Repository | Visibility | Main commit (SHA-1) | Git tree SHA-1 | Open PRs | Branch API `protected` |
|---|---|---|---|---:|---|
| `oreslang-demos` | public | `4ee37219e94a460ae862361720a546f0201b991f` | `3f141eb7f964cf5a46ade9be47dae5ad19c8335f` | 0 | false |
| `vscode-plugins` | public | `ceacdd490f01cac5f09b75ca74a2b9714deb53b7` | `4b668b8a037978ce8e1269ae68c1eb07c169db90` | 0 | false |
| `oreslang-cli` | public | `afae1b14b85c185948c6cbf33f5efac3a459f468` | `57ff96f34efb5d55e6d00622831dc14d1e791d75` | 0 | false |
| `attolang-language-spec.md` | private | `f8206a3f5b94e59a1bc75c6f8f5a69a9013e2f52` | `46c0c949af114575998f28586c2969cb18ce4c59` | 2 | false |
| `oreslang-format` | private | `fb06ab8f73412b0693321058f7429ec3615d079f` | `761b4706dfad0ae7862f27b2a05e70be3377af51` | 7 | false |
| `oreslang-common-source` | private | `4849b6176591fa0d817bc958da977cce25f875ed` | `8a0090826d70ab98056ba91c71ad3f95754c7e8a` | 13 | false |
| `oreslang-demo-restful-server` | public | `7c4b6c9c4ccd1d1f664bb1d698ab76b075110a42` | `8155459f8c7f6fd45cd870b4d7d5a662b975154d` | 9 | false |
| `sublime-text-plugins` | public | `84c77c18fe3252000f474cd0fd784f25f0aa794b` | `3001a0c8fd94e5197949ad2f9a14c07ef1b89e03` | 2 | false |

The **recommended first transfer pilot is `oreslang-demos`** because it is public with no open PRs at audit time; this is not a claim that it is safe from all package, redirect, permissions, or CI risks. Recheck the commit, tree, branch protection, consumers, owner approval, and Actions execution immediately before transfer. The earlier language-spec pilot suggestion is superseded by this audit.

## Scope / successor interpretation

The user clarified that the migration is **repository-by-repository from `ores-truffle-oreslang` to `atto-lang`**. The corrected mapping now has **74 proposed direct transfers to `atto-lang` (41 language/core and 33 userland/product-oriented)**, **zero direct old-org transfers to `atto-co`**, and six temporary legacy holds. The 33 product-oriented repositories are **flagged for a possible future AttoCo separation**, not authorized to move during this migration. Every original repository must have a retirement/exit disposition before the old namespace is retired.

## Open audit limitations

- No complete org-wide dependency crawl or exact-head CI execution was possible.
- Stable GitHub repository IDs were confirmed for all 80 at baseline, but moving heads and branch rules must be reread for each transfer.
- No destination organization permissions, package registry, webhooks, secrets, team memberships, or paid-plan entitlements were verified.
- No migration or private-source copy has been performed.
