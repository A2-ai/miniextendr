+++
title = "Strict Mode"
weight = 41
description = "Strict mode rejects lossy type conversions that would silently widen or truncate values. When enabled, out-of-range values panic instead of being coerced."
+++

Strict mode rejects lossy type conversions that would silently widen or truncate
values. When enabled, out-of-range values panic instead of being coerced.

## What It Affects

Strict mode applies to **lossy integer types** only — types where R's native
integer range (`i32`, excluding `NA_integer_`) cannot represent all values:

| Type | Scalar | Vec | Option | Vec\<Option\> |
|------|--------|-----|--------|---------------|
| `i64` | Yes | Yes | Yes | Yes |
| `u64` | Yes | Yes | Yes | Yes |
| `isize` | Yes | Yes | Yes | Yes |
| `usize` | Yes | Yes | Yes | Yes |

Types like `i32`, `f64`, `String`, `bool` are **not affected** — they have
lossless R representations.

## Behavior Comparison

### Output (Rust → R)

**Normal mode:** Values outside `i32` range silently widen to `f64` (REALSXP).

```rust
// Normal: 2^40 silently becomes double
let big: i64 = 1_099_511_627_776;
big.into_sexp()  // → REALSXP (double)
```

**Strict mode:** Values outside `i32` range panic.

```rust
#[miniextendr(strict)]
pub fn strict_big() -> i64 {
    1_099_511_627_776  // PANIC: i64 value outside R integer range
}
```

Valid range: `(i32::MIN, i32::MAX]` — note `i32::MIN` (-2147483648) is excluded
because it represents `NA_integer_` in R.

### Input (R → Rust)

**Normal mode:** Accepts INTSXP, REALSXP, RAWSXP, and LGLSXP with coercion.

```r
# Normal: logical TRUE coerced to i64
my_func(TRUE)   # OK → 1i64
my_func(1L)     # OK → 1i64
my_func(1.0)    # OK → 1i64
my_func(as.raw(1))  # OK → 1i64
```

**Strict mode:** Only accepts INTSXP and REALSXP. Rejects RAWSXP and LGLSXP.
REALSXP values must be whole numbers in range.

```r
# Strict: only numeric types accepted
strict_func(1L)     # OK
strict_func(1.0)    # OK (whole number)
strict_func(1.5)    # PANIC: fractional value
strict_func(TRUE)   # PANIC: expected integer or double, got LGLSXP
strict_func(as.raw(1))  # PANIC: expected integer or double, got RAWSXP
```

## How to Enable

### Per-Function

```rust
#[miniextendr(strict)]
pub fn exact_value(x: i64) -> i64 { x }
```

### Per-Impl Block

```rust
#[miniextendr(s3, strict)]
impl Calculator {
    pub fn compute(&self, n: u64) -> u64 { n * 2 }
}
```

### Per-Function Override

```rust
#[miniextendr(strict)]               // impl-level: strict
impl Calculator {
    pub fn normal_method(&self, x: i64) -> i64 { x }     // strict (inherited)

    #[miniextendr(no_strict)]
    pub fn relaxed_method(&self, x: i64) -> i64 { x }    // not strict (overridden)
}
```

### Project-Wide Default

In `Cargo.toml`:

```toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["default-strict"] }
```

When `default-strict` is enabled, all `#[miniextendr]` functions default to strict.
Use `#[miniextendr(no_strict)]` to opt out.

## Error Messages

Strict panics produce descriptive messages:

```
strict conversion failed: i64 value 1099511627776 is outside R integer range
(-2147483647..=2147483647); use a non-strict function to allow lossy f64 widening
```

```
strict conversion failed for parameter 'count': expected integer or double, got LGLSXP
```

These panics are caught and converted to R errors.

## When to Use Strict Mode

**Use strict when:**
- Building numeric-critical APIs where silent precision loss is a bug
- Working with database IDs, timestamps, or bit flags that must be exact
- You want to catch accidental type coercions during development

**Skip strict when:**
- Values are known to be small (fit in i32)
- Approximate f64 representation is acceptable
- You want maximum R interoperability (accept any numeric-like input)

## See Also

- [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md) — `strict` attribute reference
- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) — Normal conversion behavior
- [FEATURE_DEFAULTS.md](FEATURE_DEFAULTS.md) — Project-wide feature flags
