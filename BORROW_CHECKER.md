# AttoLang borrow checker and ownership model

> **Language design guide — 18 November 2026.** This guide describes the **target AttoLang semantics**, including the current tips of relevant `atto-spec` feature branches, **as though those language-design PRs are incorporated**. It is not evidence that every parser, compiler, runtime, or backend already implements the complete model. Examples marked **target syntax** may not compile in current releases. The canonical normative authority remains [`atto-spec`](https://github.com/atto-lang/atto-spec); this organization-level document is an explanation, not a replacement specification.

AttoLang makes *authority over memory* explicit while keeping the routine use of objects pleasant. Most code reads an object, calls methods, and passes objects to functions **without writing pointer punctuation**. When code must change ownership, share an allocation, or request a mutable loan, it names that operation:

- `borrow!` — temporarily access an object without owning it.
- `take!` — transfer unique ownership.
- `share!` — create or duplicate **read-only shared ownership** of suitably frozen data.
- `copy!` — make a **shallow** copy, subject to alias-safety checks.
- `copy!!` — make an **independent deep** copy of a mutable reachable graph.

These operations have different meanings. The compiler must never substitute one for another merely to make a program type-check.

## 1. The mental model: owners, places, loans, and domains

An **owner** has the authority to retain and eventually release a resource. An ordinary uniquely owned mutable allocation has one authoritative owner at a time. A **place** is a statically tracked location such as a local variable, field, or indexed element. A **loan** temporarily grants read or exclusive write access to a place **without** transferring its ownership.

The borrow checker follows:

1. **Identity and provenance:** what allocation or value a place refers to; the owner, its generation, and the path through fields/containers.
2. **Rights:** unique ownership, shared read loan, exclusive write loan, frozen shared ownership, or an explicitly guarded synchronization capability.
3. **Lifetime:** when a loan begins, its last permissible use, the owner's scope/region, and all exit paths.
4. **Memory domain:** actor, arena, worker, isolate, tenant, foreign allocation, or protected shared region; membership is not inferred from the type name alone.
5. **Effects and exits:** calls, returns, branches, loops, throw/raise/panic, cancellation, `await`, `select`, critical sections, FFI, and actor mailbox admission.

A compiler can represent these in typed HIR/MIR as categories such as `OwnedUnique<T>`, `BorrowRead<T, lt>`, `BorrowWrite<T, lt>`, and `SharedFrozen<T>`. These are *semantic states*, not mandatory surface types. Borrow checking is a source/typed-IR proof; runtime generation, bounds, domain, and retirement checks still matter for handles and pointers.

### The most important invariant

**There must never be an untracked writable alias.** Multiple readers can coexist, or one exclusive writer can exist, but a writer must exclude all overlapping readers and writers. Moving or destroying an owner while its loan is live is forbidden. A mutable object reachable by two actors must be independently owned by each actor, or accessed under a separately proven lease/protection mechanism; an ordinary shared handle never grants unguarded writable access.

| Active access to the same place | New read loan | New exclusive write loan | `take!` / `drop!` |
| --- | --- | --- | --- |
| No loan | Allowed if owner/provenance valid | Allowed only with writable authority | Allowed only with unique ownership |
| One or more read loans | Allowed | Rejected | Rejected |
| Exclusive write loan | Rejected | Rejected | Rejected |
| Frozen shared owner | Read-only access allowed | Rejected without an explicit guard/unique recovery | No arbitrary unique-owner drop |

Field-disjointness and other refinements are permitted **only** when the compiler proves the places do not alias. Unknown provenance or unknown effects are not permission to proceed.

## 2. Ordinary function calls borrow by default

For **nonprimitive** function/method arguments, the intended default is an implicit, checked, non-owning **read loan**. A formal `Foo f` does **not** mean an implicit ownership move, deep copy, shared reference-count increment, or heap allocation.

~~~atto
define class Foo as
  pub let int value = 1;
end

fnc inspect(Foo item): int {
  return item.value;
}

pub fnc main(): int {
  val foo = new Foo();
  val first = inspect(foo);          // implicit read loan
  val second = inspect(borrow! foo); // same loan rights, written explicitly
  return first + second;             // foo remains owned by this caller
}
~~~

A caller should write `inspect(foo)`, not `inspect(borrow! foo)` everywhere. The explicit form is for clarity, disambiguation, or a place where the compiler demands an ownership/lifetime explanation. **Explicit syntax cannot turn an unprovable loan into a valid loan.**

A function that mutates the referred-to object needs **exclusive access**. The target spelling at the caller is `adjust(mut foo)`, with `adjust(borrow! mut foo)` as an explicit equivalent. The compiler must know the transitive, statically sealed mutation effects of the callee (including callbacks and virtual/indirect calls); when effects cannot be proven, it must require an explicit contract or reject the call.

~~~atto
// Target syntax; effect inference and call-form grammar are still staged.
fnc adjust(mut Foo item): void {
  item.value += 1;
}

pub fnc example(): int {
  val mut Foo foo = new Foo();
  adjust(mut foo);              // exclusive, non-owning temporary loan
  // adjust(foo);               // reject: mutable loan required
  return foo.value;
}
~~~

Primitive scalars such as `int`, `bool` and `float` retain value-passing behavior. Other aggregates/collections need their own proven representation and ownership rules before receiving the same sugar. Importantly, older compiler profiles may still treat a bare class formal as consuming: **migrate them with a versioned, explicit compatibility rule**, never with a silent ABI change.

At a call, evaluate the receiver and arguments once and in source order. A loan established for an earlier argument must remain valid through the later arguments and the call. Named arguments, defaults, spread forms, overloads and functional pipelines cannot reorder evaluations or shorten the borrow window. The chosen parameter ownership modes belong in the typed signature before AOT lowering.

## 3. `borrow!` — a temporary loan, not a pointer

- `borrow! owner` issues a **shared, readonly loan**. Many simultaneous readers are fine.
- `borrow! mut owner` issues an **exclusive, writable loan**, requiring writable owner authority and no overlapping loan.
- The owner is still responsible for the allocation and may use it again after the loan expires. During a conflicting loan, however, the checker prevents mutation, movement, replacement or destruction that would invalidate the view.
- A loan cannot be promoted into an owned object, retained after its owner dies, sent as an unrestricted asynchronous actor message, or stored in a longer-lived object without an authenticated lifetime relationship.

~~~atto
// Illustrative ownership behavior.
val mut Foo owner = new Foo();
val a = borrow! owner;
val b = borrow! owner;       // two readers coexist
// val w = borrow! mut owner; // reject while a or b may still be used
// val moved = take! owner;   // reject while a or b are live
~~~

Loans are **not necessarily held until the end of a textual block**. A verified non-lexical lifetime (NLL) analysis may retire a loan after its last possible use across all control-flow paths. If the compiler cannot prove that earlier end, it conservatively retains the loan and rejects conflicting operations. A borrow is not an exposed integer address; access may lower to registers, compiler-verified object handles, offsets or native addresses internally.

### `mut` controls permission, not magic

`val` versus `let` controls whether a *name can be rebound*; `mut` controls whether it can mutate a *referent*. These rights are orthogonal:

| Declaration | Rebind the name? | Mutate a permitted referent? |
| --- | --- | --- |
| `val Foo x = expr;` | No | No |
| `val mut Foo x = expr;` | No | Yes |
| `let Foo x = expr;` | Yes | No |
| `let mut Foo x = expr;` | Yes | Yes |

A `mut` name cannot bypass `final` fields, ownership moves, readonly/shared handles, active loans, private fields, sealed actor state, or foreign-memory boundaries.

## 4. `take!` — explicit transfer of unique ownership

`take! value` consumes the source ownership **capability** and transfers it. The old owning place cannot be used again; there is **no byte-copy requirement**. An invalidated source is not a nullable handle, and moving an owner with outstanding loans is rejected.

~~~atto
val first = new Parcel();
val second = take! first; // second owns the original Parcel
// first.use();           // compile error: moved owner
~~~

For a function that genuinely needs ownership, the **resolved formal must explicitly be a consuming formal**. A candidate spelling is `take! Parcel p`; the formal grammar/ABI remains subject to its compiler admission stage. A normal `Parcel p` formal borrows in the new language profile. Therefore `consume(take! parcel)` is deliberate; `inspect(parcel)` is not a move.

### Atomic actor handoff

Sending a moved value is more subtle than a local move. The mailbox may be full, closed, revoked or cancelled. The intended actor/Mailman protocol is **prepare → commit or abort**:

1. Verify the entire reachable graph is uniquely owned, loan-free, `Sendable`, and valid for the destination actor/domain; reserve capacity.
2. Stage/escrow transfer authority so the sender cannot race a pending send.
3. **Commit once** at the mailbox admission linearization point, publishing one complete message and invalidating the sender's authority.
4. **Abort before commit** by returning one recoverable unique owner and publishing nothing; never leak, silently destroy, or double-deliver the payload. The exact API convention for returning the payload on a rejected `Result` is still subject to review.

A receiver crash **after** a successful commit does not magically restore the sender's ownership. The receiver/runtime owns retirement and cleanup. A cross-domain move can require physical relocation/serialization even if the *source-level* operation is `take!`; zero-copy is an optimization only within proved compatible domains.

## 5. `share!` — same object, frozen permissions

`share!` **does not copy the graph**. It establishes/duplicates ownership of the *same logical allocation* under **readonly shared rights**.

| Input | Effect |
| --- | --- |
| Unique mutable owner | Validate transitive deep freeze; consume the unique writable capability, publish a shared owner |
| Already frozen shared owner | Duplicate a shared owner of the **same** allocation, with lifetime/accounting checks |
| Bare temporary read loan | **Reject** promotion into owned shared authority |
| Unproven mutable aliases, actor-private pointer or unsafe interior mutation | **Reject** until separately proved safe |

~~~atto
// Target shared-owner semantics.
val owner = make_immutable_ready_snapshot();
val shared = share! owner;   // same underlying graph; owner is consumed
val another = share! shared; // same underlying graph; another shared owner
// shared.state = 42;       // direct mutable access forbidden
~~~

**Deep freeze is transitive**: fields, arrays, maps, closures/captures and cycles must contain no reachable ordinary mutable edge with a surviving writable alias. Merely casting a view to “readonly” does not remove somebody else's write permission. `freeze! take! value` can produce a frozen **unique** owner *without* creating shared ownership; `share!` then separately determines shared publication. Neither operation automatically permits tenant/actor transport.

Need safe shared mutation? Use an explicit, trusted synchronization container such as `Shared<RwLock<T>>` (library surface still proposed), rather than weakening `share!`:

~~~atto
// TARGET library API, not a currently portable executable fixture.
val state = share! RwLock(Counter());
val alias = share! state;
val count = alias.with_read(|counter| -> counter.value);
state.with_write(|mut counter| -> { counter.value = 42; });
~~~

The guard grants many readers **or** one writer, never both, and cannot escape, be sent, or remain active across unproven suspension, callback effects or actor turns. All normal/controlled-exceptional exits release the guard once. `share!` does not silently acquire locks; refcounting is **not** a data lock, and strong reference counts alone cannot guarantee reclamation of cyclic graphs. Synchronization and immutable `Sendable` are distinct from `Serializable`.

## 6. `copy!` — shallow copy with checked child aliases

**The explicit revision of the copy contract is: `copy!` is shallow, *not deep*.** This supersedes historical deep-by-default proposals and keeps `copy!` distinct from `copy!!`.

- An identity-bearing class gets a **fresh root of its concrete dynamic type**, with its immediate stored fields copied. The constructor is not rerun just to copy the object.
- Embedded scalar/value fields are duplicated by value.
- Reference-typed child fields still refer to the **original children** *only if those references are legally shareable with verified owner/lifetime rights*.
- If copying would duplicate a unique owner or introduce an illegal mutable alias, **reject `copy!`** (e.g. `E_COPY_SHALLOW_ALIAS`). Never secretly upgrade it to `copy!!`.
- A shallow `Array<int>` duplicates the **outer array storage** and integer slots, not just a descriptor/address; for nested reference-typed arrays it does **not** automatically duplicate the inner arrays.
- For maps/sets/lists, copy container bookkeeping as required and prove element/key/value alias and equality/hash invariants. Empty runtime collections do not waive static element-type proof.

~~~text
source_root -> child_A (unique mutable)

copy! source_root
  produces fresh_root
  but cannot give fresh_root an illicit second unique owner of child_A
  => compile rejection unless child_A is legally shareable
~~~

The zero-argument method `obj.copy()` is the corresponding object-level shallow operation *if* its separately proven `Copyable<Self>` capability exists. Having `DeepCopyable` does not imply `Copyable`. Class/aggregate witnesses may be synthesized from proven fields, including inherited fields; an override declaration or annotation is not itself proof.

## 7. `copy!!` — deep clone of an independent mutable graph

`copy!! value` and the admitted zero-argument `value.copy_deep()` mean **deep graph duplication**. Every independently mutable reachable allocation gets a fresh destination allocation. A deep copy is **not** `memcpy` of a root pointer, `copy!` on the outermost object, a hidden `share!`, or serialization by default.

### Shared descendants and cycles: preserve topology, not source aliases

~~~text
SOURCE                            CLONE FROM ONE copy!! CALL

root --left-->  node A            root' --left-->  node A'
     --right--> node A                 --right--> node A'

A.next --> A (cycle)              A'.next --> A' (cycle)

A' is NOT A.
Within the clone, left and right still refer to the SAME A'.
~~~

A correct generated copier uses one **source-identity → destination-identity memo** for an entire invocation (with allocation-domain/type provenance), and:

1. Check a complete transitive type/field/ownership witness, graph/copy budget and destination domain.
2. Allocate an **unpublished destination shell** for the actual **concrete subtype**; insert it in the memo **before** descending into children.
3. Copy embedded values and recursively clone each owning mutable reference using that same memo. Repeated references reuse the destination already registered; cycles terminate with the correct new cycle.
4. Include all inherited, private, generic-instantiated, array/container, ADT variant and dynamically dispatched fields. Do not slice a Derived object into Base.
5. Reuse immutable interned/backing data **only** if it has a proven immutable/lifetime/domain sharing permission. A mutable `Shared<T>` pointee requires an independent clone or rejection.
6. On success, publish one fully formed root. On failure, release unpublished allocations exactly once using a rollback ledger; do not expose a partial object, mutate the source, or leave a stray alias.

**Each separate `copy!!` call gets a separate memo and graph**, including two sends to different actors. Shared descendants are preserved **inside** one clone; mutable descendants are not shared **between** recipient clones or with the source.

~~~atto
// Target actor-send contract; requires separate Sendable and mailbox proof.
worker_a.send(copy!! work_item);
worker_b.send(copy!! work_item);
// source stays owned; each accepted recipient has an independent graph.
~~~

A clone must not rerun constructors, arbitrary initializers, I/O, or unknown destructor/copy hooks as a side effect. The compiler emits a **closed-world AOT copy plan** with field offsets/type tags, custom verified copy hooks, and concrete subtype dispatch. A user-facing `copy_deep()` has **no context argument**; the memo and destination allocator are internal. An overridden `super.copy_deep()` uses a whole-concrete-type compiler baseline, not an ancestor-only shallow clone. Missing layout/version, unsupported cycles, erased dynamic type, unsafe FFI field, resource, finalizer, actor, socket, lock, or unverified override **fails closed**.

`copy!!` alone does **not** imply `Sendable` or `Serializable`; copying a process-local socket or raw foreign pointer does not make it a legal actor message. Hard-real-time builds require bounded nodes, depth, time and allocation or a deliberate compile/admission rejection. A proven optimizer may elide a physical copy only when whole-graph uniqueness and unobservability, constructor/copy-hook effects, identity and failure behavior establish complete equivalence; a still-live source cannot be silently consumed.

### What about common types?

| Value | `copy!` | `copy!!` |
| --- | --- | --- |
| `int`, `bool`, `float`, identity-free value | Copy value | Copy value |
| Immutable `string` / interned backing | Reuse safe immutable backing | Reuse safe immutable backing |
| Class with scalar fields only | Fresh concrete root with copied fields | Fresh concrete root with copied fields |
| `Array<int>` | Fresh outer array, same integer values | Same observable outcome |
| `Array<Array<int>>` | Fresh outer array; inner alias only if permitted | Fresh outer and inner arrays |
| Class with repeated mutable child/cycle | Reject unsafe shared child | New graph preserving internal aliases/cycles if copier verified |
| Actor, mutex, channel, file/socket, foreign resource | Reject unless specially proven | Reject unless specially proven |

## 8. Pass by value versus pass by reference — and how Atto compares

**"Pass by value" is not the same as "deep copy." "Pass by reference" is not the same as "shared ownership."** Before comparing languages, separate the **parameter binding**, **object identity**, **permission to mutate**, and **responsibility for lifetime/reclamation**.

When a function is called, there are several possible semantics:

| Concept | What it means | Does the caller still own the original? | Does the callee reach the same object? |
| --- | --- | --- | --- |
| Pass a scalar **by value** | Initialize an independent parameter with the scalar value | Yes | Not applicable |
| Pass a **copy of a reference/pointer value** | Callee has its own handle whose target may be shared | Yes, in ordinary Go/Java usage | Yes |
| Pass an object **by reference/borrow** | Give a time-limited view/alias with specified read/write rights | Yes, provided it remains alive | Yes |
| **Move ownership** | Transfer the resource's ownership right; former owner is invalid | No | Yes, unless a cross-domain relocation is required |
| **Shallow copy** | New root/outer storage, child references copied only when legal | Yes | New root, possibly shared children |
| **Deep copy** | New mutable reachable graph preserving internal cycles/repeated aliases | Yes | No mutable reachable source-object alias |
| **Share frozen ownership** | Additional lasting owner of the same immutable allocation | No exclusive writable owner; shared owners remain | Yes |

An ABI may implement value parameters with pointers to hidden storage. Conversely, copying a pointer into a parameter is still **pass by value of the pointer**. What matters to the programmer is the **source-level observable contract**, not whether one register contains a machine address.

### The AttoLang default

Primitive/value-copy scalars (`int`, `bool`, `float`) are passed as values. **Ordinary nonprimitive object arguments are implicitly borrowed** in the intended language profile: `fnc read(Counter c)` is a non-owning parameter; `read(counter)` is a read loan. Mutating code must request an exclusive loan. An ordinary borrow does not copy the graph, allocate a new owner, change ownership, or necessarily expose a raw address.

~~~atto
// TARGET semantics: implementation varies by backend and migration profile.
define class Counter as
  pub let int value = 1;
end

fnc inspect(Counter x): int {
  return x.value;                     // read-only borrowed parameter
}

fnc increment(mut Counter x): void {
  x.value += 1;                       // exclusive borrowed parameter
}

pub fnc example(): int {
  val mut Counter original = new Counter();
  val before = inspect(original);     // implicit shared loan, original remains owner
  increment(mut original);            // explicit exclusive call loan

  val independent = copy!! original;  // independent clone if DeepCopyable is proven
  val transferred = take! original;   // move unique ownership
  // inspect(original);              // REJECT: use after take!
  return before + inspect(transferred) + inspect(independent);
}
~~~

A **consuming** function needs an explicit owned-formal contract (candidate syntax `take! Counter x`) and the caller writes `take! original`. The compiler cannot quietly treat `inspect(original)` as a move. The exact borrowed-default signature lowering is staged; historical compiler profiles may still consume a plain class formal and must undergo a versioned migration.

### Comparison matrix

| Language | Ordinary function argument | How a caller-visible mutation happens | Explicit pointer/reference syntax | Memory and isolation model |
| --- | --- | --- | --- | --- |
| **AttoLang (target)** | Scalars copy; nonprimitive objects **borrow by default** | `f(mut obj)` / `borrow! mut obj` demands exclusive access; not a new owner | No unary `&x` or `*p`; explicit ownership verbs and proposed checked `Pointer<T>` | Static ownership/loan rules, domain-aware actors, explicit `take!`/`copy!!`/frozen `share!` |
| **Go** | **Always by value**, including pointer, slice, map and interface values | Copy a pointer or reference-bearing descriptor to the same storage | `&x`, `*p`, `*T`; safe Go has no pointer arithmetic | GC, separate synchronization, no general borrow checker |
| **Erlang** | Immutable **term/value** is passed; no mutable caller-variable alias | Produce a new term and return it; mutate process-local *state* by receiving messages | No C-style raw pointer syntax; Erlang `reference()` is an opaque unique ID, **not an address** | Single-assignment terms, process isolation, copying messages with runtime sharing optimizations |
| **Rust** | `T` argument is copied if `Copy` or generally **moved** otherwise; `&T`/`&mut T` borrow | `&mut x` gives exclusive mutable borrow | `&x`, `&mut x`, `*r`, `Box<T>`, raw `*const T` | Compile-time ownership, lifetimes, `Send`/`Sync`, explicit safe-vs-unsafe boundaries |
| **Java** | **Always by value**, including copies of object-reference values | Callee and caller reference the same mutable object | No ordinary C-style `T*` or unary `&x`/`*p` | GC-managed object references, runtime/thread synchronization; no static unique-ownership checker |
| **C++** | `T` value copied/moved, `T&` reference aliases, `T*` passes pointer value | `T&` or `T*` reaches caller object | `&x`, `*p`, `T*`, `T&`, `->`, pointer arithmetic | RAII, explicit moves, smart pointers; unchecked raw pointers and data races remain dangerous |

### Go: pass by value even when the value points elsewhere

Go **always passes an argument value**. A `*T` parameter receives a *copy of a pointer*; both copies can point to the same object. A `[]T` slice parameter receives a *copy of a slice descriptor*; both descriptors can refer to the same backing array. Maps similarly share underlying map storage.

~~~go
type Counter struct { N int }

func byValue(c Counter) { c.N++ }      // modifies a copy
func byPointer(c *Counter) { c.N++ }   // modifies the original pointee
func changeFirst(xs []int) { xs[0] = 42 } // can mutate shared backing array

func example() {
    c := Counter{N: 1}
    byValue(c)                       // c.N still 1
    byPointer(&c)                    // c.N now 2
}
~~~

A Go struct copy can still be **shallow** if its fields contain pointers, slices, maps or other reference-bearing values. A copied slice header's length/capacity are local, but its elements may be shared; `append` may or may not allocate a new backing array. Go can automatically dereference pointers for field selection (`p.Field`), so it is inaccurate to say Go always requires `*` at an object use. Safe Go forbids ordinary pointer arithmetic, while `unsafe` permits special low-level operations. There is no general compiler-enforced shared-read/exclusive-write loan model comparable to Atto's target.

**Atto contrast:** `inspect(obj)` expresses a checked temporary readonly loan, `increment(mut obj)` an exclusive loan, and `copy!! obj` the explicit independent mutable graph. None is merely "pass a pointer and hope other aliases behave."

Sources: [Go FAQ](https://go.dev/doc/faq), [language specification](https://go.dev/ref/spec), [Go slices and backing arrays](https://go.dev/blog/slices).

### Erlang: immutability instead of borrowing mutable object references

Erlang variables are **single-assignment**, ordinary terms are immutable, and changing a structure yields a new structure instead of editing the caller's original.

~~~erlang
increment(Map) ->
    Map#{count := maps:get(count, Map) + 1}.

example() ->
    Original = #{count => 1},
    Changed = increment(Original),
    {maps:get(count, Original), maps:get(count, Changed)}.
    %% {1, 2}
~~~

Erlang processes hold their own changing *process state* (usually by tail-recursively receiving messages and using the returned new state), but **cannot mutate each other's ordinary terms**. Messages generally cross process heaps by copying terms; refcounted large binaries and other immutable runtime data have sharing optimizations. Therefore, "pass by value" in Erlang must not be misconstrued as a mandatory eager **deep physical copy on every function call**. An Erlang `reference()` created by `make_ref()` is a unique opaque *term identifier*, not a dereferenceable memory pointer. ETS and NIFs are separate interoperability/storage mechanisms.

**Atto contrast:** Atto follows the spirit of isolated actor state but also permits **mutable uniquely owned graphs**. Those graphs require tracked temporary borrows, explicit `take!` ownership transfer, and independent `copy!!` when multiple actors need separately mutable messages. Zero-copy same-domain transfer is possible only with the required ownership and domain proof.

Sources: [Erlang expressions and single assignment](https://www.erlang.org/docs/26/reference_manual/expressions.html), [Erlang message-passing efficiency](https://www.erlang.org/docs/26/efficiency_guide/processes.html).

### Rust: `T`, `&T` and `&mut T` have different ownership contracts

~~~rust
struct Counter { n: i32 }

fn inspect(c: &Counter) -> i32 { c.n }      // shared borrow
fn increment(c: &mut Counter) { c.n += 1; } // exclusive mutable borrow
fn consume(c: Counter) { let _owned = c; }  // ownership received

fn example() {
    let mut c = Counter { n: 1 };
    inspect(&c);
    increment(&mut c);
    consume(c);     // move; Counter is not Copy
    // inspect(&c); // rejected: moved value
}
~~~

In Rust, a non-`Copy` `T` passed to a `T` formal generally **moves ownership**. `Copy` duplicates appropriate value types, while `Clone` is a type-defined explicit operation that **does not guarantee a fully independent graph** (for example, cloning an `Arc<T>` can create another owner of the *same* allocation). Rust references (`&T`, `&mut T`) are values passed to functions, with lifetimes and exclusivity statically checked. Rust also uses **automatic borrowing/dereferencing in many method-call contexts**; it is not merely manual raw-pointer programming.

**Atto contrast:** make the read loan implicit in `inspect(c)`, request exclusive access through `increment(mut c)`, and require an explicit `take! c` at an owned handoff. Atto's `copy!!` promises deep mutable graph separation where proven, unlike the unconstrained meaning of an arbitrary Rust `Clone` implementation.

Sources: [Rust ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html), [Rust references and borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).

### Java: copied reference value, same object

A Java object-reference argument is copied into a new local parameter variable. The object is **not** copied. The callee can mutate the original object through that reference, but **reassigning the parameter** does not reassign the caller's variable.

~~~java
class Counter { int n = 1; }

static void modify(Counter c) {
    c.n = 2;                // caller sees this field change
    c = new Counter();      // does NOT rebind caller's variable
    c.n = 100;
}

static void example() {
    Counter original = new Counter();
    modify(original);
    // original.n is 2, not 100
}
~~~

Thus Java is **pass-by-value of a reference**, *not* C++-style pass-by-reference of the caller's variable. GC handles object lifetime; it is **not** a borrow checker. `final Counter c` prevents reassigning `c`, not mutating the reachable object graph or precluding other mutable aliases.

**Atto contrast:** Atto may lower a borrowed parameter to a JVM reference internally, but must also track permissions and owner lifetime; it cannot allow arbitrary mutation/escape through a readonly loan. Java/GraalVM's memory model and garbage collector are implementation foundations, not substitutes for Atto's language-level safety checks.

Sources: [Oracle tutorial on argument passing](https://docs.oracle.com/javase/tutorial/java/javaOO/arguments.html), [JLS `4 parameter variables](https://docs.oracle.com/javase/specs/jls/se25/html/jls-4.html).

### C++: by-value, reference and pointer are *three* parameter forms

~~~cpp
struct Counter { int n = 1; };

void by_value(Counter c)    { ++c.n; }       // independent value
void by_ref(Counter& c)     { ++c.n; }       // caller object
void by_ptr(Counter* c)     { ++c->n; }      // pointer copied; caller object
void read(const Counter& c) { (void)c.n; }   // readonly through this alias

void example() {
    Counter c;
    by_value(c);  // still 1
    by_ref(c);    // now 2
    by_ptr(&c);   // now 3
}
~~~

`T&` is a language reference alias; `T*` is a separate pointer *value*. A `const T&` restricts mutation via that alias but does not prove nobody else is writing to the same object. C++ copy construction performs memberwise copies according to member type; copying a raw pointer member duplicates its address, **not the pointed-to allocation**. `std::move` makes a value eligible for a move-aware operation rather than automatically physically moving data; `std::unique_ptr`, `std::shared_ptr` and RAII provide important safe idioms. Nevertheless, dangling raw pointers, invalid manual lifetime assumptions and data races can cause undefined behavior.

**Atto contrast:** Atto aims to replace a routine selection among `const T&`, `T&`, `T*` and move/sharing wrappers with ordinary borrowed parameters plus explicit `mut`, `take!` and `share!` operations. **Atto `copy!` additionally rejects unsafe shallow nested ownership duplication**, while `copy!!` has a separate proven graph-duplication contract.

Sources: [C++ references](https://en.cppreference.com/cpp/language/reference), [pointers](https://en.cppreference.com/cpp/language/pointer), [copy constructors](https://en.cppreference.com/cpp/language/copy_constructor).

### Which Atto operation do you actually want?

| Intent | Target AttoLang spelling | Key guarantee |
| --- | --- | --- |
| Read object without taking ownership | `inspect(obj)` | Non-owning inferred readonly loan |
| Make borrow boundary visible | `inspect(borrow! obj)` | Same rights as an inferred loan |
| Temporarily mutate object | `update(mut obj)` | Exclusive non-owning loan; caller remains owner |
| Transfer unique ownership | `consume(take! obj)` with a consuming formal | Prior owner's access invalidated |
| Duplicate only immediate fields safely | `copy! obj` | Fresh root, nested aliases only when provably legal |
| Create a separate mutable reachable graph | `copy!! obj` | New reachable mutable nodes, cycles/repeated aliases preserved inside clone |
| Add an owner of the same frozen allocation | `share! obj` | Same immutable graph, no unchecked mutable sharing |
| Typed low-level memory control | `Pointer<T>` `read_at` / `write_at` (proposal) | Checked provenance, bounds, generation and region lifetime |

**Key principle:** the way a parameter is *represented* (address, register or handle), whether its referent is *shared*, and whether the callee may *read, mutate, retain, drop or move* it are separate questions. Atto's explicit ownership vocabulary and default loans address the last two without requiring source-level unary `&`/`*` for every ordinary object operation.

## 9. Other explicit lifecycle operations

AttoLang's expanded ownership vocabulary gives important supporting controls without C/C++/Rust pointer punctuation:

| Operation | Intended meaning and restrictions |
| --- | --- |
| `freeze! take! value` | Consume unique write authority to produce a transitively immutable uniquely owned value; no live mutating alias |
| `weak! shared` | Create a **non-owning** weak handle; an authenticated, atomic upgrade returns `Option<Shared<T>>` and checks domain/generation/liveness |
| `pin! take! value` | Require a stable **pointee storage address**, not a fixed OS thread; cannot relocate across domains/allocators without a separately proved stable-address handoff |
| `drop! value` | Consuming statement that releases/invalidates owner exactly once, when no incompatible loan exists; cleanup on controlled exits must be proved |
| `lt! a` (alias `lifetime! a`) | Declare a named compile-time lifetime binder; reference it as `lt=a` in a named generic argument |
| `ref! owner` | In the bounded checked-handle compiler profile, consume the class owner into a compiler-minted `CheckedRef<Class>` token |
| `deref! handle` | Borrow a checked-handle referent as a scoped **readonly** view, without exposing a raw machine address |
| `unref! handle` | Consume the checked handle and restore an owned class value once all scoped views end |

For `weak!`, observing a stale generation, retired region or foreign domain must not resurrect a resource. For `pin!`, moving a handle does not imply moving its pointee, but *actor handoff, task capture or arena migration* needs an authenticated stable-address proof (`E_PIN_BOUNDARY_TRANSFER` if absent). A pin **alone** does not prove safe borrows across `await`. For `drop!`, a backend lacking authenticated destructor/cleanup lowering must reject resource-owning drops rather than report success for a no-op; fatal process termination cannot promise language-level destructors.

### Checked-handle example (bounded compiler profile)

~~~atto
define class Parcel as
  pub let int value = 42;
end

pub fnc main(): int {
  val original = new Parcel();
  val handle = ref! original;

  block {
    val view = deref! handle;
    stdio.println(view.value);
  }

  val restored = unref! handle;
  return restored.value;
}
~~~

This is **not** general-purpose `Pointer<T>` support. The current scoped checked-handle design is for local class owners in one worker with limited control flow, no mutable dereference, no arbitrary raw addresses, and no cross-actor/foreign transfer. In that profile, use the explicit `block { ... }` scope, not a bare statement-position `{ ... }`. Full generation reuse, actor-domain enforcement, exception cleanup and all-backend support require independent implementation.

## 10. Lifetimes: usually inferred, explicit when relationships matter

AttoLang deliberately does **not** import Rust's apostrophe lifetime syntax. A named lifetime binder uses `lt! name`, and the long spelling `lifetime! name` formats to `lt! name`. References use a **named** lifetime argument, e.g. `Borrow<Child, lt=a>`.

~~~atto
// Target signatures; not guaranteed in all current backends.
fnc child_of(Parent parent): Borrow<Child> {
  return parent.child; // only if provenance proves the child's owner
}

// Multiple possible origins require an explicit output relationship.
fnc choose_left<lt! a, lt! b>(
  Borrow<Foo, lt=a> left,
  Borrow<Foo, lt=b> right
): Borrow<Foo, lt=a> {
  return left;
}
~~~

A plain `fnc read(Foo foo)` has an inferred loan scoped to the caller's actual owner. A `Borrow<Child>` result may elide its lifetime **only when one unambiguous input or `self` owns the child**. A result with multiple possible owners needs an explicit provenance/outlives contract. Borrowed data cannot be converted into a returned **owned** value just because field and result types look alike.

The checker handles lexical scopes, last uses, reborrowing, nested calls, closures/captures, field projections, branches/joins, loops/backedges, exceptional exits, and cleanup. It must conservatively reject uncertain aliases, unconstrained public signatures, unknown dynamic effects, actor escapes, and missing source-to-IR provenance. Neither annotating a lifetime nor writing `borrow!` manufactures ownership.

## 11. Async, actors and sealed state

Loans are especially dangerous when execution may be suspended, resumed elsewhere or outlive the owner:

- **`await` / `select` / task suspension:** reject live borrowed class/Box views unless there is a separately verified pinned continuation/frame, valid owner outlives relation and cancellation cleanup. In initial profiles, a blanket rejection is safer than an unproven runtime workaround.
- **Actor state:** state is sealed. Access belongs within `handle_state(critical-read |s| -> ...)` or `handle_state(critical-write |s| -> ...)`. Read access admits parallel readers; write is exclusive. No escaped state loan, unknown reentrant call or unproven suspension inside a guard.
- **Actor messages:** choose `take!` for verified unique `Sendable` transfer, `copy!!` for verified independent `DeepCopyable + Sendable` graphs, or explicitly authorized deeply immutable sharing. An ordinary `copy!` or `borrow!` does not establish an asynchronous mail permission. Scoped cross-actor read/write lease APIs require their own bounded lifetime protocol; their conceptual existence does not authorize arbitrary send of a borrow.
- **Failures:** account for checked return, `throw`, `raise`, `panic`, `defer`/`recover`, cancellation and supervisor retirement. Exactly-once controlled cleanup must be proved. An actor panic escalates through supervisor CONTROL; it does not let an actor bypass memory/ownership obligations. Hard aborts and fatal runtime traps have different guarantees.
- **Isolation:** `Sendable` concerns safe in-process/approved-domain transfer; `Serializable` concerns wire encoding. They are distinct. Arbitrary `Pointer<T>`, raw FFI addresses and actor-private arena references are not messages.

For Mailman delivery, the source-level operation and its runtime ticket must agree. Runtime envelopes carry authenticated actor incarnation, domain/generation, type/layout and transitive graph authorization; mailbox commit is linearizable. Failed admission rolls back private copies/escrow without exposing a partial graph; committed receiver ownership cannot be reclaimed by the sender. The proposed READ_LEASE/WRITE_LEASE modes need a protocol beyond local `borrow!`.

## 12. Why AttoLang does **not** require unary `*` and `&` for ordinary memory access

In Go and C/C++, `&x` commonly takes an address and `*p` dereferences one. Rust uses `&x` and `&mut x` to form references and `*r` for dereference. The same short glyphs sit beside very different concerns: raw storage addresses, temporary borrows, mutability, references, pointer indirection and lifetime rules. In C/C++, pointer arithmetic and unchecked dereference additionally expose undefined-behavior hazards.

**AttoLang's design choice is to spell out the *intent and authority*, not make programmers continually annotate where a compiler stores data.** Most object access is ordinary `obj.field` and `obj.method()`; most nonprimitive parameters borrow by default. When a program really does something exceptional, `borrow!`, `take!`, `share!`, `copy!`, `copy!!`, `pin!` and the checked-handle operations say *what* must happen, so a reviewer can distinguish borrowing from sharing, moving and copying at a glance.

~~~cpp
// C++-style syntax: pointer-shaped syntax is exposed at the use site.
int x = 1;
int* p = &x;
*p = 42;
~~~

~~~atto
// AttoLang target: no raw pointer syntax is needed for ordinary mutation.
val mut Foo item = new Foo();
adjust(mut item);             // exclusive checked call loan
val detached = copy!! item;   // an independent owned copy if proven
~~~

**This does not mean AttoLang forbids multiplication or bitwise operations.** Binary `*` remains multiplication and binary `&` remains integer bitwise AND in the accepted operator design. What AttoLang excludes is **unary `&x` as source-level address-of and unary `*p` as unguarded pointer dereference**. Other punctuation such as `->` remains available for callable/lambda semantics, not C-style pointer member access.

### How low-level control still works

**Typed, checked memory capabilities** provide control without naked addresses:

- **`Pointer<T>` (candidate builtin):** typed `Pointer.alloc<T>(count)`, `p.read_at(i)`, `p.write_at(i, value)`, and `p.close()`. Require correct element type/layout, generation, actor/tenant/region identity, bounds, initialization, alignment, rights, revocation and checked offset arithmetic. A missing pointer is `Option<Pointer<T>>`, not a forged null address. Exact allocation failure and method result types still require finalization.
- **Explicit ownership operators:** transfer with `take!`, freeze/share with `share!`, scoped reads/writes with `borrow!` and `mut`, duplicate with `copy!`/`copy!!`, stabilize storage with `pin!`, and retire with `drop!`. None is an unchecked cast.
- **`ref!` / `deref!` / `unref!`:** compiler-minted, lifecycle-checked handles for an admitted *narrow* profile; these are not a general foreign pointer system.
- **Layout and allocation control:** checked arrays/slices, value aggregates, named AoS/SoA/AoSoA layout parameters where supported, actor-owned arenas, bounded pools, aligned buffers and closed-world typed codegen can deliver contiguous, predictable low-level data access. Safety comes from static representation/provenance and runtime guards where necessary, **not** from an address-of glyph.
- **Foreign libraries / device memory:** a privileged, typed, manifest-authorized FFI boundary can lower to real machine addresses internally. Its adapter owns the ABI, pinned lifetime, unsafe host interaction and cleanup proof. Guest `Pointer<T>` must not be implicitly convertible from integers, serialized addresses or an arbitrary FFI result.
- **Native and AOT performance:** compiler-generated direct field loads/stores, optimized scalar copies, statically specialized pointer methods, SIMD and proven zero-copy same-domain transfers do not need user-level `*`/`&` syntax. Bounds/ownership guards may be optimized only when their safety proof is preserved.

Conceptual low-level example (**proposed API**, not a claim of executable pointer support):

~~~atto
pub fnc example(): Result<int, PointerError> {
  let cells: Pointer<int> = Pointer.alloc<int>(4)?;
  cells.write_at(0, 42)?;
  val result: int = cells.read_at(0)?;
  cells.close()?;
  return Ok(result);
}
~~~

The compiler/ABI can use native machine pointers, Wasm linear-memory offsets, or checked handle tables **behind** the abstraction. That implementation freedom is an advantage: users express the same safe semantics without repeatedly writing address-of/dereference punctuation or accidentally relying on a C layout. Where exact zero-overhead access is required, the compiler proves bounds and lifetime or rejects an unsupported profile instead of offering unchecked undefined behavior. Pointer operations are **not** automatically free of runtime cost; a hard-real-time target must bound that cost.

## 13. Compiler enforcement, diagnostics and proof gates

A conforming compiler should:

1. Resolve overload/generic/call effects and ownership modes; reject unknown provenance or effect summaries rather than infer unique rights from type equality.
2. Build typed place/owner/loan facts in HIR/MIR; preserve those facts through borrow-by-default sugar, `Borrow<T>` explicit form, branch merges, loops, exception/cleanup edges and generated AOT calls.
3. Enforce move/drop-after-loan, use-after-move, shared/exclusive conflicts, foreign/domain/actor restrictions, and nonescaping guards before emitting a guest artifact.
4. Check distinct `Copyable` / `DeepCopyable` / `Sendable` witnesses and deep copy context/rollback; preserve source-order once-only evaluation, concrete dynamic types and inherited/private fields.
5. Emit accurate physical code and runtime checks for bounds, initialization, alignment, generation, authenticated domain permissions, mailbox commit/abort and exactly-once reclamation.
6. **Fail closed** when a backend cannot lower a feature. An accepted lexer token, a source-model test or a Rust host oracle does **not** establish compiled AttoLang guest execution.

Examples of useful diagnostic families include `E_BORROW`, `E_BORROW_MUT_REQUIRED`, `E_BORROW_AMBIGUOUS`, `E_MOVED`, `E_COPY_SHALLOW_ALIAS`, `E_COPY_GRAPH_UNPROVEN`, `E_COPY_AOT_WITNESS`, `E_COPY_ACTOR_ISOLATION`, `E_SHARED_MUTATION`, `E_SHARE_ALIAS`, `E_GUARD_ESCAPE`, `E_GUARD_SUSPEND`, `E_PIN_BOUNDARY_TRANSFER` and appropriate unsupported-backend diagnostics. Diagnostic codes in draft contracts are **targets**; exact current implementation mappings require compiler-specific verification.

**Minimum regression suite:** shared-read/read pass; shared-read/write fail; use-after-`take!` fail; moving/dropping with a live loan fail; implicit `f(x)` and explicit `f(borrow! x)` equivalence; `f(mut x)` exclusive effect; dangling returned loan fail; escaping closure/`await`/`select` fail; unsafe `copy!` child alias fail; `copy!!` repeated child/cycle, inherited subtype and independent recipient graphs pass *only if supported*; partial-copy rollback; frozen shared write fail; weak stale-generation upgrade fail; pin relocation fail; bad pointer bounds/ABI/domain fail; mailbox rejection preserves unique ownership; cross-actor mutable alias fail; exactly-once controlled cleanup.

For each backend (Rust/browser, independent Wasm, Java/GraalVM, LLVM, .NET Native AOT, Forge, and others), publish exact source/head/tree SHA, compiler/runtime artifact hashes, target/toolchain, **actual compiled guest** positive and negative execution, no-artifact rejection evidence, race/sanitizer or equivalent runtime gates, and separate `PASS` / `FAIL` / `UNSUPPORTED` / `NOT_EXECUTED` results. A GitHub Actions run with zero executed steps is `NOT_EXECUTED`, not a passing build. Full graph copying, general `Pointer<T>`, cross-domain leases, and five-plus-backend parity **must not be represented as implemented merely because a feature branch merges**.

## 14. Specification and implementation reading list

This guide intentionally reconciles the **heads of pending feature work** with the merged specification rather than relying on `main` alone. Relevant sources and scopes:

- [Ownership vocabulary, including pinned handoff amendments — `atto-spec` PR #94](https://github.com/atto-lang/atto-spec/pull/94), on top of [ownership vocabulary PR #92](https://github.com/atto-lang/atto-spec/pull/92).
- [Implicit borrowing of nonprimitive arguments — `atto-spec` PR #89](https://github.com/atto-lang/atto-spec/pull/89); [local name/referent mutability](https://github.com/atto-lang/atto-spec/blob/main/language/local-binding-mutability-v1.md).
- [Frozen `share!` and explicitly guarded mutation — `atto-spec` PR #90](https://github.com/atto-lang/atto-spec/pull/90).
- [Shallow `copy!` vs deep `copy!!`, fresh roots, inherited dispatch and actor fan-out — `atto-spec` PR #39](https://github.com/atto-lang/atto-spec/pull/39). Its later revisions supersede early deep-default text.
- [Ownership/Box/Arc/suspension proposal — `atto-spec` PR #27](https://github.com/atto-lang/atto-spec/pull/27); [Mailman ownership protocol — PR #64](https://github.com/atto-lang/atto-spec/pull/64); [critical state leases — PR #29](https://github.com/atto-lang/atto-spec/pull/29).
- [`Pointer<T>` safe AOT contract — `atto-spec` PR #18](https://github.com/atto-lang/atto-spec/pull/18); [primitive/object storage proposal — PR #91](https://github.com/atto-lang/atto-spec/pull/91); [foreign ABI authority proposal — PR #54](https://github.com/atto-lang/atto-spec/pull/54).
- [Rust compiler checked handles `ref!`/`deref!`/`unref!` — `atto-source.rs` PR #264](https://github.com/atto-lang/atto-source.rs/pull/264); [bounded AOT flat-class deep copies — PR #265](https://github.com/atto-lang/atto-source.rs/pull/265); [expanded ownership parser/checker work — PR #344](https://github.com/atto-lang/atto-source.rs/pull/344).
- [Independent ownership capability model — `atto-borrow-checker.rs` PR #43](https://github.com/atto-lang/atto-borrow-checker.rs/pull/43); [cross-backend conformance repository](https://github.com/atto-lang/atto-platform-conformance).

**Bottom line:** AttoLang does not throw away low-level control. It replaces repetitive, ambiguous pointer punctuation with explicit ownership actions, typed checked capabilities, proven lifetimes and backend freedom to use the most efficient safe representation.
