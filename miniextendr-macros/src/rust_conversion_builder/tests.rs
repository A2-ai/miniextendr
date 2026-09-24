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

/// The `Err` arm names the parameter by its R formal (`_x` → `x`), renders the
/// type with source spacing, drops the old fixed "wrong type, length, or
/// contains NA" phrase, and passes an empty crate class list by default.
#[test]
fn test_conversion_err_arm_context_and_param() {
    let builder = RustConversionBuilder::new();
    let param = parse_param("_nums: AsFromStrVec<i32>");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        let s = stmts[0].to_string();
        assert!(s.contains("conversion_condition_value"), "{s}");
        assert!(s.contains("__mx_conversion_err_parts"), "{s}");
        assert!(
            s.contains("\"failed to convert parameter 'nums' to AsFromStrVec<i32>\""),
            "{s}"
        );
        assert!(s.contains("\"nums\" , & []"), "{s}");
        assert!(!s.contains("wrong type"), "{s}");
    }
}

/// Every conversion site binds a typed `let`, which the `Err` arm's probe
/// needs; borrowed slices and `&str` carry the lifetime-erased type.
#[test]
fn test_borrowed_bindings_are_typed_with_erased_lifetimes() {
    let builder = RustConversionBuilder::new();
    for (src, binding) in [
        ("x: &'a [f64]", "let x : & '_ [f64]"),
        ("s: &'a str", "let s : & '_ str"),
        ("x: &[f64]", "let x : & [f64]"),
    ] {
        let param = parse_param(src);
        let syn::FnArg::Typed(pat_type) = param else {
            unreachable!()
        };
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let s = builder.build_conversion(&pat_type, &sexp_ident)[0].to_string();
        assert!(s.contains(binding), "{src}: {s}");
    }
}

/// The crate-level `conversion_error_class` is emitted into every arm.
#[test]
fn test_conversion_err_arm_carries_the_crate_class() {
    let builder = RustConversionBuilder::new().with_conversion_error_class(vec![
        "pkg_error_argument".to_string(),
        "pkg_error".to_string(),
    ]);
    for src in ["x: i32", "x: &[f64]", "s: &str"] {
        let param = parse_param(src);
        let syn::FnArg::Typed(pat_type) = param else {
            unreachable!()
        };
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let s = builder.build_conversion(&pat_type, &sexp_ident)[0].to_string();
        assert!(
            s.contains("& [\"pkg_error_argument\" , \"pkg_error\"]"),
            "{src}: {s}"
        );
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
        assert!(stmts[0].to_string().contains("TryFromSexp"));
        assert!(stmts[0].to_string().contains("u16"));
    }
}
