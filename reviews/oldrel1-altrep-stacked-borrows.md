# oldrel-1 R CMD check crash: ALTREP `Dataptr` `&T`/`&mut T` aliasing

## Symptom

`R CMD check / Linux oldrel-1` (R 4.5 on Ubuntu 24.04 hosted runner) failed with:

```
[Rust] LazyIntSeq: Materializing 10 elements...
[Rust] LazyIntSeq: Materialization complete!
malloc(): unsorted double linked list corrupted
Aborted (core dumped)
```

`Linux release` (R 4.5.x) and `Linux devel` passed on the same commit. Only oldrel-1
crashed, and only sporadically — earlier runs of the same test on the same job were
fine, the crash happened mid-stream after several materialize calls.

## Root cause

`__impl_altvec_dataptr!` in `miniextendr-api/src/altrep_impl.rs` had this shape in the
read-only branch:

```rust
let d = unsafe { altrep_extract_ref(x) };           // &T
let ro = AltrepDataptr::dataptr_or_null(d);         // returns Option<*const T>
if let Some(p) = ro { return p.cast_mut(); }
let d = unsafe { altrep_extract_mut(x) };           // &mut T — &T from above still live
AltrepDataptr::dataptr(d, false)...
```

The `&T` from `altrep_extract_ref` is shadowed (not dropped) when the fall-through
`altrep_extract_mut` constructs a `&mut T` to the same data. Under Stacked Borrows
this is UB — the prior shared retag is still on the borrow stack when a unique retag
is pushed. Concrete consequences depend on what each downstream function does with
the pointers; in this case, ALTREP types whose `dataptr_or_null` returns `None`
(e.g., types that materialize lazily, including `LazyIntSeq` after a partial
materialize) hit the buggy path and corrupted the allocator.

CLAUDE.md flags exactly this class of bug:

> **Pointer provenance**: cache `*mut T` via a mutable path (`&mut T`, `Box::into_raw`,
> `downcast_mut`, `ptr::from_mut`). Never write through a `cached_ptr` derived from
> `&T` / `downcast_ref` — UB under Stacked Borrows.

## Fix

Scope the `&T` borrow into a block so it's dropped before `altrep_extract_mut`:

```rust
let ro = {
    let d = unsafe { altrep_extract_ref(x) };
    AltrepDataptr::dataptr_or_null(d)
};
if let Some(p) = ro { return p.cast_mut(); }
let d = unsafe { altrep_extract_mut(x) };
AltrepDataptr::dataptr(d, false)...
```

`dataptr_or_null` returns by value (no borrow held), so closing the inner block
releases `d` before the mutable retag.

## Why oldrel-1 specifically

The crash is glibc's allocator (`malloc(): unsorted double linked list corrupted`),
which means the violation lands in libc — not directly in Rust code. R 4.5 on
Ubuntu 24.04 ships glibc 2.39, which is stricter about double-linked-list integrity
than older glibc versions on the `release` and `devel` runners. The UB exists on
all platforms; the allocator just catches it deterministically on this one.

## Verification

`just check` clean. Local R CMD check on macOS arm64 with the fix produces no
regression. Linux oldrel-1 must be re-run on CI to confirm the fix lands.
