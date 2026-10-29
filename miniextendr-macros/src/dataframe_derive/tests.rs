use super::*;

/// Stringify the derive output (whitespace-normalised) for substring assertions.
fn expand(input: DeriveInput) -> String {
    derive_dataframe_row(input).unwrap().to_string()
}

#[test]
fn struct_as_factor_is_one_readable_column() {
    let input: DeriveInput = syn::parse_quote! {
        struct Row {
            #[dataframe(as_factor, rename = "direction")]
            dir: Direction,
        }
    };
    let syn::Data::Struct(data) = &input.data else {
        unreachable!()
    };
    let resolved = resolve_struct_field(data.fields.iter().next().unwrap(), 0, false)
        .unwrap()
        .unwrap();
    assert!(matches!(&resolved, ResolvedField::Single(_)));
    assert!(field_reader_capable(&resolved));
    let code = expand(input);
    assert!(code.contains("FactorVec"));
    assert!(code.contains("Self :: try_from_dataframe (sexp)"));
}

#[test]
fn struct_as_factor_rejects_non_enum_shapes_at_the_field() {
    for ty in [quote! { i32 }, quote! { Vec<i32> }, quote! { [i32; 2] }] {
        let input = syn::parse_quote! {
            struct Row { #[dataframe(as_factor)] value: #ty }
        };
        let error = derive_dataframe_row(input).unwrap_err().to_string();
        assert!(error.contains("`as_factor` is only valid"), "{error}");
    }
}

#[test]
fn enum_columns_compare_standard_spellings_without_string_coercion() {
    let string_ty = syn::parse_quote!(String);
    // A `macro_rules!` `$t:ty` reaches the derive in an invisible group.
    let from_macro = proc_macro2::Group::new(proc_macro2::Delimiter::None, quote!(i32));
    for (first, second) in [
        (quote!(#from_macro), quote!(i32)),
        (quote!((i32)), quote!(::core::primitive::i32)),
        (quote!(&'a str), quote!(&'a ::core::primitive::str)),
        (quote!(i32), quote!(::core::primitive::i32)),
        (quote!(std::primitive::i32), quote!(i32)),
        (quote!(String), quote!(std::string::String)),
        (
            quote!(::alloc::string::String),
            quote!(::std::string::String),
        ),
        (
            quote!(Vec<i32>),
            quote!(::std::vec::Vec<::core::primitive::i32>),
        ),
        (
            quote!(Box<[i32]>),
            quote!(::alloc::boxed::Box<[std::primitive::i32]>),
        ),
        (quote!(&[i32]), quote!(&[::core::primitive::i32])),
        (
            quote!(Option<String>),
            quote!(::core::option::Option<::alloc::string::String>),
        ),
        // A datetime crate path matches itself behind `::` or the
        // `miniextendr_api` re-export.
        (quote!(jiff::Timestamp), quote!(::jiff::Timestamp)),
        (
            quote!(miniextendr_api::jiff::civil::Date),
            quote!(jiff::civil::Date),
        ),
        (quote!(::miniextendr_api::time::Date), quote!(time::Date)),
    ] {
        let first: syn::Type = syn::parse2(first).unwrap();
        let second: syn::Type = syn::parse2(second).unwrap();
        for coerce in [false, true] {
            let mut registry = ColumnRegistry::new(coerce, &string_ty);
            registry
                .register("value", &first, 0, &syn::parse_quote!(A), Span::call_site())
                .unwrap();
            registry
                .register(
                    "value",
                    &second,
                    1,
                    &syn::parse_quote!(B),
                    Span::call_site(),
                )
                .unwrap();
            assert_eq!(registry.columns[0].ty, first);
            assert!(!registry.columns[0].string_coerced);
            assert_eq!(registry.columns[0].present_in, [0, 1]);
        }
    }
}

#[test]
fn enum_columns_still_detect_real_conflicts_and_distinct_user_paths() {
    let string_ty = syn::parse_quote!(String);
    for (first, second) in [
        (quote!(f64), quote!(String)),
        (quote!(Vec<i32>), quote!(Vec<f64>)),
        (quote!(Box<[i32]>), quote!(Vec<i32>)),
        (quote!(custom::String), quote!(std::string::String)),
        (quote!(left::Value), quote!(right::Value)),
        // Not a prelude name: a bare `HashMap` is whatever the user imported.
        (
            quote!(HashMap<String, i32>),
            quote!(std::collections::HashMap<String, i32>),
        ),
        // Aliases and module-relative paths are compared as written.
        (quote!(primitive::i32), quote!(i32)),
        (quote!(Id), quote!(i32)),
        (
            quote!(std::cmp::Ordering),
            quote!(std::sync::atomic::Ordering),
        ),
        // Two datetime types, and a bare name the derive can't resolve.
        (quote!(jiff::civil::Date), quote!(time::Date)),
        (quote!(Timestamp), quote!(jiff::Timestamp)),
    ] {
        let first = syn::parse2(first).unwrap();
        let second = syn::parse2(second).unwrap();
        let mut registry = ColumnRegistry::new(false, &string_ty);
        registry
            .register("value", &first, 0, &syn::parse_quote!(A), Span::call_site())
            .unwrap();
        assert!(
            registry
                .register(
                    "value",
                    &second,
                    1,
                    &syn::parse_quote!(B),
                    Span::call_site()
                )
                .is_err()
        );
        registry.coerce_to_string = true;
        registry
            .register(
                "value",
                &second,
                1,
                &syn::parse_quote!(B),
                Span::call_site(),
            )
            .unwrap();
        assert!(registry.columns[0].string_coerced);
        assert_eq!(registry.columns[0].ty, string_ty);
    }
}

/// The conflict error names both variants and both spellings, and offers
/// one spelling before `conflicts = "string"` (#2084).
#[test]
fn enum_type_conflicts_name_both_spellings() {
    let input: DeriveInput = syn::parse_quote! {
        enum Row {
            A { id: Id },
            B { id: i32 },
        }
    };
    let error = derive_dataframe_row(input).unwrap_err().to_string();
    assert!(
        error.contains("variant `B` has type `i32`, variant `A` has `Id`"),
        "{error}"
    );
    assert!(error.contains(SPELL_TYPES_ALIKE), "{error}");
    assert!(error.contains("conflicts = \"string\""), "{error}");

    let input: DeriveInput = syn::parse_quote! {
        enum Row {
            A { #[dataframe(expand)] values: Vec<i32> },
            B { #[dataframe(expand)] values: Vec<f64> },
        }
    };
    let error = derive_dataframe_row(input).unwrap_err().to_string();
    assert!(
        error.contains("variant `B` has element type `f64`, variant `A` has `i32`"),
        "{error}"
    );
    assert!(error.contains(SPELL_TYPES_ALIKE), "{error}");
}

/// `Option<scalar>` struct fields are a single NA-able column (#1437): the
/// derive accepts them, keeps the full `Option<T>` as the companion element
/// type, and still emits the readers (the reader gate already knew the shape).
#[test]
fn option_scalar_struct_field_is_single_nullable_column() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Obs {
            id: i32,
            weight: Option<f64>,
            label: Option<String>,
            flag: Option<bool>,
        }
    });
    assert!(code.contains("weight"), "column kept, got:\n{code}");
    assert!(
        code.contains("Option < f64 >"),
        "companion stores Option<f64>, got:\n{code}"
    );
    assert!(code.contains("fn try_from_dataframe"));
    assert!(code.contains("fn try_from_dataframe_par"));
}

#[test]
fn nullable_raw_columns_explain_the_missing_na_representation() {
    for ty in [
        quote! { Option<u8> },
        quote! { ::core::option::Option<::core::primitive::u8> },
        quote! { std::option::Option<std::primitive::u8> },
    ] {
        let input = syn::parse_quote! { struct Row { bytes: #ty } };
        let error = derive_dataframe_row(input).unwrap_err().to_string();
        assert!(error.contains("raw vectors have no NA value"), "{error}");
        assert!(error.contains("Option<u16>"), "{error}");
        assert!(error.contains("Option<i32>"), "{error}");
    }
    for ty in [
        quote! { u8 },
        quote! { Option<u16> },
        quote! { Option<i32> },
    ] {
        let input = syn::parse_quote! { struct Row { bytes: #ty } };
        assert!(derive_dataframe_row(input).is_ok());
    }
}

/// Every shape that pads a `u8` column with NA is rejected at the field;
/// shapes that never need NA cells still derive (#2025).
#[test]
fn padded_raw_columns_name_the_field() {
    for field in [
        quote! { #[dataframe(width = 2)] bytes: Vec<u8> },
        quote! { #[dataframe(expand)] bytes: Vec<u8> },
        quote! { #[dataframe(expand)] bytes: Box<[::core::primitive::u8]> },
    ] {
        let input = syn::parse_quote! { struct Row { #field } };
        let error = derive_dataframe_row(input).unwrap_err().to_string();
        assert!(error.contains("pads shorter rows with NA"), "{error}");
        assert!(error.contains("raw vectors have no NA value"), "{error}");
    }
    for variant in [
        quote! { Bytes { byte: u8 } },
        quote! { Bytes(::core::primitive::u8) },
        quote! { Bytes { byte: Option<u8> } },
        quote! { Bytes([u8; 2]) },
        quote! { Bytes { #[dataframe(width = 2)] bytes: Vec<u8> } },
        quote! { Bytes(#[dataframe(expand)] Vec<u8>) },
    ] {
        let input = syn::parse_quote! {
            #[dataframe(tag = "kind")]
            enum Row { #variant, Empty }
        };
        let error = derive_dataframe_row(input).unwrap_err().to_string();
        assert!(
            error.contains("NA in rows of the other variants"),
            "{error}"
        );
        assert!(!error.contains("Use `u8`"), "{error}");
    }
    for input in [
        syn::parse_quote! { struct Row { byte: u8, pair: [u8; 2], bytes: Vec<u8> } },
        syn::parse_quote! {
            #[dataframe(tag = "kind")]
            enum Row { Bytes { bytes: Vec<u8> }, Empty }
        },
    ] {
        assert!(derive_dataframe_row(input).is_ok());
    }
}

/// The #484 footgun stays closed: `Option` around a map / struct / collection
/// is still rejected, now with a hint naming the supported scalar shape.
#[test]
fn option_non_scalar_struct_field_still_rejected() {
    for field_ty in [
        quote::quote!(Option<HashMap<String, i32>>),
        quote::quote!(Option<Inner>),
        quote::quote!(Option<Vec<f64>>),
    ] {
        let input: DeriveInput = syn::parse_quote! {
            #[derive(DataFrameRow)]
            struct Row {
                id: i32,
                payload: #field_ty,
            }
        };
        let err = derive_dataframe_row(input)
            .expect_err("Option<non-scalar> must be rejected")
            .to_string();
        assert!(
            err.contains("does not support `Option<…>`")
                && err.contains("supported only around a scalar"),
            "message must name the supported shape, got: {err}"
        );
    }
}

/// A datetime type spelled by its crate path is one column, `Option<T>` an
/// NA-able one, both read back; each field asserts the feature marker.
#[test]
fn datetime_crate_paths_are_readable_columns() {
    let input: DeriveInput = syn::parse_quote! {
        struct Obs {
            at: jiff::Timestamp,
            seen: Option<::jiff::Timestamp>,
            zoned: miniextendr_api::jiff::Zoned,
            lag: jiff::SignedDuration,
            day: Option<jiff::civil::Date>,
            stamp: time::OffsetDateTime,
            due: Option<::miniextendr_api::time::Date>,
        }
    };
    let syn::Data::Struct(data) = &input.data else {
        unreachable!()
    };
    for (i, field) in data.fields.iter().enumerate() {
        let resolved = resolve_struct_field(field, i, false).unwrap().unwrap();
        assert!(matches!(&resolved, ResolvedField::Single(_)), "field {i}");
        assert!(field_reader_capable(&resolved), "field {i}");
    }
    let code = expand(input);
    assert_eq!(code.matches("DataFrameDateTime").count(), 7, "{code}");
    assert!(code.contains("_assert_datetime_column :: < jiff :: Timestamp >"));
    assert!(code.contains("_assert_datetime_column :: < :: jiff :: Timestamp >"));
    assert!(code.contains("fn try_from_dataframe"));
    assert!(code.contains("fn try_from_dataframe_par"));
}

/// Only crate paths are datetime columns: a bare or module-relative name may
/// be the user's own struct and still flattens, and types without column
/// conversions are not recognized. `Option` around a bare name stays an
/// error whose hint names the crate-path spelling.
#[test]
fn datetime_bare_names_keep_their_meaning() {
    for ty in [
        quote!(Timestamp),
        quote!(Date),
        quote!(civil::Date),
        quote!(crate::time::Date),
        quote!(self::jiff::Timestamp),
        quote!(jiff::Span),
        quote!(jiff::civil::DateTime),
        quote!(time::Duration),
    ] {
        let ty: syn::Type = syn::parse2(ty).unwrap();
        assert!(
            matches!(classify_field_type(&ty), Ok(FieldTypeKind::Struct { .. })),
            "{}",
            quote!(#ty)
        );
    }
    let input: DeriveInput = syn::parse_quote! {
        struct Row { at: Option<Timestamp> }
    };
    let error = derive_dataframe_row(input).unwrap_err().to_string();
    assert!(error.contains("`Option<jiff::Timestamp>`"), "{error}");
}

/// An enum variant's datetime field is one `Vec<Option<T>>` column shared
/// with another variant's spelling of the same path, and reads back;
/// `Option<T>` there is rejected like any `Option<scalar>`.
#[test]
fn datetime_enum_variant_fields() {
    let code = expand(syn::parse_quote! {
        #[dataframe(tag = "kind")]
        enum Event {
            Start { at: jiff::Timestamp },
            Stop { at: ::jiff::Timestamp, day: time::Date },
        }
    });
    assert_eq!(code.matches("DataFrameDateTime").count(), 3, "{code}");
    assert!(code.contains("fn try_from_dataframe"), "{code}");

    let input: DeriveInput = syn::parse_quote! {
        #[dataframe(tag = "kind")]
        enum Event {
            Start { at: Option<jiff::Timestamp> },
            Stop { code: i32 },
        }
    };
    let error = derive_dataframe_row(input).unwrap_err().to_string();
    assert!(
        error.contains("already become `Option<T>` columns"),
        "{error}"
    );
}

/// Enum variant fields already become `Option<T>` columns (absent for the
/// other variants), so an `Option<scalar>` field there is rejected with a
/// pointer to the bare scalar instead of failing on `Vec<Option<Option<T>>>`.
/// A qualified payload gets the same message, not the generic wrapper one.
#[test]
fn option_scalar_enum_variant_field_rejected() {
    for note_ty in [
        quote::quote!(Option<String>),
        quote::quote!(::core::option::Option<::std::string::String>),
    ] {
        let input: DeriveInput = syn::parse_quote! {
            #[derive(DataFrameRow)]
            #[dataframe(tag = "kind")]
            enum Event {
                Click { x: f64, note: #note_ty },
                Key { code: i32 },
            }
        };
        let err = derive_dataframe_row(input)
            .expect_err("Option<scalar> in enum variant must be rejected")
            .to_string();
        assert!(
            err.contains("already become `Option<T>` columns"),
            "got: {err}"
        );
    }
}

/// A qualified scalar classifies like its bare name (#2036): a leading `::`
/// or a path prefix keeps the field a reader scalar and an `Option<…>`
/// payload, so struct and enum rows keep both readers.
#[test]
fn qualified_scalar_fields_keep_readers() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Obs {
            id: ::core::primitive::i32,
            label: ::std::string::String,
            note: ::core::option::Option<::alloc::string::String>,
            weight: Option<std::primitive::f64>,
            relative: Option<std::string::String>,
            #[dataframe(width = 2)]
            pinned: Vec<::std::string::String>,
            keyed: BTreeMap<::core::primitive::i32, ::std::string::String>,
        }
    });
    assert!(code.contains("fn try_from_dataframe ("), "got:\n{code}");
    assert!(code.contains("fn try_from_dataframe_par ("), "got:\n{code}");

    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        #[dataframe(tag = "kind")]
        enum Event {
            Named { label: ::std::string::String, pair: [::core::primitive::f64; 2] },
            Keyed { keyed: BTreeMap<::std::string::String, std::primitive::i32> },
            Empty,
        }
    });
    assert!(code.contains("fn try_from_dataframe ("), "got:\n{code}");
    assert!(code.contains("fn try_from_dataframe_par ("), "got:\n{code}");
}

/// Borrowed expansions write columns, but owned R columns cannot recreate
/// their borrows. Check each shape independently so one unsupported field
/// cannot mask another shape's accidentally generated reader (#2028).
#[test]
fn borrowed_expansions_have_no_struct_or_enum_reader() {
    for field in [
        quote! { values: &'a [f64; 2] },
        quote! { #[dataframe(width = 2)] values: &'a [f64] },
        quote! { #[dataframe(expand)] values: &'a [f64] },
    ] {
        for input in [
            syn::parse_quote! { struct Row<'a> { #field } },
            syn::parse_quote! {
                #[dataframe(tag = "kind")]
                enum Row<'a> { Values { #field }, Empty }
            },
        ] {
            let tokens = derive_dataframe_row(input).unwrap();
            syn::parse2::<syn::File>(tokens.clone()).unwrap();
            let code = tokens.to_string();
            assert!(!code.contains("fn try_from_dataframe"), "{code}");
            assert!(!code.contains("fn from_dataframe_rows"), "{code}");
        }
    }
}

/// Scalar named struct: the baseline `try_from_dataframe_par` shape (#765).
/// Both the sequential and parallel readers must be emitted, and the parallel
/// one must drive a `into_par_iter()` row-assembly region.
#[test]
fn scalar_struct_gets_parallel_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Measurement {
            time: f64,
            value: f64,
        }
    });
    assert!(code.contains("fn try_from_dataframe"));
    assert!(code.contains("fn try_from_dataframe_par"));
    assert!(code.contains("into_par_iter"));
}

/// `[T; N]` fixed-array expansion field (#782/#808): the reader regroups the
/// `pos_1`/`pos_2` columns back into the array inside the parallel loop, with
/// zero SEXP access in `into_par_iter` (the invariant #764 protects).
#[test]
fn fixed_array_struct_gets_parallel_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Point {
            #[dataframe(rename = "pos")]
            pos: [f64; 2],
        }
    });
    assert!(code.contains("fn try_from_dataframe_par"));
    assert!(code.contains("into_par_iter"));
    // Regrouped from the suffixed expansion columns.
    assert!(code.contains("pos_1"));
    assert!(code.contains("pos_2"));
}

/// `Vec<T>` + `width = N` expansion field (#782/#808): the reader flattens the
/// `scores_1`/`scores_2` Option columns per row back into the vec.
#[test]
fn pinned_vec_struct_gets_parallel_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Scored {
            #[dataframe(width = 2)]
            scores: Vec<i32>,
        }
    });
    assert!(code.contains("fn try_from_dataframe_par"));
    assert!(code.contains("into_par_iter"));
    assert!(code.contains("scores_1"));
    assert!(code.contains("scores_2"));
}

/// `Vec<T>` + `expand` auto-expansion field (#782/#808): the reader discovers
/// `tags_<i>` columns at runtime and flattens per row. Still a true parallel
/// reader (the column discovery happens on the R thread, up front).
#[test]
fn auto_expand_struct_gets_parallel_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Tagged {
            #[dataframe(expand)]
            tags: Vec<i32>,
        }
    });
    assert!(code.contains("fn try_from_dataframe_par"));
    assert!(code.contains("into_par_iter"));
}

/// Struct-flatten field (#485/#808): the struct still gets readers, but the
/// parallel variant deliberately delegates to the sequential one to avoid
/// imposing `Inner: Clone` for by-index parallel assembly (#764 design note).
#[test]
fn struct_flatten_par_delegates_to_sequential() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Outer {
            id: i32,
            inner: Inner,
        }
    });
    assert!(code.contains("fn try_from_dataframe"));
    assert!(code.contains("fn try_from_dataframe_par"));
    // The `_par` body delegates rather than running its own `into_par_iter`.
    assert!(code.contains("Self :: try_from_dataframe (sexp)"));
}

/// Tagged enum companion (#807/#816): enums now get full readers too, including
/// a parallel per-row tag-dispatch loop. Documents that the #764 "no reader at
/// all today" framing for enums is now stale.
#[test]
fn tagged_enum_gets_parallel_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        #[dataframe(tag = "_type")]
        enum Event {
            Click { x: i32, y: i32 },
            Key { code: i32 },
        }
    });
    assert!(code.contains("fn try_from_dataframe"));
    assert!(code.contains("fn try_from_dataframe_par"));
    assert!(code.contains("into_par_iter"));
}

/// Struct-path `HashMap<String, V>` map column (#764): the list-of-named-lists
/// column reads back whole via `Vec<map>: TryFromSexp` on the R thread, so the
/// struct gets both readers (it shares the scalar `pull_col` path — zero SEXP
/// access inside `into_par_iter`). Flips the pre-#764 lock-in test from #920.
#[test]
fn struct_with_string_keyed_map_field_gets_parallel_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Config {
            opts: ::std::collections::HashMap<String, i32>,
        }
    });
    assert!(code.contains("fn try_from_dataframe"));
    assert!(code.contains("fn try_from_dataframe_par"));
    assert!(code.contains("into_par_iter"));
}

/// `BTreeMap<String, Option<scalar>>` is also reader-capable: the
/// `Vec<map>: TryFromSexp` impl is generic over `V: TryFromSexp`, and
/// `Option<scalar>` qualifies (NULL list elements → `None`).
#[test]
fn struct_with_btreemap_option_value_gets_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Config {
            opts: ::std::collections::BTreeMap<String, Option<f64>>,
        }
    });
    assert!(code.contains("fn try_from_dataframe"));
    assert!(code.contains("fn try_from_dataframe_par"));
}

/// Non-`String` bare-scalar map keys (#919): the struct gets parallel `_keys`/`_values`
/// list-columns and both readers. The write side uses `Vec<Vec<K>>/Vec<Vec<V>>: IntoR`
/// (VECSXP of typed vectors); the read side zips them back into the map type per row.
#[test]
fn struct_with_non_string_keyed_map_gets_parallel_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Config {
            opts: ::std::collections::HashMap<i32, f64>,
        }
    });
    assert!(
        code.contains("fn try_from_dataframe"),
        "non-String bare-scalar map keys should produce a reader via _keys/_values columns"
    );
    assert!(
        code.contains("fn try_from_dataframe_par"),
        "non-String bare-scalar map keys should produce a parallel reader"
    );
    assert!(
        code.contains("opts_keys"),
        "non-String map field `opts` should expand to `opts_keys` column"
    );
    assert!(
        code.contains("opts_values"),
        "non-String map field `opts` should expand to `opts_values` column"
    );
}

/// Same as above but using an unqualified path `std::collections::BTreeMap` (no
/// leading `::`) — path form used in actual rpkg fixtures.
#[test]
fn struct_with_btreemap_int_key_unqualified_path() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Tally {
            id: i32,
            tally: std::collections::BTreeMap<i32, f64>,
        }
    });
    assert!(
        code.contains("tally_keys"),
        "unqualified std::collections::BTreeMap<i32, f64> must expand to tally_keys"
    );
    assert!(
        code.contains("fn try_from_dataframe"),
        "unqualified BTreeMap<i32, f64> should produce a reader"
    );
}

/// Float-keyed maps (`f32`/`f64`) are rejected with a clear error.
#[test]
#[should_panic]
fn struct_with_float_keyed_map_is_rejected() {
    let _code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Config {
            opts: ::std::collections::HashMap<f64, i32>,
        }
    });
}

/// Custom-hasher maps (`HashMap<K, V, S>`) are rejected from the reader path:
/// the `Vec<HashMap<String, V>>: TryFromSexp` impl only covers the default
/// hasher, so emitting a reader would not compile.
#[test]
fn struct_with_custom_hasher_map_has_no_reader() {
    let code = expand(syn::parse_quote! {
        #[derive(DataFrameRow)]
        struct Config {
            opts: ::std::collections::HashMap<String, i32, MyHasher>,
        }
    });
    assert!(
        !code.contains("fn try_from_dataframe"),
        "custom-hasher maps lack `Vec<map>: TryFromSexp`; the reader must stay gated out"
    );
}
