# AttoLang PR governance drift audit — 2026-11-21

## Scope and finding

Read the default-branch `AGENTS.md` of all **88** GitHub repositories returned for `atto-lang` (including the central `.github` repository). This audit inspects repository-local instructions, **not** effective GitHub rulesets or branch protection.

| Repository-local policy class | Count | Evidence pattern |
| --- | ---: | --- |
| Atto boilerplate still says both named humans and fresh *four-party* reviews | **72** | `Ready age <72h: ... ORESoftware + the1mills`, `fresh post-boundary reviews from all four` |
| Legacy Oreslang boilerplate imported into Atto repos | **15** | Link to `ores-truffle-oreslang/.github`, `Ready <72h: ... ORESoftware + the1mills`, `four-party review` |
| Central `atto-lang/.github` | **1** | Canonical instructions; PR #55 restores the Ready windows following #41's human-only regression |

The above default-branch instruction files are readable, but **87 local copies need an approved, independently tested reconciliation**. These findings do **not** prove that any given branch protection is misconfigured. Current integration access returns GitHub `403 Resource not accessible by integration` on branch-protection reads.

## Canonical target

- Human slot: exactly one independently authenticated GitHub `APPROVED` review from **either** `the1mills` (ID `2148448`) **OR** `ORESoftware` (ID `11139560`), never the author. Persist it across ordinary commits, merges/rebases from `main`, and legitimate Ready-cycle restarts unless it is dismissed/revoked or a blocking review supersedes it.
- Ready age **0–72h**: one eligible human approval **plus independently verified current-cycle, current-head OpenAI and Anthropic attestations**.
- Ready age **72–240h**: one eligible human approval **plus mandatory semantic competing/overlapping-PR audit**; no additional AI slot.
- Ready age **240h or more**: preserve the original valid human approval and require fresh post-boundary, current-head OpenAI **and** Anthropic attestations.
- Draft promotion: at 72 consecutive hours in the current Draft cycle, automatically mark Ready only after passing material exact-head CI, verified labels, and absence of `keep-as-draft`, `keep-draft`, and `not-ready`. Those labels also prohibit automated merging.
- All ages: require GitHub-enforced protections, no conflicts or blocking change requests, exact-head security/material CI, and independently authenticated review evidence. An advisory JavaScript assessment is **not** authorization to merge.

The authoritative source is [`atto-lang/.github/AGENTS.md`](https://github.com/atto-lang/.github/blob/main/AGENTS.md), as restored and tested by [PR #55](https://github.com/atto-lang/.github/pull/55).

## Safe rollout for 87 local copies

1. After the central correction is merged, identify the existing policy block between `<!-- org-merge-review-policy:begin -->` and `<!-- org-merge-review-policy:end -->` in each repository. Preserve all project-specific instructions **outside** that block, including legacy migration notes.
2. Replace only the scoped block with a short pointer to Atto's canonical instructions and the one-human / 0–72h / 72–240h / 240h+ / Draft-hold summary. Do **not** redirect an Atto repository to an Oreslang governance source.
3. Review/test each proposed local file change on an exact head SHA, keep all material safety checks, and merge only under the effective required GitHub protections. Do not merge 87 bulk changes based on this report alone.
4. When authorized to administer branch settings, read and record each repo's *effective* ruleset and branch protection. Target one review, `dismiss_stale_reviews=false`, `require_last_push_approval=false`, and trusted-owner identity enforcement via compatible CODEOWNERS or a trusted required check. Keep all other necessary protections.
5. Where Actions minutes are unavailable, use only an approved **private** Git-tree carrier and verify every blob and the complete root tree SHA before running carrier-local Actions. No cross-org checkout credentials or source disclosure.

## Immediate test evidence

[PR #55](https://github.com/atto-lang/.github/pull/55) has a GitHub Actions regression workflow for the recovered windows, reviewer identity, persistent human approval, hold labels and AI provenance. Its exact-head checks must be re-read after any change to that PR. [PR #41](https://github.com/atto-lang/.github/pull/41) is the overlapping scanner fix that makes [PR #54](https://github.com/atto-lang/.github/pull/54) obsolete.
