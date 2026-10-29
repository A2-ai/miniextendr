use super::*;

#[test]
fn test_parse_basic() {
    let input: TypedDataframeInput = syn::parse_quote! {
        pub TheophDf {
            subject: i32,
            weight: f64,
        }
    };
    assert_eq!(input.name.to_string(), "TheophDf");
    assert_eq!(input.fields.len(), 2);
    assert_eq!(input.fields[0].name.to_string(), "subject");
    assert!(!input.fields[0].optional);
    assert!(input.allow_extra);
}

#[test]
fn test_parse_optional() {
    let input: TypedDataframeInput = syn::parse_quote! {
        pub Df { x: i32, y: Option<f64> }
    };
    assert!(!input.fields[0].optional);
    assert!(input.fields[1].optional);
}

#[test]
fn test_parse_exact_mode() {
    let input: TypedDataframeInput = syn::parse_quote! {
        @exact;
        pub Df { x: i32 }
    };
    assert!(!input.allow_extra);
}

#[test]
fn test_parse_with_doc_attrs() {
    let input: TypedDataframeInput = syn::parse_quote! {
        /// Theoph PK shape.
        pub TheophDf {
            /// Subject id.
            subject: i32,
        }
    };
    assert!(!input.attrs.is_empty());
    assert!(!input.fields[0].attrs.is_empty());
}

#[test]
fn test_parse_empty_fails() {
    let result: syn::Result<TypedDataframeInput> = syn::parse2(quote! {
        pub Df {}
    });
    assert!(result.is_err());
}
