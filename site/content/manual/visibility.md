+++
title = "Visibility and Export Control"
weight = 50
description = "#[miniextendr] functions exist at three scopes: the Rust binary (C symbol), the package namespace (callable inside the package), and the public API (importable by other packages). Rust pub and the #[miniextendr] export attributes control which scope each function occupies. This document explains the mapping."
+++

`#[miniextendr]` functions exist at three scopes: the Rust binary (C symbol), the package
namespace (callable inside the package), and the public API (importable by other packages).
Rust `pub` and the `#[miniextendr]` export attributes control which scope each function
occupies. This document explains the mapping.

---

## Three-Tier Model

Every `#[miniextendr]` function always gets a C symbol registered in `R_CallMethodDef`.
What varies is whether the generated R wrapper carries `@export`, `@keywords internal`,
or neither.

| Rust visibility | `#[miniextendr]` option | `@export` | `@keywords internal` | NAMESPACE |
|-----------------|------------------------|-----------|----------------------|-----------|
| `pub fn` | (none) | yes | no | exported |
| `pub fn` | `noexport` | no | no | not in NAMESPACE |
| `pub fn` | `internal` | no | yes | not in NAMESPACE |
| non-`pub fn` | (none) | no | no | not in NAMESPACE |
| non-`pub fn` | `export` | yes | no | exported |

Non-`pub` functions behave identically to `pub` + `noexport`: an R wrapper is generated and
the C symbol is registered, but no `@export` is emitted.

---

## The C Symbol Is Always Registered

Every `#[miniextendr]` function — regardless of Rust visibility or export flags —
produces a `R_CallMethodDef` entry and an R wrapper. This means `.Call(C_fn_name, ...)`
works from any R code inside the package, even for non-exported functions.

NAMESPACE controls importability from _outside_ the package. It has no effect on
`.Call()` visibility within the package itself.

```rust
// C_internal_helper is callable via .Call() from R code in the same package,
// but does not appear in NAMESPACE and cannot be imported by downstream packages.
fn internal_helper(x: i32) -> i32 {
    x + 1
}
```

---

## Function Attribute Reference

| Attribute | Effect on R wrapper |
|-----------|---------------------|
| `noexport` | Omit `@export`; wrapper exists and C symbol is registered |
| `internal` | Omit `@export`; add `@keywords internal`; appears in `?help` only when searched directly |
| `export` | Force `@export` on a non-`pub` function |
| `r_name = "..."` | Rename the R wrapper (e.g. `r_name = "is.widget"`); does not affect NAMESPACE membership |
| `c_symbol = "..."` | Rename the C symbol used in `.Call()` and `R_CallMethodDef` |

### When to use each option

| Goal | Use |
|------|-----|
| Public API function | `pub fn` (default) |
| `pub` for Rust trait bounds, but package-internal | `#[miniextendr(noexport)]` |
| Internal helper that should appear in `?help` search | `#[miniextendr(internal)]` |
| Non-`pub` function that must be exported | `#[miniextendr(export)]` (rare; prefer making the fn `pub`) |
| Rename the R-facing name | `r_name = "my.function"` |
| Rename the C symbol (e.g. to avoid collision) | `c_symbol = "pkg_my_fn"` |

### Mutually exclusive combinations

The following combinations are compile errors:

- `internal + noexport` — `internal` is a strict superset; drop `noexport`
- `export + noexport` — contradictory
- `export + internal` — contradictory

---

## Examples

### Default: exported public function

```rust
#[miniextendr]
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

```r
# Generated:
#' @export
add <- function(x, y) .Call(C_add, x, y)
```

`add` appears in `NAMESPACE` and is importable by downstream packages.

### noexport: suppress NAMESPACE entry

```rust
// pub needed so this type's method can call it via a trait bound,
// but we don't want it in the public API.
#[miniextendr(noexport)]
pub fn validate_internal(x: i32) -> bool {
    x > 0
}
```

No `@export` is emitted. The R wrapper exists and `.Call(C_validate_internal, ...)` works
inside the package, but `validate_internal` is not in NAMESPACE.

### internal: hide from normal help search

```rust
#[miniextendr(internal)]
pub fn debug_repr(x: i32) -> String {
    format!("debug: {}", x)
}
```

```r
# Generated:
#' @keywords internal
debug_repr <- function(x) .Call(C_debug_repr, x)
```

`@keywords internal` hides the function from `help.search()` results by default. It remains
accessible to R users who know the name (`?debug_repr` still works) but does not clutter
discovery.

### export: force-export a non-pub function

```rust
// Rare — prefer making the function pub instead.
#[miniextendr(export)]
fn legacy_compat() -> i32 {
    42
}
```

`@export` is emitted despite the function not being `pub` in Rust.

### r_name: rename the R wrapper

```rust
#[miniextendr(r_name = "is.widget")]
pub fn is_widget(x: i32) -> bool {
    x == 1
}
```

The R wrapper is named `is.widget` (valid R identifier style). The C symbol remains
`C_is_widget`. NAMESPACE gets `export(is.widget)`.

---

## Impl and Class-Level Export Control

The `noexport` and `internal` flags can be applied to an entire `impl` block, suppressing
or marking all methods at once.

```rust
// All methods in this impl are internal
#[miniextendr(internal)]
impl DebugType {
    pub fn dump(&self) -> String { ... }
    pub fn inspect(&self) -> i32 { ... }
}
```

```rust
// Suppress @export on the whole class
#[miniextendr(noexport)]
impl InternalHelper {
    pub fn run(&self) -> i32 { ... }
}
```

Individual methods can override the impl-level flag:

```rust
#[miniextendr(noexport)]
impl MyType {
    pub fn internal_method(&self) -> i32 { ... }

    // Override: this one is exported despite the block-level noexport
    #[miniextendr(export)]
    pub fn public_method(&self) -> String { ... }
}
```

---

## Lint Rules

### MXL106: non-`pub` function not exported

```
warning[MXL106]: registered function `my_helper` is not `pub`
```

This fires when a `#[miniextendr]` function is not `pub`. The C symbol is registered
and the R wrapper exists, but users cannot call it from outside the package. Fix: add
`pub`, or add `#[miniextendr(export)]` if you intentionally want to export a non-`pub` fn.

### MXL203: `internal + noexport` redundancy

```
warning[MXL203]: `internal` and `noexport` are redundant together
```

`internal` already suppresses `@export` _and_ adds `@keywords internal`. Drop `noexport`.

```rust
// Bad
#[miniextendr(internal, noexport)]
pub fn helper() -> i32 { 42 }

// Good
#[miniextendr(internal)]
pub fn helper() -> i32 { 42 }
```

---

## Quick Decision Guide

```
Is the function part of the package's public API?
├── Yes → pub fn (default)
├── No — should it appear in help search?
│   ├── Yes, but not exported → pub fn + #[miniextendr(internal)]
│   └── No → non-pub fn  (or pub + #[miniextendr(noexport)])
└── Need pub for Rust trait bounds only? → pub fn + #[miniextendr(noexport)]
```

---

## See Also

- [MINIEXTENDR_ATTRIBUTE.md](../miniextendr-attribute/) — complete `#[miniextendr]` option reference
- [CLASS_SYSTEMS.md](../class-systems/) — class-level export control details
- [MACRO_ERRORS.md](../macro-errors/) — MXL106, MXL203, and other lint details
