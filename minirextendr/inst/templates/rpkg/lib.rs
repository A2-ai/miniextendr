use miniextendr_api::miniextendr;

miniextendr_api::miniextendr_init!({{package_rs}});

// ---- Adding new functions ----
//
// 1. Add your #[miniextendr] function below
// 2. Rebuild:
//
//      Rscript -e 'devtools::document()'  # Compiles Rust + generates R wrappers
//      Rscript -e 'devtools::install()'   # Install the package
//
//    devtools::document() handles everything in one step: bootstrap.R runs
//    ./configure, make compiles Rust, R wrappers are generated via linkme,
//    and roxygen2 updates NAMESPACE — no manual ./configure or two-pass install.

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

// ---- Classes ----
//
// You can expose Rust structs as R6 classes. Here's a simple example:
//
//   use miniextendr_api::ExternalPtr;
//
//   #[derive(ExternalPtr)]
//   pub struct Counter {
//       value: i32,
//   }
//
//   #[miniextendr]
//   impl Counter {
//       pub fn new() -> Self {
//           Counter { value: 0 }
//       }
//
//       pub fn increment(&mut self) {
//           self.value += 1;
//       }
//
//       pub fn get(&self) -> i32 {
//           self.value
//       }
//   }

