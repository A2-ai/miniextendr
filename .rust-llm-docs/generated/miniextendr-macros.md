# miniextendr_macros v0.1.0

# miniextendr-macros - Procedural macros for Rust <-> R interop

This crate provides the procedural macros that power miniextendr's code
generation. Most users should depend on `miniextendr-api` and use its
re-exports, but this crate can be used directly when you only need macros.

Primary macros and derives:
- `#[miniextendr]` on functions, impl blocks, trait defs, and trait impls.
- `#[r_ffi_checked]` for main-thread routing of C-ABI wrappers.
- Derives: `ExternalPtr`, `RNativeType`, ALTREP derives, `RFactor`.
- Helpers: `typed_list` for typed list builders.

R wrapper generation is driven by Rust doc comments (roxygen tags are
extracted). The `document` binary collects these wrappers and writes
`R/miniextendr_wrappers.R` during package build.

## Quick start

```ignore
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

## Macro expansion pipeline

### Overview

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                         #[miniextendr] on fn                             │
│                                                                          │
│  1. Parse: syn::ItemFn → MiniextendrFunctionParsed                       │
│  2. Analyze return type (Result<T>, Option<T>, raw SEXP, etc.)           │
│  3. Generate:                                                            │
│     ├── C wrapper: extern "C-unwind" fn C_<name>(call: SEXP, ...) → SEXP │
│     ├── R wrapper: const R_WRAPPER_<NAME>: &str = "..."                  │
│     └── Registration: const call_method_def_<name>: R_CallMethodDef      │
│  4. Original function preserved (with added attributes)                  │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                    #[miniextendr(env|r6|s3|s4|s7)] on impl               │
│                                                                          │
│  1. Parse: syn::ItemImpl → extract methods                               │
│  2. For each method:                                                     │
│     ├── Generate C wrapper (handles self parameter)                      │
│     ├── Generate R method wrapper string                                 │
│     └── Generate registration entry                                      │
│  3. Generate class definition code per class system:                     │
│     ├── env: new.env() + method assignment                               │
│     ├── r6: R6Class() definition                                         │
│     ├── s3: S3 generics + methods                                        │
│     ├── s4: setClass() + setMethod()                                     │
│     └── s7: new_class() definition                                       │
│  4. Emit const with combined R code                                      │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                         #[miniextendr] on trait                          │
│                                                                          │
│  1. Parse: syn::ItemTrait → extract method signatures                    │
│  2. Generate:                                                            │
│     ├── Trait tag constant: const TAG_<TRAIT>: mx_tag = ...              │
│     ├── Vtable struct: struct __vtable_<Trait> { ... }                   │
│     └── CCalls table: static MX_CCALL_<TRAIT>: [...] = ...               │
│  3. Original trait preserved                                             │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                    #[miniextendr] impl Trait for Type                    │
│                                                                          │
│  1. Parse: syn::ItemImpl (trait impl)                                    │
│  2. Generate:                                                            │
│     ├── Vtable instance: static __VTABLE_<TRAIT>_FOR_<TYPE>: ...         │
│     ├── Wrapper struct: struct __MxWrapper<Type> { erased, data }        │
│     ├── Query function: fn __mx_query_<type>(tag) → vtable ptr           │
│     └── Base vtable: static __MX_BASE_VTABLE_<TYPE>: ...                 │
│  3. Original impl preserved                                              │
└──────────────────────────────────────────────────────────────────────────┘

```

### Key Modules

| Module | Purpose |
|--------|---------|
| `miniextendr_fn` | Function parsing and attribute handling |
| `c_wrapper_builder` | C wrapper generation (`extern "C-unwind"`) |
| `r_wrapper_builder` | R wrapper code generation |
| `rust_conversion_builder` | Rust→SEXP return value conversion |
| `miniextendr_impl` | `impl Type` block processing |
| `r_class_formatter` | Class system code generation (env/r6/s3/s4/s7) |
| `miniextendr_trait` | Trait ABI metadata generation |
| `miniextendr_impl_trait` | `impl Trait for Type` vtable generation |
| `altrep` / `altrep_derive` | ALTREP struct derivation |
| `externalptr_derive` | `#[derive(ExternalPtr)]` |
| `roxygen` | Roxygen doc comment handling |

### Generated Symbol Naming

For a function `my_func`:
- C wrapper: `C_my_func`
- R wrapper const: `R_WRAPPER_MY_FUNC`
- Registration: `call_method_def_my_func`

For a type `MyType` with trait `Counter`:
- Vtable: `__VTABLE_COUNTER_FOR_MYTYPE`
- Wrapper: `__MxWrapperMyType`
- Query: `__mx_query_mytype`

## Return Type Handling

The `return_type_analysis` module determines how to convert Rust returns to SEXP:

| Rust Type | Strategy | R Result |
|-----------|----------|----------|
| `T: IntoR` | `.into_sexp()` | Converted value |
| `Result<T, E>` | Unwrap or R error | Value or error |
| `Option<T>` | `Some` → value, `None` → `NULL` | Value or NULL |
| `SEXP` | Pass through | Raw SEXP |
| `()` | Invisible NULL | `invisible(NULL)` |

Use `#[miniextendr(unwrap_in_r)]` to return `Result<T, E>` to R without unwrapping.

## Thread Strategy

By default, `#[miniextendr]` functions run on R's main thread. Opt into
worker-thread execution with `#[miniextendr(worker)]` (requires the
`worker-thread` feature on `miniextendr-api`). A worker opt-in is ignored
when the signature requires main-thread execution (returns/takes `SEXP`,
uses variadic dots, or sets `check_interrupt`).

**Note**: `ExternalPtr<T>` is `Send` — it can be returned from worker
thread functions. All R API operations on `ExternalPtr` are serialized
through `with_r_thread`.

## Class Systems

The `r_class_formatter` module generates R code for different class systems:

| System | Generated R Code | Self Parameter |
|--------|------------------|----------------|
| `env` | `new.env()` with methods | `self` environment |
| `r6` | `R6Class()` | `self` environment |
| `s3` | `structure()` + generics | First argument |
| `s4` | `setClass()` + `setMethod()` | First argument |
| `s7` | `new_class()` | `self` property |
