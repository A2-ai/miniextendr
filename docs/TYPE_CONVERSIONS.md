# Type Conversions in miniextendr

This guide documents how miniextendr converts between R and Rust types, including NA handling, coercion rules, and edge cases.

## Basic Type Mappings

### Scalar Types

| R Type | Rust Type | Notes |
|--------|-----------|-------|
| `integer` (length 1) | `i32` | NA → panic |
| `numeric` (length 1) | `f64` | NA preserved as `NA_REAL` |
| `logical` (length 1) | `bool` | NA → panic |
| `character` (length 1) | `String`, `&str` | NA → panic |
| `raw` (length 1) | `u8` | No NA in raw |
| `complex` (length 1) | `Rcomplex` | Has real/imag NA |

### Vector Types

| R Type | Rust Type | Notes |
|--------|-----------|-------|
| `integer` | `Vec<i32>`, `&[i32]` | NA = `i32::MIN` |
| `numeric` | `Vec<f64>`, `&[f64]` | NA = special bit pattern |
| `logical` | `Vec<i32>` | TRUE=1, FALSE=0, NA=`i32::MIN` |
| `character` | `Vec<String>` | NA → panic |
| `raw` | `Vec<u8>`, `&[u8]` | No NA |
| `list` | Various | See Lists section |

### Option Types (NA-Safe)

| R Type | Rust Type | NA Handling |
|--------|-----------|-------------|
| `integer` | `Option<i32>` | NA → `None` |
| `numeric` | `Option<f64>` | NA → `None` |
| `logical` | `Option<bool>` | NA → `None` |
| `character` | `Option<String>` | NA → `None` |

---

## NA Value Representation

### Integer NA

```rust
pub const NA_INTEGER: i32 = i32::MIN;  // -2147483648
```

In R, `NA_integer_` is represented as `i32::MIN`. This means:
- Valid integers: `-2147483647` to `2147483647`
- `i32::MIN` is reserved for NA

**Implication:** You cannot represent `i32::MIN` as a valid value in R integers.

### Logical NA

```rust
pub const NA_LOGICAL: i32 = i32::MIN;  // Same as integer
```

R logicals are stored as integers internally:
- `TRUE` = 1
- `FALSE` = 0
- `NA` = `i32::MIN`

### Real (Double) NA

```rust
pub const NA_REAL: f64 = f64::from_bits(0x7FF0_0000_0000_07A2);
```

R's `NA_real_` is a specific IEEE 754 NaN with a particular bit pattern.

**Critical:** This is different from regular `f64::NAN`:

```rust
// These are DIFFERENT values
let na = NA_REAL;           // R's NA
let nan = f64::NAN;         // Regular IEEE NaN

// Detection requires bit comparison
fn is_na_real(value: f64) -> bool {
    value.to_bits() == NA_REAL.to_bits()
}

// Regular NaN check does NOT detect NA
value.is_nan()  // Returns true for both NA and NaN
```

**Implication:** When working with `f64` vectors, regular NaN values pass through unchanged. Only `NA_REAL` is treated as NA.

### String NA

R's `NA_character_` is a special `CHARSXP` pointer (`R_NaString`).

miniextendr converts string NA to panic by default. Use `Option<String>` for NA-safe access:

```rust
#[miniextendr]
pub fn handle_string(s: Option<String>) -> String {
    s.unwrap_or_else(|| "was NA".to_string())
}
```

---

## Coercion System

miniextendr provides automatic type coercion for numeric types.

### Coercion Precedence

Two traits control coercion:

1. **`Coerce<R>`** - Infallible (always succeeds)
2. **`TryCoerce<R>`** - Fallible (can fail)

When both exist for a type pair, `Coerce` takes precedence:

```rust
// Blanket impl ensures Coerce always wins
impl<T, R> TryCoerce<R> for T where T: Coerce<R> {
    fn try_coerce(self) -> Result<R, Infallible> {
        Ok(self.coerce())
    }
}
```

### Infallible Coercions (Coerce)

| From | To | Notes |
|------|-----|-------|
| `i32` | `f64` | Widening (no precision loss) |
| `i32` | `i32` | Identity |
| `f64` | `f64` | Identity |
| `Option<T>` | `T` | `None` → NA value |

### Fallible Coercions (TryCoerce)

| From | To | Fails When |
|------|-----|------------|
| `f64` | `i32` | NaN, infinity, fractional, overflow |
| `i32` | `u32` | Negative value |
| `i32` | `NonZeroI32` | Zero value |
| `f64` | `u64` | Negative, NaN, overflow |

### Enabling Coercion

Use `#[miniextendr(coerce)]` to enable automatic coercion:

```rust
// Without coerce: f64 parameter requires numeric input
#[miniextendr]
pub fn square(x: f64) -> f64 { x * x }

// With coerce: accepts integer, coerces to f64
#[miniextendr(coerce)]
pub fn square_coerce(x: f64) -> f64 { x * x }
```

```r
square(2L)        # Error: expected numeric
square_coerce(2L) # 4.0 (integer coerced to double)
```

### Per-Parameter Coercion

```rust
#[miniextendr]
pub fn mixed(
    #[miniextendr(coerce)] x: f64,  // Coerce this one
    y: i32,                          // No coercion
) -> f64 {
    x + y as f64
}
```

---

## Option-to-NA Conversion

When returning `Option<T>`, `None` converts to R's NA:

```rust
#[miniextendr]
pub fn maybe_value(x: i32) -> Option<i32> {
    if x > 0 { Some(x) } else { None }
}
```

```r
maybe_value(5)   # 5
maybe_value(-1)  # NA
```

### Coercion for Options

`Option<T>` coerces to `T` with `None` → NA:

```rust
// This works with coercion enabled:
// R's NA_integer_ → None → coerced to NA_real_
#[miniextendr(coerce)]
pub fn option_coerce(x: f64) -> f64 { x }
```

---

## Vector NA Handling

### Reading Vectors with NA

For vectors with potential NA values, use `Option` element type:

```rust
#[miniextendr]
pub fn count_na(x: Vec<Option<i32>>) -> i32 {
    x.iter().filter(|v| v.is_none()).count() as i32
}
```

### Writing Vectors with NA

Return `Vec<Option<T>>` to include NA values:

```rust
#[miniextendr]
pub fn add_na_at_end(x: Vec<i32>) -> Vec<Option<i32>> {
    let mut result: Vec<Option<i32>> = x.into_iter().map(Some).collect();
    result.push(None);  // Adds NA
    result
}
```

---

## Slice Lifetimes

When using slice parameters (`&[T]`), be aware of lifetime implications:

```rust
// SAFE: Slice is only used during function execution
#[miniextendr]
pub fn sum(x: &[f64]) -> f64 {
    x.iter().sum()
}
```

The slice has a `'static` lifetime annotation, but this is a **lie** for API convenience. The actual lifetime is tied to R's GC protection of the SEXP.

**Safe patterns:**
- Use slice within the function
- Copy data if you need to store it

**Unsafe patterns:**
- Storing the slice in a struct that outlives the function
- Returning the slice (won't compile anyway)

---

## String Lifetimes

R interns all strings (`CHARSXP`). When you get a `&str` from R:

```rust
#[miniextendr]
pub fn process_string(s: &str) -> String {
    // s is valid for the entire R session (interned)
    s.to_uppercase()
}
```

The `&'static str` lifetime is actually correct here because R never garbage collects interned strings.

---

## ExternalPtr Semantics

When using `#[derive(ExternalPtr)]`:

```rust
#[derive(ExternalPtr)]
pub struct MyData {
    values: Vec<f64>,
}
```

The Rust data is heap-allocated and owned by R:

1. `new()` allocates Rust data on heap
2. Pointer stored in R's external pointer SEXP
3. R's GC tracks the SEXP
4. When SEXP is collected, Rust `Drop` runs
5. Heap memory freed

**Thread safety:** The pointer can be safely accessed from any thread, but R API calls must happen on the main thread.

---

## Complex Types

### Lists

Lists convert to various Rust types:

```rust
// Named list → HashMap
#[miniextendr]
pub fn process_map(x: HashMap<String, i32>) -> i32 {
    x.values().sum()
}

// List → Vec of heterogeneous items (requires SEXP)
#[miniextendr]
pub fn list_length(x: List) -> i32 {
    x.len() as i32
}
```

### Data Frames

Data frames are lists with special attributes. Access columns:

```rust
#[miniextendr]
pub fn get_column(df: List, name: &str) -> Vec<f64> {
    // df[name] returns the column
    // Convert as needed
}
```

### Matrices

With the `ndarray` feature:

```rust
use ndarray::Array2;

#[miniextendr]
pub fn matrix_sum(x: Array2<f64>) -> f64 {
    x.sum()
}
```

---

## Error Cases

### Type Mismatch

When R type doesn't match expected Rust type:

```rust
#[miniextendr]
pub fn needs_integer(x: i32) -> i32 { x }
```

```r
needs_integer(1.5)
# Error: failed to convert parameter 'x' to i32: wrong type
```

### NA in Non-Option

When NA is passed to non-Option parameter:

```rust
#[miniextendr]
pub fn needs_value(x: i32) -> i32 { x }
```

```r
needs_value(NA_integer_)
# Error: failed to convert parameter 'x' to i32: contains NA
```

### Coercion Failure

When coercion fails:

```rust
#[miniextendr(coerce)]
pub fn needs_int(x: i32) -> i32 { x }
```

```r
needs_int(1.5)
# Error: failed to coerce parameter 'x' to i32: fractional value
```

---

## Feature-Gated Types

Many additional types are available via Cargo features:

| Feature | Types |
|---------|-------|
| `num-bigint` | `BigInt`, `BigUint` |
| `rust_decimal` | `Decimal` |
| `uuid` | `Uuid` |
| `time` | `Date`, `Time`, `OffsetDateTime` |
| `ndarray` | `Array1`, `Array2`, etc. |
| `nalgebra` | `Matrix`, `Vector`, etc. |
| `indexmap` | `IndexMap`, `IndexSet` |
| `serde` | JSON conversion |
| `serde_r` | Native R serialization |

Enable in `Cargo.toml`:

```toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["uuid", "time"] }
```

---

## Best Practices

1. **Use `Option<T>` for NA-safe parameters**
   ```rust
   pub fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }
   ```

2. **Use slices for read-only vector access** (zero-copy)
   ```rust
   pub fn sum(x: &[f64]) -> f64 { x.iter().sum() }
   ```

3. **Use `Vec<T>` when you need to modify**
   ```rust
   pub fn double(x: Vec<i32>) -> Vec<i32> { x.into_iter().map(|v| v*2).collect() }
   ```

4. **Enable coercion for flexible numeric APIs**
   ```rust
   #[miniextendr(coerce)]
   pub fn flexible(x: f64) -> f64 { x }
   ```

5. **Return `Option<T>` to produce NA values**
   ```rust
   pub fn maybe(x: i32) -> Option<i32> { if x > 0 { Some(x) } else { None } }
   ```
