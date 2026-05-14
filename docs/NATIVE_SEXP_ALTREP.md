# Native-SEXP ALTREP Storage Guide

This guide explains the **native-SEXP storage pattern** for custom ALTREP
vectors — where the backing data lives directly in the ALTREP `data1` slot as
a plain R vector, rather than being wrapped in an `ExternalPtr`.

**See also**: [ALTREP.md](ALTREP.md) for the full ALTREP system documentation,
[ALTREP_QUICKREF.md](ALTREP_QUICKREF.md) for the quick reference.

---

## What Is the Native-SEXP Pattern?

Most ALTREP implementations in miniextendr store their backing data in an
`ExternalPtr` in `data1`.  The `AltrepExtract` blanket impl handles the
`ExternalPtr` → `&T` unwrapping automatically.

The native-SEXP pattern takes a different approach: instead of wrapping data in
an `ExternalPtr`, the `data1` slot holds a **plain R vector** (e.g., an
`INTSXP`) that contains the data directly.  This is exactly how R's own
built-in ALTREP classes work:

- **`compact_intseq`** stores `c(from, to, by)` as a `REALSXP` in `data1`
  and computes each element on access.
- **`deferred_string`** stores the original integer/real vector in `data1` and
  converts to strings lazily.

---

## When to Use This Pattern

Use native-SEXP storage when:

- **State fits naturally in an R vector** — e.g., a compact integer sequence
  parameterised by `from` / `to` / `by`, a dictionary encoding stored as an
  integer index vector, or a computed-on-access sequence where `data1` caches
  the materialised data.
- **You want `data2 = R_NilValue`** — because `data1` already IS the
  materialized representation.  When R requests `Dataptr`, the trampoline can
  forward `DATAPTR_RO(data1)` directly without a separate materialization step.
- **You prefer not to allocate a heap-pinned Rust object** — no `Box<T>`,
  no `ExternalPtr`, no finalizer.  GC tracks `data1` automatically through
  normal R reference counting.

---

## How `AltrepExtract` Enables This

`AltrepExtract` is the trait that tells the ALTREP trampolines how to extract
a `&Self` (or `&mut Self`) from an ALTREP SEXP.  The default blanket
implementation extracts via `ExternalPtr` downcast from `data1`.

Power users can override this trait to use a different storage strategy.  For
the native-SEXP pattern, the trick is to make `Self` a zero-sized type (ZST)
and return a static singleton:

```rust
pub struct NativeSexpIntAltrep;

static mut INSTANCE: NativeSexpIntAltrep = NativeSexpIntAltrep;

impl AltrepExtract for NativeSexpIntAltrep {
    unsafe fn altrep_extract_ref(_x: SEXP) -> &'static Self {
        // SAFETY: ZST — no data can be aliased.
        unsafe { &*std::ptr::addr_of!(INSTANCE) }
    }

    unsafe fn altrep_extract_mut(_x: SEXP) -> &'static mut Self {
        // SAFETY: ZST with no interior state.
        // `addr_of_mut!` avoids Stacked Borrows UB on mutable static refs.
        unsafe { std::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap_unchecked() }
    }
}
```

The singleton carries no data; all element-access methods read from `data1`
on the ALTREP SEXP (accessed via `x.altrep_data1_raw_unchecked()` in the
low-level trait methods).

---

## Implementation Overview

Because this pattern bypasses the `ExternalPtr` route, you need to implement
the full low-level trait hierarchy manually rather than using
`#[derive(AltrepInteger)]`:

1. **`AltrepExtract`** — return the static singleton (no ExternalPtr lookup).
2. **`AltrepLen` + `AltIntegerData`** — stubs only; the low-level `Altrep` /
   `AltInteger` impls read from `data1` directly.
3. **`AltrepDataptr<i32>`** — stub; the low-level `AltVec::dataptr` forwards
   `DATAPTR_RO(data1)`.
4. **`impl_inferbase_integer!(T)`** — maps the type to `INTSXP` and installs
   method tables.
5. **`Altrep` + `AltVec` + `AltInteger`** — low-level method tables, each
   reading `data1` via `x.altrep_data1_raw_unchecked()`.
6. **`RegisterAltrep`** — `OnceLock`-based class registration.
7. **`IntoR`** — allocate a plain `INTSXP`, fill it, and call
   `cls.new_altrep(data1, SEXP::nil())`.

### `data1` vs `data2`

| Slot   | Contents                                               |
|--------|--------------------------------------------------------|
| data1  | plain `INTSXP` — the actual element storage            |
| data2  | `R_NilValue` — no separate materialization cache needed |

Because `data1` is already a contiguous R integer buffer, `Dataptr` is trivial:
just return `DATAPTR_RO(data1)`.  No `data2` materialization step (no
`materialize_altrep_data2` call) is needed.

---

## Pointer Provenance: `addr_of_mut!`

Do **not** write `&mut INSTANCE` directly.  Under Stacked Borrows (Miri /
future Rust rules), creating a mutable reference to a static that might alias
with an existing shared reference is UB, even for ZSTs.

The safe pattern uses a raw pointer:

```rust
unsafe { std::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap_unchecked() }
```

This:
1. Produces a raw `*mut NativeSexpIntAltrep` without creating an intermediate
   mutable reference.
2. Reborrrows it as `&mut Self` via `as_mut()` — valid because the ZST
   occupies no memory.
3. `unwrap_unchecked()` avoids a branch on a provably non-null pointer.

---

## Proof-of-Concept Fixture

The reference implementation is in
`rpkg/src/rust/native_sexp_altrep_fixture.rs`.  It exports:

- **`native_sexp_altrep_new(values)`** — constructor.  Takes an integer vector
  from R, stores it as a plain `INTSXP` in `data1`, and returns the ALTREP
  SEXP.
- **`gc_stress_native_sexp_altrep()`** — no-arg GC-torture fixture (see
  `GCTORTURE_TESTING.md`).

R tests are in `tests/testthat/test-native-sexp-altrep.R`.

---

## Example Usage

```rust
use miniextendr_api::altrep::RegisterAltrep;
use miniextendr_api::altrep_data::{AltIntegerData, AltrepDataptr, AltrepExtract, AltrepLen};
use miniextendr_api::altrep_traits::{AltInteger, AltVec, Altrep, AltrepGuard};
use miniextendr_api::ffi::{DATAPTR_RO, R_xlen_t, Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::{impl_inferbase_integer, miniextendr};

pub struct MyNativeSexpAltrep;
static mut MY_INSTANCE: MyNativeSexpAltrep = MyNativeSexpAltrep;

impl AltrepExtract for MyNativeSexpAltrep {
    unsafe fn altrep_extract_ref(_x: SEXP) -> &'static Self {
        unsafe { &*std::ptr::addr_of!(MY_INSTANCE) }
    }
    unsafe fn altrep_extract_mut(_x: SEXP) -> &'static mut Self {
        unsafe { std::ptr::addr_of_mut!(MY_INSTANCE).as_mut().unwrap_unchecked() }
    }
}

impl AltrepLen for MyNativeSexpAltrep { fn len(&self) -> usize { 0 } }
impl AltIntegerData for MyNativeSexpAltrep { fn elt(&self, _i: usize) -> i32 { 0 } }
impl AltrepDataptr<i32> for MyNativeSexpAltrep {
    fn dataptr(&mut self, _w: bool) -> Option<*mut i32> { None }
}

impl_inferbase_integer!(MyNativeSexpAltrep);

impl Altrep for MyNativeSexpAltrep {
    const GUARD: AltrepGuard = AltrepGuard::RUnwind;
    fn length(x: SEXP) -> R_xlen_t {
        unsafe { x.altrep_data1_raw_unchecked() }.xlength()
    }
}
impl AltVec for MyNativeSexpAltrep {
    const HAS_DATAPTR: bool = true;
    fn dataptr(x: SEXP, _w: bool) -> *mut core::ffi::c_void {
        let data1 = unsafe { x.altrep_data1_raw_unchecked() };
        unsafe { DATAPTR_RO(data1) }.cast_mut()
    }
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        let data1 = unsafe { x.altrep_data1_raw_unchecked() };
        unsafe { DATAPTR_RO(data1) }
    }
}
impl AltInteger for MyNativeSexpAltrep {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        let data1 = unsafe { x.altrep_data1_raw_unchecked() };
        data1.integer_elt(i)
    }
}
```

---

## Tradeoffs vs ExternalPtr-Backed ALTREP

| Aspect              | ExternalPtr (default)         | Native-SEXP (this pattern)    |
|---------------------|-------------------------------|-------------------------------|
| Storage             | Rust heap via `Box<T>`        | R heap via `INTSXP` etc.      |
| GC lifecycle        | Finalizer on ExternalPtr      | Automatic R refcounting       |
| Serialization       | Manual (via `AltrepSerialize`)| Easy — R knows how to save it |
| Dataptr             | May need `data2` cache        | `DATAPTR_RO(data1)` trivially |
| Mutable state       | Possible via `&mut T`         | Possible via `DATAPTR` write  |
| Computed sequences  | Less natural                  | Natural — compute from params |
