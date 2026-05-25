# altrep_impl.rs consolidation (Path B from #682)

## Goal

`miniextendr-api/src/altrep_impl.rs` is 2591 lines. ~80% is declarative-macro
plumbing for 7 ALTREP family `impl_alt*_from_data!` macros (Integer, Real,
Logical, Raw, String, List, Complex) plus a meta-macro
`impl_builtin_altrep_family!` and a long tail of builtin instantiations.

Pick the **highest-leverage subset of #682 for one PR**: Path (b) — declarative
consolidation. Defer Path (a) (proc-macro rewrite of the declarative layer) to
a follow-up issue.

User-facing surface (`#[derive(AltrepInteger)]` etc.) **must not change**.
Internal call sites in the same crate are fair game ("no backwards
compatibility").

## What lives where today

- `impl_alt*_from_data!` × 7 families (Integer, Real, Logical, Raw, String,
  List, Complex). Each takes the type plus an optional comma-separated knob
  list out of: `dataptr` / `serialize` / `subset` / `materializing_dataptr`.
  Most families also accept reverse spellings (`serialize, dataptr` aliases
  to `dataptr, serialize`).
- `__impl_altrep_base!` vs `__impl_altrep_base_with_serialize!` — two macros
  with the same shape except the second adds 4 items for serialization.
- `__impl_altvec_dataptr!`, `__impl_altvec_string_dataptr!`,
  `__impl_altvec_integer_dataptr!` / `_real_` / `_logical_` / `_raw_` /
  `_complex_` — the per-family thin wrappers add a trivial `AltrepDataptr<T>`
  with `None` and delegate to `__impl_altvec_dataptr!($ty, T)`. They exist
  only to inject the element type.
- `__impl_alt_from_data!` — combinator that picks between
  `base` / `base_with_serialize` plus a `dataptr($elem)` / `string_dataptr` /
  `subset` arm; tied together with `$methods` and `$inferbase` callback
  idents.
- Built-in instantiations (`Vec<i32>`, `Box<[String]>`, `Cow<…>`, `Range<…>`,
  `[T; N]` const-generic arrays, `&'static [T]`) cover lines ~1665–2591.

## Reverse-spelling redirects (clutter to delete)

- `($ty, serialize, dataptr)` → `impl_alt*_from_data!($ty, dataptr, serialize)`
- `($ty, serialize, subset)` → `impl_alt*_from_data!($ty, subset, serialize)`

No external call site uses the reverse spelling (verified via `grep -rn`).
Pick alphabetical canonical order (`dataptr, serialize` and
`subset, serialize`). Delete the reverse aliases.

## Materializing vs default — fully duplicate

For Integer / Real / Logical / Raw / String / Complex, the `materializing_dataptr`
arm in `impl_alt*_from_data!` is byte-identical to the default arm with no
knobs. Both go: `__impl_altrep_base!` → `__impl_altvec_<family>_dataptr!` →
`__impl_alt*_methods!` → `impl_inferbase_*!`. The per-family
`__impl_altvec_<family>_dataptr!` is the materializing path because it
provides the trivial `AltrepDataptr<T>` impl returning `None`.

`($ty, materializing_dataptr, serialize)` is identical to `($ty, serialize)`.

Only call site for the materializing arms is `impl_builtin_altrep_family!`'s
`materializing` mode. Migrate it to call `($ty)` and `($ty, serialize)`
instead, then delete the `materializing_dataptr` arms entirely.

## Canonical macro shape

After consolidation, each `impl_alt<family>_from_data!` has just these arms:

```text
($ty)                          // default (materializing if no native dataptr)
($ty, dataptr)                 // direct contiguous native dataptr
($ty, serialize)               // default + serialization
($ty, subset)                  // default + extract_subset
($ty, dataptr, serialize)
($ty, subset, serialize)
```

That's 6 arms per family (down from 10). Path (a) is the only way to collapse
further without a proc-macro rewrite (declarative macros can't accept
unordered comma-separated optional tokens).

## `__impl_altrep_base!` merge

Today:

```rust
__impl_altrep_base!($ty);                   // length only, default guard RUnwind
__impl_altrep_base!($ty, $guard);           // length only, explicit guard
__impl_altrep_base_with_serialize!($ty);    // length + serialize, default guard
__impl_altrep_base_with_serialize!($ty, $guard);  // length + serialize, explicit guard
```

After:

```rust
__impl_altrep_base!($ty);                            // length only, default guard
__impl_altrep_base!($ty, $guard);                    // length only, explicit guard
__impl_altrep_base!($ty, with_serialize);            // length + serialize, default guard
__impl_altrep_base!($ty, $guard, with_serialize);    // length + serialize, explicit guard
```

The `with_serialize` token is a literal flag. (Putting it after `$guard`
keeps the `$guard:ident` arm unambiguous with the `with_serialize` arm.)

Update three internal call sites:
1. `__impl_alt_from_data!` `serialize` / `subset, serialize` /
   `dataptr, serialize` / `string_dataptr, serialize` arms.
2. `altrep_derive.rs` expanded path (line 354 — `has_serialize`).
3. `altrep_derive.rs` AltrepList non-default-guard path (line 765/778).

## Raw `Rf_protect`/`Rf_unprotect` sites

Two spots emit raw `Rf_protect(Rf_allocVector(STRSXP, n))`:
- `__impl_altvec_string_dataptr!` (lines ~355–370)
- `__impl_altstring_methods!::elt` (lines ~1196–1210)

Both run inside ALTREP callbacks (no `with_r_thread` debug-assert needed),
so MXL301 permits `_unchecked` variants. Both use `Rf_allocVector` (checked)
+ `Rf_protect` (checked) today — switch all three to `_unchecked` uniformly
inside these two macros for consistency with the `__impl_altrep_base_with_serialize!`
`unserialize` arm (lines 229–237), which already uses unchecked.

Do **not** migrate to `OwnedProtect` — that would change the control-flow
shape inside macro expansions and is out of scope (PR #509 covers
scope-as-allocator across the codebase).

## File split

Keep `altrep_impl.rs` + `altrep_impl/` per CLAUDE.md "no mod.rs".

```
altrep_impl.rs         // pub fn checked_mkchar, altrep_region_buf; re-exports submods
altrep_impl/
    macros.rs          // all declarative macros (~1.6K → ~1.1K)
    builtins.rs        // impl_builtin_altrep_family! invocations + impl_register_altrep_builtin
    arrays.rs          // const-generic [T; N] impls
    static_slices.rs   // &'static [T] hand-rolled impls
```

`impl_builtin_altrep_family!` and `impl_register_altrep_builtin!` are
crate-private macros (not `#[macro_export]`), so they need to live in the
file that invokes them — moving them to `altrep_impl/macros.rs` would
require `#[macro_export]`, which would pollute the public surface. Keep
them defined inline at the top of `altrep_impl/builtins.rs`.

The public `impl_alt*_from_data!` macros (and their `__impl_…` helpers) are
all `#[macro_export]`, so they can move to `altrep_impl/macros.rs` freely —
`#[macro_export]` makes them crate-root reachable regardless of where the
file lives.

`altrep_impl.rs` keeps:
- `checked_mkchar` (referenced by `__impl_altstring_methods!` expansion via
  `$crate::altrep_impl::checked_mkchar`)
- `altrep_region_buf` (referenced by `altrep_bridge.rs` via
  `crate::altrep_impl::altrep_region_buf`)
- `mod macros;`, `mod builtins;`, `mod arrays;`, `mod static_slices;` lines

## Implementation order

1. Plan committed (this file).
2. Switch raw FFI to `_unchecked` in the two STRSXP allocation sites.
3. Merge `__impl_altrep_base!` and `_with_serialize!` into single macro
   with `with_serialize` token. Update three internal call sites.
4. Delete `materializing_dataptr` arms from 6 families. Update
   `impl_builtin_altrep_family!` to call `($ty)` / `($ty, serialize)`.
5. Delete reverse-spelling aliases from 6 families.
6. Verify `just check` and `cargo build` clean after each step.
7. File-split into `altrep_impl/` directory.
8. Final `just check`, `just clippy`, CI's `clippy_all` feature set,
   `just devtools-test`, UI snapshots.

## Verification checklist

- `cargo build -p miniextendr-api`
- `just check` (workspace check across all manifests)
- `just clippy`
- CI's `clippy_all` feature set (read from `.github/workflows/ci.yml`)
- `just devtools-test` (exercises ALTREP via existing fixtures)
- `TRYBUILD=overwrite cargo test -p miniextendr-macros` should produce **no diff**

## Out of scope (file follow-up issues)

- **Path (a) proc-macro rewrite** — file an issue
  `refactor(api): replace altrep_impl declarative macros with proc-macro emit (Path A from #682)`.
- **`altrep_data/builtins.rs` audit (1874 lines)** — file an issue.

If both follow-ups are filed and Path (b) lands cleanly, #682 stays open
referencing Path (a) until completed, or we close #682 and let the
follow-up issues carry the remainder. Decision deferred until after the
implementation lands.
