# Retire the old Oreslang GitHub organization safely

## Target end state

`atto-lang` is the canonical GitHub organization for the AttoLang language and runtime. Every repository currently owned by `ores-truffle-oreslang` must receive a **repo-by-repo outcome**: transferred to its reviewed final destination, archived with retained history and an explicit exception, or retained temporarily only for an owned compatibility obligation.

The proposed 33 direct `atto-co` transfers are product/library carve-outs, **not** an automatic license to move code to a new account. The original owner must approve every target. Avoid redundant two-hop transfers unless the user explicitly changes the destination policy, with a new reviewed migration manifest.

## Exit milestones

1. **Transfer readiness:** target organizations and responsible admins verified; per-repo security, billing, branch protection, CI and package identities fully audited.
2. **Public, low-dependency pilot:** `oreslang-demos` after hardening, verified receipt, exact source/destination Git identity, protections and non-zero executed CI steps.
3. **Developer tooling and specs:** move one repo at a time, with active PR owner coordination; preserve package/editor IDs and public executable aliases.
4. **Compilers and runtime:** treat six mandatory backend/compiler repos, shared source, target profiles and parity as a coordinated critical path with guest execution and exact-head receipts.
5. **AttoCo product carve-outs:** after destination owners confirm, transfer userland libraries/services directly to the product organization in dependency order. Version and pin cross-org API contracts.
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
