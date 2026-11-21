# Atto source compiler PR governance rollout — 2026-11-21

## Scope and policy target

All 11 `atto-lang/atto-source.*` repositories have a staged PR to replace obsolete four-party *human* wording while preserving the user-required Ready-cycle windows:

- At 0–72h Ready: **one** eligible independent human approval from `the1mills` **OR** `ORESoftware`, plus current-head/current-cycle independently verified OpenAI and Anthropic attestations.
- At 72–240h Ready (3–10 days): the same once-per-PR human approval, no additional AI slot, and a mandatory competing/overlapping-PR audit.
- At ≥240h Ready: reuse the same eligible human approval and require fresh post-240h/current-head OpenAI and Anthropic attestations.
- Legitimate Draft→Ready restarts the Ready clock but not a still-valid human approval. Drafts ≥72h can promote only on genuine exact-head material CI, verified no-hold labels and no hard blockers.
- `keep-as-draft`, `keep-draft`, and `not-ready` block automatic promotion and automated merging. Existing GitHub rules, CI, source-export restrictions, CODEOWNERS and unresolved changes requests remain in force.

Canonical policy proposal: [`atto-lang/.github#56`](https://github.com/atto-lang/.github/pull/56). The other open [`atto-lang/.github#55`](https://github.com/atto-lang/.github/pull/55) was concurrently rewritten to remove Ready-age windows; reconcile before merging either. No policy prose alone installs branch protection.

## All source PRs (staged, **not merged**)

| Backend | PR |
| --- | --- |
| Cranelift | [atto-source.cranelift#81](https://github.com/atto-lang/atto-source.cranelift/pull/81) |
| LLVM | [atto-source.llvm#839](https://github.com/atto-lang/atto-source.llvm/pull/839) |
| JavaScript | [atto-source.js#114](https://github.com/atto-lang/atto-source.js/pull/114) |
| HDL | [atto-source.hdl#46](https://github.com/atto-lang/atto-source.hdl/pull/46) |
| Forge | [atto-source.forge#217](https://github.com/atto-lang/atto-source.forge/pull/217) |
| Realtime | [atto-source.realtime#10](https://github.com/atto-lang/atto-source.realtime/pull/10) |
| Rust | [atto-source.rs#500](https://github.com/atto-lang/atto-source.rs/pull/500) |
| Java | [atto-source.java#1727](https://github.com/atto-lang/atto-source.java/pull/1727) |
| WASM | [atto-source.wasm#264](https://github.com/atto-lang/atto-source.wasm/pull/264) |
| Atto-native | [atto-source.atto#16](https://github.com/atto-lang/atto-source.atto/pull/16) |
| .NET | [atto-source.net#199](https://github.com/atto-lang/atto-source.net/pull/199) |

The 11 staged PRs use one identical Git blob for `.github/tests/atto-pr-policy-contract.rs` (`7da8ed70537fd5513edb2f042b845e3cb3a45222`). The local governance policy blocks all contain the same 0–72h / 72–240h / ≥240h contract. Other compiler-specific `AGENTS.md` content, including WASM source-first guidance, is preserved. The standalone Rust source-contract check **does not** enforce live GitHub reviewer identity or merge authorization.

## Exact-tree private carrier test evidence

Atto source-repository Actions jobs failed with **zero executed steps**, including newly staged policy jobs. Do not interpret those as executed FAIL or PASS. Authorized private carrier: `ores-data-platform/atto-ci`, using only carrier-local `GITHUB_TOKEN` and a temporary isolated `ci/*` branch.

| Source PR | Verified full source Git tree SHA | Actual carrier Actions |
| --- | --- | --- |
| [Realtime #10](https://github.com/atto-lang/atto-source.realtime/pull/10) | `9e17c4ebbc3ab5dbcef0707d0364155d77b51ade` | [Run 44946284803](https://github.com/ores-data-platform/atto-ci/actions/runs/44946284803), 4/4 Rust tests PASS |
| [Atto-native #16](https://github.com/atto-lang/atto-source.atto/pull/16) | `8c43facf69c602580dae1184709ca9c942d1c390` | [Run 44946335529](https://github.com/ores-data-platform/atto-ci/actions/runs/44946335529), Rust policy check PASS |

Every source blob was copied through authenticated Git object APIs to the **private** carrier with SHA equality verified. The full source tree was reconstructed and the source subtree's Git tree SHA compared to the source repo's root Git tree. The carrier's **separate** test-control workflow verifies `git rev-parse HEAD:source` before running `rustc --test`. Control workflow is **outside** that exact source subtree. Restored source PR heads were read back and their full root tree SHAs matched the tested trees.

Those receipts establish **only the Rust policy-contract test**, not compiler execution or full 11-backend language/runtime parity. For the other nine backends, the policy text/blob is staged and verified, but required backend CI stays **NOT_EXECUTED** until genuine runner admission and source-specific tests pass.

## Remaining blockers (do not misreport as complete)

1. Merge canonical #56 only after the actual qualifying human review and applicable AI/time-window gates, and reconcile overlapping #55.
2. Re-read current PR heads and all required GitHub protections before merging any of the eleven staged PRs. Hold labels, unresolved changes requested, unexecuted jobs, and conflicts remain blockers.
3. Obtain admin-capable GitHub integration permissions to inspect/update effective branch rulesets. Current connected GitHub App gets `403 Resource not accessible by integration` on branch-protection reads. Target one required approval, `dismiss_stale_reviews=false`, `require_last_push_approval=false`, correct identity enforcement, and preservation of sensitive CODEOWNERS.
4. Most compiler source repositories currently have `@ORESoftware`-only CODEOWNERS entries on sensitive private-source export/workflow paths. These can block an otherwise valid `the1mills` approval if CODEOWNER review is enforced. Audit the exact effective rules against the exporter/security policy before changing any of them. Do not overwrite the sole-owner private export safety contract by adding a broad last-match rule.
5. Audit the remaining non-source Atto local copies and deploy a trusted GitHub identity/time-window/hold-label merge check; static policy-contract CI is not a substitute.
