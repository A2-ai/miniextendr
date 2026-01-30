# Minimal Reproduction Case

This document provides a minimal reproduction of the heterogeneous type issue in the DataFrameRow macro.

## Minimal Test Case

### Input Struct
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct SimplePerson {
    pub name: String,
    pub age: i32,
}
```

### Expected Generated Code
```rust
pub struct SimplePersonDataFrame {
    pub name: Vec<String>,
    pub age: Vec<i32>
}

impl From<Vec<SimplePerson>> for SimplePersonDataFrame {
    fn from(rows: Vec<SimplePerson>) -> Self {
        SimplePersonDataFrame {
            name: rows.iter().map(|r| r.name.clone()).collect::<Vec<String>>(),
            age: rows.iter().map(|r| r.age.clone()).collect::<Vec<i32>>()
        }
    }
}
```

### Actual Error
```
error[E0308]: mismatched types
  --> dataframe_examples.rs:26:24
   |
26 | #[derive(Clone, Debug, DataFrameRow)]
   |                        ^^^^^^^^^^^^ expected `Vec<String>`, found `Vec<i32>`
   |
   = note: expected struct `Vec<std::string::String>`
              found struct `Vec<i32>`
   = note: this error originates in the derive macro `DataFrameRow` (in Nightly builds, run with -Z macro-backtrace for more info)
```

## Core Macro Logic

The macro implementation uses this pattern:

```rust
pub fn derive_dataframe_row(input: DeriveInput) -> syn::Result<TokenStream> {
    // Extract fields
    let field_info: Vec<(&Ident, &Type)> = fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            (name, ty)
        })
        .collect();

    // Generate DataFrame struct
    let df_fields_tokens: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { pub #name: Vec<#ty> }
        })
        .collect();

    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name {
            #(#df_fields_tokens),*
        }
    };

    // Generate From impl with explicit types
    let mut from_struct_tokens = TokenStream::new();
    for (i, (name, ty)) in field_info.iter().enumerate() {
        if i > 0 {
            from_struct_tokens.extend(quote! { , });
        }
        from_struct_tokens.extend(quote! {
            #name: rows.iter().map(|r| r.#name.clone()).collect::<Vec<#ty>>()
        });
    }

    let from_vec_impl = quote! {
        impl From<Vec<#row_name>> for #df_name {
            fn from(rows: Vec<#row_name>) -> Self {
                #df_name {
                    #from_struct_tokens
                }
            }
        }
    };

    Ok(quote! {
        #dataframe_struct
        #from_vec_impl
    })
}
```

## Key Observation

When this same code structure is written manually (not via macro), it compiles successfully:

```rust
// THIS WORKS ✅
pub struct SimplePersonDataFrame {
    pub name: Vec<String>,
    pub age: Vec<i32>,
}

impl From<Vec<SimplePerson>> for SimplePersonDataFrame {
    fn from(rows: Vec<SimplePerson>) -> Self {
        SimplePersonDataFrame {
            name: rows.iter().map(|r| r.name.clone()).collect::<Vec<String>>(),
            age: rows.iter().map(|r| r.age.clone()).collect::<Vec<i32>>(),
        }
    }
}
```

## Debugging Questions

1. **Is the issue with TokenStream assembly?**
   - Building tokens via loop and `.extend()` vs using repetition `#()*`
   - Both approaches have been tried, both fail

2. **Is the issue with type parameter handling?**
   - The `#ty` interpolation should carry full type information
   - Works for struct fields, fails for impl body?

3. **Is the issue with quote! expansion order?**
   - Does quote! somehow share type inference context across fields?
   - Would generating separate quote! blocks for each field help?

4. **Could this be a rust-analyzer vs rustc difference?**
   - Does the error appear only in certain compilation modes?

## Test Variations to Try

### V1: Separate quote! blocks per field
Instead of building all fields in one pass, generate each field separately:

```rust
let name_field = quote! { name: rows.iter().map(|r| r.name.clone()).collect::<Vec<String>>() };
let age_field = quote! { age: rows.iter().map(|r| r.age.clone()).collect::<Vec<i32>>() };

let from_vec_impl = quote! {
    impl From<Vec<SimplePerson>> for SimplePersonDataFrame {
        fn from(rows: Vec<SimplePerson>) -> Self {
            SimplePersonDataFrame {
                #name_field,
                #age_field
            }
        }
    }
};
```

### V2: Use turbofish on iterator methods
```rust
rows.iter::<SimplePerson>().map(|r: &SimplePerson| r.name.clone())
```

### V3: Explicit type ascription
```rust
let name_vec: Vec<String> = rows.iter().map(|r| r.name.clone()).collect();
let age_vec: Vec<i32> = rows.iter().map(|r| r.age.clone()).collect();
```

### V4: Build struct via constructor function
```rust
fn from(rows: Vec<SimplePerson>) -> Self {
    let name = rows.iter().map(|r| r.name.clone()).collect::<Vec<String>>();
    let age = rows.iter().map(|r| r.age.clone()).collect::<Vec<i32>>();
    SimplePersonDataFrame { name, age }
}
```

## Cargo Expand Output Request

The `cargo expand` tool should show the exact expanded macro output. Running:
```bash
cargo expand --lib dataframe_examples::SimplePerson
```

Would help verify if the generated code truly matches expectations or if there's a subtle difference in the token structure.

## Platform Details

- Rust: 1.93.0 (254b59607 2026-01-19)
- OS: macOS (darwin 25.2.0)
- Proc-macro2: 1.x
- Quote: 1.x
- Syn: 2.x

## Contact

For questions or if you can provide insights, please comment on the related issue or contact the maintainers.
