use super::*;

const CLASSES: &[&str] = &["pkg_error_argument", "pkg_error"];

fn sample_inputs() -> (Vec<CallDefRow>, Vec<AltrepRegRow>, Vec<TraitDispatchRow>) {
    let call_defs = vec![
        CallDefRow {
            name: "miniextendr_my_fn".into(),
            num_args: 0,
        },
        CallDefRow {
            name: "miniextendr_other".into(),
            num_args: 3,
        },
    ];
    let altrep_regs = vec![AltrepRegRow {
        symbol: "__mx_altrep_reg_MyType".into(),
    }];
    let trait_dispatches = vec![TraitDispatchRow {
        concrete_tag: mx_tag::new(0xdead_beef_dead_beef, 0x1234_5678_1234_5678),
        trait_tag: mx_tag::new(0xcafe_babe_cafe_babe, 0xfeed_face_feed_face),
        vtable_symbol: "__VTABLE_mypkg_Counter_FOR_MyType".into(),
    }];
    (call_defs, altrep_regs, trait_dispatches)
}

#[test]
fn case_distinct_vtables_keep_separate_externs_and_dispatch_rows() {
    let upper = TraitDispatchRow {
        concrete_tag: mx_tag::new(1, 2),
        trait_tag: mx_tag::new(3, 4),
        vtable_symbol: "__VTABLE_mypkg_Counter_FOR_Foo".into(),
    };
    let lower = TraitDispatchRow {
        concrete_tag: upper.concrete_tag,
        trait_tag: mx_tag::new(5, 6),
        vtable_symbol: "__VTABLE_mypkg_counter_FOR_Foo".into(),
    };
    let out = format_wasm_registry(&[], &[], &[upper, lower], &[]);
    for symbol in [
        "__VTABLE_mypkg_Counter_FOR_Foo",
        "__VTABLE_mypkg_counter_FOR_Foo",
    ] {
        assert!(out.contains(&format!(
            "#[allow(non_upper_case_globals)]\n    pub static {symbol}: u8;"
        )));
        assert!(out.contains(&format!("from_ref(&{symbol})")));
    }
}

#[test]
fn header_carries_generator_version_and_content_hash() {
    let (a, b, c) = sample_inputs();
    let out = format_wasm_registry(&a, &b, &c, CLASSES);
    assert!(
        out.contains(&format!("generator-version: {GENERATOR_VERSION}")),
        "expected generator-version line; got:\n{out}"
    );
    assert!(
        out.contains("content-hash:      "),
        "expected content-hash line; got:\n{out}"
    );
}

#[test]
fn content_hash_is_deterministic() {
    let (a, b, c) = sample_inputs();
    let first = format_wasm_registry(&a, &b, &c, CLASSES);
    let second = format_wasm_registry(&a, &b, &c, CLASSES);
    assert_eq!(first, second);
}

#[test]
fn registration_order_does_not_change_snapshot() {
    let (mut calls, mut altreps, mut dispatches) = sample_inputs();
    altreps.push(AltrepRegRow {
        symbol: "__mx_altrep_reg_AnotherType".into(),
    });
    dispatches.push(TraitDispatchRow {
        concrete_tag: mx_tag::new(1, 2),
        trait_tag: mx_tag::new(3, 4),
        vtable_symbol: "__VTABLE_ANOTHER".into(),
    });
    let expected = format_wasm_registry(&calls, &altreps, &dispatches, CLASSES);
    calls.reverse();
    altreps.reverse();
    dispatches.reverse();
    assert_eq!(
        expected,
        format_wasm_registry(&calls, &altreps, &dispatches, CLASSES)
    );
}

#[test]
fn conversion_error_class_precedence_is_preserved() {
    let (calls, altreps, dispatches) = sample_inputs();
    let forward = format_wasm_registry(&calls, &altreps, &dispatches, CLASSES);
    let reversed = format_wasm_registry(&calls, &altreps, &dispatches, &[CLASSES[1], CLASSES[0]]);
    assert_ne!(forward, reversed);
    assert!(
        forward.find("\"pkg_error_argument\"").unwrap() < forward.find("\"pkg_error\"").unwrap()
    );
}

#[test]
fn content_hash_changes_when_body_changes() {
    let (a, b, c) = sample_inputs();
    let (mut a2, _, _) = sample_inputs();
    a2.push(CallDefRow {
        name: "different_fn".into(),
        num_args: 1,
    });
    let first = format_wasm_registry(&a, &b, &c, CLASSES);
    let second = format_wasm_registry(&a2, &b, &c, CLASSES);
    assert_ne!(first, second);
}

#[test]
fn emits_extern_decls_for_every_referenced_symbol() {
    let (a, b, c) = sample_inputs();
    let out = format_wasm_registry(&a, &b, &c, CLASSES);
    // call wrappers in the C-unwind block — params bound to `_` so the
    // declaration parses (extern fn decls require parameter bindings).
    assert!(out.contains("pub fn miniextendr_my_fn() -> SEXP;"));
    assert!(out.contains("pub fn miniextendr_other(_: SEXP, _: SEXP, _: SEXP) -> SEXP;"));
    // ALTREP registrations + vtables in the registration block
    assert!(out.contains("pub safe fn __mx_altrep_reg_MyType();"));
    assert!(out.contains("pub static __VTABLE_mypkg_Counter_FOR_MyType: u8;"));
}

#[test]
fn emits_named_slice_constants() {
    let (a, b, c) = sample_inputs();
    let out = format_wasm_registry(&a, &b, &c, CLASSES);
    assert!(out.contains("pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef]"));
    assert!(out.contains("pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration]"));
    assert!(out.contains("pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry]"));
    assert!(out.contains("pub static MX_CONVERSION_ERROR_CLASS_WASM: &[&str]"));
}

/// #1768: the crate's `conversion_error_class` crosses to wasm32 in the
/// snapshot, in order, as Rust string literals.
#[test]
fn emits_conversion_error_class_literals() {
    let (a, b, c) = sample_inputs();
    let out = format_wasm_registry(&a, &b, &c, &["pkg_error_argument", r#"odd"\class"#]);
    assert!(
        out.contains(
            "pub static MX_CONVERSION_ERROR_CLASS_WASM: &[&str] = &[\n    \
                 \"pkg_error_argument\",\n    \"odd\\\"\\\\class\",\n];"
        ),
        "expected the escaped class literals in order; got:\n{out}"
    );
}

#[test]
fn conversion_error_class_changes_the_content_hash() {
    let (a, b, c) = sample_inputs();
    assert_ne!(
        format_wasm_registry(&a, &b, &c, CLASSES),
        format_wasm_registry(&a, &b, &c, &[])
    );
}

#[test]
fn renders_mx_tag_with_const_constructor() {
    let (a, b, c) = sample_inputs();
    let out = format_wasm_registry(&a, &b, &c, CLASSES);
    assert!(
        out.contains("::miniextendr_api::abi::mx_tag::new(0xdeadbeefdeadbeef, 0x1234567812345678)"),
        "expected fully-qualified concrete_tag literal; got:\n{out}"
    );
    assert!(
        out.contains("::miniextendr_api::abi::mx_tag::new(0xcafebabecafebabe, 0xfeedfacefeedface)"),
        "expected fully-qualified trait_tag literal; got:\n{out}"
    );
}

/// #1307: a minimal package (call defs only — no trait impls, no ALTREP)
/// produces an empty dispatch table; the generated file must not carry
/// imports that only dispatch entries would use, or wasm32 `cargo check`
/// reports `unused_imports` on every build.
#[test]
fn dispatch_free_output_has_no_dispatch_only_imports() {
    let (call_defs, _, _) = sample_inputs();
    let out = format_wasm_registry(&call_defs, &[], &[], &[]);
    assert!(
        !out.contains("mx_tag"),
        "dispatch-free output must not mention mx_tag; got:\n{out}"
    );
    assert!(
        !out.contains("c_void"),
        "dispatch-free output must not mention c_void; got:\n{out}"
    );
}

/// Every emitted `use` line must be referenced in the body that follows —
/// the generalized no-unused-imports property for the minimal-package
/// shape (#1307). Imports are single-segment or `{A, B}` groups; each
/// imported ident must reappear after the import block.
#[test]
fn every_import_is_used_in_dispatch_free_output() {
    let (call_defs, _, _) = sample_inputs();
    let out = format_wasm_registry(&call_defs, &[], &[], &[]);
    let (imports, rest): (Vec<&str>, Vec<&str>) = out.lines().partition(|l| l.starts_with("use "));
    assert!(!imports.is_empty(), "output has imports; got:\n{out}");
    let body = rest.join("\n");
    for line in imports {
        let idents = line
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .rsplit("::")
            .next()
            .unwrap()
            .trim_matches(['{', '}'])
            .split(',')
            .map(str::trim);
        for ident in idents {
            assert!(
                body.contains(ident),
                "import `{ident}` from `{line}` is unused; got:\n{out}"
            );
        }
    }
}

/// With dispatch entries present, the emitted entries must reference
/// `mx_tag` / `c_void` fully qualified (no header import exists to
/// resolve bare names — #1307 dropped them).
#[test]
fn dispatch_entries_reference_items_fully_qualified() {
    let (a, b, c) = sample_inputs();
    let out = format_wasm_registry(&a, &b, &c, CLASSES);
    assert!(
        !out.contains("use ::miniextendr_api::abi::mx_tag;"),
        "mx_tag header import must not be emitted; got:\n{out}"
    );
    assert!(
        !out.contains("use ::core::ffi::c_void;"),
        "c_void header import must not be emitted; got:\n{out}"
    );
    assert!(
        out.contains("::miniextendr_api::abi::mx_tag::new("),
        "dispatch tags must be fully qualified; got:\n{out}"
    );
    assert!(
        out.contains(".cast::<::core::ffi::c_void>()"),
        "vtable cast must be fully qualified; got:\n{out}"
    );
}

#[test]
fn empty_inputs_produce_empty_slices() {
    let out = format_wasm_registry(&[], &[], &[], &[]);
    assert!(!out.contains("use ::miniextendr_api::SEXP;"));
    assert!(out.contains("pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[\n];"));
    assert!(
        out.contains("pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &[\n];")
    );
    assert!(out.contains("pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[\n];"));
    assert!(out.contains("pub static MX_CONVERSION_ERROR_CLASS_WASM: &[&str] = &[\n];"));
}

#[test]
fn param_list_uses_underscore_bindings() {
    assert_eq!(sexp_param_list(0), "");
    assert_eq!(sexp_param_list(1), "_: SEXP");
    assert_eq!(sexp_param_list(3), "_: SEXP, _: SEXP, _: SEXP");
}

#[test]
fn type_list_is_bare_types() {
    assert_eq!(sexp_type_list(0), "");
    assert_eq!(sexp_type_list(1), "SEXP");
    assert_eq!(sexp_type_list(3), "SEXP, SEXP, SEXP");
}

#[test]
fn altrep_register_decls_use_safe_unwind_abi() {
    let (a, b, c) = sample_inputs();
    let out = format_wasm_registry(&a, &b, &c, CLASSES);
    assert!(
        out.contains("unsafe extern \"C-unwind\" {\n    pub safe fn __mx_altrep_reg_MyType();"),
        "expected `safe extern C-unwind` to match AltrepRegistration.register; got:\n{out}"
    );
}
