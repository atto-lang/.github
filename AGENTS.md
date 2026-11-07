# AttoLang contributor and agent directives

The repository is the source for organization-wide contributor policy. Follow [controlled-dynamism and AOT/zero-GC review rules](docs/AOT_ZERO_GC_DYNAMISM_POLICY.md) for all proposed language, compiler and runtime changes. These rules apply to both human and AI-authored changes.

- **Declare the deployment profile** (JIT/hybrid, host-only AOT, sealed guest AOT or precompiled-module AOT) and the memory claim (managed-host, zero guest tracing-GC, or process zero-GC).
- **No uncontrolled runtime dynamism in sealed AOT:** reject runtime eval/code emission, open-ended reflection/class/method generation and implicit runtime interpreter fallback. Generate proxies, dispatch metadata, macros, generics and closure machines at build time.
- **No implicit memory escape:** check ownership, cycles, actor isolation, suspended frames, cancellation, generation fencing and deterministic cleanup before native lowering.
- **Fail closed with negative `.atto` fixtures** and require exact-head stepful compiler/guest evidence; `steps=[]` and `steps=null` are NOT_EXECUTED.
- Preserve [Rust-only tooling](docs/RUST_ONLY_POLICY.md), no private-source transfer without approval, existing security reviews and the canonical `.atto` source-first approach.

This document is instruction, not evidence that a backend has implemented any feature.
