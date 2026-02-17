//! Benchmarks for miniextendr-lint scan performance.
//!
//! Creates synthetic fixture trees of various sizes and measures the time
//! to build the crate index and run all lint rules. Tests scaling behavior
//! from small (10 files) to large (1000 files) crate trees.

use std::fs;

use tempfile::TempDir;

fn main() {
    divan::main();
}

// =============================================================================
// Fixture generators
// =============================================================================

/// Generate a file with `n_fns` miniextendr functions and a matching module.
fn generate_module_file(n_fns: usize, mod_name: &str) -> String {
    let mut code = String::new();
    code.push_str("use miniextendr_api::{miniextendr, miniextendr_module};\n\n");

    // Function definitions
    for i in 0..n_fns {
        code.push_str(&format!(
            "#[miniextendr]\npub fn {mod_name}_fn_{i}(x: i32) -> i32 {{ x + {i} }}\n\n"
        ));
    }

    // Module declaration
    code.push_str("miniextendr_module! {\n");
    code.push_str(&format!("    mod {mod_name};\n"));
    for i in 0..n_fns {
        code.push_str(&format!("    fn {mod_name}_fn_{i};\n"));
    }
    code.push_str("}\n");

    code
}

/// Generate a file with an impl block (n_methods methods) and matching module.
fn generate_impl_file(n_methods: usize, type_name: &str, mod_name: &str) -> String {
    let mut code = String::new();
    code.push_str("use miniextendr_api::{miniextendr, miniextendr_module, ExternalPtr};\n\n");

    // Struct definition
    code.push_str(&format!(
        "#[derive(ExternalPtr)]\npub struct {type_name} {{ value: i32 }}\n\n"
    ));

    // Impl block
    code.push_str(&format!("#[miniextendr]\nimpl {type_name} {{\n"));
    code.push_str(&format!(
        "    pub fn new(v: i32) -> Self {{ {type_name} {{ value: v }} }}\n"
    ));
    for i in 0..n_methods {
        code.push_str(&format!(
            "    pub fn method_{i}(&self) -> i32 {{ self.value + {i} }}\n"
        ));
    }
    code.push_str("}\n\n");

    // Module declaration
    code.push_str("miniextendr_module! {\n");
    code.push_str(&format!("    mod {mod_name};\n"));
    code.push_str(&format!("    impl {type_name};\n"));
    code.push_str("}\n");

    code
}

/// Generate a lib.rs that uses N sub-modules.
fn generate_lib_rs(n_modules: usize) -> String {
    let mut code = String::new();
    code.push_str("use miniextendr_api::miniextendr_module;\n\n");

    for i in 0..n_modules {
        code.push_str(&format!("mod mod_{i};\n"));
    }
    code.push('\n');

    code.push_str("miniextendr_module! {\n");
    code.push_str("    mod testcrate;\n");
    for i in 0..n_modules {
        code.push_str(&format!("    use mod_{i};\n"));
    }
    code.push_str("}\n");

    code
}

/// Create a synthetic crate with `n_modules` sub-modules, each containing
/// `fns_per_module` functions. Returns the TempDir (must stay alive for path validity).
fn create_fixture(n_modules: usize, fns_per_module: usize) -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("create src dir");

    // lib.rs
    fs::write(src.join("lib.rs"), generate_lib_rs(n_modules)).expect("write lib.rs");

    // Sub-module files
    for i in 0..n_modules {
        let mod_name = format!("mod_{i}");
        let content = generate_module_file(fns_per_module, &mod_name);
        fs::write(src.join(format!("{mod_name}.rs")), content).expect("write module");
    }

    dir
}

/// Create a fixture with impl blocks (struct types with methods).
fn create_impl_fixture(n_modules: usize, methods_per_type: usize) -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("create src dir");

    // lib.rs
    fs::write(src.join("lib.rs"), generate_lib_rs(n_modules)).expect("write lib.rs");

    // Sub-module files with impl blocks
    for i in 0..n_modules {
        let mod_name = format!("mod_{i}");
        let type_name = format!("Type{i}");
        let content = generate_impl_file(methods_per_type, &type_name, &mod_name);
        fs::write(src.join(format!("{mod_name}.rs")), content).expect("write module");
    }

    dir
}

// =============================================================================
// Group 1: Full lint scan (index build + all rules)
// =============================================================================

mod full_scan {
    use super::*;

    /// 10 modules × 5 fns = 50 functions total.
    #[divan::bench]
    fn small_10_modules() {
        let dir = create_fixture(10, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 100 modules × 5 fns = 500 functions total.
    #[divan::bench]
    fn medium_100_modules() {
        let dir = create_fixture(100, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 10 modules × 50 fns = 500 functions (fewer files, more fns per file).
    #[divan::bench]
    fn dense_10_modules_50fns() {
        let dir = create_fixture(10, 50);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 500 modules × 5 fns = 2500 functions total.
    #[divan::bench]
    fn large_500_modules() {
        let dir = create_fixture(500, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }
}

// =============================================================================
// Group 2: Impl-heavy scan (struct + method blocks)
// =============================================================================

mod impl_scan {
    use super::*;

    /// 10 types × 5 methods each.
    #[divan::bench]
    fn small_10_types() {
        let dir = create_impl_fixture(10, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 100 types × 5 methods each.
    #[divan::bench]
    fn medium_100_types() {
        let dir = create_impl_fixture(100, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 10 types × 20 methods each (method-heavy).
    #[divan::bench]
    fn dense_10_types_20methods() {
        let dir = create_impl_fixture(10, 20);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }
}

// =============================================================================
// Group 3: Index build only (isolate parsing cost from rule execution)
// =============================================================================

mod index_build {
    use super::*;
    use miniextendr_lint::CrateIndex;

    /// Build index for 10 modules × 5 fns.
    #[divan::bench]
    fn small_10_modules() {
        let dir = create_fixture(10, 5);
        let index = CrateIndex::build(dir.path()).expect("index");
        divan::black_box(index.files.len());
    }

    /// Build index for 100 modules × 5 fns.
    #[divan::bench]
    fn medium_100_modules() {
        let dir = create_fixture(100, 5);
        let index = CrateIndex::build(dir.path()).expect("index");
        divan::black_box(index.files.len());
    }

    /// Build index for 500 modules × 5 fns.
    #[divan::bench]
    fn large_500_modules() {
        let dir = create_fixture(500, 5);
        let index = CrateIndex::build(dir.path()).expect("index");
        divan::black_box(index.files.len());
    }
}

// =============================================================================
// Group 4: Scaling comparison (same total fn count, different file counts)
// =============================================================================

mod scaling {
    use super::*;

    /// 500 fns in 10 files (50 fns/file).
    #[divan::bench]
    fn fns_500_files_10() {
        let dir = create_fixture(10, 50);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 500 fns in 50 files (10 fns/file).
    #[divan::bench]
    fn fns_500_files_50() {
        let dir = create_fixture(50, 10);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 500 fns in 100 files (5 fns/file).
    #[divan::bench]
    fn fns_500_files_100() {
        let dir = create_fixture(100, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    /// 500 fns in 500 files (1 fn/file).
    #[divan::bench]
    fn fns_500_files_500() {
        let dir = create_fixture(500, 1);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }
}
