# Security policy and responsible disclosure

This repository is **public** and contains organization-wide engineering guidance. Never commit credentials, private source inventories, security incident details, internal mirrors, vulnerability exploit instructions or proprietary test logs.

## Reporting a security vulnerability

**Do not open a public issue** describing an exploitable vulnerability or post secrets in a PR. Prefer **GitHub's private vulnerability reporting / security advisory channel** for the affected repository **when enabled** (the repository's Security tab). If that private channel is not configured, contact an authorized maintainer through an established private organizational channel and request a safe disclosure route; do not assume a public issue is private. Provide affected repo/revision, severity, impact and a minimal safe reproduction privately. Do not send production secrets or personal data.

Maintainers should acknowledge, triage, document scope, arrange coordinated fixes and remediation communications, and agree on a disclosure timeline appropriate to the issue. No fixed SLA is asserted until organization owners publish one.

## Engineering security baseline

- **Access:** least-privilege repository/team permissions, strong authentication, protected default branches, required independent review and scheduled access recertification. Revoke access promptly during offboarding.
- **Supply chain:** pin toolchains, workflows and dependency locks; verify immutable source Git commit/tree and release artifact digests; scrutinize dependency or runner changes.
- **Execution:** compiler source is potentially hostile. Bound parsing/IR recursion, allocations, offsets, execution time, package archive extraction, file descriptors and compiler subprocesses. Do not run untrusted guest programs outside the approved sandbox/profile.
- **Runtime:** exercise actor lifecycle, capability gating, mutable memory/domain boundaries, stale handles, cancellation paths and supervisor error propagation with adversarial negative tests. No claim that actors alone provide OS-process isolation.
- **Secrets:** use approved secret management, never print secret values in build logs, and never copy credentials into sample fixtures.
- **CI:** a missing runner or zero executed steps is NOT_EXECUTED, not proof of safety; never waive tests/protection to get a green badge.

## Private-source alternate CI safeguard

When Actions minutes are unavailable, do **not** mirror a private compiler into a public repository, paste repository tokens, or authorize a carrier solely because its Git hash matches. An approved publisher must verify **private visibility of both repositories**, the authorized carrier allowlist, its write scope and exact source commit/tree **before and within the write path**.

Use a new isolated temporary branch and Git object APIs, verify every copied blob SHA and the full root tree SHA, execute with carrier-local credentials only, and retain immutable provenance and audit logs according to approved policy. Do not overwrite main, merge the mirror into the carrier, or weaken source protection. If a carrier cannot meet these requirements, stop and record NOT_EXECUTED.

## CODEOWNERS and limitations

The [CODEOWNERS](CODEOWNERS) file in this repository applies to **this repository only**. Organization default community files do not install CODEOWNERS, branch rules or security settings in unrelated compiler repos. Each sensitive implementation repository needs its own verified code owners and repository rules; granting an engineer write access does not automatically make a review independent.

See [TESTING.md](TESTING.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
