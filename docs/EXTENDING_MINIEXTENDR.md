# Extending miniextendr

This guide explains how to extend miniextendr with custom types, enabling them to be passed between Rust and R.

## Quick Start

To make your type work with miniextendr, you have two main options:

1. **Implement `RNativeType`** - For types with the same memory layout as R's native types
2. **Implement `TryFromSexp`/`IntoR` directly** - For types requiring custom conversion logic

## Option 1: RNativeType (Recommended)

If your type has the same memory layout as `i32`, `f64`, `u8`, `RLogical`, or
`Rcomplex`, implement `RNativeType` to participate in the blanket scalar,
slice, vector, and collection conversions.

### Example: Newtype Wrapper

```rust
use miniextendr_api::{RNativeType, SEXP, SEXPTYPE, SexpExt};
use miniextendr_api::altrep_traits::NA_REAL;

/// A temperature in Celsius, stored as f64
#[repr(transparent)]
pub struct Celsius(pub f64);

impl RNativeType for Celsius {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;
    const R_NA: Self = Self(NA_REAL);

    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // Safe because Celsius is repr(transparent) over f64
        unsafe { miniextendr_api::sys::REAL(sexp).cast::<Self>() }
    }

    fn elt(sexp: SEXP, i: isize) -> Self {
        Self(sexp.real_elt(i))
    }
}
```

### What You Get Automatically

With just that impl, these all work:

```rust
// Scalar
fn get_temp() -> Celsius { ... }
fn set_temp(t: Celsius) { ... }

// Vectors
fn get_temps() -> Vec<Celsius> { ... }
fn process_temps(temps: &[Celsius]) { ... }

// Collections
fn temp_map() -> HashMap<String, Celsius> { ... }
fn temp_deque() -> VecDeque<Celsius> { ... }

// Optional integrations (with features enabled)
fn temp_tinyvec() -> TinyVec<[Celsius; 8]> { ... }
fn temp_nalgebra() -> DVector<Celsius> { ... }
fn temp_ndarray() -> Array1<Celsius> { ... }

// All Option<> variants
fn maybe_temp() -> Option<Celsius> { ... }
fn maybe_temps() -> Option<Vec<Celsius>> { ... }
```

### Requirements for RNativeType

Your type must:

1. **Be `#[repr(transparent)]`** over an R-native element representation
2. **Implement `Copy`** (required by the trait bound)
3. **Be `'static`** (no borrowed data)
4. **Define `R_NA`** for missing sparse/padded slots and implement both
   `dataptr_mut` and ALTREP-aware `elt`

### Memory Layout Correspondence

| Rust Type | R Type | SEXPTYPE |
|-----------|--------|----------|
| `i32` | integer | `INTSXP` |
| `f64` | numeric | `REALSXP` |
| `u8` | raw | `RAWSXP` |
| `RLogical` | logical | `LGLSXP` |
| `Rcomplex` | complex | `CPLXSXP` |

**Cannot be RNativeType**: `i8`, `i16`, `f32`, `i64`, `String` - no matching R storage type.

---

## Option 2: Direct TryFromSexp/IntoR Implementation

For types that don't match R's memory layout, implement the conversion traits directly.

### Example: Custom String Type

```rust
use miniextendr_api::sys::SEXP;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::into_r::IntoR;

pub struct Username(String);

impl TryFromSexp for Username {
    type Error = SexpError;
    // Reads what `String` reads: only character input.
    const CHARACTER_ONLY: bool = <String as TryFromSexp>::CHARACTER_ONLY;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: String = TryFromSexp::try_from_sexp(sexp)?;
        if s.is_empty() {
            return Err(SexpError::InvalidValue("username cannot be empty".into()));
        }
        Ok(Username(s))
    }
}

impl IntoR for Username {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.0.into_sexp())
    }
}
```

`try_into_sexp` and the associated `Error` are the required `IntoR` surface;
the panicking and unchecked convenience methods have defaults. Likewise, omit
`try_from_sexp_unchecked` unless your type has a real checked-wrapper bypass.

`TryFromSexp::CHARACTER_ONLY` says that the conversion accepts only character
input: every value it converts, other than `NULL`, is a character vector or a
factor. `#[miniextendr]` reads it for a `match_arg` / `choices` parameter typed
`Either<T, R>`, whose choice check takes every character or factor argument, so
an `R` with `CHARACTER_ONLY = true` is a compile error there (see [Choice or
Another Value](ENUMS_AND_FACTORS.md#choice-or-another-value-either-t-r)). Set
it to `true` when your impl reads only strings or factors, forward it (as
`Username` does) when it delegates to another conversion, and leave the
default `false` otherwise. `false` is always safe: the check is skipped. Never
set `true` on a type that also converts numbers, logicals or lists; that would
reject valid parameters.

### Example: Type Parsed From a String

For a type parsed out of an R string, use `try_from_sexp_via_str_parse!`. It
gives the type all four argument shapes, `T`, `Option<T>`, `Vec<T>` and
`Vec<Option<T>>`, with the same `NA` policy and error text as the built-in
uuid, url, regex and num-bigint conversions, which use the same macro:

```rust
use std::str::FromStr;

pub struct Slug(String);

impl FromStr for Slug {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || !s.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
            return Err("expected lowercase letters and '-'");
        }
        Ok(Slug(s.to_owned()))
    }
}

// The label names the value in a parse error: `invalid slug: <err>`.
miniextendr_api::try_from_sexp_via_str_parse!(Slug, "slug", |s| s.parse::<Slug>());

#[miniextendr]
pub fn count_slugs(x: Vec<Option<Slug>>) -> i32 {
    x.iter().flatten().count() as i32
}
```

The parse body is a closure-style `|s| expr` with `s: &str`, returning
`Result<T, E>` for any `E: Display`. It need not be `FromStr`. The four shapes
read R input like this:

| Rust type | `NA_character_` / `NULL` | Parse failure |
|-----------|--------------------------|---------------|
| `Slug` | `SexpError::Na` | `invalid slug: <err>` |
| `Option<Slug>` | `None` | `invalid slug: <err>` |
| `Vec<Slug>` | `NA is not allowed (element 2)` | `invalid slug: <err> (elements 3, 5)` |
| `Vec<Option<Slug>>` | `None` | `invalid slug: <err> (elements 3, 5)` |

A vector reports every failing element in one error, the elements numbered as
R counts them, with the first 10 listed and the rest summarized as `and N
more`. All four read only character input (`CHARACTER_ONLY`). `Box<[Slug]>`
converts as `Vec<Slug>` does.

The macro writes impls on your type only, which the orphan rule allows: the
parse step (`from_r::ParseRStr`), `TryFromSexp`, and `TryFromSexpElement`.
The three container impls can't be written in your crate (`Option` and `Vec`
are foreign to it, E0117). They are blankets in `miniextendr-api` over
`TryFromSexpElement`, the trait `#[derive(TryFromSexp)]` implements for a
newtype too. So don't use the macro on a type that also derives
`TryFromSexp`, and use it only on a non-generic type.

A newtype over a type that already converts (`struct UserId(Uuid)`) gets the
same container shapes from `#[derive(TryFromSexp)]`, each one where the inner
type's container converts.

### Example: Newtype That Refuses Some Values

The containers of an element type forward: `Option<T>` reads
`Option<T::Inner>`, `Vec<T>` reads `Vec<T::Inner>` and `Vec<Option<T>>` reads
`Vec<Option<T::Inner>>`, then each wraps what it read. None of them calls the
newtype's own `try_from_sexp`, so a check written there would hold for a
scalar argument and silently not for the containers. A value the inner type
accepts but the newtype must refuse is refused by a check on the R value,
`TryFromSexpElement::check_sexp`, which the derive takes as
`#[try_from_sexp(validate = path)]`:

```rust
use miniextendr_api::condition::RError;
use miniextendr_api::{SEXP, SexpExt, TryFromSexp, miniextendr};

/// A number with a unit class drops the unit (`difftime`) or counts from 1970
/// (`Date`, `POSIXct`).
fn plain_number(x: SEXP) -> Result<(), RError> {
    for class in [c"difftime", c"Date", c"POSIXt"] {
        if x.inherits_class(class) {
            let class = class.to_str().unwrap();
            return Err(RError::new(format!("got a {class}; give a plain number"))
                .class(["pkg_unit_error", "pkg_error"])
                .data("unit_class", class));
        }
    }
    Ok(())
}

#[derive(TryFromSexp)]
#[try_from_sexp(validate = plain_number)]
pub struct Elapsed(f64);

#[miniextendr]
pub fn total_time(x: Vec<Elapsed>) -> f64 {
    x.iter().map(|x| x.0).sum()
}
```

The check gets the argument's SEXP before the inner type reads it, attributes
and all, in every shape:

| Shape | When the check runs |
|-------|---------------------|
| `Elapsed` | before the inner conversion |
| `Option<Elapsed>` | on any input but `NULL`, which stays "not given" |
| `Vec<Elapsed>`, `Vec<Option<Elapsed>>` | once, on the whole vector, where R keeps its class |

It is a `fn(SEXP) -> Result<(), E>` with `E: Into<SexpError>`. Return an
`RError`, or any `RConditionError` type such as a `#[derive(RConditionError)]`
enum, for a refusal with its own classes and fields, or a plain `SexpError`
(`SexpError::InvalidValue(...)`) for one without. A classed refusal is the
argument error, with the parameter context around it:

```r
e <- tryCatch(total_time(Sys.Date()), error = identity)
conditionMessage(e)  # "invalid 'x' argument: got a Date; give a plain number"
class(e)             # "pkg_unit_error" "pkg_error" "rust_error" "simpleError" "error" "condition"
e$param              # "x"
e$unit_class         # "Date"
```

The message is `'<p>' must be <expected>: <message>` (`invalid '<p>'
argument: <message>` when the wrapper has no wording for the type): the check
sees a value, not the parameter, so the wrapper names it. The refusal's
classes come before the crate's `conversion_error_class`; `e$param` and
`e$rust_type` are added, `kind` is `"conversion"`, and the call is the
wrapper's call as written (`RError::without_call()` has no effect on an
argument error). Inside an `Either`, the refusal is reported with its classes
whenever the other arm refused the kind of value: an `Either<Option<Elapsed>,
DataFrame>` given a `difftime` raises the check's condition.

With `validate`, the newtype's scalar error is `SexpError` (every container's
already is), so the inner type's error must convert into it, as every built-in
conversion's does and every `RConditionError` type's does (its classes kept).
The attribute goes on the struct, and `validate` is its only key.

A hand-written `TryFromSexpElement` gets the same hook by overriding
`check_sexp`. Its scalar `TryFromSexp` must then call `Self::check_sexp`
itself, then convert the inner type and wrap it, and do nothing more: no
container runs it.

**Raising from a conversion.** A `TryFromSexp` impl can also raise a classed
condition with `rust_error!(class = "...", "...")`. That arrives with its
class and the wrapper's call as written, but as the function's error, not the
argument's: `kind = "error"`, no `e$param` or `e$rust_type`, no
`conversion_error_class`, and no parameter in the message. Inside an `Either`
arm it ends the conversion, so the other arm is never tried. For a refusal
that should read as an argument error, return it from a check instead.

### When to Use Direct Implementation

- Type requires validation and converts only as a scalar (like `Username`
  above; for a newtype that also converts in `Option` / `Vec`, use
  `#[try_from_sexp(validate = ...)]`)
- Type stores borrowed data
- Conversion involves complex transformation
- Type maps to R list or other complex structure

---

## Adding Coercion Support

If your type can be losslessly converted to/from R's numeric types, implement the marker traits:

```rust
use miniextendr_api::markers::{WidensToF64, WidensToI32};
use miniextendr_api::coerce::Coerce;

// If Celsius can be losslessly widened to f64
impl WidensToF64 for Celsius {}

// Now this works:
impl From<Celsius> for f64 {
    fn from(c: Celsius) -> f64 { c.0 }
}

// And you get automatic coercion:
// Vec<Celsius>.coerce() -> Vec<f64>
```

### Available Marker Traits

| Trait | Meaning | Use When |
|-------|---------|----------|
| `WidensToI32` | Losslessly converts to `i32` | 8/16-bit signed integers |
| `WidensToF64` | Losslessly converts to `f64` | Any numeric that fits in f64 |

---

## Working with ExternalPtr

For complex types that shouldn't be copied to R, use `ExternalPtr`:

```rust
use miniextendr_api::ExternalPtr;

#[derive(ExternalPtr)]
pub struct LargeDataset {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

// Now you can pass it by reference:
#[miniextendr]
fn create_dataset() -> ExternalPtr<LargeDataset> {
    ExternalPtr::new(LargeDataset { ... })
}

#[miniextendr]
fn process_dataset(data: &LargeDataset) -> f64 {
    data.data.iter().sum()
}
```

### When to Use ExternalPtr

- Large data structures (avoid copying)
- Mutable state between R calls
- Types that don't have R equivalents
- Opaque handles to Rust resources

---

## Complete Example: Custom Numeric Type

```rust
use miniextendr_api::{RNativeType, SEXP, SEXPTYPE, SexpExt};
use miniextendr_api::altrep_traits::NA_REAL;
use miniextendr_api::markers::WidensToF64;

/// Probability value in [0, 1]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Probability(f64);

impl Probability {
    pub fn new(value: f64) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

// Enable automatic conversions for all containers
impl RNativeType for Probability {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;
    const R_NA: Self = Self(NA_REAL);

    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        unsafe { miniextendr_api::sys::REAL(sexp).cast::<Self>() }
    }

    fn elt(sexp: SEXP, i: isize) -> Self {
        Self(sexp.real_elt(i))
    }
}

// Enable coercion to f64
impl WidensToF64 for Probability {}

impl From<Probability> for f64 {
    fn from(p: Probability) -> f64 {
        p.0
    }
}

// Now all these work:
// Vec<Probability>, &[Probability], DVector<Probability>, etc.
```

---

## Checklist for New Types

1. **Choose your approach**:
   - `#[repr(transparent)]` newtype over primitive? → `RNativeType`
   - Complex type or needs validation? → Direct `TryFromSexp`/`IntoR`
   - Large/mutable? → `ExternalPtr`

2. **Implement required traits**:
   - [ ] `RNativeType` OR `TryFromSexp` + `IntoR`
   - [ ] `Copy` (if using `RNativeType`)
   - [ ] `TypedExternal` (if using `ExternalPtr`)

3. **Optional enhancements**:
   - [ ] `WidensToI32`/`WidensToF64` for coercion
   - [ ] `Ord` for `BinaryHeap` support
   - [ ] `Hash` for `HashSet`/`HashMap` key support
   - [ ] `CHARACTER_ONLY` if the conversion reads only character or factor input

4. **Test your type**:
   - [ ] Scalar round-trip: Rust → R → Rust
   - [ ] Vector round-trip: `Vec<T>` both directions
   - [ ] Option handling: `None` ↔ `NULL` or `NA`
   - [ ] Edge cases: empty vectors, single elements
