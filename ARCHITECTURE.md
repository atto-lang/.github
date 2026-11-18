# Compiler engineering architecture

**Scope:** engineering orientation for AttoLang, the independently developed Oreslang-named compiler implementations, and integration with Zed-Pkg. This is an architecture map, **not** a declaration that every backend supports every described feature.

## Source of truth and repository boundaries

- [AttoLang specification](https://github.com/atto-lang/atto-spec): normative grammar, typing, ownership, actor/effect semantics, compatibility and approved proposals. An accepted proposal is not proof of implementation.
- [AttoLang common source](https://github.com/atto-lang/atto-common-source): shared contracts, model types and future common semantic infrastructure where verified. Never assume common code is already linked into every backend.
- [AttoLang platform conformance](https://github.com/atto-lang/atto-platform-conformance): portable fixtures, negative oracles, provenance and backend-specific execution receipts.
- Individual **atto-source.\*** repositories: independently own parsing/admission, semantic lowering, generated target code, runtime support and local tests. The Rust browser/Wasm compiler is a constrained executable subset, not an all-language reference.
- Existing **oreslang-source.\*** implementations: historically named, independent compilers with backend-specific runtime and legacy compatibility. Java/GraalVM and LLVM have distinct implementation paths and verification states; other targets must not inherit claims from either.
- [Zed-Pkg CLI](https://github.com/zed-pkg/zed-cli): external package installation, version/lockfile and provenance surfaces. The compiler must explicitly declare how manifests, import roots and package resolution interact; package-manager behavior is not language semantics.

The public language name is **AttoLang**; Oreslang names, the .ores extension, compiler binaries, package coordinates, ABIs and protocols remain compatibility surfaces until a versioned migration with tests. New portable language source should prefer .atto **where that backend actually admits it**.

## Target compilation pipeline and ownership

~~~text
.atto / compatible .ores source + package/import manifest
    -> lexer/parser and AST with source spans
    -> name/module/type/visibility resolution and diagnostics
    -> typed HIR (effects, capabilities, ownership and actor identity)
    -> verified MIR/CFG (moves, loans, lifetime/suspension boundaries)
    -> backend-specific IR and verified lowering
    -> native binary / JVM / Native Image / WebAssembly / other target
    -> actor runtime, capability provider, scheduler and I/O
~~~

This is a **target decomposition**: some compilers have different IR names/stages or have only parse/admission profiles. HIR and MIR are not automatically shared artifacts. In particular LLVM IR is not the common MIR. Treat a missing verifier or unsupported guest lowering as a **fail-closed feature gap** rather than success.

**Backend boundary:** shared specifications prescribe observable program behavior. Backends own ABI, representation, physical memory layout, code generation, platform I/O and optimization. Changes to externally observable behavior require an RFC and portable conformance fixtures; target-specific optimizations must preserve semantics.

## Runtime invariants to design and verify

1. Actors own mutable state and isolate messages and lifetimes; sealed actor-state access and critical read/write are governed by the specification. A parser admitting a construct does not prove its guest runtime semantics.
2. Borrow/move/share/deep-copy rules apply across calls, actor messages, awaits, cancellation and exceptional exits. A deep copy must not leave mutable nested aliases pointing into the source graph; a transfer invalidates the former owner.
3. Supervisor CONTROL notices, actor teardown, mailbox closure and panic/raise/throw semantics require real concurrency and lifecycle tests, not only a host model or serialized JSON shape.
4. Capability-gated host interactions are deny-by-default; actor isolation alone is not an operating-system security boundary.
5. LLVM/native paths aim for controlled AOT and isolated actor storage; Java/Graal paths include JVM and Native Image. **Zero GC, hard real-time, Native AOT and tenant isolation are acceptance goals, not blanket current guarantees.**
6. Ensure deterministic ABI/wire formats, checked allocation/offset math and per-backend declarations of unsupported profiles. Parser success, host-unit success and executable guest proof are distinct.

## Design review checklist

For every cross-cutting change identify: specification revision, expected typed IR changes, backend scope, ABI and runtime impact, ownership/exception/async behavior, adversarial negative cases, conformance oracle, build provenance, migration plan and an explicit **PASSED / FAILED / UNSUPPORTED / NOT_EXECUTED** matrix.

Read [RFC_PROCESS.md](RFC_PROCESS.md), [TESTING.md](TESTING.md), and [ROADMAP.md](ROADMAP.md). The code and workflow at an exact source SHA, not this document, determine implemented functionality.
