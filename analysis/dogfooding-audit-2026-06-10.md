# Dogfooding audit: raw protect-discipline sites across miniextendr-*

**Date**: 2026-06-10. **Context**: #673 (serde dogfooding) asked for
`OwnedProtect` / `ProtectScope` / `ListBuilder` adoption inside the serde
modules; this audit extends the question to every crate: *where else does the
codebase still hand-roll `Rf_protect` / `Rf_unprotect` counting instead of
using its own RAII layer?*

Method: grep for `Rf_protect|Rf_unprotect` (excluding `_unchecked`-only
matches and comments) across `miniextendr-{api,macros,engine,lint,cli}` at
`a52ecc07`, then classify each site by reading the surrounding code.

## Headline numbers (raw-protect lines per file, `miniextendr-api/src/`)

| File | Sites | Verdict |
|---|---|---|
| `into_r.rs` | 31 | migrate — one repeated idiom, see below |
| `optionals/jiff_impl.rs` | 15 | migrate (mechanical) |
| `optionals/datetime_realsxp.rs` | 8 | migrate (macro-emitted, 4 blocks) |
| `sys.rs` | 7 | infrastructure — keep raw |
| `refcount_protect.rs` | 7 | infrastructure — keep raw |
| `unwind_protect.rs` | 6 | infrastructure — keep raw |
| `list.rs` | 6 | migrate (2 blocks) |
| `into_r/collections.rs` | 6 | migrate (2 blocks) |
| `error_value.rs` | 6 | migrate with care (R-devel-sensitive, #344) |
| `protect_pool.rs` | 5 | infrastructure — keep raw |
| `encoding.rs` | 4 | migrate (1 block) |
| `convert.rs` | 4 | migrate (2 identical blocks) |
| `externalptr.rs` | 3 (+unchecked twins) | borderline — see below |
| `rarray.rs` | 2 | migrate (1 block) |
| `r_memory.rs` | 2 | keep raw (init-time probe) |
| `cached_class.rs` | 2 | migrate (1 block) |
| `serde/columnar.rs` | 0 | done (#673 columnar PR) |
| `serde/ser.rs` | 0 | done (#943) |
| `serde/de.rs` | 0 | nothing to do |

`miniextendr-macros/src/altrep.rs` has 3 sites in *generated* code (the
`IntoRAltrep` wrap); `miniextendr-engine`, `miniextendr-lint`,
`miniextendr-cli` have zero.

## The dominant idiom: protect-container-fill-unprotect

~24 of the 31 `into_r.rs` sites are copies of one block (checked and
`_unchecked` twins):

```rust
let list = Rf_allocVector(VECSXP, n);
Rf_protect(list);
for (i, inner) in self.into_iter().enumerate() {
    list.set_vector_elt(i, inner.into_sexp());   // allocates per element
}
Rf_unprotect(1);
list
```

Instances: `Vec<Vec<T>>`, `Vec<&[T]>`, `Vec<&[String]>`, `Vec<Vec<String>>`
(`into_r.rs:1470–1624`), the tuple-impl macro (`into_r.rs:1935–1968`),
`Vec<Box<[T]>>` / `Vec<Box<[String]>>` / `Vec<[T; N]>`
(`into_r.rs:2095–2156`), `Vec<HashMap<…>>` helper (`into_r.rs:2394`), plus
`list.rs:1112–1158` (`Vec<List>` / `Vec<Option<List>>`).

The protect usage is *correct* everywhere I read — the win is structural
(miscounts unrepresentable) and deduplication, not bug-fixing. Two options:

1. **Minimal**: swap each block's protect pair for
   `let list = OwnedProtect::new(Rf_allocVector(...))`. Zero design work,
   purely mechanical, ~30 minutes.
2. **Better**: add one private helper in `into_r.rs`
   (`unsafe fn vecsxp_from_iter(n, impl FnMut(usize) -> SEXP) -> SEXP` or
   reuse `ListBuilder`), and collapse the ~12 duplicate blocks onto it. The
   `_unchecked` twins need either a second helper or a const-generic flag.

Recommendation: (2) for `into_r.rs`/`list.rs` where the duplication is dense;
(1) for one-off sites elsewhere.

## Per-file notes (migratable)

- **`optionals/jiff_impl.rs`** — six protect-pairs around scalar/vector
  REALSXP builds + class/attr setting (`:189`, `:249`, `:353`, `:438`,
  `:467–480`, `:984`). All linear allocate→fill→set-attr→unprotect; direct
  `OwnedProtect` swaps. The `:467` difftime block sets two attrs with two
  separate protect pairs — one `ProtectScope` covers both.
- **`optionals/datetime_realsxp.rs`** — the four blocks live inside the
  `impl_realsxp_datetime!` declarative macro (`:203`, `:218`, `:236`, `:253`),
  so one edit fixes all expansions. Same `OwnedProtect` shape.
- **`into_r/collections.rs`** — map→named-list builder (`:59–81` checked,
  `:93–111` unchecked): `Rf_protect(list); Rf_protect(names); … Rf_unprotect(2)`.
  This is exactly `ListBuilder` + `StrVecBuilder` (or `NamedList`) territory —
  same shape `serde/columnar.rs` already migrated to.
- **`convert.rs`** — two identical `protect(sexp); set_names_on_sexp; unprotect(1)`
  blocks (`:731`, `:748`). `OwnedProtect` one-liners.
- **`encoding.rs`** — `l10n_info()` eval (`:65–85`): protect call + result,
  unprotect(2) before the UTF-8 error branch (correctly ordered today).
  `ProtectScope` swap keeps it correct under future edits.
- **`rarray.rs`** (`:862`), **`cached_class.rs`** (`:227`) — single
  protect-pairs around `set_dim` / `set_attr`; `OwnedProtect` swaps.
- **`into_r.rs` `eval_base_noarg`** (`:2421`) — protect around
  `R_tryEvalSilent`; `OwnedProtect` swap.
- **`error_value.rs`** (`:190–237`) — the `prot` counter in
  `make_rust_condition_value` predates `ProtectScope` and is the file the
  #344 R-devel segfault was fixed in. A `ProtectScope` expresses the same
  discipline with less ceremony, **but** this is the most GC-sensitive
  function in the crate: migrate alone in its own PR with a gctorture pass on
  R-devel, not batched.
- **`miniextendr-macros/src/altrep.rs`** (`:158–187`) — generated
  `into_sexp`/`into_sexp_unchecked` protect `data1` across `new_altrep`.
  Could emit `OwnedProtect`, but generated code currently avoids depending on
  `gc_protect` internals; low value, fine to leave.

## Keep raw (infrastructure / intentional)

- `gc_protect.rs`, `sys.rs` — they *are* the abstraction.
- `protect_pool.rs`, `refcount_protect.rs`, `unwind_protect.rs` — protection
  machinery with non-scope lifetimes (pool slots, refcounts, unwind
  boundaries); RAII scopes don't model them.
- `r_memory.rs:106` — one-shot init probe measuring SEXP layout; trivial and
  self-contained.
- `externalptr.rs:601–660` — EXTPTRSXP construction protects two objects
  across finalizer registration, then hands rooting to the pool
  (`root_owned`). The protect window interleaves with pool insertion;
  `OwnedProtect` would work but the explicit pairing documents the
  pool-handoff boundary. Borderline — migrate only if touching the file
  anyway.

## Suggested sequencing

1. ~~serde/columnar.rs~~ (landed), ~~serde/ser.rs~~ (#943).
2. `into_r.rs` + `list.rs` consolidation behind a shared helper — biggest
   win, one PR.
3. `optionals/{jiff_impl,datetime_realsxp}.rs` + small one-offs
   (`convert.rs`, `rarray.rs`, `cached_class.rs`, `encoding.rs`,
   `into_r/collections.rs`) — mechanical batch, one PR.
4. `error_value.rs` — solo PR, R-devel gctorture verification.

Each migration PR: no behavior change, no new public API, existing tests +
`gctorture(TRUE)` sweep per `docs/GCTORTURE_TESTING.md`.
