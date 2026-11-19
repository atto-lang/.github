# AttoLang four-party ready-cycle review audit (read-only)

This synthetic, *advisory-only* validator checks the verified ready-cycle timeline, exact head SHA and last push, independently authenticated provider attestations for OpenAI/Anthropic, and distinct numeric GitHub identities for ORESoftware/the1mills. The ready-age policy is <72h four distinct; 72h–240h one independent; >=240h four fresh current-SHA approvals.

It rejects four approvals from one GitHub ID as only one principal. That failure class was observed in merged atto-lang/atto-source.rs#385 (four APPROVED entries from ORESoftware ID 11139560). This cannot retroactively certify or undo the merge; a separate post-merge security review is needed.

**Not enforcement:** inputs must come from a trusted GitHub/App and provider-attestation integration, not PR text. This module performs no network read, merge, GitHub branch rule changes, approval, or release. Merge gating needs an independently installed trusted app and protected branch/status-check configuration. The tests use synthetic evidence and are not approval artifacts.
