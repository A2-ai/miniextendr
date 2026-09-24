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
        invisible: None,
        has_self,
        is_mut: false,
        worker: false,
        unsafe_main_thread: false,
        coerce: false,
        check_interrupt: false,
        rng: false,
        unwrap_in_r: false,
        serialize: false,
        return_wrap: None,
        param_defaults: Default::default(),
        doc_tags: vec![],
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

/// Audit A10: `noexport` (without `internal`) must produce no Rd contribution
/// at all — same as a user-written `@noRd` — not just suppress `@export`.
/// Before the fix, `noexport` only stripped `@export`/`@exportMethod` lines,
/// leaving `@rdname`/`@name`/`@title` intact, so the trait's methods still
/// landed (undocumented-looking but alias-contributing) on the type's shared
/// help page.
#[test]
fn test_noexport_suppresses_all_roxygen_s4() {
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
        !result.contains("#'"),
        "noexport should strip ALL roxygen tags (no alias/rdname contribution) for S4, got:\n{}",
        result
    );
}

/// Companion to the above: `internal` must keep the opposite behavior — the
/// method stays documented (under `@keywords internal`), so it still
/// contributes to the type's shared `@rdname` help page.
#[test]
fn test_internal_still_contributes_rdname_s4() {
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
        result.contains("@rdname"),
        "internal should still contribute to the shared @rdname page for S4, got:\n{}",
        result
    );
}

/// S3/vctrs analog: `noexport` emits `@noRd` (like a user-written `@noRd`)
/// instead of merely dropping `@export`, and — unlike a plain user `@noRd` —
/// also drops the S3 dispatch-registration `@export` line (mirrors the
/// inherent-impl S3 generator's `should_register_s3method = !noexport`, #431:
/// `noexport` means zero observable NAMESPACE trace, not "undocumented but
/// still dispatchable").
#[test]
fn test_noexport_emits_no_rd_marker_s3() {
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
        result.contains("#' @noRd"),
        "noexport should emit @noRd for S3, got:\n{}",
        result
    );
    assert!(
        !result.contains("@export"),
        "noexport should also drop S3 dispatch @export for S3, got:\n{}",
        result
    );
}

/// `internal` must NOT emit `@noRd` — it stays documented (just unexported).
#[test]
fn test_internal_does_not_emit_no_rd_marker_s3() {
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
        !result.contains("@noRd"),
        "internal should NOT emit @noRd for S3 (stays documented), got:\n{}",
        result
    );
    assert!(
        result.contains("@rdname"),
        "internal should still contribute to the shared @rdname page for S3, got:\n{}",
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
        result.contains(
            ".Call(C_miniextendr_macros_Foo__Bar__value, .call = match.call(), self@.ptr)"
        ),
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

/// An operator `r_name` on a trait method (#1475): the prefixed S7 generic is
/// quoted in symbol position and gets no `Foo_[[` shortcut, and static methods
/// quote their namespace member in every class system.
#[test]
fn test_trait_operator_r_names_are_quoted() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    for generic in ["[", "[[", "$", "%custom%"] {
        let mut method = make_test_method("value", true);
        method.r_name = Some(generic.to_string());
        let wrapper = generate_trait_r_wrapper(
            &type_ident,
            &trait_name,
            &[method],
            &[],
            opts(ClassSystem::S7, false, false, false),
        )
        .unwrap();
        let generic_name = format!("s7_trait_Bar_{generic}");
        for expected in [
            format!("if (!exists(\"{generic_name}\", mode = \"function\")) {{"),
            format!(
                "  `{generic_name}` <- S7::new_generic(\"{generic_name}\", \"x\", function(x, ...) S7::S7_dispatch())"
            ),
            format!("S7::method(`{generic_name}`, .s7_class_Foo) <- function(x, ...) {{"),
        ] {
            assert!(
                wrapper.contains(&expected),
                "missing `{expected}`:\n{wrapper}"
            );
        }
        assert!(!wrapper.contains(&format!("Foo_{generic}")), "{wrapper}");
        assert!(!wrapper.contains("Fast-path shortcut"), "{wrapper}");
    }

    for (class_system, definition) in [
        (ClassSystem::Env, "Foo$Bar$`[[` <- function() {"),
        (ClassSystem::S3, "Foo$Bar$`[[` <- function() {"),
        (ClassSystem::R6, "Foo$Bar$`[[` <- function() {"),
        (ClassSystem::S4, "`Foo_Bar_[[` <- function() {"),
        (ClassSystem::S7, ".Foo__Bar$`[[` <- function() {"),
    ] {
        let mut method = make_test_method("make", false);
        method.r_name = Some("[[".to_string());
        let wrapper = generate_trait_r_wrapper(
            &type_ident,
            &trait_name,
            &[method],
            &[],
            opts(class_system, false, false, false),
        )
        .unwrap();
        assert!(wrapper.contains(definition), "{class_system:?}:\n{wrapper}");
    }

    // Env / R6 instance members are namespace assignments too.
    for class_system in [ClassSystem::Env, ClassSystem::R6] {
        let mut method = make_test_method("value", true);
        method.r_name = Some("[[".to_string());
        let wrapper = generate_trait_r_wrapper(
            &type_ident,
            &trait_name,
            &[method],
            &[],
            opts(class_system, false, false, false),
        )
        .unwrap();
        assert!(
            wrapper.contains("Foo$Bar$`[[` <- function(x) {"),
            "{class_system:?}:\n{wrapper}"
        );
    }
}

/// Void trait-impl shortcut methods chain via `self`, visibly (#1213); an
/// `Invisible<()>` return marks the tail `invisible(self)`.
#[test]
fn test_s7_trait_impl_void_shortcut_returns_self() {
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
        result.contains("  self\n"),
        "void shortcut should return `self`, got:\n{}",
        result
    );
    assert!(
        !result.contains("invisible("),
        "unmarked void shortcut must be visible (#1213), got:\n{}",
        result
    );

    // `-> Invisible<()>` (already peeled at parse time; `invisible` records it).
    let mut quiet = make_test_method("hush", true);
    quiet.sig = syn::parse_quote!(fn hush(&mut self));
    quiet.is_mut = true;
    quiet.invisible = Some(true);
    // `-> Invisible<i32>`: the converted value is returned invisibly.
    let mut peek = make_test_method("peek", true);
    peek.invisible = Some(true);
    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[quiet, peek],
        &[],
        opts(ClassSystem::S7, false, false, false),
    )
    .unwrap();
    assert!(
        result.contains("  invisible(self)"),
        "marked void shortcut should return invisible(self), got:\n{}",
        result
    );
    assert!(
        result.contains("invisible(x)"),
        "marked void generic method should return invisible(x), got:\n{}",
        result
    );
    assert!(
        result.contains("invisible(.val)"),
        "marked value method should return invisible(.val), got:\n{}",
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
        result.contains(
            ".Call(C_miniextendr_macros_Foo__Bar__scale, .call = match.call(), .ptr, x_factor)"
        ),
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
        result.contains(
            ".Call(C_miniextendr_macros_Foo__Bar__scale, .call = match.call(), .ptr, x_factor)"
        ),
        "generic body: x_factor must reach the .Call() intact, got:\n{}",
        result
    );
    assert!(
        result.contains(
            ".Call(C_miniextendr_macros_Foo__Bar__scale, .call = match.call(), self@.ptr, x_factor)"
        ),
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
        result.contains(
            ".Call(C_miniextendr_macros_Foo__Bar__scale, .call = match.call(), .ptr, x_factor)"
        ),
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
/// precondition validation an identical inherent method would have.
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
        result.contains("if (!isTRUE(is.integer(amount)))"),
        "trait method with a typed param should emit precondition guards, got:\n{}",
        result
    );
    assert!(
        result.contains(".miniextendr_arg_error(\"amount\", \"must be integer\")"),
        "precondition message should mention the param, got:\n{}",
        result
    );
}

/// BUG2 regression: trait methods had no `match_arg`/`choices` attribute
/// support at all before this refactor (`TraitMethod` carried no per-param
/// map). This exercises the shared `TraitMethodContext::match_arg_prelude` /
/// `build_match_arg_prelude` primitive directly via `per_param` (bypassing
/// attribute parsing) — `#[miniextendr(match_arg(...))]` itself is not yet
/// exposed as a trait-method attribute because, unlike `choices(...)`, it
/// needs an enum's derived `MatchArg::CHOICES` resolved through a C-wrapper
/// helper the inherent-impl path has
/// (`generate_method_match_arg_helpers` in `miniextendr_impl.rs`) and the
/// trait path doesn't yet; see the tracked follow-up issue. `choices(...)`
/// (no derivation needed) *is* exposed and covered by
/// `test_bug2_choices_prelude_emitted_for_trait_method` below.
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
        result.contains("mode <- .miniextendr_match_arg(mode, "),
        "match_arg param should get a .miniextendr_match_arg() prelude line, got:\n{}",
        result
    );
}

/// BUG2 regression, exercised via the actual macro-exposed surface:
/// `#[miniextendr(choices(mode = "fast, slow"))]` on a trait method now
/// produces both the `c("fast", "slow")` formal default (via the shared
/// `effective_r_defaults`, also required for `match.arg()` to find its
/// choice list — see its docs) and the `match.arg()` prelude line. Verified
/// end-to-end (not just codegen strings) by
/// `rpkg/tests/testthat/test-trait-method-emitter.R`.
#[test]
fn test_bug2_choices_prelude_emitted_for_trait_method() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("set_mode", true);
    method.sig = syn::parse_quote!(fn set_mode(&mut self, mode: String));
    method
        .per_param
        .entry("mode".to_string())
        .or_default()
        .choices = Some(vec!["fast".to_string(), "slow".to_string()]);

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S3, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("mode = c(\"fast\", \"slow\")"),
        "choices param should get a formal default match.arg() can read, got:\n{}",
        result
    );
    assert!(
        result.contains("mode <- .miniextendr_match_arg(mode, c(\"fast\", \"slow\"), \"mode\")"),
        "choices param should get a .miniextendr_match_arg() prelude line, got:\n{}",
        result
    );
}

/// Trait methods parse the method-level `inherits(p(class = ..., message = ...))`
/// / `no_na(p(message = ...))` forms with the inherent-impl parser's helpers,
/// from the attribute through to the guard.
#[test]
fn test_trait_method_checks_take_custom_messages() {
    let impl_item: syn::ItemImpl = syn::parse_quote! {
        impl Bar for Foo {
            #[miniextendr(
                no_na(x_factor(message = "`x_factor` must be a number, not NA")),
                inherits(model(class = "pkg_model", message = "`model` must be a `pkg_model`"))
            )]
            fn scale(&mut self, x_factor: f64, model: List) -> f64 { unimplemented!() }
        }
    };
    let methods = super::vtable::extract_methods(&impl_item).unwrap();
    let result = generate_trait_r_wrapper(
        &format_ident!("Foo"),
        &format_ident!("Bar"),
        &methods,
        &[],
        opts(ClassSystem::S3, false, false, false),
    )
    .unwrap();
    for guard in [
        "if (!isTRUE(!anyNA(x_factor))) .miniextendr_arg_error(\"x_factor\", message = \"`x_factor` must be a number, not NA\")",
        "if (!isTRUE(inherits(model, \"pkg_model\"))) .miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model`\")",
    ] {
        assert!(result.contains(guard), "missing `{guard}` in:\n{result}");
    }

    let impl_item: syn::ItemImpl = syn::parse_quote! {
        impl Bar for Foo {
            #[miniextendr(inherits(model(message = "m")))]
            fn scale(&mut self, model: List) -> f64 { unimplemented!() }
        }
    };
    let Err(err) = super::vtable::extract_methods(&impl_item) else {
        panic!("a class check without a class must be rejected");
    };
    assert!(
        err.to_string()
            .contains("`inherits(model(...))` needs `class = \"...\"`"),
        "{err}"
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

/// A trait method with an omittable `choices(...)` parameter and a plain one.
fn choice_trait_method() -> TraitMethod {
    let mut method = make_test_method("pick", true);
    method.sig = syn::parse_quote!(
        fn pick(&self, level: Missing<Option<String>>, n: i32) -> String
    );
    let mut attrs = crate::miniextendr_fn::ParamAttrs {
        choices: Some(vec!["low".to_string(), "high".to_string()]),
        ..Default::default()
    };
    let ty: syn::Type = syn::parse_quote!(Missing<Option<String>>);
    crate::miniextendr_fn::classify_choice_param(&mut attrs, "level", &ty, false).unwrap();
    method.per_param.insert("level".to_string(), attrs);
    method
}

const LEVEL_DOC: &str =
    "#' @param level One of \"low\", \"high\", or NULL; omitting the argument means no choice.";

/// The S3 method block of a trait method documents every formal of its
/// `\usage` the doc comment leaves out, a `choices(...)` param with the same
/// text a standalone function gets, and the generic is exported by name
/// (roxygen2 cannot see it inside the `exists()` guard, so a bare `@export`
/// would export the method `pick.Foo` as a plain function instead).
#[test]
fn s3_trait_method_documents_params_and_exports_generic_by_name() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[choice_trait_method()],
        &[],
        opts(ClassSystem::S3, false, false, false),
    )
    .unwrap();
    assert!(result.contains(LEVEL_DOC), "got:\n{result}");
    assert!(
        result.contains("#' @param n (undocumented)"),
        "got:\n{result}"
    );
    assert!(result.contains("#' @export pick\n"), "got:\n{result}");

    // A method-level `@rdname` naming another topic leaves the arguments to
    // that topic's block (#1590).
    let mut split = choice_trait_method();
    split.doc_tags.push("@rdname pick_family".to_string());
    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[split],
        &[],
        opts(ClassSystem::S3, false, false, false),
    )
    .unwrap();
    assert!(!result.contains("@param level"), "got:\n{result}");
    assert!(!result.contains("@param n "), "got:\n{result}");
}

/// The S7 fast-path shortcut documents a `choices(...)` param with its choice
/// text instead of `(undocumented)`.
#[test]
fn s7_trait_shortcut_documents_choice_params() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[choice_trait_method()],
        &[],
        opts(ClassSystem::S7, false, false, false),
    )
    .unwrap();
    assert!(result.contains(LEVEL_DOC), "got:\n{result}");
    assert!(
        result.contains("#' @param n (undocumented)"),
        "got:\n{result}"
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

// region: #1141 / #1115 — namespace-shape unification + Self-return re-wrapping

/// #1141 / #1115: R6 instance trait methods now live in the class-scoped
/// `Type$Trait$method` namespace (env-style) instead of a standalone,
/// unqualified `r6_trait_<Trait>_<method>(x)` function.
#[test]
fn test_r6_instance_method_uses_class_qualified_namespace() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let method = make_test_method("value", true);

    let result = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::R6, false, false, false),
    )
    .unwrap();

    assert!(
        result.contains("Foo$Bar$value <- function(x)"),
        "R6 instance method should use the class-scoped Foo$Bar$value shape, got:\n{}",
        result
    );
    assert!(
        result.contains("  .ptr <- x$.__enclos_env__$private$.ptr"),
        "R6 instance method should still extract the receiver ptr from the R6 object, got:\n{}",
        result
    );
    assert!(
        !result.contains("r6_trait_"),
        "the unqualified r6_trait_* standalone name must be gone (#1115), got:\n{}",
        result
    );
}

/// #1115 regression at the codegen level: two R6 impls of one trait on
/// different types produce distinct, class-qualified wrapper targets rather
/// than a shared `r6_trait_<Trait>_<method>` name that aborted wrapper-gen via
/// the duplicate-definition guard. (The end-to-end install-cleanly regression
/// is `rpkg/tests/testthat/test-trait-r6-collision.R`.)
#[test]
fn test_two_r6_impls_of_one_trait_are_distinct() {
    let trait_name = format_ident!("Bar");
    let a = generate_trait_r_wrapper(
        &format_ident!("Foo"),
        &trait_name,
        &[make_test_method("value", true)],
        &[],
        opts(ClassSystem::R6, false, false, false),
    )
    .unwrap();
    let b = generate_trait_r_wrapper(
        &format_ident!("Baz"),
        &trait_name,
        &[make_test_method("value", true)],
        &[],
        opts(ClassSystem::R6, false, false, false),
    )
    .unwrap();

    assert!(a.contains("Foo$Bar$value <- function"), "got:\n{}", a);
    assert!(b.contains("Baz$Bar$value <- function"), "got:\n{}", b);
    assert!(
        !a.contains("r6_trait_") && !b.contains("r6_trait_"),
        "pre-fix both emitted the colliding r6_trait_Bar_value; must be class-qualified now"
    );
}

/// #1141 resp-4: a `-> Self` instance trait method re-wraps the bare
/// `ExternalPtr` `.val` into a classed object, using each class system's
/// constructor idiom — the same shared `MethodReturnBuilder` the inherent-impl
/// generators use. Before this, trait Self-returns leaked a bare, unclassed
/// pointer to the caller.
#[test]
fn test_self_return_rewrapped_per_class_system() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("dup", true);
    method.sig = syn::parse_quote!(fn dup(&self) -> Self);

    let emit = |cs| {
        generate_trait_r_wrapper(
            &type_ident,
            &trait_name,
            &[method.clone()],
            &[],
            opts(cs, false, false, false),
        )
        .unwrap()
    };

    let env = emit(ClassSystem::Env);
    assert!(
        env.contains("class(.val) <- \"Foo\""),
        "env -> Self should stamp the class attribute, got:\n{env}"
    );

    let s3 = emit(ClassSystem::S3);
    assert!(
        s3.contains("structure(.val, class = \"Foo\")"),
        "s3 -> Self should wrap via structure(), got:\n{s3}"
    );

    let s4 = emit(ClassSystem::S4);
    assert!(
        s4.contains("methods::new(\"Foo\", ptr = .val)"),
        "s4 -> Self should wrap via methods::new(), got:\n{s4}"
    );

    let s7 = emit(ClassSystem::S7);
    assert!(
        s7.contains("Foo(.ptr = .val)"),
        "s7 -> Self should wrap via the S7 constructor, got:\n{s7}"
    );

    let r6 = emit(ClassSystem::R6);
    assert!(
        r6.contains("Foo$new(.ptr = .val)"),
        "r6 -> Self should wrap via ClassName$new(.ptr = ), got:\n{r6}"
    );
}

/// A `-> Self` *static* trait factory method (e.g. `default()`) re-wraps too,
/// not just instance methods.
#[test]
fn test_static_self_return_rewrapped() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("make", false);
    method.sig = syn::parse_quote!(fn make() -> Self);

    let s4 = generate_trait_r_wrapper(
        &type_ident,
        &trait_name,
        &[method],
        &[],
        opts(ClassSystem::S4, false, false, false),
    )
    .unwrap();

    assert!(
        s4.contains("methods::new(\"Foo\", ptr = .val)"),
        "static -> Self should re-wrap via methods::new(), got:\n{s4}"
    );
}
// endregion

/// A method-level `/// @rdname other` on a trait-impl method moves that
/// method's wrapper block onto the requested page on every class system, while
/// the S4/S7 generics and the type's shared page stay put (#1438). The S3
/// generic guard is documented under the method's own `value.Foo` name, so it
/// moves with the method (otherwise that alias lands on two pages).
#[test]
fn test_trait_method_rdname_override_all_systems() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut inst = make_test_method("value", true);
    inst.doc_tags = vec!["@rdname foo_value".to_string()];
    let mut stat = make_test_method("make", false);
    stat.doc_tags = vec!["@rdname foo_make".to_string()];
    let methods = vec![inst, stat];

    for class_system in [
        ClassSystem::Env,
        ClassSystem::R6,
        ClassSystem::S3,
        ClassSystem::S4,
        ClassSystem::S7,
    ] {
        let result = generate_trait_r_wrapper(
            &type_ident,
            &trait_name,
            &methods,
            &[],
            opts(class_system, false, false, false),
        )
        .unwrap();

        // S7 registers the instance method by assignment with no roxygen of
        // its own (the generic block documents it on the type page); the
        // override lands on the per-class shortcut instead. S3 emits two
        // blocks (generic guard named `value.Foo` + method) and both move.
        // Every other system documents the method block directly.
        let expected_value = if matches!(class_system, ClassSystem::S3) {
            2
        } else {
            1
        };
        assert_eq!(
            result.matches("#' @rdname foo_value").count(),
            expected_value,
            "{class_system:?}: instance method page, got:\n{result}"
        );
        assert_eq!(
            result.matches("#' @rdname foo_make").count(),
            1,
            "{class_system:?}: static method page, got:\n{result}"
        );
        // Prose-less blocks that were split off need a structural @title;
        // static blocks already open with a prose line (implicit title).
        if !matches!(class_system, ClassSystem::S7) {
            assert!(
                result.contains("#' @title "),
                "{class_system:?}: split instance-method block needs a @title, got:\n{result}"
            );
        }
        if matches!(class_system, ClassSystem::S4 | ClassSystem::S7) {
            assert!(
                result.contains("#' @rdname Foo"),
                "{class_system:?}: the generic stays on the type page, got:\n{result}"
            );
        }
        if matches!(class_system, ClassSystem::S3) {
            // The guard block's alias is `value.Foo` — the method's own — so it
            // must not stay behind on the type page (duplicated alias across
            // two Rd files), and it drops its filler title on the split page.
            let guard = result
                .find("#' @name value.Foo")
                .unwrap_or_else(|| panic!("missing S3 guard block in:\n{result}"));
            let guard_rdname = result[guard..]
                .lines()
                .find_map(|l| l.strip_prefix("#' @rdname "))
                .unwrap_or_else(|| panic!("no @rdname after the guard in:\n{result}"));
            assert_eq!(
                guard_rdname, "foo_value",
                "S3 generic guard must follow the split method, got:\n{result}"
            );
            assert!(
                !result.contains("#' @title S3 generic for `value`"),
                "split S3 guard must not carry the filler title, got:\n{result}"
            );
            // The author's topic describes the method and documents `x` and
            // `...` itself (#1590).
            assert!(
                !result.contains("#' @param x ") && !result.contains("#' @param ... "),
                "split S3 blocks must leave `x` / `...` to the topic, got:\n{result}"
            );
            assert!(
                !result.contains("#' @description S3 generic for `value`"),
                "split S3 guard must not add a structural description, got:\n{result}"
            );
        }
        // Every block on an author topic sorts after the topic's own block
        // (#1590), and only those: one per `@rdname foo_*` line.
        assert_eq!(
            result
                .matches(&format!("#' {}", crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS))
                .count(),
            expected_value + 1,
            "{class_system:?}: got:\n{result}"
        );
    }
}

/// The S7 trait shortcut documents each formal. On the type page an
/// undocumented one gets the `(undocumented)` filler; a method-level
/// `@rdname` sends the shortcut to a topic that documents its arguments, so
/// the filler is left out there (#1590). The method's own `@param` is kept.
#[test]
fn test_s7_trait_shortcut_param_filler_follows_method_page() {
    let type_ident = format_ident!("Foo");
    let trait_name = format_ident!("Bar");
    let mut method = make_test_method("step", true);
    method.sig = syn::parse_quote!(fn step(&self, by: i32, times: i32) -> i32);
    let by_doc = "@param by Step size.".to_string();
    method.doc_tags = vec![by_doc.clone()];
    let generate = |method: &TraitMethod| {
        generate_trait_r_wrapper(
            &type_ident,
            &trait_name,
            std::slice::from_ref(method),
            &[],
            opts(ClassSystem::S7, false, false, false),
        )
        .unwrap()
    };

    // No method-level `@rdname`, or a redundant one naming the type page.
    for rdname in [None, Some("Foo")] {
        method.doc_tags = std::iter::once(by_doc.clone())
            .chain(rdname.map(|topic| format!("@rdname {topic}")))
            .collect();
        let type_page = generate(&method);
        assert!(
            type_page.contains("#' @param times (undocumented)"),
            "{rdname:?}: got:\n{type_page}"
        );
    }

    method.doc_tags = vec![by_doc.clone(), "@rdname foo_steps".to_string()];
    let split = generate(&method);
    assert!(!split.contains("#' @param times "), "got:\n{split}");
    // The structural `self` / `...` lines are left to the topic too (#1590).
    assert!(!split.contains("#' @param self "), "got:\n{split}");
    assert!(!split.contains("#' @param ... "), "got:\n{split}");
    assert_eq!(
        split.matches("#' @param by Step size.").count(),
        1,
        "got:\n{split}"
    );
}

#[test]
fn explicit_cross_class_returns_work_in_every_trait_wrapper_generator() {
    for system in [
        ClassSystem::Env,
        ClassSystem::R6,
        ClassSystem::S3,
        ClassSystem::S4,
        ClassSystem::S7,
        ClassSystem::Vctrs,
    ] {
        let mut instance = make_test_method("build", true);
        instance.return_wrap =
            crate::return_wrap::resolve(&syn::parse_quote!(-> WrapAsR6<Board>), None)
                .unwrap()
                .0;
        let mut factory = make_test_method("create", false);
        factory.return_wrap = instance.return_wrap.clone();
        let wrapper = generate_trait_r_wrapper(
            &format_ident!("Factory"),
            &format_ident!("Builder"),
            &[instance, factory],
            &[],
            opts(system, false, false, false),
        )
        .unwrap();
        assert!(
            wrapper.matches("Board$new(.ptr = .val)").count() >= 2,
            "{system:?}: {wrapper}"
        );
    }
}

#[test]
fn s3_operator_names_are_quoted_in_trait_wrappers() {
    for generic in ["[", "[[", "$", "==", "+", "%custom%"] {
        let mut method = make_test_method("operator", true);
        method.r_name = Some(generic.to_string());
        let wrapper = generate_trait_r_wrapper(
            &format_ident!("Foo"),
            &format_ident!("Operators"),
            &[method],
            &[],
            opts(ClassSystem::S3, false, false, false),
        )
        .unwrap();
        assert!(
            wrapper.contains(&format!("`{generic}.Foo` <- function(x, ...)")),
            "{wrapper}"
        );
        assert!(
            wrapper.contains(&format!(
                "`{generic}` <- function(x, ...) UseMethod(\"{generic}\")"
            )),
            "{wrapper}"
        );
        assert!(
            wrapper.contains(&format!(
                "S7::method(`{generic}`, S7::new_S3_class(\"Foo\")) <- `{generic}.Foo`"
            )),
            "{wrapper}"
        );
    }
}

// region: author page tags on trait-impl methods (#1590)

/// Generate one class system's trait wrapper for `methods` on `Foo: Bar`.
fn page_tag_wrapper(class_system: ClassSystem, methods: &[TraitMethod]) -> syn::Result<String> {
    generate_trait_r_wrapper(
        &format_ident!("Foo"),
        &format_ident!("Bar"),
        methods,
        &[],
        opts(class_system, false, false, false),
    )
}

/// The roxygen block (the `#'` lines) directly above the first line of `r`
/// that starts with `anchor`.
fn block_above<'a>(r: &'a str, anchor: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = r.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with(anchor))
        .unwrap_or_else(|| panic!("no line starting with {anchor:?} in:\n{r}"));
    let start = lines[..at]
        .iter()
        .rposition(|l| !l.starts_with("#'"))
        .map_or(0, |i| i + 1);
    lines[start..at].to_vec()
}

/// A method-level `@describeIn` lists the method's own block in the
/// destination's "Functions" section wherever that block documents an R
/// function or method: S3 / vctrs instance methods (`generic.Foo`), S4
/// instance methods (`setMethod()`), S4 statics (`Foo_Bar_make`) and the S7
/// shortcut (`Foo_value`). The block then carries the author's (wrapped) tag
/// but no generated `@name` / `@rdname` / structural title, which roxygen2
/// rejects next to it, sorts after the topic's block, and leaves `x` / `...`
/// / `self` to the topic.
#[test]
fn test_trait_method_describe_in_routes_own_block() {
    let describe_in = "@describeIn family Value of a\nfoo.".to_string();
    let mut inst = make_test_method("value", true);
    inst.doc_tags = vec![describe_in.clone()];
    let mut stat = make_test_method("make", false);
    stat.doc_tags = vec![describe_in.clone()];
    let order = format!("#' {}", crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS);

    // (class system, methods, anchor line of each method's own block)
    let cases: Vec<(ClassSystem, Vec<TraitMethod>, Vec<&str>)> = vec![
        (ClassSystem::S3, vec![inst.clone()], vec!["value.Foo <- "]),
        (
            ClassSystem::Vctrs,
            vec![inst.clone()],
            vec!["value.Foo <- "],
        ),
        (
            ClassSystem::S4,
            vec![inst.clone(), stat.clone()],
            vec![
                "methods::setMethod(\"s4_trait_Bar_value\"",
                "Foo_Bar_make <- ",
            ],
        ),
        (ClassSystem::S7, vec![inst.clone()], vec!["Foo_value <- "]),
    ];
    for (class_system, methods, anchors) in cases {
        let r = page_tag_wrapper(class_system, &methods).unwrap();
        for anchor in anchors {
            let block = block_above(&r, anchor);
            let at = block
                .iter()
                .position(|l| *l == "#' @describeIn family Value of a")
                .unwrap_or_else(|| panic!("{class_system:?} {anchor}: no @describeIn in:\n{r}"));
            assert_eq!(
                block[at + 1],
                "#' foo.",
                "{class_system:?}: wrapped tag, got:\n{r}"
            );
            for tag in ["#' @name ", "#' @rdname ", "#' @title ", "#' @param "] {
                assert!(
                    !block.iter().any(|l| l.starts_with(tag)),
                    "{class_system:?} {anchor}: `{tag}` next to @describeIn, got:\n{r}"
                );
            }
            assert!(
                block.contains(&order.as_str()),
                "{class_system:?}: got:\n{r}"
            );
            // No prose of the block's own (an intro line or the S7 advisory's
            // description) that would join the destination's description or
            // title it.
            let prose = block
                .iter()
                .filter(|l| !l.starts_with("#' @") && **l != "#' foo.")
                .count();
            // S7 keeps the advisory's first line: the block's title, inert on
            // the destination (the topic's block comes first).
            let expected_prose = usize::from(matches!(class_system, ClassSystem::S7));
            assert_eq!(prose, expected_prose, "{class_system:?}: got:\n{r}");
        }
        if matches!(class_system, ClassSystem::S3 | ClassSystem::Vctrs) {
            // The generic block is named after the S3 method (`value.Foo`, its
            // alias), so it follows the method onto the destination page.
            let generic = block_above(&r, "if (!base::exists(\"value\"");
            assert!(generic.contains(&"#' @rdname family"), "got:\n{r}");
            assert!(generic.contains(&order.as_str()), "got:\n{r}");
            assert!(
                !generic
                    .iter()
                    .any(|l| l.starts_with("#' @title ") || l.starts_with("#' @param ")),
                "got:\n{r}"
            );
            assert!(!r.contains("S3 generic for `value`"), "got:\n{r}");
        }
        if matches!(class_system, ClassSystem::S4 | ClassSystem::S7) {
            // The S4 / S7 generic stays on the type page.
            assert!(r.contains("#' @rdname Foo"), "{class_system:?}: got:\n{r}");
        }
    }
}

/// Where the method's own block is a `Type$Trait$method` namespace member
/// (every static method except S4's, and Env / R6 instance methods), or an S7
/// instance method has no shortcut, roxygen2 has no object to list, so
/// `@describeIn` is a compile error naming `@rdname`.
#[test]
fn test_trait_method_describe_in_rejected_without_listable_object() {
    let with_describe_in = |mut method: TraitMethod| {
        method.doc_tags = vec!["@describeIn family A method.".to_string()];
        method
    };
    let inst = with_describe_in(make_test_method("value", true));
    let stat = with_describe_in(make_test_method("make", false));
    let mut no_shortcut = inst.clone();
    no_shortcut.no_shortcut = true;
    let cases = [
        (
            ClassSystem::Env,
            &inst,
            "Env trait instance method `value`:",
        ),
        (ClassSystem::Env, &stat, "Env trait static method `make`:"),
        (ClassSystem::R6, &inst, "R6 trait instance method `value`:"),
        (ClassSystem::R6, &stat, "R6 trait static method `make`:"),
        (ClassSystem::S3, &stat, "S3 trait static method `make`:"),
        (
            ClassSystem::Vctrs,
            &stat,
            "vctrs trait static method `make`:",
        ),
        (ClassSystem::S7, &stat, "S7 trait static method `make`:"),
        (
            ClassSystem::S7,
            &no_shortcut,
            "S7 trait instance method `value` without a fast-path shortcut:",
        ),
    ];
    for (class_system, method, kind) in cases {
        let err = page_tag_wrapper(class_system, std::slice::from_ref(method))
            .expect_err(&format!("{class_system:?}: {kind} must be rejected"))
            .to_string();
        assert!(
            err.contains(&format!("`@describeIn` is not supported on the {kind}")),
            "{class_system:?}: got {err}"
        );
        assert!(err.contains("Use `@rdname <topic>`"), "got {err}");
    }
    // Every rejected method is reported at once.
    let err = page_tag_wrapper(ClassSystem::Env, &[inst.clone(), stat.clone()]).unwrap_err();
    assert_eq!(err.into_iter().count(), 2);
    // `@rdname` stays available everywhere.
    let mut rdname = make_test_method("make", false);
    rdname.doc_tags = vec!["@rdname family".to_string()];
    for class_system in [
        ClassSystem::Env,
        ClassSystem::R6,
        ClassSystem::S3,
        ClassSystem::S4,
        ClassSystem::S7,
        ClassSystem::Vctrs,
    ] {
        page_tag_wrapper(class_system, std::slice::from_ref(&rdname)).unwrap();
    }
}

/// The author's `@name`, `@order` and parameter-inheritance tags reach the
/// method's own block: `@name` replaces the generated topic name, an author
/// `@order` replaces `@order NaN`, and `@inheritParams` / `@inherit` /
/// `@inheritDotParams` are forwarded verbatim (roxygen2 fills the arguments
/// the page leaves undocumented). Prose is not forwarded.
#[test]
fn test_trait_method_forwards_author_page_tags() {
    let mut method = make_test_method("make", false);
    method.doc_tags = vec![
        "@description Dropped: trait wrappers forward no prose.".to_string(),
        "@name foo_make".to_string(),
        "@rdname family".to_string(),
        "@order 2".to_string(),
        "@inheritParams family".to_string(),
        "@inherit other return".to_string(),
        "@inheritDotParams base::paste".to_string(),
    ];
    for (class_system, generated_name) in [
        (ClassSystem::Env, "Foo$Bar$make"),
        (ClassSystem::R6, "Foo$Bar$make"),
        (ClassSystem::S3, "Foo$Bar$make"),
        (ClassSystem::S4, "Foo_Bar_make"),
        (ClassSystem::S7, "Foo$Bar$make"),
    ] {
        let r = page_tag_wrapper(class_system, std::slice::from_ref(&method)).unwrap();
        for tag in &method.doc_tags[1..] {
            assert_eq!(
                r.matches(&format!("#' {tag}\n")).count(),
                1,
                "{class_system:?}: `{tag}` once, got:\n{r}"
            );
        }
        assert!(
            !r.contains(&format!("#' @name {generated_name}\n")),
            "got:\n{r}"
        );
        assert!(!r.contains("Dropped"), "got:\n{r}");
        assert!(
            !r.contains(crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS),
            "{class_system:?}: the author's @order wins, got:\n{r}"
        );
    }
}

/// On the type page an S7 shortcut documents each formal with a filler; a
/// method that inherits its arguments gets no `(undocumented)` filler (it
/// would block the inheritance) but keeps the structural `self` / `...`
/// lines, since it stays on the type page (#1590).
#[test]
fn test_s7_trait_shortcut_inherit_params_drops_fillers_only() {
    let mut method = make_test_method("step", true);
    method.sig = syn::parse_quote!(fn step(&self, by: i32) -> i32);
    method.doc_tags = vec!["@inheritParams family".to_string()];
    let r = page_tag_wrapper(ClassSystem::S7, &[method]).unwrap();
    let block = block_above(&r, "Foo_step <- ");
    assert!(!r.contains("#' @param by "), "got:\n{r}");
    assert!(
        block.contains(&"#' @param self A `Foo` object."),
        "got:\n{r}"
    );
    assert!(
        block.iter().any(|l| l.starts_with("#' @param ... ")),
        "got:\n{r}"
    );
    assert!(block.contains(&"#' @inheritParams family"), "got:\n{r}");
    assert!(block.contains(&"#' @rdname Foo"), "got:\n{r}");
    assert!(
        !r.contains(crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS),
        "the type page is no author topic, got:\n{r}"
    );
}

/// A wrapped `@param` keeps its continuation line inside the roxygen block on
/// every block that carries the method's parameters (a bare continuation
/// line would be R code).
#[test]
fn test_trait_method_wrapped_param_stays_in_roxygen() {
    let mut method = make_test_method("add", true);
    method.sig = syn::parse_quote!(fn add(&mut self, n: i32) -> i32);
    method.doc_tags = vec!["@param n Amount\nto add.".to_string()];
    for (class_system, blocks) in [
        (ClassSystem::S3, 1),
        (ClassSystem::Vctrs, 1),
        (ClassSystem::S4, 2),
        (ClassSystem::S7, 1),
    ] {
        let r = page_tag_wrapper(class_system, std::slice::from_ref(&method)).unwrap();
        assert_eq!(
            r.matches("#' @param n Amount\n#' to add.\n").count(),
            blocks,
            "{class_system:?}: got:\n{r}"
        );
        assert!(
            !r.lines().any(|l| l == "to add."),
            "{class_system:?}: got:\n{r}"
        );
    }
}

// endregion
