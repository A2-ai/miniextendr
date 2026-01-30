# Plan: Deeper vctrs Integration (Rust-Inferred S3 Vectors)

Goal: Generate full vctrs‑compatible S3 vector classes from Rust types with minimal annotations, covering vctrs invariants, coercion/casting, proxy/restore, record/list‑of, equality/comparison, and arithmetic/math.

## 1) vctrs features to cover (from background/vctrs-main)

Core class mechanics:

- `new_vctr()` base class + constructor helper
- `format()` + `vec_ptype_abbr()`
- `vec_proxy()` + `vec_restore()`
- `vec_size()` / `vec_ptype()` compatibility

Coercion + casting:

- `vec_ptype2()` (double dispatch)
- `vec_cast()` (double dispatch)
- `vec_cast_common()` / `vec_ptype_common()` integration

Record / list‑of types:

- `new_rcrd()` for record classes
- `new_list_of()` for list‑of classes

Comparison and ordering:

- `vec_proxy_equal()`
- `vec_proxy_compare()`
- `vec_proxy_order()`

Arithmetic and math:

- `vec_math()`
- `vec_arith()` (double dispatch)

## 2) Rust surface syntax (inferred)

### 2.1 Class declaration

```rs
#[derive(Vctrs)]
#[vctrs(class = "pkg_class", base = "double" | "integer" | "list" | "record")]
struct Percent { data: Vec<f64> }
```

### 2.2 Attributes (type + method)

```rs
#[vctrs(format)]
fn format(&self) -> Vec<String> { ... }

#[vctrs(ptype_abbr)]
fn ptype_abbr() -> &'static str { ... }

#[vctrs(proxy)]
fn proxy(&self) -> SEXP { ... }

#[vctrs(restore)]
fn restore(data: SEXP, to: &Self) -> Self { ... }

#[vctrs(ptype2)]
fn ptype2(&self, other: &SEXP) -> SEXP { ... }

#[vctrs(cast)]
fn cast(&self, to: &SEXP) -> SEXP { ... }

#[vctrs(proxy_equal)]
fn proxy_equal(&self) -> SEXP { ... }

#[vctrs(proxy_compare)]
fn proxy_compare(&self) -> SEXP { ... }

#[vctrs(proxy_order)]
fn proxy_order(&self) -> SEXP { ... }

#[vctrs(arith)]
fn arith(op: &str, x: &Self, y: &SEXP) -> SEXP { ... }

#[vctrs(math)]
fn math(op: &str, x: &Self) -> SEXP { ... }
```

## 3) Inference rules

- Underlying data vector inferred from struct fields or `#[vctrs(base=...)]`.
- If `base=record`, map fields to `new_rcrd(list(...))`.
- If `base=list`, use `new_list_of()` and infer `ptype` from element type.
- If `format()` absent, generate default using `vec_data()`.
- If `vec_restore()` not provided and attributes are data‑independent, default restore copies attributes.

## 4) Codegen strategy

- Generate `new_<class>()` constructor and `as_<class>()` cast helper.
- Emit `vec_proxy.<class>()` / `vec_restore.<class>()` methods.
- Emit `vec_ptype2.<class>.<class>()` and `vec_cast.<class>.<class>()` by default.
- For additional coercions, allow Rust‑defined `#[vctrs(ptype2)]` / `#[vctrs(cast)]` methods.
- Generate `vec_proxy_equal/compare/order` when annotated.
- Generate `vec_arith.<class>` + double‑dispatch helpers when annotated.
- Generate `vec_math.<class>` for math ops when annotated.

### 4.1 Example: Percent (double base)

Rust:

```rust
#[derive(Vctrs)]
#[vctrs(class = "percent", base = "double")]
pub struct Percent {
    data: Vec<f64>,
}

#[vctrs(format)]
fn format(x: &Percent) -> Vec<String> { ... }

#[vctrs(ptype_abbr)]
fn ptype_abbr() -> &'static str { "prcnt" }

#[vctrs(cast)]
fn cast(x: &Percent, to: &SEXP) -> SEXP { ... }
```

Generated R (sketch):

```r
new_percent <- function(x = double()) {
  .Call(C_new_percent, .call = match.call(), x)
}

format.percent <- function(x, ...) {
  .Call(C_format_percent, .call = match.call(), x, list(...))
}

vec_ptype_abbr.percent <- function(x, ...) {
  .Call(C_vec_ptype_abbr_percent, .call = match.call(), x, list(...))
}

vec_proxy.percent <- function(x, ...) {
  .Call(C_vec_proxy_percent, .call = match.call(), x, list(...))
}

vec_restore.percent <- function(x, to, ...) {
  .Call(C_vec_restore_percent, .call = match.call(), x, to, list(...))
}

vec_ptype2.percent.percent <- function(x, y, ...) {
  .Call(C_vec_ptype2_percent_percent, .call = match.call(), x, y, list(...))
}

vec_cast.percent.percent <- function(x, to, ...) {
  .Call(C_vec_cast_percent_percent, .call = match.call(), x, to, list(...))
}
```

### 4.2 Example: Record class (rational)

Rust:

```rust
#[derive(Vctrs)]
#[vctrs(class = "rational", base = "record")]
pub struct Rational {
    n: Vec<i32>,
    d: Vec<i32>,
}

#[vctrs(proxy_equal)]
fn proxy_equal(x: &Rational) -> SEXP { ... }
```

Generated R (sketch):

```r
new_rational <- function(n = integer(), d = integer()) {
  .Call(C_new_rational, .call = match.call(), n, d)
}

vec_proxy_equal.rational <- function(x, ...) {
  .Call(C_vec_proxy_equal_rational, .call = match.call(), x, list(...))
}
```

### 4.3 Example: List-of class (polynomial)

Rust:

```rust
#[derive(Vctrs)]
#[vctrs(class = "poly_list", base = "list")]
pub struct PolyList {
    data: Vec<Vec<i32>>,
}
```

Generated R (sketch):

```r
new_poly_list <- function(x = list()) {
  .Call(C_new_poly_list, .call = match.call(), x)
}
```

## 5) Rust feature gates

- Default `vctrs` feature includes all codegen.
- Optional `vctrs-minimal` disables arithmetic/math/proxy‑compare features.

## 6) Tests (rpkg/tests/testthat)

- Constructor + format/ptype_abbr
- vec_proxy/restore round‑trip
- vec_ptype2/vec_cast with base types
- Record/list‑of creation and coercion
- proxy_equal/compare/order correctness
- vec_arith and vec_math coverage

## 7) Docs/examples

- Add Rust examples for:
  - simple vctr (percent)
  - record vctr (rational)
  - list‑of vctr (polynomial)
- Update docs to map Rust annotations to vctrs methods
