use super::*;

fn parse_param(s: &str) -> syn::FnArg {
    let sig: syn::Signature = syn::parse_str(&format!("fn test({})", s)).unwrap();
    sig.inputs.into_iter().next().unwrap()
}

#[test]
fn test_unit_type() {
    let builder = RustConversionBuilder::new();
    let param = parse_param("_unused: ()");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("let"));
    }
}

#[test]
fn test_basic_conversion() {
    let builder = RustConversionBuilder::new();
    let param = parse_param("x: i32");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("TryFromSexp"));
    }
}

#[test]
fn test_slice_conversion() {
    let builder = RustConversionBuilder::new();
    let param = parse_param("x: &[i32]");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("TryFromSexp"));
    }
}

#[test]
fn test_str_conversion_main_thread_borrows_zero_copy() {
    // Main-thread path (`build_conversion`): `&str` borrows R's CHARSXP pool
    // directly — a single zero-copy `TryFromSexp` binding, no `String` allocation.
    let builder = RustConversionBuilder::new();
    let param = parse_param("s: &str");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1); // single zero-copy borrow
        let s = stmts[0].to_string();
        assert!(s.contains("TryFromSexp"));
        // No owning-String detour and no Borrow trait on the main-thread path.
        assert!(!s.contains("String"));
        assert!(!s.contains("Borrow"));
    }
}

#[test]
fn test_str_conversion_worker_copies_then_borrows() {
    // Worker path (`build_conversion_split`): `&str` must be owned-then-borrowed
    // because a borrowed view over R's CHARSXP pool is `!Send`.
    let builder = RustConversionBuilder::new();
    let param = parse_param("s: &str");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let (owned, borrowed) = builder.build_conversion_split(&pat_type, &sexp_ident);
        assert_eq!(owned.len(), 1); // String storage on main thread
        assert_eq!(borrowed.len(), 1); // borrow inside worker closure
        assert!(owned[0].to_string().contains("String"));
        assert!(borrowed[0].to_string().contains("Borrow"));
    }
}

#[test]
fn test_coercion() {
    let builder = RustConversionBuilder::new().with_coerce_param("x".to_string());
    let param = parse_param("x: u16");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("TryCoerce"));
        assert!(stmts[0].to_string().contains("u16"));
    }
}

#[test]
fn native_metadata_matches_selected_conversion_paths() {
    let arg = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
    for (ty, specialized) in [
        ("()", true),
        ("&Dots", true),
        ("&str", true),
        ("i64", true),
        ("u16", true),
        ("Option<Mode>", true),
        ("Vec<Mode>", true),
        ("Box<[Mode]>", true),
        ("[Mode; 2]", true),
        ("&[Mode]", true),
        ("&mut i32", false),
        ("HiddenBorrow", false),
    ] {
        let builder = RustConversionBuilder::new()
            .with_strict()
            .with_coerce_all()
            .with_match_arg_optional("x".into())
            .with_match_arg_several_ok("x".into());
        let syn::FnArg::Typed(param) = parse_param(&format!("x: {ty}")) else {
            unreachable!()
        };
        assert_eq!(
            builder.native_borrow_metadata(&param, &arg).is_none(),
            specialized,
            "{ty}"
        );
    }
    // Coercion flags do not change borrowed / optional / boxed fallback conversions.
    for ty in [
        "&mut i32",
        "Option<&mut i32>",
        "Box<[&mut i32]>",
        "HiddenBorrow",
    ] {
        let builder = RustConversionBuilder::new().with_coerce_all();
        let syn::FnArg::Typed(param) = parse_param(&format!("x: {ty}")) else {
            unreachable!()
        };
        assert!(
            builder.native_borrow_metadata(&param, &arg).is_some(),
            "{ty}"
        );
    }
}
