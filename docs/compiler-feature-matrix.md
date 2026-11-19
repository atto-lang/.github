# AttoLang compiler × feature capability matrix

> **Status:** Architecture targets and ownership mapping — **NOT** a claim of implemented or certified parity.  
> **Updated:** 2026-11-19  
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

## 1. Compiler and execution capabilities

| Compiler | Native AOT | Wasm | JS output | JIT | AOT/JIT instrumentation | Hot loading | Browser | Node/Bun | JVM |  .NET |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | S | — | S | P | S | H | H | — | — |
| **`atto-source.wasm`** | — | P | — | H | S | S | P | P | — | — |
| **`atto-source.rs`** | — | P | — | H | S | S | P | P | — | — |
| **`atto-source.java`** | P (Graal) | — | — | H | P | S | — | — | P | — |
| **`atto-source.net`** | P (NativeAOT) | H | — | H | P | S | — | — | — | P |
| **`atto-source.js`** | — | — | P | H | S | P | P | P | — | — |
| **`atto-source.forge`** | P | S | — | P | P | P | — | S | — | — |
| **`atto-source.cranelift`** | S | — | — | P | P | S | — | S | — | — |
| **`atto-source.atto`** | H | H | H | H | S | S | H | H | H | H |
| **`atto-source.hdl`** | — | — | — | — | C | — | — | — | — | — |
| **`atto-source.realtime`** | H | — | — | — | P | — | — | — | S | S |

Native AOT includes Graal Native Image and .NET NativeAOT. Wasm output is separate from native machine-code AOT. Host JIT services are not the same as Atto-controlled JIT compilation. Hot loading must not override sealing, cap fences or proof provenance.

## 2. Security, sandboxing and tenant isolation

| Compiler | Custom TenantVM | Graal isolates | Wasm guest isolation | Deno flags | Socket ACLs | Deny forbidden I/O | Tenant quotas | Memory sandbox | Actor fault containment |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | — | S | P | P | P | P | G | G |
| **`atto-source.wasm`** | H | — | P | P | H | P | P | H | S |
| **`atto-source.rs`** | H | — | H | P | H | H | S | H | H |
| **`atto-source.java`** | H | P | — | P | P | P | P | G | G |
| **`atto-source.net`** | H | — | P | P | P | P | P | G | G |
| **`atto-source.js`** | — | — | — | S | H | S | S | — | — |
| **`atto-source.forge`** | P | — | S | P | P | P | P | G | G |
| **`atto-source.cranelift`** | S | — | S | P | S | P | P | G | G |
| **`atto-source.atto`** | H | H | H | P | H | H | H | H | H |
| **`atto-source.hdl`** | — | — | — | C | — | P | S | C | C |
| **`atto-source.realtime`** | H | — | — | S | S | S | P | H | H |

**Security model:** the central ambition is multiple mutually untrusted tenants in one OS process. Every socket capability is brokered per tenant, independently authorized for `read`/`write`/`accept`/`connect`/`bind`, revocable, quota-bound and generation-checked. Deny filesystem, subprocess, environment and arbitrary foreign-code I/O unless the policy explicitly permits access. Limit permitted I/O to explicitly brokered network activity for the untrusted profile.

**Never equate:** worker threads with secure JavaScript sandboxing; separate actor arenas with arbitrary native memory fault containment; Native Image/NativeAOT with zero GC; or host/signal handler recovery with reliable cross-tenant isolation. An unverified native backend must use stronger containment (such as an OS process) for hostile code.

## 3. Actor memory and runtime

| Compiler | Actor heaps | 32-bit offsets | Zero guest GC | Minimal guest GC | Actor stacks | Deterministic reclamation | Scheduler | Supervisors | ActorRegistry | ActorGroups | Actor graphs |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | P | P | — | P | P | P | P | P | P | P |
| **`atto-source.wasm`** | P | P | P | — | P | P | P | P | P | P | P |
| **`atto-source.rs`** | H | H | H | — | H | H | H | H | H | H | H |
| **`atto-source.java`** | P | P | — | P | P | S | P | P | P | P | P |
| **`atto-source.net`** | P | P | — | P | P | S | P | P | P | P | P |
| **`atto-source.js`** | S | C | — | H | H | S | P | S | S | S | S |
| **`atto-source.forge`** | P | P | P | — | P | P | P | P | P | P | P |
| **`atto-source.cranelift`** | S | S | P | — | S | S | S | S | S | S | S |
| **`atto-source.atto`** | H | H | H | H | H | H | H | H | H | H | H |
| **`atto-source.hdl`** | C | C | C | — | C | P | C | — | — | C | C |
| **`atto-source.realtime`** | H | S | P | — | P | P | P | H | H | H | H |

"Zero guest GC" means deterministic or explicitly controlled reclamation of **guest actor-owned allocations**, not elimination of all host-runtime GC. Actor stack implementations may be native stacks, guarded stack segments or compiler-managed continuation frames. No host process survival from a guest stack overflow or native segmentation fault is guaranteed until specifically proven.

## 4. Language, real-time and hardware profiles

**Soft real-time and hard real-time are deliberately separate columns and separate contracts.**

| Compiler | Ownership / borrowing | Sealed actor state | Panic / raise / throw | Cooperative preemption | Soft real-time | Hard real-time | Embedded | FPGA / ASIC | Self-hosting | Conformance |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **`atto-source.llvm`** | P | P | P | P | P | P (bounded profiles) | P | — | — | P |
| **`atto-source.wasm`** | P | P | P | P | C | — | C | — | — | P |
| **`atto-source.rs`** | P | P | P | S | C | — | — | — | — | P |
| **`atto-source.java`** | P | P | P | P | P | — | — | — | — | P |
| **`atto-source.net`** | P | P | P | P | P | — | S | — | — | P |
| **`atto-source.js`** | S | S | S | S | — | — | — | — | — | P |
| **`atto-source.forge`** | P | P | P | P | P | S (bounded profiles) | S | — | — | P |
| **`atto-source.cranelift`** | P | P | S | P | S | — | — | — | — | P |
| **`atto-source.atto`** | P | P | P | H | H | H | H | — | P | P |
| **`atto-source.hdl`** | C | C | C | C | P | P (verified hardware) | P | P | — | P |
| **`atto-source.realtime`** | P | P | P | P | P | P | P | S | — | P |

### Real-time definitions

| Class | Minimum requirement | Primary target interpretation |
| --- | --- | --- |
| **Soft real-time** | Bounded *typical* latency and operational SLOs; isolated GC, scheduler and OS jitter may exist. Missed deadlines degrade service rather than violate a hard safety promise. | JVM/Graal and .NET are primary soft-RT environments; native and bounded Wasm can also target it. |
| **Hard real-time** | Demonstrated upper bounds on execution, memory allocation, scheduler response and I/O latency for an explicitly admitted program/hardware target; fail closed if the proof cannot be established. | LLVM/realtime and qualified synthesizable HDL profiles; Forge only if a particular target demonstrates comparable bounds. |

**Hard real-time is not an automatic LLVM, zero-GC, FPGA or bare-metal property.** It applies only to a specified target, admitted subset, WCET/scheduling evidence, bounded calls/allocations, interrupt and I/O behavior, and defined failure mode. No promise is made for unrestricted programs. Soft real-time is a latency objective rather than a mathematical deadline guarantee.

## 5. Primary backend responsibilities

| Backend | Main responsibility | Runtime / isolation approach | Memory model |
| --- | --- | --- | --- |
| [`atto-source.llvm`](https://github.com/atto-lang/atto-source.llvm) | Main high-performance native compiler | Custom validated TenantVM, runtime-owned capabilities | Zero-GC actor arenas, checked 32-bit offsets and bounded private stacks |
| [`atto-source.wasm`](https://github.com/atto-lang/atto-source.wasm) | Wasm guest backend, binary admission and worker host | Wasm validation/linear-memory isolation; workers for scheduling | Bounded guest linear memory and actor frames |
| [`atto-source.rs`](https://github.com/atto-lang/atto-source.rs) | Rust-authored compiler producing guest Wasm; browser/Node compiler | Delegates guest isolation to Wasm execution host | Compiler memory managed by Rust/host; emitted guest memory via Wasm |
| [`atto-source.java`](https://github.com/atto-lang/atto-source.java) | JVM/Graal compiler and Graal Native Image | Graal isolate mechanisms plus controlled host entrypoints | Arena-backed actor heaps with 32-bit offsets; minimize guest-data GC |
| [`atto-source.net`](https://github.com/atto-lang/atto-source.net) | .NET CIL and NativeAOT, Windows/Linux | Wasm guests for untrusted tenants; managed host capability broker | Arena-backed actor heaps with bounded guest lifetimes; minimize guest-data GC |
| [`atto-source.js`](https://github.com/atto-lang/atto-source.js) | Atto-to-JavaScript transpilation for trusted runtime environments | Node/browser/Bun host isolation only; no arbitrary hostile guest JS tenancy | Host GC and logical actor memory |
| [`atto-source.forge`](https://github.com/atto-lang/atto-source.forge) | Independent native JIT and AOT engine | Custom validated TenantVM sharing the native isolation contract | Zero-GC actor heaps and bounded private stacks |
| [`atto-source.cranelift`](https://github.com/atto-lang/atto-source.cranelift) | Low-latency Rust/Cranelift codegen and scalar JIT | Reuse validated TenantVM; scalar trusted JIT is not an untrusted sandbox | Native actor arenas as target; verifier ahead of codegen |
| [`atto-source.atto`](https://github.com/atto-lang/atto-source.atto) | Future self-hosted compiler | Bootstrapped through an already-certified backend | Inherits the selected backend's runtime |
| [`atto-source.hdl`](https://github.com/atto-lang/atto-source.hdl) | Hardware-description/synthesizable subset | Physical target isolation and explicit external I/O constraints | Static, bounded buffers and hardware resources |
| [`atto-source.realtime`](https://github.com/atto-lang/atto-source.realtime) | Timing-aware admission/compilation profiles | Host/native target-specific isolation contract | Bounded memory, stack, scheduling and worst-case timing profiles |

## 6. Reverse map — feature → compiler owners

| Feature | Intended owners / participants |
| --- | --- |
| **Native executable AOT** | llvm, java (Graal Native Image), net (NativeAOT), forge; cranelift planned |
| **Wasm guest compiler/host** | rs emits guest Wasm; wasm owns Wasm runtime/binary admission; llvm can target Wasm |
| **Native custom TenantVM** | llvm and forge primary; cranelift reuses same security contract |
| **Graal isolate-backed tenancy** | java |
| **Wasm-backed .NET tenancy** | net (host) + Wasm sandbox component |
| **Deno-style permission enforcement** | llvm, wasm, rs, java, net, forge, cranelift, atto; JS constrained and HDL specialized |
| **Per-socket read/write capabilities** | llvm, forge, java, net primary; wasm/rs delegated to host |
| **Zero guest GC** | llvm, wasm, forge, cranelift; realtime as a target constraint, HDL statically bounded |
| **Minimal guest-data GC** | java and net; not a claim about host VM GC |
| **Actor-local heaps and stacks** | llvm, forge, java, net, wasm; others delegate or provide constrained forms |
| **Soft real-time** | llvm, java, net, forge, realtime, hdl; wasm/rs/cranelift are constrained profiles |
| **Hard real-time** | llvm and realtime target profiles, HDL hardware targets; forge only with bounded certified profile |
| **Self-hosting** | atto bootstrapped by pinned compiler toolchain |
| **AOT-compatible telemetry instrumentation** | llvm, java, net, forge, realtime primary; others platform specific |

The reverse map expresses architecture responsibility and does not grant independent readiness or deployment authority.

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
| **Actor organization** | ActorRegistry, ActorGroups, supervisor CONTROL notices, self-discovered graphs, mailbox lifecycle and telemetry sidecars use common semantics. |
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

