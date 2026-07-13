# Plan: #1214 — `Strict<T>`/`Lax<T>` per-value markers + per-parameter attribute grammar

Date: 2026-07-13. Anchors verified against main @ 175674a7.
Branch: `feat/1214-per-param-strictness`.

Decision baked in (maintainer, 2026-07-13): **#1214 proceeds** with (a)
non-colliding marker names and (b) a per-parameter attribute grammar as the
second spelling, shaped so #1338 (`allow_missing`) can later ride it. The two
former blockers are resolved below. Error surface: the batched grammar already
shipped by #1192 / #1097 / #1324 — this plan adds NO new error grammar.

## Baked naming decision: `Strict<T>` + `Lax<T>`

- `Coerce<T>` (the issue's original proposal) is unusable: `pub trait
  Coerce<R>` exists at `miniextendr-api/src/coerce.rs:64` and is
  prelude-exported (`prelude.rs:70`). The obvious rename `Coerced<T>` collides
  again: `pub struct Coerced<T, R>` at `coerce.rs:920`. Detection is
  last-path-segment matching, so a same-named marker would misdetect every
  existing `Coerced<i64, i32>`-shaped signature — both names are off the
  table permanently, not merely awkward.
- **`Lax<T>` is the coerce-side marker name.** Rationale: (1) zero collisions
  — no `struct|enum|trait|type Lax` and no `Lax<` usage anywhere in
  `miniextendr-api/src`, `miniextendr-macros/src`, `rpkg/src/rust` (verified
  2026-07-13; same check passed for `Strict`); (2) it is the framework's own
  established antonym — the api docs already describe the default `IntoR`
  widening as the "lax" path (`strict.rs` module docs, lib.rs "Choosing the
  right API"); (3) the pair reads as one axis in a signature:
  `fn f(a: Lax<u16>, b: Strict<i64>, c: f64)`; (4) it dodges overloading the
  word "coerce", which is already three things (trait `Coerce`, struct
  `Coerced`, attribute key `coerce`).
- **`Coerced<T, R>` is absorbed as-is, not replaced** (per the 2026-07-11
  unified-design comment on #1213): it already works as a bare argument type
  via its `TryFromSexp` bridge (`from_r.rs:1003+`, container impls in
  `optionals/tinyvec_impl.rs` / `nalgebra_impl.rs`) with zero macro changes.
  Division of labor: `Coerced<T, R>` = explicit source-type pairing, runtime
  trait only, composes at any depth, never macro-detected; `Lax<T>` = policy
  marker meaning "apply the same `CoercionMapping`-inferred pipeline as the
  `coerce` attribute" — no `R` to spell. Polish items from that comment stay
  separate follow-ups (export `Coerced` from `prelude.rs`; optional defaulted
  second type param) — file/link issues, do not fold in here.

## What already exists (the two premise corrections, re-verified)

- **Per-parameter `coerce` exists on bare fns**: attribute ON the parameter
  (`fn greet(#[miniextendr(default = "\"World\"")] name: String)` shape —
  `rpkg/src/rust/default_tests.rs:8`), parsed by `parse_per_param_attr` into
  `PerParamMiniextendrAttr { has_coerce, has_match_arg, default_value,
  choices, has_several_ok }` (`miniextendr_fn.rs:414-513`), conflict-checked
  by `validate_per_param_attr_conflicts` (`:314-403`), consumed via
  `RustConversionBuilder::should_coerce` (`rust_conversion_builder.rs:74`,
  mapping from `CoercionMapping::from_type`, `miniextendr_fn.rs:49-115`).
- **Impl methods use a different spelling**: method-level keys NAMING the
  parameter — `match_arg(p1, p2)`, `choices(p = "a, b")`, `default(p = "v")`,
  `match_arg_several_ok(...)`, `choices_several_ok(...)`
  (`miniextendr_impl.rs:1463-1525`, into `method_attrs.per_param`). `coerce`
  on the impl path is method-wide only (`:1323-1326`, `builder.coerce_all()`
  at `:2887-2888`); `strict` is impl-block-wide only (`:1124-1127`,
  `builder.strict()` at `:2899-2900`).
- The real gaps (this plan's scope): per-parameter `strict` (both paths),
  per-parameter opt-outs (`no_coerce`/`no_strict`) under fn/impl-wide flags,
  per-param `coerce` on the impl path, and **return-position** strictness.

## Design — markers (first spelling)

1. New file `miniextendr-api/src/conversion_markers.rs` (mod from `lib.rs`,
   re-export from `prelude.rs`):
   ```rust
   #[repr(transparent)]
   pub struct Strict<T>(pub T);   // + new(), into_inner(), Deref<Target=T>
   #[repr(transparent)]
   pub struct Lax<T>(pub T);      // same surface
   ```
2. Macro detection = new `type_ends_with` call sites (`miniextendr_fn.rs:125`),
   peeled the same way `Missing<T>` already is
   (`get_missing_inner_type`-style; see the unwrap at `miniextendr_fn.rs:365`).
   Macro-generated glue converts the inner type then wraps (`Strict(v)` /
   `Lax(v)`) so the user's fn body receives the declared type.
3. **`Strict<T>`, argument position**: peel, then route through
   `strict_input_conversion_for_type` (`return_type_analysis.rs:377-422` —
   the `strict::checked_try_from_sexp_*` / `checked_vec_*` /
   `checked_vec_option_*` family, `miniextendr-api/src/strict.rs`,
   parameter-named panics, already batched post-#1324). Applies regardless of
   fn-level `strict`/`no_strict`.
4. **`Strict<T>`, return position**: peel in `analyze_return_type`
   (`return_type_analysis.rs:76`) and route through
   `strict_conversion_for_type` (`:310-365`, the `checked_into_sexp_*`
   family) regardless of fn-level mode. For impl methods, the same peel
   happens where the method return conversion consults `strict`
   (`miniextendr_impl.rs:2562` threading). Accepted inner types, BOTH
   positions: exactly the existing dispatch domain — `LOSSY_SCALARS`
   (`return_type_analysis.rs:304`: i64/u64/isize/usize) and their
   `Vec<_>`/`Option<_>`/`Vec<Option<_>>` wrappings. Anything else is
   `compile_error!` — loud, unlike fn-level `strict`'s silent fall-through
   (which stays as-is for the fn-level flag).
5. **`Lax<T>`, argument position only**: peel, then require
   `CoercionMapping::from_type(inner)` to return `Some`
   (u16/i16/i8/u32/u64/i64/isize/usize/bool/f32 + their `Vec<_>`s,
   `miniextendr_fn.rs:49-115`) — else `compile_error!`. Emission is the
   existing per-param-coerce arm (`rust_conversion_builder.rs:472-473`),
   applied regardless of fn-level `coerce`/`no_coerce`. `Lax<T>` in return
   position is `compile_error!` (return-side lax IS the default; nothing to
   select).
6. Runtime trait impls for the markers (`TryFromSexp for Strict<T>`, etc.)
   are NOT in scope — the checked helpers want a parameter name the trait
   can't know, and `Coerced<T, R>` already covers nested/manual use on the
   lax side. File a follow-up issue ("marker trait impls for nested
   `Vec<Strict<T>>` use") and link it in the PR body.

## Design — per-parameter attribute grammar (second spelling)

7. **Grammar contract (this is the #1338 extensibility hook — state it in
   the code docs verbatim):** a boolean per-parameter conversion policy is
   spelled as a **bare ident on the parameter** for bare fns
   (`#[miniextendr(strict)] x: i64`) and as a **`key(param1, param2, ...)`
   list at method level** for impl methods (`#[miniextendr(strict(x, y))]`).
   Valued policies use `key = "..."` / `key(param = "...")` respectively
   (existing `default`/`choices` shapes). Adding a future key (e.g. #1338's
   `allow_missing`) = one parse arm per path + one validation entry + one
   consumption site, no grammar redesign.
8. New keys, both paths: `strict`, `no_strict`, `no_coerce` (and `coerce` on
   the impl path's per-param list form — `Meta::List` `coerce(p1)` is
   distinguishable from the existing method-wide `Meta::Path` `coerce`).
   Wire: extend `PerParamMiniextendrAttr` + `parse_per_param_attr`
   (`miniextendr_fn.rs:414-513`) and the method-level parser
   (`miniextendr_impl.rs:1463-1525`); extend `RustConversionBuilder` with
   `with_strict_param` / `with_no_strict_param` / `with_no_coerce_param`
   beside `with_coerce_param` (`rust_conversion_builder.rs:53-76`), and give
   `should_coerce` (`:74`) a per-param-opt-out arm.
9. **Precedence (bake into code + docs as one rule):** per-parameter
   spellings (marker type and per-param attr are EQUAL precedence — if both
   appear on one parameter and disagree, `compile_error!`; agreement is
   allowed but pointless) > fn/method/impl-wide flags (`strict`/`no_strict`
   at `miniextendr_fn.rs:1346+` / `miniextendr_impl.rs:1124-1127`;
   `coerce`/`no_coerce` fn- and method-level) > crate feature defaults
   (`strict-default`, `coerce-default` — resolution sites
   `miniextendr_impl.rs:1191`, `:1757`).
10. **Validation** (extend `validate_per_param_attr_conflicts`,
    `miniextendr_fn.rs:314-403`, and mirror on the impl path):
    - `strict` + `coerce` on one param (any spelling mix, e.g. `Strict<T>` +
      per-param `coerce`, or `Lax<T>` + per-param `strict`) → error: they
      select mutually exclusive pipelines.
    - `strict` + `match_arg`/`choices` on one param → error (same shape as
      the existing coerce×match_arg rule at `:321-340`).
    - `strict`/`no_strict` together, `coerce`/`no_coerce` together → error.
    - `Strict<Lax<T>>` / `Lax<Strict<T>>` → error (nested markers).
    - `Missing<Strict<T>>` and `Missing<Lax<T>>` are ALLOWED — extend the
      `Missing` nesting validation (`:289-305`) to peel the new markers;
      `Missing<Coerced<T, R>>` needs no change (never macro-detected).
    - Marker in a position its direction forbids (`Lax` in return, either
      marker on a `&Dots` param) → error.
11. Syntactic limits: same as `Dots`/`Missing`/#1213's markers — aliases and
    `use ... as` renames defeat last-segment detection; the miss falls back
    to the default pipeline for the WRAPPER type itself, which (having no
    `TryFromSexp` impl, see item 6) fails to compile rather than silently
    converting laxly. Document once in the shared marker section
    (`docs/MINIEXTENDR_ATTRIBUTE.md`) rather than per family.

## Error surface (reuse, don't invent)

- Argument-side `Strict` vectors: `strict::checked_vec_*` panics are already
  batched by #1324 (cap `BATCHED_ERROR_CAP = 10`, `from_r.rs:1993`;
  `BatchedErrors` `:2016`; `"; and N more"` tail `:2046`).
- `Lax` per-param coercion: the `CoercionMapping::Vec` emission arm batches
  element errors since #1192.
- Return-side `Strict`: scalar `checked_into_sexp_*` panic messages are
  pinned prose — do not reword; vector variants batched post-#1324.
- New compile errors get trybuild coverage; new runtime paths pin the
  EXISTING grammar in testthat. Zero new error types, zero reworded messages.

## Fixtures + tests

12. New `rpkg/src/rust/strict_lax_marker_tests.rs` (`noexport` where the
    R-facing name doesn't matter): mixed signature
    `fn f(a: Lax<u16>, b: Strict<i64>, c: f64) -> f64`; return-position
    `-> Strict<i64>` and `-> Strict<Vec<i64>>`; `Missing<Strict<i64>>`;
    per-param attr spellings on a bare fn (`#[miniextendr(strict)] x: i64`,
    `#[miniextendr(no_coerce)]` under fn-wide `coerce`); an impl method using
    `#[miniextendr(strict(x), coerce(y))]`. No SEXP storage anywhere → no
    gc_stress fixture needed (say so in the PR body).
13. testthat `test-strict-lax-markers.R`: overflow through `Strict<i64>` arg
    raises the strict panic message; vector overflow shows the batched
    `and N more` grammar; `Lax<u16>` accepts an R integer where the bare
    param errors; `no_coerce` opt-out rejects what fn-wide coerce accepted;
    return `Strict<i64>` overflow raises; in-range values round-trip.
14. trybuild UI cases for every `compile_error!` in item 10 plus
    `Strict<String>` (non-lossy inner) and return-position `Lax<T>`.
    `TRYBUILD=overwrite cargo test -p miniextendr-macros`, review the diff
    (#1239: CI authoritative if stdlib spans appear).
15. Docs: extend `docs/MINIEXTENDR_ATTRIBUTE.md` (attr keys + grammar
    contract + precedence rule) and `docs/COERCE.md` (marker vs `Coerced<T,R>`
    division of labor); rustdoc on `conversion_markers.rs` carries the
    positions/inner-type tables. Cross-link #1338 at the grammar-contract
    paragraph.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
cargo test -p miniextendr-macros 2>&1 > /tmp/1214-macros.log   # Read it
cargo test -p miniextendr-api 2>&1 > /tmp/1214-api.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
# double install: new fixture exports need the second pass
just test 2>&1 > /tmp/1214-rust.log
just devtools-test 2>&1 > /tmp/1214-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1214-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + clippy_all/_s7 per ci.yml
cargo fmt --all
```

Commit regenerated `NAMESPACE` + `man/*.Rd` in sync (pre-commit hook;
`git config core.hooksPath .githooks`).

## Must NOT touch

- `coerce.rs` traits/`Coerced<T, R>`/`WidensTo*` markers and the E0119
  landscape (`audit/coerce.md` issue #4) — absorption means zero edits there.
- `strict.rs` helper signatures and panic prose; `BatchedErrors`/#1192
  machinery; `IntoRAs` (#1097, closed) — consumers only.
- Fn-level `strict`/`coerce` semantics for UNANNOTATED params (feature
  defaults `strict-default`/`coerce-default` resolution unchanged).
- `match_arg`/`choices`/`default`/`several_ok` behavior (only new conflict
  arms beside them).
- #1213's `Invisible`/`Visible` surfaces (branch
  `feat/1213-visibility-markers`); if both branches add a shared peel helper
  in `type_inspect.rs`, coordinate merge order — either first, rebase the
  second.

## Done criteria

- One parameter can be strict while its neighbor coerces, in both spellings,
  on both bare fns and impl methods; return-position `Strict<T>` works;
  opt-outs override fn-wide flags; every illegal combination in item 10 is a
  pinned compile error; error messages are byte-identical to the existing
  strict/batched grammar; grammar contract documented with the #1338 hook;
  suites + three clippy legs + fmt green; `Fixes #1214` with follow-up issues
  (marker trait impls; `Coerced` prelude export + defaulted param, if not
  already filed) linked.

## Escalation rule

If reality diverges — the impl path can't see parameter attrs where this plan
assumes, `CoercionMapping`'s domain and the coerce attr's actual acceptance
differ, the strict helpers turn out not to batch on some path, or marker
peeling collides with `Missing`'s existing unwrap order — **stop, commit
nothing further, and report back. Do not improvise.**
