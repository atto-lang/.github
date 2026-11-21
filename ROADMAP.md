# Compiler engineering roadmap

**Status: prioritization proposal, 2026-11-18.** This document is **not** a report of complete functionality or a commitment to delivery dates. The authoritative status of a feature is its spec, exact compiler implementation, issue/PR and executable conformance receipt. Maintain a GitHub Project for named owners, dependencies and milestones; do not use this public repository as a private backlog mirror.

## P0 — admission, safety and executable foundations

**Exit criteria:** developers can produce reviewed, reproducible evidence for an agreed minimal language/runtime slice without bypassing CI or hiding unsupported features.

1. CI governance and execution: protected branches, qualified reviewers, exact-head source/tree receipts, verified runner admission, and safe private carrier capacity when allowed. No zero-step passes.
2. Semantic soundness: typed resolution, HIR/MIR verification where present, ownership/borrow-by-default and move/drop rules, checked exceptional effects, name/visibility and deterministic diagnostics.
3. Actor correctness: message admission and closing, sealed state read/write, supervision priority CONTROL delivery, panic/raise/throw and cancellation, tenant boundary and capability denial tests.
4. Executable backend acceptance: real native/JVM/Wasm/Native Image paths on each claimed target with positive, negative and lifecycle tests; record AOT and sanitizer evidence separately.
5. Zed-Pkg compiler integration contract: manifest/import discovery, target-aware package identity, frozen-lock and artifact provenance, adversarial path/metadata tests and reproducible consumer builds.

## P1 — portable parity and developer experience

- Expand portable language conformance by vertical feature slices (collections, Option/Result and pipelines, generics, memory layout, standard I/O, modules/imports).
- Validate compiler guest execution across independently implemented backends instead of treating reference-only tests as parity.
- Standardize diagnostics, formatter/IDE contracts, source package consumption, hermetic build recipes and documentation.
- Characterize memory usage, actor throughput/tail latency and AOT binary size with reproducible workloads, not unsourced performance claims.

## P2 — ecosystem breadth and performance engineering

- Broader platform SDKs, embedded and browser integrations, package publishing, cross-platform distribution, advanced scheduler and optimization profiles.
- Additional backend targets and heterogeneous/accelerator work only where source, runtime and conformance evidence justify support.
- Public demos and examples that compile/run at verified exact revisions.

## P3 — longer-horizon research

- Compiler self-hosting, additional VM/tenant architectures, ambitious distributed-runtime profiles and experimental hardware targets.
- Promote only after an approved RFC, implementation owner and executable qualification plan.

## Milestone gates and dependencies

| Gate | Deliverable | Must precede |
| --- | --- | --- |
| M0: operational baseline | Maintainers, branch rules, runnable CI, build receipts, secure review flow | Claims of verified delivery |
| M1: semantic contract | Approved RFC + typed checks + negative fixture suite | Backend codegen promotion |
| M2: backend slice | Correct emission, actual guest tests, deterministic artifacts | Portable parity claim |
| M3: cross-backend result | Independent per-target evidence and discrepancy log | User-facing compatibility promise |
| M4: distribution | Pinned Zed package and compiler consumer integration, supply-chain checks | Release or onboarding examples |

Each milestone issue must have a directly responsible engineer, independent reviewer, prerequisites, exact test commands, pass/fail oracle and explicit definition of done.

## Planning process

Use two-week planning windows with architecture checkpoints and issue-level acceptance criteria; prioritize P0 blockers before taking on speculative breadth. Link individual backend work to an umbrella conformance/spec task. Track **PASSED / FAILED / UNSUPPORTED / NOT_EXECUTED**, not a single undifferentiated percentage called parity.

Related guidance: [RFC_PROCESS.md](RFC_PROCESS.md), [ONBOARDING.md](ONBOARDING.md), [TESTING.md](TESTING.md).
