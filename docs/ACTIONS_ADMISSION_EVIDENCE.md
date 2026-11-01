# Verified GitHub Actions runner admission

`scripts/ci-evidence/verify-actions-admission.sh` accepts either immutable fixture JSON or `live owner/repo run-id head-sha [attempt]`. Live mode uses only audited `/usr/bin/gh`, reads run metadata and **attempt-specific** job inventory, and rejects partial pagination, duplicate jobs, unassigned runner, empty steps, failure/skip, wrong head/attempt and malformed evidence. No script step may be counted as native Oreslang compiler execution merely because CI succeeded.

`scripts/ci-evidence/test-verify-actions-admission.sh` runs 1 positive and 19 adversarial negative controls against synthetic job JSON. The selftest job requires real assigned runner steps itself; a zero-step check is **NOT_EXECUTED**.

The verifier is source-free and may be tested in another organization without exporting any private Oreslang source. Never use it to authorize a private source transfer or merge without independent review and approved destination identity.

The verifier does not prove source-tree contents or compiler binary identity; those need separate exact SHA/tree/blob and guest receipts.
