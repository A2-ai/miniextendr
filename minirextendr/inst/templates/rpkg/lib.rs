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
