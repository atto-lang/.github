# AttoLang compiler × feature capability matrix

> **Status:** Architecture targets and ownership mapping — **NOT** a claim of implemented or certified parity.  
> **Updated:** 2026-11-19 (PR- and conversation-audited architecture target; sampled evidence, not exact-head certification)  
> **Scope:** 11 `atto-source.*` backends, actor runtime, tenant isolation, permissions, compilation, and real-time capabilities.  
> **Normative sources:** [atto-spec](https://github.com/atto-lang/atto-spec), [atto-target-profiles](https://github.com/atto-lang/atto-target-profiles), [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance).  
> **Related policy:** [Architecture orientation](../ARCHITECTURE.md); [AOT / zero-GC proposal](https://github.com/atto-lang/.github/pull/1).

## Meaning of matrix entries

| Mark | Meaning |
| --- | --- |
| **P** | Primary *target* / implementation owner; not an assertion that it exists or passes. |
| **S** | Secondary or planned supported target; requires implementation and proof. |
| **H** | Delegated to host/runtime or another compiler; not an independent implementation. |
| **C** | Constrained or target-specific subset; unsupported features must fail closed. |
| **G** | Security-critical goal requiring adversarial execution evidence before any guarantee. |
| **—** | Outside this backend's intended supported profile. |

**Critical naming distinction:** `atto-source.rs` implements a Rust-authored compiler that emits Wasm and can itself be built for wasm32; `atto-source.wasm` owns a separate Wasm integration, binary-admission and worker execution path. Do **not** count them as two independently proven, general-purpose source compilers.

## 0. Audited current implementation evidence (November 19, 2026)

**This section is deliberately separate from the target grids below.** It is a targeted PR and source audit, not a universal exact-head test certification. A PR being merged means its code entered its canonical repository; it does **not** establish complete feature support. Private CI receipts described by PRs are not independently re-certified by this document. Treat unverified capabilities as **NOT_EXECUTED** or **UNSUPPORTED** until independently tested.

| Backend | Implemented / evidenced narrow slice | Not established | Representative sources |
| --- | --- | --- | --- |
| **`atto-source.llvm`** | Bounded native Atto AOT, FFI and compact Offset32/Ref32 actor storage; separate process-worker prototype | Not general in-process hostile actor crash containment; pthread SIGSEGV may terminate process; hard RT unproved | [1](https://github.com/atto-lang/atto-source.llvm/pull/21), [2](https://github.com/atto-lang/atto-source.llvm/pull/12), [3](https://github.com/atto-lang/atto-source.llvm/pull/47) |
| **`atto-source.wasm`** | Binary validation, typed ABI checking, Node Worker protocols, host ActorRef/ActorGroup registry | Host/runtime evidence is not a second independent full source compiler or portable guest ActorGroup implementation | [1](https://github.com/atto-lang/atto-source.wasm/pull/8), [2](https://github.com/atto-lang/atto-source.wasm/pull/21), [3](https://github.com/atto-lang/atto-source.wasm/pull/35) |
| **`atto-source.rs`** | Rust-authored source-to-Wasm compiler, executed bounded browser guests and actor-owned JSPI suspending receive subset | Not full language parity, general native scheduler or independently bounded actor stacks; release/receipt gates remain | [1](https://github.com/atto-lang/atto-source.rs/pull/52), [2](https://github.com/atto-lang/atto-source.rs/pull/384), [3](https://github.com/atto-lang/atto-source.rs/pull/23) |
| **`atto-source.java`** | Restricted JVM and Graal Native Image guest compilation, arena Ref32 and checked structural/async subsets | No proved arbitrary untrusted guest isolation, full portability, host zero-GC or certified RT | [1](https://github.com/atto-lang/atto-source.java/pull/29), [2](https://github.com/atto-lang/atto-source.java/pull/30), [3](https://github.com/atto-lang/atto-source.java/pull/38) |
| **`atto-source.net`** | Direct native Atto-to-CIL signed-i64 compiler and real CoreCLR/NativeAOT exit-42 guest; exact-head private carrier receipt | Wasmtime-backed hostile guest path remains experimental and deny-by-default, especially under Windows CET; no actor frontend | [1](https://github.com/atto-lang/atto-source.net/pull/9), [2](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/563) |
| **`atto-source.js`** | Constrained checked source-to-JS/ESM scalar, actor and structural compiler slices; Worker host primitives | No secure hostile JavaScript TenantVM, general borrowing/compiler parity, or production hot loading | [1](https://github.com/atto-lang/atto-source.js/pull/29), [2](https://github.com/atto-lang/atto-source.js/pull/6) |
| **`atto-source.forge`** | Native bounded scalar compiler via deterministic C++ emission and checked-i64 native guest tests | No general native actor compiler, custom TenantVM or working general JIT despite architecture targets | [1](https://github.com/atto-lang/atto-source.forge/pull/15), [2](https://github.com/atto-lang/atto-source.forge/pull/25) |
| **`atto-source.cranelift`** | Stage-0 trusted signed-i64 native JIT and executable scalar fixtures; fail-closed admission guards | No general AOT, zero-GC actor runtime, actor compilation or verified native sandbox | [1](https://github.com/atto-lang/atto-source.cranelift/pull/13), [2](https://github.com/atto-lang/atto-source.cranelift/pull/32) |
| **`atto-source.atto`** | Draft Stage-0 Atto-authored scalar seed compiled/executed with a pinned external LLVM compiler | NOT SELF-HOSTED; no Stage 1/2/3 compiler rebuild or independent full language compilation | [1](https://github.com/atto-lang/atto-source.atto/pull/1) |
| **`atto-source.hdl`** | Experimental target definition and repository setup for HDL/synthesizable subset | No operational HDL compiler, FPGA/ASIC timing signoff or verified hardware hard real-time | [1](https://github.com/atto-lang/atto-source.hdl) |
| **`atto-source.realtime`** | Planned target-profile/WCET/scheduling admission analysis repository | BOOTSTRAP ONLY: no operating compiler, target scheduler or hard-real-time proof | [1](https://github.com/atto-lang/atto-source.realtime) |

**Cross-backend security/real-time status:** NO row here claims certified same-process hostile native-actor SIGSEGV containment, broad actor graph/ownership parity, universal hot-loading, or general hard real-time. Bounded native/Wasm guest execution is not equivalent to those guarantees.

## 1. Compiler and execution capabilities

| Compiler | Native AOT | Wasm output | Wasm admission / host | JS output | JIT | AOT/JIT instrumentation | Hot loading | Browser | Node/Bun | JVM |  .NET |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | S | — | — | S | P | S | H | H | — | — |
| **`atto-source.wasm`** | — | C (LLVM bridge) | P | — | H | S | S | P | P | — | — |
| **`atto-source.rs`** | — | P | H | — | H | S | S | P | P | — | — |
| **`atto-source.java`** | P (Graal) | — | — | — | H (JVM only) | P | S | — | — | P | — |
| **`atto-source.net`** | P (NativeAOT) | — | P (Wasmtime goal) | — | H (CoreCLR only) | P | S | — | — | — | P |
| **`atto-source.js`** | — | — | — | P | H | S | S | P | P | — | — |
| **`atto-source.forge`** | P | S | H | — | P | P | S | — | S | — | — |
| **`atto-source.cranelift`** | S | — | — | — | P (scalar JIT) | P | S | — | S | — | — |
| **`atto-source.atto`** | H | H | H | H | H | S | S | H | H | H | H |
| **`atto-source.hdl`** | — | — | — | — | — | C | — | — | — | — | — |
| **`atto-source.realtime`** | H | — | — | — | — | P | — | — | — | S | S |

Wasm *emission* and Wasm *admission/hosting* are distinct responsibilities: `atto-source.rs` is the Rust-written source-to-Wasm compiler; `atto-source.wasm` is an execution/integration host with a constrained LLVM bridge. Node/Bun are intended platform categories; Bun compatibility is not certified here. Graal Native Image and .NET NativeAOT have no production JIT inside the resulting AOT binary; their JVM/CoreCLR developer hosts may JIT. All dynamic instrumentation/hot-loading cells are targets, not evidence of arbitrary executable patching.

## 2. Security, sandboxing and tenant isolation

| Compiler | Custom TenantVM | Graal isolates | Wasm guest isolation | Deno flags | Socket ACLs | Deny forbidden I/O | Tenant quotas | Memory sandbox | Actor fault containment |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | — | S | P | P | P | P | G | G |
| **`atto-source.wasm`** | H | — | P | P | H | P | P | H | S |
| **`atto-source.rs`** | H | — | H | P | H | H | S | H | H |
| **`atto-source.java`** | H | P | — | P | P | P | P | G | G |
| **`atto-source.net`** | H | — | P (not production) | P | P | P | P | G | G |
| **`atto-source.js`** | — | — | — | S | H | C (trusted JS only) | S | — | — |
| **`atto-source.forge`** | P | — | S | P | P | P | P | G | G |
| **`atto-source.cranelift`** | H (shared TenantVM) | — | S | P | S | P | P | G | G |
| **`atto-source.atto`** | H | H | H | P | H | H | H | H | H |
| **`atto-source.hdl`** | — | — | — | C | — | P | S | C | C |
| **`atto-source.realtime`** | H | — | — | S | S | S | P | H | H |

**Security model:** the central ambition is multiple mutually untrusted tenants in one OS process. Every socket capability is brokered per tenant, independently authorized for `read`/`write`/`accept`/`connect`/`bind`, revocable, quota-bound and generation-checked. Deny filesystem, subprocess, environment and arbitrary foreign-code I/O unless the policy explicitly permits access. Limit permitted I/O to explicitly brokered network activity for the untrusted profile.

**Never equate:** worker threads with secure JavaScript sandboxing; separate actor arenas with arbitrary native memory fault containment; Native Image/NativeAOT with zero GC; or host/signal handler recovery with reliable cross-tenant isolation. An unverified native backend must use stronger containment (such as an OS process) for hostile code.

**Required strict untrusted-tenant I/O profile:** no direct guest filesystem, environment, subprocess, FFI, raw descriptors or arbitrary OS syscalls. Permit *only* brokered network I/O through tenant-bound, revocable, generation-checked socket handles with independent `connect`, `bind`, `accept`, `read`, `write` and `close` rights and per-operation quotas. Trusted supervisor/host services may have separately scoped resources. Use the prescribed [flags-2-env](https://github.com/flags-2-env) CLI/env mapping where applicable; the flags are not themselves sandbox enforcement.

**Host boundary:** JVM/Graal isolates are not a blanket hostile native memory sandbox. .NET's [draft Wasmtime Store isolation plan](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/563) requires build-time precompiled authenticated guest artifacts, runtime compiler-free admission, per-Store quotas and platform-specific CET safety; production admission remains DENIED. Workers are scheduling tools, not secure sandboxes for arbitrary JavaScript.

**Crash containment correction:** actor-owned arenas/private stacks alone cannot make native pthread `SIGSEGV` survivable. The merged [LLVM process-worker prototype #12](https://github.com/atto-lang/atto-source.llvm/pull/12) offers OS-process fault containment only, and its [draft #47](https://github.com/atto-lang/atto-source.llvm/pull/47) is explicit about absent native compiled-guest integration. Same-process hostile-code admission requires a *verified* memory/control-flow-safe execution domain and adversarial tests, or must fall back to a protected OS process.

## 3. Actor memory and runtime

| Compiler | Actor heaps | 32-bit offsets | Zero guest GC | Minimal guest GC | Actor stacks | Deterministic reclamation | Scheduler | Supervisors | ActorRegistry | ActorGroups | Actor graphs |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | P | P | — | P | P | P | P | P | P | P |
| **`atto-source.wasm`** | P | P | P | — | S/H (JSPI frames) | P | P | P | P | P | S |
| **`atto-source.rs`** | H | H | H | — | H | H | H | H | H | H | H |
| **`atto-source.java`** | P | P | — | P | P | S | P | P | P | P | P |
| **`atto-source.net`** | P | P | — | P | P | S | P | P | P | P | P |
| **`atto-source.js`** | S | C (compressed32) | — | H | H (Worker) | S | P | S | S | S | S |
| **`atto-source.forge`** | P | P | P | — | P | P | P | P | P | P | P |
| **`atto-source.cranelift`** | S | S | S | — | S | S | S | S | S | S | S |
| **`atto-source.atto`** | H | H | H | H | H | H | H | H | H | H | H |
| **`atto-source.hdl`** | C | C | — | — | C (static) | P | C | — | — | C | C |
| **`atto-source.realtime`** | H | S | P (profile) | — | P | P | P | H | H | H | H |

"Zero guest GC" means deterministic or explicitly controlled reclamation of **guest actor-owned allocations**, not elimination of all host-runtime GC. Actor stack implementations may be native stacks, guarded stack segments or compiler-managed continuation frames. No host process survival from a guest stack overflow or native segmentation fault is guaranteed until specifically proven.

**Actor-heap/stack caveats:** a four-byte Offset32 reference is not a generational capability: issuer, heap identity and reuse epochs are separate checks (see [LLVM #21](https://github.com/atto-lang/atto-source.llvm/pull/21) and [#42](https://github.com/atto-lang/atto-source.llvm/pull/42)). Zero guest-GC means actor-owned guest allocations avoid tracing collection; it does not mean no host JVM/CLR/V8 GC. JSPI uses host-managed continuations, not independently proven actor-owned guarded native stacks.

**Not yet compiled cross-backend behavior:** ActorRegistry typed envelope and admission receipts ([spec draft #113](https://github.com/atto-lang/atto-spec/pull/113)), capability-scoped graph self-organization/atomic child creation and telemetry sidecars ([spec draft #112](https://github.com/atto-lang/atto-spec/pull/112)), and shared/isolated/TenantVM carrier-pool separation ([spec draft #49](https://github.com/atto-lang/atto-spec/pull/49)). The [Wasm ActorGroups host work #21](https://github.com/atto-lang/atto-source.wasm/pull/21) is not evidence of compiled actor graphs on every backend. Desired tenant-scoped Workers hosting multiple actors should not be conflated with current per-actor Workers in browser tests.

## 4. Language, real-time and hardware profiles

**Soft real-time and hard real-time are deliberately separate columns and separate contracts.**

| Compiler | Ownership / borrowing | Sealed actor state | Panic / raise / throw | Cooperative preemption | Soft real-time | Hard real-time | Embedded | FPGA / ASIC | Self-hosting | Conformance |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | P | P | P | P | P (bounded target) | P | — | — | P |
| **`atto-source.wasm`** | P | P | P | P | C (best effort) | — | C | — | — | P |
| **`atto-source.rs`** | P | P | P | S | C (best effort) | — | — | — | — | P |
| **`atto-source.java`** | P | P | P | P | P (goal) | — | — | — | — | P |
| **`atto-source.net`** | P | P | P | P | P (goal) | — | S | — | — | P |
| **`atto-source.js`** | S | S | S | S | — | — | — | — | — | P |
| **`atto-source.forge`** | P | P | P | P | P | S (bounded target) | S | — | — | P |
| **`atto-source.cranelift`** | P | P | S | P | S | — | — | — | — | P |
| **`atto-source.atto`** | P | P | P | H | H | H | H | — | P | P |
| **`atto-source.hdl`** | C | C | C | C | C | P (qualified hardware) | P | P | — | P |
| **`atto-source.realtime`** | P | P | P | P | P (profile) | P (profile) | P | S | — | P |

### Real-time definitions

| Class | Minimum requirement | Primary target interpretation |
| --- | --- | --- |
| **Soft real-time** | Bounded *typical* latency and operational SLOs; isolated GC, scheduler and OS jitter may exist. Missed deadlines degrade service rather than violate a hard safety promise. | JVM/Graal and .NET are primary soft-RT environments; native and bounded Wasm can also target it. |
| **Hard real-time** | Demonstrated upper bounds on execution, memory allocation, scheduler response and I/O latency for an explicitly admitted program/hardware target; fail closed if the proof cannot be established. | LLVM/realtime and qualified synthesizable HDL profiles; Forge only if a particular target demonstrates comparable bounds. |

**Hard real-time is not an automatic LLVM, zero-GC, FPGA or bare-metal property. No Atto backend currently has a certified hard-real-time profile in this document.** It applies only to a specified target, admitted subset, WCET/scheduling evidence, bounded calls/allocations, interrupt and I/O behavior, and defined failure mode. No promise is made for unrestricted programs. Soft real-time is a latency objective rather than a mathematical deadline guarantee.

## 5. Primary backend responsibilities

| Backend | Main responsibility | Runtime / isolation approach | Memory model |
| --- | --- | --- | --- |
| [`atto-source.llvm`](https://github.com/atto-lang/atto-source.llvm) | Main high-performance native compiler | Custom TenantVM target (not yet hostile-native-sandbox certified); runtime-owned capabilities | Zero-GC actor arenas, checked 32-bit offsets and bounded private stacks |
| [`atto-source.wasm`](https://github.com/atto-lang/atto-source.wasm) | Wasm guest backend, binary admission and worker host | Wasm validation/linear-memory isolation; workers for scheduling | Bounded guest linear memory and actor frames |
| [`atto-source.rs`](https://github.com/atto-lang/atto-source.rs) | Rust-authored compiler producing guest Wasm; browser/Node compiler | Delegates guest isolation to Wasm execution host | Compiler memory managed by Rust/host; emitted guest memory via Wasm |
| [`atto-source.java`](https://github.com/atto-lang/atto-source.java) | JVM/Graal compiler and Graal Native Image | Graal isolate mechanisms plus controlled host entrypoints | Arena-backed actor heaps with 32-bit offsets; minimize guest-data GC |
| [`atto-source.net`](https://github.com/atto-lang/atto-source.net) | .NET CIL and NativeAOT, Windows/Linux | Planned Wasmtime guest stores (production admission denied); managed host capability broker | Arena-backed actor heaps with bounded guest lifetimes; minimize guest-data GC |
| [`atto-source.js`](https://github.com/atto-lang/atto-source.js) | Atto-to-JavaScript transpilation for trusted runtime environments | Workers for trusted host concurrency only; no hostile guest JS security boundary | Host GC and logical actor memory |
| [`atto-source.forge`](https://github.com/atto-lang/atto-source.forge) | Independent native JIT and AOT engine | Custom TenantVM target; native isolation contract not yet certified | Zero-GC actor heaps and bounded private stacks |
| [`atto-source.cranelift`](https://github.com/atto-lang/atto-source.cranelift) | Low-latency Rust/Cranelift codegen and scalar JIT | Future shared TenantVM contract; current scalar JIT is trusted, not an untrusted sandbox | Native actor arenas as target; verifier ahead of codegen |
| [`atto-source.atto`](https://github.com/atto-lang/atto-source.atto) | Future self-hosted compiler | Stage-0 seed compiled by a pinned external LLVM compiler (not self-hosted) | Inherits the selected backend's runtime |
| [`atto-source.hdl`](https://github.com/atto-lang/atto-source.hdl) | Hardware-description/synthesizable subset | Hardware-target isolation to be defined with explicit external I/O constraints | Static, bounded buffers and hardware resources |
| [`atto-source.realtime`](https://github.com/atto-lang/atto-source.realtime) | Timing-aware admission/compilation profiles | Host/native target-specific isolation contract | Bounded memory, stack, scheduling and worst-case timing profiles |

## 6. Reverse map — feature → compiler owners

| Feature | Intended owners / participants |
| --- | --- |
| **Native executable AOT** | llvm, java (Graal Native Image), net (NativeAOT), forge as targets; direct-CIL .net has a narrow executed NativeAOT guest; Cranelift currently has scalar JIT, not full native AOT |
| **Wasm guest compiler/host** | rs emits guest Wasm; wasm owns Wasm runtime/binary admission; llvm can target Wasm |
| **Native custom TenantVM** | llvm and forge primary design targets; Cranelift should reuse their security contract, not invent an independent sandbox; none certified for malicious in-process native guest execution |
| **Graal isolate-backed tenancy** | java |
| **Wasm-backed .NET tenancy** | net (host) + Wasmtime Stores as a planned, denied-production, strict-AOT guest sandbox |
| **Deno-style permission enforcement** | llvm, wasm, rs, java, net, forge, cranelift, atto; JS constrained and HDL specialized |
| **Per-socket read/write capabilities** | llvm, forge, java, net primary; wasm/rs delegated to host |
| **Zero guest GC** | llvm, wasm, forge, Cranelift as intended guest-actor allocation goals, not blanket proven support; realtime imposes bounded critical-path constraints; static HDL hardware buffers are not a GC runtime |
| **Minimal guest-data GC** | java and net; not a claim about host VM GC |
| **Actor-local heaps and stacks** | llvm, forge, java, net, wasm; others delegate or provide constrained forms |
| **Soft real-time** | llvm, java, net, forge, realtime target profiles; wasm/rs/cranelift best effort under constraints; HDL target-specific, none universally certified |
| **Hard real-time** | llvm plus realtime restricted target profiles, hardware HDL subsets; Forge conditional on future bounded profile; **no qualified real-time evidence yet** |
| **Self-hosting** | Atto-authored Stage-0 kernel executed using a pinned external LLVM compiler; Stage 1/2/3 rebuilding NOT_IMPLEMENTED |
| **AOT-compatible telemetry instrumentation** | llvm, java, net, forge, realtime primary; others platform specific |

The reverse map expresses architecture responsibility and does not grant independent readiness or deployment authority. See the audited evidence matrix above before making any feature-support claim.

## 7. Portable contracts and required verification gates

| Requirement | Acceptance condition |
| --- | --- |
| **Unified language semantics** | Same observable behavior for supported portable programs, including ownership, actors, exceptions and wire formats. |
| **Secure tenant admission** | Signed/pinned or otherwise authenticated compiled guest artifact; verifier and sealing before execution; reject unsupported features. |
| **TenantVM separation** | Check heap bounds, control flow, imports, pointer provenance, generation identity and capability handles across domains. |
| **Actor fault containment** | Crash/stack-overflow tests must show affected actor teardown and unaffected adjacent tenants; native SIGSEGV handlers alone do not suffice. |
| **Per-actor memory** | Actor-owned arenas and bounded stacks/continuation frames; overflow-safe 32-bit offsets and deterministic cleanup. |
| **AOT/JIT coherence** | JIT may optimize but must preserve AOT semantics; shared event/probe ABI and stable safepoint/patchpoint contracts. |
| **Dynamic instrumentation** | Pre-emitted AOT probes with runtime enable masks; dynamic JIT recompilation optional and capability-gated. |
| **Permission flags** | Deno-like deny-by-default permissions, including filesystem, environment, network, subprocess, FFI and imports; explicit flags-2-env mapping where applicable. |
| **Socket capabilities** | Broker handles per tenant with independent read/write/accept/bind/connect rights, quotas, generation fencing and revocation; never expose raw OS descriptors. |
| **Actor organization** | Proposed ActorRegistry, ActorGroups, supervisor CONTROL, typed job admission, atomic children, dynamic graphs, mailbox lifecycle and telemetry sidecar semantics. Candidate [spec #112](https://github.com/atto-lang/atto-spec/pull/112) and [#113](https://github.com/atto-lang/atto-spec/pull/113) are not portable guest support. |
| **Safe ownership** | borrow!, take!, share!, copy!, copy!!, pin!, drop! must preserve moves/loans across actor send, suspension, cancellation and error exit. |
| **Proof receipts** | Exact source/tree and toolchain SHA; compiled positive/negative guest cases, targeted platform, nonzero CI steps and explicit PASSED/FAILED/UNSUPPORTED/NOT_EXECUTED. |

### Fault-containment requirements

A tenant is not adequately isolated solely because its heap is a different region or it has its own stack. Validation, executable memory controls, bounds checks, non-forgeable capabilities, CFI, strict host-call admission, allocation/quota and watchdog enforcement, teardown, and side-channel threat modeling all matter. Treat speculative execution and microarchitectural side channels separately; software sandboxing does not automatically eliminate them.

For native targets, `SIGSEGV` or stack-overflow catching must **not** be represented as a sufficient per-actor reliability or security guarantee. A failure may have corrupted shared process state before the signal. Only an appropriately constrained/verifiable guest execution model, backed by hostile-guest tests, can make a meaningful actor-fault-containment claim. Unknown native code must run in a stronger isolation domain.

### AOT and JIT instrumentation design

1. The compiler defines a stable typed instrumentation IR and event ABI, independent of target backend.
2. AOT emits dormant probes, safepoints or patchpoints that can be enabled by runtime configuration without source recompilation.
3. JIT may insert additional profiling and optimized probes but must preserve the same observable semantics and tracing identities.
4. Instrumentation is capability-scoped, tenant-aware, budgeted and non-bypassing of memory/ownership checks.
5. Hot upgrades preserve active actor identities, generation fences and stack/lifetime safety; unsupported update paths fail closed.

### Evidence notation — keep separate from this design matrix

Use **PASSED**, **FAILED**, **UNSUPPORTED**, or **NOT_EXECUTED**, tied to an exact backend commit/tree SHA, target toolchain, compiled artifact hash, **real guest** positive/negative execution, nonzero executed CI steps, and captured logs. Host unit tests, documentation, static parsing, unrun workflows, `steps=[]`, or CI runs on a different commit do not certify any target capability.

Backend implementation maturity differs substantially today: the Rust-to-Wasm compiler has bounded executable guest support; the Wasm repository is primarily an integration/execution host; Cranelift includes stage-0 scalar JIT work; Forge has native host primitives; the self-hosted compiler is not self-hosted; and the realtime repository is bootstrap-only. Revalidate every claim against current code and exact-head execution evidence.

## 8. Maintenance and change ownership

- Update this **architectural** matrix when the intended compiler/feature ownership changes.
- Update [atto-spec](https://github.com/atto-lang/atto-spec) for normative language-semantic changes, and [atto-target-profiles](https://github.com/atto-lang/atto-target-profiles) for formal target capabilities.
- Publish independent backend proof receipts in [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance) and [atto-compilers-parity-testing](https://github.com/atto-lang/atto-compilers-parity-testing).
- Never silently promote an architectural `P`, `S`, `H`, `C`, or `G` to an implemented guarantee. Every support claim must have separately verified evidence.

