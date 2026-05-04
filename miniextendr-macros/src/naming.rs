//! Naming helpers for the static symbols the `#[miniextendr]` attribute emits.
//!
//! These formatters live in one place so the attribute macro and any later
//! consumer (e.g. a registration-chasing linter) can compute the exact same
//! identifiers from a source `syn::Ident`.

/// Identifier for the generated `const &str` holding the R wrapper source.
///
/// Returns `R_WRAPPER_{RUST_IDENT}` (uppercased).
pub(crate) fn r_wrapper_const_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let upper = rust_ident.to_string().to_uppercase();
    quote::format_ident!("R_WRAPPER_{upper}")
}

/// Convert a PascalCase string to snake_case.
///
/// Inserts an underscore before each uppercase letter (except the first),
/// then lowercases the entire result. For example, `"InProgress"` becomes
/// `"in_progress"`.
pub(crate) fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.char_indices() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

/// Convert a PascalCase string to kebab-case (`InProgress` → `in-progress`).
pub(crate) fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

/// Apply a `rename_all` transformation to a variant name.
///
/// Supports `"snake_case"`, `"kebab-case"`, `"lower"`, `"upper"`. Returns the
/// name unchanged if `rename_all` is `None` or unrecognised.
pub(crate) fn apply_rename_all(name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("snake_case") => to_snake_case(name),
        Some("kebab-case") => to_kebab_case(name),
        Some("lower") => name.to_lowercase(),
        Some("upper") => name.to_uppercase(),
        _ => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("InProgress"), "in_progress");
        assert_eq!(to_snake_case("ABC"), "a_b_c");
        assert_eq!(to_snake_case("Red"), "red");
    }

    #[test]
    fn kebab_case() {
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("InProgress"), "in-progress");
    }

    #[test]
    fn rename_all() {
        assert_eq!(
            apply_rename_all("HelloWorld", Some("snake_case")),
            "hello_world"
        );
        assert_eq!(
            apply_rename_all("HelloWorld", Some("kebab-case")),
            "hello-world"
        );
        assert_eq!(apply_rename_all("HelloWorld", Some("lower")), "helloworld");
        assert_eq!(apply_rename_all("HelloWorld", Some("upper")), "HELLOWORLD");
        assert_eq!(apply_rename_all("HelloWorld", None), "HelloWorld");
    }
}
