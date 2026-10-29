use super::*;

#[test]
fn test_simple_derive() {
    let input: DeriveInput = syn::parse_quote! {
        enum Mode {
            Fast,
            Safe,
            Debug,
        }
    };

    let result = derive_match_arg(input).unwrap();
    let code = result.to_string();
    assert!(code.contains("Fast"));
    assert!(code.contains("Safe"));
    assert!(code.contains("Debug"));
    assert!(code.contains("CHOICES"));
    assert!(code.contains("from_choice"));
    assert!(code.contains("to_choice"));
}

#[test]
fn test_rename_all() {
    let input: DeriveInput = syn::parse_quote! {
        #[match_arg(rename_all = "snake_case")]
        enum Mode {
            FastMode,
            SafeMode,
        }
    };

    let result = derive_match_arg(input).unwrap();
    let code = result.to_string();
    assert!(code.contains("fast_mode"));
    assert!(code.contains("safe_mode"));
}

#[test]
fn test_rename_variant() {
    let input: DeriveInput = syn::parse_quote! {
        enum Priority {
            #[match_arg(rename = "lo")]
            Low,
            #[match_arg(rename = "hi")]
            High,
        }
    };

    let result = derive_match_arg(input).unwrap();
    let code = result.to_string();
    assert!(code.contains("\"lo\""));
    assert!(code.contains("\"hi\""));
}

#[test]
fn test_reject_fields() {
    let input: DeriveInput = syn::parse_quote! {
        enum Bad {
            A(i32),
        }
    };

    let result = derive_match_arg(input);
    assert!(result.is_err());
}

#[test]
fn test_reject_struct() {
    let input: DeriveInput = syn::parse_quote! {
        struct Bad;
    };

    let result = derive_match_arg(input);
    assert!(result.is_err());
}

#[test]
fn test_reject_empty() {
    let input: DeriveInput = syn::parse_quote! {
        enum Empty {}
    };

    let result = derive_match_arg(input);
    assert!(result.is_err());
}

#[test]
fn test_into_r_impl_present() {
    // The derive emits IntoR for the scalar EnumName.
    // Vec<EnumName> IntoR is covered by the blanket impl<T: MatchArg> IntoR for Vec<T>
    // in miniextendr-api — it is NOT emitted by the derive.
    let input: DeriveInput = syn::parse_quote! {
        enum Mode {
            Fast,
            Safe,
            Debug,
        }
    };

    let result = derive_match_arg(input).unwrap();
    let code = result.to_string();
    // Scalar IntoR impl must be present
    assert!(code.contains("IntoR for Mode"));
    // Vec<Mode> IntoR must NOT be emitted by the derive (covered by blanket in miniextendr-api)
    assert!(!code.contains("IntoR for :: std :: vec :: Vec < Mode >"));
    assert!(!code.contains("match_arg_vec_into_sexp"));
}

/// `alias = "..."` fills `ALIASES` with (alias, the variant's choice after
/// renaming) entries in written order, is repeatable, is not renamed by
/// `rename_all`, and leaves `CHOICES` alone (#1843). An alias is listed
/// unless its own attribute says `listed = false` (#2148).
#[test]
fn test_aliases() {
    let input: DeriveInput = syn::parse_quote! {
        #[match_arg(rename_all = "lower")]
        enum Shade {
            Red,
            #[match_arg(alias = "grey", alias = "Grau")]
            #[match_arg(alias = "gris", listed = false)]
            Gray,
            #[match_arg(rename = "navy", alias = "Blue", listed = true)]
            Blue,
        }
    };
    let code = derive_match_arg(input).unwrap().to_string();
    assert!(
        code.contains(
            r#"const CHOICES : & 'static [& 'static str] = & ["red" , "gray" , "navy"] ;"#
        ),
        "{code}"
    );
    let alias = |kind: &str, alias: &str, choice: &str| {
        format!(
            ":: miniextendr_api :: match_arg :: MatchArgAlias :: {kind} ({alias:?} , {choice:?})"
        )
    };
    let entries = [
        alias("new", "grey", "gray"),
        alias("new", "Grau", "gray"),
        alias("unlisted", "gris", "gray"),
        alias("new", "Blue", "navy"),
    ];
    assert!(
        code.contains(&format!(
            "const ALIASES : & 'static [:: miniextendr_api :: match_arg :: MatchArgAlias] = & [{}] ;",
            entries.join(" , ")
        )),
        "{code}"
    );
    // An alias is not a choice: `from_choice` matches the choices only.
    assert!(!code.contains(r#""grey" => Some"#), "{code}");
}

/// Without aliases `ALIASES` is empty.
#[test]
fn test_no_aliases() {
    let input: DeriveInput = syn::parse_quote! {
        enum Mode { Fast, Safe }
    };
    let code = derive_match_arg(input).unwrap().to_string();
    assert!(
        code.contains(
            r#"const ALIASES : & 'static [:: miniextendr_api :: match_arg :: MatchArgAlias] = & [] ;"#
        ),
        "{code}"
    );
}

/// `listed` qualifies the aliases of its own attribute: one with none, a
/// repeated one, and a value that is not `true` / `false` are refused.
#[test]
fn test_reject_listed() {
    let error = |input: DeriveInput| match derive_match_arg(input) {
        Ok(_) => String::new(),
        Err(e) => e.to_string(),
    };
    let no_alias = "`listed` applies to the aliases of its own #[match_arg(...)], and this \
                        one has none: write it next to `alias = \"...\"`";
    assert_eq!(
        error(syn::parse_quote! {
            enum Shade {
                #[match_arg(alias = "grey")]
                #[match_arg(listed = false)]
                Gray,
            }
        }),
        no_alias
    );
    assert_eq!(
        error(syn::parse_quote! {
            #[match_arg(listed = false)]
            enum Shade { Gray }
        }),
        no_alias
    );
    assert_eq!(
        error(syn::parse_quote! {
            enum Shade {
                #[match_arg(alias = "grey", listed = false, listed = true)]
                Gray,
            }
        }),
        "`listed` is given more than once in this #[match_arg(...)]"
    );
    for value in [quote!("false"), quote!(no), quote!(0)] {
        let input: DeriveInput = syn::parse_quote! {
            enum Shade {
                #[match_arg(alias = "grey", listed = #value)]
                Gray,
            }
        };
        assert_eq!(
            error(input),
            "expected `listed = true` or `listed = false`",
            "{value}"
        );
    }
}

/// The error messages of the aliases the derive refuses, all reported at
/// once.
#[test]
fn test_reject_aliases() {
    let errors = |input: DeriveInput| -> Vec<String> {
        match derive_match_arg(input) {
            Ok(_) => Vec::new(),
            Err(e) => e.into_iter().map(|e| e.to_string()).collect(),
        }
    };
    assert_eq!(
        errors(syn::parse_quote! {
            #[match_arg(rename_all = "lower")]
            enum Shade {
                #[match_arg(alias = "")]
                Red,
                #[match_arg(alias = "NA", alias = "blue", alias = "grey")]
                Gray,
                #[match_arg(alias = "grey", alias = "bl")]
                Blue,
                #[match_arg(alias = "gr")]
                Green,
            }
        }),
        [
            "an alias can't be empty: the empty string matches no choice",
            "alias \"NA\" would read as a missing value",
            "alias \"blue\" is already a choice",
            "alias \"grey\" is given more than once",
            "alias \"gr\" for \"green\" is a prefix of the choice \"gray\", \
                 so it would change what typing \"gr\" selects",
        ]
    );
    // A prefix of its own choice only is allowed ("bl" above).
    assert!(
        errors(syn::parse_quote! {
            enum Shade {
                Red,
                #[match_arg(alias = "Gr")]
                Gray,
            }
        })
        .is_empty()
    );
    // Aliases go on variants.
    assert_eq!(
        errors(syn::parse_quote! {
            #[match_arg(alias = "grey")]
            enum Shade { Red, Gray }
        }),
        ["`alias` names another spelling of one choice: put \
              #[match_arg(alias = \"...\")] on that choice's variant"]
    );
}

#[test]
fn test_duplicate_choice_names() {
    let input: DeriveInput = syn::parse_quote! {
        enum Dup {
            #[match_arg(rename = "same")]
            A,
            #[match_arg(rename = "same")]
            B,
        }
    };

    let result = derive_match_arg(input);
    assert!(result.is_err());
}
