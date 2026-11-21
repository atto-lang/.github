# Repository-local governance drift audit — 2026-11-21

**Current governing target:** exactly one independent actual GitHub APPROVED
review from either `the1mills` (ID 2148448) or `ORESoftware` (ID 11139560).
This rule does NOT require AI-provider reviews, age-dependent Ready-cycle
quorums, renewal at 240h, or human review after every push. The PR author
cannot self-approve. Existing stronger effective GitHub protections remain
binding until independently migrated; a policy document does not install a
ruleset.

## Historical default-branch policy inventory

Earlier work attached to this PR reported scanning 88 atto-lang repositories:
72 Atto-local boilerplate copies still requiring four parties, 15 imported
Oreslang boilerplate copies, and one central canonical repository. Those
counts describe that earlier source-text inventory, **not** effective GitHub
ruleset state, not an exhaustive independently refreshed scan in this change,
and not approval to bulk-replace protected policies.

The central canonical policy is
[atto-lang/.github AGENTS.md](https://github.com/atto-lang/.github/blob/main/AGENTS.md).
Any repository-local `AGENTS.md` conflicting with its one-owner policy
requires a separately reviewed patch, retaining project-specific instructions
and respecting the current effective GitHub code-owner and branch rules.
Under the one-owner rule, `the1mills`-authored PRs require genuine
`ORESoftware` approval and vice versa.

## Safe rollout

1. Read the current repo's effective rules and `CODEOWNERS` where authorized.
   Report `403 Resource not accessible by integration` rather than inferring
   the protection is absent or has migrated.
2. Keep distinct Draft-age >=72h promotion, current-head material CI,
   `keep-as-draft`/`keep-draft`/`not-ready` holds, security review and
   mandatory semantic overlapping-PR audit.
3. Scope local boilerplate changes to the explicit org policy block; preserve
   unrelated project security requirements and migration crosswalk notes.
4. Aim for one genuine accepted trusted-owner approval per Ready PR (independent
   of author), with no repeated approval requests for unchanged heads. A
   dismissed approval or currently enforced last-pusher rule remains binding.
5. Do not infer merge permission from the advisory JavaScript evaluator. All
   effective GitHub protections, executed required checks and exact HEAD
   constraints remain authoritative.

**No source code, branch protection or GitHub rulesets were modified by this
audit document.**
