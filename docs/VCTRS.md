# vctrs Integration with `#[derive(Vctrs)]`

miniextendr provides the `#[derive(Vctrs)]` macro to create vctrs-compatible S3 vector classes from Rust structs. These types integrate seamlessly with the tidyverse ecosystem.

## Quick Start

```rust
use miniextendr_api::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "percent", base = "double", abbr = "%")]
pub struct Percent {
    #[vctrs(data)]
    values: Vec<f64>,
}
```

This generates a vctrs vector type that:
- Prints with `%` abbreviation in tibbles
- Preserves class through subsetting, combining, and operations
- Supports coercion with `vec_ptype2`/`vec_cast`

## Vector Types

### Simple Vectors (`base = "double"`, `"integer"`, etc.)

Backed by a single atomic vector.

```rust
#[derive(Vctrs)]
#[vctrs(class = "temperature", base = "double", abbr = "°C")]
pub struct Temperature {
    #[vctrs(data)]
    celsius: Vec<f64>,
}
```

**R usage:**
```r
t <- new_temperature(c(20.0, 25.0, 30.0))
t[1:2]              # Still temperature class
vctrs::vec_c(t, t)  # Combines correctly
```

### Record Types (`base = "record"`)

Multiple parallel fields, like a data frame row.

```rust
#[derive(Vctrs)]
#[vctrs(class = "rational", base = "record")]
pub struct Rational {
    #[vctrs(data)]
    n: Vec<i32>,  // numerator
    d: Vec<i32>,  // denominator
}
```

**R usage:**
```r
r <- new_rational(c(1L, 2L), c(2L, 3L))  # 1/2, 2/3
vctrs::field(r, "n")  # c(1L, 2L)
format(r)             # "1/2", "2/3"
```

### List-of Types (`base = "list"`)

Lists where each element has the same prototype.

```rust
#[derive(Vctrs)]
#[vctrs(class = "int_lists", base = "list", ptype = "integer()")]
pub struct IntLists {
    #[vctrs(data)]
    lists: Vec<Vec<i32>>,
}
```

**R usage:**
```r
x <- new_int_lists(list(1:3, 4:6))
x[[1]]              # 1:3
x[1]                # Still int_lists class
```

## Attributes Reference

### Container-Level Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `class = "name"` | Yes | R class name |
| `base = "type"` | No | Base type: `"double"`, `"integer"`, `"logical"`, `"character"`, `"list"`, `"record"`. Default: `"double"` |
| `abbr = "str"` | No | Abbreviation for `vec_ptype_abbr` (shown in tibble headers) |
| `ptype = "expr"` | No | R expression for list-of prototype, e.g., `"integer()"` |
| `coerce = "type"` | No | Generate coercion methods with this type (can repeat) |
| `inherit_base` | No | Include base type in class vector. Default: `true` for list/record, `false` otherwise |

### Field-Level Attributes

| Attribute | Description |
|-----------|-------------|
| `#[vctrs(data)]` | Mark as the underlying data field (required for `IntoVctrs`) |
| `#[vctrs(skip)]` | Exclude from record fields |

### Advanced Feature Attributes

| Attribute | Description |
|-----------|-------------|
| `proxy_equal` | Generate `vec_proxy_equal` for equality testing |
| `proxy_compare` | Generate `vec_proxy_compare` for comparison/sorting |
| `proxy_order` | Generate `vec_proxy_order` for ordering |
| `arith` | Generate `vec_arith` methods for arithmetic operations |
| `math` | Generate `vec_math` methods for math functions |

## Proxy Methods

Control how vctrs compares, sorts, and orders your type.

```rust
#[derive(Vctrs)]
#[vctrs(class = "point", base = "record", proxy_equal, proxy_compare, proxy_order)]
pub struct Point {
    #[vctrs(data)]
    x: Vec<f64>,
    y: Vec<f64>,
}
```

**Generated methods:**
- `vec_proxy_equal.point()` - Used by `vec_equal()`, `vec_unique()`
- `vec_proxy_compare.point()` - Used by `vec_compare()`, `sort()`
- `vec_proxy_order.point()` - Used by `vec_order()`, `order()`

For record types, the proxy is a data frame of the fields, enabling lexicographic comparison.

## Arithmetic Operations

Enable arithmetic with the `arith` attribute.

```rust
#[derive(Vctrs)]
#[vctrs(class = "meter", base = "double", abbr = "m", arith)]
pub struct Meter {
    #[vctrs(data)]
    values: Vec<f64>,
}
```

**Generated methods:**
- `vec_arith.meter()` - Base dispatcher for double dispatch
- `vec_arith.meter.meter()` - meter op meter
- `vec_arith.meter.numeric()` - meter op numeric
- `vec_arith.numeric.meter()` - numeric op meter
- `vec_arith.meter.MISSING()` - Unary operations (`-x`, `+x`)

**R usage:**
```r
m <- new_meter(c(1.0, 2.0))
m + m           # meter: 2, 4
m * 2           # meter: 2, 4
2 * m           # meter: 2, 4
-m              # meter: -1, -2
```

The result preserves the `meter` class. Operations use `vec_arith_base()` internally.

## Math Functions

Enable math functions with the `math` attribute.

```rust
#[derive(Vctrs)]
#[vctrs(class = "positive", base = "double", math)]
pub struct Positive {
    #[vctrs(data)]
    values: Vec<f64>,
}
```

**Generated method:**
- `vec_math.positive()` - Handles `abs`, `sqrt`, `log`, `exp`, etc.

**R usage:**
```r
p <- new_positive(c(4.0, 9.0, 16.0))
sqrt(p)         # positive: 2, 3, 4
abs(p)          # positive: 4, 9, 16
log(p)          # positive: 1.39, 2.20, 2.77
```

## Cross-Type Coercion

Allow your type to coerce with other types using `coerce = "type"`.

```rust
#[derive(Vctrs)]
#[vctrs(class = "percent", base = "double", coerce = "double")]
pub struct Percent {
    #[vctrs(data)]
    values: Vec<f64>,
}
```

**Generated methods:**
- `vec_ptype2.percent.double()` / `vec_ptype2.double.percent()` - Common type is `percent`
- `vec_cast.percent.double()` - Cast double to percent
- `vec_cast.double.percent()` - Cast percent to double (strips class)

**R usage:**
```r
p <- new_percent(c(0.25, 0.50))
vctrs::vec_c(p, 0.75)           # percent: 0.25, 0.50, 0.75
vctrs::vec_cast(0.5, new_percent(double()))  # percent: 0.5
```

## Complete Example

A fully-featured vctrs type with all advanced features:

```rust
#[derive(Vctrs)]
#[vctrs(
    class = "measurement",
    base = "double",
    abbr = "msr",
    coerce = "double",
    arith,
    math
)]
pub struct Measurement {
    #[vctrs(data)]
    values: Vec<f64>,
}
```

**R usage:**
```r
m <- new_measurement(c(1.0, 2.0, 3.0))

# Printing
print(m)                    # <measurement[3]> 1 2 3
tibble::tibble(x = m)       # Shows "msr" in header

# Subsetting
m[1:2]                      # measurement preserved

# Combining
vctrs::vec_c(m, m)          # measurement preserved

# Arithmetic
m + m                       # measurement: 2, 4, 6
m * 2                       # measurement: 2, 4, 6
2 * m                       # measurement: 2, 4, 6

# Math
sqrt(m)                     # measurement: 1, 1.41, 1.73
abs(-m)                     # measurement: 1, 2, 3

# Coercion with double
vctrs::vec_c(m, 4.0)        # measurement: 1, 2, 3, 4
```

## Module Registration

Register vctrs types in `miniextendr_module!`:

```rust
miniextendr_module! {
    mod mypackage;
    vctrs Percent;
    vctrs Rational;
    vctrs Measurement;
}
```

## Generated R Methods

For a type with all features enabled, the macro generates:

| Method | Purpose |
|--------|---------|
| `format.<class>()` | String representation |
| `vec_ptype_abbr.<class>()` | Abbreviation (if `abbr` set) |
| `vec_ptype_full.<class>()` | Full type name |
| `vec_proxy.<class>()` | Underlying data for operations |
| `vec_restore.<class>()` | Restore class after subsetting |
| `vec_ptype2.<class>.<class>()` | Self-coercion prototype |
| `vec_cast.<class>.<class>()` | Self-cast (identity) |
| `vec_proxy_equal.<class>()` | Equality proxy (if `proxy_equal`) |
| `vec_proxy_compare.<class>()` | Comparison proxy (if `proxy_compare`) |
| `vec_proxy_order.<class>()` | Ordering proxy (if `proxy_order`) |
| `vec_arith.<class>()` | Arithmetic dispatcher (if `arith`) |
| `vec_arith.<class>.<class>()` | Class-class arithmetic |
| `vec_arith.<class>.numeric()` | Class-numeric arithmetic |
| `vec_arith.numeric.<class>()` | Numeric-class arithmetic |
| `vec_arith.<class>.MISSING()` | Unary operations |
| `vec_math.<class>()` | Math functions (if `math`) |

## Tips

1. **Always mark one field with `#[vctrs(data)]`** - This is required for `IntoVctrs` to work.

2. **Use `abbr` for tibble display** - Short abbreviations look better in tibble column headers.

3. **Record fields are ordered** - Field order in the struct determines format output order.

4. **Arithmetic preserves class** - The result of `meter + meter` is `meter`, not `double`.

5. **Consider what operations make sense** - Not all types should support all arithmetic. A `date` might support subtraction but not multiplication.

## See Also

- [vctrs package documentation](https://vctrs.r-lib.org/)
- [vctrs S3 vector guide](https://vctrs.r-lib.org/articles/s3-vector.html)
- `rpkg/tests/testthat/test-vctrs-derive.R` - Test examples
- `rpkg/src/rust/vctrs_derive_example.rs` - Rust examples
