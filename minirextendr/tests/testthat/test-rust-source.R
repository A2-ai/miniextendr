# Tests for rust_source() and related functions
# These tests cover the internal helpers without requiring a Rust toolchain.

# ---- validate_rust_input ----

test_that("validate_rust_input requires exactly one of file/code", {
  expect_error(validate_rust_input(NULL, NULL), "One of `file` or `code`")
  expect_error(validate_rust_input("a.rs", "code"), "Only one of")
})

test_that("validate_rust_input returns code string directly", {
  result <- validate_rust_input(NULL, "fn main() {}")
  expect_equal(result, "fn main() {}")
})

test_that("validate_rust_input reads file contents", {
  tmp <- tempfile(fileext = ".rs")
  on.exit(unlink(tmp))
  writeLines(c("fn main() {", "    println!(\"hello\");", "}"), tmp)

  result <- validate_rust_input(tmp, NULL)
  expect_true(grepl("fn main", result))
  expect_true(grepl("println", result))
})

test_that("validate_rust_input errors for missing file", {
  expect_error(validate_rust_input("/no/such/file.rs", NULL), "File not found")
})

# ---- compute_inline_hash ----

test_that("compute_inline_hash is deterministic", {
  h1 <- compute_inline_hash("fn add(x: i32) -> i32 { x + 1 }")
  h2 <- compute_inline_hash("fn add(x: i32) -> i32 { x + 1 }")
  expect_equal(h1, h2)
})

test_that("compute_inline_hash differs for different code", {
  h1 <- compute_inline_hash("fn add(x: i32) -> i32 { x + 1 }")
  h2 <- compute_inline_hash("fn sub(x: i32) -> i32 { x - 1 }")
  expect_false(identical(unname(h1), unname(h2)))
})

test_that("compute_inline_hash differs for different features", {
  h1 <- compute_inline_hash("fn f() {}", features = character())
  h2 <- compute_inline_hash("fn f() {}", features = "rayon")
  expect_false(identical(unname(h1), unname(h2)))
})

test_that("compute_inline_hash normalizes feature order", {
  h1 <- compute_inline_hash("fn f() {}", features = c("a", "b"))
  h2 <- compute_inline_hash("fn f() {}", features = c("b", "a"))
  expect_equal(h1, h2)
})

test_that("compute_inline_hash trims whitespace", {
  h1 <- compute_inline_hash("fn f() {}")
  h2 <- compute_inline_hash("  fn f() {}  ")
  expect_equal(h1, h2)
})

# ---- parse_module_exports ----

test_that("parse_module_exports extracts fn names", {
  code <- '
miniextendr_module! {
    mod mymod;
    fn add;
    fn hello;
}
'
  result <- parse_module_exports(code)
  expect_equal(result$fns, c("add", "hello"))
  expect_equal(result$impls, character())
})

test_that("parse_module_exports extracts impl names", {
  code <- '
miniextendr_module! {
    mod mymod;
    fn create_counter;
    impl Counter;
}
'
  result <- parse_module_exports(code)
  expect_equal(result$fns, "create_counter")
  expect_equal(result$impls, "Counter")
})

test_that("parse_module_exports returns empty for no module block", {
  code <- 'pub fn add(x: i32) -> i32 { x + 1 }'
  result <- parse_module_exports(code)
  expect_equal(result$fns, character())
  expect_equal(result$impls, character())
})

test_that("parse_module_exports handles mixed fn and impl", {
  code <- '
miniextendr_module! {
    mod test;
    fn greet;
    impl Person;
    fn farewell;
    impl Dog;
}
'
  result <- parse_module_exports(code)
  expect_equal(result$fns, c("greet", "farewell"))
  expect_equal(result$impls, c("Person", "Dog"))
})

# ---- rewrite_module_name ----

test_that("rewrite_module_name replaces module name", {
  code <- '
miniextendr_module! {
    mod placeholder;
    fn add;
}
'
  result <- rewrite_module_name(code, "mxinline12345678")
  expect_true(grepl("mod mxinline12345678;", result))
  expect_false(grepl("mod placeholder;", result))
})

test_that("rewrite_module_name handles various spacing", {
  code1 <- 'miniextendr_module! { mod foo; fn x; }'
  expect_true(grepl("mod bar;", rewrite_module_name(code1, "bar")))

  code2 <- 'miniextendr_module!{\n    mod foo;\n    fn x;\n}'
  expect_true(grepl("mod bar;", rewrite_module_name(code2, "bar")))
})

test_that("rewrite_module_name preserves other code", {
  code <- '
use miniextendr_api::miniextendr;
pub fn add(a: f64) -> f64 { a + 1.0 }
miniextendr_module! {
    mod old_name;
    fn add;
}
'
  result <- rewrite_module_name(code, "new_name")
  expect_true(grepl("pub fn add", result))
  expect_true(grepl("use miniextendr_api", result))
  expect_true(grepl("mod new_name;", result))
})

# ---- extract_pub_fn_names ----

test_that("extract_pub_fn_names finds pub fn declarations", {
  code <- '
pub fn add(a: f64, b: f64) -> f64 { a + b }
pub fn hello(name: &str) -> String { format!("Hello, {}!", name) }
fn private_fn() {}
'
  result <- extract_pub_fn_names(code)
  expect_equal(result, c("add", "hello"))
})

test_that("extract_pub_fn_names returns empty for no pub fn", {
  code <- 'fn private() {}'
  expect_equal(extract_pub_fn_names(code), character())
})

# ---- extract_impl_names ----

test_that("extract_impl_names finds #[miniextendr] impl blocks", {
  code <- '
#[miniextendr]
impl Counter {
    pub fn new() -> Self { Counter { value: 0 } }
}
'
  result <- extract_impl_names(code)
  expect_equal(result, "Counter")
})

test_that("extract_impl_names returns empty for no impl blocks", {
  code <- 'pub fn add(x: i32) -> i32 { x + 1 }'
  expect_equal(extract_impl_names(code), character())
})

# ---- inline_cache_dir ----

test_that("inline_cache_dir returns a path under minirextendr cache", {
  dir <- inline_cache_dir()
  expect_true(grepl("minirextendr", dir))
  expect_true(grepl("rust_source", dir))
})

# ---- scaffold_inline_package (structure only, no build) ----

test_that("scaffold_inline_package creates correct directory structure", {
  skip_if_not_installed("minirextendr")

  tmp <- withr::local_tempdir()
  vendor_dir <- fs::path(tmp, "vendor")
  fs::dir_create(fs::path(vendor_dir, "miniextendr-api"), recurse = TRUE)
  fs::dir_create(fs::path(vendor_dir, "miniextendr-lint"), recurse = TRUE)

  code <- '
use miniextendr_api::{miniextendr, miniextendr_module};

#[miniextendr]
pub fn add_one(x: i32) -> i32 { x + 1 }

miniextendr_module! {
    mod placeholder;
    fn add_one;
}
'
  hash <- "abcdef1234567890abcdef1234567890"
  pkg_name <- "mxinlineabcdef12"
  pkg_rs <- "mxinlineabcdef12"

  scaffold_inline_package(code, hash, character(), pkg_name, pkg_rs,
                           tmp, quiet = TRUE)

  pkg_dir <- fs::path(tmp, hash, "pkg")

  # Check directory structure exists
  expect_true(fs::dir_exists(pkg_dir))
  expect_true(fs::file_exists(fs::path(pkg_dir, "DESCRIPTION")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "NAMESPACE")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "rust", "lib.rs")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "rust", "Cargo.toml")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "rust", "build.rs")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "rust", "document.rs")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "rust", "document.rs.in")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "entrypoint.c")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "entrypoint.c.in")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "mx_abi.c")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "src", "Makevars.in")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "configure.ac")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "inst", "include", "mx_abi.h")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "tools", "config.guess")))
  expect_true(fs::file_exists(fs::path(pkg_dir, "tools", "config.sub")))

  # Check DESCRIPTION content
  desc <- readLines(fs::path(pkg_dir, "DESCRIPTION"))
  expect_true(any(grepl(paste0("Package: ", pkg_name), desc)))

  # Check NAMESPACE exports
  ns <- readLines(fs::path(pkg_dir, "NAMESPACE"))
  expect_true(any(grepl("export\\(add_one\\)", ns)))
  expect_true(any(grepl(paste0("useDynLib\\(", pkg_name), ns)))

  # Check lib.rs has rewritten module name
  lib_rs <- paste(readLines(fs::path(pkg_dir, "src", "rust", "lib.rs")),
                  collapse = "\n")
  expect_true(grepl(paste0("mod ", pkg_rs, ";"), lib_rs))
  expect_false(grepl("mod placeholder;", lib_rs))

  # Check entrypoint.c has correct package name
  entry_c <- paste(readLines(fs::path(pkg_dir, "src", "entrypoint.c")),
                   collapse = "\n")
  expect_true(grepl(paste0("R_init_", pkg_name), entry_c))

  # Check vendor symlink
  expect_true(fs::link_exists(fs::path(pkg_dir, "vendor")) ||
              fs::dir_exists(fs::path(pkg_dir, "vendor")))
})

test_that("scaffold_inline_package handles features in Cargo.toml", {
  skip_if_not_installed("minirextendr")

  tmp <- withr::local_tempdir()
  vendor_dir <- fs::path(tmp, "vendor")
  fs::dir_create(fs::path(vendor_dir, "miniextendr-api"), recurse = TRUE)
  fs::dir_create(fs::path(vendor_dir, "miniextendr-lint"), recurse = TRUE)

  code <- '
use miniextendr_api::{miniextendr, miniextendr_module};
#[miniextendr]
pub fn f() -> i32 { 1 }
miniextendr_module! { mod placeholder; fn f; }
'
  scaffold_inline_package(code, "hash123", c("rayon"), "mxtest", "mxtest",
                           tmp, quiet = TRUE)

  cargo <- paste(readLines(fs::path(tmp, "hash123", "pkg", "src", "rust", "Cargo.toml")),
                 collapse = "\n")
  expect_true(grepl('"rayon"', cargo))
})

# ---- rust_source_clean ----

test_that("rust_source_clean handles empty cache", {
  withr::local_envvar(list(HOME = withr::local_tempdir()))
  # May emit informational messages, that's fine — just ensure no error
  expect_no_error(suppressMessages(rust_source_clean()))
})

# ---- rust_function wrapper ----

test_that("rust_function errors with no pub fn", {
  expect_error(
    rust_function("fn private() {}"),
    "No `pub fn` declarations"
  )
})
