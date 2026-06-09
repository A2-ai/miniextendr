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
//    Do NOT install with a bare `devtools::install()` / `R CMD INSTALL .` /
//    `devtools::document()`: the package's bootstrap step (R CMD build's
//    bootstrap.R) vendors dependencies into inst/vendor.tar.xz, which flips
//    ./configure into offline "tarball" mode. Tarball mode ships pre-generated
//    wrappers and SKIPS wrapper generation, so on a fresh build those paths
//    produce an empty namespace (library() exposes no functions).
//
//    miniextendr_build() handles this: on a fresh package (no
//    R/<pkg>-wrappers.R yet) it first generates the wrappers via an in-place
//    source-mode install (which never auto-vendors), and on later builds sets
//    MINIEXTENDR_FORCE_WRAPPER_GEN to force regeneration if a tarball latch is
//    present.

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

