# Oreslang source ownership syntax policy

Status: normative for Oreslang-language syntax in every compiler/backend, library, test, documentation example, open PR, and maintained branch.

## Explicit ownership operations

Oreslang does **not** use Rust/C/C++ pointer sigils or a public pointer/reference API for borrowing, references, ownership transfer, dereference, or address formation.

- `rt copy value` — explicit copy, subject to Copy/ownership rules.
- `rt share value` — produce an authorized shared, deeply immutable or otherwise safely shared capability; never bypass domain/actor isolation.
- `rt take value` — transfer/move ownership, invalidating the source binding where required.
- `rt borrow value` — immutable scoped borrow.
- `rt mut borrow value` — exclusive mutable scoped borrow.

Canonical mutable-borrow spelling is `rt mut borrow`.

Ordinary field/index/member access operates on the resulting value or loan. Oreslang source does not require an explicit dereference operator.

## Forbidden Oreslang pointer-like forms

Do not introduce any of these as user-facing Oreslang syntax or types:

- `&x`, `&mut x`, `*ptr`, `&T`, `&mut T`, `*const T`, `*mut T`, or pointer-typed `T*`.
- `Pointer<T>`, `RawPointer<T>`, or another general source-visible pointer/address type.
- `rt ref`, `rt deref`, `rt unref`, address-of, explicit source dereference, pointer arithmetic, integer-to-pointer casts, or pointer-to-integer casts.
- Silent desugaring of any forbidden form into `rt copy`, `rt share`, `rt take`, `rt borrow`, or `rt mut borrow`.

Reject forbidden forms with a targeted diagnostic that suggests the appropriate ownership operation where one exists. If no safe ownership operation is equivalent, reject without an escape hatch.

The ban is **context-sensitive**: `*` as multiplication and `&` as a separately specified bitwise operator, where valid, are not pointer operations. C/C++/Rust implementation code, generated LLVM IR, runtime ABI pointers, and quoted examples of invalid Oreslang in negative tests are not prohibited.

Backend/runtime internals may use host-language pointers, addresses, region handles, offsets, or capabilities inside the trusted implementation boundary. Those mechanisms must not leak into guest/source syntax.

## Source-level semantics

The source model is ownership- and loan-oriented, not pointer-oriented:

- Need another owned value: `rt copy`.
- Need safe shared authority: `rt share`.
- Need to transfer ownership: `rt take`.
- Need temporary read access: `rt borrow`.
- Need temporary exclusive mutation: `rt mut borrow`.

A backend may lower these operations to native pointers, handles, arena offsets, references, GC roots, linear-memory offsets, or another representation. That lowering is an implementation detail and must preserve the same source semantics.

## Conformance requirements

1. Positive and negative fixtures for each of the five explicit operations. Include field/element borrows, nested expressions, reborrowing, temporaries, shadowing, moves, aliasing, actor state, and async/task suspension boundaries.
2. Negative fixtures for every forbidden source-pointer form, including `&value`, `&mut value`, `*ptr`, `T*`, `Pointer<T>`, `rt ref`, `rt deref`, and `rt unref`. Assert diagnostic code/span and no code generation or artifact publication.
3. Verify the real frontend/parser/type checker, formatter and source emission paths. Formatter must preserve/canonicalize `rt mut borrow`; no target may interpret `&`, `*`, or the legacy pointer intrinsics as implicit ownership operations.
4. Run the equivalent contract through GraalVM/Java, LLVM, Wasm, .NET, JS and Forge. An unsupported backend must fail closed, not report conformance from a stub.
5. Compare every open PR head with its target base, not merely default branches; audit maintained feature branches independently. Recheck the merged tree on the protected integration branch.
6. Report coverage as `verified`, `violation`, `unsupported`, or `not examined`; never claim full parity without actual compiler evidence.
7. Host-language test code may contain native `&`/`*` syntax. Guest Oreslang fixtures may contain forbidden forms only when they are explicit negative tests.

## PR review gate

Changes adding Oreslang syntax/examples must include explicit-ownership conformance tests. A violating source addition blocks merge until fixed; negative test inputs intentionally containing forbidden forms are exempt when clearly marked. Avoid auto-rewriting code without resolving semantic ownership/lifetime effects.

Drafts or branches that introduce public `Pointer<T>`, `rt ref`, `rt deref`, `rt unref`, or source raw-pointer arithmetic must be rewritten, superseded, or closed before merge. Runtime/TCB pointer machinery can remain only if it is inaccessible as Oreslang source syntax.
