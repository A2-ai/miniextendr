# Conversion-trait duplication audit — 2026-06-03

Audit of duplicate / near-duplicate implementations across the R↔Rust
conversion traits in `miniextendr-api`. Scope: `TryFromSexp`, `IntoR`,
`IntoRAs`, `Coerce`, `TryCoerce` and their supporting macros + free functions.

## Method / tooling

This audit was driven by a machine-generated corpus rather than eyeballing.
The pipeline (committed under `.rust-llm-docs/`) is:

1. `cargo doc --no-deps -p miniextendr-api` with a broad feature set
   (`full` minus `datafusion`, plus `jiff`) and
   `RUSTDOCFLAGS="-Z unstable-options --output-format json"` →
   `target/doc/miniextendr_api.json` (rustdoc JSON, format v57, 18 360 items).
2. `.rust-llm-docs/rustdoc_megadoc.py` → `.rust-llm-docs/generated/miniextendr-api.md`
   (244 KB single-file API digest, LLM-parseable).
3. `.rust-llm-docs/rustdoc_impl_inventory.py` (new, written for this audit) →
   `.rust-llm-docs/generated/conversion-impl-inventory.md`. This walks every
   `impl` in the rustdoc JSON, renders the fully-resolved `for`-type, groups by
   trait, and **clusters impls by source span** so macro-expanded families
   collapse to one line and hand-rolled one-offs stand out.

The inventory is the evidence base; quoted `file:line` below was confirmed by
reading source.

## The conversion landscape (inventory summary)

| Trait | impls | where |
|---|---|---|
| `TryFromSexp` | 439 | `from_r.rs` + `from_r/*` |
| `IntoR` | 322 | `into_r.rs` + `into_r/*` |
| `IntoRAs` | 135 | `into_r_as.rs` |
| `TryCoerce` | 95 | `coerce.rs` |
| `Coerce` | 53 | `coerce.rs` |
| `AltrepSerialize` | 27 | `altrep_data/*` |

~11 500 LoC across the 8 core conversion files, driven by **54 `macro_rules!`**.

The architecture is, broadly, *good*: real conversion logic is factored into
shared free functions (`try_from_sexp_numeric_vec`, `coerce_slice_to_vec`,
`alloc_r_vector`, `try_coerce_scalar`, …) and thin macros stamp out the trait
wrappers. The duplication is **not** copy-pasted logic in 400 places — it is
concentrated in a few systematic axes where the macro layer (and a couple of
free functions) fork instead of compose.

## The duplication is a combinatorial matrix collapsed into parallel macros

Every conversion is a point in a 4-axis space:

- **container**: scalar · `Vec<T>` · `Box<[T]>` · `&[T]` · `HashSet`/`BTreeSet` · `HashMap`/`BTreeMap`
- **NA/Option**: plain (`T`, reject NA) vs `Option<T>` (NA → `None`)
- **element category**: native `RNativeType` · numeric-coerced · logical/bool · string · list-element
- **checked/unchecked**: `*_unchecked` thread-skip variant vs checked
- (`IntoRAs` adds a 5th: **target SEXP type** — `i32` / `f64` / `u8`)

Instead of one macro parameterized over these axes, the crate has a separate
macro per *combination slice*. That's why there are 54 macros for what is
conceptually a handful of operations. The five concrete findings below are the
high-value collapses.

---

## Finding 1 — `Vec<T>` vs `Vec<Option<T>>` free functions duplicate the SEXP-dispatch shell

**This is the exact case from the audit request.**

- `try_from_sexp_numeric_vec<T>` — `from_r.rs:1041`
- `try_from_sexp_numeric_option_vec<T>` — `from_r/na_vectors.rs:306`

Both have the **identical** where-bounds (`i32/f64/u8: TryCoerce<T>` …) and the
**identical** `match sexp.type_of()` shell over `INTSXP / REALSXP / RAWSXP /
LGLSXP` with the identical `_ =>` error string. They differ in exactly one
thing: the per-element closure.

- plain: `coerce_slice_to_vec(slice)` — coerce every element, NA round-trips.
- option: `if is_na(v) { None } else { coerce_value(v).map(Some) }`.

~30 lines of dispatch duplicated to vary one closure. Collapsible to a single
generic shell that takes the per-element map:

```rust
fn from_numeric_vec_with<T, U>(sexp: SEXP, map_i32, map_f64, map_u8, map_lgl) -> Result<Vec<U>, SexpError>
```

with `_vec` passing the coerce-through closures and `_option_vec` passing the
NA-aware ones. (Same idea applies to the `Box<[T]>` / `Box<[Option<T>]>` and
set variants that call these.)

## Finding 2 — `Vec<…>` and `Box<[…]>` macros are byte-identical, *and* inconsistent with their own bool sibling

In `from_r/na_vectors.rs`:

- `impl_vec_option_try_from_sexp!` — line **27**
- `impl_boxed_slice_option_try_from_sexp!` — line **59**

The two macro bodies are **byte-for-byte identical** — same type-check, same
`r_slice`, same `.iter().map(|&v| if is_na ...).collect()`. The only difference
is `Vec<Option<$t>>` vs `Box<[Option<$t>]>` in the header (the `.collect()`
target is inferred).

Meanwhile, immediately below at **line 126**, `Box<[Option<bool>]>` is *not*
duplicated — it delegates:

```rust
let vec: Vec<Option<bool>> = TryFromSexp::try_from_sexp(sexp)?;
Ok(vec.into_boxed_slice())
```

and `Box<[bool]>` at `from_r.rs:1124` does the same. So the codebase already
knows the right pattern (`Box<[T]>` = `Vec<T>` + `into_boxed_slice`) — it's just
applied inconsistently. **The entire `*_boxed_slice_*` macro family
(`impl_boxed_slice_option_try_from_sexp`, `impl_boxed_slice_try_from_sexp_native`)
can be deleted** in favour of a single blanket-style delegation:
`Box<[T]>: TryFromSexp where Vec<T>: TryFromSexp`. Same for the `IntoR` side.

The duplicated `try_from_sexp_unchecked` is also inconsistent: the
delegation-style macros omit it (rely on the default), the inlined ones
hand-write a second near-identical body.

## Finding 3 — `IntoRAs` forks per target SEXP type *and* per container; bodies identical modulo a type name

In `into_r_as.rs`:

- `impl_vec_into_r_as_i32!` — line **605**
- `impl_vec_into_r_as_f64!` — line **678**
- `impl_vec_into_r_as_u8!`  — line **821**

All three generate `IntoRAs<TARGET> for Vec<$from>` **and**
`IntoRAs<TARGET> for &[$from]`. Comparing `_i32` and `_f64`, the four method
bodies are identical except for the target type token (`i32`↔`f64`) passed to
`try_coerce_scalar(val, $from_name, "i32"|"f64")`. The `Vec` and `&[]` arms
inside each macro differ only by `into_iter()`/`iter()` and `val`/`&val`.

Collapsible to one macro:
`impl_vec_into_r_as!($target_ty, $target_name; $from, $from_name)` covering both
containers — **3 macros → 1**, and the scalar siblings
(`impl_into_r_as_i32_scalar` / `_f64_scalar` / `_u8_scalar`, lines 311/359/459)
fold the same way.

## Finding 4 — `TryCoerce` narrowing: five byte-identical macros differing only in the target type

In `coerce.rs`:

- `impl_try_i32!` — line **533**
- `impl_try_u8!`  — line **554**
- `impl_try_u16!` — line **617**
- `impl_try_i16!` — line **641**
- `impl_try_i8!`  — line **664**

Every one is, verbatim:

```rust
impl TryCoerce<$TARGET> for $t {
    type Error = CoerceError;
    #[inline]
    fn try_coerce(self) -> Result<$TARGET, CoerceError> {
        self.try_into().map_err(|_| CoerceError::Overflow)
    }
}
```

This is the cleanest collapse in the crate — **5 macros → 1** parameterized over
the target, with zero behaviour change:

```rust
macro_rules! impl_try_narrow {
    ($target:ty; $($from:ty),+ $(,)?) => { $(
        impl TryCoerce<$target> for $from {
            type Error = CoerceError;
            #[inline] fn try_coerce(self) -> Result<$target, CoerceError> {
                self.try_into().map_err(|_| CoerceError::Overflow)
            }
        }
    )+ };
}
impl_try_narrow!(i32; u32, u64, usize, i64, isize);
impl_try_narrow!(u8;  i8, i16, i32, i64, u16, u32, u64, usize, isize);
impl_try_narrow!(u16; i8, i16, i32, i64, u32, u64, usize, isize);
impl_try_narrow!(i16; i32, i64, u16, u32, u64, usize, isize);
impl_try_narrow!(i8;  ...);
```

## Finding 5 — `IntoR` repeats the 4-method wrapper ~106× and handles `_unchecked` inconsistently

`fn try_into_sexp(` appears **106 times** in `into_r.rs` + `into_r/*`. The
overwhelmingly common shape for an infallible conversion is:

```rust
type Error = std::convert::Infallible;
fn try_into_sexp(self) -> Result<SEXP, _> { Ok(self.into_sexp()) }
unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, _> { Ok(unsafe { self.into_sexp_unchecked() }) }
fn into_sexp(self) -> SEXP { ... real work ... }
```

The three wrapper methods are pure boilerplate around the one real method.
Worse, the `_unchecked` axis is handled three different ways across the
`Vec<Option<T>> → R` macros alone:

- `impl_vec_option_into_r!` (`into_r.rs:1632`) — hand-writes a full second
  `into_sexp_unchecked` body (`alloc_r_vector_unchecked`).
- `impl_vec_option_smart_i64_into_r!` (`into_r.rs:1672`) and
  `impl_vec_option_coerce_into_r!` (`into_r.rs:1722`) — `try_into_sexp_unchecked`
  just calls `self.try_into_sexp()` (no real unchecked path).

Three element-category macros (`_into_r` native / `_smart_i64` / `_coerce`) for
the *same* `Vec<Option<T>>` target is itself an axis fork. A small
`impl_infallible_into_r! { for $ty => $into_sexp_expr }` helper (or a sealed
`IntoSexp` core trait + blanket `IntoR`) would erase the wrapper-method
repetition crate-wide and make the unchecked policy uniform.

---

## What is already well-factored (do NOT "fix")

- **Logic lives in free functions** — `coerce_slice_to_vec`,
  `try_from_sexp_numeric_vec/_set`, `alloc_r_vector`, `try_coerce_scalar`,
  `logical_iter_to_lglsxp`. Macros are thin. Good.
- **`Option<ExternalPtr<T>>`** (`from_r.rs:1273`) correctly *delegates* to the
  scalar `ExternalPtr<T>` impl (`from_r.rs:1226`) + NILSXP→None. This is the
  target pattern, not duplication.
- **`references.rs:448-452`** — the 12-impl-per-span pointer families
  (`&T`/`&mut T`/`Vec<&[T]>`/…) are already macro-generated via
  `impl_ref_conversions_for!`. Leave them.
- **`ndarray_impl.rs`** — 7-impl-per-span `Array1..6 + ArrayD` families, already
  macroized per element type.
- **`i32` bespoke impl** (`from_r.rs:399`) is *intentionally* not macroized — it
  adds the `NA_integer_` (`i32::MIN`) rejection the generic scalar macro skips.
  Keep the carve-out; just document why (it already has a comment).

## Recommended consolidations (flat priority order)

1. **Finding 4** (`TryCoerce` 5→1 macro) — trivial, zero-risk, do first.
2. **Finding 2** (`Box<[…]>` delegate instead of parallel macro) — deletes a
   macro family, fixes an existing inconsistency. Low risk.
3. **Finding 3** (`IntoRAs` target/container fork → 1 macro each for scalar/vec).
4. **Finding 1** (`_vec` / `_vec_option` free-fn shell unification) — most
   behavioural care needed (NA semantics), highest payoff for the stated
   motivation.
5. **Finding 5** (`IntoR` wrapper-method boilerplate + uniform `_unchecked`) —
   largest surface; do last, possibly as a sealed-core-trait refactor.

Each should be its own PR/issue (per repo "deferred items = GitHub issues").
None changes R-observable behaviour; all are pure internal consolidation, so
existing testthat + Rust unit tests are the regression net. Re-run the
`rustdoc_impl_inventory.py` after each to confirm the `for`-type set is
unchanged (same impls, fewer macros).

## Cross-crate note

The conversion *traits* are implemented only in `miniextendr-api`.
`miniextendr-macros` emits *calls* into these traits (it does not `impl` them),
so there is no conversion-impl duplication to dedup in the macro crate — the
right lever there is ensuring generated code targets the consolidated macros
above. No action needed outside `miniextendr-api`.

---

# Part 2 — Manual impls that an existing (or small new) macro could absorb

Second lens (requested follow-up): given the 54 macros, which **hand-rolled**
impls could instead be macro invocations? Generated by
`.rust-llm-docs/rustdoc_manual_vs_macro.py` →
`.rust-llm-docs/generated/conversion-manual-vs-macro.md`. The heuristic:
impls sharing a source span are macro-expanded; an impl alone at its span is
hand-rolled. Per container shape, "N hand-rolled vs M macro-generated" with
M>0 means a macro for that shape already exists.

The headline: **the macro infrastructure for the container family already
exists and works — it's just representation-specific, and ~100 impls live in
representations it doesn't cover.**

## M1 — The `optionals/*` integrations hand-roll the container family ~100×

`from_r.rs` exports three reusable container macros (all `#[macro_export]`):

- `impl_option_try_from_sexp!(T)` — `Option<T>`: NILSXP → `None`, else delegate (line **1482**)
- `impl_vec_try_from_sexp_list!(T)` — `Vec<T>` from a VECSXP list (line **1511**)
- `impl_vec_option_try_from_sexp_list!(T)` — `Vec<Option<T>>` from VECSXP, NULL → `None` (line **1570**)

These are used correctly by the integrations whose R representation **is** a
list / nullable element — `aho_corasick` (all three), `serde` `JsonValue` (all
three), `toml`, `bitvec`. Proof: `aho_corasick_impl.rs` has **0** hand-rolled
container impls.

But integrations whose R representation is a **character vector** (`BigInt`,
`BigUint`, `Decimal`, `Uuid`, `Url`, `Regex`) or a **numeric pair / scalar**
(`Complex<f64>`, `OrderedFloat`, jiff/time types) have **no equivalent macro**
and therefore hand-roll the entire family. Hand-rolled `Option<X>` / `Vec<X>` /
`Vec<Option<X>>` / `Box<[X]>` container impls per file:

| file | hand-rolled container impls |
|---|---|
| `jiff_impl.rs` | 24 |
| `time_impl.rs` | 12 |
| `ordered_float_impl.rs` | 12 |
| `num_bigint_impl.rs` | 12 |
| `num_complex_impl.rs` / `rust_decimal_impl.rs` / `uuid_impl.rs` / `url_impl.rs` | 6 each |
| `toml_impl.rs` (5) · `bytes_impl.rs` (4) · `regex_impl.rs` (3) · `bitvec_impl.rs` (2) | — |
| **total across `optionals/`** | **101** |

Every body is mechanical. `num_bigint_impl.rs` is representative — the scalar
is `parse_bigint(&s)` inbound (`:155`) and `.to_string()` outbound (`:315`),
and the six container impls are pure derivations:

```rust
// TryFromSexp for Vec<Option<BigInt>>     (na_bigint:276)
let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
values.into_iter().map(|opt| match opt { Some(s) => parse_bigint(&s).map(Some), None => Ok(None) }).collect()
// IntoR for Vec<Option<BigInt>>           (:390)  ->  self.into_iter().map(|o| o.map(|v| v.to_string())).collect::<Vec<_>>().into_sexp()
```

**Recommendation — one macro, parameterised by the *base type* (not the
representation).** Every one of these conversions is structurally the same
operation:

> `Vec<X>` = `Vec<Base>`**.map**(`scalar_fn`)

where `Base` is a type that *already* has the full container conversions
(`String`, `f64`, `i32`, `Rcomplex`, …) and `scalar_fn` is the per-element
`Base → X` (inbound) / `X → Base` (outbound) map:

| type(s) | `Base` | scalar map |
|---|---|---|
| `BigInt` `BigUint` `Decimal` `Uuid` `Url` `Regex` | `String` | `parse_*` / `to_string` |
| `OrderedFloat<f64>` | `f64` | `OrderedFloat` / `.0` |
| `Complex<f64>` | `Rcomplex` | `from_rcomplex` / `to_rcomplex` |
| jiff/time scalars | `String` or `f64` | parse / format |

So the representation is *not* a barrier — it's just the choice of `Base`. A
single macro absorbs all of them:

```rust
// generates X, Option<X>, Vec<X>, Vec<Option<X>>, Box<[X]>, Box<[Option<X>]>
// for both TryFromSexp and IntoR, by delegating to the Base container conversions.
impl_via_base!(BigInt,            String,   parse_bigint,            |v| v.to_string());
impl_via_base!(OrderedFloat<f64>, f64,      |v| Ok(OrderedFloat(v)), |o| o.0);
impl_via_base!(Complex<f64>,      Rcomplex, |c| Ok(from_rcomplex(c)),  to_rcomplex);
```

That `OrderedFloat` *already* delegates its `Vec<Option<_>>` impl to
`Vec<Option<f64>>` (`ordered_float_impl.rs:192`) — but hand-rolls its plain
`Vec<_>` against the REALSXP slice (`:152`) instead of delegating to `Vec<f64>`
— is direct proof the pattern fits and is only applied inconsistently today.
~80 hand-rolled impls collapse to ~13 macro calls + each type's two scalar
impls.

## M1b — Can ONE macro fit *every* representation (including list-backed)?

Almost. There are exactly two element-extraction modes, and the line between
them is a performance choice, not a semantic one:

1. **Bulk-extract (atomic representations: string, numeric, complex)** —
   `let base: Vec<Base> = TryFromSexp::try_from_sexp(sexp)?; base.map(scalar)`.
   One slice read, then a cheap map. This is `impl_via_base!` above and covers
   ~80 hand-rolled impls.
2. **Per-element SEXP (list representation: `JsonValue`, `AhoCorasick`, …)** —
   each element is itself a structured sub-SEXP, pulled with `vector_elt(i)` and
   fed to the *full* `X::try_from_sexp`. There is no `Vec<Base>` to bulk-read.
   This is the existing `impl_vec_try_from_sexp_list!` family — already generic,
   already used.

A **single universal** macro is technically possible: re-wrap element *i* of any
SEXP as a length-1 SEXP and call the scalar `X::try_from_sexp` on it. That
unifies modes 1 and 2 — but it allocates **and GC-protects a temporary SEXP per
element** for atomic vectors, where mode 1 does one slice read. That cost is
exactly why the code bulk-extracts today. So the pragmatic target is **two
generic macros** (`impl_via_base!` for atomic, `impl_vec_*_list!` for list),
not one — together they cover 100 % of the optionals with ~13 + a handful of
calls, vs 101 hand-rolled impls now.

The *most* elegant form — a `trait RViaBase { type Base; fn from_base/into_base }`
with **blanket** `impl<X: RViaBase> TryFromSexp for Vec<X> where Vec<X::Base>:
TryFromSexp` — needs no per-type call at all, but **fails Rust coherence**: the
native `Vec<i32>` / `Vec<f64>` concrete impls overlap a blanket `Vec<X>` unless
the compiler can prove `i32: !RViaBase` (it can't, without `min_specialization`).
So: macro, not blanket. Document this so the next person doesn't re-derive the
dead end.

## M2 — `IntoR` has no scalar→container macro *at all*

The from_r side has the list macros; the **`IntoR` side has none** — even the
list/NULL integrations that use `impl_*_list!` inbound hand-roll their `IntoR`
`Option<X>`/`Vec<X>`/`Vec<Option<X>>` outbound. The shape is invariant: the
4-method wrapper (Finding 5) around `self.map/iter(transform).into_sexp()`.
Either an `impl_into_r_containers!(X, transform)` macro, or blanket impls
`IntoR for Option<T>/Vec<T>/Box<[T]> where T: IntoR`. **Check first** why the
blanket doesn't already exist: the native numeric types have bespoke
`Vec<Option<T>>`/`Box<[T]>` `IntoR` impls (smart-i64, coerce, alloc paths) that
a blanket would collide with under coherence — so a macro (opt-in per type) is
the safer lever than a blanket.

## M3 — `Box<[T]>` / `Box<[Option<T>]>` is never macroized (0 of all impls)

Across every shape in every trait, `Box<[_]>` has **0 macro-generated** members
and 8 `Box<[X]>` + 4 `Box<[Option<X>]>` hand-rolled `TryFromSexp` impls (plus 4
`IntoR`). Every one delegates to the `Vec` impl + `into_boxed_slice()`. A single
blanket `impl<T> TryFromSexp for Box<[T]> where Vec<T>: TryFromSexp` (and the
`IntoR` dual) erases all of them. This is the same lever as Part-1 Finding 2
(the `*_boxed_slice_*` macros) — together they retire the entire `Box<[…]>`
hand-rolling surface.

## M4 — `Coerce` widening one-liners aren't macroized

`coerce.rs:579-612` hand-writes five infallible widenings
(`impl Coerce<u16> for u8 { self.into() }`, `<i16> for i8`, `<i16> for u8`,
`<u32> for u8`, `<u32> for u16`). They are the infallible dual of the
`impl_try_*!` narrowing macros (Part-1 Finding 4). An `impl_widen!` macro
covers them:

```rust
macro_rules! impl_widen { ($($from:ty => $to:ty),+ $(,)?) => { $(
    impl Coerce<$to> for $from { #[inline(always)] fn coerce(self) -> $to { self.into() } }
)+ }; }
impl_widen!(u8 => u16, u8 => i16, i8 => i16, u8 => u32, u16 => u32);
```

## M5 — Verify the list macros are used in full where applicable

`bitvec_impl.rs` uses `impl_option_try_from_sexp!` but still hand-rolls its
`Vec` side (2 container impls); `toml_impl.rs` uses only the `Option` macro.
Each list/NULL-backed integration should invoke **all three** applicable list
macros, not a subset — a quick per-file consistency pass.

## Combined priority (Parts 1 + 2)

1. **P1-F4 + M4** — `TryCoerce` narrow (5→1) and `Coerce` widen one-liners.
   Trivial, zero-risk, same file. Do together first.
2. **P1-F2 + M3** — retire the whole `Box<[…]>` surface (macros + hand-rolled)
   via `Vec`-delegating blankets. One coherent change.
3. **M1 + M2** — string/scalar-backed container macro family (`from_r` + `IntoR`).
   Biggest line win (~150 impls across both directions); most behavioural care
   (NA-of-string vs NULL semantics — keep each type's current trigger).
4. **P1-F3** — `IntoRAs` target/container fork → one macro each.
5. **P1-F1** — `_vec` / `_vec_option` free-fn shell unification.
6. **P1-F5** — crate-wide `IntoR` wrapper-method boilerplate (sealed core trait).

All behaviour-preserving. After each, re-run `rustdoc_impl_inventory.py` and
diff the `for`-type set — it must be unchanged (same impls exist; only the
macro count drops).

---

# Part 3 — Implementation plan

## Decision: which macro vehicle? (declarative vs proc-macro)

The conversion impls live in `miniextendr-api`, and the existing 54 macros are
all declarative `macro_rules!`. Three candidate vehicles for the new
consolidating macros:

| Vehicle | Fit | Verdict |
|---|---|---|
| **(a) declarative `macro_rules!`** in `miniextendr-api` | Function-like; takes `(X, Base, from_fn, into_fn)` and stamps the family. In-crate, zero new deps, consistent with the existing 54 macros, fast to compile, good hygiene for this shape. | **Use this.** |
| **(b) function-like proc-macro** in `miniextendr-macros` | `miniextendr_via_base! { BigInt, base = String, … }`. Richer arg parsing, better error spans, can emit per-type doc comments. But cross-crate, heavier, and the win over (a) is marginal for a fixed 4-argument shape. | Only if (a)'s error messages/doc-gen become a real pain. Not the starting point. |
| **(c) `#[derive(RConvert)]`** proc-macro | **Impossible for the work at hand**: `BigInt` / `OrderedFloat` / `Complex` are *foreign* types — you cannot place a `#[derive]` on a type defined in another crate, and the orphan rule blocks deriving conversions "for" them from our crate. | Reserve for a *future, separate* feature (below). Not applicable to deduping the existing optionals. |

**The deciding fact is the orphan rule.** Every type we're deduping is foreign
(`num_bigint::BigInt`, `ordered_float::OrderedFloat`, `num_complex::Complex`,
`jiff::*`, `uuid::Uuid`, …). A derive needs to annotate the type's *definition*,
which we don't own; an attribute/derive can't attach to a foreign type. So the
conversions must be written as free-standing `impl` blocks in our crate, and the
right tool to stamp out free-standing impls from a compact spec is a
**function-like declarative macro**. Proc-macro machinery buys nothing here that
`macro_rules!` doesn't, and costs a crate boundary.

**Where a proc-macro *derive* genuinely wins (future, out of scope):** an
end-user who defines their *own* newtype (`struct Money(Decimal)`,
`struct UserId(Uuid)`) in their package owns that type and could write
`#[derive(RVia)] #[rvia(base = Decimal)] struct Money(Decimal);` to get all
six conversions for free. That's a real ergonomics feature for downstream
packages, it slots into the existing `miniextendr-macros` derive ecosystem
(`#[derive(ExternalPtr)]`, `#[derive(DataFrameRow)]`, …), and the declarative
`impl_via_base!` from this plan becomes its codegen backend. **File as its own
enhancement issue** — it is additive scope, not dedup, and shipping it does not
require the dedup to land first (or vice-versa).

## Macros to introduce (the toolkit)

1. `impl_try_narrow!($target; $($from),+)` — collapses `impl_try_i32/u8/u16/i16/i8`. (P1-F4)
2. `impl_widen!($($from => $to),+)` — the infallible `Coerce` duals. (M4)
3. `impl_via_base!($X, $Base, $from_fn, $into_fn)` — the centrepiece. Emits, for
   both `TryFromSexp` and `IntoR`: `X`, `Option<X>`, `Vec<X>`, `Vec<Option<X>>`,
   `Box<[X]>`, `Box<[Option<X>]>` by delegating to `$Base`'s existing container
   conversions and mapping through `$from_fn` / `$into_fn`. (M1 + M2)
4. `Box<[T]>` / `Box<[Option<T>]>` delegation to `Vec` — a tiny blanket-ish macro
   (or, where coherence permits, a real blanket) used by everything. (P1-F2 + M3)
5. `impl_vec_into_r_as!($target_ty, $target_name; $from, $from_name)` — one macro
   for both `Vec`/`&[]` and all three targets. (P1-F3)
6. free fn `from_numeric_vec_with(sexp, elem_map)` — the shared SEXP-dispatch
   shell behind `_vec` and `_vec_option`. (P1-F1)

## Flat work list (one PR each, in order)

Each PR: introduce/use the macro, delete the displaced impls, regenerate the
rustdoc inventory and **diff the `for`-type set (must be unchanged)**, run
`just test` + `just devtools-test` + the `clippy_all` feature set, and
`gctorture(TRUE)` over any touched no-arg fixture if SEXP storage is on the path.

1. **PR-1 `coerce` scalar macros** — add `impl_try_narrow!` + `impl_widen!`,
   replace the 5 narrowing macros (`coerce.rs:533-672`) and 5 widening
   one-liners (`:579-612`). Net: ~10 macros/impls → 2 macros. Zero risk.
2. **PR-2 retire `Box<[…]>`** — add the `Vec`-delegating macro/blanket; delete
   `impl_boxed_slice_option_try_from_sexp!` + `impl_boxed_slice_try_from_sexp_native!`
   and the ~12 hand-rolled `Box<[X]>`/`Box<[Option<X>]>` `TryFromSexp` + 4 `IntoR`
   impls. Resolves the bool-vs-f64 inconsistency.
3. **PR-3 `impl_via_base!` (from_r)** — introduce the macro; convert
   `num_bigint`, `rust_decimal`, `uuid`, `url`, `regex` (String base) and
   `ordered_float`, `num_complex` (f64/Rcomplex base). **Risk: preserve each
   type's existing `None` trigger** — `Option<X>` must inherit whatever
   `Option<Base>` already does (NA_character_ for String, NA_real_ for f64, NULL
   where present). The macro delegates to `Option<Base>`, so it inherits
   correctly *by construction* — but add a test per type asserting the trigger.
4. **PR-4 `impl_via_base!` (IntoR side)** — same types, outbound. Folds the
   4-method wrapper for these into the macro. (M2)
5. **PR-5 `jiff` / `time`** — the heaviest hand-rollers (24 + 12). Apply
   `impl_via_base!` per scalar type; some are ExternalPtr-backed (Span/DateTime)
   and stay as-is — only the String/f64-backed scalars convert.
6. **PR-6 `IntoRAs`** — `impl_vec_into_r_as!` unifying target × container.
7. **PR-7 `_vec`/`_vec_option` shell** — `from_numeric_vec_with`. Most
   behavioural care (NA semantics); smallest type set, easy to test exhaustively.
8. **PR-8 `IntoR` wrapper boilerplate** — sealed `IntoSexp` core + blanket `IntoR`
   (or `impl_infallible_into_r!`); uniform `_unchecked` policy. Largest surface,
   do last. (P1-F5)

## Risks / invariants to hold

- **`None`-trigger semantics must not change.** Today `Option<BigInt>` triggers
  on NA_character_, `Option<Complex>` on NA/NULL, `Option<ExternalPtr>` on NILSXP.
  `impl_via_base!` inherits the base's behaviour — verify, don't unify.
- **Blanket coherence**: a blanket `impl<X> … for Vec<X>` collides with the
  concrete native `Vec<i32>`/`Vec<f64>` impls without specialization — confirmed
  dead end (M1b). Stay with macros.
- **`_unchecked` fast paths**: the native numeric `Vec`/`Box` impls have real
  `*_unchecked` bodies — keep them; only the delegating wrappers may collapse
  unchecked to checked.
- **Inventory diff is the oracle**: the whole point of the corpus tooling is that
  every one of these PRs must leave `conversion-impl-inventory.md`'s `for`-type
  set byte-identical. If an impl disappears, behaviour changed — stop.
