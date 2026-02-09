use miniextendr_api::{miniextendr, miniextendr_module};

// ---- Adding new functions ----
//
// 1. Add your #[miniextendr] function below
// 2. Register it in the miniextendr_module! block at the bottom
// 3. Rebuild with the two-install dance:
//
//      ./configure                    # Generate build files
//      R CMD INSTALL .                # 1st install: compiles Rust + new macros
//      Rscript -e 'devtools::document()'  # Re-generate R wrappers
//      R CMD INSTALL .                # 2nd install: includes new R wrappers
//
//    The first install compiles your Rust code; devtools::document() then
//    runs the compiled macros to emit R/miniextendr_wrappers.R; the second
//    install bundles those new wrappers into the package.
//
// NOTE: `cargo check` won't work until ./configure has been run at least once,
// because document.rs contains template placeholders that need to be resolved.

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
//
// Then register with: `impl Counter;` in the miniextendr_module! below.

// Register the module with R
miniextendr_module! {
    mod {{package_rs}};
    fn add;
    fn hello;
}
