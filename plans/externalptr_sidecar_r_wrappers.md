# ExternalPtr sidecar accessors in R wrappers (design)

## Goals
- Expose **R-facing accessors** for sidecar (`#[r_data]`) slots when those fields are `pub`.
- Keep the Rust API **SEXP-only** (no `Robj`).
- Avoid breaking existing ExternalPtr and class wrapper behavior.

## Non-goals
- Typed conversions for sidecar slots (SEXP-in/SEXP-out only).
- Supporting `...` in struct fields (Option A uses marker types).
- Auto-renaming or user-specified field names in `#[r_data]` (no `name = ...`).

## Constraints
- `#[derive(ExternalPtr)]` sees struct fields; `#[miniextendr] impl` **does not**.
- R wrapper strings are **compile-time constants**, so they cannot be assembled from
  other consts at macro-expansion time.
- `.Call` entrypoints **cannot be generic**.

## Rust-side markers (Option A)
```rust
#[derive(ExternalPtr)]
pub struct MyType {
    pub x: i32,

    // Selector
    #[r_data]
    pub r: RSidecar,

    // Sidecar slots
    #[r_data]
    pub prot_a: RData,
    #[r_data]
    pub prot_b: RData,
}
```
- `RSidecar` and `RData` are ZST markers.
- Only `pub` `#[r_data]` slots are exported to R wrappers.

## R wrapper strategy (baseline)
### 1) Standalone R functions (v1)
For each `pub` sidecar slot, generate two R functions:
- Getter: `<type>_get_<field>(x)`
- Setter: `<type>_set_<field>(x, value)`

Example (R):
```r
MyType_get_prot_a <- function(x) .Call(C__mx_rdata_get_MyType_prot_a, x)
MyType_set_prot_a <- function(x, value) {
  .Call(C__mx_rdata_set_MyType_prot_a, x, value)
  invisible(x)
}
```
Notes:
- Getter returns `SEXP` directly.
- Setter returns `invisible(x)` to enable chaining.
- These functions are **not class-system specific** and work with any ExternalPtr.

### 2) Registration wiring
The derive macro emits:
- `RDATA_CALL_DEFS_<TYPE>`: `&[R_CallMethodDef]`
- `R_WRAPPERS_RDATA_<TYPE>`: `&str` (R wrapper text, possibly empty)

`miniextendr_module!` is extended to automatically include these when `impl Type;`
(or `impl Type as "label";`) is registered. For types with no `pub` sidecar slots,
these are empty no-ops.

## Class integration (future/optional)
Auto-wiring into R6/S3/S4/S7 wrappers is **non-trivial** because the `#[miniextendr]`
impl macro cannot see struct fields and wrapper strings are const-only.

If we later want class-specific accessors:
- Add an explicit opt-in on the impl block, e.g. `#[miniextendr(r6, r_data_accessors)]`.
- Require a macro-friendly metadata channel from the derive (e.g., a `macro_rules!`
  that expands to the field list) **or** switch wrapper assembly to runtime string
  concatenation (bigger change).

For now, users can manually wrap the generated functions in R if they want
`$field`-style access.

## Naming & conflicts
- Use `<type>_get_<field>` / `<type>_set_<field>` to avoid collisions.
- The `<type>` portion uses the same **sanitized type name** rules as
  `miniextendr_module::type_name_sanitized`.
- If a generated function name would collide with an existing exported function,
  emit a compile error.

## Generic types
`.Call` wrappers cannot be generic. If a type has generic parameters **and**
`pub` `#[r_data]` fields, emit a compile error advising a concrete wrapper type.

## Thread safety
Accessors are `SEXP`-based and must run on R's main thread. The generated
`.Call` wrappers will be forced to main-thread execution (consistent with
existing `SEXP` handling in `miniextendr`).

## Tests
- UI tests:
  - Multiple selector fields error.
  - `#[r_data]` on non-marker types error.
  - Generic type with `pub` sidecar slot errors.
- Runtime tests:
  - Getter returns the stored SEXP.
  - Setter updates the sidecar slot and returns `invisible(x)`.
  - prot sidecar length matches `PROT_BASE_LEN + PROT_SIDECAR_LEN`.
