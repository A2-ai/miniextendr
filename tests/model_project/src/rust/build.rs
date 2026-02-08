// build.rs - Runs miniextendr-lint at build time to check consistency
// between #[miniextendr] attributes and miniextendr_module! declarations.
fn main() {
    miniextendr_lint::build_script();
}
