# Ready-cycle approval gate: protected deployment plan

This module is a source-only, READ-ONLY policy evaluator stacked on draft PRs #49 and #50. It does not register a GitHub App, change branch protection, authorize merges, or count itself as a verified OpenAI/Anthropic reviewer.

## Required contract
- During the first 72h of the latest Ready cycle: BOTH named GitHub humans (ORESoftware account ID 11139560 and the1mills account ID 2148448) AND independently verified OpenAI and Anthropic approvals.
- From 72h to before 240h: ONE of the named humans AND ONE of the named AI providers.
- At and after 240h: fresh approvals from all four, submitted after the 240h boundary.
- Any legitimate Ready -> Draft -> Ready sequence starts a fresh clock, and previous-cycle approvals do not count. No author self-approval. Both canonical keep-as-draft and legacy keep-draft are absolute Draft-state/merge holds; not-ready blocks readiness and merging.

## Live read-only audit
The atto-ready-audit binary requests the current PR, complete lifecycle timeline, human review history, and named check runs using authenticated GitHub CLI, with no checkouts of untrusted PR code. The evaluator rejects wrong identities, stale SHA, duplicate or out-of-order evidence, incomplete timeline/review/check pages, unresolved changes requested, source/provider spoofing and malformed timestamps. All results are advisory only and merge_authorized is always false.

Two independent trust boundaries are intentionally NOT inferred from PR API data:
1. Last-push timestamp: record a verified GitHub App synchronize/creation webhook with delivery ID, exact head SHA and UTC event timestamp in a trusted event store; never substitute commit author date or PR-body text. Current read-only collector sets push_verified=false.
2. OpenAI/Anthropic identity: separately verify each provider's installed GitHub App ownership, numeric ID, attestation review semantics, and ability to issue the exact check context atto-ai/openai/review-attested or atto-ai/anthropic/review-attested. The test values 100001/100002 are synthetic and must NEVER be production identities. Ordinary GitHub Actions app id 15368 never counts. Current read-only collector has NO trusted bindings and deliberately returns HOLD.

## Deployment phases
1. Independent code/security review and compliant merge of #49, #50, and this stacked Draft PR without bypassing approval rules. Author the1mills cannot self-approve; day 3-10 fast lane requires independent ORESoftware plus authenticated AI.
2. Admin-authorized GitHub App and immutable webhook event storage. Authenticate webhook HMAC signatures, validate delivery IDs and monotonic event histories, reject replay, and re-fetch live data at each evaluation. Store and verify exact SHA push receipts and provider-specific attestation bindings with out-of-band evidence.
3. Deploy the application as a trusted service or protected automation, not as untrusted pull_request code. Protect CI permission scope and only run immutable, independently reviewed controller source. Start with read-only audits on atto-lang/.github and atto-lang/atto-lang.github.io.
4. Require two distinct source-bound statuses under branch rules: atto-governance/draft-hold and atto-governance/reviews-verified. Require exact-head material CI and stronger CODEOWNERS/branch protection separately. An issuer name alone is NOT sufficient; bind required check to the expected trusted GitHub App.
5. Reconcile on every relevant label, review, Ready/Draft and synchronize event AND periodically/at 72h and 240h. Serialize per PR and re-evaluate in an authorized merge broker immediately before merge to narrow races. Prevent ordinary UI/admin bypass where platform capabilities allow; otherwise report limits, not absolute enforcement.
6. Pilot manual Ready with keep-as-draft, label-add race on same SHA, absent and duplicated webhook, missing app binding, forged human ID, exact 72h/240h boundaries, re-draft reset, stale approvals and source Actions zero-step failures. Record actual executed job steps and rollback instructions.
7. Expand to every active atto-source.* repository, then the remaining atto-lang organization, verifying installed check issuer, protection rules, active reconcile service, SHA-pinned policy source, test evidence, and approved exceptions. Private cross-org CI source mirroring requires independently approved private destination plus per-blob and full-root-tree SHA matching.

Until all steps are deployed and live-tested, this is a validated policy implementation, NOT a production-enforced organization rule. A successful simulated unit test is not evidence of a live PR conversion or a real AI provider review.
