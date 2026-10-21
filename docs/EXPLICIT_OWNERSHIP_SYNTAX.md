# Oreslang source ownership syntax policy

Status: normative for Oreslang-language syntax in every compiler/backend, library, test, documentation example, open PR, and maintained branch.

## Explicit ownership operations

Oreslang does **not** use Rust/C/C++ pointer sigils for borrowing, references, ownership transfer or dereference.

- `rt copy value` — explicit copy, subject to Copy/ownership rules.
- `rt share value` — produce an authorized shared, deeply immutable or otherwise safely shared capability; never bypass domain/actor isolation.
- `rt take value` — transfer/move ownership, invalidating the source binding where required.
- `rt borrow value` — immutable scoped borrow.
- `rt borrow mut value` — exclusive mutable scoped borrow.

Canonical order for mutable borrows is `rt borrow mut` (not `rt mut borrow`). Define precise lifetime, suspension, aliasing and task/actor-boundary rules separately; spelling does not weaken those checks.

## Forbidden Oreslang pointer-like forms

Do not introduce `&x`, `&mut x`, `*ptr`, `&T`, `&mut T`, `*const T`, `*mut T`, or pointer-typed `T*` as ownership/reference/dereference syntax. Do not silently translate such forms to `rt` operations. Reject them with a targeted diagnostic that suggests the correct `rt` form.

The ban is **context-sensitive**: `*` as multiplication and `&` as a separately specified bitwise operator, where valid, are not pointer operations. C/C++/Rust implementation code, generated LLVM IR, runtime ABI pointers, and quoted examples of invalid Oreslang in negative tests are not prohibited.

Do not merely grep for individual punctuation characters: the lexer/parser must classify operators by grammatical context. Likewise distinguish Oreslang source from host-language code and Markdown prose.

## Conformance requirements

1. Positive and negative fixtures for each of the five explicit operations. Include field/element borrows, nested expressions, reborrowing, temporaries, shadowing, moves, aliasing, actor state, and async/task suspension boundaries.
2. Negative fixtures for each forbidden pointer-like form, asserting diagnostic code/span and no code generation or artifact publication.
3. Verify the real frontend/parser/type checker, formatter and source emission paths. Formatter must preserve/canonicalize `rt borrow mut`; no target may interpret `&` or `*` as an implicit borrow or dereference.
4. Run equivalent test contract through GraalVM/Java, LLVM, Wasm, .NET, JS and Forge. An unsupported backend must fail closed, not report conformance from a stub.
5. Compare every open PR head with its target base, not merely default branches; audit maintained feature branches independently. Recheck the merged tree on the protected integration branch.
6. Report coverage as `verified`, `violation`, `unsupported`, or `not examined`; never claim full parity without actual compiler evidence.

## PR review gate

Changes adding Oreslang syntax/examples must include explicit-ownership conformance tests. A violating source addition blocks merge until fixed; negative test inputs intentionally containing forbidden forms are exempt when clearly marked. Avoid auto-rewriting code without resolving semantic ownership/lifetime effects.

This rule concerns **Oreslang-language semantics**; host backends may internally use conventional pointers/references to implement bounds-checked arenas and runtime memory safety.
