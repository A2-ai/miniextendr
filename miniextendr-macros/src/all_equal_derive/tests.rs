use super::*;

fn input(tokens: proc_macro2::TokenStream) -> DeriveInput {
    syn::parse2(tokens).expect("a struct")
}

fn error(tokens: proc_macro2::TokenStream) -> String {
    match derive_all_equal(input(tokens)) {
        Ok(_) => panic!("expected the derive to be refused"),
        Err(err) => err.to_string(),
    }
}

/// The guard after each `.Call()`: the method's frame resolves to its call.
const GUARD: &str = "  if (inherits(.val, \"rust_condition_value\") && \
                     isTRUE(attr(.val, \"__rust_condition__\"))) \
                     return(.miniextendr_raise_condition(.val, environment()))";

#[test]
fn struct_settings_are_formal_defaults() {
    let input = input(quote! {
        #[all_equal(tolerance = 1e-6, check_attributes = false)]
        struct Fit { values: Vec<f64> }
    });
    let settings = parse_struct_settings(&input).unwrap();
    assert_eq!(
        settings,
        [
            (Setting::Tolerance, "1e-6".to_owned()),
            (Setting::CheckAttributes, "FALSE".to_owned()),
        ]
    );
    let fields = compared_fields(&input).unwrap();
    let method = r_method("C_x", &settings, &fields);
    assert!(method.starts_with(
        "function(target, current, tolerance = 1e-6, scale, ..., check.attributes = FALSE, check.names) {\n"
    ));
}

#[test]
fn numbers_read_as_r_numbers() {
    assert_eq!(r_number(0.0), "0");
    assert_eq!(r_number(2.0), "2");
    assert_eq!(r_number(1e-6), "1e-6");
    assert_eq!(r_number(0.5), "0.5");
}

/// The example of the issue: a label, a fixed tolerance, a nested class and
/// a skipped field.
#[test]
fn method_compares_each_field_with_its_settings() {
    let input = input(quote! {
        #[all_equal(tolerance = 1e-6)]
        pub struct Fit {
            #[all_equal(label = "Estimates")]
            estimates: Vec<f64>,
            #[all_equal(tolerance = 0)]
            iterations: i32,
            model: ExternalPtr<Model>,
            #[all_equal(skip)]
            cache: RefCell<Option<Vec<f64>>>,
        }
    });
    let settings = parse_struct_settings(&input).unwrap();
    let fields = compared_fields(&input).unwrap();
    let read = |index: usize, var: &str, receiver: &str, value: &str| {
        format!("  .val <- .Call(C_x, {receiver}, {index}L)\n{GUARD}\n  {var} <- {value}\n")
    };
    let field = |index: usize, value: &str| {
        format!(
            "{}{}",
            read(index, ".target", "target", value),
            read(index, ".current", "current", value)
        )
    };
    let expected = format!(
        "function(target, current, tolerance = 1e-6, scale, ..., check.attributes, check.names) {{\n\
         \x20 if (data.class(target) != data.class(current)) {{\n\
         \x20   return(gettextf(\"target is %s, current is %s\", data.class(target), data.class(current), domain = \"R-base\"))\n\
         \x20 }}\n\
         \x20 .settings <- list()\n\
         \x20 .settings[\"tolerance\"] <- list(tolerance)\n\
         \x20 if (!missing(scale)) .settings[\"scale\"] <- list(scale)\n\
         \x20 if (!missing(check.attributes)) .settings[\"check.attributes\"] <- list(check.attributes)\n\
         \x20 if (!missing(check.names)) .settings[\"check.names\"] <- list(check.names)\n\
         \x20 .dots <- list(...)\n\
         \x20 .msg <- character(0)\n\
         \x20 # `estimates`\n\
         {}\
         \x20 .val <- do.call(all.equal, c(list(.target, .current), .settings, .dots))\n\
         \x20 if (is.character(.val)) .msg <- c(.msg, paste0(\"Estimates: \", .val))\n\
         \x20 # `iterations`\n\
         {}\
         \x20 .args <- .settings\n\
         \x20 .args[\"tolerance\"] <- list(0)\n\
         \x20 .val <- do.call(all.equal, c(list(.target, .current), .args, .dots))\n\
         \x20 if (is.character(.val)) .msg <- c(.msg, paste0(paste0(\"Component \", dQuote(\"iterations\"), \": \"), .val))\n\
         \x20 # `model`\n\
         {}\
         \x20 .val <- do.call(all.equal, c(list(.target, .current), .settings, .dots))\n\
         \x20 if (is.character(.val)) .msg <- c(.msg, paste0(paste0(\"Component \", dQuote(\"model\"), \": \"), .val))\n\
         \x20 if (length(.msg)) .msg else TRUE\n\
         }}",
        field(1, ".val"),
        field(2, ".val"),
        field(3, ".__MX_WRAP_RETURN_Model__(.val)"),
    );
    assert_eq!(r_method("C_x", &settings, &fields), expected);

    let docs = r_docs("Fit", &settings, &fields);
    assert!(docs.contains(
        "#' The fields compared: `estimates`, `iterations` (`tolerance = 0`) and `model`.\n"
    ));
    assert!(docs.contains("#'   The type's defaults are `tolerance = 1e-6`.\n"));
}

#[test]
fn field_values_follow_the_field() {
    let input = input(quote! {
        pub struct Kinds {
            #[r_data]
            r: RSidecar,
            #[r_data]
            pub keys: Sidecar<Vec<i32>>,
            #[r_data(get = "Self::total", name = "sum")]
            pub total: Computed,
            #[all_equal(with = "Self::summary")]
            summary: Summary,
            parent: Option<ExternalPtr<crate::Parent>>,
            plain: String,
        }
    });
    let fields = compared_fields(&input).unwrap();
    let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["keys", "sum", "summary", "parent", "plain"]);
    assert!(matches!(fields[0].value, FieldValue::Sidecar(0)));
    let FieldValue::Getter(total, _) = &fields[1].value else {
        panic!("a computed field reads its getter");
    };
    assert_eq!(quote!(#total).to_string(), "Kinds :: total");
    let FieldValue::Getter(summary, _) = &fields[2].value else {
        panic!("`with` names a getter");
    };
    assert_eq!(quote!(#summary).to_string(), "Kinds :: summary");
    assert!(matches!(
        fields[3].value,
        FieldValue::Pointer { optional: true }
    ));
    assert!(matches!(
        &fields[3].wrap,
        Some(ClassWrap { class, list: false, optional: true }) if class == "Parent"
    ));
    assert!(matches!(fields[4].value, FieldValue::Converted));
    assert!(fields[4].wrap.is_none());

    // An optional pointer stays `NULL` when it is `None`.
    let method = r_method("C_x", &Settings::new(), &fields);
    assert!(
        method.contains(
            "  .target <- if (is.null(.val)) NULL else .__MX_WRAP_RETURN_Parent__(.val)\n"
        )
    );
}

/// A field whose type names a pointer class gets the class's R class, as a
/// method returning that type does: by value, as a pointer, and in `Option`
/// and `Vec`. A pointer field is read as it is; the R value of any other
/// field is converted from a clone.
#[test]
fn class_fields_get_their_class() {
    let input = input(quote! {
        pub struct Holder {
            by_value: Model,
            pointer: ExternalPtr<Model>,
            maybe: Option<ExternalPtr<Model>>,
            pointers: Vec<ExternalPtr<Model>>,
            values: Option<Vec<Model>>,
            numbers: Vec<f64>,
            name: Option<String>,
            table: HashMap<String, Model>,
        }
    });
    let fields = compared_fields(&input).unwrap();
    let wraps: Vec<Option<String>> = fields
        .iter()
        .map(|field| field.wrap.as_ref().map(ClassWrap::r_expression))
        .collect();
    assert_eq!(
        wraps,
        [
            Some(".__MX_WRAP_RETURN_Model__(.val)".to_owned()),
            Some(".__MX_WRAP_RETURN_Model__(.val)".to_owned()),
            Some("if (is.null(.val)) NULL else .__MX_WRAP_RETURN_Model__(.val)".to_owned()),
            Some(".__MX_WRAP_LIST_RETURN_Model__(.val)".to_owned()),
            Some("if (is.null(.val)) NULL else .__MX_WRAP_LIST_RETURN_Model__(.val)".to_owned()),
            None,
            None,
            None,
        ]
    );
    let pointers: Vec<Option<bool>> = fields
        .iter()
        .map(|field| match field.value {
            FieldValue::Pointer { optional } => Some(optional),
            _ => None,
        })
        .collect();
    assert_eq!(
        pointers,
        [None, Some(false), Some(true), None, None, None, None, None]
    );
}

#[test]
fn tuple_fields_are_numbered() {
    let input = input(quote! { struct Pair(f64, #[all_equal(label = "Second")] f64); });
    let fields = compared_fields(&input).unwrap();
    assert_eq!(fields[0].prefix(), "\"Component 1: \"");
    assert_eq!(fields[1].prefix(), "\"Second: \"");
}

#[test]
fn derive_emits_the_entry_and_its_registration() {
    let tokens = derive_all_equal(input(quote! {
        pub struct Fit {
            values: Vec<f64>,
            #[r_data]
            pub keys: Sidecar<Vec<i32>>,
        }
    }))
    .unwrap()
    .to_string();
    let c_name = crate::naming::all_equal_c_name("Fit");
    assert!(tokens.contains(&format!("pub unsafe extern \"C-unwind\" fn {c_name}")));
    assert!(tokens.contains(
        "1i32 => { let __mx_data = unsafe { :: miniextendr_api :: externalptr :: sidecar_r_struct :: < Fit > (x) } ;"
    ));
    assert!(tokens.contains(
        ":: miniextendr_api :: into_r :: IntoR :: into_sexp (:: core :: clone :: Clone :: clone (& __mx_value . values) ,)"
    ));
    assert!(tokens.contains(
        "2i32 => unsafe { :: miniextendr_api :: externalptr :: sidecar_r_get :: < Fit > (x , 0usize , true) } ,"
    ));
    assert!(tokens.contains("__MX_CALL_DEF_ALL_EQUAL_Fit"));
    assert!(tokens.contains("MX_ALL_EQUAL"));
    assert!(tokens.contains("rust_type : \"Fit\""));
}

#[test]
fn refusals() {
    assert!(error(quote! { enum E { A } }).contains("`AllEqual` derives on a struct"));
    assert!(error(quote! { struct G<T> { v: T } }).contains("does not support generic structs"));
    assert!(
        error(quote! { struct S { #[all_equal(skip)] v: i32 } }).contains("no field to compare")
    );
    assert!(error(quote! { struct S; }).contains("no field to compare"));
    assert_eq!(
        error(quote! { struct S { #[all_equal(skip, label = "V")] v: i32 } }),
        "`skip` leaves the field out of the comparison, so it takes no other option"
    );
    assert!(
        error(quote! { struct S { #[all_equal(tolerence = 0)] v: i32 } })
            .starts_with("unknown `all_equal` option `tolerence`; a field takes")
    );
    assert!(
        error(quote! { #[all_equal(label = "S")] struct S { v: i32 } })
            .starts_with("`label` applies to a field")
    );
    assert_eq!(
        error(quote! { #[all_equal(tolerance = 1, tolerance = 2)] struct S { v: i32 } }),
        "`tolerance` is given twice"
    );
    assert_eq!(
        error(quote! { struct S { #[all_equal(tolerance = -1)] v: f64 } }),
        "`tolerance` takes a non-negative number, e.g. `tolerance = 1e-6`"
    );
    assert_eq!(
        error(quote! { struct S { #[all_equal(scale = 0)] v: f64 } }),
        "`scale` takes a positive number, e.g. `scale = 1`"
    );
    assert_eq!(
        error(quote! { #[all_equal(check_names = "no")] struct S { v: f64 } }),
        "`check_names` takes `true` or `false`"
    );
    assert!(
        error(quote! { struct S { #[all_equal(with = "not a path")] v: f64 } })
            .starts_with("`with` must be a path to a getter function, e.g. `Self::f`")
    );
}
