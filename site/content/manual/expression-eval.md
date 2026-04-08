+++
title = "Expression Evaluation Helpers"
weight = 25
+++

Safe wrappers for building and evaluating R function calls from Rust.

## Types

| Type | Purpose |
|------|---------|
| `RSymbol` | Interned R symbol (SYMSXP) -- never GC'd |
| `RCall` | Builder for R function calls (LANGSXP) |
| `REnv` | Well-known R environments (Global, Base, Empty) |

## Quick Example

```rust
use miniextendr_api::expression::{RCall, REnv};
use miniextendr_api::ffi::Rf_mkString;

unsafe {
    // Call paste0("hello", " world") in base
    let result = RCall::new("paste0")
        .arg(Rf_mkString(c"hello".as_ptr()))
        .arg(Rf_mkString(c" world".as_ptr()))
        .eval(REnv::base().as_sexp())?;
}
```

## RSymbol

Wraps R's `Rf_install()` for interned symbols. Symbols are never garbage collected, so `RSymbol` needs no GC protection.

```rust
use miniextendr_api::expression::RSymbol;

// From a Rust string (allocates a CString)
let sym = unsafe { RSymbol::new("my_var") };

// From a C string literal (zero allocation)
let sym = unsafe { RSymbol::from_cstr(c"my_var") };

// Use as SEXP
let sexp = sym.as_sexp();
```

## RCall

Builds R function calls with positional and named arguments.

```rust
use miniextendr_api::expression::RCall;

unsafe {
    // Positional arguments
    let result = RCall::new("sum")
        .arg(my_vector_sexp)
        .eval(env)?;

    // Named arguments
    let result = RCall::new("paste")
        .arg(x_sexp)
        .arg(y_sexp)
        .named_arg("sep", sep_sexp)
        .eval(env)?;
}
```

### Error Handling

`eval()` uses `R_tryEvalSilent` and returns `Result<SEXP, String>`. On failure, the error message comes from R's `geterrmessage()`.

```rust
match RCall::new("stop").arg(msg_sexp).eval(env) {
    Ok(result) => { /* success */ },
    Err(msg) => { /* msg contains R's error message */ },
}
```

### GC Protection

`RCall` protects all intermediate SEXPs (the call object and argument list) during construction. The **returned SEXP is unprotected** -- caller must protect it if it will survive across R API calls.

## REnv

Provides handles to R's well-known environments:

```rust
use miniextendr_api::expression::REnv;

let global = REnv::global();  // R_GlobalEnv
let base = REnv::base();      // R_BaseEnv
let empty = REnv::empty();    // R_EmptyEnv

// Use as SEXP
let sexp = base.as_sexp();
```

## Safety Requirements

All functions in this module require:

- Being called from the **R main thread** (they use R API calls)
- `unsafe` blocks (they call into C)

In `#[miniextendr]` functions, use these inside `unsafe(main_thread)` blocks or within ALTREP callbacks (which already run on the main thread).

## Use Cases

- **S4 slot access**: The `s4_helpers` module uses `RCall` internally
- **Calling R functions from ALTREP callbacks**: When `elt()` needs to call R
- **Dynamic dispatch**: Building R function calls based on runtime data
- **Package interop**: Calling functions from other R packages

## See Also

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#s4-helpers-module) -- S4 helpers built on RCall
- [THREADS.md](THREADS.md) -- Main thread requirements
- [GC_PROTECT.md](GC_PROTECT.md) -- Protecting returned SEXPs
