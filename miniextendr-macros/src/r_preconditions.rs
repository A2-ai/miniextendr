//! R-side precondition generation for type checking.
//!
//! Generates `stopifnot()` checks in R wrapper functions that run BEFORE the `.Call()` boundary.
//! This gives users clear, idiomatic R error messages with proper stack traces instead of
//! Rust panic messages.
//!
//! Each assertion checks ONE thing with a precise error message:
//!
//! ```r
//! add <- function(a, b) {
//!   stopifnot(
//!     "'a' must be numeric, logical, or raw" = is.numeric(a) || is.logical(a) || is.raw(a),
//!     "'a' must have length 1" = length(a) == 1L,
//!     "'b' must be numeric, logical, or raw" = is.numeric(b) || is.logical(b) || is.raw(b),
//!     "'b' must have length 1" = length(b) == 1L
//!   )
//!   .Call(C_add, .call = match.call(), a, b)
//! }
//! ```

use std::collections::HashSet;

/// A single `stopifnot()` assertion: `"message" = condition`.
struct RAssertion {
    message: String,
    condition: String,
}

impl RAssertion {
    fn new(message: impl Into<String>, condition: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            condition: condition.into(),
        }
    }

    /// Format as a `stopifnot()` named argument: `"message" = condition`.
    fn to_stopifnot_arg(&self) -> String {
        format!("\"{}\" = {}", self.message, self.condition)
    }

    /// Wrap for nullable: prepend `is.null(param) || ` to the condition,
    /// and adjust the message to mention NULL.
    fn nullable(self, param: &str) -> Self {
        let message = if self.message.contains("must be ") {
            // "'x' must be character" → "'x' must be NULL or character"
            self.message.replacen("must be ", "must be NULL or ", 1)
        } else if self.message.contains("must have ") {
            // "'x' must have length 1" → "'x' must be NULL or have length 1"
            self.message
                .replacen("must have ", "must be NULL or have ", 1)
        } else {
            format!("{} (or NULL)", self.message)
        };
        Self {
            message,
            condition: format!("is.null({}) || {}", param, self.condition),
        }
    }
}

/// Classification of an R-side type check.
///
/// Numeric checks use a broad predicate: `is.numeric || is.logical || is.raw`.
/// R coerces logical→numeric freely, and raw→integer is valid for byte-sized types.
/// Borderline cases (e.g., raw→i64 in strict mode) pass the precondition and reach
/// Rust's strict checker, which produces better contextual error messages.
enum RTypeCheck {
    /// Numeric scalar: type check + length check (2 assertions)
    ScalarNumeric,
    /// Non-negative numeric scalar: type + length + non-negative (3 assertions)
    ScalarNonNeg,
    /// Non-numeric scalar: `is.<type>(x)` + length check (2 assertions)
    Scalar(&'static str),
    /// Numeric vector: type check only (1 assertion)
    VectorNumeric,
    /// Non-numeric vector: `is.<type>(x)` only (1 assertion)
    Vector(&'static str),
    /// Nullable wrapper: inner assertions with `is.null(x) ||` guard
    Nullable(Box<RTypeCheck>),
    /// List check: `is.list(x)` (1 assertion)
    List,
}

/// The numeric type predicate accepted by R's coercion layer.
fn numeric_type_check(param: &str) -> String {
    format!(
        "is.numeric({p}) || is.logical({p}) || is.raw({p})",
        p = param
    )
}

impl RTypeCheck {
    /// Produce the individual assertions for this type check.
    fn assertions(&self, param: &str) -> Vec<RAssertion> {
        match self {
            RTypeCheck::ScalarNumeric => vec![
                RAssertion::new(
                    format!("'{}' must be numeric, logical, or raw", param),
                    numeric_type_check(param),
                ),
                RAssertion::new(
                    format!("'{}' must have length 1", param),
                    format!("length({}) == 1L", param),
                ),
            ],
            RTypeCheck::ScalarNonNeg => vec![
                RAssertion::new(
                    format!("'{}' must be numeric, logical, or raw", param),
                    numeric_type_check(param),
                ),
                RAssertion::new(
                    format!("'{}' must have length 1", param),
                    format!("length({}) == 1L", param),
                ),
                RAssertion::new(
                    format!("'{}' must be non-negative", param),
                    // raw is always non-negative; guard with is.raw() to avoid
                    // "comparison not implemented" error for raw values
                    format!("is.raw({p}) || {p} >= 0", p = param),
                ),
            ],
            RTypeCheck::Scalar(r_type) => vec![
                RAssertion::new(
                    format!("'{}' must be {}", param, r_type),
                    format!("is.{}({})", r_type, param),
                ),
                RAssertion::new(
                    format!("'{}' must have length 1", param),
                    format!("length({}) == 1L", param),
                ),
            ],
            RTypeCheck::VectorNumeric => vec![RAssertion::new(
                format!("'{}' must be numeric, logical, or raw", param),
                numeric_type_check(param),
            )],
            RTypeCheck::Vector(r_type) => vec![RAssertion::new(
                format!("'{}' must be {}", param, r_type),
                format!("is.{}({})", r_type, param),
            )],
            RTypeCheck::Nullable(inner) => inner
                .assertions(param)
                .into_iter()
                .map(|a| a.nullable(param))
                .collect(),
            RTypeCheck::List => vec![RAssertion::new(
                format!("'{}' must be a list", param),
                format!("is.list({})", param),
            )],
        }
    }
}

/// Map a Rust type to its R-side type check, if applicable.
///
/// Returns `None` for types that should skip precondition checks (SEXP, Dots, ExternalPtr, etc.).
fn r_check_for_type(ty: &syn::Type) -> Option<RTypeCheck> {
    match ty {
        syn::Type::Path(type_path) => r_check_for_type_path(type_path),
        syn::Type::Reference(type_ref) => r_check_for_reference(type_ref),
        _ => None,
    }
}

/// Handle `syn::TypePath` — the most common case (e.g., `i32`, `Vec<f64>`, `Option<String>`).
fn r_check_for_type_path(type_path: &syn::TypePath) -> Option<RTypeCheck> {
    let segment = type_path.path.segments.last()?;
    let ident = segment.ident.to_string();

    match ident.as_str() {
        // Numeric scalars (accepts numeric, logical, and raw via R coercion)
        "i32" | "f64" | "f32" | "i8" | "i16" | "i64" | "isize" => Some(RTypeCheck::ScalarNumeric),

        // Unsigned numeric scalars (non-negative constraint)
        "u16" | "u32" | "u64" | "usize" => Some(RTypeCheck::ScalarNonNeg),

        // Logical scalar
        "bool" | "Rbool" | "Rboolean" => Some(RTypeCheck::Scalar("logical")),

        // Character scalar
        "String" | "char" | "PathBuf" => Some(RTypeCheck::Scalar("character")),

        // Raw scalar
        "u8" => Some(RTypeCheck::Scalar("raw")),

        // Complex scalar
        "Rcomplex" => Some(RTypeCheck::Scalar("complex")),

        // Option<T> → Nullable
        "Option" => {
            let inner_ty = extract_single_generic_arg(segment)?;
            r_check_for_type(inner_ty).map(|inner| RTypeCheck::Nullable(Box::new(inner)))
        }

        // Vec<T> → Vector (depends on element type)
        "Vec" => {
            let inner_ty = extract_single_generic_arg(segment)?;
            r_check_for_vec_element(inner_ty)
        }

        // Map types and named list → List
        "HashMap" | "BTreeMap" | "NamedList" => Some(RTypeCheck::List),

        // List (bare) → List
        "List" | "ListMut" => Some(RTypeCheck::List),

        // Skip types: SEXP, Dots, Missing, ExternalPtr, RLogical, etc.
        "SEXP" | "Dots" | "Missing" | "ExternalPtr" | "OwnedProtect" => None,

        // Unknown type → skip (let Rust side validate)
        _ => None,
    }
}

/// Handle reference types like `&str`, `&[u8]`, `&Path`, `&Dots`.
fn r_check_for_reference(type_ref: &syn::TypeReference) -> Option<RTypeCheck> {
    match type_ref.elem.as_ref() {
        // &str → character scalar
        syn::Type::Path(tp) => {
            let seg = tp.path.segments.last()?;
            match seg.ident.to_string().as_str() {
                "str" => Some(RTypeCheck::Scalar("character")),
                "Path" => Some(RTypeCheck::Scalar("character")),
                "Dots" => None,
                _ => None,
            }
        }
        // &[T] → vector check based on element type
        syn::Type::Slice(slice) => r_check_for_vec_element(&slice.elem),
        _ => None,
    }
}

/// Map a Vec/slice element type to the appropriate vector check.
fn r_check_for_vec_element(elem_ty: &syn::Type) -> Option<RTypeCheck> {
    let syn::Type::Path(tp) = elem_ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    let ident = seg.ident.to_string();

    match ident.as_str() {
        // Numeric vectors (accepts numeric, logical, and raw via R coercion)
        "i32" | "f64" | "f32" | "i8" | "i16" | "u16" | "u32" | "i64" | "u64" | "isize"
        | "usize" => Some(RTypeCheck::VectorNumeric),

        // Logical vector
        "bool" => Some(RTypeCheck::Vector("logical")),

        // Character vector
        "String" => Some(RTypeCheck::Vector("character")),

        // Raw vector
        "u8" => Some(RTypeCheck::Vector("raw")),

        // Complex vector
        "Rcomplex" => Some(RTypeCheck::Vector("complex")),

        // Vec<Option<T>> — e.g., Vec<Option<String>> for nullable strings
        "Option" => {
            let inner = extract_single_generic_arg(seg)?;
            // Vec<Option<String>> → character, Vec<Option<i32>> → numeric, etc.
            r_check_for_vec_element(inner)
        }

        _ => None,
    }
}

/// Extract the single generic type argument from a path segment.
///
/// e.g., `Option<String>` → `String`, `Vec<i32>` → `i32`
fn extract_single_generic_arg(segment: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments
        && let Some(syn::GenericArgument::Type(ty)) = args.args.first()
    {
        return Some(ty);
    }
    None
}

/// A parameter that needs a fallback precheck (unknown to the static table).
#[allow(dead_code)]
pub struct FallbackParam {
    /// R-normalized parameter name.
    pub r_name: String,
    /// The Rust type (for generating type-aware validation).
    pub rust_type: syn::Type,
}

/// Output of precondition analysis: static checks + fallback params.
pub struct PreconditionOutput {
    /// Lines forming a `stopifnot(...)` call for known types.
    pub static_checks: Vec<String>,
    /// Parameters needing fallback validation (unknown custom types).
    #[allow(dead_code)]
    pub fallback_params: Vec<FallbackParam>,
}

/// Types that should never get a fallback precheck (they're handled specially
/// or can't be meaningfully prechecked from R).
fn is_skip_type(ident: &str) -> bool {
    matches!(
        ident,
        "SEXP" | "Dots" | "Missing" | "ExternalPtr" | "OwnedProtect"
    )
}

/// Check if a type should get a fallback precheck (unknown to static table,
/// not a skip type, not a reference).
fn needs_fallback(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            let Some(seg) = tp.path.segments.last() else {
                return false;
            };
            !is_skip_type(&seg.ident.to_string())
        }
        // References (&str, &[T], &Dots) are handled by static table or skipped
        syn::Type::Reference(_) => false,
        _ => false,
    }
}

/// Build precondition checks for a function's parameters.
///
/// Returns:
/// - **`static_checks`**: Lines forming a `stopifnot(...)` call for known types
/// - **`fallback_params`**: Parameters needing validation (unknown custom types)
///
/// Static checks produce R-side `stopifnot()`:
/// ```r
/// stopifnot(
///   "'a' must be numeric, logical, or raw" = is.numeric(a) || is.logical(a) || is.raw(a),
///   "'a' must have length 1" = length(a) == 1L
/// )
/// ```
///
/// Skips:
/// - `self`/`&self`/`&mut self` (receiver args)
/// - Parameters in `skip_params` (e.g., match_arg params already validated)
/// - Skip types (SEXP, Dots, ExternalPtr, etc.)
pub fn build_precondition_checks(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    skip_params: &HashSet<String>,
) -> PreconditionOutput {
    let mut args = Vec::new();
    let mut fallback_params = Vec::new();

    for arg in inputs {
        // Skip receiver (self/&self/&mut self)
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };

        // Extract parameter name
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };

        // Use the R-normalized name for the check (matches the R formal)
        let r_name = crate::r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();

        // Skip match_arg params (already validated by match.arg())
        if skip_params.contains(&r_name) {
            continue;
        }

        // Map the Rust type to R assertions (known types)
        if let Some(check) = r_check_for_type(pt.ty.as_ref()) {
            for assertion in check.assertions(&r_name) {
                args.push(assertion.to_stopifnot_arg());
            }
        } else if needs_fallback(pt.ty.as_ref()) {
            // Unknown type → record for potential future validation
            fallback_params.push(FallbackParam {
                r_name,
                rust_type: (*pt.ty).clone(),
            });
        }
    }

    let static_checks = match args.len() {
        0 => Vec::new(),
        1 => vec![format!("stopifnot({})", args[0])],
        _ => {
            let mut lines = Vec::with_capacity(args.len() + 2);
            lines.push("stopifnot(".to_string());
            for (i, arg) in args.iter().enumerate() {
                let comma = if i < args.len() - 1 { "," } else { "" };
                lines.push(format!("  {}{}", arg, comma));
            }
            lines.push(")".to_string());
            lines
        }
    };

    PreconditionOutput {
        static_checks,
        fallback_params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to parse a type string into syn::Type
    fn parse_type(s: &str) -> syn::Type {
        syn::parse_str(s).unwrap()
    }

    /// Helper to get assertions for a type
    fn assertions_for(ty_str: &str, param: &str) -> Vec<RAssertion> {
        let ty = parse_type(ty_str);
        r_check_for_type(&ty).unwrap().assertions(param)
    }

    #[test]
    fn scalar_numeric_produces_two_assertions() {
        let asserts = assertions_for("i32", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message, "'x' must be numeric, logical, or raw");
        assert_eq!(
            asserts[0].condition,
            "is.numeric(x) || is.logical(x) || is.raw(x)"
        );
        assert_eq!(asserts[1].message, "'x' must have length 1");
        assert_eq!(asserts[1].condition, "length(x) == 1L");
    }

    #[test]
    fn all_signed_numeric_types_use_scalar_numeric() {
        for ty_str in &["i32", "f64", "f32", "i8", "i16", "i64", "isize"] {
            let asserts = assertions_for(ty_str, "x");
            assert_eq!(asserts.len(), 2, "{} should produce 2 assertions", ty_str);
            assert!(
                asserts[0].condition.contains("is.numeric(x)"),
                "{} type check",
                ty_str
            );
            assert!(
                asserts[0].condition.contains("is.logical(x)"),
                "{} accepts logical",
                ty_str
            );
            assert!(
                asserts[0].condition.contains("is.raw(x)"),
                "{} accepts raw",
                ty_str
            );
        }
    }

    #[test]
    fn scalar_non_neg_produces_three_assertions() {
        let asserts = assertions_for("u32", "n");
        assert_eq!(asserts.len(), 3);
        assert_eq!(asserts[0].message, "'n' must be numeric, logical, or raw");
        assert_eq!(asserts[1].message, "'n' must have length 1");
        assert_eq!(asserts[2].message, "'n' must be non-negative");
        assert_eq!(asserts[2].condition, "is.raw(n) || n >= 0");
    }

    #[test]
    fn all_unsigned_types_use_scalar_non_neg() {
        for ty_str in &["u16", "u32", "u64", "usize"] {
            let asserts = assertions_for(ty_str, "x");
            assert_eq!(asserts.len(), 3, "{} should produce 3 assertions", ty_str);
            assert!(
                asserts[2].condition.contains(">= 0"),
                "{} non-neg check",
                ty_str
            );
        }
    }

    #[test]
    fn scalar_logical() {
        let asserts = assertions_for("bool", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message, "'x' must be logical");
        assert_eq!(asserts[0].condition, "is.logical(x)");
        assert_eq!(asserts[1].condition, "length(x) == 1L");
    }

    #[test]
    fn scalar_character() {
        for ty_str in &["String", "char", "PathBuf"] {
            let asserts = assertions_for(ty_str, "s");
            assert_eq!(asserts.len(), 2);
            assert_eq!(asserts[0].message, "'s' must be character");
            assert_eq!(asserts[0].condition, "is.character(s)");
        }
    }

    #[test]
    fn ref_str() {
        let ty: syn::Type = syn::parse_str("& str").unwrap();
        let asserts = r_check_for_type(&ty).unwrap().assertions("s");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].condition, "is.character(s)");
    }

    #[test]
    fn scalar_raw() {
        let asserts = assertions_for("u8", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message, "'x' must be raw");
        assert_eq!(asserts[0].condition, "is.raw(x)");
    }

    #[test]
    fn vector_numeric_produces_one_assertion() {
        for ty_str in &["Vec<f64>", "Vec<i8>", "Vec<i32>", "Vec<i64>"] {
            let asserts = assertions_for(ty_str, "x");
            assert_eq!(asserts.len(), 1, "{} should produce 1 assertion", ty_str);
            assert_eq!(
                asserts[0].condition,
                "is.numeric(x) || is.logical(x) || is.raw(x)"
            );
        }
    }

    #[test]
    fn vector_character() {
        let asserts = assertions_for("Vec<String>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.character(x)");
    }

    #[test]
    fn vector_optional_string() {
        let asserts = assertions_for("Vec<Option<String>>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.character(x)");
    }

    #[test]
    fn slice_u8() {
        let ty: syn::Type = syn::parse_str("& [u8]").unwrap();
        let asserts = r_check_for_type(&ty).unwrap().assertions("x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.raw(x)");
    }

    #[test]
    fn nullable_wraps_inner_assertions() {
        let asserts = assertions_for("Option<i32>", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(
            asserts[0].message,
            "'x' must be NULL or numeric, logical, or raw"
        );
        assert_eq!(
            asserts[0].condition,
            "is.null(x) || is.numeric(x) || is.logical(x) || is.raw(x)"
        );
        assert_eq!(asserts[1].message, "'x' must be NULL or have length 1");
        assert_eq!(asserts[1].condition, "is.null(x) || length(x) == 1L");
    }

    #[test]
    fn nullable_character() {
        let asserts = assertions_for("Option<String>", "s");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message, "'s' must be NULL or character");
        assert_eq!(asserts[0].condition, "is.null(s) || is.character(s)");
        assert_eq!(asserts[1].message, "'s' must be NULL or have length 1");
    }

    #[test]
    fn map_types() {
        for ty_str in &["HashMap<String, i32>", "BTreeMap<String, f64>"] {
            let ty = parse_type(ty_str);
            let asserts = r_check_for_type(&ty).unwrap().assertions("x");
            assert_eq!(asserts.len(), 1);
            assert_eq!(asserts[0].condition, "is.list(x)");
        }
    }

    #[test]
    fn skip_types() {
        for ty_str in &["SEXP", "ExternalPtr<MyType>"] {
            let ty = parse_type(ty_str);
            assert!(
                r_check_for_type(&ty).is_none(),
                "{} should be skipped",
                ty_str
            );
        }
    }

    #[test]
    fn single_param_produces_multi_line() {
        // i32 produces 2 assertions → always multi-line now
        let sig: syn::Signature = syn::parse_str("fn f(n: i32)").unwrap();
        let output = build_precondition_checks(&sig.inputs, &HashSet::new());
        let checks = &output.static_checks;
        assert_eq!(checks.len(), 4); // stopifnot( + 2 args + )
        assert_eq!(checks[0], "stopifnot(");
        assert!(checks[1].contains("numeric, logical, or raw"));
        assert!(checks[2].contains("length 1"));
        assert_eq!(checks[3], ")");
        assert!(output.fallback_params.is_empty());
    }

    #[test]
    fn vector_param_single_line() {
        // Vec<f64> produces 1 assertion → single line
        let sig: syn::Signature = syn::parse_str("fn f(x: Vec<f64>)").unwrap();
        let output = build_precondition_checks(&sig.inputs, &HashSet::new());
        let checks = &output.static_checks;
        assert_eq!(checks.len(), 1);
        assert!(checks[0].starts_with("stopifnot("));
        assert!(checks[0].ends_with(')'));
    }

    #[test]
    fn two_scalar_params_produces_six_lines() {
        let sig: syn::Signature = syn::parse_str("fn f(a: i32, b: f64)").unwrap();
        let output = build_precondition_checks(&sig.inputs, &HashSet::new());
        let checks = &output.static_checks;
        // stopifnot( + 4 assertions (2 per param) + )
        assert_eq!(checks.len(), 6);
        assert_eq!(checks[0], "stopifnot(");
        assert!(checks[1].contains("'a'") && checks[1].contains("numeric"));
        assert!(checks[2].contains("'a'") && checks[2].contains("length 1"));
        assert!(checks[3].contains("'b'") && checks[3].contains("numeric"));
        assert!(checks[4].contains("'b'") && checks[4].contains("length 1"));
        assert_eq!(checks[5], ")");
    }

    #[test]
    fn build_checks_skips_match_arg() {
        let sig: syn::Signature = syn::parse_str("fn f(n: i32, mode: String)").unwrap();
        let mut skip = HashSet::new();
        skip.insert("mode".to_string());
        let output = build_precondition_checks(&sig.inputs, &skip);
        // Only n's 2 assertions remain
        let joined = output.static_checks.join("\n");
        assert!(joined.contains("'n'"));
        assert!(!joined.contains("'mode'"));
    }

    #[test]
    fn unknown_type_produces_fallback() {
        let sig: syn::Signature = syn::parse_str("fn f(x: MyCustomType)").unwrap();
        let output = build_precondition_checks(&sig.inputs, &HashSet::new());
        assert!(output.static_checks.is_empty());
        assert_eq!(output.fallback_params.len(), 1);
        assert_eq!(output.fallback_params[0].r_name, "x");
    }

    #[test]
    fn mixed_known_and_unknown_types() {
        let sig: syn::Signature = syn::parse_str("fn f(a: i32, b: MyType, c: String)").unwrap();
        let output = build_precondition_checks(&sig.inputs, &HashSet::new());
        // a (i32) and c (String) are known → static checks
        let joined = output.static_checks.join("\n");
        assert!(joined.contains("'a'"));
        assert!(joined.contains("'c'"));
        assert!(!joined.contains("'b'"));
        // b (MyType) is unknown → fallback
        assert_eq!(output.fallback_params.len(), 1);
        assert_eq!(output.fallback_params[0].r_name, "b");
    }

    #[test]
    fn sexp_not_fallback() {
        let sig: syn::Signature = syn::parse_str("fn f(x: SEXP)").unwrap();
        let output = build_precondition_checks(&sig.inputs, &HashSet::new());
        assert!(output.static_checks.is_empty());
        assert!(output.fallback_params.is_empty());
    }
}
