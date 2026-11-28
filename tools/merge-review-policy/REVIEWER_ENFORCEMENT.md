# Named-reviewer enforcement — GitHub-native pilot

**Target:** A pull request in `ores-truffle-oreslang` is review-eligible when
exactly one *valid independent* approval is supplied by `oresoftware` **or**
`the1mills` (subject to any stronger existing review rules), with all existing
required CI, CODEOWNERS, stale-approval, last-push, and security gates preserved.

**Status: PROPOSED — NOT CONFIGURED.** This is a human-reviewed rollout
procedure, not evidence of a working ruleset. No organization or repository
ruleset, protection, or team membership has been modified by this document.

## GitHub platform contract

GitHub's **required reviewer** ruleset feature was generally available as of
February 17, 2026. For each required reviewer entry in the `pull_request` rule,
GitHub's REST API documents these fields:

```json
{
  "required_reviewers": [
    {
      "reviewer": { "id": 123456, "type": "Team" },
      "file_patterns": ["*"],
      "minimum_approvals": 1
    }
  ]
}
```

**The numeric `123456` is a DOCUMENTATION PLACEHOLDER, never a team ID to
apply.** The JSON above is only the relevant *nested parameter fragment*; it
is **not** a complete ruleset-creation request. The pattern `*` is a
candidate to verify, **not proof** that root, hidden, nested, generated,
configuration, and CODEOWNERS files are all covered. File-pattern negation
and precedence must be checked using real repository changes. Do not use an
exclusion (`!`) that lets security-critical paths bypass review.

Official references:
- https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets
- https://docs.github.com/en/rest/orgs/rules
- https://github.blog/changelog/2026-02-17-required-reviewer-rule-is-now-generally-available/
- https://docs.github.com/en/organizations/managing-organization-settings/creating-rulesets-for-repositories-in-your-organization

For an **organization-level** ruleset targeting private repositories, check
that the actual GitHub organization plan supports the feature (GitHub Team or
Enterprise per GitHub docs). The current connected GitHub integration has
returned HTTP 403 on branch protection and a plan-related HTTP 403 on private
rulesets, so no such privilege or entitlement is established.

## Pilot: owner-admin review and setup

1. Inventory all active repositories and their default branches with a
   **separate org-owner-authenticated source**. Reconcile the inventory against
   the reviewed TSV and expected count. Do not assume the former count of 80
   is still correct, and do not copy private repository source into public
   runner workspaces.
2. Confirm the `merge-approvers` team actually exists in
   `ores-truffle-oreslang`. Create it **only if explicitly authorized by the
   org owner**. Verify its **only intended reviewers** are `oresoftware`
   and `the1mills`; record exact member logins, team ID, visibility and
   privileges. The team must have **write or higher** on the pilot repository
   for its approvals to count. Do not grant excess repository privileges
   solely to pass an approval check.
3. Choose a single eligible pilot repository. Obtain an owner-authenticated
   snapshot of *all* active org/inherited/repository rulesets, branch protection,
   status checks, bypass actors and CODEOWNERS. Incompatible/unknown rules
   are a manual blocker. Record the immutable pre-change snapshot.
4. Define an org-level **branch** ruleset targeting only this repository's
   default branch. Start **disabled** or in `evaluate` mode, where available;
   **do not claim enforcement** in either mode. Add a `pull_request` rule
   containing `required_reviewers` for the verified Team ID and
   `minimum_approvals=1`. Review all file patterns and effective settings
   before activation. Never create additional bypass actors.
5. Test file-pattern coverage at minimum with: root `README.md`,
   `src/example.rs`, `src/nested/example.rs`, `.github/CODEOWNERS`,
   `.github/workflows/example.yml`, and a dotfile. A ruleset is not
   authorizing until the team requirement applies to **every** intended file,
   including changes to the rule-defining policy files.
6. Where an existing *numeric* required approval count exceeds one, first
   verify the complete effective policy, then separately review a change to
   that count. The scoped Rust tool in this PR only patches that numeric field;
   it **does not** set up the named team rule. Do not weaken unrelated CI,
   CODEOWNERS, last-push, stale-approval or dismissal rules.
7. Restore executed private GitHub Actions evidence first. A job with
   `runner_id=0` or `steps=[]` is an admission failure, not a passing or
   failing Rust test. Keep exact submitted PR SHA distinct from GitHub's
   synthetic merge ref.
8. Activate the pilot **only with owner approval and verified plan support**,
   then run the acceptance matrix below against GitHub's actual effective
   pull-request merge state. Roll back the *new pilot rule* if it produces
   unexpected blockers or bypasses. Preserve the original protection snapshot.

## Required live merge-eligibility matrix

| Scenario | Expected effect |
| --- | --- |
| PR by `the1mills`, approval by `oresoftware` | Named reviewer gate satisfied |
| PR by `oresoftware`, approval by `the1mills` | Named reviewer gate satisfied |
| PR with no approval | Blocked |
| Only an unrelated user's approval | Named reviewer gate **not** satisfied |
| Author's own review, if GitHub permits submission | Not a valid independent approval |
| Named reviewer approves but required CI fails/is missing | Blocked |
| Stale review or incompatible last-push approval | Blocked under active stale/last-push settings |
| Changes requested by an authorized reviewer | Blocked until the review is resolved |
| Changes to root, nested, dotfile, workflow and CODEOWNERS | Required reviewer team applies to all intended paths |

A successful named-reviewer gate **does not** automatically make the PR
mergeable: every other effective GitHub requirement must still pass.
Likewise, one numeric approval does **not** prove that an approver belongs to
the named reviewer team.

## Rollout exit criteria

A rollout receipt must contain the exact repository, default branch, ruleset
ID, Team ID and current membership, before/after protection and active ruleset
snapshots, exact PR head/tree SHA, nonzero executed CI steps, approval identity,
and GitHub effective merge eligibility for each positive and negative case.
Proceed to the next batch only after an independent reviewer confirms these
receipts. Log unsupported repositories and authorization failures as
`MANUAL`; never report those as policy-compliant.

Track the engineering implementation in
https://github.com/ores-truffle-oreslang/.github/pull/36 and entitlement/
admin remediation in
https://github.com/ores-truffle-oreslang/.github/issues/37.
