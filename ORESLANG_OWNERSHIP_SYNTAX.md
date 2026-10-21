# Oreslang pointerless ownership contract

Oreslang user-facing syntax MUST NOT use C, C++ or Rust pointer/address/dereference spellings such as `&value`, `&mut value`, `*ptr`, `*const T`, or `*mut T` for ownership or borrowing.

Canonical syntax:

- `rt copy value` — explicit copy
- `rt share value` — immutable share
- `rt take value` — move ownership
- `rt borrow value` — immutable loan
- `rt borrow mut value` — exclusive mutable loan

`rt mut borrow value` is not canonical. Parser, type checker, formatter, examples, language tests, compiler diagnostics, and user-facing API documentation MUST agree on the canonical order. No backend is allowed to silently reinterpret pointer operators as ownership operators.

Scope: every branch and pull request across the Oreslang organization, including GraalVM, LLVM, WASM, .NET, Rust, and JavaScript backends. New PRs should include positive examples and negative compiler tests for source expressions `&value`, `&mut value`, and `*ptr` in ownership contexts. Confirm that loans do not escape through asynchronous tasks, closures, channels, FFI, actors, `rt copy`, `rt share`, or composite values.

This is a **source-language** restriction, not a blanket ban on punctuation or host compiler internals. Legitimate bitwise operations, multiplication, Markdown formatting, LLVM IR, C ABI declarations, and host-language pointers are excluded from the syntax rule. Lex and parse before classifying a violation; do not use naive punctuation grep to automatically rewrite files.

For cross-compiler parity, fail closed when ownership provenance or suspend-safety cannot be proven; distinguish immutable sharing from mutable loans; prohibit dangling loans, use-after-`rt take`, exclusive-loan aliasing, and borrowed capture escaping a Task. Require compile-fail tests alongside accepted code.

Do not mark a branch or PR compliant merely because it includes this document. Verify the exact head SHA against compiler tests; recheck after every rebase or merge.
