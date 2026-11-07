# Organization-wide one-approval review audit and guarded patch

For `ores-truffle-oreslang`, the requested merge policy is: **one valid approval from `oresoftware` or `the1mills` meets the numeric review threshold**. Approvals must be independent; authors cannot approve their own PRs. Required CI, code-owner, last-push, and other security gates remain in place.

> **IMPORTANT:** This tool does not change review settings merely by being merged. It needs an authorized operator with GitHub admin permissions for the target repositories and a working `gh auth login` session. It does not authorize bypasses or remove code-owner restrictions. It cannot provide a blanket guarantee on GitHub Free private repositories where protection enforcement may be unavailable.

Run after `gh auth status`:

```sh
# Read-only audit; includes public/private accessible repositories
cargo run --manifest-path tools/merge-review-policy/Cargo.toml --

# Verify a single repository change first
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- --repo oreslang-source.java --apply

# Only after auditing the output, apply exact numeric changes across accessible repos
cargo run --manifest-path tools/merge-review-policy/Cargo.toml -- --apply
```

Fail-closed behavior:

- **Dry run by default.** `--apply` is required for each write.
- Uses GitHub's branch-protection review endpoint, not the full protection `PUT`, so it doesn't overwrite required checks or unrelated security settings.
- Updates an existing numeric threshold to exactly **one**. If the branch is protected but no numeric review rule exists, it is marked `MANUAL`: creating review protection would require a separate, carefully audited full-policy change.
- Refuses to mutate a repository with an active local or inherited ruleset without inspecting that ruleset separately. Rulesets can override a repository's numeric threshold.
- Refuses to auto-modify code-owner approval, because changing CODEOWNERS without auditing existing ownership would weaken or alter security semantics. Those repositories are marked `MANUAL`.
- Marks unprotected or inaccessible branches `MANUAL`. It never creates a new branch-protection rule or silently assumes it is enforceable on the current organization plan.
- Reads the branch protection again after every write and reports verification success/failure.
- `gh` uses the signed-in user's credentials; never store tokens in this repository or place them in CLI arguments.

**Limitations:** GitHub's numeric review count does not, by itself, restrict *which* collaborator approves; it permits valid reviews by any eligible reviewer. Enforcing strictly `oresoftware`/`the1mills` requires an effective CODEOWNERS and code-owner review requirement (or a plan-supported team/ruleset). The tool deliberately does not simulate or bypass such controls. It also cannot clear blocking `CHANGES_REQUESTED` reviews; those require authorized dismissal or renewed approval.

Review all `MANUAL` results before claiming organization-wide parity. Add executed tests and a checked deployment receipt before promotion.
