use super::*;

#[test]
fn test_type_to_uppercase_name() {
    // Simple type
    let ty: syn::Type = syn::parse_quote!(MyType);
    assert_eq!(type_to_uppercase_name(&ty), "MYTYPE");

    // Path type
    let ty: syn::Type = syn::parse_quote!(path::to::MyType);
    assert_eq!(type_to_uppercase_name(&ty), "MYTYPE");
}

/// Regression test for #394: generic monomorphisations must not produce the same
/// vtable static name.  Before the fix, both `MyType<u32>` and `MyType<f64>`
/// resolved to `MYTYPE`, causing a silent collision (wrong vtable wins).
#[test]
fn test_type_to_uppercase_name_generic_distinct() {
    let ty_u32: syn::Type = syn::parse_quote!(MyType<u32>);
    let ty_f64: syn::Type = syn::parse_quote!(MyType<f64>);
    let ty_plain: syn::Type = syn::parse_quote!(MyType);

    let name_u32 = type_to_uppercase_name(&ty_u32);
    let name_f64 = type_to_uppercase_name(&ty_f64);
    let name_plain = type_to_uppercase_name(&ty_plain);

    // Both generic names must start with the base ident
    assert!(
        name_u32.starts_with("MYTYPE_"),
        "MyType<u32> should have a hash suffix, got: {name_u32}"
    );
    assert!(
        name_f64.starts_with("MYTYPE_"),
        "MyType<f64> should have a hash suffix, got: {name_f64}"
    );

    // The two monomorphisations must be distinct
    assert_ne!(
        name_u32, name_f64,
        "MyType<u32> and MyType<f64> must produce distinct vtable names"
    );

    // Non-generic plain type must NOT get a hash suffix (clean name for simple cases)
    assert_eq!(
        name_plain, "MYTYPE",
        "plain MyType should not have a hash suffix"
    );

    // Hash suffix must be 16 hex chars (FNV-1a-64 output)
    let suffix_u32 = name_u32.strip_prefix("MYTYPE_").unwrap();
    assert_eq!(
        suffix_u32.len(),
        16,
        "hash suffix must be 16 hex chars, got: {suffix_u32}"
    );
    assert!(
        suffix_u32.chars().all(|c| c.is_ascii_hexdigit()),
        "hash suffix must be lowercase hex, got: {suffix_u32}"
    );

    // Hashes must be deterministic: calling again must yield the same result
    let ty_u32_again: syn::Type = syn::parse_quote!(MyType<u32>);
    assert_eq!(
        type_to_uppercase_name(&ty_u32_again),
        name_u32,
        "type_to_uppercase_name must be deterministic across calls"
    );
}

/// Helper to build a simple TraitMethod for testing R wrapper generation.
fn make_test_method(name: &str, has_self: bool) -> TraitMethod {
    let ident = format_ident!("{}", name);
    let sig: syn::Signature = if has_self {
        syn::parse_quote!(fn #ident(&self) -> i32)
    } else {
        syn::parse_quote!(fn #ident() -> i32)
    };
    TraitMethod {
        ident,
        sig,
        has_self,
        is_mut: false,
        worker: false,
        unsafe_main_thread: false,
        coerce: false,
        check_interrupt: false,
        rng: false,
        unwrap_in_r: false,
        error_in_r: false,
        param_defaults: Default::default(),
        param_tags: vec![],
        skip: false,
        r_name: None,
        strict: false,
        lifecycle: None,
        r_entry: None,
        r_post_checks: None,
        r_on_exit: None,
    }
}

/// Helper to build TraitWrapperOpts for tests.
fn opts(
    class_system: ClassSystem,
    class_has_no_rd: bool,
    internal: bool,
    noexport: bool,
) -> TraitWrapperOpts {
    TraitWrapperOpts {
        class_system,
        class_has_no_rd,
        internal,
        noexport,
    }
}

// S3 generates @exportMethod, S4 generates @export + @exportMethod.
// Env does not generate @export at all (uses $ dispatch), so internal/noexport
// are no-ops for Env — we test with S3 and S4 instead.

#[test]
fn test_internal_suppresses_export_s3() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::S3, false, true, false),
    )
    .unwrap();

    assert!(
        result.contains("@keywords internal"),
        "internal should add @keywords internal for S3, got:\n{}",
        result
    );
    assert!(
        !result.contains("@export"),
        "internal should suppress @export for S3, got:\n{}",
        result
    );
}

#[test]
fn test_noexport_suppresses_export_s3() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::S3, false, false, true),
    )
    .unwrap();

    assert!(
        !result.contains("@keywords internal"),
        "noexport should NOT add @keywords internal for S3, got:\n{}",
        result
    );
    assert!(
        !result.contains("@export"),
        "noexport should suppress @export for S3, got:\n{}",
        result
    );
}

#[test]
fn test_internal_suppresses_export_s4() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::S4, false, true, false),
    )
    .unwrap();

    assert!(
        result.contains("@keywords internal"),
        "internal should add @keywords internal for S4, got:\n{}",
        result
    );
    assert!(
        !result.contains("@export"),
        "internal should suppress all @export/@exportMethod for S4, got:\n{}",
        result
    );
}

#[test]
fn test_noexport_suppresses_export_s4() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::S4, false, false, true),
    )
    .unwrap();

    assert!(
        !result.contains("@keywords internal"),
        "noexport should NOT add @keywords internal for S4, got:\n{}",
        result
    );
    assert!(
        !result.contains("@export"),
        "noexport should suppress @export for S4, got:\n{}",
        result
    );
}

#[test]
fn test_no_flags_preserves_export_s3() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::S3, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("@export"),
        "no flags should preserve @export, got:\n{}",
        result
    );
}

#[test]
fn test_nord_takes_precedence_over_internal_env() {
    // When @noRd is set, it strips all docs — internal/noexport don't matter
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::Env, true, true, false),
    )
    .unwrap();

    // @noRd strips all roxygen tags for non-S3 class systems
    assert!(
        !result.contains("#'"),
        "@noRd should strip all roxygen tags, got:\n{}",
        result
    );
}

#[test]
fn test_env_no_export_tags_even_without_flags() {
    // Env class system doesn't generate @export — internal/noexport are no-ops
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::Env, false, false, false),
    )
    .unwrap();

    // Env trait wrappers use $ dispatch, no @export tags
    assert!(
        !result.contains("@export"),
        "Env trait wrappers should not have @export, got:\n{}",
        result
    );
}

#[test]
fn test_internal_adds_keywords_internal_env() {
    // Even though Env has no @export, internal should add @keywords internal
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::Env, false, true, false),
    )
    .unwrap();

    assert!(
        result.contains("@keywords internal"),
        "internal should add @keywords internal even for Env, got:\n{}",
        result
    );
}
