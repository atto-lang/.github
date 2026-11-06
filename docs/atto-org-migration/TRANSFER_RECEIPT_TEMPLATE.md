# Transfer acceptance receipt template (one per repository)

Every repository transfer must have a **separate** signed/reviewed receipt (e.g. \`receipts/<repo-id>.md\`). Copy this template before preflight. No field may be silently inferred from a green check icon.

## Identity (before transfer)
- Source full name:
- Stable GitHub repository ID:
- Intended destination full name (no simultaneous rename):
- Source visibility / intended destination visibility:
- Source default branch:
- Source exact commit SHA:
- Source Git tree SHA:
- Source open PR count and issue count:
- Source release/tag count:
- Source active package, Pages, webhook and deployment integrations:
- Source branch protection and ruleset evidence URLs (403/unavailable = BLOCKED):
- Source GitHub App installation / Actions access and billing preflight:
- External dependency/consumer change list:
- Human migration owner:
- Independent security reviewer:
- Destination org owner/admin approving transfer:
- Approval evidence IDs and timestamps:

## Preflight fail-closed gates
- [ ] Source/target owners and security reviewer explicitly approved *this repository and destination*.
- [ ] Private-source access and private fork security boundaries reviewed (if applicable).
- [ ] No name collision or unintended visibility elevation; feature/plan parity verified.
- [ ] Current source commit SHA/tree and active PR/release state refreshed.
- [ ] Branch rules, signed releases, CODEOWNERS and permission model reviewed.
- [ ] All relevant CI jobs have **non-zero executed steps** on the pinned source head.
- [ ] Real guest compile+run and negative checks demonstrated if language/runtime parity is asserted.
- [ ] Consumers, automation, package identities, Pages and rollback/cutover strategy documented.
- [ ] Independent coordinator claim is recorded at issue #26 to avoid agents working on same repo.

## Execution
- Transfer action URL/audit log:
- Executed by:
- UTC timestamp:
- New repository URL:
- Destination stable repository ID:
- Destination exact default-branch SHA:
- Destination Git tree SHA:
- Mismatches, if any:
- Actions/permissions/rulesets post-transfer review:
- PR/issue/history and release/tag preservation check:
- Package namespace / marketplace / Pages / webhook verification:
- Source URL redirect check (not valid for every GitHub service):
- Re-run Actions run IDs / job IDs / attempts / executed step count:
- Re-run negative and guest execution evidence:
- Cutover PRs, consumers verified, and final approvals:

## Mandatory assertions
- [ ] Same stable repository ID before and after.
- [ ] Exact source head SHA and Git tree verified unchanged at the point of transfer.
- [ ] Desired visibility and effective collaborator access verified.
- [ ] Branch protection is **enforced**, not just listed in a proposed document.
- [ ] No zero-step or host-only result was credited as compiled-guest evidence.
- [ ] Old namespace cleanup or explicit compatibility exception is recorded.
- [ ] Manifest row updated to \`verified\` only after this receipt is complete.

**Rollback:** Stop dependent cutovers on any mismatch. Restoring repo ownership, external package identities, or GitHub Pages behavior is *not* guaranteed to be atomic or automatically reversible. Escalate with the destination owner and restore integrations according to the reviewed plan rather than attempting an unapproved reverse transfer.
