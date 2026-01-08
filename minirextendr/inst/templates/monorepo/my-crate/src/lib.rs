//! {{crate_name}}: core Rust library for {{rpkg_name}}.
//!
//! This crate provides the Rust implementation. The R package in `{{rpkg_name}}/`
//! exposes this functionality to R users via miniextendr.

/// Example function that can be called from R
pub fn hello() -> &'static str {
    "Hello from {{crate_name}}!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from {{crate_name}}!");
    }
}
