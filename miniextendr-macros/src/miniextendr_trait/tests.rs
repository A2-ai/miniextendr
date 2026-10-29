use super::*;

// Helper to parse a trait and validate it
fn validate_trait_str(input: proc_macro2::TokenStream) -> syn::Result<()> {
    let trait_item: ItemTrait = syn::parse2(input)?;
    validate_trait(&trait_item)
}

// region: Trait-level validation tests

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
fn trait_accepts_generic_parameters() {
    // Generic type parameters on traits are allowed (e.g., RExtend<T>)
    let result = validate_trait_str(quote::quote! {
        pub trait Container<T> {
            fn get(&self) -> T;
        }
    });
    assert!(result.is_ok());
}

#[test]
fn trait_accepts_lifetime_bounds() {
    // Lifetime bounds on traits are allowed
    let result = validate_trait_str(quote::quote! {
        pub trait Borrower<'a> {
            fn borrow(&self) -> &'a str;
        }
    });
    assert!(result.is_ok());
}
// endregion

// region: Method-level validation tests

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
fn method_rejects_type_generic_parameters() {
    let result = validate_trait_str(quote::quote! {
        pub trait Generic {
            fn convert<T>(&self, value: T) -> T;
        }
    });
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("cannot have generic type or const parameters")
    );
}

#[test]
fn method_accepts_lifetime_parameters() {
    // Lifetime params are erased at codegen — they are FFI-safe on #[no_mangle] fns.
    let result = validate_trait_str(quote::quote! {
        pub trait Borrower {
            fn borrow<'a>(&self, s: &'a str) -> &'a str;
        }
    });
    assert!(
        result.is_ok(),
        "lifetime params should be accepted: {:?}",
        result.err()
    );
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
// endregion

// region: Code generation tests

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

    assert!(output_str.contains("TAG_Counter"));
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

    assert!(output_str.contains("__Counter_build_vtable"));
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
    assert!(output_str.contains("__Factory_value_shim"));
    assert!(!output_str.contains("__Factory_create_shim"));
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

/// Pointer invariants belong to the runtime, outside the caller's module.
#[test]
fn view_keeps_typed_runtime_storage_and_forwards_its_owner() {
    let item: ItemTrait = syn::parse_quote! {
        pub trait Counter { fn value(&self) -> i32; }
    };
    let output = generate_trait_abi(&item).to_string();
    assert!(
        output.contains("TraitViewData < CounterVTable >"),
        "{output}"
    );
    assert!(
        output.contains("TraitViewData :: new (data , vtable , owner)"),
        "{output}"
    );
    assert!(!output.contains("pub data :"), "{output}");
    assert!(!output.contains("pub vtable :"), "{output}");
}

/// The View protects each converted argument before the next conversion
/// allocates, and the returned SEXP until it is converted back to Rust.
#[test]
fn view_method_roots_arguments_and_result() {
    let method: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn join(&self, a: String, b: Vec<f64>) -> String;
    })
    .unwrap();
    let info = extract_method_info(&method).unwrap();
    let view = generate_view_method(&info).unwrap().to_string();
    assert!(
        view.contains(":: miniextendr_api :: gc_protect :: ProtectScope :: new ()"),
        "{view}"
    );
    for arg in ["a", "b"] {
        assert!(
            view.contains(&format!(
                "__mx_roots . protect_raw (:: miniextendr_api :: trait_abi :: to_sexp ({arg}))"
            )),
            "{view}"
        );
    }
    assert!(
        view.contains("let result = __mx_roots . protect_raw ({"),
        "{view}"
    );
}

/// A method with no arguments and no return value has nothing to root.
#[test]
fn view_method_without_arguments_or_result_opens_no_scope() {
    let method: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn touch(&mut self);
    })
    .unwrap();
    let info = extract_method_info(&method).unwrap();
    let view = generate_view_method(&info).unwrap().to_string();
    assert!(!view.contains("ProtectScope"), "{view}");
}
// endregion

// region: MethodInfo extraction tests

/// `formal` on a trait method declaration is refused (#1842): the
/// declaration's parameter names are the R formals of every implementation.
#[test]
fn extract_method_info_refuses_formal() {
    for attr in [
        quote::quote!(#[miniextendr(formal(k = "k.factor"))]),
        quote::quote!(#[miniextendr(formal = "k.factor")]),
    ] {
        let method: syn::TraitItemFn = syn::parse2(quote::quote! {
            #attr
            fn scaled(&self, k: f64) -> f64;
        })
        .unwrap();
        let err = extract_method_info(&method).expect_err("refused");
        assert_eq!(
            err.to_string(),
            crate::miniextendr_fn::TRAIT_FORMAL_UNSUPPORTED
        );
    }
}

#[test]
fn extract_method_info_captures_name() {
    let method: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn my_method(&self) -> i32;
    })
    .unwrap();

    let info = extract_method_info(&method).unwrap();
    assert_eq!(info.name.to_string(), "my_method");
}

#[test]
fn extract_method_info_detects_self() {
    let with_self: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn method(&self) -> i32;
    })
    .unwrap();
    let info = extract_method_info(&with_self).unwrap();
    assert!(info.has_self);

    let without_self: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn static_method() -> i32;
    })
    .unwrap();
    let info = extract_method_info(&without_self).unwrap();
    assert!(!info.has_self);
}

#[test]
fn extract_method_info_detects_mutability() {
    let immutable: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn get(&self) -> i32;
    })
    .unwrap();
    let info = extract_method_info(&immutable).unwrap();
    assert!(!info.is_mut);

    let mutable: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn set(&mut self, value: i32);
    })
    .unwrap();
    let info = extract_method_info(&mutable).unwrap();
    assert!(info.is_mut);
}

#[test]
fn extract_method_info_extracts_params() {
    let method: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn add(&mut self, a: i32, b: String);
    })
    .unwrap();

    let info = extract_method_info(&method).unwrap();
    assert_eq!(info.param_names.len(), 2);
    assert_eq!(info.param_names[0].to_string(), "a");
    assert_eq!(info.param_names[1].to_string(), "b");
    assert_eq!(info.param_types.len(), 2);
}

/// A `Missing<T>` argument crosses the View's vtable call through
/// `missing_to_sexp` (absent → R's missing-argument sentinel), every other
/// argument through `to_sexp`; `Missing` has no `IntoR` (#1551).
#[test]
fn view_method_passes_missing_args_as_the_sentinel() {
    let method: syn::TraitItemFn = syn::parse2(quote::quote! {
        fn pick(&self, level: Missing<Option<String>>, n: i32) -> String;
    })
    .unwrap();
    let info = extract_method_info(&method).unwrap();
    let view = generate_view_method(&info).unwrap().to_string();
    assert!(
        view.contains(":: miniextendr_api :: trait_abi :: missing_to_sexp (level)"),
        "{view}"
    );
    assert!(
        view.contains(":: miniextendr_api :: trait_abi :: to_sexp (n)"),
        "{view}"
    );
}
// endregion

// region: TPIE `macro_rules!` contract

/// The TPIE helper a trait emits takes the impl's `no_preconditions` as its
/// last token and forwards it to the expansion input.
#[test]
fn tpie_macro_forwards_no_preconditions() {
    let trait_item: ItemTrait = syn::parse_quote! {
        pub trait Scale {
            fn scaled(&self, k: f64) -> f64;
        }
    };
    let out = generate_trait_abi(&trait_item).to_string();
    assert!(
        out.contains("$ noexport : tt , $ no_preconditions : tt)"),
        "{out}"
    );
    assert!(
        out.contains("no_preconditions = $ no_preconditions ;"),
        "{out}"
    );
}

// endregion

/// A `Checked<T>` / `Unchecked<T>` parameter in a trait method is refused:
/// the View passes R values on, and each impl's wrapper owns the checks.
#[test]
fn trait_method_rejects_precondition_markers() {
    for (method, expected) in [
        (
            syn::parse_quote!(
                fn scaled(&self, k: Checked<f64>) -> f64;
            ),
            "spell it on the impl, `#[miniextendr(preconditions(k))]`",
        ),
        (
            syn::parse_quote!(
                fn scaled(&self, k: miniextendr_api::Unchecked<f64>) -> f64;
            ),
            "spell it on the impl, `#[miniextendr(no_preconditions(k))]`",
        ),
    ] {
        let method: syn::TraitItemFn = method;
        let err = extract_method_info(&method)
            .expect_err("rejected")
            .to_string();
        assert!(err.contains("in a `#[miniextendr]` trait method"), "{err}");
        assert!(err.contains(expected), "{err}");
    }
}

/// Mixed vtable signatures must not share an interface tag, including when
/// only the position of the owner-aware slot changes.
#[test]
fn owner_interface_tags_distinguish_calling_conventions_and_method_order() {
    fn tag_expression(item: ItemTrait) -> String {
        let generated: syn::File = syn::parse2(generate_trait_abi(&item)).unwrap();
        generated
            .items
            .iter()
            .find_map(|item| {
                let syn::Item::Const(item) = item else {
                    return None;
                };
                (item.ident == "TAG_Layout")
                    .then(|| quote::ToTokens::to_token_stream(&item.expr).to_string())
            })
            .unwrap()
    }
    let ordinary = tag_expression(syn::parse_quote! {
        pub trait Layout {
            fn owned(&self) -> i32;
            fn plain(&self) -> i32;
        }
    });
    let owner_first = tag_expression(syn::parse_quote! {
        pub trait Layout: TypedExternal {
            fn owned(self: ExternalPtrRef<'_, Self>) -> i32;
            fn plain(&self) -> i32;
        }
    });
    let owner_second = tag_expression(syn::parse_quote! {
        pub trait Layout: TypedExternal {
            fn plain(&self) -> i32;
            fn owned(self: ExternalPtrRef<'_, Self>) -> i32;
        }
    });
    let owner_mut = tag_expression(syn::parse_quote! {
        pub trait Layout: TypedExternal {
            fn owned(self: ExternalPtrMut<'_, Self>) -> i32;
            fn plain(&self) -> i32;
        }
    });
    assert_ne!(owner_first, ordinary);
    assert_ne!(owner_first, owner_second);
    assert_ne!(owner_first, owner_mut);
}

/// The identity covers the whole exported interface, not only owner receivers.
#[test]
fn interface_tags_track_exported_signatures() {
    fn tag(source: &str) -> String {
        let item: ItemTrait = syn::parse_str(source).unwrap();
        let methods: Vec<_> = item
            .items
            .iter()
            .filter_map(|item| {
                let syn::TraitItem::Fn(method) = item else {
                    return None;
                };
                let info = extract_method_info(method).unwrap();
                (info.has_self && !info.skip).then_some(info)
            })
            .collect();
        trait_abi_tag_path(&item.ident, &methods)
    }

    let original = tag("trait Layout { fn read(&self, count: i32) -> i32; fn reset(&mut self); }");
    for changed in [
        "trait Layout { fn read(&self, count: i32) -> i32; }",
        "trait Layout { fn read(&self, count: i32) -> i32; fn size(&self) -> i32; fn reset(&mut self); }",
        "trait Layout { fn reset(&mut self); fn read(&self, count: i32) -> i32; }",
        "trait Layout { fn read(&mut self, count: i32) -> i32; fn reset(&mut self); }",
        "trait Layout { fn read(&self, count: f64) -> i32; fn reset(&mut self); }",
        "trait Layout { fn read(&self, count: i32) -> f64; fn reset(&mut self); }",
        "trait Layout { fn read(&self, count: i32, extra: i32) -> i32; fn reset(&mut self); }",
        "trait Layout { fn renamed(&self, count: i32) -> i32; fn reset(&mut self); }",
        "trait Layout { #[miniextendr(r_name = \"peek\")] fn read(&self, count: i32) -> i32; fn reset(&mut self); }",
        "trait Layout { #[miniextendr(skip)] fn read(&self, count: i32) -> i32; fn reset(&mut self); }",
        "trait Other { fn read(&self, count: i32) -> i32; fn reset(&mut self); }",
    ] {
        assert_ne!(original, tag(changed), "{changed}");
    }
    for unchanged in [
        "trait Layout { fn read(&self, renamed_arg: i32) -> i32; fn reset(&mut self) -> (); }",
        "trait Layout { #[doc = \"New docs\"] fn read(&self, count: i32) -> i32 { count } fn reset(&mut self) {} }",
        "trait Layout { fn read(&self, count: i32) -> i32; fn reset(&mut self); fn new() -> Self; }",
        "trait Layout { fn read(&self, count: i32) -> i32; #[miniextendr(skip)] fn hidden(&self); fn reset(&mut self); }",
        "trait Layout { #[miniextendr(r_name = \"read\")] fn read(&self, count: i32) -> i32; fn reset(&mut self); }",
    ] {
        assert_eq!(original, tag(unchanged), "{unchanged}");
    }
}

#[test]
fn every_trait_exports_its_metadata_under_the_traits_name() {
    for trait_item in [
        syn::parse_quote! { pub trait Collection<T> {
            #[miniextendr(r_name = "len")]
            fn r_len(&self) -> i32;
            #[miniextendr(skip)]
            fn hidden(&self, value: T);
        } },
        syn::parse_quote! { pub trait Collection {
            type Item;
            #[miniextendr(r_name = "len")]
            fn r_len(&self) -> i32;
            #[miniextendr(skip)]
            fn hidden(&self, value: Self::Item);
        } },
    ] {
        let output = generate_trait_abi(&trait_item);
        let file: syn::File = syn::parse2(output.clone()).unwrap();
        let exports: Vec<_> = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Use(item) => Some(quote::ToTokens::to_token_stream(item).to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(exports.len(), 1);
        assert!(exports[0].contains("as Collection"));
        let text = output.to_string();
        assert!(text.contains("@ impl"));
        assert!(text.contains("r_name = \"len\""));
        assert!(text.contains("miniextendr (skip)"));
    }
}

#[test]
fn metadata_macros_do_not_collide_for_same_named_traits_in_separate_modules() {
    let trait_item = syn::parse_quote! { pub trait Repeated { fn value(&self) -> i32; } };
    let export = || {
        let file: syn::File = syn::parse2(generate_trait_abi(&trait_item)).unwrap();
        file.items
            .into_iter()
            .find_map(|item| match item {
                syn::Item::Macro(item) if item.mac.path.is_ident("macro_rules") => item.ident,
                _ => None,
            })
            .unwrap()
    };
    assert_ne!(export(), export());
}

/// The declaration's metadata carries each method's docs, for the wrappers of
/// a populated impl and of an empty one.
#[test]
fn trait_metadata_carries_method_docs() {
    let declaration: ItemTrait = syn::parse_quote! {
        pub trait Counter {
            /// Add to the counter.
            fn add(&mut self, n: i32);
            /// The value.
            fn value(&self) -> i32 { 0 }
        }
    };
    let text = generate_trait_abi(&declaration).to_string();
    // The populated-impl arm's required and default entries.
    for entry in [
        "# [doc = r\" Add to the counter.\"] fn add (n : ()) ;",
        "# [doc = r\" The value.\"] fn value (& self) -> i32 { }",
    ] {
        assert!(text.contains(entry), "{entry}: {text}");
    }
    // The empty-impl arm's `method { ... }` entries.
    for entry in [
        "method { r_name = add ; # [doc = r\" Add to the counter.\"] fn add",
        "method { r_name = value ; # [doc = r\" The value.\"] fn value",
    ] {
        assert!(text.contains(entry), "{entry}: {text}");
    }
}

/// A declaration's shim expands in the declaring crate but runs in the
/// implementing package: its conversion failure reads that package's classes
/// when it fails, never ones written where the trait was declared.
#[test]
fn declaration_shim_conversion_errors_take_the_implementers_class() {
    let declaration: ItemTrait = syn::parse_quote! {
        pub trait Counter {
            fn add(&mut self, n: i32);
        }
    };
    let text = generate_trait_abi(&declaration).to_string();
    let read = ":: miniextendr_api :: trait_abi :: conversion_error_class ()";
    assert_eq!(text.matches(read).count(), 1, "{text}");
    assert!(
        !text.contains("& [] , :: miniextendr_api :: __mx_conversion_err_parts"),
        "{text}"
    );
}

#[test]
fn declaration_shims_live_on_the_trait_and_are_excluded_from_trait_objects() {
    let declaration: ItemTrait = syn::parse_quote! {
        pub trait Local {
            type Item;
            fn read(&self) -> Self::Item;
            fn default_value(&self, value: LocalType) -> i32 { value.0 }
        }
    };
    let file: syn::File = syn::parse2(generate_trait_abi(&declaration)).unwrap();
    let declared = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Trait(item) => Some(item),
            _ => None,
        })
        .unwrap();
    let callbacks: Vec<_> = declared
        .items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Fn(method) if method.sig.ident.to_string().ends_with("_shim") => {
                Some(method)
            }
            _ => None,
        })
        .collect();
    assert_eq!(callbacks.len(), 2);
    for callback in callbacks {
        assert!(callback.sig.generics.params.is_empty());
        assert!(callback.default.is_some());
        let text = quote::ToTokens::to_token_stream(callback).to_string();
        assert!(text.contains("Self : :: core :: marker :: Sized"), "{text}");
        assert!(!text.contains("__ImplT"), "{text}");
        assert!(text.contains("with_r_unwind_protect_shim"));
    }
    assert!(!file.items.iter().any(|item| {
        matches!(item, syn::Item::Fn(function) if function.sig.ident.to_string().ends_with("_shim"))
    }));
}
