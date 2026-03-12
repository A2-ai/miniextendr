//! Benchmarks for miniextendr-lint scan performance.
//!
//! Creates synthetic fixture trees of various sizes and measures the time
//! to build the crate index and run all lint rules.

use std::fs;

use tempfile::TempDir;

fn main() {
    divan::main();
}

// =============================================================================
// Fixture generators
// =============================================================================

/// Generate a file with `n_fns` miniextendr functions.
fn generate_module_file(n_fns: usize, mod_name: &str) -> String {
    let mut code = String::new();
    for i in 0..n_fns {
        code.push_str(&format!(
            "#[miniextendr]\npub fn {mod_name}_fn_{i}(x: i32) -> i32 {{ x + {i} }}\n\n"
        ));
    }
    code
}

/// Generate a file with an impl block (n_methods methods).
fn generate_impl_file(n_methods: usize, type_name: &str) -> String {
    let mut code = String::new();
    code.push_str(&format!(
        "#[derive(ExternalPtr)]\npub struct {type_name} {{ value: i32 }}\n\n"
    ));
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
    code
}

/// Generate a lib.rs that declares N sub-modules.
fn generate_lib_rs(n_modules: usize) -> String {
    let mut code = String::new();
    for i in 0..n_modules {
        code.push_str(&format!("mod mod_{i};\n"));
    }
    code
}

/// Create a synthetic crate with `n_modules` sub-modules, each containing
/// `fns_per_module` functions.
fn create_fixture(n_modules: usize, fns_per_module: usize) -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("create src dir");

    fs::write(src.join("lib.rs"), generate_lib_rs(n_modules)).expect("write lib.rs");

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

    fs::write(src.join("lib.rs"), generate_lib_rs(n_modules)).expect("write lib.rs");

    for i in 0..n_modules {
        let mod_name = format!("mod_{i}");
        let type_name = format!("Type{i}");
        let content = generate_impl_file(methods_per_type, &type_name);
        fs::write(src.join(format!("{mod_name}.rs")), content).expect("write module");
    }

    dir
}

// =============================================================================
// Group 1: Full lint scan (index build + all rules)
// =============================================================================

mod full_scan {
    use super::*;

    #[divan::bench]
    fn small_10_modules() {
        let dir = create_fixture(10, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    #[divan::bench]
    fn medium_100_modules() {
        let dir = create_fixture(100, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    #[divan::bench]
    fn dense_10_modules_50fns() {
        let dir = create_fixture(10, 50);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

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

    #[divan::bench]
    fn small_10_types() {
        let dir = create_impl_fixture(10, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    #[divan::bench]
    fn medium_100_types() {
        let dir = create_impl_fixture(100, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

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

    #[divan::bench]
    fn small_10_modules() {
        let dir = create_fixture(10, 5);
        let index = CrateIndex::build(dir.path()).expect("index");
        divan::black_box(index.files.len());
    }

    #[divan::bench]
    fn medium_100_modules() {
        let dir = create_fixture(100, 5);
        let index = CrateIndex::build(dir.path()).expect("index");
        divan::black_box(index.files.len());
    }

    #[divan::bench]
    fn large_500_modules() {
        let dir = create_fixture(500, 5);
        let index = CrateIndex::build(dir.path()).expect("index");
        divan::black_box(index.files.len());
    }
}

// =============================================================================
// Group 4: Scaling comparison
// =============================================================================

mod scaling {
    use super::*;

    #[divan::bench]
    fn fns_500_files_10() {
        let dir = create_fixture(10, 50);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    #[divan::bench]
    fn fns_500_files_50() {
        let dir = create_fixture(50, 10);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    #[divan::bench]
    fn fns_500_files_100() {
        let dir = create_fixture(100, 5);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }

    #[divan::bench]
    fn fns_500_files_500() {
        let dir = create_fixture(500, 1);
        let report = miniextendr_lint::run(dir.path()).expect("lint");
        divan::black_box(report.diagnostics.len());
    }
}
