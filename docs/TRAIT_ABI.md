# Trait ABI

The trait ABI lets R (and other packages) call Rust trait methods without knowing the concrete Rust type at compile time. It does this by storing a tiny "header + vtable" next to the object and using R external pointers to carry it around.

## R names on trait implementations

An annotated implementation inherits `#[miniextendr(r_name = "...")]` and
`#[miniextendr(skip)]` from the trait declaration. This also applies to impls
with method bodies, generic traits and traits with associated types. An
explicit `r_name` on an impl method overrides the inherited R name; a method
skipped by the declaration stays skipped. Other method options, docs and
parameter attributes belong to explicit implementation methods. An omitted
default method, of a generic trait or not, uses its declaration's docs and
parameter names. An implementation parameter written as a pattern (`_`, a
tuple) takes the declaration's name in its R formal; a named one keeps its own.

The declaration supplies this metadata through a hidden macro re-exported
under the trait's name, in Rust's macro namespace. Implementers do not need
to import or invoke that macro separately. The defining module path, a
crate-root re-export and an imported or renamed trait all work. Renaming an
import keeps the original trait's R namespace and ABI tag. The trait's name
is taken in the macro namespace as well, so a macro or derive of the same name
can't sit in the trait's module. `#[miniextendr]` on an impl of a trait without
`#[miniextendr]` fails with "cannot find macro" at the trait's name. An empty
impl, `#[miniextendr] impl Trait for T {}`, registers the type's own Rust impl
of the trait, default methods included. It works for any non-generic trait
without associated types, one defined in the implementing crate as well.

The declaration supplies the exact `repr(C)` vtable field order and callback
types. An impl emits its own layout with those fields and registers it under
the declaration's fingerprint, so it never guesses paths to `TAG_*`, a
vtable type or a builder next to the imported name. Non-generic shims are
hidden associated methods with `Self: Sized`: they retain declaration-local
type resolution and default methods without preventing `dyn Trait`. Generic
traits retain concrete shims. Required methods supply their signatures in the
impl; omitted defaults supply theirs through the declaration metadata, with
the impl's concrete type/const arguments substituted. Omitted generic arguments
use the declaration's defaults, and `Self::Item` resolves through the concrete
trait impl. Both ABI callbacks and R wrappers call the original Rust default;
no default body is copied into the impl. Explicit overrides keep their own
signature, body and wrapper options. Static defaults get R wrappers too.
The exported vtable static symbols used by webR remain unchanged.

Inherited defaults and empty annotated impls infer argument types from the
concrete trait method. Private declaration-local aliases therefore need no
imports in the impl module, even across crates or when that module defines a
conflicting alias. The copied signature still describes the R-facing shape and
picks the conversion path (strict and coercing reads, rooted borrows);
only the type each conversion produces comes from the inferred concrete type,
native-borrow metadata included. The signature helper neither calls the method
nor constructs argument values, and adds no conversion bounds to the trait
itself.

A trait method must be safe unless the declaration marks it
`#[miniextendr(skip)]`: a skipped method gets no vtable slot or R wrapper, so
it may be `unsafe`, and an annotated impl inherits the `skip`. The declaration
rejects any other `unsafe fn` (see
[`unsafe fn`](MINIEXTENDR_ATTRIBUTE.md#unsafe-fn)), so no vtable shim,
inherited-default wrapper or empty impl ever calls one.

Argument conversion failures from either shim path carry `kind = "conversion"`,
`param` (the declaration's parameter name), and `rust_type`. Renamed or ignored
bindings in a generic implementation do not change those parameter names or
native-alias diagnostics. Custom `RConditionData` messages, classes and fields
survive the View round trip; the outer wrapper supplies the consumer's call.
The shim uses the `conversion_error_class` configuration of the crate where it
is generated (the declaration crate for non-generic traits, the implementation
crate for generic traits).

## Key Concepts

### Type Tags (`mx_tag`)

A 128-bit identifier used to query an object's interfaces. Generated trait tags
hash a versioned signature description: the trait name and every exported
instance method in declaration order, including its Rust and R names, receiver
convention, argument types and return type. Adding, removing or reordering a
method changes the tag, as does changing a signature. An incompatible consumer
gets a failed query before it can read a vtable slot. Old name-only tags also
fail to match; rebuild both packages after upgrading the ABI generator.

Package and module paths are excluded so independently compiled declarations of
the same interface agree. Docs, argument variable names, default bodies, static
methods and skipped methods do not affect the tag. Types are compared by their
parsed token spelling, independent of source whitespace. This is a syntactic
interface contract, not Rust type identity: packages must use the same spelling
and agree on what imported names, aliases and associated types mean. An alias
and its expanded type can conservatively produce different tags.

Concrete type tags are separate from these trait interface tags.

### Erased Header (`mx_erased`)

A tiny header that points to a base vtable. This is the common prefix of all type-erased objects:

```c
typedef struct mx_erased {
    const mx_base_vtable *base;
} mx_erased;
```

### Base Vtable (`mx_base_vtable`)

Every erased object has a base vtable with:
- `drop`: Destructor for cleanup when R garbage collects the object
- `concrete_tag`: Type tag for downcasting to the concrete type
- `query`: Function that returns trait vtables by tag

```c
typedef struct mx_base_vtable {
    void (*drop)(mx_erased *ptr);
    mx_tag concrete_tag;
    const void *(*query)(mx_erased *ptr, mx_tag trait_tag);
} mx_base_vtable;
```

### Trait Vtables

Each trait gets its own vtable with function pointers for each method. Ordinary
receivers use `mx_meth`; explicit `ExternalPtrRef` / `ExternalPtrMut` receivers
use `mx_owner_meth`, whose owner must stay rooted for the entire call:

```c
typedef SEXP (*mx_meth)(void *data, int argc, const SEXP *argv);
typedef SEXP (*mx_owner_meth)(void *data, SEXP owner, int argc, const SEXP *argv);
```

## How It Works

### 1. Compile Time

The `#[miniextendr]` macro on a trait generates:
- Tag constant (`TAG_Counter`)
- Vtable struct (`CounterVTable`)
- View struct (`CounterView`) for calling methods
- Method shims that convert between R and Rust types

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}
```

### 2. Object Creation

When you create an object for trait dispatch, a wrapper struct is allocated with `mx_erased` at the front and the real Rust data after it:

```rust
#[repr(C)]
struct __MxWrapperMyCounter {
    erased: mx_erased,  // Must be first
    data: RefCell<MyCounter>,
}
```

The data pointer a View carries points at that `RefCell`. Only the producer's
shims look behind it; a View passes it through unread.

### 3. Packaging with `mx_wrap`

`mx_wrap` turns `*mut mx_erased` into an R external pointer (`EXTPTRSXP`). The finalizer uses the base vtable's `drop` function to clean up when garbage collected.

```rust
// In a constructor
let wrapper = Box::new(__MxWrapperMyCounter {
    erased: mx_erased { base: &__MX_BASE_VTABLE_MyCounter },
    data: RefCell::new(my_counter),
});
let sexp = unsafe { mx_wrap(Box::into_raw(wrapper) as *mut mx_erased) };
```

A derived type with `Sidecar<T>` fields also exposes an internal sidecar
initialization interface through its base `query`. After `mx_wrap` roots the
external pointer and registers its finalizer, it calls this interface to attach
a `VECSXP` protection list and flush pending fields into it. The layout matches
ordinary `ExternalPtr` slots: type ID, user slot, then sidecars in field order.
Types without sidecars allocate no protection list. Pending values are rooted
from `Sidecar::new` until they sit in a slot; root `mx_wrap`'s result before
allocating.

This storage remains `ErasedObject<T>`, not ordinary `ExternalPtr<T>`/`Any`
storage. A trait method needing its sidecar owner takes
`self: ExternalPtrRef<'_, Self>` or `self: ExternalPtrMut<'_, Self>` on a trait
bounded by `TypedExternal`. Its shim constructs a borrowed handle from the
checked data pointer and caller-rooted owner after argument alias preflight.
Generated sidecar accessors accept the borrowed handles. Plain `&T` methods
still carry no owner implicitly.

The View passes its retained owner only to these methods. A raw Rust-owned
View constructed with `from_raw_parts(..., None)` refuses an owner-receiver
method before dispatch. Traits containing owner receivers have distinct tags
from otherwise same-named plain-receiver declarations, since their vtable
fields have different signatures. The base-vtable layout does not change.

### 4. Dispatch with `mx_query`

When you want to call a trait method, `mx_query(sexp, TAG_<Trait>)` asks the object for the trait vtable:

```rust
// Get the Counter vtable for this object
let vtable = mx_query(sexp, TAG_Counter);
if vtable.is_null() {
    // Object doesn't implement Counter
}
```

### 5. Method Call

If the vtable exists, the generated shim converts R args, calls the Rust method, and converts the result back to R:

```rust
// CounterView holds the data and vtable pointers and roots `sexp`
let mut view = unsafe { CounterView::try_from_sexp(sexp) }?;
let value = view.value();  // Calls through vtable
view.increment();
```

### View lifetime

A View made by `try_from_sexp` roots its receiver until the View drops, in the
same pool that owning `ExternalPtr` handles use. A View can therefore outlive
the `.Call` argument or `PROTECT` that held the receiver when it was made, and
the object's finalizer cannot drop the Rust value while a View still points
at it. Views drop in any order. The pointers live in the runtime's private
`TraitViewData`, so a View can only be built through the unsafe
`TraitView::from_raw_parts`. `from_raw_parts(data, vtable, None)` roots
nothing; its caller must keep `data` alive while the View exists.

Rooting keeps the receiver alive. Borrows are the producer's: each shim
borrows the object's `RefCell` before it makes a reference, shared for `&self`
and exclusive for `&mut self` or an `ExternalPtrMut` receiver, and holds the
borrow until the result is converted. A call that conflicts with one still
running on the same object, whether it comes through a View, a generated
method or a field accessor, raises a `miniextendr_borrow_conflict` error. A
View never reads the data, so a consumer needs no part in this.

### Argument types

The View converts each argument to R with `IntoR`, and the shim reads it back
with `TryFromSexp`, so a trait-method parameter type needs both. `Missing<T>`
crosses as R's missing-argument sentinel when absent. The reading markers
(`AsNumeric`, `AsNumericVec`, `AsCharacter`, `AsCharacterVec`) cross as their
inner value, and `AsFromStr<T>` / `AsFromStrVec<T>` as their `Display` text.
The borrowed views `StrVec<'_>` and `RCow<'_, T>` have no `TryFromSexp`: the
shim binds each to a local copy of its argument slot, so the implementation
sees it for the call only, and a trait method cannot return one (#1427).
Parameter checks such as `no_na`, `inherits` and `not_inherits` run in the implementing
method's R-facing wrappers: they check R arguments, not the typed values a
View caller passes.

A `&Self` / `&mut Self` parameter is borrowed by the shim like the receiver,
after the check that refuses its aliasing the receiver or another such
parameter mutably, and the method gets a plain reference. An
`ExternalPtr<T>` parameter arrives as a handle the shim doesn't borrow; the
method reads it with `borrow()` / `borrow_mut()` (`EXTERNALPTR.md`,
"Arguments").

A trait method takes no `&RngGuard` (a compile error): the guard is no R value,
so it can't cross. Draw inside `with_rng(|rng| ...)` in the body (`RNG.md`,
"Trait Methods"; #2196).

### Failures

The shim is the only check a caller that skips the View gets, so it checks
its arguments itself. It first checks the argument count. Then, like a
generated C wrapper, it refuses arguments that borrow the same R vector when
at least one borrow is mutable, before it converts any of them. A wrong
count, an aliased borrow, a failed conversion and a panic in the method all
come back from the shim as a tagged condition. The View re-panics with it, so
a `#[miniextendr]` caller raises it as an R condition with its `rust_*`
classes. None of them raises a bare R error, so Rust values on either side of
the call are dropped normally.

## Multiple Traits Per Type

A single type can implement multiple traits. All are automatically registered via `#[miniextendr]`:

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}

#[miniextendr]
pub trait Resettable {
    fn reset(&mut self);
}

pub struct MyCounter { value: i32 }

#[miniextendr]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.value }
    fn increment(&mut self) { self.value += 1; }
}

#[miniextendr]
impl Resettable for MyCounter {
    fn reset(&mut self) { self.value = 0; }
}

// Registration is automatic via #[miniextendr].
```

The framework groups trait impls by concrete type. `MyCounter` gets a single wrapper with a query function that handles both traits:

```rust
unsafe extern "C" fn __mx_query_mycounter(
    _ptr: *mut mx_erased,
    trait_tag: mx_tag,
) -> *const c_void {
    if trait_tag == TAG_Counter {
        return &__VTABLE_mypkg_Counter_FOR_MyCounter as *const _;
    }
    if trait_tag == TAG_Resettable {
        return &__VTABLE_mypkg_Resettable_FOR_MyCounter as *const _;
    }
    std::ptr::null()
}
```

From R or consumer packages, both views work on the same object:

```rust
// Get Counter view
let mut counter_view = unsafe { CounterView::try_from_sexp(sexp) }?;
counter_view.increment();

// Get Resettable view from same object
let mut reset_view = unsafe { ResettableView::try_from_sexp(sexp) }?;
reset_view.reset();
```

## Cross-Package Usage

The trait ABI enables cross-package dispatch where:
- **Producer package**: Defines traits and concrete types
- **Consumer package**: Uses trait views without knowing concrete types

### Architecture

```text
Every package (rpkg, producer.pkg, consumer.pkg, ...)
────────────────────────────────────────────────────
mx_abi.rs (compiled into each package's staticlib via miniextendr-api)
  mx_abi_register()                  ← called by package_init() via miniextendr_init!
    init_tag()                       ← Rf_install("miniextendr::mx_erased")
    R_RegisterCCallable(...)         ← registers for cross-package use
  mx_wrap() / mx_get() / mx_query() ← linked directly via extern "C"
```

- Each package includes `mx_abi.rs` from miniextendr-api (compiled into the Rust staticlib)
- Rust calls `mx_wrap`/`mx_get`/`mx_query` directly (no `R_GetCCallable` indirection)
- Cross-package dispatch works because all packages share the same `Rf_install("miniextendr::mx_erased")` tag symbol

### Requirements

1. **Use `miniextendr_init!` in lib.rs**: This calls `package_init()` which includes `mx_abi_register()` automatically.

```rust
// lib.rs
miniextendr_api::miniextendr_init!(mypkg);
```

2. **Main thread only**: All trait ABI operations must happen on R's main thread (which is where `.Call` runs).

3. **Null checks**: If a type doesn't implement a trait, `mx_query` returns null. The generated `TraitView::try_from_sexp` handles this gracefully.

## Source Files

- `miniextendr-api/src/trait_abi.rs` - Core types and traits
- `miniextendr-api/src/trait_abi/view_data.rs` - `TraitViewData`, a View's pointers and receiver root
- `miniextendr-api/src/trait_abi/ccall.rs` - C-callable wrappers and init
- `miniextendr-api/src/abi.rs` - FFI-compatible struct definitions
- `miniextendr-api/src/mx_abi.rs` - Rust implementation of mx_wrap/mx_get/mx_query
- `rpkg/inst/include/mx_abi.h` - C header for consumer packages
