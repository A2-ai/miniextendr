use super::*;

fn derive(input: DeriveInput) -> String {
    derive_r_condition(input).unwrap().to_string()
}

fn derive_err(input: DeriveInput) -> String {
    derive_r_condition(input).unwrap_err().to_string()
}

#[test]
fn condition_rejects_invalid_and_duplicate_kinds() {
    for input in [
        syn::parse_quote! {
            #[condition(kind = panic)]
            struct Invalid;
        },
        syn::parse_quote! {
            #[condition(kind = warning, kind = error)]
            struct Duplicate;
        },
        syn::parse_quote! {
            enum InvalidVariant {
                #[condition(kind = conversion)]
                Bad,
            }
        },
    ] {
        assert!(derive_r_condition(input).is_err());
    }
}

#[test]
fn kind_defaults_to_error_and_a_variant_overrides_the_type() {
    let code = derive(syn::parse_quote! {
        struct Plain;
    });
    assert!(code.contains("Self :: Error { message , class , data , call }"));
    let code = derive(syn::parse_quote! {
        #[condition(kind = warning)]
        enum Mixed {
            #[condition(kind = message)]
            Note,
            #[condition(kind = error)]
            Fatal,
            Drift,
        }
    });
    assert!(code.contains("Mixed :: Note => Self :: Message"));
    assert!(code.contains("Mixed :: Fatal => Self :: Error"));
    assert!(code.contains("Mixed :: Drift => Self :: Warning"));
}

#[test]
fn enum_classes_default_to_snake_case_family_and_member() {
    let code = derive(syn::parse_quote! {
        enum PkgError {
            MissingField { field: String },
            OutOfRange { value: f64, max: f64 },
            Plain,
        }
    });
    assert!(code.contains("\"pkg_error_missing_field\""));
    assert!(code.contains("\"pkg_error_out_of_range\""));
    assert!(code.contains("\"pkg_error_plain\""));
    assert!(code.contains("\"pkg_error\""));
    // No `message` anywhere: Display is the message.
    assert!(code.contains("ToString :: to_string (self)"));
    assert!(!code.contains("format !"));
    // Fields become data entries under their own names.
    assert!(code.contains("\"field\""));
    assert!(code.contains("\"max\""));
    assert!(code.contains("Clone :: clone (max)"));
}

#[test]
fn explicit_classes_messages_and_field_options() {
    let code = derive(syn::parse_quote! {
        #[condition(class = "pkg_warning")]
        enum PkgWarning {
            #[condition(message = "dropped {dropped} of {total} rows")]
            Truncated { dropped: i32, total: i32 },
            #[condition(class = "pkg_warning_slow_path", message = "slow")]
            Slow,
            Other {
                #[condition(rename = "n_attempts")]
                attempts: i32,
                #[condition(debug)]
                range: std::ops::Range<i32>,
                #[condition(skip)]
                detail: Vec<u8>,
            },
        }
    });
    assert!(code.contains("\"pkg_warning_truncated\""));
    assert!(code.contains("\"pkg_warning_slow_path\""));
    assert!(code.contains("\"pkg_warning_other\""));
    assert!(code.contains("format ! (\"dropped {dropped} of {total} rows\")"));
    // A variant without `message` keeps the Display fallback arm.
    assert!(code.contains("_ => :: std :: string :: ToString :: to_string (self)"));
    assert!(code.contains("\"n_attempts\""));
    assert!(!code.contains("\"attempts\""));
    assert!(code.contains("RValue :: debug (range)"));
    assert!(!code.contains("\"detail\""));
    // The skipped field is not bound in the data pattern.
    assert!(code.contains("Self :: Other { attempts , range , .. } =>"));
}

#[test]
fn struct_shapes() {
    let named = derive(syn::parse_quote! {
        #[condition(message = "took {attempts} attempts")]
        struct RetryNotice {
            attempts: i32,
        }
    });
    assert!(named.contains("\"retry_notice\""));
    assert!(named.contains("let Self { attempts } = self"));
    assert!(named.contains("format ! (\"took {attempts} attempts\")"));

    let tuple = derive(syn::parse_quote! {
        struct Pair(#[condition(rename = "left")] i32, #[condition(skip)] String);
    });
    assert!(tuple.contains("\"left\""));
    assert!(tuple.contains("Self (_0 , _)"));

    let unit = derive(syn::parse_quote! {
        struct Nothing;
    });
    assert!(unit.contains("Option :: None"));
    assert!(unit.contains("\"nothing\""));
}

#[test]
fn call_none_on_the_type_or_a_variant() {
    // On a variant: only that variant is call-less.
    let code = derive(syn::parse_quote! {
        #[condition(class = "pkg_warning")]
        enum PkgWarning {
            #[condition(call = none, message = "row {row} overrides an earlier one")]
            Overridden { row: i32 },
            Truncated { dropped: i32 },
        }
    });
    assert!(code.contains("fn call (& self)"));
    assert!(code.contains(
        "Self :: Overridden { .. } => :: miniextendr_api :: condition :: ConditionCall :: None"
    ));
    assert!(code.contains("_ => :: miniextendr_api :: condition :: ConditionCall :: Inherit"));

    // On the type, or on every variant: no match at all.
    for code in [
        derive(syn::parse_quote! {
            #[condition(call = none)]
            enum E { A, B { n: i32 } }
        }),
        derive(syn::parse_quote! {
            enum E {
                #[condition(call = none)]
                A,
                #[condition(call = none)]
                B { n: i32 },
            }
        }),
        derive(syn::parse_quote! {
            #[condition(call = none, message = "plain")]
            struct Notice;
        }),
    ] {
        assert!(code.contains("fn call (& self) -> :: miniextendr_api :: condition :: ConditionCall { :: miniextendr_api :: condition :: ConditionCall :: None }"), "{code}");
    }

    // Without `call = none` the trait default stays.
    let code = derive(syn::parse_quote! {
        enum E { A, B }
    });
    assert!(!code.contains("fn call"));
}

#[test]
fn rejects_bad_call_values() {
    let err = derive_err(syn::parse_quote! {
        #[condition(call = wrapper)]
        struct S;
    });
    assert!(err.contains("unknown `call` value `wrapper`"), "{err}");

    let err = derive_err(syn::parse_quote! {
        #[condition(call = "none")]
        struct S;
    });
    assert!(err.contains("`call` takes `none`"), "{err}");

    let err = derive_err(syn::parse_quote! {
        enum E {
            #[condition(call = none, call = none)]
            A,
        }
    });
    assert!(err.contains("`call` is given twice"), "{err}");

    let err = derive_err(syn::parse_quote! {
        struct S {
            #[condition(call = none)]
            a: i32,
        }
    });
    assert!(
        err.contains("unknown `condition` option on a field"),
        "{err}"
    );
}

#[test]
fn rejects_type_and_const_parameters_and_unions() {
    let err = derive_err(syn::parse_quote! {
        struct Generic<T> {
            value: T,
        }
    });
    assert!(err.contains("type or const parameters"), "{err}");

    let err = derive_err(syn::parse_quote! {
        struct Sized<'a, const N: usize> {
            value: &'a [i32; N],
        }
    });
    assert!(err.contains("type or const parameters"), "{err}");

    let err = derive_err(syn::parse_quote! {
        union U {
            a: u32,
            b: f32,
        }
    });
    assert!(err.contains("not unions"));
}

#[test]
fn rejects_unnamed_tuple_fields_and_reserved_or_duplicate_names() {
    let err = derive_err(syn::parse_quote! {
        enum E {
            Raw(i32),
        }
    });
    assert!(err.contains("tuple field 0 of variant `E::Raw` needs a `data` name"));

    let err = derive_err(syn::parse_quote! {
        struct S {
            kind: String,
        }
    });
    assert!(err.contains("`kind` is one of the condition's own slots"));

    let err = derive_err(syn::parse_quote! {
        struct S {
            #[condition(rename = "message")]
            text: String,
        }
    });
    assert!(err.contains("`message` is one of the condition's own slots"));

    let err = derive_err(syn::parse_quote! {
        struct S {
            a: i32,
            #[condition(rename = "a")]
            b: i32,
        }
    });
    assert!(err.contains("duplicate `data` field name `a`"));
}

#[test]
fn rejects_misplaced_or_unknown_options() {
    let err = derive_err(syn::parse_quote! {
        #[condition(message = "nope")]
        enum E {
            A,
        }
    });
    assert!(err.contains("goes on the variants"));

    let err = derive_err(syn::parse_quote! {
        #[condition(prefix = "x")]
        struct S {
            a: i32,
        }
    });
    assert!(err.contains("unknown `condition` option on the type"));

    let err = derive_err(syn::parse_quote! {
        struct S {
            #[condition(class = "x")]
            a: i32,
        }
    });
    assert!(err.contains("unknown `condition` option on a field"));

    let err = derive_err(syn::parse_quote! {
        struct S {
            #[condition(skip, rename = "b")]
            a: i32,
        }
    });
    assert!(err.contains("`skip` cannot be combined"));

    let err = derive_err(syn::parse_quote! {
        enum Empty {}
    });
    assert!(err.contains("at least one variant"));

    let err = derive_err(syn::parse_quote! {
        #[condition(argument_message = "nope")]
        enum E {
            A,
        }
    });
    assert!(
        err.contains("`argument_message` goes on the variants"),
        "{err}"
    );

    let err = derive_err(syn::parse_quote! {
        #[condition(argument_message = "a", argument_message = "b")]
        struct S;
    });
    assert!(err.contains("`argument_message` is given twice"), "{err}");
}

/// `argument_message` on a struct, or on some variants of an enum (the
/// rest keep `None`), is a `format!` string over the fields; without it
/// the impl keeps the trait's default (#1833).
#[test]
fn argument_message_on_a_struct_or_on_variants() {
    let code = derive(syn::parse_quote! {
        #[condition(argument_message = "use model_from_df() for a {what}")]
        struct ModelRefused {
            what: String,
        }
    });
    assert!(code.contains("fn argument_message (& self)"), "{code}");
    assert!(
        code.contains("Some (:: std :: format ! (\"use model_from_df() for a {what}\"))"),
        "{code}"
    );

    let code = derive(syn::parse_quote! {
        enum Refused {
            #[condition(argument_message = "use model_from_df() for a data frame")]
            DataFrame,
            Other { class: String },
        }
    });
    assert!(code.contains("fn argument_message (& self)"), "{code}");
    assert!(
        code.contains("_ => :: core :: option :: Option :: None"),
        "{code}"
    );

    let code = derive(syn::parse_quote! {
        struct Plain;
    });
    assert!(!code.contains("fn argument_message"), "{code}");
}

#[test]
fn lifetime_parameters_reach_both_impls() {
    let code = derive(syn::parse_quote! {
        #[condition(class = "pkg_engine_warning")]
        struct EngineWarning<'a>(#[condition(skip)] &'a str);
    });
    assert!(
        code.contains("impl < 'a > :: core :: convert :: From < EngineWarning < 'a > >"),
        "{code}"
    );
    assert!(
        code.contains("fn from (value : EngineWarning < 'a >)"),
        "{code}"
    );
    assert!(
        code.contains("RConditionData for EngineWarning < 'a >"),
        "{code}"
    );
}

#[test]
fn class_list_on_a_struct_is_its_whole_class_vector() {
    let code = derive(syn::parse_quote! {
        #[condition(class("pkg_message_selection_dropped", "pkg_message"))]
        struct SelectionDropped {
            missing: Vec<String>,
        }
    });
    assert!(
        code.contains(
            r#"vec ! [:: std :: string :: String :: from ("pkg_message_selection_dropped") , :: std :: string :: String :: from ("pkg_message")]"#
        ),
        "{code}"
    );
}

#[test]
fn class_list_on_a_variant_replaces_the_member() {
    let code = derive(syn::parse_quote! {
        #[condition(class = "pkg_warning")]
        enum PkgWarning {
            #[condition(class("pkg_warning_engine_masked", "pkg_engine_warning"))]
            Masked,
            Plain,
        }
    });
    assert!(
        code.contains(
            r#"Self :: Masked => :: std :: vec ! [:: std :: string :: String :: from ("pkg_warning_engine_masked") , :: std :: string :: String :: from ("pkg_engine_warning") , :: std :: string :: String :: from ("pkg_warning") ,]"#
        ),
        "{code}"
    );
    assert!(code.contains(r#""pkg_warning_plain""#), "{code}");
}

#[test]
fn class_list_errors() {
    let err = derive_err(syn::parse_quote! {
        #[condition(class("a", "b"))]
        enum E {
            One,
        }
    });
    assert!(
        err.contains("on an enum, `class(...)` goes on a variant"),
        "{err}"
    );

    let err = derive_err(syn::parse_quote! {
        #[condition(class())]
        struct S;
    });
    assert!(err.contains("needs at least one class"), "{err}");

    let err = derive_err(syn::parse_quote! {
        #[condition(class = "a", class("b"))]
        struct S;
    });
    assert!(err.contains("`class` is given twice"), "{err}");
}

#[test]
fn data_entries_follow_the_fields() {
    let code = derive(syn::parse_quote! {
        #[condition(class = "pkg_warning", data(param = "unit_table"))]
        struct UnitTableIgnored;
    });
    assert!(
        code.contains(
            r#"Some (:: std :: vec ! [(:: std :: string :: String :: from ("param") , :: miniextendr_api :: RValue :: from ("unit_table"))])"#
        ),
        "{code}"
    );

    // On an enum: the fields' entries, then the type's, then the variant's.
    let code = derive(syn::parse_quote! {
        #[condition(data(source = "engine"))]
        enum E {
            #[condition(data(r#type = 2i32))]
            One { n: i32 },
        }
    });
    let n = code.find(r#""n""#).expect("field entry");
    let source = code.find(r#""source""#).expect("type entry");
    let ty = code.find(r#""type""#).expect("variant entry");
    assert!(n < source && source < ty, "{code}");
}

#[test]
fn data_entry_names_are_checked_like_fields() {
    let err = derive_err(syn::parse_quote! {
        #[condition(data(kind = "x"))]
        struct S;
    });
    assert!(
        err.contains("`kind` is one of the condition's own slots"),
        "{err}"
    );

    let err = derive_err(syn::parse_quote! {
        #[condition(data(n = 1i32))]
        struct S {
            n: i32,
        }
    });
    assert!(err.contains("duplicate `data` field name `n`"), "{err}");

    let err = derive_err(syn::parse_quote! {
        #[condition(data(a::b = 1i32))]
        struct S;
    });
    assert!(err.contains("a `data` entry is `name = value`"), "{err}");
}

#[test]
fn skip_none_leaves_a_none_entry_out() {
    let code = derive(syn::parse_quote! {
        struct Masked {
            verb: String,
            #[condition(skip_none)]
            given_to: Option<String>,
        }
    });
    assert!(
        code.contains("if let :: core :: option :: Option :: Some (__value) = given_to"),
        "{code}"
    );
    assert!(
        code.contains("(! __data . is_empty ()) . then_some (__data)"),
        "{code}"
    );
    // The plain field is pushed whatever its value.
    assert!(
        code.contains(r#"__data . push ((:: std :: string :: String :: from ("verb")"#),
        "{code}"
    );
}

#[test]
fn skip_none_errors() {
    let err = derive_err(syn::parse_quote! {
        struct S {
            #[condition(skip_none)]
            n: i32,
        }
    });
    assert!(
        err.contains("`skip_none` needs an `Option<_>` field"),
        "{err}"
    );

    let err = derive_err(syn::parse_quote! {
        struct S {
            #[condition(skip, skip_none)]
            n: Option<i32>,
        }
    });
    assert!(
        err.contains("`skip` cannot be combined with `rename`, `debug` or `skip_none`"),
        "{err}"
    );
}
