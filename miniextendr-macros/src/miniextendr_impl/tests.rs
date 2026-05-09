use super::*;

// region: Helper function for parsing impl blocks

fn default_impl_attrs(class_system: ClassSystem) -> ImplAttrs {
    ImplAttrs {
        class_system,
        class_name: None,
        label: None,
        vctrs_attrs: VctrsAttrs::default(),
        r6_inherit: None,
        r6_portable: None,
        r6_cloneable: None,
        r6_lock_objects: None,
        r6_lock_class: None,
        s7_parent: None,
        s7_abstract: false,
        r_data_accessors: false,
        strict: false,
        internal: false,
        noexport: false,
        blanket: false,
    }
}

fn parse_impl(class_system: ClassSystem, code: syn::ItemImpl) -> ParsedImpl {
    let attrs = default_impl_attrs(class_system);
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

fn parse_impl_with_class_name(
    class_system: ClassSystem,
    class_name: &str,
    code: syn::ItemImpl,
) -> ParsedImpl {
    let mut attrs = default_impl_attrs(class_system);
    attrs.class_name = Some(class_name.to_string());
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

fn parse_impl_with_label(
    class_system: ClassSystem,
    label: &str,
    code: syn::ItemImpl,
) -> ParsedImpl {
    let mut attrs = default_impl_attrs(class_system);
    attrs.label = Some(label.to_string());
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}
// endregion

// region: Env class system tests

#[test]
fn env_wrappers_preserve_static_params() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl ReceiverCounter {
            pub fn new(initial: i32) -> Self {
                unimplemented!()
            }

            pub fn add(&self, amount: i32) -> i32 {
                amount
            }

            pub fn default_counter(step: i32) -> Self {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    assert!(wrapper.contains("ReceiverCounter$new <- function(initial)"));
    assert!(wrapper.contains("ReceiverCounter$add <- function(amount)"));
    assert!(wrapper.contains("ReceiverCounter$default_counter <- function(step)"));
}

#[test]
fn env_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn add(&mut self, n: i32) -> i32 { unimplemented!() }
            pub fn from_string(s: String) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // Verify class environment creation
    assert!(wrapper.contains("Counter <- new.env(parent = emptyenv())"));

    // Verify constructor
    assert!(wrapper.contains("Counter$new <- function(value)"));
    assert!(wrapper.contains(".Call(C_Counter__new"));
    assert!(wrapper.contains("class(self) <- \"Counter\""));

    // Verify instance methods
    assert!(wrapper.contains("Counter$get <- function()"));
    assert!(wrapper.contains("Counter$increment <- function()"));
    assert!(wrapper.contains("Counter$add <- function(n)"));
    assert!(wrapper.contains(".Call(C_Counter__get, .call = match.call(), self)"));
    assert!(wrapper.contains(".Call(C_Counter__increment, .call = match.call(), self)"));
    assert!(wrapper.contains(".Call(C_Counter__add, .call = match.call(), self, n)"));

    // Verify static methods
    assert!(wrapper.contains("Counter$from_string <- function(s)"));
    assert!(wrapper.contains(".Call(C_Counter__from_string, .call = match.call(), s)"));

    // Verify $ dispatch
    assert!(wrapper.contains("`$.Counter` <- function(self, name)"));
    assert!(wrapper.contains("`[[.Counter` <- `$.Counter`"));
}

#[test]
fn env_wrapper_with_custom_class_name() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyRustType {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl_with_class_name(ClassSystem::Env, "RCounter", item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    assert!(wrapper.contains("RCounter <- new.env(parent = emptyenv())"));
    assert!(wrapper.contains("RCounter$new <- function()"));
    assert!(wrapper.contains("class(self) <- \"RCounter\""));
}
// endregion

// region: R6 class system tests

#[test]
fn r6_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn from_value(v: i32) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Verify R6Class definition
    assert!(wrapper.contains("Counter <- R6::R6Class(\"Counter\","));

    // Verify public list
    assert!(wrapper.contains("public = list("));

    // Verify initialize with .ptr parameter (because from_value returns Self)
    assert!(wrapper.contains("initialize = function(value, .ptr = NULL)"));
    assert!(wrapper.contains("if (!is.null(.ptr))"));
    assert!(wrapper.contains("private$.ptr <- .ptr"));
    assert!(wrapper.contains(".val <- .Call(C_Counter__new"));
    assert!(wrapper.contains("private$.ptr <- .val"));

    // Verify public instance methods
    assert!(wrapper.contains("get = function()"));
    assert!(wrapper.contains("increment = function()"));
    assert!(wrapper.contains(".Call(C_Counter__get, .call = match.call(), private$.ptr)"));
    assert!(wrapper.contains(".Call(C_Counter__increment, .call = match.call(), private$.ptr)"));

    // Verify private list
    assert!(wrapper.contains("private = list("));
    assert!(wrapper.contains(".ptr = NULL"));

    // Verify class options
    assert!(wrapper.contains("lock_objects = TRUE"));
    assert!(wrapper.contains("lock_class = FALSE"));
    assert!(wrapper.contains("cloneable = FALSE"));

    // Verify static methods as separate functions
    assert!(wrapper.contains("Counter$from_value <- function(v)"));
    assert!(wrapper.contains(".Call(C_Counter__from_value, .call = match.call(), v)"));
}

#[test]
fn r6_wrapper_private_methods() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            fn internal_compute(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Public method in public list
    assert!(wrapper.contains("get = function()"));

    // Private method should be in private list
    assert!(wrapper.contains("internal_compute = function()"));
}

#[test]
fn r6_wrapper_roxygen_imports() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(wrapper.contains("@importFrom R6 R6Class"));
}

#[test]
fn r6_wrapper_inherit() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Child {
            pub fn new() -> Self { unimplemented!() }
            pub fn child_method(&self) -> i32 { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_inherit = Some("ParentClass".to_string());
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    // inherit = uses a placeholder; resolver replaces at cdylib write time
    assert!(
        wrapper
            .contains("Child <- R6::R6Class(\"Child\", inherit = .__MX_CLASS_REF_ParentClass__,")
    );
}

#[test]
fn r6_wrapper_cloneable_and_locks() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_cloneable = Some(true);
    attrs.r6_lock_objects = Some(false);
    attrs.r6_lock_class = Some(true);
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(wrapper.contains("cloneable = TRUE"));
    assert!(wrapper.contains("lock_objects = FALSE,"));
    assert!(wrapper.contains("lock_class = TRUE,"));
}

#[test]
fn r6_wrapper_non_portable() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_portable = Some(false);
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(wrapper.contains("portable = FALSE,"));
}

#[test]
fn r6_wrapper_defaults_unchanged() {
    // Verify that default R6 options match the old hardcoded values
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Defaults: lock_objects=TRUE, lock_class=FALSE, cloneable=FALSE
    assert!(wrapper.contains("lock_objects = TRUE,"));
    assert!(wrapper.contains("lock_class = FALSE,"));
    assert!(wrapper.contains("cloneable = FALSE"));
    // No inherit or portable=FALSE by default
    assert!(!wrapper.contains("inherit ="));
    assert!(!wrapper.contains("portable = FALSE"));
}

#[test]
fn r6_active_binding_noexport_emits_field_internal() {
    // `#[miniextendr(noexport)]` on an R6 active binding emits a minimal
    // `#' @field name (internal)` description. The roxygen2 8.0.0 NEWS claims
    // `@field name NULL` is the opt-out, but in practice `r6_resolve_fields`
    // still warns "Undocumented R6 active binding" for that form because
    // `expected` is introspected from the class definition and is not pruned
    // in sync with the NULL-description discard. A minimal real description
    // satisfies the warning and keeps the binding clearly marked as internal.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Sensor {
            pub fn new(v: f64, r: i32) -> Self { unimplemented!() }
            #[miniextendr(r6(active))]
            pub fn value(&self) -> f64 { unimplemented!() }
            #[miniextendr(r6(active), noexport)]
            pub fn raw_bytes(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Exported active binding: normal `@field name <description>` form.
    // (Default text when no doc comment provided is "Active binding.")
    assert!(
        wrapper.contains("#' @field value Active binding."),
        "exported active binding must have '@field value Active binding.'\n{}",
        wrapper
    );

    // Noexported active binding: minimal `(internal)` description.
    assert!(
        wrapper.contains("#' @field raw_bytes (internal)"),
        "noexported active binding must emit '@field raw_bytes (internal)'\n{}",
        wrapper
    );

    // Must NOT emit "Active binding." for the noexported binding, and must NOT
    // emit the old `NULL`-opt-out form.
    assert!(
        !wrapper.contains("#' @field raw_bytes Active binding."),
        "noexported active binding must not have regular description\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field raw_bytes NULL"),
        "noexported active binding must not use the (broken) NULL opt-out\n{}",
        wrapper
    );
}

#[test]
fn r6_active_binding_internal_emits_field_internal() {
    // `#[miniextendr(internal)]` on an R6 active binding emits the same
    // `#' @field name (internal)` form as `noexport`.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Device {
            pub fn new() -> Self { unimplemented!() }
            #[miniextendr(r6(active))]
            pub fn status(&self) -> i32 { unimplemented!() }
            #[miniextendr(r6(active), internal)]
            pub fn debug_ptr(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(
        wrapper.contains("#' @field debug_ptr (internal)"),
        "internal active binding must emit '@field debug_ptr (internal)'\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field debug_ptr Active binding."),
        "internal active binding must not have regular description\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field debug_ptr NULL"),
        "internal active binding must not use the (broken) NULL opt-out\n{}",
        wrapper
    );
}
// endregion

// region: S3 class system tests

#[test]
fn s3_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn zero() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let wrapper = generate_s3_r_wrapper(&parsed);

    // Verify constructor (lowercase convention)
    assert!(wrapper.contains("new_counter <- function(value)"));
    assert!(wrapper.contains(".val <- .Call(C_Counter__new"));
    assert!(wrapper.contains("structure(.val, class = \"Counter\")"));

    // Verify S3 generics are created
    assert!(wrapper.contains("get <- function(x, ...) UseMethod(\"get\")"));
    assert!(wrapper.contains("increment <- function(x, ...) UseMethod(\"increment\")"));

    // Verify S3 methods
    assert!(wrapper.contains("#' @method get Counter"));
    assert!(wrapper.contains("get.Counter <- function(x, ...)"));
    assert!(wrapper.contains(".Call(C_Counter__get, .call = match.call(), x)"));

    assert!(wrapper.contains("#' @method increment Counter"));
    assert!(wrapper.contains("increment.Counter <- function(x, ...)"));
    assert!(wrapper.contains(".Call(C_Counter__increment, .call = match.call(), x)"));

    // Verify static methods with prefix
    assert!(wrapper.contains("counter_zero <- function()"));

    // Verify class environment for trait namespace compatibility
    assert!(wrapper.contains("Counter <- new.env(parent = emptyenv())"));
}

#[test]
fn s3_wrapper_generic_override() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s3(generic = "print"))]
            pub fn show(&self) -> String { unimplemented!() }

            #[miniextendr(s3(generic = "length"))]
            pub fn len(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let wrapper = generate_s3_r_wrapper(&parsed);

    // Should NOT create new generics for print and length (they exist in base R)
    assert!(!wrapper.contains("print <- function(x, ...) UseMethod(\"print\")"));
    assert!(!wrapper.contains("length <- function(x, ...) UseMethod(\"length\")"));

    // Should create S3 methods using the generic name
    assert!(wrapper.contains("#' @method print Counter"));
    assert!(wrapper.contains("print.Counter <- function(x, ...)"));
    assert!(wrapper.contains("#' @method length Counter"));
    assert!(wrapper.contains("length.Counter <- function(x, ...)"));
}

#[test]
fn s3_internal_keeps_s3method_export() {
    // Regression: #431. `internal` on an S3 impl must suppress NAMESPACE
    // export of the bare generic (so `R CMD check --as-cran` doesn't flag
    // an exported-but-undocumented generic) while keeping `S3method()`
    // registration on each method — otherwise dispatch on instances of the
    // class breaks for the package's own tests and any downstream caller.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::S3);
    attrs.internal = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).expect("parse");
    let wrapper = generate_s3_r_wrapper(&parsed);

    // Generic is defined but NOT exported.
    assert!(wrapper.contains("get <- function(x, ...) UseMethod(\"get\")"));
    assert!(
        !wrapper.contains("#' @export get"),
        "internal S3 must not export the bare generic"
    );

    // The method block keeps `#' @method get Counter` + `#' @export` so
    // roxygen2 emits `S3method(get, Counter)` in NAMESPACE.
    assert!(wrapper.contains("#' @method get Counter"));
    let method_export = wrapper
        .lines()
        .skip_while(|l| !l.contains("#' @method get Counter"))
        .take(6)
        .any(|l| l.trim() == "#' @export");
    assert!(
        method_export,
        "internal S3 must keep S3method() registration\n--- wrapper ---\n{wrapper}"
    );
}
// endregion

// region: S4 class system tests

#[test]
fn s4_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S4, item_impl);
    let wrapper = generate_s4_r_wrapper(&parsed);

    // Verify setClass
    assert!(wrapper.contains("methods::setClass(\"Counter\", slots = c(ptr = \"externalptr\"))"));

    // Verify @importFrom methods
    assert!(wrapper.contains("@importFrom methods setClass setGeneric setMethod new"));

    // Verify @slot documentation
    assert!(wrapper.contains("@slot ptr External pointer to Rust `Counter` struct"));

    // Verify constructor
    assert!(wrapper.contains("Counter <- function(value)"));
    assert!(wrapper.contains(".val <- .Call(C_Counter__new"));
    assert!(wrapper.contains("methods::new(\"Counter\", ptr = .val)"));

    // Verify S4 generics (unconditional - setGeneric is idempotent)
    assert!(
        wrapper.contains(
            "methods::setGeneric(\"s4_get\", function(x, ...) standardGeneric(\"s4_get\"))"
        )
    );
    assert!(wrapper.contains(
        "methods::setGeneric(\"s4_increment\", function(x, ...) standardGeneric(\"s4_increment\"))"
    ));

    // Verify setMethod calls
    assert!(wrapper.contains("methods::setMethod(\"s4_get\", \"Counter\""));
    assert!(wrapper.contains("methods::setMethod(\"s4_increment\", \"Counter\""));

    // Verify @exportMethod tags
    assert!(wrapper.contains("@exportMethod s4_get"));
    assert!(wrapper.contains("@exportMethod s4_increment"));
}

#[test]
fn s4_wrapper_generic_override() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s4(generic = "show"))]
            pub fn display(&self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S4, item_impl);
    let wrapper = generate_s4_r_wrapper(&parsed);

    // Should use "show" generic instead of "s4_display"
    assert!(wrapper.contains("methods::setMethod(\"show\", \"Counter\""));
    assert!(wrapper.contains("@exportMethod show"));
}
// endregion

// region: S7 class system tests

#[test]
fn s7_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn from_parts(a: i32, b: i32) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Verify S7 class definition
    assert!(wrapper.contains("Counter <- S7::new_class(\"Counter\","));

    // Verify @importFrom S7
    assert!(
        wrapper
            .contains("@importFrom S7 new_class class_any new_object S7_object new_generic method")
    );

    // Verify properties
    assert!(wrapper.contains("properties = list("));
    assert!(wrapper.contains(".ptr = S7::class_any"));

    // Verify constructor with .ptr param (because from_parts returns Self)
    assert!(wrapper.contains("constructor = function(value, .ptr = NULL)"));
    assert!(wrapper.contains("if (!is.null(.ptr))"));
    assert!(wrapper.contains("S7::new_object(S7::S7_object(), .ptr = .ptr)"));
    assert!(wrapper.contains(".val <- .Call(C_Counter__new"));
    assert!(wrapper.contains("S7::new_object(S7::S7_object(), .ptr = .val)"));

    // Verify S7 generics (now multi-line braced form)
    assert!(wrapper.contains("if (!exists(\"get\", mode = \"function\")) {"));
    assert!(
        wrapper.contains(
            "  get <- S7::new_generic(\"get\", \"x\", function(x, ...) S7::S7_dispatch())"
        )
    );
    assert!(wrapper.contains("if (!exists(\"increment\", mode = \"function\")) {"));
    assert!(wrapper.contains(
        "  increment <- S7::new_generic(\"increment\", \"x\", function(x, ...) S7::S7_dispatch())"
    ));

    // Verify S7 method definitions
    assert!(wrapper.contains("S7::method(get, Counter) <- function(x, ...)"));
    assert!(wrapper.contains("S7::method(increment, Counter) <- function(x, ...)"));
    assert!(wrapper.contains(".Call(C_Counter__get, .call = match.call(), x@.ptr)"));
    assert!(wrapper.contains(".Call(C_Counter__increment, .call = match.call(), x@.ptr)"));

    // Verify static methods
    assert!(wrapper.contains("Counter_from_parts <- function(a, b)"));
}

#[test]
fn s7_wrapper_generic_override() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s7(generic = "base::print"))]
            pub fn show(&self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should use external generic for base::print
    assert!(wrapper.contains("print <- S7::new_external_generic(\"base\", \"print\")"));
    assert!(wrapper.contains("S7::method(print, Counter) <- function(x, ...)"));
}
// endregion

// region: Label support tests

#[test]
fn label_affects_c_wrapper_names() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl_with_label(ClassSystem::Env, "basic", item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // C wrapper names should include label
    assert!(wrapper.contains("C_Counter_basic_new"));
    assert!(wrapper.contains("C_Counter_basic_get"));
}
// endregion

// region: Parameter defaults tests

#[test]
fn parameter_defaults_in_r_wrapper() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(defaults(step = "1L", verbose = "FALSE"))]
            pub fn increment(&mut self, step: i32, verbose: bool) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // R wrapper should include defaults
    assert!(wrapper.contains("Counter$increment <- function(step = 1L, verbose = FALSE)"));
}

#[test]
fn parameter_defaults_r6() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(defaults(n = "10L"))]
            pub fn add(&mut self, n: i32) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // R6 method should include default
    assert!(wrapper.contains("add = function(n = 10L)"));
}
// endregion

// region: Roxygen propagation tests

#[test]
fn roxygen_tags_propagate_to_wrapper() {
    // The roxygen system propagates explicit @tags (like @param, @return)
    // Plain doc comments are NOT automatically converted to @description
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            /// @param value Initial value
            /// @return The new Counter instance
            pub fn new(value: i32) -> Self { unimplemented!() }

            /// @return The counter value
            pub fn get(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // For env-class, @param tags are converted to \describe blocks (avoids R CMD check warning)
    assert!(
        wrapper.contains("\\item{\\code{value}}{Initial value}"),
        "wrapper should contain param as \\describe item"
    );
    assert!(
        wrapper.contains("#' @return The counter value"),
        "wrapper should contain @return tag"
    );

    // Generated tags should be present
    assert!(wrapper.contains("#' @name Counter$new"));
    assert!(wrapper.contains("#' @name Counter$get"));
    assert!(wrapper.contains("#' @rdname Counter"));
    assert!(wrapper.contains("#' @source Generated by miniextendr"));
    assert!(wrapper.contains("#' @export"));
}
// endregion

// region: Return strategy tests (method chaining, Self returns)

#[test]
fn returns_self_method_chains_in_env() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn increment(&mut self) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // increment returns Self, so it should return self (the R object, not the .Call result)
    // The return strategy should handle this
    assert!(wrapper.contains("Counter$increment <- function()"));
}

#[test]
fn returns_unit_method_in_r6() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn reset(&mut self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // reset returns unit, should have invisible(self) for chaining
    assert!(wrapper.contains("reset = function()"));
}
// endregion

// region: vctrs class system tests

fn parse_impl_vctrs(vctrs_attrs: VctrsAttrs, code: syn::ItemImpl) -> ParsedImpl {
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = vctrs_attrs;
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

#[test]
fn vctrs_wrapper_vctr_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Percent {
            pub fn new(x: f64) -> Vec<f64> { unimplemented!() }
            // Static helpers — &self is not allowed on vctrs impls (MXL120)
            pub fn scale(amounts: Vec<f64>, factor: f64) -> Vec<f64> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: Some(false),
        ptype: None,
        abbr: Some("pct".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Verify constructor (vctrs convention: new_<class>)
    assert!(wrapper.contains("new_percent <- function(x)"));
    assert!(wrapper.contains(".val <- .Call(C_Percent__new"));
    assert!(wrapper.contains("data <- .val"));
    assert!(
        wrapper.contains("vctrs::new_vctr(data, class = \"Percent\", inherit_base_type = FALSE)")
    );

    // Verify vec_ptype_abbr
    assert!(wrapper.contains("vec_ptype_abbr.Percent <- function(x, ...) \"pct\""));

    // Verify vec_ptype2 self-coercion
    assert!(wrapper.contains("#' @method vec_ptype2 Percent.Percent"));
    assert!(wrapper.contains("vec_ptype2.Percent.Percent <- function(x, y, ...) vctrs::new_vctr(double(), class = \"Percent\", inherit_base_type = FALSE)"));

    // Verify vec_cast self-coercion
    assert!(wrapper.contains("#' @method vec_cast Percent.Percent"));
    assert!(wrapper.contains("vec_cast.Percent.Percent <- function(x, to, ...) x"));

    // Verify static helper emitted as regular function
    assert!(wrapper.contains("percent_scale <- function(amounts, factor)"));

    // Verify imports
    assert!(wrapper.contains("@importFrom vctrs"));
}

#[test]
fn vctrs_wrapper_rcrd_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Rational {
            pub fn new(n: i32, d: i32) -> Vec<i32> { unimplemented!() }
            // Static helpers — &self is not allowed on vctrs impls (MXL120)
            pub fn numerator(n: Vec<i32>, _d: Vec<i32>) -> Vec<i32> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Rcrd,
        base: None,
        inherit_base_type: None,
        ptype: None,
        abbr: Some("rat".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Verify constructor uses new_rcrd
    assert!(wrapper.contains("new_rational <- function(n, d)"));
    assert!(wrapper.contains("vctrs::new_rcrd(data, class = \"Rational\")"));

    // Verify vec_ptype_abbr
    assert!(wrapper.contains("vec_ptype_abbr.Rational <- function(x, ...) \"rat\""));

    // Verify vec_ptype2 for record uses x[0] pattern
    assert!(wrapper.contains("vec_ptype2.Rational.Rational <- function(x, y, ...) x[0]"));

    // Verify vec_cast self-coercion
    assert!(wrapper.contains("vec_cast.Rational.Rational <- function(x, to, ...) x"));
}

#[test]
fn vctrs_wrapper_list_of_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl IntList {
            pub fn new(data: Vec<Vec<i32>>) -> Vec<Vec<i32>> { unimplemented!() }
            // Static helper — &self is not allowed on vctrs impls (MXL120)
            pub fn len(data: Vec<Vec<i32>>) -> i32 { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::ListOf,
        base: None,
        inherit_base_type: None,
        ptype: Some("integer()".to_string()),
        abbr: Some("int[]".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Verify constructor uses new_list_of with ptype
    assert!(wrapper.contains("new_intlist <- function(data)"));
    assert!(wrapper.contains("vctrs::new_list_of(data, class = \"IntList\", ptype = integer())"));

    // Verify vec_ptype_abbr
    assert!(wrapper.contains("vec_ptype_abbr.IntList <- function(x, ...) \"int[]\""));

    // Verify vec_ptype2 for list_of
    assert!(wrapper.contains("vec_ptype2.IntList.IntList <- function(x, y, ...) vctrs::new_list_of(list(), class = \"IntList\", ptype = integer())"));

    // Verify vec_cast self-coercion
    assert!(wrapper.contains("vec_cast.IntList.IntList <- function(x, to, ...) x"));
}

#[test]
fn vctrs_wrapper_no_abbr() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Simple {
            pub fn new(x: f64) -> Vec<f64> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: None,
        inherit_base_type: None,
        ptype: None,
        abbr: None, // No abbreviation
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Should NOT have vec_ptype_abbr
    assert!(!wrapper.contains("vec_ptype_abbr.Simple"));

    // But should still have ptype2 and cast
    assert!(wrapper.contains("vec_ptype2.Simple.Simple"));
    assert!(wrapper.contains("vec_cast.Simple.Simple"));
}

#[test]
fn vctrs_protocol_method_override() {
    // vctrs protocol overrides must use static methods (MXL120 rejects &self receivers).
    // The vctrs(format) attribute maps a static method to the format.<Class> S3 method.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Currency {
            pub fn new(amounts: Vec<f64>) -> Vec<f64> { unimplemented!() }

            // Static helper: regular function named currency_symbol
            pub fn symbol(amounts: Vec<f64>) -> Vec<String> { unimplemented!() }

            // vctrs protocol override: maps to format.Currency
            #[miniextendr(vctrs(format))]
            pub fn format_currency(amounts: Vec<f64>) -> Vec<String> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: None,
        ptype: None,
        abbr: Some("$".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // format_currency method should be generated as format.Currency, not format_currency.Currency
    assert!(wrapper.contains("#' @method format Currency"));
    // Protocol methods get a trailing `...` so `format(x, nsmall = 2)` doesn't error
    // with "unused argument (nsmall = 2)" when R dispatches to format.Currency.
    assert!(wrapper.contains("format.Currency <- function(amounts, ...)"));

    // Should NOT create a new S3 generic for "format" (it's a base R function)
    assert!(!wrapper.contains("format <- function(x, ...) UseMethod(\"format\")"));

    // symbol static helper (non-protocol) should keep fixed formals — no trailing `...`
    assert!(wrapper.contains("currency_symbol <- function(amounts)"));
    assert!(!wrapper.contains("currency_symbol <- function(amounts, ...)"));
}
// endregion

// region: S7 property class type tests

#[test]
fn s7_property_class_types() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Range {
            pub fn new(start: f64, end: f64) -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(getter, prop = "midpoint"))]
            pub fn get_midpoint(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(setter, prop = "midpoint"))]
            pub fn set_midpoint(&mut self, value: f64) { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn is_valid(&self) -> bool { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn name(&self) -> String { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Debug: print the generated wrapper
    eprintln!("Generated S7 wrapper:\n{}", wrapper);

    // Verify class types are included in property definitions
    assert!(
        wrapper.contains("length = S7::new_property(class = S7::class_double, getter ="),
        "length property missing class type"
    );
    assert!(
        wrapper.contains("midpoint = S7::new_property(class = S7::class_double, getter ="),
        "midpoint property missing class type"
    );
    assert!(
        wrapper.contains("is_valid = S7::new_property(class = S7::class_logical, getter ="),
        "is_valid property missing class type"
    );
    assert!(
        wrapper.contains("name = S7::new_property(class = S7::class_character, getter ="),
        "name property missing class type"
    );

    // Verify imports include the class types
    assert!(
        wrapper.contains("class_double"),
        "missing class_double import"
    );
    assert!(
        wrapper.contains("class_logical"),
        "missing class_logical import"
    );
    assert!(
        wrapper.contains("class_character"),
        "missing class_character import"
    );
}

#[test]
fn s7_property_option_class_type() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Container {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn maybe_value(&self) -> Option<i32> { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Option<i32> should map to NULL | S7::class_integer
    assert!(
        wrapper
            .contains("maybe_value = S7::new_property(class = NULL | S7::class_integer, getter =")
    );
}

#[test]
fn s7_property_mirrors_s7_tests_rs() {
    // This test mirrors the exact structure of s7_tests.rs::S7Range
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl S7Range {
            pub fn new(start: f64, end: f64) -> Self {
                S7Range { start, end }
            }

            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 {
                self.end - self.start
            }

            #[miniextendr(s7(getter, prop = "midpoint"))]
            pub fn get_midpoint(&self) -> f64 {
                (self.start + self.end) / 2.0
            }

            #[miniextendr(s7(setter, prop = "midpoint"))]
            pub fn set_midpoint(&mut self, value: f64) {
                let half_length = (self.end - self.start) / 2.0;
                self.start = value - half_length;
                self.end = value + half_length;
            }

            pub fn s7_start(&self) -> f64 {
                self.start
            }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);

    // Debug: check method attributes
    for method in &parsed.methods {
        if method.ident == "length" {
            eprintln!(
                "length method attrs: s7_getter={}, s7_setter={}",
                method.method_attrs.s7.getter, method.method_attrs.s7.setter
            );
            eprintln!("length return type: {:?}", method.sig.output);
        }
    }

    let wrapper = generate_s7_r_wrapper(&parsed);
    eprintln!("Generated wrapper for S7Range:\n{}", wrapper);

    // Should have class type for length property
    assert!(
        wrapper.contains("length = S7::new_property(class = S7::class_double"),
        "length property should have class = S7::class_double"
    );
}
// endregion

// region: S7 type mapping tests

#[test]
fn s7_type_mapping_scalars() {
    use super::rust_type_to_s7_class;

    // Integer types
    let ty: syn::Type = syn::parse_quote!(i32);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );

    let ty: syn::Type = syn::parse_quote!(i16);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );

    // Float types
    let ty: syn::Type = syn::parse_quote!(f64);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_double".to_string())
    );

    let ty: syn::Type = syn::parse_quote!(f32);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_double".to_string())
    );

    // Logical
    let ty: syn::Type = syn::parse_quote!(bool);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_logical".to_string())
    );

    // Raw
    let ty: syn::Type = syn::parse_quote!(u8);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_raw".to_string())
    );

    // Character
    let ty: syn::Type = syn::parse_quote!(String);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_references() {
    use super::rust_type_to_s7_class;

    // &str maps to character
    let ty: syn::Type = syn::parse_quote!(&str);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_vec() {
    use super::rust_type_to_s7_class;

    // Vec<i32> -> class_integer
    let ty: syn::Type = syn::parse_quote!(Vec<i32>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );

    // Vec<f64> -> class_double
    let ty: syn::Type = syn::parse_quote!(Vec<f64>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_double".to_string())
    );

    // Vec<String> -> class_character
    let ty: syn::Type = syn::parse_quote!(Vec<String>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_option() {
    use super::rust_type_to_s7_class;

    // Option<i32> -> NULL | class_integer
    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("NULL | S7::class_integer".to_string())
    );

    // Option<String> -> NULL | class_character
    let ty: syn::Type = syn::parse_quote!(Option<String>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("NULL | S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_result() {
    use super::rust_type_to_s7_class;

    // Result<i32, E> -> class_integer (from Ok type)
    let ty: syn::Type = syn::parse_quote!(Result<i32, String>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );
}

#[test]
fn s7_type_mapping_unknown() {
    use super::rust_type_to_s7_class;

    // Bare PascalCase types emit the quiet-fallback CLASS_REF placeholder
    // (#203). The cdylib resolver substitutes it with the registered R class
    // name for S7 types, or `S7::class_any` for unregistered / non-S7 ones.
    let ty: syn::Type = syn::parse_quote!(MyCustomType);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some(".__MX_CLASS_REF_OR_ANY_MyCustomType__".to_string())
    );

    // Generic types (path with args) still return None so the caller omits
    // the `class =` entirely — S7 defaults to class_any in that case too.
    let ty: syn::Type = syn::parse_quote!(ExternalPtr<Foo>);
    assert_eq!(rust_type_to_s7_class(&ty), None);
}
// endregion

// region: S7 Phase 2: validation/defaults/required/frozen/deprecated tests

#[test]
fn s7_property_default_value() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Range {
            #[miniextendr(s7(getter, default = "0.0"))]
            pub fn score(&self) -> f64 { self.score }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include default = 0.0 in property definition
    assert!(
        wrapper.contains("default = 0.0"),
        "Expected default value in property, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_required() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl User {
            #[miniextendr(s7(getter, required))]
            pub fn id(&self) -> String { self.id.clone() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include error message for required property
    assert!(
        wrapper.contains("@id is required"),
        "Expected required error in property, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("stop("),
        "Expected stop() call for required property, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_frozen() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Config {
            #[miniextendr(s7(getter, frozen))]
            pub fn created_at(&self) -> f64 { self.created_at }

            #[miniextendr(s7(setter, prop = "created_at"))]
            pub fn set_created_at(&mut self, value: f64) { self.created_at = value; }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include frozen check in setter
    assert!(
        wrapper.contains("is frozen"),
        "Expected frozen error message in setter, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("cannot be modified"),
        "Expected frozen check in setter, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_deprecated() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Legacy {
            #[miniextendr(s7(getter, deprecated = "Use 'value' instead"))]
            pub fn old_value(&self) -> i32 { self.value }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include deprecation warning in getter
    assert!(
        wrapper.contains("is deprecated"),
        "Expected deprecation warning in getter, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("Use 'value' instead"),
        "Expected deprecation message in getter, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("warning("),
        "Expected warning() call for deprecated property, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_validator() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Score {
            #[miniextendr(s7(getter))]
            pub fn score(&self) -> f64 { self.score }

            #[miniextendr(s7(validate, prop = "score"))]
            pub fn validate_score(value: f64) -> Result<(), String> {
                if value < 0.0 || value > 100.0 {
                    Err("score must be between 0 and 100".into())
                } else {
                    Ok(())
                }
            }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include validator function in property
    assert!(
        wrapper.contains("validator = function(value)"),
        "Expected validator in property, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("C_Score__validate_score"),
        "Expected validator C function call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_combined_patterns() {
    // Test combining default + deprecated
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Config {
            #[miniextendr(s7(getter, default = "\"default\"", deprecated = "Will be removed"))]
            pub fn legacy_name(&self) -> String { self.name.clone() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have both default and deprecation
    assert!(
        wrapper.contains("default = \"default\""),
        "Expected default value, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("Will be removed"),
        "Expected deprecation message, got:\n{}",
        wrapper
    );
}
// endregion

// region: S7 Phase 3: Generic dispatch control tests

#[test]
fn s7_generic_no_dots() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            #[miniextendr(s7(no_dots))]
            pub fn length(&self) -> i32 { self.len }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have generic without ... in signature
    assert!(
        wrapper.contains("function(x) S7::S7_dispatch()"),
        "Expected no_dots generic, got:\n{}",
        wrapper
    );
    // Should NOT have ... in the generic definition
    assert!(
        !wrapper.contains("function(x, ...) S7::S7_dispatch()"),
        "Expected no_dots to remove ..., got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_multi_dispatch() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Dog {
            #[miniextendr(s7(dispatch = "x,y"))]
            pub fn compare(&self, other: i32) -> i32 { 0 }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have c("x", "y") dispatch args
    assert!(
        wrapper.contains(r#"c("x", "y")"#),
        "Expected multi-dispatch args, got:\n{}",
        wrapper
    );
    // Should have function(x, y, ...) signature
    assert!(
        wrapper.contains("function(x, y, ...) S7::S7_dispatch()"),
        "Expected multi-dispatch signature, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_multi_dispatch_no_dots() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Matrix {
            #[miniextendr(s7(dispatch = "x,y", no_dots))]
            pub fn multiply(&self, other: i32) -> i32 { 0 }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have c("x", "y") dispatch args
    assert!(
        wrapper.contains(r#"c("x", "y")"#),
        "Expected multi-dispatch args, got:\n{}",
        wrapper
    );
    // Should have function(x, y) signature without ...
    assert!(
        wrapper.contains("function(x, y) S7::S7_dispatch()"),
        "Expected strict multi-dispatch signature, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_fallback() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Printer {
            #[miniextendr(s7(fallback))]
            pub fn describe(&self) -> String { "unknown".to_string() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should register method for class_any instead of Printer
    assert!(
        wrapper.contains("S7::method(describe, S7::class_any)"),
        "Expected fallback to class_any, got:\n{}",
        wrapper
    );
    // Fallback should use safe self extraction (inherits check), not raw x@.ptr
    assert!(
        wrapper.contains("inherits(x, \"S7_object\")"),
        "Expected safe self extraction with inherits check, got:\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains(".Call(wrap__Printer__describe, x@.ptr,"),
        "Fallback should NOT use raw x@.ptr in .Call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_override_fallback() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Printer {
            #[miniextendr(s7(generic = "base::print", fallback))]
            pub fn print_it(&self) -> String { "printed".to_string() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Generic-override + fallback should use class_any, not Printer
    assert!(
        wrapper.contains("S7::method(print, S7::class_any)"),
        "Expected generic-override fallback to class_any, got:\n{}",
        wrapper
    );
    // Should also use safe self extraction
    assert!(
        wrapper.contains("inherits(x, \"S7_object\")"),
        "Expected safe self extraction in generic-override fallback, got:\n{}",
        wrapper
    );
}
// endregion

// region: S7 Phase 4: convert() methods from Rust From/TryFrom patterns

#[test]
fn s7_convert_from() {
    // Test convert_from: converts FROM another type TO this type
    // Pattern: static method takes OtherType and returns Self
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Point3D {
            pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }

            #[miniextendr(s7(convert_from = "Point2D"))]
            pub fn from_2d(p: Point2D) -> Self { Self { x: 0.0, y: 0.0, z: 0.0 } }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should generate S7::method(convert, list(.__MX_CLASS_REF_Point2D__, Point3D))
    // from_type is a cross-reference → placeholder; resolver replaces at cdylib write time.
    assert!(
        wrapper.contains("S7::method(convert, list(.__MX_CLASS_REF_Point2D__, Point3D))"),
        "Expected placeholder for cross-ref in convert method, got:\n{}",
        wrapper
    );
    // The method should call the C wrapper with from@.ptr
    assert!(
        wrapper.contains("from@.ptr"),
        "Expected from@.ptr in convert call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_convert_to() {
    // Test convert_to: converts FROM this type TO another type
    // Pattern: instance method takes &self and returns OtherType
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Point3D {
            pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }

            #[miniextendr(s7(convert_to = "Point2D"))]
            pub fn to_2d(&self) -> Point2D { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should generate S7::method(convert, list(Point3D, .__MX_CLASS_REF_Point2D__))
    // to_type is a cross-reference → placeholder; resolver replaces at cdylib write time.
    assert!(
        wrapper.contains("S7::method(convert, list(Point3D, .__MX_CLASS_REF_Point2D__))"),
        "Expected placeholder for cross-ref in convert method, got:\n{}",
        wrapper
    );
    // The method should call the C wrapper with from@.ptr
    assert!(
        wrapper.contains("from@.ptr"),
        "Expected from@.ptr in convert call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_convert_bidirectional() {
    // Test both convert_from and convert_to on the same class
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Celsius {
            pub fn new(value: f64) -> Self { Self { value } }

            #[miniextendr(s7(convert_from = "Fahrenheit"))]
            pub fn from_fahrenheit(f: Fahrenheit) -> Self { unimplemented!() }

            #[miniextendr(s7(convert_to = "Fahrenheit"))]
            pub fn to_fahrenheit(&self) -> Fahrenheit { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Both cross-references use placeholders; resolver replaces at cdylib write time.
    assert!(
        wrapper.contains("S7::method(convert, list(.__MX_CLASS_REF_Fahrenheit__, Celsius))"),
        "Expected placeholder for Fahrenheit in convert_from, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("S7::method(convert, list(Celsius, .__MX_CLASS_REF_Fahrenheit__))"),
        "Expected placeholder for Fahrenheit in convert_to, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_convert_from_and_to_mutually_exclusive() {
    // Test that specifying both convert_from and convert_to on the same method is an error
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Converter {
            pub fn new() -> Self { Self {} }

            // This should be invalid - can't have both on same method
            #[miniextendr(s7(convert_from = "TypeA", convert_to = "TypeB"))]
            pub fn invalid_convert(&self) -> TypeB { unimplemented!() }
        }
    };

    // This should fail during parsing/validation
    let result = std::panic::catch_unwind(|| parse_impl(ClassSystem::S7, impl_code));

    // The parse_impl function should panic or return an error for this invalid config
    // If it doesn't panic, we need to check the behavior differently
    if result.is_ok() {
        // If parsing succeeded, the validation should have caught this
        // The current implementation validates during parse_impl
        panic!("Expected error when both convert_from and convert_to are specified on same method");
    }
}

#[test]
fn s7_wrapper_parent() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Circle {
            pub fn new(radius: f64) -> Self { unimplemented!() }
            pub fn area(&self) -> f64 { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.s7_parent = Some("Shape".to_string());
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    // parent = uses a placeholder; resolver replaces at cdylib write time
    assert!(
        wrapper.contains("Circle <- S7::new_class(\"Circle\", parent = .__MX_CLASS_REF_Shape__,")
    );
}

#[test]
fn s7_wrapper_abstract() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Shape {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.s7_abstract = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    assert!(wrapper.contains("abstract = TRUE,"));
}

#[test]
fn s7_wrapper_defaults_no_parent_no_abstract() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // No parent or abstract by default
    assert!(!wrapper.contains("parent ="));
    assert!(!wrapper.contains("abstract = TRUE"));
}
// endregion

// region: ImplAttrs parsing tests

#[test]
fn parse_r6_with_options() {
    let attrs: ImplAttrs =
        syn::parse_str("r6(inherit = \"ParentClass\", cloneable, lock_class = true)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert_eq!(attrs.r6_inherit, Some("ParentClass".to_string()));
    assert_eq!(attrs.r6_cloneable, Some(true));
    assert_eq!(attrs.r6_lock_class, Some(true));
    assert_eq!(attrs.r6_portable, None);
    assert_eq!(attrs.r6_lock_objects, None);
}

#[test]
fn parse_r6_plain() {
    let attrs: ImplAttrs = syn::parse_str("r6").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert_eq!(attrs.r6_inherit, None);
    assert_eq!(attrs.r6_cloneable, None);
}

#[test]
fn parse_s7_with_parent() {
    let attrs: ImplAttrs = syn::parse_str("s7(parent = \"Shape\")").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert_eq!(attrs.s7_parent, Some("Shape".to_string()));
    assert!(!attrs.s7_abstract);
}

#[test]
fn parse_s7_abstract() {
    let attrs: ImplAttrs = syn::parse_str("s7(abstract)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert!(attrs.s7_abstract);
}

#[test]
fn parse_s7_parent_and_abstract() {
    let attrs: ImplAttrs = syn::parse_str("s7(parent = \"Base\", abstract)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert_eq!(attrs.s7_parent, Some("Base".to_string()));
    assert!(attrs.s7_abstract);
}
// endregion

// region: r_data_accessors parsing tests

#[test]
fn parse_r6_with_r_data_accessors() {
    let attrs: ImplAttrs = syn::parse_str("r6(r_data_accessors)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert!(attrs.r_data_accessors);
}

#[test]
fn parse_r6_with_r_data_accessors_and_options() {
    let attrs: ImplAttrs = syn::parse_str("r6(cloneable, lock_class, r_data_accessors)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert!(attrs.r_data_accessors);
    assert_eq!(attrs.r6_cloneable, Some(true));
    assert_eq!(attrs.r6_lock_class, Some(true));
}

#[test]
fn parse_s7_with_r_data_accessors() {
    let attrs: ImplAttrs = syn::parse_str("s7(r_data_accessors)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert!(attrs.r_data_accessors);
}

#[test]
fn parse_r6_without_r_data_accessors() {
    let attrs: ImplAttrs = syn::parse_str("r6(cloneable)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert!(!attrs.r_data_accessors);
}
// endregion

// region: R6 r_data_accessors wrapper generation test

#[test]
fn r6_wrapper_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Should contain the call to .rdata_active_bindings_MyType
    assert!(
        wrapper.contains(".rdata_active_bindings_MyType(MyType)"),
        "Expected .rdata_active_bindings_MyType(MyType) in:\n{}",
        wrapper
    );
}

#[test]
fn r6_wrapper_no_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let attrs = default_impl_attrs(ClassSystem::R6);
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Should NOT contain the call to .rdata_active_bindings
    assert!(
        !wrapper.contains(".rdata_active_bindings"),
        "Should not have .rdata_active_bindings in:\n{}",
        wrapper
    );
}
// endregion

// region: S7 r_data_accessors wrapper generation test

#[test]
fn s7_wrapper_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should use c(list(...), .rdata_properties_MyType) pattern
    assert!(
        wrapper.contains("properties = c(list("),
        "Expected 'properties = c(list(' in:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains(".rdata_properties_MyType)"),
        "Expected '.rdata_properties_MyType)' in:\n{}",
        wrapper
    );
}

#[test]
fn s7_wrapper_no_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let attrs = default_impl_attrs(ClassSystem::S7);
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should use regular properties = list(...) pattern
    assert!(
        wrapper.contains("properties = list("),
        "Expected 'properties = list(' in:\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains(".rdata_properties"),
        "Should not have .rdata_properties in:\n{}",
        wrapper
    );
}
// endregion

// region: Insta snapshot tests for R wrapper output stability
//
// These tests capture the full generated R wrapper output as snapshots.
// Run `cargo insta review` to review and accept changes after modifying codegen.

#[test]
fn snapshot_env_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            /// Create a new counter
            /// @param value Initial value
            pub fn new(value: i32) -> Self { unimplemented!() }
            /// Get the current value
            pub fn get(&self) -> i32 { unimplemented!() }
            /// Increment by one
            pub fn increment(&mut self) { unimplemented!() }
            /// Add a value and return the result
            pub fn add(&mut self, n: i32) -> i32 { unimplemented!() }
            /// Create from string
            pub fn from_string(s: String) -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::Env, item_impl);
    insta::assert_snapshot!(generate_env_r_wrapper(&parsed));
}

#[test]
fn snapshot_env_defaults() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Widget {
            pub fn new() -> Self { unimplemented!() }
            #[miniextendr(defaults(step = "1L", verbose = "FALSE"))]
            pub fn update(&mut self, step: i32, verbose: bool) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::Env, item_impl);
    insta::assert_snapshot!(generate_env_r_wrapper(&parsed));
}

#[test]
fn snapshot_r6_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            /// Create a new counter
            /// @param value Initial value
            pub fn new(value: i32) -> Self { unimplemented!() }
            /// Get the current value
            pub fn get(&self) -> i32 { unimplemented!() }
            /// Increment by one
            pub fn increment(&mut self) { unimplemented!() }
            /// Create from value (static factory)
            pub fn from_value(v: i32) -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::R6, item_impl);
    insta::assert_snapshot!(generate_r6_r_wrapper(&parsed));
}

#[test]
fn snapshot_r6_with_options() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Child {
            pub fn new() -> Self { unimplemented!() }
            pub fn greet(&self) -> String { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_inherit = Some("ParentClass".to_string());
    attrs.r6_cloneable = Some(true);
    attrs.r6_lock_class = Some(true);
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_r6_r_wrapper(&parsed));
}

#[test]
fn snapshot_r6_private_methods() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Secure {
            pub fn new() -> Self { unimplemented!() }
            pub fn public_api(&self) -> i32 { unimplemented!() }
            fn internal_compute(&self) -> i32 { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::R6, item_impl);
    insta::assert_snapshot!(generate_r6_r_wrapper(&parsed));
}

#[test]
fn snapshot_s3_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn zero() -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S3, item_impl);
    insta::assert_snapshot!(generate_s3_r_wrapper(&parsed));
}

#[test]
fn snapshot_s4_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S4, item_impl);
    insta::assert_snapshot!(generate_s4_r_wrapper(&parsed));
}

#[test]
fn snapshot_s7_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn from_parts(a: i32, b: i32) -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

#[test]
fn snapshot_s7_properties() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Range {
            pub fn new(start: f64, end: f64) -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(getter, prop = "midpoint"))]
            pub fn get_midpoint(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(setter, prop = "midpoint"))]
            pub fn set_midpoint(&mut self, value: f64) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with `r_data_accessors` and NO impl-block properties.
/// Verifies that the sidecar prop docs placeholder is emitted.
#[test]
fn snapshot_s7_sidecar_only_props() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl SidecarOnly {
            pub fn new(value: i32) -> Self { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with `r_data_accessors` AND impl-block properties.
/// Verifies impl-block @prop lines come first, then sidecar placeholder.
#[test]
fn snapshot_s7_sidecar_and_impl_props() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Mixed {
            pub fn new(x: f64) -> Self { unimplemented!() }

            /// The computed length.
            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with constructor params that have defaults and varargs.
/// In-scope NIT from #379: constructor-param @param doc coverage for these cases.
#[test]
fn snapshot_s7_prop_tags_with_defaults_and_varargs() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl WithDefaults {
            /// Constructor with defaults.
            /// @param name Character name.
            /// @param ... Additional parameters.
            pub fn new(name: String, scale: f64, mode: Option<i32>) -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn name(&self) -> String { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with documented impl-block properties.
/// Verifies that getter doc comments are propagated to @prop lines.
#[test]
fn snapshot_s7_documented_props() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Documented {
            pub fn new() -> Self { unimplemented!() }

            /// The integer count value.
            #[miniextendr(s7(getter))]
            pub fn count(&self) -> i32 { unimplemented!() }

            #[miniextendr(s7(setter))]
            pub fn count(&mut self, value: i32) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

#[test]
fn snapshot_vctrs_vctr() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Percent {
            pub fn new(x: f64) -> Vec<f64> { unimplemented!() }
            // Static methods only — &self not allowed on vctrs impls (MXL120)
            pub fn value(amounts: Vec<f64>) -> Vec<f64> { unimplemented!() }
            pub fn scale(amounts: Vec<f64>, factor: f64) -> Vec<f64> { unimplemented!() }
        }
    };
    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: Some(false),
        ptype: None,
        abbr: Some("pct".to_string()),
    };
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = vctrs_attrs;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_vctrs_r_wrapper(&parsed));
}

#[test]
fn snapshot_vctrs_rcrd() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Rational {
            pub fn new(n: i32, d: i32) -> Vec<i32> { unimplemented!() }
            // Static helper — &self not allowed on vctrs impls (MXL120)
            pub fn numerator(n: Vec<i32>) -> Vec<i32> { unimplemented!() }
        }
    };
    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Rcrd,
        base: None,
        inherit_base_type: None,
        ptype: None,
        abbr: Some("rat".to_string()),
    };
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = vctrs_attrs;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_vctrs_r_wrapper(&parsed));
}
// endregion
