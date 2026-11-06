# AttoLang merge-receipt v1 — fail-closed offline snapshot gate

This repository publishes an **offline untrusted JSON validator**, not a trusted GitHub collector, an authorization to merge, a formal proof, production branch protection, or compiler-backed AttoLang guest evidence. Run with Node.js 22+:

```sh
node --test tests/verify-merge-receipt.test.mjs
node tools/verify-merge-receipt.mjs /path/to/receipt.json
```

Input is JSON with `schema: "atto.merge-receipt/v1"` and `pr`, `checkout`, `protection`, `workflow`, `reviews`, `merge` fields. See the runnable `good()` fixture in `tests/verify-merge-receipt.test.mjs` for the exact data shape.

Checks include original 40-hex PR head and tree identity, non-synthetic clean physical checkout, run/attempt/job identity, duplicate names, completed workflow and jobs with real runner IDs and nonempty steps, at least one successful real test step, independent current-head approval, no unresolved change requests, actual main-branch protection assertions, CODEOWNERS, stale-approval dismissal, no admin bypass, merge/ancestry identity, and chronology. `steps=[]` or `steps=null` is runner-admission failure, **not** compiler or test evidence.

## Never translate offline validity into release approval

The input JSON is completely untrusted and may be forged, including Git hashes, review data, timestamps, protected status and step counts. Even a conforming snapshot returns `SNAPSHOT_VALID_RECHECK_LIVE` and exit status **3** (never zero). A negative snapshot returns `BLOCKED`, exit **1**. Usage, read or parse failure exits **2**. A shell's `set -e` must not accidentally mark a valid snapshot as certified success. No possible exit code grants authorization.

Only a separate authenticated live collector, read-only GitHub permissions, source-owned execution logs and an independent reviewer can establish release readiness. Re-read PR source head before/after qualification, verify Git tree/blob identity, check the complete job attempt and actual executed steps, verify reviewer approvals were submitted by other humans and not stale, read back the effective protected branch and required App-bound checks, and verify postmerge `main` ancestry. See existing platform-conformance verifier work; this does not overwrite or adopt its pending PR branches.

## Administrator activation sequence

1. Restore private source-owned GitHub runner admission; demonstrate that a real test job executes against an exact head. Public source-free canaries do not establish guest backend parity.
2. Configure verified `main` branch protection/rulesets on each relevant repository: PR-only changes, >=1 independent approval, enforced CODEOWNERS, stale approval dismissal, conversation resolution, no admin bypass/force push. Confirm effective `protected=true` via authenticated readback.
3. Pin required status check contexts **and** the authorized GitHub App. Test missing check, skipped check, runner admission `steps=[]`, stale review, self review and failed native test as negative controls *before* enabling a required check. Do not require checks that cannot execute.
4. After a SHA-guarded merge, verify original head, merge SHA, source Git tree, and `main` ancestry; append immutable receipts. Previously merged but governance-unqualified PRs require independent **retrospective** review, not fabricated historical approval.
5. Keep host-only results and all five backend compiler-native `.atto` guest executions in different evidence categories. Do not mirror private source to unapproved organizations or external carriers.

Implementation in this PR is limited to **three new files**. It does not modify the existing `.github` governance #27 or audit #28 branches, production code, workflows, permissions, security policies, or repository default branches.
