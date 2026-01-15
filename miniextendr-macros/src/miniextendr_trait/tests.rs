use super::*;

// Helper to parse a trait and validate it
fn validate_trait_str(input: proc_macro2::TokenStream) -> syn::Result<()> {
    let trait_item: ItemTrait = syn::parse2(input)?;
    validate_trait(&trait_item)
}

// ==========================================================================
// Trait-level validation tests
// ==========================================================================

#[test]
fn trait_accepts_basic_trait() {
    let result = validate_trait_str(quote::quote! {
        pub trait Counter {
            fn value(&self) -> i32;
            fn increment(&mut self);
        }
    });
    assert!(result.is_ok());
}

#[test]
fn trait_accepts_private_trait() {
    let result = validate_trait_str(quote::quote! {
        trait Internal {
            fn get(&self) -> i32;
        }
    });
    assert!(result.is_ok());
}

#[test]
fn trait_rejects_generic_parameters() {
    let result = validate_trait_str(quote::quote! {
        pub trait Container<T> {
            fn get(&self) -> T;
        }
    });
    let err = result.unwrap_err();
    assert!(err.to_string().contains("cannot have generic parameters"));
}

#[test]
fn trait_accepts_lifetime_bounds() {
    // Lifetime bounds on trait are currently caught by "generic parameters"
    // This documents the current behavior
    let result = validate_trait_str(quote::quote! {
        pub trait Borrower<'a> {
            fn borrow(&self) -> &'a str;
        }
    });
    // Currently rejected - lifetimes count as generic params
    assert!(result.is_err());
}

// ==========================================================================
// Method-level validation tests
// ==========================================================================

#[test]
fn method_accepts_immutable_self() {
    let result = validate_trait_str(quote::quote! {
        pub trait Getter {
            fn get(&self) -> i32;
        }
    });
    assert!(result.is_ok());
}

#[test]
fn method_accepts_mutable_self() {
    let result = validate_trait_str(quote::quote! {
        pub trait Setter {
            fn set(&mut self, value: i32);
        }
    });
    assert!(result.is_ok());
}

#[test]
fn method_accepts_static_methods() {
    let result = validate_trait_str(quote::quote! {
        pub trait Factory {
            fn create() -> Self;
            fn default_value() -> i32;
        }
    });
    assert!(result.is_ok());
}

#[test]
fn method_accepts_associated_constants() {
    let result = validate_trait_str(quote::quote! {
        pub trait Constants {
            const MAX_VALUE: i32;
            fn get(&self) -> i32;
        }
    });
    assert!(result.is_ok());
}

#[test]
fn method_accepts_default_impl() {
    let result = validate_trait_str(quote::quote! {
        pub trait Defaulted {
            fn value(&self) -> i32;
            fn is_zero(&self) -> bool {
                self.value() == 0
            }
        }
    });
    assert!(result.is_ok());
}

#[test]
fn method_rejects_self_by_value() {
    let result = validate_trait_str(quote::quote! {
        pub trait Consumer {
            fn consume(self) -> i32;
        }
    });
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("receiver must be `&self` or `&mut self`")
    );
}

#[test]
fn method_rejects_async() {
    let result = validate_trait_str(quote::quote! {
        pub trait AsyncTrait {
            async fn fetch(&self) -> i32;
        }
    });
    let err = result.unwrap_err();
    assert!(err.to_string().contains("cannot be async"));
}

#[test]
fn method_rejects_generic_parameters() {
    let result = validate_trait_str(quote::quote! {
        pub trait Generic {
            fn convert<T>(&self, value: T) -> T;
        }
    });
    let err = result.unwrap_err();
    assert!(err.to_string().contains("cannot have generic parameters"));
}

#[test]
fn method_accepts_explicit_self_ref() {
    // `self: &Self` is semantically equivalent to `&self` and should be accepted
    let result = validate_trait_str(quote::quote! {
        pub trait ExplicitSelf {
            fn method(self: &Self) -> i32;
        }
    });
    assert!(result.is_ok(), "self: &Self should be accepted");
}

#[test]
fn method_accepts_explicit_self_mut_ref() {
    // `self: &mut Self` is semantically equivalent to `&mut self` and should be accepted
    let result = validate_trait_str(quote::quote! {
        pub trait ExplicitSelfMut {
            fn method(self: &mut Self) -> i32;
        }
    });
    assert!(result.is_ok(), "self: &mut Self should be accepted");
}

#[test]
fn method_rejects_box_self() {
    // `self: Box<Self>` is by-value (consumes self), should be rejected
    let result = validate_trait_str(quote::quote! {
        pub trait BoxSelf {
            fn method(self: Box<Self>) -> i32;
        }
    });
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("receiver must be `&self` or `&mut self`"),
        "unexpected error: {}",
        err
    );
}

// ==========================================================================
// Code generation tests
// ==========================================================================

#[test]
fn generate_produces_tag_constant() {
    let trait_item: ItemTrait = syn::parse2(quote::quote! {
        pub trait Counter {
            fn value(&self) -> i32;
        }
    })
    .unwrap();

    let output = generate_trait_abi(&trait_item);
    let output_str = output.to_string();

    assert!(output_str.contains("TAG_COUNTER"));
    assert!(output_str.contains("mx_tag"));
}

#[test]
fn generate_produces_vtable_struct() {
    let trait_item: ItemTrait = syn::parse2(quote::quote! {
        pub trait Counter {
            fn value(&self) -> i32;
            fn increment(&mut self);
        }
    })
    .unwrap();

    let output = generate_trait_abi(&trait_item);
    let output_str = output.to_string();

    assert!(output_str.contains("CounterVTable"));
    assert!(output_str.contains("mx_meth"));
    // Should have fields for both methods
    assert!(output_str.contains("value"));
    assert!(output_str.contains("increment"));
}

#[test]
fn generate_produces_view_struct() {
    let trait_item: ItemTrait = syn::parse2(quote::quote! {
        pub trait Counter {
            fn value(&self) -> i32;
        }
    })
    .unwrap();

    let output = generate_trait_abi(&trait_item);
    let output_str = output.to_string();

    assert!(output_str.contains("CounterView"));
    assert!(output_str.contains("data"));
    assert!(output_str.contains("vtable"));
}

#[test]
fn generate_produces_vtable_builder() {
    let trait_item: ItemTrait = syn::parse2(quote::quote! {
        pub trait Counter {
            fn value(&self) -> i32;
        }
    })
    .unwrap();

    let output = generate_trait_abi(&trait_item);
    let output_str = output.to_string();

    assert!(output_str.contains("__counter_build_vtable"));
}

#[test]
fn generate_excludes_static_methods_from_vtable() {
    let trait_item: ItemTrait = syn::parse2(quote::quote! {
        pub trait Factory {
            fn value(&self) -> i32;       // Instance - in vtable
            fn create() -> Self;           // Static - NOT in vtable
        }
    })
    .unwrap();

    let output = generate_trait_abi(&trait_item);
    let output_str = output.to_string();

    // vtable should contain `value` but not `create`
    // The shim function names can help us verify
    assert!(output_str.contains("__factory_value_shim"));
    assert!(!output_str.contains("__factory_create_shim"));
}

#[test]
fn generate_preserves_original_trait() {
    let trait_item: ItemTrait = syn::parse2(quote::quote! {
        /// Documentation comment
        pub trait Counter {
            fn value(&self) -> i32;
        }
    })
    .unwrap();

    let output = generate_trait_abi(&trait_item);
    let output_str = output.to_string();

    // Original trait should be in output (pass-through)
    assert!(output_str.contains("pub trait Counter"));
    assert!(output_str.contains("fn value"));
}

// ==========================================================================
// MethodInfo extraction tests
// ==========================================================================

#[test]
fn extract_method_info_captures_name() {
    let method: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn my_method(&self) -> i32;
    })
    .unwrap();

    let info = extract_method_info(&method);
    assert_eq!(info.name.to_string(), "my_method");
}

#[test]
fn extract_method_info_detects_self() {
    let with_self: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn method(&self) -> i32;
    })
    .unwrap();
    let info = extract_method_info(&with_self);
    assert!(info.has_self);

    let without_self: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn static_method() -> i32;
    })
    .unwrap();
    let info = extract_method_info(&without_self);
    assert!(!info.has_self);
}

#[test]
fn extract_method_info_detects_mutability() {
    let immutable: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn get(&self) -> i32;
    })
    .unwrap();
    let info = extract_method_info(&immutable);
    assert!(!info.is_mut);

    let mutable: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn set(&mut self, value: i32);
    })
    .unwrap();
    let info = extract_method_info(&mutable);
    assert!(info.is_mut);
}

#[test]
fn extract_method_info_extracts_params() {
    let method: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn add(&mut self, a: i32, b: String);
    })
    .unwrap();

    let info = extract_method_info(&method);
    assert_eq!(info.param_names.len(), 2);
    assert_eq!(info.param_names[0].to_string(), "a");
    assert_eq!(info.param_names[1].to_string(), "b");
    assert_eq!(info.param_types.len(), 2);
}

#[test]
fn extract_method_info_detects_default() {
    let without_default: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn required(&self) -> i32;
    })
    .unwrap();
    let info = extract_method_info(&without_default);
    assert!(!info.has_default);

    let with_default: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn defaulted(&self) -> i32 { 42 }
    })
    .unwrap();
    let info = extract_method_info(&with_default);
    assert!(info.has_default);
}
