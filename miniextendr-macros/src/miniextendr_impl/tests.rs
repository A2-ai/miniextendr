use super::*;

// =============================================================================
// Helper function for parsing impl blocks
// =============================================================================

fn parse_impl(class_system: ClassSystem, code: syn::ItemImpl) -> ParsedImpl {
    let attrs = ImplAttrs {
        class_system,
        class_name: None,
        label: None,
        vctrs_attrs: VctrsAttrs::default(),
    };
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

fn parse_impl_with_class_name(
    class_system: ClassSystem,
    class_name: &str,
    code: syn::ItemImpl,
) -> ParsedImpl {
    let attrs = ImplAttrs {
        class_system,
        class_name: Some(class_name.to_string()),
        label: None,
        vctrs_attrs: VctrsAttrs::default(),
    };
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

fn parse_impl_with_label(
    class_system: ClassSystem,
    label: &str,
    code: syn::ItemImpl,
) -> ParsedImpl {
    let attrs = ImplAttrs {
        class_system,
        class_name: None,
        label: Some(label.to_string()),
        vctrs_attrs: VctrsAttrs::default(),
    };
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

// =============================================================================
// Env class system tests
// =============================================================================

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

// =============================================================================
// R6 class system tests
// =============================================================================

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
    assert!(wrapper.contains("private$.ptr <- .Call(C_Counter__new"));

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

// =============================================================================
// S3 class system tests
// =============================================================================

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
    assert!(wrapper.contains("structure(.Call(C_Counter__new"));
    assert!(wrapper.contains("class = \"Counter\""));

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

// =============================================================================
// S4 class system tests
// =============================================================================

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
    assert!(wrapper.contains("@importFrom methods setClass setGeneric setMethod new isGeneric"));

    // Verify @slot documentation
    assert!(wrapper.contains("@slot ptr External pointer to Rust `Counter` struct"));

    // Verify constructor
    assert!(wrapper.contains("Counter <- function(value)"));
    assert!(wrapper.contains("methods::new(\"Counter\", ptr = .Call(C_Counter__new"));

    // Verify S4 generics
    assert!(wrapper.contains(
        "if (!methods::isGeneric(\"s4_get\")) methods::setGeneric(\"s4_get\", function(x, ...) standardGeneric(\"s4_get\"))"
    ));
    assert!(wrapper.contains(
        "if (!methods::isGeneric(\"s4_increment\")) methods::setGeneric(\"s4_increment\", function(x, ...) standardGeneric(\"s4_increment\"))"
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

// =============================================================================
// S7 class system tests
// =============================================================================

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
    assert!(wrapper.contains("S7::new_object(S7::S7_object(), .ptr = .Call(C_Counter__new"));

    // Verify S7 generics
    assert!(wrapper.contains(
        "if (!exists(\"get\", mode = \"function\")) get <- S7::new_generic(\"get\", \"x\", function(x, ...) S7::S7_dispatch())"
    ));
    assert!(wrapper.contains(
        "if (!exists(\"increment\", mode = \"function\")) increment <- S7::new_generic(\"increment\", \"x\", function(x, ...) S7::S7_dispatch())"
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

// =============================================================================
// Label support tests
// =============================================================================

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

// =============================================================================
// Parameter defaults tests
// =============================================================================

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

// =============================================================================
// Roxygen propagation tests
// =============================================================================

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

    // Explicit roxygen tags should propagate
    assert!(
        wrapper.contains("#' @param value Initial value"),
        "wrapper should contain @param tag"
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

// =============================================================================
// Return strategy tests (method chaining, Self returns)
// =============================================================================

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

// =============================================================================
// vctrs class system tests
// =============================================================================

fn parse_impl_vctrs(vctrs_attrs: VctrsAttrs, code: syn::ItemImpl) -> ParsedImpl {
    let attrs = ImplAttrs {
        class_system: ClassSystem::Vctrs,
        class_name: None,
        label: None,
        vctrs_attrs,
    };
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

#[test]
fn vctrs_wrapper_vctr_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Percent {
            pub fn new(x: f64) -> Self { unimplemented!() }
            pub fn value(&self) -> f64 { unimplemented!() }
            pub fn scale(&mut self, factor: f64) { unimplemented!() }
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
    assert!(wrapper.contains("data <- .Call(C_Percent__new"));
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

    // Verify S3 generics
    assert!(wrapper.contains("value <- function(x, ...) UseMethod(\"value\")"));
    assert!(wrapper.contains("scale <- function(x, ...) UseMethod(\"scale\")"));

    // Verify S3 methods
    assert!(wrapper.contains("#' @method value Percent"));
    assert!(wrapper.contains("value.Percent <- function(x, ...)"));
    assert!(wrapper.contains("#' @method scale Percent"));
    assert!(wrapper.contains("scale.Percent <- function(x, factor, ...)"));

    // Verify imports
    assert!(wrapper.contains("@importFrom vctrs"));
}

#[test]
fn vctrs_wrapper_rcrd_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Rational {
            pub fn new(n: i32, d: i32) -> Self { unimplemented!() }
            pub fn numerator(&self) -> i32 { unimplemented!() }
            pub fn denominator(&self) -> i32 { unimplemented!() }
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
            pub fn new(data: Vec<Vec<i32>>) -> Self { unimplemented!() }
            pub fn len(&self) -> i32 { unimplemented!() }
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
            pub fn new(x: f64) -> Self { unimplemented!() }
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
