# Oreslang pointerless ownership branch audit — 2026-10-21

This ledger records branch/PR lineages that were reviewed during the pointerless
ownership audit. It is a merge/restack guard, not a deletion list.

## Canonical source contract

Guest Oreslang expresses ownership and loans with:

- `rt copy value`
- `rt share value`
- `rt take value`
- `rt borrow value`
- `rt mut borrow value`

Borrowed type positions use `borrow T` and `borrow mut T`.

Guest source does not use `&` or `*` for address/reference/dereference
operations and does not expose `Pointer<T>`, `RawPointer<T>`, `rt ref`,
`rt deref`, `rt unref`, `rt raw`, source pointer arithmetic, or pointer
casts. Internal compiler/runtime pointers, handles, offsets, HIR/MIR
capabilities, and host-language `&`/`*` operations are implementation
details and are not banned.

## Current enforcement line

These are the current source-policy/enforcement PRs and should be treated as the
restack target rather than historical pointer-language branches:

- org policy: `.github#14`
- Java/Graal frontend: `oreslang-source.java#1841`
- LLVM frontend: `oreslang-source.llvm#823`
- .NET fail-closed gate: `oreslang-source.net#105`
- Forge fail-closed gate: `oreslang-source.forge#102`
- JS fail-closed gate: `oreslang-source.js#32`
- Wasm real-frontend negative gate: `oreslang-source.wasm#110` (draft/dependent on LLVM)
- shared source contract: `oreslang-common-source#117`

## Historical source-language alternatives — do not merge directly

The following branch lineages contain, contained, or were designed around a
source surface that is no longer canonical. They may contain useful
implementation ideas, but any useful commits must be semantically restacked
onto the current enforcement line without reviving their source syntax.

### Java/Graal

- `fix/rt-ownership-audit-20261007` — public `Pointer<T>` plus
  `rt ref/deref/unref`; PR #443 is closed.
- `audit/consuming-pointer-unref-20261015` — consuming public pointer API;
  PR #1021 is closed.
- `audit/pointer-borrow-zero-copy-20261015` — public pointer integration
  lineage; PR #1022 is closed.
- `audit/pointer-parent-merged-20261015` — unassociated branch containing
  source `Pointer<T>`, `rt ref`, `rt deref`, and `rt unref`; historical,
  incompatible source design.
- `feat/rt-pointerless-reference-semantics-20261004` — older `rt ptr` /
  `rt deref` source design; PR #186 is closed.
- `feat/pointerless-borrow-checking` — historical borrow-checker branch on a
  non-main base with legacy `rt borrow mut` and pointer-sigil fixtures; PR #49
  is closed. Restack only the useful ownership-analysis logic.

### LLVM

- `feat/logical-ref-source-ownership-20261020` — exposed source `Ref<T>`
  actor state and payload access; PR #803 is closed. Keep managed-reference
  representation internal to HIR/MIR/backend lowering.
- `audit/pointer-fail-closed-20261015` — older parser-only gate; PR #415 is
  closed in favor of #823.

### Shared contract

- `audit/pointer-capabilities-20261015` — public `Pointer<T>` contract;
  PR #38 is closed.
- `audit/pointer-zero-copy-contract-20261015` — public pointer release/deref
  contract; PR #39 is closed.
- `audit/trusted-raw-pointer-escape-20261017` — guest raw-pointer escape-hatch
  contract; PR #60 is closed.

## Internal-only pointer/reference work that may continue

The following work is allowed only as trusted implementation machinery. It must
not leak a second source ownership/reference model:

- LLVM #416: host `ActorOwnedGraphHeap` lifetime/loan hardening; source parser
  has been aligned to `rt mut borrow` and permanent pointer-form rejection.
- LLVM #526: `TrustedRawPointer<T>` backend/core proof object; guest source
  cannot name or create it.
- LLVM #527: host bounded pointer/MMIO substrate only.
- LLVM #758: compressed/native managed-reference representation is HIR/MIR
  internal; no source `Ref<T>` / `Pointer<T>`.
- Rust backend #6: compressed reference representation is internal MIR/backend
  state, not guest source syntax.
- Java #468: `Ref<T>/WeakRef<T>` may only survive review as a nominal
  actor-local cell/container API. It must not acquire address-of, explicit
  dereference, pointer arithmetic, or alternate borrow syntax.

## Review/restack rule

Before merging or reopening any older ownership/pointer/reference branch:

1. Rebase/restack it on the current backend enforcement PR or its merged
   descendant.
2. Search added guest source, fixtures, docs, diagnostics, and generated
   Oreslang for legacy pointer forms.
3. Preserve host/backend pointer code only when it remains inaccessible as
   guest syntax.
4. Keep negative fixtures containing forbidden forms clearly marked as
   rejection tests.
5. Do not restore legacy syntax merely to make an old branch's tests pass;
   update the branch parser/tests to the canonical language contract instead.
6. Require executed compiler evidence before claiming backend parity.
