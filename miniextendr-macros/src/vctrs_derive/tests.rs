use super::*;

#[test]
fn test_simple_vctr() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "percent", base = "double")]
        struct Percent {
            data: Vec<f64>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    assert!(code.contains("VctrsClass"));
    assert!(code.contains("CLASS_NAME"));
    assert!(code.contains("percent"));
    assert!(code.contains("REALSXP"));
}

#[test]
fn test_simple_vctr_with_data_field() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "percent", base = "double")]
        struct Percent {
            #[vctrs(data)]
            data: Vec<f64>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    // Should generate VctrsClass
    assert!(code.contains("VctrsClass"));
    // Should generate IntoVctrs with data field
    assert!(code.contains("IntoVctrs"));
    assert!(code.contains("self . data"));
    assert!(code.contains("new_vctr"));
}

#[test]
fn test_record_vctr() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "rational", base = "record")]
        struct Rational {
            n: Vec<i32>,
            d: Vec<i32>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    assert!(code.contains("VctrsClass"));
    assert!(code.contains("VctrsRecord"));
    assert!(code.contains("Rcrd"));
    assert!(code.contains("field_names"));
}

#[test]
fn test_record_vctr_with_data() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "rational", base = "record")]
        struct Rational {
            #[vctrs(data)]
            n: Vec<i32>,
            d: Vec<i32>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    // Should generate all three traits
    assert!(code.contains("VctrsClass"));
    assert!(code.contains("VctrsRecord"));
    assert!(code.contains("IntoVctrs"));
    // Record uses new_rcrd
    assert!(code.contains("new_rcrd"));
    // Field names should be included
    assert!(code.contains("\"n\""));
    assert!(code.contains("\"d\""));
}

#[test]
fn test_skip_field() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "rational", base = "record")]
        struct Rational {
            #[vctrs(data)]
            n: Vec<i32>,
            d: Vec<i32>,
            #[vctrs(skip)]
            cached: Option<f64>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    // Field names should NOT include cached
    assert!(code.contains("\"n\""));
    assert!(code.contains("\"d\""));
    assert!(!code.contains("\"cached\""));
}

#[test]
fn test_with_abbr() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "percent", base = "double", abbr = "pct")]
        struct Percent {
            data: Vec<f64>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    assert!(code.contains("pct"));
}

#[test]
fn test_empty_struct_error() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "empty", base = "double")]
        struct Empty {}
    };

    let result = derive_vctrs(input);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("requires at least one field")
    );
}

#[test]
fn test_generic_struct_error() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "generic", base = "double")]
        struct Generic<T> {
            data: Vec<T>,
        }
    };

    let result = derive_vctrs(input);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("does not support generic")
    );
}

#[test]
fn test_missing_class_error() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(base = "double")]
        struct Percent {
            data: Vec<f64>,
        }
    };

    let result = derive_vctrs(input);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("requires #[vctrs(class")
    );
}

#[test]
fn test_enum_error() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "color")]
        enum Color {
            Red,
            Green,
            Blue,
        }
    };

    let result = derive_vctrs(input);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("can only be applied to structs")
    );
}

#[test]
fn test_r_wrappers_const_generated() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "percent", base = "double")]
        struct Percent {
            data: Vec<f64>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    // Should generate the R_WRAPPERS_VCTRS_Percent distributed-slice entry
    assert!(code.contains("static R_WRAPPERS_VCTRS_Percent"));
}

#[test]
fn test_r_wrappers_content_simple() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "percent",
        base: "double",
        abbr: Some("pct"),
        record_fields: &[],
        coerce_with: &[],
        extends: &[],
        inherit_base: false,
        ptype: None,
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: false,
        math: false,
    });

    // Should have format method using unclass (not vec_data to avoid recursion)
    assert!(r_code.contains("format.percent"));
    assert!(r_code.contains("unclass(x)"));

    // Should have vec_ptype_abbr since abbr provided
    assert!(r_code.contains("vec_ptype_abbr.percent"));
    assert!(r_code.contains("\"pct\""));

    // Should have vec_ptype_full
    assert!(r_code.contains("vec_ptype_full.percent"));

    // Should have vec_proxy using unclass (not vec_data to avoid recursion)
    assert!(r_code.contains("vec_proxy.percent"));

    // Should have vec_restore
    assert!(r_code.contains("vec_restore.percent"));

    // Should have self-coercion methods
    assert!(r_code.contains("vec_ptype2.percent.percent"));
    assert!(r_code.contains("vec_cast.percent.percent"));
}

#[test]
fn test_r_wrappers_content_record() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "rational",
        base: "record",
        abbr: None,
        record_fields: &["n".to_string(), "d".to_string()],
        coerce_with: &[],
        extends: &[],
        inherit_base: true, // records default to inherit_base = true
        ptype: None,
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: false,
        math: false,
    });

    // Should have format method with vctrs::field accessors
    assert!(r_code.contains("format.rational"));
    assert!(r_code.contains("vctrs::field(x, \"n\")"));
    assert!(r_code.contains("vctrs::field(x, \"d\")"));

    // Should NOT have vec_ptype_abbr since no abbr
    assert!(!r_code.contains("vec_ptype_abbr.rational"));

    // Should have vec_ptype_full
    assert!(r_code.contains("vec_ptype_full.rational"));

    // Should have vec_proxy and vec_restore for records
    assert!(r_code.contains("vec_proxy.rational"));
    assert!(r_code.contains("vec_restore.rational"));
    // vec_proxy uses new_data_frame, vec_restore uses new_rcrd
    assert!(r_code.contains("new_data_frame"));
    assert!(r_code.contains("new_rcrd"));

    // Should have self-coercion (uses vec_ptype for records)
    assert!(r_code.contains("vec_ptype2.rational.rational"));
    assert!(r_code.contains("vec_ptype(x)"));
    assert!(r_code.contains("vec_cast.rational.rational"));
}

#[test]
fn test_r_wrappers_no_abbr() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "mytype",
        base: "integer",
        abbr: None,
        record_fields: &[],
        coerce_with: &[],
        extends: &[],
        inherit_base: false,
        ptype: None,
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: false,
        math: false,
    });

    // Should NOT have vec_ptype_abbr
    assert!(!r_code.contains("vec_ptype_abbr.mytype"));

    // Should still have format and vec_ptype_full
    assert!(r_code.contains("format.mytype"));
    assert!(r_code.contains("vec_ptype_full.mytype"));
}

#[test]
fn test_r_wrappers_with_coercion() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "percent",
        base: "double",
        abbr: Some("%"),
        record_fields: &[],
        coerce_with: &["double".to_string()],
        extends: &[],
        inherit_base: false,
        ptype: None,
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: false,
        math: false,
    });

    // Should have self-coercion
    assert!(r_code.contains("vec_ptype2.percent.percent"));
    assert!(r_code.contains("vec_cast.percent.percent"));

    // Should have coercion with double
    assert!(r_code.contains("vec_ptype2.percent.double"));
    assert!(r_code.contains("vec_ptype2.double.percent"));
    assert!(r_code.contains("vec_cast.percent.double"));
    assert!(r_code.contains("vec_cast.double.percent"));
}

#[test]
fn test_r_wrappers_list_of() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "list_of_integers",
        base: "list",
        abbr: Some("list<int>"),
        record_fields: &[],
        coerce_with: &[],
        extends: &[],
        inherit_base: true,
        ptype: Some("integer()"),
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: false,
        math: false,
    });

    // Should have format for list_of
    assert!(r_code.contains("format.list_of_integers"));
    assert!(r_code.contains("vapply"));

    // Should have vec_ptype_abbr
    assert!(r_code.contains("vec_ptype_abbr.list_of_integers"));
    assert!(r_code.contains("list<int>"));

    // Should have vec_proxy for list_of
    assert!(r_code.contains("vec_proxy.list_of_integers"));
    assert!(r_code.contains("new_data_frame"));

    // Should have vec_restore with ptype
    assert!(r_code.contains("vec_restore.list_of_integers"));
    assert!(r_code.contains("new_list_of"));
    assert!(r_code.contains("integer()"));

    // Should have vec_ptype2 for list_of
    assert!(r_code.contains("vec_ptype2.list_of_integers.list_of_integers"));
    assert!(r_code.contains("vec_ptype_common"));
}

#[test]
fn test_r_wrappers_proxy_methods() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "mynum",
        base: "double",
        abbr: None,
        record_fields: &[],
        coerce_with: &[],
        extends: &[],
        inherit_base: false,
        ptype: None,
        proxy_equal: true,
        proxy_compare: true,
        proxy_order: true,
        arith: false,
        math: false,
    });

    // Should have proxy_equal method
    assert!(r_code.contains("vec_proxy_equal.mynum"));

    // Should have proxy_compare method
    assert!(r_code.contains("vec_proxy_compare.mynum"));

    // Should have proxy_order method
    assert!(r_code.contains("vec_proxy_order.mynum"));
}

#[test]
fn test_r_wrappers_arith_methods() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "mynum",
        base: "double",
        abbr: None,
        record_fields: &[],
        coerce_with: &[],
        extends: &[],
        inherit_base: false,
        ptype: None,
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: true,
        math: false,
    });

    // Should have base dispatcher for double dispatch
    assert!(r_code.contains("vec_arith.mynum <- function(op, x, y, ...)"));
    assert!(r_code.contains("UseMethod(\"vec_arith.mynum\", y)"));

    // Should have default fallback with @method tag
    assert!(r_code.contains("@method vec_arith.mynum default"));
    assert!(r_code.contains("vec_arith.mynum.default"));
    assert!(r_code.contains("stop_incompatible_op"));

    // Should have vec_arith methods with @method tags for proper S3 registration
    assert!(r_code.contains("@method vec_arith.mynum mynum"));
    assert!(r_code.contains("vec_arith.mynum.mynum"));
    assert!(r_code.contains("@method vec_arith.mynum numeric"));
    assert!(r_code.contains("vec_arith.mynum.numeric"));
    // vec_arith.numeric.mynum uses @method since vec_arith.numeric
    // is exported by vctrs (we import it)
    assert!(r_code.contains("@method vec_arith.numeric mynum"));
    assert!(r_code.contains("@importFrom vctrs vec_arith.numeric"));
    assert!(r_code.contains("vec_arith.numeric.mynum"));
    assert!(r_code.contains("@method vec_arith.mynum MISSING"));
    assert!(r_code.contains("vec_arith.mynum.MISSING"));
    assert!(r_code.contains("vec_arith_base"));
}

#[test]
fn test_r_wrappers_math_methods() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "mynum",
        base: "double",
        abbr: None,
        record_fields: &[],
        coerce_with: &[],
        extends: &[],
        inherit_base: false,
        ptype: None,
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: false,
        math: true,
    });

    // Should have vec_math method
    assert!(r_code.contains("vec_math.mynum"));
    assert!(r_code.contains("vec_math_base"));
}

#[test]
fn test_r_class_vector_helper() {
    // No parents -> bare string.
    assert_eq!(r_class_vector("percent", &[]), "\"percent\"");
    // One parent -> c("child", "parent").
    assert_eq!(
        r_class_vector("fancy_percent", &["percent".to_string()]),
        "c(\"fancy_percent\", \"percent\")"
    );
    // Multiple parents preserve order, child first.
    assert_eq!(
        r_class_vector("c", &["b".to_string(), "a".to_string()]),
        "c(\"c\", \"b\", \"a\")"
    );
}

#[test]
fn test_r_wrappers_with_extends() {
    let r_code = generate_r_wrappers(&RWrapperOptions {
        class: "fancy_percent",
        base: "double",
        abbr: None,
        record_fields: &[],
        coerce_with: &[],
        extends: &["percent".to_string()],
        inherit_base: false,
        ptype: None,
        proxy_equal: false,
        proxy_compare: false,
        proxy_order: false,
        arith: false,
        math: false,
    });

    // The constructed class vector (in vec_restore / vec_ptype2) carries the
    // parent so S3 + coercion fall through.
    assert!(r_code.contains("class = c(\"fancy_percent\", \"percent\")"));

    // Cross-coercion stubs between child and parent are emitted.
    assert!(r_code.contains("vec_ptype2.fancy_percent.percent"));
    assert!(r_code.contains("vec_ptype2.percent.fancy_percent"));
    assert!(r_code.contains("vec_cast.percent.fancy_percent"));
    assert!(r_code.contains("vec_cast.fancy_percent.percent"));

    // ptype2 returns the parent (supertype) prototype.
    assert!(r_code.contains("class = \"percent\""));

    // The default format method is still present; fall-through to the
    // parent's format happens at runtime via the class vector.
    assert!(r_code.contains("format.fancy_percent"));
}

#[test]
fn test_extends_emits_additional_classes_impl() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "fancy_percent", base = "double", extends = "percent")]
        struct FancyPercent {
            #[vctrs(data)]
            data: Vec<f64>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    // additional_classes() override is generated and lists the parent.
    assert!(code.contains("additional_classes"));
    assert!(code.contains("\"percent\""));
}

#[test]
fn test_no_extends_keeps_class_name_slice() {
    let input: DeriveInput = syn::parse_quote! {
        #[vctrs(class = "percent", base = "double")]
        struct Percent {
            #[vctrs(data)]
            data: Vec<f64>,
        }
    };

    let result = derive_vctrs(input).unwrap();
    let code = result.to_string();

    // Without extends, no additional_classes override and the slice stays
    // the unchanged &[CLASS_NAME].
    assert!(!code.contains("additional_classes"));
    assert!(code.contains("VctrsClass > :: CLASS_NAME"));
}
