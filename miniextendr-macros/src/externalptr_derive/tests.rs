use super::{ClassSystem, generate_class_integration_r_code, generate_r_wrapper_for_slot};

const ALL_CLASS_SYSTEMS: [ClassSystem; 6] = [
    ClassSystem::Env,
    ClassSystem::R6,
    ClassSystem::S3,
    ClassSystem::S4,
    ClassSystem::S7,
    ClassSystem::Vctrs,
];

/// Sidecar accessor C functions take only `x` (getter) or `x, value` (setter) —
/// no `__miniextendr_call` parameter. The R wrappers must pass no `.call` argument
/// because it would be counted as an extra positional argument by `.Call()`, causing
/// "Incorrect number of arguments" errors at runtime. Cover every class system variant.
#[test]
fn sidecar_accessors_pass_no_call_slot() {
    let getter_c = "C__mx_rdata_get_T_f";
    let setter_c = "C__mx_rdata_set_T_f";

    for cs in ALL_CLASS_SYSTEMS {
        let out = generate_r_wrapper_for_slot(cs, "T", "f", getter_c, Some(setter_c), true, true);
        // Correct form: no .call argument — the C function accepts x (getter)
        // or x, value (setter), then the `require_live` flag, `FALSE` for a
        // standalone accessor (#1891).
        assert!(
            out.contains(&format!(".Call({getter_c}, x, FALSE)")),
            "{cs:?} getter should call without .call:\n{out}"
        );
        assert!(
            out.contains(&format!(".Call({setter_c}, x, value, FALSE)")),
            "{cs:?} setter should call without .call:\n{out}"
        );
        // No `.call =` in either spelling.
        assert!(
            !out.contains(".call ="),
            "{cs:?} sidecar wrapper must not pass a .call argument:\n{out}"
        );
    }
}

/// Every sidecar R wrapper (getter and setter, all class systems) must
/// carry the tagged-condition guard so Rust panics / conversion failures
/// transported by `with_r_unwind_protect(…, None)` are re-raised as
/// structured R conditions instead of leaking the tagged SEXP to the user.
#[test]
fn sidecar_accessors_reraise_tagged_conditions() {
    let getter_c = "C__mx_rdata_get_T_f";
    let setter_c = "C__mx_rdata_set_T_f";

    for cs in ALL_CLASS_SYSTEMS {
        let out = generate_r_wrapper_for_slot(cs, "T", "f", getter_c, Some(setter_c), true, true);
        assert_eq!(
            out.matches(".miniextendr_raise_condition(.val, sys.call())")
                .count(),
            2,
            "{cs:?} getter+setter should each carry the condition guard:\n{out}"
        );
        // A computed field gets the getter only (#1883).
        let out = generate_r_wrapper_for_slot(cs, "T", "f", getter_c, None, true, true);
        assert!(out.contains("T_get_f <- function(x)"), "{out}");
        assert!(!out.contains("T_set_f"), "{out}");
        assert_eq!(
            out.matches(".miniextendr_raise_condition(.val, sys.call())")
                .count(),
            1,
            "{cs:?} computed getter should carry the condition guard:\n{out}"
        );
    }
}

/// A sidecar setter's conversion failure is the argument error of #1594
/// on its `value` formal (`e$param`), naming the field: `'f' must be
/// <expected>`, the field's Rust type as `e$rust_type`, and the reason probed from the error (a
/// `Conversion` slot) or worded from the rejected value (a scalar slot).
#[test]
fn sidecar_setter_raises_the_argument_error() {
    let body = |ty: syn::Type, kind: super::SlotKind| {
        let slot = super::SidecarSlot {
            name: syn::Ident::new("f", proc_macro2::Span::call_site()),
            r_name: String::from("f"),
            ty,
            vis: syn::parse_quote!(pub),
            is_public: true,
            kind,
            access: super::SidecarAccess::default(),
            prop_doc: None,
            setter_invisible: None,
            read_only: false,
            null_if_none: false,
            cache: None,
        };
        super::generate_setter_body(&syn::Ident::new("T", proc_macro2::Span::call_site()), &slot)
            .to_string()
    };

    let s = body(syn::parse_quote!(i32), super::SlotKind::ScalarInt);
    assert!(s.contains("\"'f' must be a number\" , \"value\""), "{s}");
    assert!(s.contains("scalar_rejection_reason (value)"), "{s}");
    assert!(s.contains("Some (\"i32\")"), "{s}");
    assert!(s.contains("__mx_conversion_err_parts ! ("), "{s}");
    assert!(!s.contains("failed to convert"), "{s}");

    let s = body(syn::parse_quote!(bool), super::SlotKind::ScalarLogical);
    assert!(s.contains("\"'f' must be TRUE or FALSE\""), "{s}");

    let s = body(syn::parse_quote!(Vec<String>), super::SlotKind::Conversion);
    assert!(s.contains("\"'f' must be character\" , \"value\""), "{s}");
    assert!(s.contains("Some (\"Vec<String>\")"), "{s}");
    assert!(s.contains(", true)"), "{s}");

    // No static expectation: the error may supply one at run time (a
    // `match_arg` enum field), else `invalid 'f' argument`.
    let s = body(syn::parse_quote!(MyType), super::SlotKind::Conversion);
    assert!(s.contains("__mx_conversion_expectation ! (e)"), "{s}");
    assert!(s.contains("conversion_prefix (\"f\" , false ,"), "{s}");
    assert!(
        s.contains(") , \"value\" , :: core :: option :: Option :: Some (\"MyType\")"),
        "{s}"
    );
    assert!(s.contains("Some (\"MyType\")"), "{s}");

    // A `Nullable<T>` field reads as `Option<T>` does, so its refusal says
    // what an `Option<T>` field's says, with `NULL or` (#2270); the Rust
    // type is the field's own.
    let expected =
        crate::r_preconditions::conversion_expectation(&syn::parse_quote!(Option<String>), false)
            .expect("Option<String> has an expectation");
    assert!(expected.starts_with("NULL or "), "{expected}");
    let option = body(
        syn::parse_quote!(Option<String>),
        super::SlotKind::Conversion,
    );
    let nullable = body(
        syn::parse_quote!(Nullable<String>),
        super::SlotKind::Conversion,
    );
    for s in [&option, &nullable] {
        assert!(
            s.contains(&format!("\"'f' must be {expected}\" , \"value\"")),
            "{s}"
        );
    }
    assert!(
        nullable.contains("Some (\"Nullable<String>\")"),
        "{nullable}"
    );
    assert!(
        nullable.contains("< Nullable < String > as TryFromSexp > :: try_from_sexp (value)"),
        "{nullable}"
    );
}

/// The R6 active-binding and S7 property integration code also call the
/// sidecar C entrypoints directly; they need the same guard (with
/// `sys.call()` fallback attribution) and must not pass `.call =`.
#[test]
fn sidecar_class_integration_reraises_tagged_conditions() {
    let slot = super::SidecarSlot {
        name: syn::Ident::new("f", proc_macro2::Span::call_site()),
        r_name: String::from("f"),
        ty: syn::parse_quote!(i32),
        vis: syn::parse_quote!(pub),
        is_public: true,
        kind: super::SlotKind::ScalarInt,
        access: super::SidecarAccess::default(),
        prop_doc: None,
        setter_invisible: None,
        read_only: false,
        null_if_none: false,
        cache: None,
    };
    let slots = [&slot];

    for cs in [ClassSystem::R6, ClassSystem::S7] {
        let out = generate_class_integration_r_code(cs, "T", &slots, &Default::default());
        assert_eq!(
            out.matches(".miniextendr_raise_condition(.val, sys.call())")
                .count(),
            2,
            "{cs:?} integration getter+setter should each carry the condition guard:\n{out}"
        );
        assert!(
            !out.contains(".call ="),
            "{cs:?} integration code must not pass a .call argument:\n{out}"
        );
        // The bindings / properties keep reading a save: `FALSE` as the
        // accessors' `require_live` flag (#1891).
        let getter_c = crate::naming::sidecar_getter_c_name("T", "f");
        let setter_c = crate::naming::sidecar_setter_c_name("T", "f");
        assert!(
            out.contains(&format!(".Call({getter_c}, ")) && out.contains(", FALSE)"),
            "{out}"
        );
        assert!(
            out.contains(&format!(".Call({setter_c}, ")) && out.contains(", value, FALSE)"),
            "{out}"
        );
    }
}

/// `#[r_data(name = "...")]` renames the R field: the R6 binding, the S7
/// property and its `@prop` entry, the standalone accessors and the field
/// list use the R name, and so does a save's slot name (#2215); the C
/// symbols, the `CallDef` statics and the Rust accessors keep the Rust
/// identifier (#1891).
#[test]
fn r_data_name_renames_the_r_field_only() {
    let input: syn::DeriveInput = syn::parse_str(
        "struct Engine { #[r_data] _r: RSidecar, #[r_data(name = \"keys\")] pub r_keys: Sidecar<Vec<i32>>, \
             #[r_data(name = \"n.rows\")] pub n_rows: i32 }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::R6).unwrap();
    assert_eq!(info.slots[0].r_name, "keys");
    assert_eq!(info.slots[1].r_name, "n.rows");
    let pub_slots: Vec<_> = info.slots.iter().collect();

    let helpers =
        super::generate_field_syntax_helpers("Engine", &pub_slots, &Default::default(), false);
    assert!(
        helpers.contains(".rdata_fields_Engine <- c(\"keys\", \"n.rows\")\n"),
        "{helpers}"
    );
    let getter = crate::naming::sidecar_getter_c_name("Engine", "r_keys");
    assert!(
        helpers.contains(&format!("  keys = .Call({getter}, x, TRUE)")),
        "{helpers}"
    );
    assert!(
        helpers.contains("its fields are `keys`, `n.rows`"),
        "{helpers}"
    );

    let r6 = super::generate_class_integration_r_code(
        ClassSystem::R6,
        "Engine",
        &pub_slots,
        &Default::default(),
    );
    assert!(r6.contains("cls$set(\"active\", \"keys\","), "{r6}");
    assert!(
        r6.contains(&format!(".Call({getter}, private$.ptr, FALSE)")),
        "{r6}"
    );
    assert!(!r6.contains("\"r_keys\""), "{r6}");
    let s7 = super::generate_class_integration_r_code(
        ClassSystem::S7,
        "Engine",
        &pub_slots,
        &Default::default(),
    );
    assert!(s7.contains("    keys = S7::new_property("), "{s7}");

    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(out.contains("Engine_get_keys <- function(x)"), "{out}");
    assert!(
        out.contains("Engine_set_keys <- function(x, value)"),
        "{out}"
    );
    assert!(out.contains("Engine_get_n.rows <- function(x)"), "{out}");
    assert!(!out.contains("Engine_get_r_keys"), "{out}");
    assert!(out.contains(&getter), "{out}");
    assert!(
        out.contains("__MX_CALL_DEF_RDATA_GET_Engine_r_keys"),
        "{out}"
    );
    assert!(
        out.contains("__MX_CALL_DEF_RDATA_GET_Engine_n_rows"),
        "{out}"
    );
    // The Rust accessors keep the Rust name; the visitor hook, which
    // names the save's slots, passes the R name (#2215).
    assert!(out.contains("pub fn r_keys (ptr :"), "{out}");
    assert!(out.contains("pub fn set_r_keys (ptr :"), "{out}");
    assert!(
        out.contains("visit (0usize , \"keys\" , & self . r_keys)"),
        "{out}"
    );
    // The setter's message names the R field.
    assert!(out.contains("'n.rows' must be a number"), "{out}");
}

/// The R name must be non-empty, syntactic, unique, not `.ptr`, and not
/// on the selector.
#[test]
fn r_data_name_is_checked() {
    let refused = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_sidecar_info(&input, ClassSystem::Env)
            .err()
            .map(|err| err.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"))
    };
    let message = refused("struct E { #[r_data(name = \"\")] pub count: i32 }");
    assert!(message.contains("can't be empty"), "{message}");
    let message = refused("struct E { #[r_data(name = \".ptr\")] pub count: i32 }");
    assert!(message.contains("`.ptr` is the element"), "{message}");
    let message = refused("struct E { #[r_data(name = \"n rows\")] pub count: i32 }");
    assert!(message.contains("not a syntactic R name"), "{message}");
    let message = refused("struct E { #[r_data(name = \"for\")] pub count: i32 }");
    assert!(message.contains("not a syntactic R name"), "{message}");
    let message = refused(
        "struct E { #[r_data] pub keys: i32, #[r_data(name = \"keys\")] pub r_keys: Sidecar<Vec<i32>> }",
    );
    assert!(
        message.contains("`keys` is already taken by the field `keys`"),
        "{message}"
    );
    let message =
        refused("struct E { #[r_data(name = \"r\")] _r: RSidecar, #[r_data] pub count: i32 }");
    assert!(message.contains("not the RSidecar selector"), "{message}");
    let message = refused("struct E { #[r_data(name = \"a\", name = \"b\")] pub count: i32 }");
    assert!(message.contains("duplicate `name`"), "{message}");
}

/// A `Computed` field (#1883): its getter arm calls the `get` function on
/// the live struct (a leading `Self` resolved to the type), it takes its
/// declared place among the other fields, it gets no setter C function,
/// `CallDef` or `Type_set_<f>()`, and the setter switch raises
/// `miniextendr_read_only_field` for it.
#[test]
fn computed_fields_get_a_getter_and_a_read_only_setter_arm() {
    let input: syn::DeriveInput = syn::parse_str(
        "struct Engine { #[r_data] _r: RSidecar, \
             #[r_data(get = \"Self::group_by\")] pub group_by: Computed, \
             #[r_data(name = \"keys\")] pub r_keys: Sidecar<Vec<i32>>, \
             #[r_data(get = \"n_rows_of\")] pub n_rows: Computed, \
             #[r_data] pub base: i32 }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::S3).unwrap();
    let kinds: Vec<_> = info.slots.iter().map(|slot| slot.kind.clone()).collect();
    assert_eq!(
        kinds,
        [
            super::SlotKind::Computed(syn::parse_quote!(Engine::group_by)),
            super::SlotKind::Sidecar(0),
            super::SlotKind::Computed(syn::parse_quote!(n_rows_of)),
            super::SlotKind::ScalarInt,
        ]
    );
    let name = syn::Ident::new("Engine", proc_macro2::Span::call_site());
    let getter = super::generate_getter_body(&name, &info.slots[0]).to_string();
    assert!(
        getter.contains("sidecar_r_struct :: < Engine > (x)"),
        "{getter}"
    );
    // The getter's value goes through the `Result` probe (#2269), then the
    // return conversion, a failed one raising (#2272).
    assert!(
        getter.contains("let __mx_value = Engine :: group_by (data) ;"),
        "{getter}"
    );
    assert!(
        getter.contains(
            "(&& & :: miniextendr_api :: externalptr :: ComputedProbe :: of (& __mx_value)) \
             . __mx_computed_read (__mx_value)"
        ),
        "{getter}"
    );
    assert!(
        getter.contains("ComputedRead :: Null => { :: miniextendr_api :: SEXP :: nil () }"),
        "{getter}"
    );
    assert!(
        getter.contains(
            "ComputedRead :: Err (e) => { :: miniextendr_api :: error_value :: \
             result_err_condition_value (:: miniextendr_api :: __mx_result_err_parts ! (e ,"
        ),
        "{getter}"
    );
    assert!(
        getter.contains(":: miniextendr_api :: into_r :: IntoR :: try_into_sexp (__mx_ok)"),
        "{getter}"
    );
    assert!(!getter.contains(":: into_sexp ("), "{getter}");
    assert!(
        super::generate_setter_body(&name, &info.slots[0]).is_empty(),
        "a computed field has no setter body"
    );

    let pub_slots: Vec<_> = info.slots.iter().collect();
    let helpers =
        super::generate_field_syntax_helpers("Engine", &pub_slots, &Default::default(), false);
    assert!(
        helpers
            .contains(".rdata_fields_Engine <- c(\"group_by\", \"keys\", \"n_rows\", \"base\")\n"),
        "{helpers}"
    );
    let group_by_getter = crate::naming::sidecar_getter_c_name("Engine", "group_by");
    assert!(
        helpers.contains(&format!("  group_by = .Call({group_by_getter}, x, TRUE),")),
        "{helpers}"
    );
    // The refusal names the receiver's R class (`.rdata_read_only_Engine`)
    // and says why (#2268).
    assert!(
        helpers.contains(
            "  group_by = .rdata_read_only_Engine(\"group_by\", x, parent.frame(), \"is computed from \
             the Rust value and can't be assigned\"),"
        ),
        "{helpers}"
    );
    let keys_setter = crate::naming::sidecar_setter_c_name("Engine", "r_keys");
    assert!(
        helpers.contains(&format!("  keys = .Call({keys_setter}, x, value, TRUE),")),
        "{helpers}"
    );

    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(out.contains("Engine_get_group_by <- function(x)"), "{out}");
    assert!(!out.contains("Engine_set_group_by"), "{out}");
    assert!(
        out.contains("__MX_CALL_DEF_RDATA_GET_Engine_group_by"),
        "{out}"
    );
    assert!(
        !out.contains("__MX_CALL_DEF_RDATA_SET_Engine_group_by"),
        "{out}"
    );
    assert!(
        !out.contains(&crate::naming::sidecar_setter_c_name("Engine", "group_by")),
        "{out}"
    );
    assert!(
        out.contains("Engine_set_keys <- function(x, value)"),
        "{out}"
    );
    // Only the `Sidecar` field's accessors read the flag; the others take
    // it unused.
    assert!(
        out.contains("_require_live : :: miniextendr_api :: SEXP"),
        "{out}"
    );
    assert!(out.contains("sidecar_require_live (require_live)"), "{out}");
}

/// A struct field converts its clone with `try_into_sexp`; a failed
/// conversion is the `result_err` condition, with no call slot (#2272).
#[test]
fn conversion_fields_raise_a_failed_conversion() {
    let input: syn::DeriveInput =
        syn::parse_str("struct Engine { #[r_data] _r: RSidecar, #[r_data] pub label: String }")
            .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::S3).unwrap();
    assert_eq!(info.slots[0].kind, super::SlotKind::Conversion);
    let name = syn::Ident::new("Engine", proc_macro2::Span::call_site());
    let getter = super::generate_getter_body(&name, &info.slots[0]).to_string();
    assert!(
        getter.contains("match :: miniextendr_api :: into_r :: IntoR :: try_into_sexp (val)"),
        "{getter}"
    );
    assert!(
        getter
            .contains("__mx_into_r_err_parts ! (__mx_error) , :: core :: option :: Option :: None"),
        "{getter}"
    );
    assert!(!getter.contains(":: into_sexp ("), "{getter}");
}

/// The `Computed` compile errors: `get` on another type, no `get`,
/// `ref` / `mut` / `setter` on it and a non-`pub` one; one under R6 / S7 is
/// accepted (#1888).
#[test]
fn computed_fields_are_checked() {
    let refused = |source: &str, system: ClassSystem| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_sidecar_info(&input, system)
            .err()
            .map(|err| err.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"))
    };
    let message = refused(
        "struct E { #[r_data(get = \"Self::f\")] pub count: i32 }",
        ClassSystem::S3,
    );
    assert!(
        message.contains("`get` applies to a `Computed` field only"),
        "{message}"
    );
    let message = refused(
        "struct E { #[r_data] pub count: Computed }",
        ClassSystem::S3,
    );
    assert!(
        message.contains("needs `#[r_data(get = \"Self::f\")]`"),
        "{message}"
    );
    let message = refused(
        "struct E { #[r_data(ref, get = \"Self::f\")] pub count: Computed }",
        ClassSystem::S3,
    );
    assert!(
        message.contains("read through its `get` function"),
        "{message}"
    );
    let message = refused(
        "struct E { #[r_data(setter = \"visible\", get = \"Self::f\")] pub count: Computed }",
        ClassSystem::S3,
    );
    assert!(
        message.contains("not the RSidecar selector or a `Computed` field"),
        "{message}"
    );
    let message = refused(
        "struct E { #[r_data(get = \"Self::f\")] count: Computed }",
        ClassSystem::S3,
    );
    assert!(message.contains("must be `pub`"), "{message}");
    // R6 and S7 take one too: a read-only binding / property (#1888).
    for system in [ClassSystem::R6, ClassSystem::S7] {
        let input: syn::DeriveInput =
            syn::parse_str("struct E { #[r_data(get = \"Self::f\")] pub count: Computed }")
                .unwrap();
        assert!(super::parse_sidecar_info(&input, system).is_ok());
    }
    let message = refused(
        "struct E { #[r_data(get = \"not a path\")] pub count: Computed }",
        ClassSystem::S3,
    );
    assert!(message.contains("`get` must be a path"), "{message}");
}

/// `#[externalptr(restored(class = ..., message = ...))]` (#1891): the
/// classes (one, or several in order) and the message reach the
/// `TypedExternal` impl; each alone works; the arguments are checked.
#[test]
fn restored_options_reach_the_typed_external_impl() {
    let attrs = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_externalptr_attrs(&input).map(|attrs| (attrs.class_system, attrs.restored))
    };
    let (system, restored) = attrs(
        "#[externalptr(s3, restored(class = [\"pkg_saved\", \"pkg_error\"], message = \"saved; rebuild it\"))] struct E;",
    )
    .unwrap();
    assert_eq!(system, ClassSystem::S3);
    assert_eq!(restored.class, ["pkg_saved", "pkg_error"]);
    assert_eq!(restored.message.as_deref(), Some("saved; rebuild it"));
    let (system, restored) =
        attrs("#[externalptr(restored(class = \"pkg_saved\"))] #[externalptr(s4)] struct E;")
            .unwrap();
    assert_eq!(system, ClassSystem::S4);
    assert_eq!(restored.class, ["pkg_saved"]);
    assert_eq!(restored.message, None);
    let (_, restored) = attrs("#[externalptr(restored(message = \"m\"))] struct E;").unwrap();
    assert!(restored.class.is_empty());
    assert_eq!(restored.message.as_deref(), Some("m"));
    let (_, restored) = attrs("#[externalptr(env)] struct E;").unwrap();
    assert_eq!(restored, super::RestoredOptions::default());

    let refused = |source: &str, expected: &str| {
        let message = attrs(source)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"));
        assert!(message.contains(expected), "`{source}`: {message}");
    };
    // syn refuses the empty parentheses itself.
    refused(
        "#[externalptr(restored())] struct E;",
        "expected nested attribute",
    );
    refused(
        "#[externalptr(restored(class = []))] struct E;",
        "at least one",
    );
    refused(
        "#[externalptr(restored(class = \"\"))] struct E;",
        "can't be empty",
    );
    refused(
        "#[externalptr(restored(message = \"\"))] struct E;",
        "can't be empty",
    );
    refused(
        "#[externalptr(restored(class = \"a\", class = \"b\"))] struct E;",
        "given twice",
    );
    refused(
        "#[externalptr(restored(class = \"a\"), restored(class = \"b\"))] struct E;",
        "given twice",
    );
    refused(
        "#[externalptr(restored(klass = \"a\"))] struct E;",
        "unknown `restored` option",
    );
    refused(
        "#[externalptr(saved)] struct E;",
        "unknown `externalptr` option",
    );

    let input: syn::DeriveInput = syn::parse_str(
        "#[externalptr(s3, restored(class = [\"pkg_saved\", \"pkg_error\"], message = \"saved; rebuild it\"))] struct E { n: i32 }",
    )
    .unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(
        out.contains(
            "const RESTORED_ERROR_CLASS : & 'static [& 'static str] = & [\"pkg_saved\" , \"pkg_error\"] ;"
        ),
        "{out}"
    );
    assert!(
        out.contains(
            "const RESTORED_ERROR_MESSAGE : :: core :: option :: Option < & 'static str > = :: core :: option :: Option :: Some (\"saved; rebuild it\") ;"
        ),
        "{out}"
    );
    let input: syn::DeriveInput = syn::parse_str("struct E { n: i32 }").unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(!out.contains("RESTORED_ERROR_"), "{out}");
}

/// A save names its slots by the R names (#2215): `name = ..`, else the
/// ident. The visitor, which flushes, loads and lays out a restored
/// value's slots, and every accessor pass the same name; the getters keep
/// the Rust names.
#[test]
fn sidecar_slots_are_named_by_the_r_name() {
    let input: syn::DeriveInput = syn::parse_str(
        "struct E { #[r_data] keys: Sidecar<Vec<i32>>, #[r_data(name = \"n.rows\")] r_rows: Sidecar<i32> }",
    )
    .unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(
        out.contains(
            "const SIDECAR_NAMES : & 'static [& 'static str] = & [\"keys\" , \"n.rows\"] ;"
        ),
        "{out}"
    );
    assert!(
        out.contains("visit (1usize , \"n.rows\" , & self . r_rows) ;"),
        "{out}"
    );
    assert!(
        out.contains("sidecar_get :: < Self , i32 > (ptr , 1usize , \"n.rows\" ,"),
        "{out}"
    );
    assert!(
        out.contains("sidecar_set :: < Self , i32 > (ptr , 1usize , \"n.rows\" ,"),
        "{out}"
    );
    assert!(!out.contains("\"r_rows\""), "{out}");
}

/// `restored(no_handle = ...)` (#2218): parsed and checked, and emitted
/// as the `TypedExternal` message.
#[test]
fn restored_no_handle_option() {
    let attrs = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_externalptr_attrs(&input)
    };
    let parsed =
        attrs("#[externalptr(s3, restored(no_handle = \"no engine\"))] struct E;").unwrap();
    assert_eq!(parsed.restored.no_handle.as_deref(), Some("no engine"));

    let refused = |source: &str, expected: &str| {
        let message = attrs(source)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"));
        assert!(message.contains(expected), "`{source}`: {message}");
    };
    refused(
        "#[externalptr(restored(no_handle = \"\"))] struct E;",
        "`no_handle` can't be empty",
    );
    refused(
        "#[externalptr(restored(no_handle = \"a\", no_handle = \"b\"))] struct E;",
        "`no_handle` is given twice",
    );
    refused(
        "#[externalptr(restored(message = \"a\", message = \"b\"))] struct E;",
        "`message` is given twice",
    );

    let input: syn::DeriveInput =
        syn::parse_str("#[externalptr(restored(no_handle = \"no engine\"))] struct E { n: i32 }")
            .unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(
        out.contains(
            "const NO_HANDLE_MESSAGE : :: core :: option :: Option < & 'static str > = :: core :: option :: Option :: Some (\"no engine\") ;"
        ),
        "{out}"
    );
    let input: syn::DeriveInput = syn::parse_str("struct E { n: i32 }").unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(!out.contains("NO_HANDLE_MESSAGE"), "{out}");
}

/// The probe for a serde value (#2186) and its carrier's registration,
/// emitted for every type: the type's own impl picks the serde arm or the
/// fallback.
#[test]
fn snapshot_probe_and_registration() {
    let input: syn::DeriveInput = syn::parse_str("struct E { n: i32 }").unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(out.contains("fn __mx_snapshot ()"), "{out}");
    assert!(out.contains("SnapshotProbe :: < Self >"), "{out}");
    assert!(out.contains("__mx_register_snapshot :: < E >"), "{out}");
    assert!(out.contains("MX_ALTREP_REGISTRATIONS"), "{out}");
}

/// `field_error(class = ...)` (#1899) and `restored(refuse =
/// "every_read")` (#1901): parsed and checked, the classes reach the
/// `TypedExternal` impl and the R helpers, and the opt-in emits the
/// liveness check; without them the helpers are `NULL`.
#[test]
fn field_error_and_every_read_options() {
    let attrs = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_externalptr_attrs(&input)
    };
    let parsed = attrs(
        "#[externalptr(s3, field_error(class = [\"pkg_config\", \"pkg_error\"]), restored(class = \"pkg_saved\", refuse = \"every_read\"))] struct E;",
    )
    .unwrap();
    assert_eq!(parsed.field_error.class, ["pkg_config", "pkg_error"]);
    assert_eq!(parsed.field_error.message, None);
    assert!(parsed.restored.every_read);
    assert_eq!(parsed.restored.class, ["pkg_saved"]);
    let parsed = attrs("#[externalptr(field_error(class = \"pkg_error\"))] struct E;").unwrap();
    assert_eq!(parsed.field_error.class, ["pkg_error"]);
    // A message alone keeps miniextendr's classes (#1920).
    let parsed = attrs("#[externalptr(field_error(message = \"not a rec\"))] struct E;").unwrap();
    assert!(parsed.field_error.class.is_empty());
    assert_eq!(parsed.field_error.message.as_deref(), Some("not a rec"));
    assert!(!parsed.restored.every_read);
    // The no-field hint (#2112).
    let parsed =
        attrs("#[externalptr(field_error(no_field_hint = \"try `y$%s`\"))] struct E;").unwrap();
    assert_eq!(
        parsed.field_error.no_field_hint.as_deref(),
        Some("try `y$%s`")
    );
    assert_eq!(parsed.field_error.message, None);

    let refused = |source: &str, expected: &str| {
        let message = attrs(source)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"));
        assert!(message.contains(expected), "`{source}`: {message}");
    };
    refused(
        "#[externalptr(restored(refuse = \"names\"))] struct E;",
        "`refuse` takes \"every_read\"",
    );
    refused(
        "#[externalptr(restored(refuse = \"every_read\", refuse = \"every_read\"))] struct E;",
        "given twice",
    );
    refused(
        "#[externalptr(field_error(class = []))] struct E;",
        "at least one",
    );
    refused(
        "#[externalptr(field_error(text = \"m\"))] struct E;",
        "unknown `field_error` option",
    );
    refused(
        "#[externalptr(field_error(message = \"\"))] struct E;",
        "`message` is empty",
    );
    refused(
        "#[externalptr(field_error(message = \"a\", message = \"b\"))] struct E;",
        "given twice",
    );
    refused(
        "#[externalptr(field_error(no_field_hint = \"\"))] struct E;",
        "`no_field_hint` is empty",
    );
    refused(
        "#[externalptr(field_error(no_field_hint = \"a\", no_field_hint = \"b\"))] struct E;",
        "given twice",
    );
    refused(
        "#[externalptr(field_error(class = \"a\"), field_error(class = \"b\"))] struct E;",
        "given twice",
    );

    let source = "#[externalptr(s3, field_error(class = [\"pkg_config\", \"pkg_error\"]), restored(refuse = \"every_read\"))] \
                      struct Engine { #[r_data] r: RSidecar, #[r_data] pub keys: Sidecar<Vec<i32>> }";
    let out = super::derive_external_ptr(syn::parse_str(source).unwrap(), true)
        .unwrap()
        .to_string();
    assert!(
        out.contains(
            "const FIELD_ERROR_CLASS : & 'static [& 'static str] = & [\"pkg_config\" , \"pkg_error\"] ;"
        ),
        "{out}"
    );
    let live = crate::naming::sidecar_live_c_name("Engine");
    assert!(out.contains(&format!("fn {live}")), "{out}");
    assert!(out.contains("sidecar_r_live :: < Engine >"), "{out}");
    assert!(
        out.contains(".rdata_error_class_Engine <- c(\\\"pkg_config\\\", \\\"pkg_error\\\")"),
        "{out}"
    );
    assert!(
        out.contains(&format!(
            ".rdata_live_Engine <- function(x, raise) .Call({live}, x, raise)"
        )),
        "{out}"
    );
    assert!(out.contains(".rdata_every_read_Engine <- TRUE"), "{out}");

    let out = super::derive_external_ptr(
        syn::parse_str(
            "struct Engine { #[r_data] r: RSidecar, #[r_data] pub keys: Sidecar<Vec<i32>> }",
        )
        .unwrap(),
        true,
    )
    .unwrap()
    .to_string();
    assert!(!out.contains("FIELD_ERROR_CLASS"), "{out}");
    assert!(!out.contains("mx_rdata_live_"), "{out}");
    assert!(out.contains(".rdata_error_class_Engine <- NULL"), "{out}");
    assert!(out.contains(".rdata_every_read_Engine <- FALSE"), "{out}");
    assert!(
        out.contains(".rdata_live_Engine <- function(x, raise) TRUE"),
        "{out}"
    );
}

/// The standalone accessors go to a `SidecarAccessorsEntry` in two
/// renderings (#1900): exported on the type's page, and internal.
#[test]
fn standalone_accessors_come_exported_and_internal() {
    for exported in [true, false] {
        let out = generate_r_wrapper_for_slot(
            ClassSystem::S3,
            "T",
            "f",
            "get_f",
            Some("set_f"),
            true,
            exported,
        );
        assert!(out.contains("T_get_f <- function(x)"), "{out}");
        assert!(out.contains("T_set_f <- function(x, value)"), "{out}");
        assert_eq!(out.contains("@export"), exported, "{out}");
        assert_eq!(out.contains("@rdname T"), exported, "{out}");
        assert_eq!(
            out.matches("#' @noRd").count(),
            if exported { 0 } else { 2 },
            "{out}"
        );
    }
    let out = super::derive_external_ptr(
        syn::parse_str("struct T { #[r_data] r: RSidecar, #[r_data] pub f: i32 }").unwrap(),
        true,
    )
    .unwrap()
    .to_string();
    assert!(out.contains("MX_SIDECAR_ACCESSORS"), "{out}");
    assert!(out.contains("R_SIDECAR_ACCESSORS_T"), "{out}");
}

#[test]
fn field_setter_visibility_reaches_standalone_and_integrated_wrappers() {
    for (attribute, explicit) in [
        ("", None),
        ("setter = \"visible\"", Some(false)),
        ("setter = \"invisible\"", Some(true)),
    ] {
        let input: syn::DeriveInput = syn::parse_str(&format!(
            "struct Example {{ #[r_data({attribute})] pub value: i32 }}"
        ))
        .unwrap();
        for system in ALL_CLASS_SYSTEMS {
            let info = super::parse_sidecar_info(&input, system).unwrap();
            let slot = &info.slots[0];
            assert_eq!(slot.setter_invisible, explicit);
            let standalone = generate_r_wrapper_for_slot(
                system,
                "Example",
                "value",
                "get_value",
                Some("set_value"),
                slot.setter_invisible.unwrap_or(true),
                true,
            );
            assert_eq!(
                standalone.contains("invisible(x)"),
                explicit.unwrap_or(true)
            );
            assert!(standalone.contains(if explicit == Some(false) {
                "pointer (visibly)"
            } else {
                "pointer (invisibly)"
            }));
            let integrated =
                generate_class_integration_r_code(system, "Example", &[slot], &Default::default());
            match system {
                ClassSystem::R6 => assert_eq!(
                    integrated.contains("invisible(self)"),
                    explicit.unwrap_or(true)
                ),
                ClassSystem::S7 => assert_eq!(
                    integrated.contains("invisible(self)"),
                    explicit.unwrap_or(false)
                ),
                _ => assert!(integrated.is_empty()),
            }
        }
    }
}

#[test]
fn setter_visibility_and_property_docs_can_share_or_split_attributes() {
    for attrs in [
        "#[r_data(prop_doc = \"A counter\", setter = \"visible\")]",
        "#[r_data] #[r_data(setter = \"visible\", prop_doc = \"A counter\")]",
    ] {
        let input: syn::DeriveInput =
            syn::parse_str(&format!("struct Example {{ {attrs} pub value: i32 }}")).unwrap();
        let info = super::parse_sidecar_info(&input, ClassSystem::S7).unwrap();
        assert_eq!(info.slots[0].prop_doc.as_deref(), Some("A counter"));
        assert_eq!(info.slots[0].setter_invisible, Some(false));
    }
}

/// `Sidecar` fields count among themselves: the scalars and conversion
/// fields between them take no protection-list position. The derive
/// emits the count, the visitor hook, and typed accessors on the handle
/// with each field's visibility; the R accessors read and write the
/// protection list by index.
#[test]
fn sidecar_fields_number_their_own_positions() {
    let input: syn::DeriveInput = syn::parse_str(
        "struct Engine { #[r_data] _r: RSidecar, #[r_data] pub a: i32, \
             #[r_data] pub keys: Sidecar<Vec<i32>>, #[r_data] pub c: String, \
             #[r_data] steps: Sidecar<Option<List>> }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::Env).unwrap();
    let kinds: Vec<_> = info.slots.iter().map(|slot| slot.kind.clone()).collect();
    assert_eq!(
        kinds,
        [
            super::SlotKind::ScalarInt,
            super::SlotKind::Sidecar(0),
            super::SlotKind::Conversion,
            super::SlotKind::Sidecar(1),
        ]
    );
    // A bare `#[r_data]` means `ref, mut`.
    assert_eq!(
        info.slots[1].access,
        super::SidecarAccess {
            get: true,
            set: true
        }
    );

    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(
        out.contains("const R_SLOT_COUNT : usize = 2usize ;"),
        "{out}"
    );
    assert!(
        out.contains("visit (0usize , \"keys\" , & self . keys) ; visit (1usize , \"steps\" , & self . steps) ;"),
        "{out}"
    );
    // The accessors are associated functions taking the handle, not
    // methods on the struct (#1856).
    assert!(
        out.contains(
            "pub fn keys (ptr : & impl :: miniextendr_api :: externalptr :: SidecarOwner < Self >) -> Vec < i32 > { :: miniextendr_api :: externalptr :: sidecar_get :: < Self , Vec < i32 > > (ptr , 0usize , \"keys\" , | this : & Self | & this . keys ,) }"
        ),
        "{out}"
    );
    assert!(
        out.contains(
            "pub fn set_keys (ptr : & mut impl :: miniextendr_api :: externalptr :: SidecarOwnerMut < Self > , value : Vec < i32 >) { :: miniextendr_api :: externalptr :: sidecar_set :: < Self , Vec < i32 > > (ptr , 0usize , \"keys\" , | this : & Self | & this . keys , value ,) }"
        ),
        "{out}"
    );
    assert!(!out.contains("fn keys (& self)"), "{out}");
    // A private field gets private accessors (allowed to be unused) and
    // no R accessor.
    assert!(
        out.contains(
            "# [allow (dead_code)] fn steps (ptr : & impl :: miniextendr_api :: externalptr :: SidecarOwner < Self >) -> Option < List >"
        ),
        "{out}"
    );
    assert!(
        out.contains(
            "# [allow (dead_code)] fn set_steps (ptr : & mut impl :: miniextendr_api :: externalptr :: SidecarOwnerMut < Self > , value : Option < List >)"
        ),
        "{out}"
    );
    assert!(out.contains("(ptr , 1usize , \"steps\" ,"), "{out}");
    assert!(!out.contains("get_steps"), "{out}");

    // The R accessors read and write the protection list, not the struct.
    let keys = &info.slots[1];
    let name = syn::Ident::new("Engine", proc_macro2::Span::call_site());
    let getter = super::generate_getter_body(&name, keys).to_string();
    // The `require_live` flag of the `.Call()` decides whether a restored
    // pointer is read or refused (#1891).
    assert!(
        getter.contains(
            "sidecar_r_get :: < Engine > (x , 0usize , :: miniextendr_api :: externalptr :: sidecar_require_live (require_live) ,)"
        ),
        "{getter}"
    );
    assert!(!getter.contains("R_ExternalPtrAddr"), "{getter}");
    let setter = super::generate_setter_body(&name, keys).to_string();
    assert!(
        setter.contains("sidecar_r_check :: < Engine > (x , require_live)"),
        "{setter}"
    );
    // The value is validated as the field's `T` and reported as it.
    assert!(
        setter.contains("< Vec < i32 > as TryFromSexp > :: try_from_sexp (value)"),
        "{setter}"
    );
    assert!(setter.contains("\"'keys' must be integer\""), "{setter}");
    assert!(setter.contains("Some (\"Vec<i32>\")"), "{setter}");
    assert!(
        setter.contains("sidecar_r_set :: < Engine > (x , 0usize , value , require_live ,)"),
        "{setter}"
    );
}

/// `ref` generates only the getter, `mut` only the setter.
#[test]
fn sidecar_access_options_select_the_accessors() {
    let input: syn::DeriveInput = syn::parse_str(
        "struct Engine { #[r_data(ref)] pub keys: Sidecar<Vec<i32>>, \
             #[r_data(mut)] pub(crate) note: Sidecar<String> }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::Env).unwrap();
    assert_eq!(
        info.slots[0].access,
        super::SidecarAccess {
            get: true,
            set: false
        }
    );
    assert_eq!(
        info.slots[1].access,
        super::SidecarAccess {
            get: false,
            set: true
        }
    );
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(
        out.contains(
            "pub fn keys (ptr : & impl :: miniextendr_api :: externalptr :: SidecarOwner < Self >) -> Vec < i32 >"
        ),
        "{out}"
    );
    assert!(!out.contains("fn set_keys"), "{out}");
    assert!(
        out.contains(
            "# [allow (dead_code)] pub (crate) fn set_note (ptr : & mut impl :: miniextendr_api :: externalptr :: SidecarOwnerMut < Self > , value : String)"
        ),
        "{out}"
    );
    assert!(!out.contains("fn note ("), "{out}");
    // Both options together, in either order, and the hook still lists
    // every field.
    let input: syn::DeriveInput =
        syn::parse_str("struct Engine { #[r_data(mut, ref)] pub keys: Sidecar<Vec<i32>> }")
            .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::Env).unwrap();
    assert_eq!(
        info.slots[0].access,
        super::SidecarAccess {
            get: true,
            set: true
        }
    );
}

/// `ref` / `mut` are refused on a plain struct field, a `Sidecar` field
/// needs `#[r_data]`, and a duplicate option is an error.
#[test]
fn sidecar_access_options_are_checked() {
    let refused = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_sidecar_info(&input, ClassSystem::Env)
            .err()
            .map(|err| err.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"))
    };
    let message = refused("struct E { #[r_data(ref)] pub count: i32 }");
    assert!(
        message.contains("apply to `Sidecar<T>` fields only"),
        "{message}"
    );
    let message = refused("struct E { pub keys: Sidecar<Vec<i32>> }");
    assert!(message.contains("needs `#[r_data]`"), "{message}");
    let message = refused("struct E { #[r_data(ref, ref)] pub keys: Sidecar<Vec<i32>> }");
    assert!(message.contains("duplicate `ref`"), "{message}");
    let message = refused("struct E { #[r_data] pub keys: Sidecar }");
    assert!(message.contains("takes one type argument"), "{message}");
}

/// A struct without `Sidecar` fields keeps the trait's default count and
/// hook, and gets no accessors.
/// The helpers of the S3 / S4 / env field methods (#1848): the field
/// names, getter and setter `switch()`es over the accessors' C symbols
/// (a raw identifier's R name backtick-quoted), and the classed error.
#[test]
fn field_syntax_helpers_switch_over_the_accessors() {
    let slot = |name: &str| super::SidecarSlot {
        name: syn::parse_str(name).unwrap(),
        r_name: crate::naming::ident_name(&syn::parse_str::<syn::Ident>(name).unwrap()),
        ty: syn::parse_quote!(i32),
        vis: syn::parse_quote!(pub),
        is_public: true,
        kind: super::SlotKind::ScalarInt,
        access: super::SidecarAccess::default(),
        prop_doc: None,
        setter_invisible: None,
        read_only: false,
        null_if_none: false,
        cache: None,
    };
    let (keys, reserved) = (slot("keys"), slot("r#for"));
    let out = super::generate_field_syntax_helpers(
        "Engine",
        &[&keys, &reserved],
        &Default::default(),
        false,
    );
    assert!(
        out.contains(".rdata_fields_Engine <- c(\"keys\", \"for\")\n"),
        "{out}"
    );
    let getter = crate::naming::sidecar_getter_c_name("Engine", "keys");
    let setter = crate::naming::sidecar_setter_c_name("Engine", "for");
    assert!(
        out.contains(&format!(
            ".rdata_get_Engine <- function(x, name) switch(name,\n  keys = .Call({getter}, x, TRUE),"
        )),
        "{out}"
    );
    assert!(
        out.contains(&format!("  `for` = .Call({setter}, x, value, TRUE))\n")),
        "{out}"
    );
    assert!(
        out.contains(
            "function(name, env, cls) stop(errorCondition(\n  sprintf(\"`%s` is not a field of `%s`; its fields are `keys`, `for`\", name, cls),"
        ),
        "{out}"
    );
    assert!(
        out.contains("class = c(.rdata_error_class_Engine, \"miniextendr_no_field\"), call = .miniextendr_frame_call(env)"),
        "{out}"
    );

    // `no_field_hint` (#2112): the hint follows the field list, each `%s`
    // the name asked for; any other `%`, a quote or a backslash in it is
    // written as given.
    let out = super::generate_field_syntax_helpers(
        "Engine",
        &[&keys],
        &super::FieldErrorOptions {
            no_field_hint: Some("See `t$%s` or \"u\\%s\" (50% off).".to_string()),
            ..Default::default()
        },
        false,
    );
    assert!(
        out.contains(
            "  paste0(sprintf(\"`%s` is not a field of `%s`; its fields are `keys`\", name, cls), \". \", \
                 \"See `t$\", name, \"` or \\\"u\\\\\", name, \"\\\" (50% off).\"),\n"
        ),
        "{out}"
    );
}

/// A read-only field (#2268): no setter C function, `CallDef` or
/// `Type_set_<f>()`, a setter arm that calls `.rdata_read_only_<Type>`, and
/// the helper itself with the `field_error` classes and the
/// `read_only_hint` sentence, each `%s` the field's name. The Rust accessors
/// of a `Sidecar` field stay.
#[test]
fn read_only_fields_get_a_getter_and_a_refusing_setter_arm() {
    let input: syn::DeriveInput = syn::parse_str(
        "#[externalptr(s3, field_error(class = \"pkg_error\", read_only_hint = \"Use `set_%s()` (100% Rust).\"))] \
         struct Engine { #[r_data] _r: RSidecar, \
             #[r_data(read_only)] pub level: i32, \
             #[r_data(mut, read_only, name = \"keys\")] pub r_keys: Sidecar<Vec<i32>>, \
             #[r_data(get = \"Self::n\")] pub n: Computed, \
             #[r_data] pub base: i32 }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::S3).unwrap();
    let setters: Vec<_> = info.slots.iter().map(|slot| slot.has_r_setter()).collect();
    assert_eq!(setters, [false, false, false, true]);
    assert!(info.slots[0].read_only && info.slots[1].read_only);

    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    for field in ["level", "r_keys", "n"] {
        assert!(
            !out.contains(&crate::naming::sidecar_setter_c_name("Engine", field)),
            "{field}: {out}"
        );
        assert!(
            !out.contains(&format!("__MX_CALL_DEF_RDATA_SET_Engine_{field}")),
            "{field}: {out}"
        );
    }
    assert!(!out.contains("Engine_set_level"), "{out}");
    assert!(!out.contains("Engine_set_keys"), "{out}");
    assert!(out.contains("Engine_get_level <- function(x)"), "{out}");
    assert!(out.contains("Engine_get_keys <- function(x)"), "{out}");
    assert!(
        out.contains("Engine_set_base <- function(x, value)"),
        "{out}"
    );
    assert!(
        out.contains("# Wraps read-only sidecar field `level` on Rust type `Engine` via `"),
        "{out}"
    );
    // `mut` keeps the Rust setter of a read-only `Sidecar` field: read-only
    // applies to R only.
    assert!(out.contains("pub fn set_r_keys (ptr :"), "{out}");
    assert!(!out.contains("pub fn r_keys (ptr :"), "{out}");

    let pub_slots: Vec<_> = info.slots.iter().collect();
    let options = super::FieldErrorOptions {
        class: vec!["pkg_error".to_string()],
        read_only_hint: Some("Use `set_%s()` (100% Rust).".to_string()),
        ..Default::default()
    };
    let helpers = super::generate_field_syntax_helpers("Engine", &pub_slots, &options, false);
    assert!(
        helpers.contains(
            ".rdata_set_Engine <- function(x, name, value) switch(name,\n  \
             level = .rdata_read_only_Engine(\"level\", x, parent.frame()),\n  \
             keys = .rdata_read_only_Engine(\"keys\", x, parent.frame()),\n  \
             n = .rdata_read_only_Engine(\"n\", x, parent.frame(), \"is computed from the Rust value \
             and can't be assigned\"),\n"
        ),
        "{helpers}"
    );
    assert!(
        helpers.contains(&format!(
            "  base = .Call({}, x, value, TRUE))\n",
            crate::naming::sidecar_setter_c_name("Engine", "base")
        )),
        "{helpers}"
    );
    assert!(
        helpers.contains(
            ".rdata_read_only_Engine <- function(name, x, env, why = \"is read-only\") stop(errorCondition(\n  \
             paste0(sprintf(\"`%s` of `%s` %s\", name, if (is.null(oldClass(x))) \"Engine\" else \
             oldClass(x)[[1L]], why), \". \", \"Use `set_\", name, \"()` (100% Rust).\"),\n  \
             class = c(.rdata_error_class_Engine, \"miniextendr_read_only_field\"), \
             call = .miniextendr_frame_call(env)))\n"
        ),
        "{helpers}"
    );
    assert!(
        helpers.contains(".rdata_error_class_Engine <- c(\"pkg_error\")\n"),
        "{helpers}"
    );

    // Without a hint the message is the sentence alone.
    let helpers =
        super::generate_field_syntax_helpers("Engine", &pub_slots, &Default::default(), false);
    assert!(
        helpers.contains(
            "  sprintf(\"`%s` of `%s` %s\", name, if (is.null(oldClass(x))) \"Engine\" else \
             oldClass(x)[[1L]], why),\n"
        ),
        "{helpers}"
    );
}

/// The standalone accessors come in a third rendering without the setters,
/// which the wrapper writer picks for `r_data_accessors = "read_only"`
/// (#2268): there a writable field's standalone setter is not written
/// either. Its setter C entry point stays, for the other renderings.
#[test]
fn standalone_accessors_come_without_setters_for_the_read_only_form() {
    let input: syn::DeriveInput = syn::parse_str(
        "#[externalptr(s3)] struct Engine { #[r_data] _r: RSidecar, \
             #[r_data(read_only)] pub level: i32, #[r_data] pub base: i32 }",
    )
    .unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(out.contains("internal_read_only :"), "{out}");
    // Exported, internal and read-only renderings each read both fields;
    // only the first two write `base`.
    assert_eq!(out.matches("Engine_get_base <- function(x)").count(), 3);
    assert_eq!(out.matches("Engine_get_level <- function(x)").count(), 3);
    assert_eq!(
        out.matches("Engine_set_base <- function(x, value)").count(),
        2
    );
    assert!(!out.contains("Engine_set_level"), "{out}");
    assert_eq!(
        out.matches("# Wraps read-only sidecar field `base`")
            .count(),
        1
    );
    assert!(
        out.contains(&crate::naming::sidecar_setter_c_name("Engine", "base")),
        "{out}"
    );
    // Both internal renderings carry the field-syntax helpers.
    assert_eq!(
        out.matches(".rdata_read_only_Engine <- function(").count(),
        2
    );
}

/// An R6 binding of a field without an R setter refuses a value with the
/// package's classes and hint and no call, and an S7 property of one has no
/// setter (#1888, #2268); a writable field keeps its setter in both.
#[test]
fn read_only_fields_get_refusing_bindings_and_setterless_properties() {
    let input: syn::DeriveInput = syn::parse_str(
        "struct Engine { #[r_data] _r: RSidecar, \
             #[r_data(read_only)] pub level: i32, \
             #[r_data(get = \"Self::n\")] pub n: Computed, \
             #[r_data] pub base: i32 }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::R6).unwrap();
    let pub_slots: Vec<_> = info.slots.iter().collect();
    let options = super::FieldErrorOptions {
        class: vec!["pkg_error_field".to_string(), "pkg_error".to_string()],
        read_only_hint: Some("Set it with `x$set_%s()`.".to_string()),
        ..Default::default()
    };
    let r6 = generate_class_integration_r_code(ClassSystem::R6, "Engine", &pub_slots, &options);
    assert!(
        r6.contains(
            "      stop(errorCondition(paste0(sprintf(\"`level` of `%s` is read-only\", class(self)[[1L]]), \
             \". \", \"Set it with `x$set_level()`.\"), class = c(\"pkg_error_field\", \"pkg_error\", \
             \"miniextendr_read_only_field\"), call = NULL))\n"
        ),
        "{r6}"
    );
    assert!(
        r6.contains(
            "      stop(errorCondition(paste0(sprintf(\"`n` of `%s` is computed from the Rust value and \
             can't be assigned\", class(self)[[1L]]), \". \", \"Set it with `x$set_n()`.\"), class = \
             c(\"pkg_error_field\", \"pkg_error\", \"miniextendr_read_only_field\"), call = NULL))\n"
        ),
        "{r6}"
    );
    for field in ["level", "n"] {
        assert!(
            !r6.contains(&crate::naming::sidecar_setter_c_name("Engine", field)),
            "{field}: {r6}"
        );
    }
    assert!(
        r6.contains(&crate::naming::sidecar_setter_c_name("Engine", "base")),
        "{r6}"
    );
    // Without options: miniextendr's class alone, no hint.
    let r6 = generate_class_integration_r_code(
        ClassSystem::R6,
        "Engine",
        &pub_slots,
        &Default::default(),
    );
    assert!(
        r6.contains(
            "stop(errorCondition(sprintf(\"`level` of `%s` is read-only\", class(self)[[1L]]), \
             class = c(\"miniextendr_read_only_field\"), call = NULL))"
        ),
        "{r6}"
    );

    let s7 = generate_class_integration_r_code(ClassSystem::S7, "Engine", &pub_slots, &options);
    assert_eq!(s7.matches("getter = function(self)").count(), 3, "{s7}");
    assert_eq!(
        s7.matches("setter = function(self, value)").count(),
        1,
        "{s7}"
    );
    let level_getter = crate::naming::sidecar_getter_c_name("Engine", "level");
    assert!(
        s7.contains(&format!(
            "    level = S7::new_property(\n        getter = function(self) {{\n          \
             .val <- .Call({level_getter}, self@.ptr, FALSE)\n"
        )),
        "{s7}"
    );
    assert!(s7.contains("          .val\n        }\n    ),\n"), "{s7}");

    // The S7 `@prop` entries say which properties have no setter, so the
    // wrapper writer takes no `Engine_set_level` name for them.
    let input: syn::DeriveInput = syn::parse_str(
        "#[externalptr(s7)] struct Engine { #[r_data] _r: RSidecar, \
             #[r_data(read_only)] pub level: i32, #[r_data] pub base: i32 }",
    )
    .unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(
        out.contains("field_name : \"level\" , prop_doc : \"(undocumented sidecar property)\" , read_only : true"),
        "{out}"
    );
    assert!(
        out.contains("field_name : \"base\" , prop_doc : \"(undocumented sidecar property)\" , read_only : false"),
        "{out}"
    );
}

/// The `read_only` option is checked: it takes no value, is given once, and
/// applies to a public field R can write, so not to a `Computed` field
/// (read-only already), the selector, a private field or one with `setter`.
/// It goes with `ref` / `mut`, which select the Rust accessors.
#[test]
fn read_only_option_is_checked() {
    let refused = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_sidecar_info(&input, ClassSystem::S3)
            .err()
            .map(|err| err.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"))
    };
    let message = refused("struct E { #[r_data(read_only, read_only)] pub count: i32 }");
    assert!(message.contains("duplicate `read_only`"), "{message}");
    let message = refused("struct E { #[r_data(read_only = \"yes\")] pub count: i32 }");
    assert!(message.contains("`read_only` takes no value"), "{message}");
    let message = refused("struct E { #[r_data(readonly)] pub count: i32 }");
    assert!(message.contains("unknown key `readonly`"), "{message}");
    let message =
        refused("struct E { #[r_data(read_only, get = \"Self::f\")] pub count: Computed }");
    assert!(message.contains("read-only already"), "{message}");
    let message = refused("struct E { #[r_data(read_only)] _r: RSidecar }");
    assert!(message.contains("the RSidecar selector"), "{message}");
    let message = refused("struct E { #[r_data(read_only)] count: i32 }");
    assert!(message.contains("a field that is not `pub`"), "{message}");
    let message = refused("struct E { #[r_data(read_only, setter = \"visible\")] pub count: i32 }");
    assert!(message.contains("a field with `setter = ...`"), "{message}");

    let input: syn::DeriveInput = syn::parse_str(
        "struct E { #[r_data(ref, read_only)] pub a: Sidecar<i32>, \
             #[r_data(read_only, mut)] pub b: Sidecar<i32>, \
             #[r_data(read_only, name = \"cc\", prop_doc = \"C.\")] pub c: String }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::S7).unwrap();
    assert!(info.slots.iter().all(|slot| slot.read_only));
    assert_eq!(
        info.slots[0].access,
        super::SidecarAccess {
            get: true,
            set: false
        }
    );
    assert_eq!(
        info.slots[1].access,
        super::SidecarAccess {
            get: false,
            set: true
        }
    );
}

/// `#[r_data(null_if_none)]` (#2270): the getter of an `Option<T>` field
/// converts it as a `Nullable<T>`, so `None` reads `NULL`; the setter is the
/// plain `Option<T>` one. Without it an `Option<T>` field converts as itself
/// (`NA` for a scalar), and a `Nullable<T>` field reads `NULL` by its own
/// conversion. It goes with `read_only`. Its S7 property takes no `class`, so
/// S7's `class_any` admits `NULL`.
#[test]
fn null_if_none_fields_read_null_for_none() {
    let input: syn::DeriveInput = syn::parse_str(
        "#[externalptr(s7)] struct Engine { #[r_data] _r: RSidecar, \
             #[r_data(null_if_none)] pub label: Option<String>, \
             #[r_data] pub plain: Option<String>, \
             #[r_data] pub tag: Nullable<String>, \
             #[r_data(null_if_none, read_only, name = \"code.id\")] pub code: Option<i32> }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::S7).unwrap();
    let flags: Vec<_> = info.slots.iter().map(|slot| slot.null_if_none).collect();
    assert_eq!(flags, [true, false, false, true]);
    assert!(
        info.slots
            .iter()
            .all(|slot| slot.kind == super::SlotKind::Conversion)
    );
    let setters: Vec<_> = info.slots.iter().map(|slot| slot.has_r_setter()).collect();
    assert_eq!(setters, [true, true, true, false]);

    let struct_name = syn::parse_quote!(Engine);
    let wrap = "let val = :: miniextendr_api :: Nullable (val) ;";
    let getter = |i: usize| super::generate_getter_body(&struct_name, &info.slots[i]).to_string();
    assert!(getter(0).contains(wrap), "{}", getter(0));
    assert!(
        getter(0).contains("let val : Option < String > ="),
        "{}",
        getter(0)
    );
    assert!(!getter(1).contains("Nullable"), "{}", getter(1));
    assert!(!getter(2).contains(wrap), "{}", getter(2));
    assert!(getter(3).contains(wrap), "{}", getter(3));

    // The setter reads `NULL` and `NA` as `None` through `Option<T>`'s own
    // conversion, as a plain `Option<T>` field's does.
    let setter = super::generate_setter_body(&struct_name, &info.slots[0]).to_string();
    let plain = super::generate_setter_body(&struct_name, &info.slots[1]).to_string();
    assert_eq!(setter.replace("label", "plain"), plain);

    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert_eq!(out.matches(wrap).count(), 2, "{out}");
    assert!(out.contains("Engine_get_code.id <- function(x)"), "{out}");
    assert!(!out.contains("Engine_set_code.id"), "{out}");
    assert!(
        out.contains("Engine_set_label <- function(x, value)"),
        "{out}"
    );

    // S7: no property is given a class, so `class_any` admits `NULL` and a
    // `NULL` write passes S7's validation.
    let pub_slots: Vec<_> = info.slots.iter().collect();
    let s7 = generate_class_integration_r_code(
        ClassSystem::S7,
        "Engine",
        &pub_slots,
        &Default::default(),
    );
    assert!(s7.contains("    label = S7::new_property(\n"), "{s7}");
    assert!(s7.contains("    code.id = S7::new_property(\n"), "{s7}");
    assert!(!s7.contains("class ="), "{s7}");
    assert_eq!(
        s7.matches("setter = function(self, value)").count(),
        3,
        "{s7}"
    );
}

/// The `null_if_none` option is checked: it takes no value, is given once,
/// and applies to a public `Option<T>` struct field only: not to the
/// selector, a `Computed` or `Sidecar` field (each named with its own
/// `NULL` spelling), a private field, a `Nullable<T>` field (`NULL`
/// already) or another type. It goes with `read_only`, `name`, `setter` and
/// `prop_doc`.
#[test]
fn null_if_none_option_is_checked() {
    let refused = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_sidecar_info(&input, ClassSystem::S3)
            .err()
            .map(|err| err.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"))
    };
    let message =
        refused("struct E { #[r_data(null_if_none, null_if_none)] pub a: Option<String> }");
    assert!(message.contains("duplicate `null_if_none`"), "{message}");
    let message = refused("struct E { #[r_data(null_if_none = \"yes\")] pub a: Option<String> }");
    assert!(
        message.contains("`null_if_none` takes no value"),
        "{message}"
    );
    let message = refused("struct E { #[r_data(nullifnone)] pub a: Option<String> }");
    assert!(
        message.contains(
            "unknown key `nullifnone`; supported: `ref`, `mut`, `read_only`, `null_if_none`,"
        ),
        "{message}"
    );
    for (source, why) in [
        (
            "struct E { #[r_data(null_if_none)] _r: RSidecar }",
            "the RSidecar selector",
        ),
        (
            "struct E { #[r_data(null_if_none, get = \"Self::f\")] pub a: Computed }",
            "its `get` function returns `Nullable<T>`",
        ),
        (
            "struct E { #[r_data(null_if_none)] pub a: Sidecar<Option<String>> }",
            "declare it `Sidecar<Nullable<T>>`",
        ),
        (
            "struct E { #[r_data(null_if_none)] a: Option<String> }",
            "a field that is not `pub`",
        ),
        (
            "struct E { #[r_data(null_if_none)] pub a: Nullable<String> }",
            "reads `NULL` for `None` already",
        ),
        (
            "struct E { #[r_data(null_if_none)] pub a: String }",
            "a `String` field: only an `Option<T>` field has a `None`",
        ),
        (
            "struct E { #[r_data(null_if_none)] pub a: Vec<Option<i32>> }",
            "a `Vec<Option<i32>>` field",
        ),
    ] {
        let message = refused(source);
        assert!(
            message.contains(
                "`null_if_none` makes a public `Option<T>` `#[r_data]` field read `NULL`"
            ),
            "{message}"
        );
        assert!(message.contains(why), "{source}: {message}");
    }

    let input: syn::DeriveInput = syn::parse_str(
        "struct E { #[r_data] _r: RSidecar, \
             #[r_data(read_only, null_if_none)] pub a: Option<String>, \
             #[r_data(null_if_none, name = \"bb\", setter = \"visible\", prop_doc = \"B.\")] \
             pub b: ::std::option::Option<Vec<i32>>, \
             #[r_data] pub c: Sidecar<Nullable<String>> }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::S7).unwrap();
    assert!(info.slots[0].null_if_none && info.slots[0].read_only);
    assert!(info.slots[1].null_if_none && info.slots[1].r_name == "bb");
    assert_eq!(info.slots[2].kind, super::SlotKind::Sidecar(0));
}

/// The helpers come with the R accessors, whatever the class system; a
/// type without them gets none.
#[test]
fn field_syntax_helpers_follow_the_r_accessors() {
    for attr in ["", "#[externalptr(r6)]", "#[externalptr(s3)]"] {
        let input: syn::DeriveInput = syn::parse_str(&format!(
            "{attr} struct Engine {{ #[r_data] _r: RSidecar, #[r_data] pub count: i32 }}"
        ))
        .unwrap();
        let out = super::derive_external_ptr(input, true).unwrap().to_string();
        assert!(out.contains(".rdata_fields_Engine"), "{attr}: {out}");
    }
    let input: syn::DeriveInput =
        syn::parse_str("struct Engine { #[r_data] pub count: i32 }").unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(!out.contains(".rdata_"), "{out}");
}

#[test]
fn no_sidecar_fields_emit_no_count_or_hook() {
    let input: syn::DeriveInput =
        syn::parse_str("struct Plain { #[r_data] _r: RSidecar, #[r_data] pub a: i32 }").unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(!out.contains("R_SLOT_COUNT"), "{out}");
    assert!(!out.contains("__mx_visit_sidecars"), "{out}");
    assert!(!out.contains("sidecar_get"), "{out}");
}

#[test]
fn r_views_in_plain_fields_are_refused_recursively() {
    for ty in [
        "List",
        "ListMut",
        "NamedList",
        "DataFrame",
        "StrVec<'static>",
        "RCow<'static, i32>",
        "RArray<i32, 2>",
        "RVector<f64>",
        "RMatrix<i32>",
        "RArray3D<f64>",
        "Factor<'static>",
        "FactorMut<'static>",
        "::miniextendr_api::list::List",
        "Option<List>",
        "Vec<Option<DataFrame>>",
        "HashMap<String, Vec<List>>",
        "Result<Vec<i32>, SEXP>",
        "Box<[List]>",
        "[Option<List>; 2]",
        "(i32, Vec<NamedList>)",
        "&'static List",
        "&'static str",
        "Option<&'static i32>",
        "Box<&'static [f64]>",
        "Arc<Mutex<Option<List>>>",
        "(List)",
        "Nullable<List>",
        "Nullable<Vec<DataFrame>>",
    ] {
        let input = syn::parse_str(&format!("struct Bad {{ #[r_data] pub value: {ty} }}")).unwrap();
        let err = super::derive_external_ptr(input, true).unwrap_err();
        assert!(err.to_string().contains("is not rooted"), "{ty}: {err}");
        assert!(err.to_string().contains("Sidecar<"), "{ty}: {err}");
    }
}

#[test]
fn owned_fields_and_sidecars_keep_their_existing_classification() {
    for ty in [
        "String",
        "Vec<Option<i32>>",
        "HashMap<String, Vec<f64>>",
        "Preserved",
        "BuiltList",
        "BuiltDataFrame",
        "ProtectedStrVec",
        "Option<BuiltList>",
        "Cow<'static, str>",
        "PhantomData<List>",
        "Nullable<String>",
    ] {
        let ty = syn::parse_str(ty).unwrap();
        assert!(!super::contains_unrooted_r_view(&ty), "{ty:?}");
    }
    for ty in [
        "Sidecar<List>",
        "Sidecar<Option<List>>",
        "Sidecar<DataFrame>",
        "Sidecar<Nullable<List>>",
    ] {
        let ty = syn::parse_str(ty).unwrap();
        let kind = super::slot_kind_for_type(&ty, 2, None, &syn::parse_quote!(Host)).unwrap();
        assert!(matches!(kind, super::SlotKind::Sidecar(2)));
    }
}

/// A bare `SEXP` under `#[r_data]` is refused: nothing would root it.
#[test]
fn bare_sexp_sidecar_field_is_refused() {
    let input: syn::DeriveInput =
        syn::parse_str("struct Engine { #[r_data] _r: RSidecar, #[r_data] pub keys: SEXP }")
            .unwrap();
    let Err(err) = super::parse_sidecar_info(&input, ClassSystem::Env) else {
        panic!("a SEXP sidecar field must be refused");
    };
    let message = err.to_string();
    assert!(message.contains("is not rooted"), "{message}");
    assert!(
        message.contains("Declare the field as `Sidecar<SEXP>`"),
        "{message}"
    );
}

/// `#[r_data(cache)]` (#2271): the getter of a conversion field passes its
/// conversion to `sidecar_r_cached` under the field's cache index (counted
/// among the cached fields only), as a `Result` so a failed conversion's
/// condition is not kept; an uncached field converts on every read. The
/// type's `CACHED_FIELD_COUNT` counts the cached fields. `cache` goes with
/// `null_if_none` (the `Nullable` wrap moves into the conversion),
/// `read_only`, `name`, `setter` and `prop_doc`, and leaves the setter as it
/// is.
#[test]
fn cache_fields_keep_their_converted_value() {
    let input: syn::DeriveInput = syn::parse_str(
        "#[externalptr(s3)] struct Engine { #[r_data] _r: RSidecar, \
             #[r_data(cache)] pub values: Vec<f64>, \
             #[r_data] pub plain: Vec<f64>, \
             #[r_data] pub keys: Sidecar<Vec<i32>>, \
             #[r_data(cache, null_if_none, read_only, name = \"lbl\")] pub label: Option<String>, \
             #[r_data(cache, setter = \"visible\", prop_doc = \"N.\")] pub names: Vec<String> }",
    )
    .unwrap();
    let info = super::parse_sidecar_info(&input, ClassSystem::S3).unwrap();
    let caches: Vec<_> = info.slots.iter().map(|slot| slot.cache).collect();
    assert_eq!(caches, [Some(0), None, None, Some(1), Some(2)]);

    let struct_name = syn::parse_quote!(Engine);
    let getter = |i: usize| super::generate_getter_body(&struct_name, &info.slots[i]).to_string();
    let cached = ":: miniextendr_api :: externalptr :: sidecar_r_cached :: < Engine > (x , ";
    assert!(
        getter(0).contains(&format!("{cached}0usize ,")),
        "{}",
        getter(0)
    );
    assert!(
        getter(0).contains(":: core :: result :: Result :: Err (unsafe"),
        "{}",
        getter(0)
    );
    assert!(!getter(1).contains("sidecar_r_cached"), "{}", getter(1));
    assert!(
        getter(3).contains(&format!("{cached}1usize ,")),
        "{}",
        getter(3)
    );
    assert!(
        getter(3).contains("let val = :: miniextendr_api :: Nullable (val) ;"),
        "{}",
        getter(3)
    );
    assert!(
        getter(4).contains(&format!("{cached}2usize ,")),
        "{}",
        getter(4)
    );

    // The setter is the uncached field's: its exclusive borrow drops the
    // kept value (`sidecar_r_struct_mut`).
    let setter = super::generate_setter_body(&struct_name, &info.slots[0]).to_string();
    let plain = super::generate_setter_body(&struct_name, &info.slots[1]).to_string();
    assert_eq!(setter.replace("values", "plain"), plain);

    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(
        out.contains("const CACHED_FIELD_COUNT : usize = 3usize ;"),
        "{out}"
    );
    assert!(out.contains("Engine_get_lbl <- function(x)"), "{out}");
    assert!(!out.contains("Engine_set_lbl"), "{out}");

    // A type without a cached field keeps the trait's default.
    let input: syn::DeriveInput =
        syn::parse_str("struct Plain { #[r_data] _r: RSidecar, #[r_data] pub values: Vec<f64> }")
            .unwrap();
    let out = super::derive_external_ptr(input, true).unwrap().to_string();
    assert!(!out.contains("CACHED_FIELD_COUNT"), "{out}");
}

/// The `cache` option is checked: it takes no value, is given once, and
/// applies to a public struct field that converts on a read: not to the
/// selector, a `Computed` field (its function may read more than the
/// struct), a `Sidecar` field (it converts nothing), a private field or a
/// scalar field (a length-1 vector is as fast as a kept one).
#[test]
fn cache_option_is_checked() {
    let refused = |source: &str| {
        let input: syn::DeriveInput = syn::parse_str(source).unwrap();
        super::parse_sidecar_info(&input, ClassSystem::S3)
            .err()
            .map(|err| err.to_string())
            .unwrap_or_else(|| panic!("`{source}` must be refused"))
    };
    let message = refused("struct E { #[r_data(cache, cache)] pub a: Vec<f64> }");
    assert!(message.contains("duplicate `cache`"), "{message}");
    let message = refused("struct E { #[r_data(cache = \"yes\")] pub a: Vec<f64> }");
    assert!(message.contains("`cache` takes no value"), "{message}");
    let message = refused("struct E { #[r_data(cached)] pub a: Vec<f64> }");
    assert!(
        message.contains(
            "unknown key `cached`; supported: `ref`, `mut`, `read_only`, `null_if_none`, `cache`,"
        ),
        "{message}"
    );
    for (source, why) in [
        (
            "struct E { #[r_data(cache)] _r: RSidecar }",
            "the RSidecar selector",
        ),
        (
            "struct E { #[r_data(cache, get = \"Self::f\")] pub a: Computed }",
            "a `Computed` field, which runs its `get` function on every read",
        ),
        (
            "struct E { #[r_data(cache)] pub a: Sidecar<Vec<f64>> }",
            "a `Sidecar<T>` field, whose getter returns the R value the pointer keeps",
        ),
        (
            "struct E { #[r_data(cache)] a: Vec<f64> }",
            "a field that is not `pub`",
        ),
        (
            "struct E { #[r_data(cache)] pub a: i32 }",
            "a scalar `i32` field, whose getter makes a length-1 vector",
        ),
        (
            "struct E { #[r_data(cache)] pub a: f64 }",
            "a scalar `f64` field",
        ),
        (
            "struct E { #[r_data(cache)] pub a: bool }",
            "a scalar `bool` field",
        ),
        (
            "struct E { #[r_data(cache)] pub a: u8 }",
            "a scalar `u8` field",
        ),
    ] {
        let message = refused(source);
        assert!(
            message.contains(
                "`cache` keeps the R value a public `#[r_data]` struct field's getter converts"
            ),
            "{message}"
        );
        assert!(message.contains(why), "{source}: {message}");
    }
}
