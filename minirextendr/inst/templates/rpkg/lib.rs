use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple function that adds two integers
///
/// @param a First integer
/// @param b Second integer
/// @return Sum of a and b
#[miniextendr]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Say hello to someone
///
/// @param name Name to greet
/// @return Greeting string
#[miniextendr]
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Register the module with R
miniextendr_module! {
    mod {{package_rs}};
    fn add;
    fn hello;
}
