# Retire the old Oreslang GitHub organization safely

## Target end state

`atto-lang` is the canonical GitHub organization for the AttoLang language and runtime. Every repository currently owned by `ores-truffle-oreslang` must receive a **repo-by-repo outcome**: transferred to `atto-lang`, archived with retained history and an explicit exception, or retained temporarily only for an owned compatibility obligation.

The 33 userland/product-oriented repositories in the old org are also mapped **to `atto-lang`**, not `atto-co`, for this migration. A later AttoCo product carve-out must be a separate approved project with a new dependency/security review; no transfer from the old organization directly to AttoCo is authorized by this plan.

## Exit milestones

1. **Transfer readiness:** target organizations and responsible admins verified; per-repo security, billing, branch protection, CI and package identities fully audited.
2. **Public, low-dependency pilot:** `oreslang-demos` after hardening, verified receipt, exact source/destination Git identity, protections and non-zero executed CI steps.
3. **Developer tooling and specs:** move one repo at a time, with active PR owner coordination; preserve package/editor IDs and public executable aliases.
4. **Compilers and runtime:** treat six mandatory backend/compiler repos, shared source, target profiles and parity as a coordinated critical path with guest execution and exact-head receipts.
5. **Userland and infrastructure-oriented repositories:** transfer them to `atto-lang` in dependency order and retain public compatibility; a later optional AttoCo carve-out is outside this phase and needs separate reviewed authorization.
6. **Legacy exceptions:** every hold needs an accountable owner and disposition. Historical `ores-truffle-oreslang.github.io` and `.github` may remain temporarily to preserve public website and notices. Record date and retirement criteria; they are not active language ownership.
7. **Decommission review:** after no unresolved code, releases, package identities, live Pages, Actions, registry, secrets or inbound links remain, conduct an independent final inventory of the old organization and every new destination. Retire old organization **only by a separately authorized owner action**.

## Hold resolution matrix

| Current repository | Minimum outcome before organization retirement |
|---|---|
| `gh-linguist-fork` | Move to appropriate maintained owner or archive after upstream strategy approved |
| `ores-truffle-oreslang.github.io` | Maintain legacy site/redirect until tested custom domain and GitHub Pages migration |
| `oreslang-language-specs.md` | Diff against canonical AttoLang spec; migrate unique content, then archive or transfer |
| `html-js-css-syntax-highlighter` | Decide generic independent package owner vs AttoCo; preserve published package identity |
| `oreslana.ores` | Independent blockchain project owner decision; no implicit core-language ownership |
| `.github` | Migrate governance to a new `.github` per destination organization; keep historical notices/links until cutover |

## Measurable completion criteria

- 80/80 original stable GitHub repository IDs have a checked receipt or explicit archived exception, tracked in the manifest.
- No lost issues, PRs, tags, release assets, documented guest evidence, or deliberately retained histories.
- No unauthorized private-source exposure, surprise public fork, or privilege expansion.
- No old organization build/release/publish dependency remains unaccounted for, including GitHub Apps, Actions references, registry publisher identity and website URLs.
- New destinations have verified branch protections, teams, security reviews, Actions runner admission and backup/restore procedures.
- Old organization is not deleted merely because GitHub redirects appear to work.

## Live coordination

Use [issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26) to claim the exact repository ID and planned transfer window. Continue to coordinate other agents and ongoing PRs, and never overwrite an active migration branch without checking the latest SHA and owner claim.

**This is a policy/acceptance plan, not a completed transfer.**
