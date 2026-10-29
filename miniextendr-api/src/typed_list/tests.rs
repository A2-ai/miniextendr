use super::*;

#[test]
fn test_type_spec_names() {
    assert_eq!(TypeSpec::Any.type_name(), "any");
    assert_eq!(TypeSpec::Numeric(None).type_name(), "numeric");
    assert_eq!(TypeSpec::Numeric(Some(4)).type_name(), "numeric(4)");
    assert_eq!(
        TypeSpec::Class("data.frame").type_name(),
        "class: data.frame"
    );
}

#[test]
fn test_typed_entry_constructors() {
    let req = TypedEntry::required("foo", TypeSpec::Integer(None));
    assert_eq!(req.name, "foo");
    assert!(!req.optional);

    let opt = TypedEntry::optional("bar", TypeSpec::Character(None));
    assert_eq!(opt.name, "bar");
    assert!(opt.optional);

    let any = TypedEntry::any("baz");
    assert_eq!(any.spec, TypeSpec::Any);
    assert!(!any.optional);
}

#[test]
fn test_sexptype_name() {
    assert_eq!(sexptype_name(SEXPTYPE::REALSXP), "numeric");
    assert_eq!(sexptype_name(SEXPTYPE::INTSXP), "integer");
    assert_eq!(sexptype_name(SEXPTYPE::STRSXP), "character");
    assert_eq!(sexptype_name(SEXPTYPE::VECSXP), "list");
}
