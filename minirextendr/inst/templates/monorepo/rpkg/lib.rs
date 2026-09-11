use miniextendr_api::miniextendr;

miniextendr_api::miniextendr_init!();

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

// The Rust library `{{crate_name}}` is available as `core_library`.
// Adapt this example to its public API when wrapping an existing library.
{{{core_example_prefix}}}/// Greeting from the core Rust library
{{{core_example_prefix}}}///
{{{core_example_prefix}}}/// @return Greeting string produced by the core crate
{{{core_example_prefix}}}#[miniextendr]
{{{core_example_prefix}}}pub fn core_greeting() -> String {
{{{core_example_prefix}}}    core_library::hello().to_string()
{{{core_example_prefix}}} }
