# Organization policy: controlled dynamism for AOT and zero guest-GC

**Effective review directive; semantic details are TARGET / DRAFT pending specification review and backend execution.** Applies to AttoLang/Oreslang compiler backends, shared core/alloc/std, examples, documentation and agent-generated changes. Do not silently change supported JVM/JIT/hybrid modes. Canonical language amendment: [AOT/zero-GC chapter 22](https://github.com/ores-truffle-oreslang/attolang-language-spec.md/pull/18). Chapter is proposed until merged; keep behavior changes independently reviewed.

## First: declare the execution and memory profile

A compiler/runtime feature PR must identify:
- **Code:** interpreter / governed JIT / hybrid / host-only AOT / **full guest AOT sealed** / **precompiled-module AOT**. Sealed AOT requires every reachable guest target, static descriptor and specialization available at link time, and forbids runtime guest interpretation/JIT/code emission. Governed precompiled-module AOT is separate, with ABI/capability/generation controls.
- **Memory:** managed host GC / **guest zero tracing-GC** / **whole-process zero-GC**. Guest-owned arenas do not certify process zero-GC. Allocation itself is allowed; stale references, leaks and double drops are not.
- **Evidence:** source tree and fixture blob SHAs, compiler head, target artifact digest, actual stepful guest execution and allocator/GC evidence. Claim no more than the tested profile.

## Feature design checklist

1. **Reject unbounded runtime dynamism in sealed AOT:** `eval`, runtime source/bytecode compilation, dynamic method/class creation, monkey-patching, unrestricted host reflection, open-world method resolution and implicit interpreter/JIT fallback. Reject before executable IR/code output with stable diagnostics.
2. **Preserve expressive static features:** virtual dispatch, inheritance, interfaces, macros, generics, typeclasses, closures and async/actors work through finite reachability metadata, build-time specialization, statically compiled state machines and generated metadata. Dynamic *values* are fine; late code generation is not.
3. **Proxies:** synthesize typed forwarding/interception methods at compile time for a known class or bounded interface. Keep receiver/private access, method arity, static-vs-instance separation, effects, capabilities and ownership correct; do not magically expose methods or create prototypes at runtime.
4. **Every allocation has a lifetime:** owner/region, bounded accounting, teardown, error behavior, and strategy for long-lived actors. Strong-reference cycles require weak edges, an explicit controlled lifetime/region, or rejection. Reference counting alone cannot reclaim arbitrary strong cycles.
5. **Suspension and cancellation:** move/borrow/share rules must hold across `await`, task migration, exceptions, actor restart and hot-upgrade generation retirement. Async task frames/closure environments remain owned or pinned; cleanup is exactly once and safe against outstanding native I/O.
6. **Messaging:** only validated Sendable data crosses channels/pub-sub. Never pass lexical closures, methods, mutable actor-state aliases, raw addresses, or concealed host capabilities.
7. **No language backdoors:** preserve no implicit conversions, no operator overloading, no `%` / `^` / `^^`, and no automatic nested `Future`/`Option` flattening. Generated proxy/macro code must obey the same checker.
8. **Guest source first:** implement portable core/alloc/std and guest-visible logic in canonical `.atto`, leaving only compiler, ABI, security and essential host primitives in Java/C#/C++/Rust/JS as appropriate. `.ores` remains migration-only.
9. **Be truthful about hosts:** GraalVM Native Image and .NET NativeAOT can compile hosts while using a managed GC; browser JS has GC and often JIT; wasm32 linear-memory guests may be deterministic while the host remains managed. No backend inherits another backend's proof.

## Minimum review and test matrix

| Positive evidence | Mandatory negative or stress counterpart |
| --- | --- |
| Static virtual dispatch and compiler-generated `Proxy<T>` wrapper | unknown target, reflection escalation, missing dispatch edge, private/static confusion |
| AOT-linked macros/generics/reflection descriptors | runtime `eval`, code injection, open-ended class creation, hidden interpreter fallback |
| Ownership-safe closures, moved task frames | escaping loan across `await`, closure/function sent across actor boundary, double-drop/use-after-take |
| Actor local heap and long-lived operation churn | allocation failure, quotas, strong-cycle misuse, actor cancellation and restart with pending I/O |
| Precompiled hot-upgrade in explicitly governed mode | unsigned/unapproved ABI, stale generation, retired frame/handle access |
| Verified guest binary/module and allocator behavior | host-only mock, parser-only acceptance, source-free canary, `steps=[]` or `steps=null` |

**No promotion from TARGET to IMPLEMENTED/PARITY without real backend-specific compiler-backed guest execution and independent review.** Record separate findings for Java/GraalVM, LLVM, .NET, Wasm, Forge/Cranelift, JavaScript, Rust and HDL profiles; do not present experimental scalar code as full language support.

## Governance preserved

Do not bypass required reviewers, branch protection or CI; zero-step runner admission is NOT_EXECUTED. Do not transfer private source trees to unapproved organizations. Do not add Python: first-party tooling remains Rust-only under [the existing policy](RUST_ONLY_POLICY.md). This guidance is additive and does not loosen the existing ownership, actor, capability or security contracts.
