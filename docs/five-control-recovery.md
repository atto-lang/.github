# Five-control cross-organization recovery (READ ONLY)

This operational diagnostic covers tasks 1–5 for Oreslang, AttoLang, and AttoCo. It **does not** alter GitHub settings, approve private-source transfers, enable publishing, certify a release, or replace an administrator's independent readback.

## Five administrative and release holds

1. **Original-source private Actions admission:** [Oreslang conformance #315](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/315) identifies a billing/payment-or-budget admission rejection class. An authorized billing owner must inspect the exact check-run annotations, payment state, product budget, and any inherited enterprise budget. Do not silently increase spending. Only an original-source-org **private** canary with nonzero runner ID, executed steps and retrieved log markers can prove runner admission recovered.
2. **Secure alternate runner pool:** No private Git tree, source blob, artifact, cache or checkout may cross organizations merely because a **source-free** canary succeeds. Require independent disposition of outstanding transfer incidents, an allowlisted **numeric private destination repository ID**, exact source commit and tree SHA, approved workflow/test plan, least-privilege credentials, protected source branch and source/destination receipt verification. Existing holds remain in effect.
3. **Oreslang five-source protection:** Protect `main` in `oreslang-source.java`, `.llvm`, `.net`, `.wasm` and `.forge`. Require an independent CODEOWNERS approval, dismiss stale reviews, last-pusher approval, an executed GitHub Actions check with trusted app identity, no force push, no deletion and no unreviewed bypass actors. Private repository rulesets and branch protection may be unavailable with the current plan/connector; the administrator must verify the effective configuration, not infer success from a proposed workflow.
4. **Canonical AttoLang compiler review:** [AttoLang Rust #30](https://github.com/atto-lang/atto-source.rs/issues/30) needs effective protected `main`, independently enforced CODEOWNERS and exact-head/merge-tree CI. Integrated [compiler PR #28](https://github.com/atto-lang/atto-source.rs/pull/28) already requests `@ORESoftware`; `@the1mills` is its author and cannot self-approve. Review only one consolidated-vs-stacked merge path; require a fresh approval on the current head.
5. **Public AttoLang Pages release:** [Pages #5](https://github.com/atto-lang/atto-lang.github.io/issues/5) requires an authorized administrator to select **GitHub Actions** (not branch-source Pages), protect `main`, and configure the `github-pages` environment for protected branches, independently approved reviewers, prevent self-review and disable admin bypass. A reviewed manual deployment must bind exact compiler bytes/digest. An external live HTTPS audit must verify manifest, Wasm MIME and digest, CSP, and 404/410 on forbidden source/test paths.

## Running the diagnostic

```sh
node --test tests/recovery-gates.test.mjs
node security/recovery-gates.mjs admin-snapshot.json
```

The source-free `canary` workflow job checks **only this public organization** and uses no checkout, artifacts, secrets, private source, marketplace actions, or caches. The separate test job checks out its immutable PR head with read-only permissions and runs synthetic fixture tests. Neither job tests private Oreslang admission. A successful CI status alone is not evidence that a remote job actually ran steps: inspect its job ID, runner ID, attempt, log markers and exact head.

The local JSON checker is limited to a regular file of at most 128 KiB, with `schema: 1`, and outputs the five named controls with explicit `HOLD` reasons. It exits 1 when any control is held, 2 on invalid input, and explicitly returns `certified: false` even for synthetic all-true evidence. Its `REPORTED_READY_NOT_CERTIFIED` value can **never** authorize a merge, source transfer, billing purchase, or public release without independent proof.

**No settings were changed by this PR.** Keep the existing private runner, compiler, and Pages release holds until verified by actual administrators and independently reviewed source.
