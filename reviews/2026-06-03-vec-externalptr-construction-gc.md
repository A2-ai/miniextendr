# `Vec<ExternalPtr<T>>` construction is GC-unsafe (#827 → #836)

## What was attempted

Implement `Vec<ExternalPtr<T>>` conversions in both directions (#827):
`TryFromSexp` (R `list()` → Vec) and `IntoR` (Vec → R `list()`), plus the
`Option`/NULL variants. The trait impls compiled, the proc-macro accepted
`Vec<ExternalPtr<Bag>>` args/returns, and 24 testthat assertions passed.

## What went wrong

Under `gctorture(TRUE)`, a round-trip fixture failed **40/40**:

```
veptr_increment_bags(bags): failed to convert parameter 'bags' ...
  invalid value: external pointer is null
```

The failure was in *argument conversion* — a handle inside `bags` had a NULL
`R_ExternalPtrAddr`, i.e. it had already been collected. A first read-back via
`veptr_sum_bags` (which reads the cached raw `*mut T` through `Deref`, never
re-checking the R address) returned the *correct* sum — a silent use-after-free
that masked the corruption. Only the re-converting path (`try_from_sexp`, which
checks the R address) surfaced it.

## Root cause

`ExternalPtr::new` → `create_extptr_sexp` does a balanced
`Rf_protect`/`Rf_unprotect(2)` during creation and returns an **unprotected**
SEXP; `Drop` is a no-op ("the finalizer handles cleanup"). That is fine for the
create-one-and-return-it case (R's `.Call` protects the result with no
intervening allocation), but building a `Vec<ExternalPtr<T>>` element-by-element
holds N unprotected handles in a Rust `Vec` — invisible to R's GC roots. Each
subsequent `ExternalPtr::new` allocates, and under GC pressure that allocation
collects the earlier handles (running their finalizers, freeing the boxes).

The conversion code is *not* at fault: the argument side wraps already-rooted
R-list elements, and the hazard is purely in constructing the `Vec` before it
reaches `IntoR`.

## Fix

Two iterations, both now landed:

1. **Initial workaround (superseded):** build the fixture `Vec`s under a
   `ProtectScope` that roots each handle as it is created
   (`vec_externalptr_tests.rs::build_rooted`), and re-root every handle inside
   the return-side `IntoR` (`vec_externalptr_to_list`) before allocating the
   VECSXP. gctorture passed, but this only papered over the construction window
   in *our* fixtures — a user's own `.map(ExternalPtr::new).collect()` would
   still corrupt.

2. **Root fix #836 (PR #841, merged):** owned `ExternalPtr` handles self-root in
   the process-wide `ProtectPool` for their whole Rust lifetime (root on
   create, release on drop; borrowed `wrap_sexp` handles take no root; the pool
   beats O(n) `R_PreserveObject` because a `Vec<ExternalPtr>` releases
   front-to-back). The natural `.map(ExternalPtr::new).collect()` is now GC-safe
   for everyone.

   With #841 merged, **this follow-up removed both workarounds**: the
   `ProtectScope` re-protect in `vec_externalptr_to_list` /
   `vec_option_externalptr_to_list` (the handed-in elements are already
   pool-rooted, and the list fill is allocation-free) and the `build_rooted`
   dance in the fixtures (which now build `Vec`s the natural way, so
   `gc_stress_vec_externalptr` is a genuine regression witness for #836 rather
   than a self-protected straw man).

## Lesson

When a feature encourages **holding multiple `ExternalPtr`s in a Rust container
across allocations**, it inherits this hazard. Always gctorture it, and never
trust a read path that goes through the cached `*mut T` (`Deref`) to *detect*
collection — it reads freed memory and lies. Re-conversion through
`R_ExternalPtrAddr` is the honest check.
