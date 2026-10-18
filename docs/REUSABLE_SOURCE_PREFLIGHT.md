# Reusable exact-source-tree preflight (issue #6)

Call `.github/workflows/reusable-source-tree-preflight.yml` from a trusted
Oreslang repository using a **pinned commit SHA** of the shared workflow.
Callers must supply the expected 40-character Git commit and tree SHA for
the *actual checked-out CI revision*. For pull requests GitHub may check out
a synthetic merge commit rather than the contributor branch head: do not
silently substitute the contributor SHA and treat mismatched trees as proven.

The job uses read-only repository permissions, pinned checkout action,
no persisted credentials, no elevated secrets, no third-party package
installation and no private-source cross-org export.

## What it proves

If and only if a runner executes the job, the emitted values prove the
local checked-out Git commit and tree matched the supplied identifiers.
An Actions run with zero job steps cannot produce this evidence.

This gate **does not** authenticate who built the compiler, verify
source-to-binary correspondence, compile code, run tests, validate
TypeSpec/JSON Schema parity, parse WIT, prove Dafny/TLA+ obligations,
or grant cross-backend conformance. Those remain separate, individually
executed and independently reviewed release gates.

Never interpret the `checked_commit` or `checked_tree` job output as
a `VERIFIED_BACKEND` result. Only an approved private runner may handle
private-source source/materialization work, with exact-tree and retention
policy checks.
