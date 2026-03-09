use miniextendr_api::miniextendr;

miniextendr_api::miniextendr_init!({{package_rs}});

/// A simple function that adds two numbers
///
/// @param a First number
/// @param b Second number
/// @return Sum of a and b
#[miniextendr]
pub fn add(a: f64, b: f64) -> f64 {
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

