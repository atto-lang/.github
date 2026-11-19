# Compiler engineer onboarding

This is the baseline checklist for engineers joining AttoLang compiler/runtime work, Oreslang-named backend compatibility, or Zed-Pkg compiler integrations. A designated technical lead adapts the tasks to each actual repository and target.

## Before access is granted (manager / repo owner)

- [ ] Assign a technical mentor, direct responsible area, reviewer, first issue and backup escalation contact.
- [ ] Complete the organization's employment/contract, IP, confidentiality and security agreements as required; do not publish personal details in GitHub.
- [ ] Enable strong authentication and least-privilege team/repo access. Never share a founder PAT or an organization-wide token.
- [ ] Confirm GitHub branch rules, CI capacity, security advisory route and permissible private mirror destinations.
- [ ] Provide machine setup, approved dependency/toolchain versions and a non-secret minimal test fixture.

## Days 1–3: understand and reproduce

- [ ] Read [ARCHITECTURE.md](ARCHITECTURE.md), [BORROW_CHECKER.md](BORROW_CHECKER.md), [BUILDING.md](BUILDING.md), [CONTRIBUTING.md](CONTRIBUTING.md), [TESTING.md](TESTING.md), [RFC_PROCESS.md](RFC_PROCESS.md), [SECURITY.md](SECURITY.md) and the assigned repository's README.
- [ ] Find the current normative language spec, matching versioned conformance fixtures and backend capability statement.
- [ ] Check out the assigned repository on an isolated branch; record exact HEAD, root tree SHA, platform and toolchain.
- [ ] Run its documented local build and native tests; run a **real guest program** if the target supports that profile.
- [ ] Report any runner admission failure or unsupported feature explicitly; do not manufacture green results.

## Days 4–7: first contribution

- [ ] Pick a small, already triaged issue with specific expected success and negative cases.
- [ ] Create a focused PR; link issue, source contract and cross-backend dependencies.
- [ ] Add Rust test tooling if new harness code is needed (no new Python tests/utilities).
- [ ] Attach executed positive and negative tests, artifact digest and exact-head receipt; request an independent reviewer.
- [ ] Learn the escalation policy for ambiguity, capability/security impacts and runtime regressions.

## Weeks 2–4: own a bounded milestone

- [ ] Take one backend/semantic/packaging vertical slice, with an issue owner and reviewer.
- [ ] Write a short design note or RFC for changed observable semantics **before** implementing divergence.
- [ ] Expand negative and adversarial regression coverage and update capability/evidence matrix.
- [ ] Demonstrate one reproducible compiler guest or package consumer acceptance gate; record what still is NOT_EXECUTED.
- [ ] Complete a retrospective: build friction, missing tests, architecture risks and next dependencies.

## First pull request minimum template

~~~text
Issue / scope:
Normative spec or approved RFC:
Exact source head + tree:
Target and toolchain:
Changed behavior and unsupported cases:
Executed positive guest tests:
Executed negative/adversarial tests:
CI jobs/step counts, artifact digest and logs:
Cross-backend contract/fixtures:
Security and compatibility implications:
Reviewer and unresolved blockers:
~~~

## Day-to-day expectations

Prefer small verifiable changes over broad speculative refactors. Do not bypass review or weaken negative tests to produce a green check. Use snake_case for AttoLang callable identifiers. Protect personal and business information in this **public** documentation repository. Direct implementation PRs to the appropriate backend; do not add compiler sources here.

See the [roadmap](ROADMAP.md) for priorities; issue assignees and accepted RFCs determine current work, not historical discussion alone.
