# API choice matrix

miniextendr has several places where two or more APIs achieve the same goal
with different tradeoffs. This page is a single-screen reference: for each
decision point, the **safer / stricter / more validated** option is on the
left, the **easier / faster / less protective** option is on the right, and
the row tells you when to pick which. Each link points at the rustdoc landing
page and the existing in-depth doc.

For the long-form decision trees see [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md),
[ALTREP.md](ALTREP.md), [STRICT_MODE.md](STRICT_MODE.md), and
[ERROR_HANDLING.md](ERROR_HANDLING.md). This page is the index, not the
textbook.

## At a glance

| Decision | Stricter / safer | Looser / faster | Pick stricter when... |
|---|---|---|---|
| **Rust → R for lossy ints** (`i64`/`u64`/`isize`/`usize`) | `#[miniextendr(strict)]` → `strict::checked_*` helpers (panic on overflow) | default `IntoR` (silently widens to `REALSXP` outside `[-2^53, 2^53]`) | the caller's data is *supposed to fit* the type you declared and a silent precision loss would mask a bug |
| **Strict mode (project-wide)** | `strict-default` feature | strict opted-in per-fn | you'd rather a misuse fail loudly across the whole package |
| **R → Rust shape mismatch** | `TryFromSexp` (returns `Result` on wrong R type) | `Coerce<R>` (widens across SEXP types) | the input is *supposed to be* one R type and accepting others would mask a caller bug |
| **Numeric narrowing across types** | `TryCoerce<R>` (fallible) | `Coerce<R>` (infallible widening) | the source range exceeds the target type |
| **Conversion impl style** | hand-rolled `TryFromSexp + IntoR` | `#[derive(RSerializeNative)]` (serde feature) | you need full control over the SEXP layout, zero-overhead, or your shape doesn't match serde's data model |
| **FFI primitive** | checked (`Rf_foo`) | `Rf_foo_unchecked` | you're not provably inside an ALTREP callback, `with_r_unwind_protect`, or `with_r_thread` block. MXL301 enforces this. |
| **Error emission** | `error!("msg", class = "my_class")` (typed condition) | `panic!(msg)` (becomes `rust_panic`) | R-side callers may want to `tryCatch` your specific error class. Never `Rf_error` directly — MXL300 forbids. |
| **Error propagation** | `Result<T, E: std::error::Error>` + `?` | embed-the-error-in-the-return-type | you want value-style propagation through Rust code; convert at the boundary via `RErrorAdapter` / `unwrap_in_r` opt-out |
| **ALTREP path** | `#[altrep(manual)]` + handwritten `AltrepLen`/`Alt*Data` | `#[derive(AltrepInteger)]` field-based | the field-based derive can't express your storage (custom backing, computed-on-access, etc.) |
| **ALTREP guard** | `r_unwind` (calls `with_r_unwind_protect`) | `rust_unwind` (default `catch_unwind`) | the ALTREP callback calls R API functions that may longjmp. Use `unsafe` only when callbacks are pure Rust and panic-free. |
| **Variadic args** | `#[miniextendr(dots = typed_list!(...))]` | `_dots: &Dots` raw access | you want missing/extra/wrong-type field errors at the macro call site, not at runtime |
| **Choice arg** | `#[miniextendr]` + `MatchArg<MyEnum>` | manual `match` on `String` | you want R-side `match.arg`-style partial matching and a default value |
| **GC protection** | `ProtectScope` (RAII, LIFO) within `.Call` | manual `Rf_protect` / `Rf_unprotect` | always. The RAII variant is correct by construction; manual is correct only by audit. |
| **Long-lived SEXP** | `preserve::insert` / `preserve::release` (cross-.Call) | `ProtectScope` (single .Call) | the SEXP needs to survive past your function's return |
| **Rust data owned by R** | `ExternalPtr<MyStruct>` | sidecar fields (multi-SEXP attribute slots) | one owned struct is the natural shape. Sidecars are for cases where R-side code wants to read individual fields without crossing the Rust boundary. |
| **Worker thread** | `worker-thread` feature on (default) | inline (off) | always, unless you've measured a hot path and confirmed inline-without-longjmp-safety is acceptable |
| **Thread re-entry** | wrap R calls in `with_r_thread` | call from any thread | you're on the worker thread or any non-main thread (which is *always* the case for user `#[miniextendr]` code) |

## Class systems (one-of-six)

This is its own decision because the six are mutually exclusive per type.
See [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) for the full flowchart; the
one-line summary is:

| System | Pick when... | Avoid when... |
|---|---|---|
| **Env** | you want raw R env semantics, fastest dispatch, no formal class | you need inheritance or method dispatch beyond `$`-lookup |
| **R6** | you want mutable, reference-semantics OOP with lifecycle hooks | you need value semantics or formal type checking |
| **S3** | the type is simple and dispatch can ride on `class()` ordering | you need formal slot validation or multi-dispatch |
| **S4** | you need formal validation, slots, multi-dispatch, and you accept the cost | startup time and `methods::` package loading is a concern |
| **S7** | you want formal validation + value semantics + inheritance (newer than S4, similar power) | you need to interop with older S4-heavy code |
| **Vctrs** | the type is a *vector* that should integrate with tidyverse `vctrs` | the type isn't vector-shaped |

Default class system is controlled by the mutually-exclusive
`r6-default` and `s7-default` features. Per-type override via the
`#[miniextendr]` attribute.

## Defaults that change behavior

A few cargo features shift the project-wide defaults; flipping them
changes which row of the matrix above is in force without changing any
call sites:

| Feature | What it changes |
|---|---|
| `strict-default` | All `#[miniextendr]` lossy conversions reject NA / truncation by default |
| `coerce-default` | Numeric scalars use `TryCoerce` path by default |
| `r6-default` / `s7-default` | Picks the default class system for `#[miniextendr] impl` blocks without an explicit attr |
| `worker-default` | All `#[miniextendr]` fns dispatch through the worker thread (implies `worker-thread`) |

See [FEATURE_DEFAULTS.md](FEATURE_DEFAULTS.md) for the full story.

## When in doubt

Pick the **left column** (stricter). The framework's default opinion is
"fail loudly, leave a trail." The looser variants exist for cases where
you've measured the cost or confirmed the looser semantics are correct
for your data.

For AI coding assistants: if you find yourself reaching for the right
column because it's easier, pause and ask whether the left column would
catch a real bug. The looser path is correct for *some* problems; it is
not the default.

## Related reading

- [STRICT_MODE.md](STRICT_MODE.md) — the strict-vs-lax conversion story
- [CONVERSION_MATRIX.md](CONVERSION_MATRIX.md) — full R × Rust type behavior
- [COERCE.md](COERCE.md) / [AS_COERCE.md](AS_COERCE.md) — coercion paths
- [PREFER_DERIVES.md](PREFER_DERIVES.md) — derive-selector attribute
- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) — class system decision tree + flowchart
- [ALTREP.md](ALTREP.md) — ALTREP field-derive vs manual flowchart
- [ALTREP_GUARDS.md](ALTREP_GUARDS.md) — guard mode tradeoffs
- [EXTERNALPTR.md](EXTERNALPTR.md) — `Box<Box<dyn Any>>` storage + provenance
- [ERROR_HANDLING.md](ERROR_HANDLING.md) — panic / error! / Result paths
- [CONDITIONS.md](CONDITIONS.md) — condition class layering
- [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md) — variadic + validation
- [FFI_GUARD.md](FFI_GUARD.md) — checked/unchecked FFI boundary
- [THREADS.md](THREADS.md) — worker-thread architecture
- [FEATURE_DEFAULTS.md](FEATURE_DEFAULTS.md) — project-wide knobs
