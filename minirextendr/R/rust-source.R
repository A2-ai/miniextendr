# Inline Rust compilation — sourceCpp()-like workflow for miniextendr

#' Compile inline Rust code and load functions into R
#'
#' Takes a `.rs` file or inline code string, scaffolds a temporary miniextendr
#' R package, builds it, and loads the compiled functions into the specified
#' environment. Similar to Rcpp's `sourceCpp()`.
#'
#' @param file Path to a `.rs` file containing Rust code with `#[miniextendr]`
#'   functions. Mutually exclusive with `code`.
#' @param code Character string of inline Rust code. Mutually exclusive with `file`.
#' @param env Environment to load compiled functions into (default: caller's environment).
#' @param cache Logical. If `TRUE` (default), reuses previously compiled packages
#'   when the code and features are unchanged.
#' @param quiet Logical. If `TRUE`, suppresses build output.
#' @param features Character vector of cargo features to enable.
#' @param use_local_crates Path to a local miniextendr repository for vendoring.
#'   If `NULL` (default), auto-detects via `MINIEXTENDR_LOCAL` env var or
#'   parent directory scan.
#' @return Invisibly returns a list with components:
#'   - `functions`: character vector of exported function names
#'   - `dll`: path to the compiled shared library
#'   - `package`: package name
#'   - `cached`: logical, whether the result was served from cache
#' @export
#'
#' @examples
#' \dontrun{
#' # Compile inline Rust code
#' rust_source(code = '
#' use miniextendr_api::miniextendr;
#'
#' #[miniextendr]
#' pub fn add_one(x: i32) -> i32 { x + 1 }
#' ')
#' add_one(41L)
#' #> [1] 42
#' }
rust_source <- function(file = NULL, code = NULL, env = parent.frame(),
                        cache = TRUE, quiet = FALSE, features = character(),
                        use_local_crates = NULL) {
  code <- validate_rust_input(file, code)

  check_rust()
  check_autoconf()

  hash <- compute_inline_hash(code, features)
  pkg_name <- paste0("mxinline", substr(hash, 1, 8))
  pkg_rs <- to_rust_name(pkg_name)

  cache_root <- inline_cache_dir()
  pkg_dir <- fs::path(cache_root, hash, "pkg")
  lib_dir <- fs::path(cache_root, hash, "lib")

  cached <- FALSE
  if (cache && fs::dir_exists(lib_dir) && length(fs::dir_ls(lib_dir)) > 0) {
    cached <- TRUE
    if (!quiet) cli::cli_alert_success("Using cached build for {.val {pkg_name}}")
  } else {
    # Scaffold + build
    if (!quiet) cli::cli_alert("Compiling inline Rust code...")

    ensure_vendor_cache(use_local_crates, quiet = quiet)
    scaffold_inline_package(code, hash, features, pkg_name, pkg_rs,
                            cache_root, quiet = quiet)
    build_inline_package(pkg_dir, lib_dir, quiet = quiet)
  }

  fn_names <- load_inline_functions(pkg_name, lib_dir, env)

  invisible(list(
    functions = fn_names,
    dll = as.character(fs::path(lib_dir, pkg_name, "libs")),
    package = pkg_name,
    cached = cached
  ))
}

#' Compile a single inline Rust function
#'
#' Convenience wrapper around [rust_source()] for compiling a single function.
#' Automatically wraps the code with `use` imports.
#'
#' @param code Character string containing a single `#[miniextendr]` function
#'   definition (including the attribute).
#' @param env Environment to load the function into (default: caller's environment).
#' @param ... Additional arguments passed to [rust_source()].
#' @return Invisibly returns the result of [rust_source()].
#' @export
#'
#' @examples
#' \dontrun{
#' rust_function('
#' #[miniextendr]
#' pub fn add_one(x: i32) -> i32 { x + 1 }
#' ')
#' add_one(41L)
#' #> [1] 42
#' }
rust_function <- function(code, env = parent.frame(), ...) {
  # Verify there's at least one pub fn
  fn_names <- extract_pub_fn_names(code)
  if (length(fn_names) == 0) {
    cli::cli_abort(c(
      "No `pub fn` declarations found in code",
      "i" = "rust_function() requires at least one `pub fn` with #[miniextendr]"
    ))
  }

  # Wrap with imports (registration is automatic via #[miniextendr])
  full_code <- paste0(
    "use miniextendr_api::miniextendr;\n\n",
    code, "\n"
  )

  rust_source(code = full_code, env = env, ...)
}

#' Clean inline Rust compilation cache
#'
#' Removes cached inline builds. Call with no arguments to clear all caches,
#' or pass a hash to clear a specific build.
#'
#' @param hash Optional MD5 hash of a specific build to clear. If `NULL`,
#'   clears all cached inline builds.
#' @return Invisibly returns `TRUE`.
#' @export
rust_source_clean <- function(hash = NULL) {
  cache_root <- inline_cache_dir()

  if (!fs::dir_exists(cache_root)) {
    cli::cli_alert_info("No inline build cache found")
    return(invisible(TRUE))
  }

  if (is.null(hash)) {
    entries <- fs::dir_ls(cache_root, type = "directory")
    # Don't delete the vendor/ symlink/dir
    entries <- entries[basename(entries) != "vendor"]
    if (length(entries) == 0) {
      cli::cli_alert_info("Cache is empty")
    } else {
      for (e in entries) fs::dir_delete(e)
      cli::cli_alert_success("Cleared {length(entries)} cached inline build(s)")
    }
  } else {
    target <- fs::path(cache_root, hash)
    if (fs::dir_exists(target)) {
      fs::dir_delete(target)
      cli::cli_alert_success("Cleared cached build {.val {hash}}")
    } else {
      cli::cli_alert_info("No cached build found for hash {.val {hash}}")
    }
  }

  invisible(TRUE)
}


# ---- Internal helpers -------------------------------------------------------

#' Validate rust_source input
#'
#' Ensures exactly one of `file` or `code` is provided and returns the code
#' as a character string.
#'
#' @param file File path or NULL
#' @param code Code string or NULL
#' @return Character string of Rust code
#' @noRd
validate_rust_input <- function(file, code) {
  if (is.null(file) && is.null(code)) {
    cli::cli_abort("One of `file` or `code` must be provided")
  }
  if (!is.null(file) && !is.null(code)) {
    cli::cli_abort("Only one of `file` or `code` may be provided, not both")
  }
  if (!is.null(file)) {
    if (!fs::file_exists(file)) {
      cli::cli_abort("File not found: {.path {file}}")
    }
    code <- paste(readLines(file, warn = FALSE), collapse = "\n")
  }
  code
}

#' Compute cache hash for inline Rust code
#'
#' Generates a deterministic MD5 hash from the normalized code content,
#' features, and minirextendr package version.
#'
#' @param code Character string of Rust code
#' @param features Character vector of cargo features
#' @return MD5 hash string
#' @noRd
compute_inline_hash <- function(code, features = character()) {
  # Normalize: trim, collapse whitespace runs, sort features
  normalized <- trimws(code)
  features_str <- paste(sort(features), collapse = ",")
  version <- as.character(utils::packageVersion("minirextendr"))
  input <- paste(normalized, features_str, version, sep = "\n")

  # Write to temp file for md5sum (it requires a file path)
  tmp <- tempfile("mx_hash_")
  on.exit(unlink(tmp), add = TRUE)
  writeLines(input, tmp)
  unname(tools::md5sum(tmp))
}

#' Get inline build cache directory
#'
#' @return Path to cache root
#' @noRd
inline_cache_dir <- function() {
  fs::path(tools::R_user_dir("minirextendr", "cache"), "rust_source")
}

#' Ensure shared vendor cache exists
#'
#' Creates a shared vendor directory at `{cache_root}/vendor/` by vendoring
#' miniextendr crates. This is symlinked into each inline package.
#'
#' @param use_local_crates Path to local miniextendr repo, or NULL for auto-detect
#' @param quiet Suppress messages
#' @noRd
ensure_vendor_cache <- function(use_local_crates = NULL, quiet = FALSE) {
  cache_root <- inline_cache_dir()
  vendor_dir <- fs::path(cache_root, "vendor")

  # Check if vendor already exists and has crates
  if (fs::dir_exists(vendor_dir)) {
    api_dir <- fs::path(vendor_dir, "miniextendr-api")
    if (fs::dir_exists(api_dir)) {
      return(invisible(vendor_dir))
    }
  }

  if (!quiet) cli::cli_alert("Setting up shared vendor cache...")
  fs::dir_create(cache_root, recurse = TRUE)

  # Auto-detect local crates if not specified
  if (is.null(use_local_crates)) {
    use_local_crates <- detect_inline_local_crates()
  }

  if (!is.null(use_local_crates)) {
    vendor_miniextendr_local(use_local_crates, vendor_dir)
  } else {
    # Download from GitHub
    vendor_miniextendr(
      path = cache_root, version = "main",
      dest = vendor_dir
    )
  }

  # External crates.io deps (syn, proc-macro2, etc.) are resolved by cargo
  # from the network or local cargo cache at build time. No need to vendor
  # them here — we only need the miniextendr crates as path deps.

  invisible(vendor_dir)
}

#' Detect local miniextendr crates for inline builds
#'
#' Checks environment variable and parent directory scan.
#'
#' @return Path to local miniextendr repo, or NULL
#' @noRd
detect_inline_local_crates <- function() {
  # Check MINIEXTENDR_LOCAL env var
  env_path <- Sys.getenv("MINIEXTENDR_LOCAL", unset = "")
  if (nzchar(env_path) && dir.exists(env_path)) {
    if (file.exists(file.path(env_path, "miniextendr-api", "Cargo.toml"))) {
      return(normalizePath(env_path, mustWork = TRUE))
    }
  }

  # Check if minirextendr is installed from the monorepo
  pkg_path <- tryCatch(
    system.file(package = "minirextendr"),
    error = function(e) ""
  )
  if (nzchar(pkg_path)) {
    # Walk up from installed location to find repo
    for (i in seq_len(5)) {
      parent <- dirname(pkg_path)
      if (parent == pkg_path) break
      pkg_path <- parent
      if (file.exists(file.path(pkg_path, "miniextendr-api", "Cargo.toml"))) {
        return(normalizePath(pkg_path, mustWork = TRUE))
      }
    }
  }

  NULL
}


#' Extract pub fn names from Rust code
#'
#' Simple regex extraction of public function names.
#'
#' @param code Rust code string
#' @return Character vector of function names
#' @noRd
extract_pub_fn_names <- function(code) {
  matches <- regmatches(
    code,
    gregexpr("\\bpub\\s+fn\\s+([a-zA-Z_][a-zA-Z0-9_]*)\\s*\\(", code)
  )[[1]]
  sub("^pub\\s+fn\\s+", "", sub("\\s*\\($", "", matches))
}

#' Extract impl block type names from Rust code
#'
#' Simple regex extraction of `impl TypeName` blocks.
#'
#' @param code Rust code string
#' @return Character vector of type names
#' @noRd
extract_impl_names <- function(code) {
  matches <- regmatches(
    code,
    gregexpr("#\\[miniextendr\\][^i]*impl\\s+([A-Z][a-zA-Z0-9_]*)\\s*\\{", code)
  )[[1]]
  if (length(matches) == 0) return(character())
  sub("^.*impl\\s+", "", sub("\\s*\\{$", "", matches))
}

#' Scaffold a temporary inline miniextendr package
#'
#' Creates the complete directory structure for an inline miniextendr
#' package, including all necessary template files.
#'
#' @param code Rust code string
#' @param hash MD5 hash of the code
#' @param features Character vector of cargo features
#' @param pkg_name R package name (e.g., "mxinline12345678")
#' @param pkg_rs Rust-safe package name
#' @param cache_root Path to cache root
#' @param quiet Suppress messages
#' @noRd
scaffold_inline_package <- function(code, hash, features, pkg_name, pkg_rs,
                                     cache_root, quiet = FALSE) {
  pkg_dir <- fs::path(cache_root, hash, "pkg")
  vendor_dir <- fs::path(cache_root, "vendor")

  # Clean up any partial previous build

  if (fs::dir_exists(pkg_dir)) {
    fs::dir_delete(pkg_dir)
  }

  # Create directory structure
  fs::dir_create(fs::path(pkg_dir, "R"), recurse = TRUE)
  fs::dir_create(fs::path(pkg_dir, "src", "rust"), recurse = TRUE)
  fs::dir_create(fs::path(pkg_dir, "inst", "include"), recurse = TRUE)
  fs::dir_create(fs::path(pkg_dir, "tools"), recurse = TRUE)

  # ---- DESCRIPTION ----
  desc_content <- paste0(
    "Package: ", pkg_name, "\n",
    "Title: Inline Rust Compilation\n",
    "Version: 0.0.1\n",
    "Description: Auto-generated package for inline Rust compilation.\n",
    "License: MIT\n",
    "SystemRequirements: Rust (>= 1.85)\n",
    "Encoding: UTF-8\n",
    "Config/build/bootstrap: TRUE\n"
  )
  writeLines(desc_content, fs::path(pkg_dir, "DESCRIPTION"))

  # ---- NAMESPACE ----
  fn_names <- extract_pub_fn_names(code)
  impl_names <- extract_impl_names(code)
  ns_lines <- c(
    paste0('useDynLib(', pkg_name, ', .registration = TRUE)'),
    paste0("export(", fn_names, ")"),
    if (length(impl_names) > 0) paste0("export(", impl_names, ")")
  )
  writeLines(ns_lines, fs::path(pkg_dir, "NAMESPACE"))

  # ---- R/{pkg_name}-package.R ----
  pkg_r <- paste0(
    '#\' @keywords internal\n',
    '"_PACKAGE"\n',
    '\n',
    '## usethis namespace: start\n',
    '#\' @useDynLib ', pkg_name, ', .registration = TRUE\n',
    '## usethis namespace: end\n',
    'NULL\n'
  )
  writeLines(pkg_r, fs::path(pkg_dir, "R", paste0(pkg_name, "-package.R")))

  # ---- Rust source: lib.rs ----
  # Prepend miniextendr_init!() macro invocation (required for R_init_* entry point)
  lib_rs_content <- paste0(
    "miniextendr_api::miniextendr_init!(", pkg_rs, ");\n\n",
    code, "\n"
  )
  writeLines(lib_rs_content, fs::path(pkg_dir, "src", "rust", "lib.rs"))

  # ---- Cargo.toml ----
  features_toml <- if (length(features) > 0) {
    feat_list <- paste0('"', features, '"', collapse = ", ")
    paste0('default = [', feat_list, ']\n')
  } else {
    'default = []\n'
  }

  cargo_toml <- paste0(
    '[package]\n',
    'name = "', pkg_rs, '"\n',
    'version = "0.1.0"\n',
    'edition = "2024"\n',
    'publish = false\n',
    '\n',
    '[workspace]\n',
    '\n',
    '[lib]\n',
    'path = "lib.rs"\n',
    'crate-type = ["staticlib"]\n',
    '\n',
    '[features]\n',
    features_toml,
    'nonapi = ["miniextendr-api/nonapi"]\n',
    'connections = ["miniextendr-api/connections"]\n',
    '\n',
    '[dependencies]\n',
    'miniextendr-api = { path = "../../vendor/miniextendr-api" }\n',
    '\n',
    '[build-dependencies]\n',
    'miniextendr-lint = { path = "../../vendor/miniextendr-lint" }\n'
  )
  writeLines(cargo_toml, fs::path(pkg_dir, "src", "rust", "Cargo.toml"))

  # ---- build.rs ----
  writeLines("fn main() {\n    miniextendr_lint::build_script();\n}",
             fs::path(pkg_dir, "src", "rust", "build.rs"))

  # ---- Template files from minirextendr templates ----
  set_template_type("rpkg")

  # stub.c — minimal C file so R's build system produces a shared library
  stub_src <- template_path("stub.c")
  fs::file_copy(stub_src, fs::path(pkg_dir, "src", "stub.c"), overwrite = TRUE)

  # mx_abi.h header
  mx_abi_h <- template_path("mx_abi.h", subdir = "inst_include")
  # Template uses {{package}} — do simple substitution
  h_content <- readLines(mx_abi_h, warn = FALSE)
  h_content <- gsub("\\{\\{package\\}\\}", pkg_name, h_content)
  writeLines(h_content, fs::path(pkg_dir, "inst", "include", "mx_abi.h"))

  # cargo-config.toml.in
  cargo_config_in <- template_path("cargo-config.toml.in")
  fs::file_copy(cargo_config_in,
                fs::path(pkg_dir, "src", "rust", "cargo-config.toml.in"))

  # win.def.in (needed by configure as input for AC_CONFIG_FILES)
  win_def_in <- template_path("win.def.in")
  fs::file_copy(win_def_in, fs::path(pkg_dir, "src", "win.def.in"))

  # Makevars.in
  makevars_in <- template_path("Makevars.in")
  fs::file_copy(makevars_in, fs::path(pkg_dir, "src", "Makevars.in"))

  # configure.ac
  configure_ac <- template_path("configure.ac")
  ac_content <- readLines(configure_ac, warn = FALSE)
  ac_content <- gsub("\\{\\{\\{features_var\\}\\}\\}", paste0(toupper(pkg_rs), "_FEATURES"),
                      ac_content)
  ac_content <- gsub("\\{\\{package\\}\\}", pkg_name, ac_content)
  writeLines(ac_content, fs::path(pkg_dir, "configure.ac"))

  # tools/config.guess and tools/config.sub
  config_guess <- script_path("config.guess")
  config_sub <- script_path("config.sub")
  fs::file_copy(config_guess, fs::path(pkg_dir, "tools", "config.guess"))
  fs::file_copy(config_sub, fs::path(pkg_dir, "tools", "config.sub"))
  fs::file_chmod(fs::path(pkg_dir, "tools", "config.guess"), "755")
  fs::file_chmod(fs::path(pkg_dir, "tools", "config.sub"), "755")

  # tools/vendor-local.R (standalone workspace vendor script)
  vendor_local_src <- template_path("vendor-local.R", subdir = "tools")
  if (fs::file_exists(vendor_local_src)) {
    fs::file_copy(vendor_local_src, fs::path(pkg_dir, "tools", "vendor-local.R"))
  }

  # Symlink vendor/ into the package
  pkg_vendor <- fs::path(pkg_dir, "vendor")
  if (!fs::link_exists(pkg_vendor) && !fs::dir_exists(pkg_vendor)) {
    fs::link_create(vendor_dir, pkg_vendor)
  }

  invisible(pkg_dir)
}

#' Build an inline miniextendr package
#'
#' Runs autoconf, configure, and R CMD INSTALL on the scaffolded package.
#'
#' @param pkg_dir Path to the scaffolded package
#' @param lib_dir Path to install the package into
#' @param quiet Suppress messages
#' @noRd
build_inline_package <- function(pkg_dir, lib_dir, quiet = FALSE) {
  fs::dir_create(lib_dir, recurse = TRUE)

  # Step 1: autoconf
  if (!quiet) cli::cli_alert("Running autoconf...")
  result <- run_with_logging(
    "autoconf", args = c("-v", "-i", "-f"),
    log_prefix = "inline-autoconf",
    wd = as.character(pkg_dir)
  )
  check_result(result, "autoconf (inline)")
  fs::file_chmod(fs::path(pkg_dir, "configure"), "755")

  # Step 2: configure (dev mode)
  if (!quiet) cli::cli_alert("Running configure...")
  result <- run_with_logging(
    "bash", args = c("./configure"),
    log_prefix = "inline-configure",
    wd = as.character(pkg_dir),
    env = c(NOT_CRAN = "true")
  )
  check_result(result, "configure (inline)")

  # Step 3: R CMD INSTALL
  if (!quiet) cli::cli_alert("Building package...")
  r_cmd <- file.path(R.home("bin"), "R")
  result <- run_with_logging(
    r_cmd,
    args = c("CMD", "INSTALL",
             paste0("--library=", as.character(lib_dir)),
             "--no-test-load",
             as.character(pkg_dir)),
    log_prefix = "inline-install",
    wd = as.character(pkg_dir),
    env = c(NOT_CRAN = "true")
  )
  check_result(result, "R CMD INSTALL (inline)")

  if (!quiet) cli::cli_alert_success("Build complete")
  invisible(lib_dir)
}

#' Load functions from an inline package into an environment
#'
#' Loads the compiled package and copies exported functions into the target
#' environment.
#'
#' @param pkg_name Package name
#' @param lib_dir Library directory containing the installed package
#' @param env Target environment
#' @return Character vector of exported function names
#' @noRd
load_inline_functions <- function(pkg_name, lib_dir, env) {
  # Load the package
  library(pkg_name, lib.loc = as.character(lib_dir), character.only = TRUE,
          warn.conflicts = FALSE)

  # Get the package namespace
  ns <- asNamespace(pkg_name)

  # Get exported names
  exports <- getNamespaceExports(pkg_name)

  # Copy exports to the target environment
  for (name in exports) {
    obj <- get(name, envir = ns)
    assign(name, obj, envir = env)
  }

  exports
}
