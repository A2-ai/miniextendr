use super::{call_method_def, call_method_def_value};

#[test]
fn registers_the_linker_symbol_with_the_given_arity() {
    let def: syn::Ident = syn::parse_quote!(__MX_CALL_DEF_RDATA_SET_Engine_rate);
    let c_ident: syn::Ident = syn::parse_quote!(C_Engine__set_rate);
    let code = call_method_def(&def, &c_ident, 3).to_string();
    assert!(
        code.contains(r#"name : c"C_Engine__set_rate" . as_ptr ()"#),
        "{code}"
    );
    assert!(code.contains("numArgs : 3"), "{code}");
    // The transmute names the arity in its source type, so a wrapper with
    // another parameter count fails to compile instead of registering.
    assert!(
        code.contains(
            "fn (:: miniextendr_api :: SEXP , :: miniextendr_api :: SEXP , :: miniextendr_api :: SEXP) -> :: miniextendr_api :: SEXP"
        ),
        "{code}"
    );
    assert!(code.contains("MX_CALL_DEFS"), "{code}");
}

/// The value alone is the same registration, for a sidecar setter's entry
/// in `MX_SIDECAR_SETTERS` (#2292).
#[test]
fn the_value_is_the_registration_without_the_static() {
    let c_ident: syn::Ident = syn::parse_quote!(C_Engine__set_rate);
    let value = call_method_def_value(&c_ident, 3).to_string();
    assert!(
        value.starts_with(":: miniextendr_api :: sys :: R_CallMethodDef {"),
        "{value}"
    );
    assert!(!value.contains("MX_CALL_DEFS"), "{value}");
    let def: syn::Ident = syn::parse_quote!(__MX_CALL_DEF_RDATA_SET_Engine_rate);
    assert!(
        call_method_def(&def, &c_ident, 3)
            .to_string()
            .contains(&value)
    );
}
