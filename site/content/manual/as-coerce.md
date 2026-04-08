+++
title = "`as.<class>()` Coercion Methods"
weight = 21
description = "This document describes how to implement R's as.<class>() coercion generics for Rust types using the #[miniextendr(as = \"...\")] attribute."
+++

This document describes how to implement R's `as.<class>()` coercion generics for Rust types using the `#[miniextendr(as = "...")]` attribute.

## Overview

R has a standard pattern for type coercion using `as.<class>()` generics like `as.data.frame()`, `as.list()`, `as.character()`, etc. The `#[miniextendr(as = "...")]` attribute allows you to implement these methods for your Rust types, generating proper S3 method dispatching.

## Quick Example

```rust
use miniextendr_api::{miniextendr, List, ProtectScope, ListBuilder, IntoR};
use miniextendr_api::as_coerce::AsCoerceError;

#[derive(ExternalPtr)]
pub struct Person {
    name: String,
    age: i32,
}

#[miniextendr(s3)]
impl Person {
    pub fn new(name: String, age: i32) -> Self {
        Self { name, age }
    }

    /// Convert to data.frame
    #[miniextendr(as = "data.frame")]
    pub fn as_data_frame(&self) -> Result<List, AsCoerceError> {
        unsafe {
            let scope = ProtectScope::new();
            let builder = ListBuilder::new(&scope, 2);

            builder.set(0, scope.protect_raw(vec![self.name.clone()].into_sexp()));
            builder.set(1, scope.protect_raw(vec![self.age].into_sexp()));

            Ok(builder.into_list()
                .set_names_str(&["name", "age"])
                .set_class_str(&["data.frame"])
                .set_row_names_int(1))
        }
    }

    /// Convert to character representation
    #[miniextendr(as = "character")]
    pub fn as_character(&self) -> Result<String, AsCoerceError> {
        Ok(format!("{} (age {})", self.name, self.age))
    }
}
```

This generates R S3 methods:

```r
as.data.frame.Person <- function(x, ...) {
    .Call(C_Person__as_data_frame, .call = match.call(), x)
}

as.character.Person <- function(x, ...) {
    .Call(C_Person__as_character, .call = match.call(), x)
}
```

## Supported Coercion Types

| `as = "..."` value | R Generic | Use Case |
|--------------------|-----------|----------|
| `"data.frame"` | `as.data.frame()` | Tabular data |
| `"list"` | `as.list()` | Named collections |
| `"character"` | `as.character()` | String representation |
| `"numeric"` | `as.numeric()` | Numeric vector |
| `"double"` | `as.double()` | Double precision vector |
| `"integer"` | `as.integer()` | Integer vector |
| `"logical"` | `as.logical()` | Logical vector |
| `"matrix"` | `as.matrix()` | Matrix representation |
| `"vector"` | `as.vector()` | Generic vector |
| `"factor"` | `as.factor()` | Factor conversion |
| `"Date"` | `as.Date()` | Date objects |
| `"POSIXct"` | `as.POSIXct()` | DateTime objects |
| `"complex"` | `as.complex()` | Complex numbers |
| `"raw"` | `as.raw()` | Raw bytes |
| `"environment"` | `as.environment()` | Environments |
| `"function"` | `as.function()` | Functions |

## Error Handling

The `AsCoerceError` enum provides structured error types:

```rust
use miniextendr_api::as_coerce::AsCoerceError;

// Conversion not supported
Err(AsCoerceError::NotSupported {
    from: "MyType",
    to: "matrix",
})

// Invalid data prevents conversion
Err(AsCoerceError::InvalidData {
    message: "columns have different lengths".into(),
})

// Precision loss during numeric conversion
Err(AsCoerceError::PrecisionLoss {
    message: "value exceeds i32 range".into(),
})

// Custom error message
Err(AsCoerceError::Custom("something went wrong".into()))
```

Errors are automatically converted to R errors with the message from `Display`.

## Return Types

Methods can return various types:

```rust
// Return SEXP directly
#[miniextendr(as = "numeric")]
pub fn as_numeric(&self) -> Result<ffi::SEXP, AsCoerceError> { ... }

// Return List (for data.frame, list)
#[miniextendr(as = "data.frame")]
pub fn as_data_frame(&self) -> Result<List, AsCoerceError> { ... }

// Return String (converted to character vector)
#[miniextendr(as = "character")]
pub fn as_character(&self) -> Result<String, AsCoerceError> { ... }
```

## Creating data.frames

Use `ListBuilder` for heterogeneous column types:

```rust
#[miniextendr(as = "data.frame")]
pub fn as_data_frame(&self) -> Result<List, AsCoerceError> {
    unsafe {
        let scope = ProtectScope::new();
        let n_cols = 3;
        let builder = ListBuilder::new(&scope, n_cols);

        // String column
        builder.set(0, scope.protect_raw(self.names.clone().into_sexp()));

        // Numeric column
        builder.set(1, scope.protect_raw(self.values.clone().into_sexp()));

        // Integer column
        builder.set(2, scope.protect_raw(self.counts.clone().into_sexp()));

        let n_rows = self.names.len();
        Ok(builder.into_list()
            .set_names_str(&["name", "value", "count"])
            .set_class_str(&["data.frame"])
            .set_row_names_int(n_rows))
    }
}
```

## Class System Compatibility

The `as = "..."` attribute works with **all class systems** - S3, R6, S7, S4, env, and vctrs. R's `as.*()` generics dispatch based on the class attribute, which all miniextendr class systems set appropriately.

```rust
// Works with any class system:

#[miniextendr(s3)]
impl MyS3Type {
    #[miniextendr(as = "data.frame")]
    pub fn as_data_frame(&self) -> Result<List, AsCoerceError> { ... }
}

#[miniextendr(r6)]
impl MyR6Type {
    #[miniextendr(as = "list")]
    pub fn as_list(&self) -> Result<List, AsCoerceError> { ... }
}

#[miniextendr(s7)]
impl MyS7Type {
    #[miniextendr(as = "character")]
    pub fn as_character(&self) -> Result<String, AsCoerceError> { ... }
}

#[miniextendr(env)]
impl MyEnvType {
    #[miniextendr(as = "data.frame")]
    pub fn as_data_frame(&self) -> Result<List, AsCoerceError> { ... }
}
```

All of these generate proper S3 method wrappers that R can dispatch to.
```

## Best Practices

1. **Use `Result<T, AsCoerceError>`** - Always return a Result to allow error handling
2. **Validate data** - Check for inconsistencies before conversion
3. **Document the conversion** - Add doc comments explaining what the conversion produces
4. **Use appropriate return types** - Use `List` for data.frame/list, `SEXP` for vectors
5. **Set proper attributes** - data.frames need class, names, and row.names

## Traits (Optional)

The `miniextendr_api::as_coerce` module also provides traits you can implement directly:

```rust
use miniextendr_api::as_coerce::{AsDataFrame, AsCoerceError};

impl AsDataFrame for MyType {
    fn as_data_frame(&self) -> Result<List, AsCoerceError> {
        // ...
    }
}
```

However, the `#[miniextendr(as = "...")]` attribute is the recommended approach as it generates the R wrappers automatically.
