use miniextendr_api::miniextendr;

miniextendr_api::miniextendr_init!();

// ---- Adding new functions ----
//
// 1. Add your #[miniextendr] function below
// 2. Rebuild:
//
//      Rscript -e 'minirextendr::miniextendr_build()'
//
//    miniextendr_build() runs autoconf + ./configure, compiles Rust, generates
//    the R wrappers (R/<pkg>-wrappers.R) via linkme, installs the package, and
//    updates NAMESPACE + man/ with roxygen2 — all in one step.
//
//    Every install (`R CMD INSTALL .`, `devtools::install()`, pak) regenerates
//    the wrappers from the freshly linked library, but only roxygen2 updates
//    NAMESPACE: a new function is exported once `devtools::document()` ran and
//    the package was installed again. miniextendr_build() does both.

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

