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
        param_defaults: Default::default(),
        param_tags: vec![],
        skip: false,
        r_name: None,
        strict: false,
        lifecycle: None,
        r_entry: None,
        r_post_checks: None,
        r_on_exit: None,
        no_shortcut: false,
        per_param: Default::default(),
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

/// Trait-impl S7 instance methods get a `<ClassName>_<method>` fast-dispatch
/// shortcut alongside the `s7_trait_<Trait>_<method>` generic (#987).
#[test]
fn test_s7_trait_impl_emits_fast_path_shortcut() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let methods = vec![make_test_method("value", true)];

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &methods,
        &[],
        opts(ClassSystem::S7, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("Foo_value <- function(self, ...)"),
        "S7 trait impl should emit a Foo_value fast-path shortcut, got:\n{}",
        result
    );
    assert!(
        result.contains(".Call(C_Foo__Bar__value, .call = match.call(), self@.ptr)"),
        "shortcut should .Call through self@.ptr directly, got:\n{}",
        result
    );
    assert!(
        result.contains("Fast-path shortcut for the `value` S7 method on `Foo`"),
        "shortcut should carry the fast-path advisory doc, got:\n{}",
        result
    );
    // The generic + S7::method registration must still be present.
    assert!(
        result.contains("s7_trait_Bar_value"),
        "the dispatched generic must still exist, got:\n{}",
        result
    );
}

/// `s7(no_shortcut)` suppresses the trait-impl fast-dispatch shortcut while
/// keeping the S7 generic + method registration (#986).
#[test]
fn test_s7_trait_impl_no_shortcut_suppresses_shortcut() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("value", true);
    method.no_shortcut = true;

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S7, false, false, false),
    )
    .unwrap();

    assert!(
        !result.contains("Foo_value <- function"),
        "no_shortcut must suppress the Foo_value shortcut, got:\n{}",
        result
    );
    assert!(
        result.contains("s7_trait_Bar_value"),
        "the dispatched generic must still exist with no_shortcut, got:\n{}",
        result
    );
}

/// Void trait-impl shortcut methods chain via `invisible(self)`.
#[test]
fn test_s7_trait_impl_void_shortcut_returns_invisible_self() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let ident = format_ident!("bump");
    let sig: syn::Signature = syn::parse_quote!(fn bump(&mut self));
    let mut method = make_test_method("bump", true);
    method.ident = ident;
    method.sig = sig;
    method.is_mut = true;

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S7, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("Foo_bump <- function(self, ...)"),
        "void method should still get a shortcut, got:\n{}",
        result
    );
    assert!(
        result.contains("  invisible(self)"),
        "void shortcut should return invisible(self), got:\n{}",
        result
    );
}

// region: refactor/trait-method-emitter regression tests
//
// BUG1 + BUG2 from audit/2026-07-03-dogfooding-macros-codegen.md finding #1:
// the trait-impl R wrapper generators used to hand-roll receiver-ptr
// extraction via `call.replace(", x", ", .ptr")` (S4/S7/R6) and skipped the
// `precondition_checks` / `match_arg_prelude` steps that inherent methods
// get. `TraitMethodContext` (miniextendr_impl_trait/method_context.rs) fixes
// both by routing all 5 generators through the same `.Call()`/prelude
// builders the inherent-impl `MethodContext` uses.

/// BUG1 regression (S4 leg): `str::replace(", x", ", .ptr")` rewrites *every*
/// match of the substring `", x"`, so a parameter whose R name starts with
/// `x` (e.g. `x_factor`) used to be corrupted into `.ptr_factor` — a runtime
/// "object '.ptr_factor' not found" error. `TraitMethodContext::instance_call`
/// passes the receiver expression directly to `DotCallBuilder::with_self`
/// instead, so no other argument can ever be touched.
#[test]
fn test_bug1_x_prefixed_param_not_corrupted_s4() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("scale", true);
    method.sig = syn::parse_quote!(fn scale(&mut self, x_factor: f64));

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S4, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains(".Call(C_Foo__Bar__scale, .call = match.call(), .ptr, x_factor)"),
        "x_factor must reach the .Call() intact, got:\n{}",
        result
    );
    assert!(
        !result.contains(".ptr_factor"),
        "x_factor must NOT be corrupted into .ptr_factor, got:\n{}",
        result
    );
}

/// BUG1 regression (S7 leg) — covers both the dispatched-generic body and the
/// fast-path shortcut, which extract the receiver via `x@.ptr` / `self@.ptr`
/// respectively.
#[test]
fn test_bug1_x_prefixed_param_not_corrupted_s7() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("scale", true);
    method.sig = syn::parse_quote!(fn scale(&mut self, x_factor: f64));

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S7, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains(".Call(C_Foo__Bar__scale, .call = match.call(), .ptr, x_factor)"),
        "generic body: x_factor must reach the .Call() intact, got:\n{}",
        result
    );
    assert!(
        result.contains(".Call(C_Foo__Bar__scale, .call = match.call(), self@.ptr, x_factor)"),
        "shortcut: x_factor must reach the .Call() intact, got:\n{}",
        result
    );
    assert!(
        !result.contains(".ptr_factor"),
        "x_factor must NOT be corrupted into .ptr_factor, got:\n{}",
        result
    );
}

/// BUG1 regression (R6 leg).
#[test]
fn test_bug1_x_prefixed_param_not_corrupted_r6() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("scale", true);
    method.sig = syn::parse_quote!(fn scale(&mut self, x_factor: f64));

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::R6, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains(".Call(C_Foo__Bar__scale, .call = match.call(), .ptr, x_factor)"),
        "x_factor must reach the .Call() intact, got:\n{}",
        result
    );
    assert!(
        !result.contains(".ptr_factor"),
        "x_factor must NOT be corrupted into .ptr_factor, got:\n{}",
        result
    );
}

/// BUG2 regression: `trait_method_preamble_lines` (the pre-refactor prelude)
/// emitted only r_entry/on.exit/lifecycle/r_post_checks — it silently skipped
/// `precondition_checks`, so a trait method's typed params got no
/// `stopifnot()` validation an identical inherent method would have.
#[test]
fn test_bug2_precondition_checks_emitted_for_trait_method() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("bump", true);
    method.sig = syn::parse_quote!(fn bump(&mut self, amount: i32));

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S3, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("stopifnot("),
        "trait method with a typed param should emit stopifnot() preconditions, got:\n{}",
        result
    );
    assert!(
        result.contains("'amount' must be numeric"),
        "precondition message should mention the param, got:\n{}",
        result
    );
}

/// BUG2 regression: trait methods had no `match_arg`/`choices` attribute
/// support at all before this refactor (`TraitMethod` carried no per-param
/// map). `#[miniextendr(match_arg(mode))]` on a trait method now produces the
/// same `base::match.arg()` validation prelude an inherent method would.
#[test]
fn test_bug2_match_arg_prelude_emitted_for_trait_method() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("set_mode", true);
    method.sig = syn::parse_quote!(fn set_mode(&mut self, mode: String));
    method
        .per_param
        .entry("mode".to_string())
        .or_default()
        .match_arg = true;

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S3, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("mode <- base::match.arg(mode)"),
        "match_arg param should get a base::match.arg() prelude line, got:\n{}",
        result
    );
}

/// Related fix bundled into the same prelude parity: trait methods used to
/// build `.Call()` args via `collect_param_idents`, which had no `Missing<T>`
/// handling. A truly-missing R argument forwarded as a bare binding errors on
/// lookup (see PR #1129) — `TraitMethodContext` now builds args via
/// `build_r_call_args_from_sig`, which forwards `Missing<T>` as
/// `if (missing(p)) quote(expr=) else p` inline in the call.
#[test]
fn test_missing_type_forwarded_inline_in_trait_method_call() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("configure", true);
    method.sig = syn::parse_quote!(fn configure(&mut self, opt: Missing<i32>));

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::Env, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("if (missing(opt)) quote(expr=) else opt"),
        "Missing<T> param should forward the R_MissingArg sentinel inline, got:\n{}",
        result
    );
}
// endregion

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
