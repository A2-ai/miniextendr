//! Known vctrs S3 generic names used to auto-insert `@importFrom vctrs`.
//!
//! Independent of the `vctrs` feature flag — users can write
//! `#[miniextendr(s3(generic = "vec_proxy", class = "my_class"))]` whether or
//! not the `Vctrs` derive is enabled.

/// All vctrs-exported generics users might implement S3 methods for.
const VCTRS_GENERICS: &[&str] = &[
    // Core proxy/restore (required for most custom types)
    "vec_proxy",
    "vec_restore",
    // Type coercion (required for vec_c, vec_rbind, etc.)
    "vec_ptype2",
    "vec_cast",
    // Equality/comparison/ordering proxies
    "vec_proxy_equal",
    "vec_proxy_compare",
    "vec_proxy_order",
    // Printing/formatting
    "vec_ptype_abbr",
    "vec_ptype_full",
    "obj_print_data",
    "obj_print_footer",
    "obj_print_header",
    // str() output
    "obj_str_data",
    "obj_str_footer",
    "obj_str_header",
    // Arithmetic (for numeric-like types)
    "vec_arith",
    "vec_math",
    // Other
    "vec_ptype_finalise",
    "vec_cbind_frame_ptype",
    // List-of conversion
    "as_list_of",
];

/// Returns `true` if `generic` is a vctrs generic that needs `@importFrom vctrs`.
#[inline]
pub(crate) fn is_vctrs_generic(generic: &str) -> bool {
    VCTRS_GENERICS.contains(&generic)
}
