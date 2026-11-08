# Repository responsibility and safe migration guide (snapshot: 2026-11-08)

**Private coordination document — not an executed or approved transfer.** This directory lives in the source organization's private `.github` repository; do not republish private repo inventory into public repositories or unaudited CI logs.

## Organization contract

- **`atto-lang`**: AttoLang public language, canonical `.atto` source, specifications, portable core/std, language libraries, independent compiler backends, conformance, development tooling, and language demos.
- **`atto-co`**: separately governed cloud/product operations such as identity, billing, deployment control plane, registry, build farm, hosted runtime, observability SaaS, integrations and marketplace. Language-level SDKs and portable contracts can remain in `atto-lang`.
- **`ores-truffle-oreslang`**: current historical source namespace while approved migration is prepared. Do not treat renamed or similarly named repos already in `atto-lang` as transferred copies; compare stable GitHub repository IDs, commit trees and provenance before integration.

**Existing migration proposal:** [draft #25](https://github.com/ores-truffle-oreslang/.github/pull/25) proposes **74 initial moves to `atto-lang`, six holds, zero direct moves to `atto-co`**, and no executed moves. [Coordination issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26) controls ownership claims. [Read-only identity audit #40](https://github.com/ores-truffle-oreslang/.github/pull/40) is separate and must not be conflated with migration approval. A later `atto-lang` → `atto-co` product carve-out requires its own approved process; do not silently bypass the approved staged plan.

## Scope map of all 80 source repositories

The following is a **policy for where new code/docs should live**, not a claim each feature is implemented or tested. The target in the last column is proposed; existing same-purpose `atto-*` projects must be audited for code/ownership overlap and are not evidence that any source GitHub repo has moved.

| Source repository | Domain | What belongs here | What does not belong here | First-migration decision |
| --- | --- | --- | --- | --- |
| `.github` | Governance | Organization policy, migration coordination, review and Rust-only tooling | Application or compiler implementations | HOLD — owner/exit decision |
| `async-testing` | Conformance | Async/structured concurrency test fixtures and harnesses | Scheduler implementation without backend proof | Proposed `atto-lang/async-testing` (not transferred) |
| `attolang-language-spec.md` | Language | Normative AttoLang syntax, types, effects and memory model | Backend-specific code or hosting operations | Proposed `atto-lang/attolang-language-spec.md` (not transferred) |
| `compiler-parity-website` | Conformance UI | Read-only compiler parity dashboard and provenance display | Changing the underlying CI evidence | Proposed `atto-lang/compiler-parity-website` (not transferred) |
| `gh-linguist-fork` | Legacy hold | Language highlighting fork and upstream contribution decisions | Production AttoLang compiler | HOLD — owner/exit decision |
| `html-js-css-syntax-highlighter` | Legacy hold | Standalone highlighting utilities pending owner decision | AttoLang normative syntax | HOLD — owner/exit decision |
| `ores-fm-mac-agent.ores` | Application | Local Mac agent experiment and capability boundaries | Cloud administration or unrestricted device access | Proposed `atto-lang/ores-fm-mac-agent.ores` (not transferred) |
| `ores-truffle-oreslang.github.io` | Legacy hold | Historical GitHub Pages presence and redirect strategy | Assumed automatic Pages redirect after transfer | HOLD — owner/exit decision |
| `oreslana.ores` | Legacy hold | Experimental project pending ownership and branding decision | Automatic reassignment to commercial organization | HOLD — owner/exit decision |
| `oreslang-ai-agent-coordination-via-actors-demo` | Examples | Actor-based multi-agent examples and bounded test scenarios | Commercial autonomous agent hosting | Proposed `atto-lang/oreslang-ai-agent-coordination-via-actors-demo` (not transferred) |
| `oreslang-api-server.ores` | Web demo | API-server guest examples and policy tests | Managed API gateway control plane | Proposed `atto-lang/oreslang-api-server.ores` (not transferred) |
| `oreslang-autograd` | ML | Automatic differentiation primitives | Production model hosting | Proposed `atto-lang/oreslang-autograd` (not transferred) |
| `oreslang-browser` | Web | AttoLang browser-side rendering/integration libraries | Browser engine security sandbox implementation | Proposed `atto-lang/oreslang-browser` (not transferred) |
| `oreslang-cli` | Tooling | Language compiler and developer CLI interface | Production cloud orchestration | Proposed `atto-lang/oreslang-cli` (not transferred) |
| `oreslang-common-source` | Language | Portable core/std source and shared language contracts | Host-specific compiler or business control plane | Proposed `atto-lang/oreslang-common-source` (not transferred) |
| `oreslang-compilers-parity-testing` | Conformance | Cross-compiler source fixtures and differential tests | Compiler feature implementation | Proposed `atto-lang/oreslang-compilers-parity-testing` (not transferred) |
| `oreslang-data-science` | Science | AttoLang scientific data and numerical examples | Generic cloud notebooks service | Proposed `atto-lang/oreslang-data-science` (not transferred) |
| `oreslang-db` | State library | AttoLang data storage primitives and embedded DB engine | Managed multi-tenant database hosting | Proposed `atto-lang/oreslang-db` (not transferred) |
| `oreslang-demo-restful-server` | Web demo | REST server tutorial and compiler-pinned examples | Production-ready backend claims | Proposed `atto-lang/oreslang-demo-restful-server` (not transferred) |
| `oreslang-demo` | Examples | Minimal end-to-end language demonstrations | Runtime implementation | Proposed `atto-lang/oreslang-demo` (not transferred) |
| `oreslang-demos` | Examples | Multi-platform CLI, desktop, web and interop demos | Production-host availability claims | Proposed `atto-lang/oreslang-demos` (not transferred) |
| `oreslang-design-patterns` | Examples | AttoLang design pattern examples and tests | Normative language spec | Proposed `atto-lang/oreslang-design-patterns` (not transferred) |
| `oreslang-durable-objects.ores` | State runtime | Guest durable-object identity, actor semantics and conformance | Commercial tenant hosting, storage billing or account management | Proposed `atto-lang/oreslang-durable-objects.ores` (not transferred) |
| `oreslang-format` | Tooling | Canonical formatter and formatting checks | Semantic compiler diagnostics | Proposed `atto-lang/oreslang-format` (not transferred) |
| `oreslang-forward-proxy-demo.ores` | Networking demo | Forward-proxy guest policy and secure demo contracts | Production Internet egress gateway | Proposed `atto-lang/oreslang-forward-proxy-demo.ores` (not transferred) |
| `oreslang-hot-loading-tests` | Conformance | Hot-load generation fencing and rollback tests | Production deployment orchestration | Proposed `atto-lang/oreslang-hot-loading-tests` (not transferred) |
| `oreslang-hpc-finance` | Science | Reproducible HPC finance kernels and validation | Financial advice or managed trading platform | Proposed `atto-lang/oreslang-hpc-finance` (not transferred) |
| `oreslang-http-routing` | Web | HTTP route matching and typed policy library | Ingress deployment or reverse-proxy infrastructure | Proposed `atto-lang/oreslang-http-routing` (not transferred) |
| `oreslang-language-specs.md` | Legacy hold | Historical language-spec material pending de-duplication decision | Competing unreviewed normative specification | HOLD — owner/exit decision |
| `oreslang-linked-queue` | Library | Bounded queue and collection algorithms | Backend scheduler internals | Proposed `atto-lang/oreslang-linked-queue` (not transferred) |
| `oreslang-logic.ores` | Science | Logic programs and reasoning experiments in AttoLang | Hosted LLM product | Proposed `atto-lang/oreslang-logic.ores` (not transferred) |
| `oreslang-ml-conformance` | ML conformance | ML portability fixtures, deterministic reference results and negative tests | Unexecuted guest parity claims | Proposed `atto-lang/oreslang-ml-conformance` (not transferred) |
| `oreslang-ml` | ML | ML algorithms composed from tensor/autograd APIs | Hosted model registry or billing | Proposed `atto-lang/oreslang-ml` (not transferred) |
| `oreslang-n8n-make-zapier-app` | Integration | Language-level automation/integration examples and adapters | Hosted integration marketplace operations | Proposed `atto-lang/oreslang-n8n-make-zapier-app` (not transferred) |
| `oreslang-orie-ieor` | Science | OR/IE research examples and experimental mathematical workflows | Normative compiler specification | Proposed `atto-lang/oreslang-orie-ieor` (not transferred) |
| `oreslang-otel` | Observability library | Portable OpenTelemetry API, propagation and emitters | Managed observability SaaS | Proposed `atto-lang/oreslang-otel` (not transferred) |
| `oreslang-phoenix-server.ores` | Web demo | Phoenix-style server and actor experiments | Production HTTP runtime certification | Proposed `atto-lang/oreslang-phoenix-server.ores` (not transferred) |
| `oreslang-physics-demo.ores` | Examples | Physics-oriented AttoLang examples and tests | Core numeric compiler assumptions | Proposed `atto-lang/oreslang-physics-demo.ores` (not transferred) |
| `oreslang-platform-conformance` | Conformance | Cross-backend executable parity manifests and signed evidence | Calling zero-step or host-only runs guest parity | Proposed `atto-lang/oreslang-platform-conformance` (not transferred) |
| `oreslang-platform.asic` | Target platform | ASIC target exploration and associated ABI checks | Unverified production silicon claims | Proposed `atto-lang/oreslang-platform.asic` (not transferred) |
| `oreslang-platform.desktop` | Target platform | Desktop platform adapters and UI-host integration | Cloud control-plane responsibilities | Proposed `atto-lang/oreslang-platform.desktop` (not transferred) |
| `oreslang-platform.embedded` | Target platform | Embedded target profile and constrained runtime support | General purpose hosted compute | Proposed `atto-lang/oreslang-platform.embedded` (not transferred) |
| `oreslang-platform.fpga` | Target platform | FPGA target exploration and associated ABI checks | Claims of validated silicon execution without receipts | Proposed `atto-lang/oreslang-platform.fpga` (not transferred) |
| `oreslang-platform.iot` | Target platform | IoT adapters and constrained device integration | Unscoped cloud product management | Proposed `atto-lang/oreslang-platform.iot` (not transferred) |
| `oreslang-platform.mobile` | Target platform | iOS/Android integration, platform capability and native bridges | Unrestricted mobile background serving guarantees | Proposed `atto-lang/oreslang-platform.mobile` (not transferred) |
| `oreslang-proof-of-concepts` | Examples | Isolated research prototypes with limitations documented | Production-supported library claims | Proposed `atto-lang/oreslang-proof-of-concepts` (not transferred) |
| `oreslang-restful-server-perf-benchmarks` | Benchmarks | Reproducible REST performance measurements with pinned receipts | Marketing-only performance numbers | Proposed `atto-lang/oreslang-restful-server-perf-benchmarks` (not transferred) |
| `oreslang-reverse-proxy-demo.ores` | Networking demo | Reverse-proxy guest policy and secure demo contracts | Production public ingress gateway | Proposed `atto-lang/oreslang-reverse-proxy-demo.ores` (not transferred) |
| `oreslang-rpc` | Web | Portable RPC transport abstraction and typed contracts | Cloud load balancer operations | Proposed `atto-lang/oreslang-rpc` (not transferred) |
| `oreslang-runtime-ui` | Runtime UI | Cross-platform UI runtime and host bindings | Cloud deployment control plane | Proposed `atto-lang/oreslang-runtime-ui` (not transferred) |
| `oreslang-runtime.realtime-java` | Runtime | Java real-time runtime scheduling and host primitives | Portable language semantics owned only by this host | Proposed `atto-lang/oreslang-runtime.realtime-java` (not transferred) |
| `oreslang-serialization-and-validation` | Library | Portable typed serialization and validation rules | Hosted data pipeline or billing | Proposed `atto-lang/oreslang-serialization-and-validation` (not transferred) |
| `oreslang-sg-stategraph.ores` | State library | StateGraph algorithms and actor state graph integration | Global service orchestration | Proposed `atto-lang/oreslang-sg-stategraph.ores` (not transferred) |
| `oreslang-source.cranelift` | Compiler | Independent Cranelift backend and native compiler experiments | Assumed identity with Rust compiler implementation | Proposed `atto-lang/oreslang-source.cranelift` (not transferred) |
| `oreslang-source.forge` | Compiler | Independent Forge C++/native compiler and runtime experiment | Reference-language authority or hosted products | Proposed `atto-lang/oreslang-source.forge` (not transferred) |
| `oreslang-source.hdl` | Compiler | HDL-oriented backend and hardware target research | Cloud hosting | Proposed `atto-lang/oreslang-source.hdl` (not transferred) |
| `oreslang-source.java` | Compiler | Java/Truffle/GraalVM frontend, interpreter, compiler and actor-host primitives | Portable stdlib semantics duplicated in Java or other backend implementations | Proposed `atto-lang/oreslang-source.java` (not transferred) |
| `oreslang-source.js` | Compiler | AttoLang-to-JavaScript ESM transpilation and trusted browser/Node worker experiments | Wasm tenant-actor memory isolation or Wasmtime in browsers | Proposed `atto-lang/oreslang-source.js` (not transferred) |
| `oreslang-source.llvm` | Compiler | LLVM native compiler and runtime integration | Java or .NET backend ownership | Proposed `atto-lang/oreslang-source.llvm` (not transferred) |
| `oreslang-source.net` | Compiler | CIL/.NET NativeAOT compiler, managed host ABI and validation | Java JVM imports or generic cloud runtime | Proposed `atto-lang/oreslang-source.net` (not transferred) |
| `oreslang-source.rs` | Compiler | Rust-authored compiler pipeline and native object generation | Unproven Wasmtime actor runtime or browser-Wasm backend | Proposed `atto-lang/oreslang-source.rs` (not transferred) |
| `oreslang-source.wasm` | Compiler | Native source-to-WebAssembly compilation bridge and Wasm execution integration | Unverified in-browser Wasmtime or JS transpiler ownership | Proposed `atto-lang/oreslang-source.wasm` (not transferred) |
| `oreslang-spin` | Web | AttoLang web application integration and framework experiments | Production cloud platform control plane | Proposed `atto-lang/oreslang-spin` (not transferred) |
| `oreslang-ssr` | Web | AttoLang server-side rendering library | Hosted SSR service operations | Proposed `atto-lang/oreslang-ssr` (not transferred) |
| `oreslang-stack-cli` | Deployment interface | Legacy stack/deployment CLI and migration-compatible client interfaces | Authoritative production cloud deploy/billing control plane | Proposed `atto-lang/oreslang-stack-cli` (not transferred) |
| `oreslang-static-analysis.rs` | Tooling | Rust static analysis and audit tooling | Hidden runtime enforcement without compiler integration | Proposed `atto-lang/oreslang-static-analysis.rs` (not transferred) |
| `oreslang-step-function-demo` | Examples | Actor workflow and saga examples with compensation fixtures | Managed workflow-as-a-service | Proposed `atto-lang/oreslang-step-function-demo` (not transferred) |
| `oreslang-target-profiles` | Target profiles | Portable target capability and ABI profile definitions | Compiler backend implementation | Proposed `atto-lang/oreslang-target-profiles` (not transferred) |
| `oreslang-tensor` | ML | Native tensor data structures and operations | Managed model serving | Proposed `atto-lang/oreslang-tensor` (not transferred) |
| `oreslang-test.cli` | Tooling | Native AttoLang test runner, test semantics and receipts | Backend-specific fake guest execution | Proposed `atto-lang/oreslang-test.cli` (not transferred) |
| `oreslang-wasm` | Wasm platform | Wasm guest ABI, host integration and runtime experiments | Claiming all browser or Wasmtime paths are interchangeable | Proposed `atto-lang/oreslang-wasm` (not transferred) |
| `oreslang-web-browser.ores` | Application | Experimental AttoLang browser application and embedding tests | General browser engine certification | Proposed `atto-lang/oreslang-web-browser.ores` (not transferred) |
| `oreslang-websocket-server-demo.ores` | Web demo | WebSocket guest demo and contract fixtures | Native full-duplex transport proof without actual tests | Proposed `atto-lang/oreslang-websocket-server-demo.ores` (not transferred) |
| `postgres-orm.ores` | State library | Schema-first ORM, typed SQL and migration previews | Production database service administration | Proposed `atto-lang/postgres-orm.ores` (not transferred) |
| `rx-ores-callbacks` | Reactive | Reactive callback bridges with safe ownership | Passing guest closures across isolated channels | Proposed `atto-lang/rx-ores-callbacks` (not transferred) |
| `rx-ores-tasks` | Reactive | Reactive task coordination adapters | Kernel scheduling policy | Proposed `atto-lang/rx-ores-tasks` (not transferred) |
| `rx-ores` | Reactive | Reactive API contracts and operators | Runtime scheduler implementation | Proposed `atto-lang/rx-ores` (not transferred) |
| `rx-oreslang-channels` | Reactive | Reactive channel interop with cancellation and backpressure | Actor mailbox internals | Proposed `atto-lang/rx-oreslang-channels` (not transferred) |
| `sublime-text-plugins` | Tooling | Sublime Text language integration | Compiler code generation | Proposed `atto-lang/sublime-text-plugins` (not transferred) |
| `vscode-plugins` | Tooling | VS Code language integration | Runtime or cloud services | Proposed `atto-lang/vscode-plugins` (not transferred) |

## Maintainer migration checklist (one repository at a time)

1. Establish a named **source owner, target owner, independent security reviewer and rollback owner**. Announce exact repository and work window in [coordination issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26). Freeze conflicting edits only after agreement.
2. Inventory **numeric GitHub repository ID, visibility, default branch SHA and Git tree, PRs/issues, tags/releases, Actions, packages, webhooks, deploy keys, CODEOWNERS, Pages and submodules**. Retain a private receipt for private repos.
3. Verify destination org rights and billing, repository name availability, branch/ruleset protection, required reviewers, CI runner admission, secrets/environments and app installations. A 403 or `protected=false` is an unresolved blocker, not successful verification.
4. Resolve collisions with already-created `atto-lang/atto-*` / `atto-co/atto-*` repos as **distinct identities**. Never overwrite, assume a mirror, or transfer a private repository merely because the destination has a similar name. Proposed first transfers retain the old repository name.
5. Obtain explicit **source/target admin and independent security** approvals. No private source copying, cross-org token checkout or transfer before those gates. Use the [PR #25 transfer runbook](https://github.com/ores-truffle-oreslang/.github/pull/25/files) and its receipt template when approved.
6. An authorized owner performs an actual **GitHub repository ownership transfer**, not a shallow clone or unrelated fork. Validate the *same repository numeric ID*, source/target head SHA and Git tree, releases and PR histories after transfer.
7. Restore/check target branch rulesets, required reviewers, packages, App permissions, deployment environments, secrets, registries, webhooks, Pages and remote URLs; preserve external compatibility with the old repository URL/CLI/package IDs where feasible.
8. Run **exact-head CI that actually executes nonzero steps** at the target. Distinguish host/static checks from real guest compilation/execution and negative tests. Zero-step / `steps=null` is `NOT_EXECUTED`, never a pass.
9. Update consumer imports, workflows, docs and badges **after** the transfer succeeds. Keep `.ores` and old `oreslang-*` interfaces working where supported; new `.atto` syntax/ABI support must be proven separately per backend. Do not combine source-file renames and repository transfers.
10. Record per-repo receipts and final status. Six holds each require an explicit owner-approved **transfer, archive, or time-bounded compatibility exception**; never declare the old organization emptied while its dependencies remain.

## Boundary and implementation policy

Portable core/std and shared language semantics go into canonical `.atto` modules, with Java/C#/C++/Rust host code limited to indispensable compiler, ABI, memory and scheduler mechanisms. AOT / zero-GC / actor isolation and cross-backend parity are engineering goals, not capabilities conferred by a README. Compiler backends are separate implementation authorities and need pinned source tests. Cloud identity/billing, hosted runtime control, deployments and tenant operations belong to `atto-co`, not language compilers or generic library/demos. Production server claims require verified networking/security tests, not control-plane-only demo fixtures.

## Review gates for these README edits

This guide is documentation only. It does not grant migration permission, rewrite a transfer manifest, make an unprotected branch safe, or certify tests. A later manifest change must reconcile the current 80-ID inventory, named approvals and executed receipts before any repository move.
