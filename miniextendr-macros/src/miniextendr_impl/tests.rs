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
    assert!(wrapper.contains("@importFrom methods setClass setGeneric setMethod new"));

    // Verify @slot documentation
    assert!(wrapper.contains("@slot ptr External pointer to Rust `Counter` struct"));

    // Verify constructor
    assert!(wrapper.contains("Counter <- function(value)"));
    assert!(wrapper.contains("methods::new(\"Counter\", ptr = .Call(C_Counter__new"));

    // Verify S4 generics (unconditional - setGeneric is idempotent)
    assert!(wrapper.contains(
        "methods::setGeneric(\"s4_get\", function(x, ...) standardGeneric(\"s4_get\"))"
    ));
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

// =============================================================================
// S7 property class type tests
// =============================================================================

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
    assert!(wrapper.contains("length = S7::new_property(class = S7::class_double, getter ="), "length property missing class type");
    assert!(wrapper.contains("midpoint = S7::new_property(class = S7::class_double, getter ="), "midpoint property missing class type");
    assert!(wrapper.contains("is_valid = S7::new_property(class = S7::class_logical, getter ="), "is_valid property missing class type");
    assert!(wrapper.contains("name = S7::new_property(class = S7::class_character, getter ="), "name property missing class type");

    // Verify imports include the class types
    assert!(wrapper.contains("class_double"), "missing class_double import");
    assert!(wrapper.contains("class_logical"), "missing class_logical import");
    assert!(wrapper.contains("class_character"), "missing class_character import");
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
    assert!(wrapper.contains("maybe_value = S7::new_property(class = NULL | S7::class_integer, getter ="));
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
            eprintln!("length method attrs: s7_getter={}, s7_setter={}",
                     method.method_attrs.s7_getter, method.method_attrs.s7_setter);
            eprintln!("length return type: {:?}", method.sig.output);
        }
    }

    let wrapper = generate_s7_r_wrapper(&parsed);
    eprintln!("Generated wrapper for S7Range:\n{}", wrapper);

    // Should have class type for length property
    assert!(wrapper.contains("length = S7::new_property(class = S7::class_double"),
            "length property should have class = S7::class_double");
}

// =============================================================================
// S7 type mapping tests
// =============================================================================

#[test]
fn s7_type_mapping_scalars() {
    use super::rust_type_to_s7_class;

    // Integer types
    let ty: syn::Type = syn::parse_quote!(i32);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_integer".to_string()));

    let ty: syn::Type = syn::parse_quote!(i16);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_integer".to_string()));

    // Float types
    let ty: syn::Type = syn::parse_quote!(f64);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_double".to_string()));

    let ty: syn::Type = syn::parse_quote!(f32);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_double".to_string()));

    // Logical
    let ty: syn::Type = syn::parse_quote!(bool);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_logical".to_string()));

    // Raw
    let ty: syn::Type = syn::parse_quote!(u8);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_raw".to_string()));

    // Character
    let ty: syn::Type = syn::parse_quote!(String);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_character".to_string()));
}

#[test]
fn s7_type_mapping_references() {
    use super::rust_type_to_s7_class;

    // &str maps to character
    let ty: syn::Type = syn::parse_quote!(&str);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_character".to_string()));
}

#[test]
fn s7_type_mapping_vec() {
    use super::rust_type_to_s7_class;

    // Vec<i32> -> class_integer
    let ty: syn::Type = syn::parse_quote!(Vec<i32>);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_integer".to_string()));

    // Vec<f64> -> class_double
    let ty: syn::Type = syn::parse_quote!(Vec<f64>);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_double".to_string()));

    // Vec<String> -> class_character
    let ty: syn::Type = syn::parse_quote!(Vec<String>);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_character".to_string()));
}

#[test]
fn s7_type_mapping_option() {
    use super::rust_type_to_s7_class;

    // Option<i32> -> NULL | class_integer
    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    assert_eq!(rust_type_to_s7_class(&ty), Some("NULL | S7::class_integer".to_string()));

    // Option<String> -> NULL | class_character
    let ty: syn::Type = syn::parse_quote!(Option<String>);
    assert_eq!(rust_type_to_s7_class(&ty), Some("NULL | S7::class_character".to_string()));
}

#[test]
fn s7_type_mapping_result() {
    use super::rust_type_to_s7_class;

    // Result<i32, E> -> class_integer (from Ok type)
    let ty: syn::Type = syn::parse_quote!(Result<i32, String>);
    assert_eq!(rust_type_to_s7_class(&ty), Some("S7::class_integer".to_string()));
}

#[test]
fn s7_type_mapping_unknown() {
    use super::rust_type_to_s7_class;

    // Unknown types return None (will use class_any)
    let ty: syn::Type = syn::parse_quote!(MyCustomType);
    assert_eq!(rust_type_to_s7_class(&ty), None);

    let ty: syn::Type = syn::parse_quote!(ExternalPtr<Foo>);
    assert_eq!(rust_type_to_s7_class(&ty), None);
}

// =============================================================================
// S7 Phase 2: validation/defaults/required/frozen/deprecated tests
// =============================================================================

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
    assert!(wrapper.contains("default = 0.0"), "Expected default value in property, got:\n{}", wrapper);
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
    assert!(wrapper.contains("@id is required"), "Expected required error in property, got:\n{}", wrapper);
    assert!(wrapper.contains("stop("), "Expected stop() call for required property, got:\n{}", wrapper);
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
    assert!(wrapper.contains("is frozen"), "Expected frozen error message in setter, got:\n{}", wrapper);
    assert!(wrapper.contains("cannot be modified"), "Expected frozen check in setter, got:\n{}", wrapper);
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
    assert!(wrapper.contains("is deprecated"), "Expected deprecation warning in getter, got:\n{}", wrapper);
    assert!(wrapper.contains("Use 'value' instead"), "Expected deprecation message in getter, got:\n{}", wrapper);
    assert!(wrapper.contains("warning("), "Expected warning() call for deprecated property, got:\n{}", wrapper);
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
    assert!(wrapper.contains("validator = function(value)"), "Expected validator in property, got:\n{}", wrapper);
    assert!(wrapper.contains("C_Score__validate_score"), "Expected validator C function call, got:\n{}", wrapper);
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
    assert!(wrapper.contains("default = \"default\""), "Expected default value, got:\n{}", wrapper);
    assert!(wrapper.contains("Will be removed"), "Expected deprecation message, got:\n{}", wrapper);
}

// =============================================================================
// S7 Phase 3: Generic dispatch control tests
// =============================================================================

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
    assert!(wrapper.contains("function(x) S7::S7_dispatch()"), "Expected no_dots generic, got:\n{}", wrapper);
    // Should NOT have ... in the generic definition
    assert!(!wrapper.contains("function(x, ...) S7::S7_dispatch()"), "Expected no_dots to remove ..., got:\n{}", wrapper);
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
    assert!(wrapper.contains(r#"c("x", "y")"#), "Expected multi-dispatch args, got:\n{}", wrapper);
    // Should have function(x, y, ...) signature
    assert!(wrapper.contains("function(x, y, ...) S7::S7_dispatch()"), "Expected multi-dispatch signature, got:\n{}", wrapper);
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
    assert!(wrapper.contains(r#"c("x", "y")"#), "Expected multi-dispatch args, got:\n{}", wrapper);
    // Should have function(x, y) signature without ...
    assert!(wrapper.contains("function(x, y) S7::S7_dispatch()"), "Expected strict multi-dispatch signature, got:\n{}", wrapper);
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
    assert!(wrapper.contains("S7::method(describe, S7::class_any)"), "Expected fallback to class_any, got:\n{}", wrapper);
}

// =============================================================================
// S7 Phase 4: convert() methods from Rust From/TryFrom patterns
// =============================================================================

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

    // Should generate S7::method(convert, list(Point2D, Point3D))
    assert!(
        wrapper.contains("S7::method(convert, list(Point2D, Point3D))"),
        "Expected convert method registration, got:\n{}",
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

    // Should generate S7::method(convert, list(Point3D, Point2D))
    assert!(
        wrapper.contains("S7::method(convert, list(Point3D, Point2D))"),
        "Expected convert method registration, got:\n{}",
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

    // Should have both convert methods
    assert!(
        wrapper.contains("S7::method(convert, list(Fahrenheit, Celsius))"),
        "Expected convert from Fahrenheit, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("S7::method(convert, list(Celsius, Fahrenheit))"),
        "Expected convert to Fahrenheit, got:\n{}",
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
    let result = std::panic::catch_unwind(|| {
        parse_impl(ClassSystem::S7, impl_code)
    });

    // The parse_impl function should panic or return an error for this invalid config
    // If it doesn't panic, we need to check the behavior differently
    if result.is_ok() {
        // If parsing succeeded, the validation should have caught this
        // The current implementation validates during parse_impl
        panic!("Expected error when both convert_from and convert_to are specified on same method");
    }
}
