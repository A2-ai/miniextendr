//! # `#[derive(Vctrs)]` - Rust Structs ↔ vctrs S3 Classes
//!
//! This module implements the `#[derive(Vctrs)]` macro which generates
//! vctrs-compatible S3 classes from Rust structs.
//!
//! ## Usage
//!
//! ```ignore
//! #[derive(Vctrs)]
//! #[vctrs(class = "percent", base = "double")]
//! pub struct Percent {
//!     #[vctrs(data)]
//!     data: Vec<f64>,
//! }
//! ```
//!
//! ## Attributes
//!
//! ### Container-level
//!
//! - `#[vctrs(class = "name")]` - R class name (required)
//! - `#[vctrs(base = "double" | "integer" | "list" | "record")]` - Base vector type
//! - `#[vctrs(abbr = "pct")]` - Abbreviation for `vec_ptype_abbr`
//! - `#[vctrs(inherit_base = true | false)]` - Whether to include base type in class vector
//! - `#[vctrs(coerce = "double" | "integer" | ...)]` - Additional types this class coerces with
//!
//! ### Field-level
//!
//! - `#[vctrs(data)]` - Mark field as the underlying data (required for `IntoVctrs`)
//! - `#[vctrs(skip)]` - Skip field when generating record fields
//!
//! ## Generated S3 Methods
//!
//! The derive macro generates the following R S3 methods:
//!
//! - `format.<class>()` - Format for printing
//! - `vec_ptype_abbr.<class>()` - Abbreviation (if provided)
//! - `vec_ptype_full.<class>()` - Full type name
//! - `vec_proxy.<class>()` - Proxy for subsetting operations
//! - `vec_restore.<class>()` - Restore from proxy after subsetting
//! - `vec_ptype2.<class>.<class>()` - Self-coercion prototype
//! - `vec_cast.<class>.<class>()` - Self-cast (identity)
//!
//! For record types, additional field accessor methods are generated.
//!
//! ## Module Registration
//!
//! Types with `#[derive(Vctrs)]` must be registered in `miniextendr_module!`:
//!
//! ```ignore
//! miniextendr_module! {
//!     mod mypackage;
//!     vctrs Percent;  // Register vctrs type
//! }
//! ```

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

/// Parsed vctrs attributes from struct definition.
#[derive(Default)]
struct VctrsAttrs {
    /// R class name (e.g., "vctrs_percent")
    class: Option<String>,
    /// Base vector type: "double", "integer", "character", "list", "record"
    base: Option<String>,
    /// Abbreviation for vec_ptype_abbr
    abbr: Option<String>,
    /// Whether to inherit base type in class vector
    inherit_base: Option<bool>,
    /// Additional types to generate coercion methods for (e.g., "double", "integer")
    coerce_with: Vec<String>,
    /// For list_of: R expression for the prototype (e.g., "integer()", "double()")
    ptype: Option<String>,
    /// Generate vec_proxy_equal method (for equality testing)
    proxy_equal: bool,
    /// Generate vec_proxy_compare method (for comparison/sorting)
    proxy_compare: bool,
    /// Generate vec_proxy_order method (for ordering)
    proxy_order: bool,
    /// Generate arithmetic methods (vec_arith)
    arith: bool,
    /// Generate math methods (vec_math)
    math: bool,
}

/// Parsed vctrs attributes from a field.
#[derive(Default)]
struct VctrsFieldAttrs {
    /// Mark as the data field for IntoVctrs
    is_data: bool,
    /// Skip this field in record generation
    skip: bool,
}

/// Information about a struct field.
struct FieldInfo {
    ident: syn::Ident,
    attrs: VctrsFieldAttrs,
}

/// Parse vctrs attributes from a struct.
fn parse_vctrs_attrs(attrs: &[syn::Attribute]) -> syn::Result<VctrsAttrs> {
    let mut result = VctrsAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("vctrs") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("class") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.class = Some(value.value());
                } else if meta.path.is_ident("base") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.base = Some(value.value());
                } else if meta.path.is_ident("abbr") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.abbr = Some(value.value());
                } else if meta.path.is_ident("inherit_base") {
                    let value: syn::LitBool = meta.value()?.parse()?;
                    result.inherit_base = Some(value.value());
                } else if meta.path.is_ident("coerce") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.coerce_with.push(value.value());
                } else if meta.path.is_ident("ptype") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.ptype = Some(value.value());
                } else if meta.path.is_ident("proxy_equal") {
                    result.proxy_equal = true;
                } else if meta.path.is_ident("proxy_compare") {
                    result.proxy_compare = true;
                } else if meta.path.is_ident("proxy_order") {
                    result.proxy_order = true;
                } else if meta.path.is_ident("arith") {
                    result.arith = true;
                } else if meta.path.is_ident("math") {
                    result.math = true;
                } else {
                    return Err(meta.error(
                        "unknown vctrs attribute; expected one of: class, base, abbr, inherit_base, coerce, ptype, proxy_equal, proxy_compare, proxy_order, arith, math",
                    ));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

/// Parse vctrs attributes from a field.
fn parse_vctrs_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<VctrsFieldAttrs> {
    let mut result = VctrsFieldAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("vctrs") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("data") {
                    result.is_data = true;
                } else if meta.path.is_ident("skip") {
                    result.skip = true;
                } else {
                    return Err(meta.error("unknown vctrs field attribute; expected: data, skip"));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

/// Extract field information from a struct.
fn extract_fields(input: &DeriveInput) -> syn::Result<Vec<FieldInfo>> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Ok(Vec::new()),
    };

    match fields {
        Fields::Named(named) => {
            let mut result = Vec::new();
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    let attrs = parse_vctrs_field_attrs(&field.attrs)?;
                    result.push(FieldInfo {
                        ident: ident.clone(),
                        attrs,
                    });
                }
            }
            Ok(result)
        }
        Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            fields,
            "vctrs types require named fields",
        )),
        Fields::Unit => Ok(Vec::new()),
    }
}

/// Map base type string to SEXPTYPE.
fn base_to_sexptype(base: &str) -> Option<TokenStream> {
    match base {
        "double" | "numeric" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::REALSXP }),
        "integer" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::INTSXP }),
        "logical" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::LGLSXP }),
        "character" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::STRSXP }),
        "raw" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::RAWSXP }),
        "list" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::VECSXP }),
        "record" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::VECSXP }),
        _ => None,
    }
}

/// Map base type string to VctrsKind.
fn base_to_kind(base: &str) -> TokenStream {
    match base {
        "record" => quote! { ::miniextendr_api::vctrs::VctrsKind::Rcrd },
        "list" => quote! { ::miniextendr_api::vctrs::VctrsKind::ListOf },
        _ => quote! { ::miniextendr_api::vctrs::VctrsKind::Vctr },
    }
}

/// Options for R wrapper generation
struct RWrapperOptions<'a> {
    class: &'a str,
    base: &'a str,
    abbr: Option<&'a str>,
    record_fields: &'a [String],
    coerce_with: &'a [String],
    inherit_base: bool,
    ptype: Option<&'a str>,
    proxy_equal: bool,
    proxy_compare: bool,
    proxy_order: bool,
    arith: bool,
    math: bool,
}

/// Generate R wrapper code for vctrs S3 methods.
///
/// This generates the following S3 methods for the vctrs class:
/// - `format.<class>()` - Format for printing
/// - `vec_ptype_abbr.<class>()` - Abbreviation (if provided)
/// - `vec_ptype_full.<class>()` - Full type name
/// - `vec_proxy.<class>()` - Proxy for subsetting operations
/// - `vec_restore.<class>()` - Restore from proxy
/// - `vec_ptype2.<class>.<class>()` - Self-coercion prototype
/// - `vec_cast.<class>.<class>()` - Self-cast (identity)
///
/// For record types, it additionally generates:
/// - Field accessor `$` methods via vctrs infrastructure
///
/// For list_of types (base = "list"):
/// - Appropriate list handling methods
///
/// Optional methods (when enabled):
/// - `vec_proxy_equal.<class>()` - For equality testing
/// - `vec_proxy_compare.<class>()` - For comparison/sorting
/// - `vec_proxy_order.<class>()` - For ordering
/// - `vec_arith.<class>.<class>()` - For arithmetic operations
/// - `vec_math.<class>()` - For math functions
fn generate_r_wrappers(opts: &RWrapperOptions) -> String {
    let class = opts.class;
    let base = opts.base;
    let abbr = opts.abbr;
    let record_fields = opts.record_fields;
    let coerce_with = opts.coerce_with;
    let inherit_base = opts.inherit_base;
    let ptype = opts.ptype;
    let proxy_equal = opts.proxy_equal;
    let proxy_compare = opts.proxy_compare;
    let proxy_order = opts.proxy_order;
    let arith = opts.arith;
    let math = opts.math;
    let mut r_code = String::new();

    // =========================================================================
    // format.<class>
    // =========================================================================
    if base == "record" {
        // Record format: paste fields together with separator
        let field_formats: Vec<String> = record_fields
            .iter()
            .map(|f| format!("vctrs::field(x, \"{f}\")"))
            .collect();
        let fields_str = field_formats.join(", \"/\", ");
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs field
#' @export
format.{class} <- function(x, ...) {{
  paste0({fields_str})
}}
"#
        ));
    } else if base == "list" {
        // List-of format: show type and format each element
        r_code.push_str(&format!(
            r#"
#' @export
format.{class} <- function(x, ...) {{
  vapply(unclass(x), function(elt) {{
    if (is.null(elt)) "<NULL>" else paste0("<", vctrs::vec_ptype_abbr(elt), "[", vctrs::vec_size(elt), "]>")
  }}, character(1))
}}
"#
        ));
    } else {
        // Simple vctr format: use underlying data representation
        // Use unclass() instead of vec_data() to avoid recursion (vec_data calls vec_proxy)
        r_code.push_str(&format!(
            r#"
#' @export
format.{class} <- function(x, ...) {{
  format(unclass(x), ...)
}}
"#
        ));
    }

    // =========================================================================
    // vec_ptype_abbr.<class>
    // =========================================================================
    if let Some(abbr) = abbr {
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype_abbr
#' @export
vec_ptype_abbr.{class} <- function(x, ...) {{
  "{abbr}"
}}
"#
        ));
    }

    // =========================================================================
    // vec_ptype_full.<class>
    // =========================================================================
    r_code.push_str(&format!(
        r#"
#' @importFrom vctrs vec_ptype_full
#' @export
vec_ptype_full.{class} <- function(x, ...) {{
  "{class}"
}}
"#
    ));

    // =========================================================================
    // vec_proxy.<class> - strip class for operations
    // =========================================================================
    if base == "record" {
        // Record proxy: convert to data frame for vctrs operations
        // vctrs expects rcrd proxy to be a data frame with n = number of records
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_proxy new_data_frame
#' @export
vec_proxy.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
        ));
    } else if base == "list" {
        // List-of proxy: use list_of_proxy (wraps elements in list for df-column behavior)
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_proxy new_data_frame
#' @export
vec_proxy.{class} <- function(x, ...) {{
  # Wrap each element in a list so it becomes a df column
  vctrs::new_data_frame(list(elt = unclass(x)))
}}
"#
        ));
    } else {
        // Simple vctr proxy: strip class to get underlying data
        // Use unclass() instead of vec_data() to avoid recursion (vec_data calls vec_proxy)
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_proxy
#' @export
vec_proxy.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
        ));
    }

    // =========================================================================
    // vec_restore.<class> - restore class after subsetting
    // =========================================================================
    if base == "record" {
        // Record restore: convert data frame back to rcrd
        // x is a data frame from vec_proxy, convert to list and wrap as rcrd
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_restore new_rcrd
#' @export
vec_restore.{class} <- function(x, to, ...) {{
  vctrs::new_rcrd(as.list(x), class = "{class}")
}}
"#
        ));
    } else if base == "list" {
        // List-of restore: extract elt column and wrap as list_of
        let ptype_expr = ptype.unwrap_or("NULL");
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_restore new_list_of
#' @export
vec_restore.{class} <- function(x, to, ...) {{
  vctrs::new_list_of(x$elt, ptype = {ptype_expr}, class = "{class}")
}}
"#
        ));
    } else {
        // Simple vctr restore: use new_vctr
        let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_restore new_vctr
#' @export
vec_restore.{class} <- function(x, to, ...) {{
  vctrs::new_vctr(x, class = "{class}", inherit_base_type = {inherit_str})
}}
"#
        ));
    }

    // =========================================================================
    // vec_ptype2.<class>.<class> - self-coercion returns empty prototype
    // =========================================================================
    if base == "record" {
        // Record ptype2: extract prototype from x using vctrs::vec_ptype
        // This is the cleanest way since we don't know field types at compile time
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype2 vec_ptype
#' @export
vec_ptype2.{class}.{class} <- function(x, y, ...) {{
  vctrs::vec_ptype(x)
}}
"#
        ));
    } else if base == "list" {
        // List-of ptype2: use new_list_of with common ptype
        let ptype_expr = ptype.unwrap_or("NULL");
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype2 new_list_of vec_ptype_common
#' @export
vec_ptype2.{class}.{class} <- function(x, y, ...) {{
  ptype <- vctrs::vec_ptype_common(attr(x, "ptype"), attr(y, "ptype"))
  vctrs::new_list_of(list(), ptype = ptype, class = "{class}")
}}
"#
        ));
    } else {
        // Simple vctr ptype2: return empty vector with class
        let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{class}.{class} <- function(x, y, ...) {{
  vctrs::new_vctr({base}(0), class = "{class}", inherit_base_type = {inherit_str})
}}
"#,
            base = base_to_r_constructor(base)
        ));
    }

    // =========================================================================
    // vec_cast.<class>.<class> - self-cast is identity
    // =========================================================================
    r_code.push_str(&format!(
        r#"
#' @importFrom vctrs vec_cast
#' @export
vec_cast.{class}.{class} <- function(x, to, ...) {{
  x
}}
"#
    ));

    // =========================================================================
    // Generate coercion methods for other types (e.g., double, integer)
    // =========================================================================
    for other_type in coerce_with {
        // vec_ptype2.<class>.<other> - class wins, return class prototype
        let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };

        if base != "record" {
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{class}.{other_type} <- function(x, y, ...) {{
  vctrs::new_vctr({base}(0), class = "{class}", inherit_base_type = {inherit_str})
}}
"#,
                base = base_to_r_constructor(base)
            ));

            // vec_ptype2.<other>.<class> - symmetric
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{other_type}.{class} <- function(x, y, ...) {{
  vctrs::new_vctr({base}(0), class = "{class}", inherit_base_type = {inherit_str})
}}
"#,
                base = base_to_r_constructor(base)
            ));

            // vec_cast.<class>.<other> - cast other to class
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_cast new_vctr
#' @export
vec_cast.{class}.{other_type} <- function(x, to, ...) {{
  vctrs::new_vctr(as.{base}(x), class = "{class}", inherit_base_type = {inherit_str})
}}
"#,
                base = base_to_r_as_func(base)
            ));

            // vec_cast.<other>.<class> - cast class to other (strip class)
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_cast vec_data
#' @export
vec_cast.{other_type}.{class} <- function(x, to, ...) {{
  vctrs::vec_data(x)
}}
"#
            ));
        }
    }

    // =========================================================================
    // vec_proxy_equal.<class> - proxy for equality testing
    // =========================================================================
    if proxy_equal {
        if base == "record" {
            // For records, use the data frame proxy (already suitable for equality)
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_equal new_data_frame
#' @export
vec_proxy_equal.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
            ));
        } else if base == "list" {
            // For list_of, compare element by element
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_equal
#' @export
vec_proxy_equal.{class} <- function(x, ...) {{
  # For list-of, use element-wise proxy
  lapply(unclass(x), vctrs::vec_proxy_equal)
}}
"#
            ));
        } else {
            // For simple vctrs, use underlying data
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_equal
#' @export
vec_proxy_equal.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
            ));
        }
    }

    // =========================================================================
    // vec_proxy_compare.<class> - proxy for comparison/sorting
    // =========================================================================
    if proxy_compare {
        if base == "record" {
            // For records, use the data frame proxy
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_compare new_data_frame
#' @export
vec_proxy_compare.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
            ));
        } else if base == "list" {
            // List-of types generally can't be compared
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_compare stop_incompatible_type
#' @export
vec_proxy_compare.{class} <- function(x, ...) {{
  vctrs::stop_incompatible_type(x, x, x_arg = "", y_arg = "", action = "compare")
}}
"#
            ));
        } else {
            // For simple vctrs, use underlying data
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_compare
#' @export
vec_proxy_compare.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
            ));
        }
    }

    // =========================================================================
    // vec_proxy_order.<class> - proxy for ordering (may differ from compare)
    // =========================================================================
    if proxy_order {
        if base == "record" {
            // For records, use the data frame proxy
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_order new_data_frame
#' @export
vec_proxy_order.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
            ));
        } else if base == "list" {
            // List-of types generally can't be ordered
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_order stop_incompatible_type
#' @export
vec_proxy_order.{class} <- function(x, ...) {{
  vctrs::stop_incompatible_type(x, x, x_arg = "", y_arg = "", action = "order")
}}
"#
            ));
        } else {
            // For simple vctrs, use underlying data
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_order
#' @export
vec_proxy_order.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
            ));
        }
    }

    // =========================================================================
    // vec_arith.<class> - arithmetic operations
    // =========================================================================
    if arith {
        // For numeric-backed vctrs, arithmetic returns the same class
        if base != "record" && base != "list" && base != "character" && base != "raw" {
            // vec_arith.<class>.<class>
            let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_arith vec_arith_base new_vctr
#' @export
vec_arith.{class}.{class} <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = "{class}", inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_arith.<class>.numeric (right-hand side)
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_arith vec_arith_base new_vctr
#' @export
vec_arith.{class}.numeric <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = "{class}", inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_arith.numeric.<class> (left-hand side)
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_arith vec_arith_base new_vctr
#' @export
vec_arith.numeric.{class} <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = "{class}", inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_arith.<class>.MISSING (unary operations like -x)
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_arith vec_arith_base new_vctr
#' @export
vec_arith.{class}.MISSING <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = "{class}", inherit_base_type = {inherit_str})
}}
"#
            ));
        }
    }

    // =========================================================================
    // vec_math.<class> - math operations (abs, sqrt, log, etc.)
    // =========================================================================
    if math {
        // For numeric-backed vctrs, math returns the same class
        if base != "record" && base != "list" && base != "character" && base != "raw" {
            let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_math vec_math_base new_vctr
#' @export
vec_math.{class} <- function(.fn, .x, ...) {{
  result <- vctrs::vec_math_base(.fn, .x, ...)
  vctrs::new_vctr(result, class = "{class}", inherit_base_type = {inherit_str})
}}
"#
            ));
        }
    }

    r_code
}

/// Map base type to R constructor function name.
fn base_to_r_constructor(base: &str) -> &'static str {
    match base {
        "double" | "numeric" => "double",
        "integer" => "integer",
        "logical" => "logical",
        "character" => "character",
        "raw" => "raw",
        "list" => "list",
        "record" => "list",
        _ => "double",
    }
}

/// Map base type to R as.* coercion function name.
fn base_to_r_as_func(base: &str) -> &'static str {
    match base {
        "double" | "numeric" => "double",
        "integer" => "integer",
        "logical" => "logical",
        "character" => "character",
        "raw" => "raw",
        _ => "double",
    }
}

/// Generate the Vctrs derive implementation.
pub fn derive_vctrs(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse struct-level attributes
    let attrs = parse_vctrs_attrs(&input.attrs)?;

    // Validate: must be a struct
    match &input.data {
        Data::Struct(_) => {}
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(Vctrs)] can only be applied to structs (use #[derive(RFactor)] for enums)",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(Vctrs)] can only be applied to structs",
            ));
        }
    };

    // Extract field information
    let fields = extract_fields(&input)?;

    // Require class attribute
    let class_name = attrs.class.ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "#[derive(Vctrs)] requires #[vctrs(class = \"name\")] attribute",
        )
    })?;

    // Get base type (default to "double")
    let base = attrs.base.as_deref().unwrap_or("double");

    // Validate base type
    let sexptype = base_to_sexptype(base).ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            format!(
                "unknown base type '{}'; expected one of: double, integer, logical, character, raw, list, record",
                base
            ),
        )
    })?;

    // Get VctrsKind
    let kind = base_to_kind(base);

    // Determine inherit_base_type
    // Default: true for list/record, false for others
    let inherit_base = attrs
        .inherit_base
        .unwrap_or(matches!(base, "list" | "record"));

    // Get abbreviation
    let abbr = match &attrs.abbr {
        Some(a) => quote! { Some(#a) },
        None => quote! { None },
    };

    // Generate VctrsClass implementation
    let vctrs_class_impl = quote! {
        impl #impl_generics ::miniextendr_api::vctrs::VctrsClass for #name #ty_generics #where_clause {
            const CLASS_NAME: &'static str = #class_name;
            const KIND: ::miniextendr_api::vctrs::VctrsKind = #kind;
            const BASE_TYPE: Option<::miniextendr_api::ffi::SEXPTYPE> = Some(#sexptype);
            const INHERIT_BASE_TYPE: bool = #inherit_base;
            const ABBR: Option<&'static str> = #abbr;
        }
    };

    // Find data field for IntoVctrs
    let data_field = fields.iter().find(|f| f.attrs.is_data);

    // Generate IntoVctrs implementation if data field is marked
    let into_vctrs_impl = if let Some(data_field) = data_field {
        let data_ident = &data_field.ident;

        match base {
            "record" => {
                // For records, we need to build a List from all non-skipped fields
                let record_fields: Vec<_> = fields.iter().filter(|f| !f.attrs.skip).collect();
                let field_names: Vec<String> =
                    record_fields.iter().map(|f| f.ident.to_string()).collect();
                let field_idents: Vec<_> = record_fields.iter().map(|f| &f.ident).collect();

                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::ffi::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;

                            // Get attrs before moving fields out of self
                            let attrs = self.attrs();

                            // Build the fields list from pairs
                            let pairs: Vec<(&str, ::miniextendr_api::ffi::SEXP)> = vec![
                                #( (#field_names, self.#field_idents.into_sexp()), )*
                            ];
                            let fields = ::miniextendr_api::list::List::from_raw_pairs(pairs);

                            ::miniextendr_api::vctrs::new_rcrd(
                                fields,
                                &[Self::CLASS_NAME],
                                &attrs,
                            )
                        }
                    }
                }
            }
            "list" => {
                // For list_of types
                // Note: ptype needs to be passed via an attribute or a trait method
                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::ffi::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;

                            // Get attrs before moving data out of self
                            let attrs = self.attrs();
                            let data = self.#data_ident.into_sexp();
                            ::miniextendr_api::vctrs::new_list_of(
                                data,
                                &[Self::CLASS_NAME],
                                &attrs,
                            )
                        }
                    }
                }
            }
            _ => {
                // For simple vctrs (double, integer, etc.)
                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::ffi::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;

                            // Get attrs before moving data out of self
                            let attrs = self.attrs();
                            let data = self.#data_ident.into_sexp();
                            ::miniextendr_api::vctrs::new_vctr(
                                data,
                                &[Self::CLASS_NAME],
                                &attrs,
                                Some(Self::INHERIT_BASE_TYPE),
                            )
                        }
                    }
                }
            }
        }
    } else {
        TokenStream::new()
    };

    // Generate VctrsRecord implementation if base is "record"
    let record_impl = if base == "record" {
        let field_names: Vec<String> = fields
            .iter()
            .filter(|f| !f.attrs.skip)
            .map(|f| f.ident.to_string())
            .collect();
        let field_name_strs: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();

        quote! {
            impl #impl_generics ::miniextendr_api::vctrs::VctrsRecord for #name #ty_generics #where_clause {
                fn field_names() -> &'static [&'static str] {
                    &[#(#field_name_strs),*]
                }
            }
        }
    } else {
        TokenStream::new()
    };

    // Generate R wrapper code for vctrs S3 methods
    let record_field_names: Vec<String> = if base == "record" {
        fields
            .iter()
            .filter(|f| !f.attrs.skip)
            .map(|f| f.ident.to_string())
            .collect()
    } else {
        Vec::new()
    };
    let r_wrappers = generate_r_wrappers(&RWrapperOptions {
        class: &class_name,
        base,
        abbr: attrs.abbr.as_deref(),
        record_fields: &record_field_names,
        coerce_with: &attrs.coerce_with,
        inherit_base,
        ptype: attrs.ptype.as_deref(),
        proxy_equal: attrs.proxy_equal,
        proxy_compare: attrs.proxy_compare,
        proxy_order: attrs.proxy_order,
        arith: attrs.arith,
        math: attrs.math,
    });

    // Generate the R_WRAPPERS_VCTRS_{TYPE} const
    let name_upper = name.to_string().to_uppercase();
    let r_wrappers_const_ident = Ident::new(
        &format!("R_WRAPPERS_VCTRS_{}", name_upper),
        Span::call_site(),
    );

    Ok(quote! {
        #vctrs_class_impl
        #record_impl
        #into_vctrs_impl

        /// Generated R wrapper code for vctrs S3 methods.
        #[doc(hidden)]
        pub const #r_wrappers_const_ident: &str = #r_wrappers;
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_vctr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("VctrsClass"));
        assert!(code.contains("CLASS_NAME"));
        assert!(code.contains("percent"));
        assert!(code.contains("REALSXP"));
    }

    #[test]
    fn test_simple_vctr_with_data_field() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                #[vctrs(data)]
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate VctrsClass
        assert!(code.contains("VctrsClass"));
        // Should generate IntoVctrs with data field
        assert!(code.contains("IntoVctrs"));
        assert!(code.contains("self . data"));
        assert!(code.contains("new_vctr"));
    }

    #[test]
    fn test_record_vctr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                n: Vec<i32>,
                d: Vec<i32>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("VctrsClass"));
        assert!(code.contains("VctrsRecord"));
        assert!(code.contains("Rcrd"));
        assert!(code.contains("field_names"));
    }

    #[test]
    fn test_record_vctr_with_data() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                #[vctrs(data)]
                n: Vec<i32>,
                d: Vec<i32>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate all three traits
        assert!(code.contains("VctrsClass"));
        assert!(code.contains("VctrsRecord"));
        assert!(code.contains("IntoVctrs"));
        // Record uses new_rcrd
        assert!(code.contains("new_rcrd"));
        // Field names should be included
        assert!(code.contains("\"n\""));
        assert!(code.contains("\"d\""));
    }

    #[test]
    fn test_skip_field() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                #[vctrs(data)]
                n: Vec<i32>,
                d: Vec<i32>,
                #[vctrs(skip)]
                cached: Option<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Field names should NOT include cached
        assert!(code.contains("\"n\""));
        assert!(code.contains("\"d\""));
        assert!(!code.contains("\"cached\""));
    }

    #[test]
    fn test_with_abbr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double", abbr = "pct")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("pct"));
    }

    #[test]
    fn test_missing_class_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires #[vctrs(class")
        );
    }

    #[test]
    fn test_enum_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "color")]
            enum Color {
                Red,
                Green,
                Blue,
            }
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only be applied to structs")
        );
    }

    #[test]
    fn test_r_wrappers_const_generated() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate R_WRAPPERS_VCTRS_PERCENT const
        assert!(code.contains("R_WRAPPERS_VCTRS_PERCENT"));
        assert!(code.contains("pub const"));
    }

    #[test]
    fn test_r_wrappers_content_simple() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "percent",
            base: "double",
            abbr: Some("pct"),
            record_fields: &[],
            coerce_with: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have format method using unclass (not vec_data to avoid recursion)
        assert!(r_code.contains("format.percent"));
        assert!(r_code.contains("unclass(x)"));

        // Should have vec_ptype_abbr since abbr provided
        assert!(r_code.contains("vec_ptype_abbr.percent"));
        assert!(r_code.contains("\"pct\""));

        // Should have vec_ptype_full
        assert!(r_code.contains("vec_ptype_full.percent"));

        // Should have vec_proxy using unclass (not vec_data to avoid recursion)
        assert!(r_code.contains("vec_proxy.percent"));

        // Should have vec_restore
        assert!(r_code.contains("vec_restore.percent"));

        // Should have self-coercion methods
        assert!(r_code.contains("vec_ptype2.percent.percent"));
        assert!(r_code.contains("vec_cast.percent.percent"));
    }

    #[test]
    fn test_r_wrappers_content_record() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "rational",
            base: "record",
            abbr: None,
            record_fields: &["n".to_string(), "d".to_string()],
            coerce_with: &[],
            inherit_base: true, // records default to inherit_base = true
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have format method with vctrs::field accessors
        assert!(r_code.contains("format.rational"));
        assert!(r_code.contains("vctrs::field(x, \"n\")"));
        assert!(r_code.contains("vctrs::field(x, \"d\")"));

        // Should NOT have vec_ptype_abbr since no abbr
        assert!(!r_code.contains("vec_ptype_abbr.rational"));

        // Should have vec_ptype_full
        assert!(r_code.contains("vec_ptype_full.rational"));

        // Should have vec_proxy and vec_restore for records
        assert!(r_code.contains("vec_proxy.rational"));
        assert!(r_code.contains("vec_restore.rational"));
        // vec_proxy uses new_data_frame, vec_restore uses new_rcrd
        assert!(r_code.contains("new_data_frame"));
        assert!(r_code.contains("new_rcrd"));

        // Should have self-coercion (uses vec_ptype for records)
        assert!(r_code.contains("vec_ptype2.rational.rational"));
        assert!(r_code.contains("vec_ptype(x)"));
        assert!(r_code.contains("vec_cast.rational.rational"));
    }

    #[test]
    fn test_r_wrappers_no_abbr() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mytype",
            base: "integer",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should NOT have vec_ptype_abbr
        assert!(!r_code.contains("vec_ptype_abbr.mytype"));

        // Should still have format and vec_ptype_full
        assert!(r_code.contains("format.mytype"));
        assert!(r_code.contains("vec_ptype_full.mytype"));
    }

    #[test]
    fn test_r_wrappers_with_coercion() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "percent",
            base: "double",
            abbr: Some("%"),
            record_fields: &[],
            coerce_with: &["double".to_string()],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have self-coercion
        assert!(r_code.contains("vec_ptype2.percent.percent"));
        assert!(r_code.contains("vec_cast.percent.percent"));

        // Should have coercion with double
        assert!(r_code.contains("vec_ptype2.percent.double"));
        assert!(r_code.contains("vec_ptype2.double.percent"));
        assert!(r_code.contains("vec_cast.percent.double"));
        assert!(r_code.contains("vec_cast.double.percent"));
    }

    #[test]
    fn test_r_wrappers_list_of() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "list_of_integers",
            base: "list",
            abbr: Some("list<int>"),
            record_fields: &[],
            coerce_with: &[],
            inherit_base: true,
            ptype: Some("integer()"),
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have format for list_of
        assert!(r_code.contains("format.list_of_integers"));
        assert!(r_code.contains("vapply"));

        // Should have vec_ptype_abbr
        assert!(r_code.contains("vec_ptype_abbr.list_of_integers"));
        assert!(r_code.contains("list<int>"));

        // Should have vec_proxy for list_of
        assert!(r_code.contains("vec_proxy.list_of_integers"));
        assert!(r_code.contains("new_data_frame"));

        // Should have vec_restore with ptype
        assert!(r_code.contains("vec_restore.list_of_integers"));
        assert!(r_code.contains("new_list_of"));
        assert!(r_code.contains("integer()"));

        // Should have vec_ptype2 for list_of
        assert!(r_code.contains("vec_ptype2.list_of_integers.list_of_integers"));
        assert!(r_code.contains("vec_ptype_common"));
    }

    #[test]
    fn test_r_wrappers_proxy_methods() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mynum",
            base: "double",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: true,
            proxy_compare: true,
            proxy_order: true,
            arith: false,
            math: false,
        });

        // Should have proxy_equal method
        assert!(r_code.contains("vec_proxy_equal.mynum"));

        // Should have proxy_compare method
        assert!(r_code.contains("vec_proxy_compare.mynum"));

        // Should have proxy_order method
        assert!(r_code.contains("vec_proxy_order.mynum"));
    }

    #[test]
    fn test_r_wrappers_arith_methods() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mynum",
            base: "double",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: true,
            math: false,
        });

        // Should have vec_arith methods
        assert!(r_code.contains("vec_arith.mynum.mynum"));
        assert!(r_code.contains("vec_arith.mynum.numeric"));
        assert!(r_code.contains("vec_arith.numeric.mynum"));
        assert!(r_code.contains("vec_arith.mynum.MISSING"));
        assert!(r_code.contains("vec_arith_base"));
    }

    #[test]
    fn test_r_wrappers_math_methods() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mynum",
            base: "double",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: true,
        });

        // Should have vec_math method
        assert!(r_code.contains("vec_math.mynum"));
        assert!(r_code.contains("vec_math_base"));
    }
}
