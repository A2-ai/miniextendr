use super::call_method_def;

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
