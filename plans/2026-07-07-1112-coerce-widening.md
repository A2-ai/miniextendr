# #1112 — coerce option: widen accepted SEXP types instead of narrowing

Branch: `fix/1112-coerce-widening`. **GATED on maintainer decision** (widen
coerce? — needs-decision list). Re-verified relevant 2026-07-11 @ 17f634d8.
NOTE: textually overlaps `fix/1217-macro-vec-coercion-batching` in
`rust_conversion_builder.rs` — whichever lands second rebases.

**Decision embedded** (recommended in the ledger; flag in PR for sign-off):
make coerce a **strict superset** of bare acceptance. Today
`coerce`/`coerce-default` *replaces* `T::try_from_sexp` with
"extract mapped R-native type, then `TryCoerce`" — so a `bool` param under
coerce accepts `1L` but **rejects `TRUE`**, and multi-source integer types
narrow to INTSXP-only. `docs/FEATURE_DEFAULTS.md` promises widening.

## Anchors (verified on main @ 88c493fd)

- `miniextendr-macros/src/miniextendr_fn.rs:16-110` — `CoercionMapping` +
  `from_type()`: scalars `u16/i16/i8/u32/u64/i64/isize/usize/bool` → `i32`,
  `f32` → `f64`; `Vec<T>` variants likewise.
- `miniextendr-macros/src/rust_conversion_builder.rs:471-530` — the emission:
  `Some(CoercionMapping::Scalar { r_native, target })` / `::Vec { .. }` arms
  replace the bare conversion.
- `rpkg/tests/testthat/test-feature-defaults.R:22-45` — pins the *current*
  narrowing behavior ("coerce-default converts bool params from R integers";
  logical-rejected-under-coerce assertions) — these flip deliberately.

## The fix: bare-first, coerce-fallback

In the two `rust_conversion_builder.rs` arms, emit a two-stage conversion
instead of a replacement:

```rust
// scalar arm (sketch)
match <#target as TryFromSexp>::try_from_sexp(#sexp) {
    Ok(v) => v,
    Err(primary) => {
        let native: #r_native = TryFromSexp::try_from_sexp(#sexp)
            .map_err(|_| primary)?;          // report the BARE error on double failure
        TryCoerce::try_coerce(native).map_err(|_| primary)?
    }
}
```

(Adapt to the builder's actual emission style — it builds token streams around
`error_in_r` handling; keep whatever `.map_err`→condition plumbing the current
arms use. The load-bearing choices:)

- **Bare conversion first** → every input accepted without coerce is accepted
  with coerce. Superset by construction; happy path cost unchanged.
- **On double failure, surface the bare error** (it names the expected R type
  users see everywhere else). Mention the coerce attempt only if trivially
  cheap to phrase.
- **Vec arm**: same shape over the slice leg. Watch the `&[#r_native]`
  extraction — the fallback still goes through the native slice + element-wise
  `TryCoerce`; per-element failures should flow into #1192's batching if that
  lands first (coordinate; whichever merges second adapts).

Consequence worth stating in the PR: for multi-source integer types
(`u16`, `i64`, …) the fallback rarely adds acceptance (bare is already
multi-source per `docs/CONVERSION_MATRIX.md`) — after this change `coerce`
mainly matters for `bool` (gains INTSXP) and documents-as-superset semantics.
That's the honest, defensible meaning of the knob.

## Steps

1. Rework the two emission arms; update `CoercionMapping`'s doc comment
   (`miniextendr_fn.rs:37-49`) to describe superset semantics.
2. Rewrite `docs/FEATURE_DEFAULTS.md`'s coerce paragraph: "coerce = bare
   acceptance **plus** conversion from the R-native scalar type (`f32` from
   `f64`, `bool` from integer, …)".
3. Flip the pins in `test-feature-defaults.R`: logical accepted under coerce
   (both `TRUE` and `1L` paths asserted), f32 both-paths, i64 unchanged
   multi-source. Keep the strict/coerce matrix-row comment (:30-33) accurate.
4. Macro snapshot tests rebaseline; UI tests if wording changed.
5. Regen + test under the coerce leg locally:
   `CARGO_FEATURES="coerce-default ..." just rcmdinstall && just devtools-test`
   — **export in the same shell as both install AND test** (MEMORY:
   lesson_cargo_features_env_per_shell). Then default-leg run too.
6. Three CI clippy legs + fmt; feature-legs workflow is the real acceptance
   gate (coerce rides the s7 leg per test comments — confirm in
   `.github/workflows/ci.yml`).

## Done

- Under `coerce-default`: `bool` accepts `TRUE` *and* `1L`; no type accepts
  less than bare; docs match implementation; pins flipped deliberately with
  the PR body explaining the semantic change. `Closes #1112`.
