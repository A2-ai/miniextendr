use super::*;

#[test]
fn test_parse_simple_entry() {
    let input: TypedListInput = syn::parse_quote!(alpha);
    assert_eq!(input.entries.len(), 1);
    assert_eq!(input.entries[0].name.to_string(), "alpha");
    assert!(!input.entries[0].optional);
    assert!(input.entries[0].spec.is_none());
    assert!(input.allow_extra);
}

#[test]
fn test_parse_optional_entry() {
    let input: TypedListInput = syn::parse_quote!(alpha?);
    assert!(input.entries[0].optional);
}

#[test]
fn test_parse_string_spec() {
    let input: TypedListInput = syn::parse_quote!(alpha => "numeric");
    match &input.entries[0].spec {
        Some(ParsedTypeSpec::StringLit(s)) => assert_eq!(s, "numeric"),
        _ => panic!("expected string literal spec"),
    }
}

#[test]
fn test_parse_call_spec() {
    let input: TypedListInput = syn::parse_quote!(alpha => numeric(4));
    match &input.entries[0].spec {
        Some(ParsedTypeSpec::TypeCall { name, len }) => {
            assert_eq!(name, "numeric");
            assert_eq!(*len, Some(4));
        }
        _ => panic!("expected type call spec"),
    }
}

#[test]
fn test_parse_call_no_len() {
    let input: TypedListInput = syn::parse_quote!(alpha => list());
    match &input.entries[0].spec {
        Some(ParsedTypeSpec::TypeCall { name, len }) => {
            assert_eq!(name, "list");
            assert_eq!(*len, None);
        }
        _ => panic!("expected type call spec"),
    }
}

#[test]
fn test_parse_exact_mode() {
    let input: TypedListInput = syn::parse_quote!(@exact; alpha);
    assert!(!input.allow_extra);
    assert_eq!(input.entries.len(), 1);
}

#[test]
fn test_parse_multiple_entries() {
    let input: TypedListInput = syn::parse_quote!(alpha => numeric(4), beta? => "list", gamma);
    assert_eq!(input.entries.len(), 3);
    assert!(!input.entries[0].optional);
    assert!(input.entries[1].optional);
    assert!(!input.entries[2].optional);
}
