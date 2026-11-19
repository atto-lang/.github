# Request for Comments (RFC) process

## When an RFC is required

Open an RFC before changing normative syntax, type rules, ownership/borrowing, actor semantics, errors/effects, module/const initialization, import/package resolution, ABI/wire formats, security capabilities or backwards compatibility across backends. Ordinary internal refactors that preserve observable semantics can proceed with an implementation issue and tests.

The [AttoLang specification](https://github.com/atto-lang/atto-spec) owns normative AttoLang language decisions. Backend-specific compatibility or migration requirements must be coordinated with the current `atto-source.*` owners and versioned in [atto-spec](https://github.com/atto-lang/atto-spec). **This organization .github repository does not ratify language semantics.**

## RFC lifecycle

1. **Draft:** open an issue and proposal PR in the canonical specification repo. State problem, target behavior, examples, alternatives, compatibility impact and security implications.
2. **Review:** request a language owner, an independent backend representative and a conformance reviewer. If Zed/import/package behavior changes, include a package-manager maintainer. Collect design objections and record decisions in the PR.
3. **Accepted / Rejected / Deferred:** decision authority is designated by repository maintainers. Record the decision, reviewers, exact commit, migration and unresolved limitations. Silence or an unreviewed PR is not acceptance.
4. **Specification and fixture:** merge the versioned contract and negative oracles. Assign independent backend implementation issues and explicit support profiles.
5. **Implementation:** each backend links its PR and fixture version. It reports supported, unsupported, not executed and failed cases honestly; a parse-only implementation stays parse-only.
6. **Promotion:** only after real guest execution, security/regression cases and independent evidence can a backend claim conformance for a profile. A proposal's acceptance does not auto-update backend support.

## RFC template

~~~text
Title:
Status: Draft | Accepted | Rejected | Deferred | Superseded
Author and reviewers:
Linked issue / specification PR:
Problem and user-visible examples:
Grammar / type / ownership / effects rules:
Runtime, actor, AOT and isolation consequences:
Error diagnostics and fail-closed behavior:
Backend differences and explicit unsupported profiles:
Package/CLI/ABI/source compatibility:
Positive, negative and adversarial conformance vectors:
Alternatives and tradeoffs:
Rollout and migrations:
Decision, date, references and unresolved concerns:
~~~

## Governing principles

- Standard callable naming uses snake_case; new source examples prefer canonical .atto where executable admission is established.
- Do not change established spellings or redefine operators by assumption. Cite the current merged specification or label examples as **TARGET**.
- Ownership operations, actor sealed-state contracts, lifetime syntax, checked effects and total environment reads must be assessed together for evaluation order and safety.
- A feature can be accepted in the language spec while remaining UNIMPLEMENTED or UNSUPPORTED in individual compilers.
- If implementations diverge, log a spec/conformance issue and fail closed rather than choosing behavior by whichever backend is easiest to modify.
- Breaking changes require versioning, migration examples, and explicit tests for retired implementation interfaces, migrated CLI names, and changed ABIs.

See [ARCHITECTURE.md](ARCHITECTURE.md), [TESTING.md](TESTING.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
