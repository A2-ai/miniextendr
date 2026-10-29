use super::*;

#[test]
fn test_parse_empty() {
    let input: ListInput = syn::parse_quote!();
    assert!(input.entries.is_empty());
}

#[test]
fn test_parse_unnamed() {
    let input: ListInput = syn::parse_quote!(1, 2, 3);
    assert_eq!(input.entries.len(), 3);
    assert!(input.entries.iter().all(|e| e.name.is_none()));
}

#[test]
fn test_parse_named_ident() {
    let input: ListInput = syn::parse_quote!(alpha = 1, beta = 2);
    assert_eq!(input.entries.len(), 2);
    match &input.entries[0].name {
        Some(ListName::Ident(i)) => assert_eq!(i.to_string(), "alpha"),
        _ => panic!("expected ident name"),
    }
}

#[test]
fn test_parse_named_string() {
    let input: ListInput = syn::parse_quote!("my-name" = 1);
    assert_eq!(input.entries.len(), 1);
    match &input.entries[0].name {
        Some(ListName::Str(s)) => assert_eq!(s.value(), "my-name"),
        _ => panic!("expected string name"),
    }
}

#[test]
fn test_parse_mixed() {
    let input: ListInput = syn::parse_quote!(alpha = 1, 2, beta = 3);
    assert_eq!(input.entries.len(), 3);
    assert!(input.entries[0].name.is_some());
    assert!(input.entries[1].name.is_none());
    assert!(input.entries[2].name.is_some());
}
