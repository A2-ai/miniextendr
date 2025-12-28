//! {{crate_name}} - A Rust library with R bindings
//!
//! This crate provides the core functionality. The R package in `{{rpkg_name}}/`
//! exposes this functionality to R users.

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
