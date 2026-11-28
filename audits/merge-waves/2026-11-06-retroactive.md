# Release hold — Oreslang / AttoLang 2026-11-06 merge waves

**Status: BLOCKED.** This is a historical audit of ten already-landed PRs across nine repositories; it is **not** a live release authorization, independent security review, or successful CI certification. Evidence snapshot: [2026-11-06-retroactive.json](2026-11-06-retroactive.json). No source files, policy allowlists, or CI carrier settings were modified.

## Exact landed commits and sampled post-merge CI

| Repo | PR | Landed SHA | Sampled post-merge Actions run | Observed execution |
| --- | ---: | --- | ---: | --- |
| [oreslang-source.forge](https://github.com/ores-truffle-oreslang/oreslang-source.forge/pull/198) | 198 | `7902eb2e3610babe1004a3e752b12079737992e6` | [42294518377](https://github.com/ores-truffle-oreslang/oreslang-source.forge/actions/runs/42294518377) | Failed sampled job, 0 steps — NOT_EXECUTED |
| [oreslang-source.js](https://github.com/ores-truffle-oreslang/oreslang-source.js/pull/215) | 215 | `b437edff0ab136d83f9aedae652a5f368b4cc5ee` | [42294569932](https://github.com/ores-truffle-oreslang/oreslang-source.js/actions/runs/42294569932) | Failed sampled job, 0 steps — NOT_EXECUTED |
| [oreslang-source.hdl](https://github.com/ores-truffle-oreslang/oreslang-source.hdl/pull/27) | 27 | `05b2a5899ca2b4f38ee27827a509dbfd1d19a3ef` | [42294592084](https://github.com/ores-truffle-oreslang/oreslang-source.hdl/actions/runs/42294592084) | Failed sampled job, 0 steps — NOT_EXECUTED |
| [oreslang-source.rs](https://github.com/ores-truffle-oreslang/oreslang-source.rs/pull/26) | 26 | `e516b09622f6d1b9d61330aff88e6c86bb6d97f1` | [42294604675](https://github.com/ores-truffle-oreslang/oreslang-source.rs/actions/runs/42294604675) | Failed sampled job, 0 steps — NOT_EXECUTED |
| [oreslang-source.net](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/374) | 374 | `5d8246bca843309ee3f66538f1385fefb5690858` | [42294558773](https://github.com/ores-truffle-oreslang/oreslang-source.net/actions/runs/42294558773) | Failed sampled job, 0 steps — NOT_EXECUTED |
| [sublime-text-plugins](https://github.com/ores-truffle-oreslang/sublime-text-plugins/pull/130) | 130 | `6c54e1042596b3274c0d921fcc41636da5f82f76` | [42297029167](https://github.com/ores-truffle-oreslang/sublime-text-plugins/actions/runs/42297029167) | Successful stepful job (15 steps); all three jobs succeeded |
| [sublime-text-plugins](https://github.com/ores-truffle-oreslang/sublime-text-plugins/pull/131) | 131 | `8320a29a2fbaccc5d5f296a1a1712000c0a6d229` | [42297059629](https://github.com/ores-truffle-oreslang/sublime-text-plugins/actions/runs/42297059629) | Successful stepful job (15 steps); all three jobs succeeded |
| [oreslang-common-source](https://github.com/ores-truffle-oreslang/oreslang-common-source/pull/243) | 243 | `3c39e798f2b6d4f8d4f0760821329321c4433355` | [42297083612](https://github.com/ores-truffle-oreslang/oreslang-common-source/actions/runs/42297083612) | Failed sampled job, 0 steps — NOT_EXECUTED |
| [oreslang-ssr](https://github.com/ores-truffle-oreslang/oreslang-ssr/pull/36) | 36 | `eeb0535f34af15b0773585d42259595b67eec996` | [42300219068](https://github.com/ores-truffle-oreslang/oreslang-ssr/actions/runs/42300219068) | Failed sampled job, 0 steps — NOT_EXECUTED |
| [oreslang-linked-queue](https://github.com/ores-truffle-oreslang/oreslang-linked-queue/pull/23) | 23 | `09ea6364794cc050639b669a783a169875c5f50a` | [42297102590](https://github.com/ores-truffle-oreslang/oreslang-linked-queue/actions/runs/42297102590) | Failed sampled job, 0 steps — NOT_EXECUTED |

All ten PRs return `merged=true` and their squash commits are ancestors of current `main` (`compare(landed_sha...main).behind_by=0`). Every one of their PRs has **zero submitted reviews** in the GitHub review API. Each of the nine repositories' `main` branches reports `protected=false`, with required status-check enforcement off. GitHub branch protection details are inaccessible to this installation (`403 Resource not accessible by integration`), so an authorized administrator must independently inspect the effective policy.

The successful Sublime runs are **landed-commit** push events: [#130 run 42297029167](https://github.com/ores-truffle-oreslang/sublime-text-plugins/actions/runs/42297029167) and [#131 run 42297059629](https://github.com/ores-truffle-oreslang/sublime-text-plugins/actions/runs/42297059629), each with three completed successful jobs and nonempty steps. For the remaining eight commits, at least one completed failed post-merge job was independently inspected and had `steps=[]`. Those are runner-admission/unexecuted observations, **not failed compiler tests**; no passing source-owned execution has been established for those eight landed commits.

## Fail-closed historical audit integrity (not a release gate)

The historical snapshot remains immutable: JSON Git blob `8ba3658779c6a836d24f2321451be5474ecbbc5e`. The Node validator independently verifies the file's Git blob identity (Git object header and file bytes) **and** checks the exact ten PR numbers, head/squash SHA pairs and sampled Actions run/job identities against separately pinned code constants. Replacing it with a plausible 10-PR/9-repository synthetic snapshot is rejected. This is a static archive, not a live API authorization or new test evidence.

From the **exact audited PR HEAD**, execute:

```sh
node --test tests/check-merge-wave-hold.test.mjs
node tools/check-merge-wave-hold.mjs
node tools/check-merge-wave-hold.mjs --release
```

The first command runs adversarial source-free host tests. The second may exit **0** *only* to confirm the historical snapshot is byte-for-byte intact and remains `BLOCKED`. The third must exit **1**, even when the intact archive passes every integrity check. Invalid input or corrupted evidence must exit **2**. No automation may interpret the ordinary integrity exit as permission to merge or release.

The CI workflow separately attempts a checkout-free, no-secrets **two-step** private runner admission canary and the Node audit tests. GitHub [PR run 42313133028](https://github.com/ores-truffle-oreslang/.github/actions/runs/42313133028) and [push run 42313127141](https://github.com/ores-truffle-oreslang/.github/actions/runs/42313127141) both failed before executing **any** steps. An independent in-process V8 audit exercised 43 structural/JSON cases against the exact committed code and snapshot, but **no exact-head Node filesystem/cryptography or hosted CI test execution is established**. Treat Node and GitHub Actions as `NOT_EXECUTED` until runner admission and substantive stepful receipts are independently observed.

## Required remediation, in order

1. **Administrative policy:** Enable and read back protected `main` or equivalent enforceable rulesets for all nine repos and priority compiler/platform repos. Require independently approved PRs, CODEOWNERS approval from a genuinely distinct reviewer, stale-approval dismissal, no unintended bypass, force-push/deletion denial, and trusted app-ID-bound required checks. Do not interpret `mergeable=true` or a checked-in CODEOWNERS file as enforceable protection. Org admin tracker: [org .github #2](https://github.com/ores-truffle-oreslang/.github/issues/2) and [conformance #69](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/69).
2. **Runner admission:** Investigate account/private repo Actions billing, entitlement and permitted runners from check-run failure annotations. Restore an *in-source, source-free* canary with actual executed steps, then exact-landed source-owned test runs. Do not transfer private compiler blobs or turn a foreign capacity canary into source proof. Tracker: [conformance #315](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/315).
3. **Post-merge security review:** Obtain an independent, explicitly **post-merge** code/security review of the ten landed diffs and current main integration. This can qualify future release decisions, but cannot truthfully recreate approvals required before the historical merges.
4. **Exact-landed CI:** For the eight unexecuted sampled lanes, run actual relevant unit/native/guest conformance at the landed SHA or a reviewed and immutable current-main descendant, record repo, head/tree, run ID/attempt, issuer-bound check context, job, step names/conclusions and artifacts. Never promote model/source-only to compiled guest/AOT parity.
5. **Blockers still open:** [Java #2575](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2575) requires bounded input **before** materialization, and [LLVM #1447](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1447) requires race-free publication of `pthread_t` and stress/TSan before merge. Those are separate open PRs, not part of the ten already-landed commits.
6. **Qualification mechanism:** Wire this snapshot only as **input** to a trusted live verifier after independent review, such as [conformance #476](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/pull/476) / [#487](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/pull/487). Do not let snapshot hashes or an offline JSON file authorize a release.

**Integrity boundary:** This report records authenticated GitHub observations but is not signed by an external trust authority. No private-source export, protected-branch settings mutation, or test rerun is implied by this document.
