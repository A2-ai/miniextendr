# Native R package integration for miniextendr
#
# Programmatic bindgen invocation: resolves include paths, detects C/C++ mode,
# applies R_NO_REMAP, -isysroot, --blocklist-file for known-problematic deps,
# and generates Rust FFI + C shim files.

# =============================================================================
# Public API
# =============================================================================

#' Add a native R package dependency for Rust FFI
#'
#' Configures your miniextendr package to use the C/C++ headers from an
#' installed R package. Runs bindgen to generate Rust FFI bindings and C shim
#' files for static inline functions.
#'
#' @param pkg Character string. Name of the R package to link to.
#' @param headers Character vector of header file paths relative to the
#'   package's `include/` directory (e.g., `"cli/progress.h"`). If `NULL`
#'   (default), discovers all `.h`/`.hpp` files automatically.
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param allowlist_pattern Optional regex pattern for bindgen `--allowlist-file`.
#'   Defaults to matching files under the package's include directory.
#'
#' @return Invisibly returns a list with paths to the generated files.
#' @export
use_native_package <- function(pkg,
                                headers = NULL,
                                path = ".",
                                allowlist_pattern = NULL) {
  with_project(path)

  # Check bindgen is installed
  assert_bindgen_installed()

  # Check for known-unsupported packages
  warn_known_bad_package(pkg)

  # Validate package is installed and has headers
  pkg_info <- discover_native_package(pkg)
  if (!pkg_info$has_include) {
    cli::cli_abort(c(
      "Package {.pkg {pkg}} does not have an {.path include/} directory",
      "i" = "Only packages with {.path inst/include/} headers can be used"
    ))
  }

  # 1. Add LinkingTo + Imports in DESCRIPTION
  add_linking_to(pkg)

  # 2. Discover headers if not specified
  if (is.null(headers)) {
    headers <- discover_native_headers(pkg, pkg_info$include_path)
    if (length(headers) == 0) {
      cli::cli_warn("No header files found in {.pkg {pkg}} include directory")
      return(invisible(NULL))
    }
    cli::cli_alert_info("Found {length(headers)} header(s) in {.pkg {pkg}}")
  }

  # 3. Resolve bindgen invocation arguments programmatically
  bindgen_args <- resolve_bindgen_args(pkg, pkg_info, headers, allowlist_pattern)

  # 4. Generate wrapper header
  wrapper_path <- write_wrapper_header(pkg, bindgen_args)

  # 5. Run bindgen
  result <- run_bindgen(pkg, wrapper_path, bindgen_args)

  # 6. Fix C shim include path (bindgen writes absolute paths)
  if (result$success && !is.null(result$static_wrappers_c)) {
    fix_shim_include(result$static_wrappers_c, wrapper_path)
  }

  # 7. Add include detection to configure.ac
  add_native_to_configure_ac(pkg)

  # 8. Add mod declaration to lib.rs
  add_native_mod_to_lib_rs(pkg)

  # 9. Report
  if (result$success) {
    cli::cli_alert_success("Generated Rust FFI bindings for {.pkg {pkg}}")
    if (!is.null(result$static_wrappers_c)) {
      cli::cli_alert_info("C shim file: {.path {result$static_wrappers_c}}")
    }
  } else {
    cli::cli_warn("bindgen failed for {.pkg {pkg}}: {result$error}")
  }

  invisible(result)
}

#' Check if bindgen can parse a package's headers
#'
#' Dry-run: resolves all args and invokes bindgen without modifying the project.
#'
#' @param pkg Character string. Package name.
#' @return List with `package`, `success`, `n_lines`, `has_static_fns`,
#'   `n_headers`, `mode`, `error`.
#' @export
check_native_package <- function(pkg) {
  assert_bindgen_installed()
  info <- discover_native_package(pkg)

  if (!info$has_include) {
    return(list(package = pkg, success = FALSE, n_lines = 0L,
                has_static_fns = FALSE, n_headers = 0L,
                mode = NA_character_, error = "no include/ directory"))
  }

  headers <- discover_native_headers(pkg, info$include_path)
  if (length(headers) == 0) {
    return(list(package = pkg, success = FALSE, n_lines = 0L,
                has_static_fns = FALSE, n_headers = 0L,
                mode = NA_character_, error = "no header files"))
  }

  args <- resolve_bindgen_args(pkg, info, headers, allowlist_pattern = NULL)

  wrapper <- tempfile(fileext = ".h")
  on.exit(unlink(wrapper), add = TRUE)
  write_wrapper_header_to(wrapper, args)

  ffi_out <- tempfile(fileext = ".rs")
  static_out <- tempfile(fileext = ".c")
  on.exit(unlink(c(ffi_out, static_out)), add = TRUE)

  result <- invoke_bindgen(wrapper, ffi_out, static_out, args)
  used_mode <- args$mode
  last_std <- args$cxx_std

  # C failed -> retry C++17
  if (!result$success && args$mode == "c") {
    args_cpp <- args
    args_cpp$mode <- "cpp"
    args_cpp$cxx_std <- "c++17"
    last_std <- "c++17"
    result <- invoke_bindgen(wrapper, ffi_out, static_out, args_cpp)
    if (result$success) used_mode <- "cpp"
  }

  # C++17 failed -> retry C++14
  if (!result$success && identical(last_std, "c++17")) {
    args_14 <- args
    args_14$mode <- "cpp"
    args_14$cxx_std <- "c++14"
    result <- invoke_bindgen(wrapper, ffi_out, static_out, args_14)
    if (result$success) used_mode <- "cpp14"
  }

  list(
    package = pkg,
    success = result$success,
    n_lines = if (result$success) length(readLines(ffi_out, warn = FALSE)) else 0L,
    has_static_fns = file.exists(static_out) && file.size(static_out) > 0,
    n_headers = length(headers),
    mode = used_mode,
    error = result$error
  )
}

# =============================================================================
# Argument resolution -- the core programmatic logic
# =============================================================================

#' Resolve all bindgen arguments for a package
#'
#' Determines include paths, C/C++ mode, clang flags, and blocklists
#' from the installed package metadata. This is purely deterministic given
#' the installed packages -- no user input needed.
#'
#' @param pkg Package name
#' @param pkg_info Result of `discover_native_package()`
#' @param headers Character vector of header paths relative to include/
#' @param allowlist_pattern Optional regex for --allowlist-file
#' @return List with all resolved arguments for bindgen
#' @noRd
resolve_bindgen_args <- function(pkg, pkg_info, headers,
                                  allowlist_pattern = NULL) {
  # -- Include paths --
  include_paths <- resolve_include_paths(pkg, pkg_info$include_path)

  # -- Detect C++ mode --
  mode <- detect_header_mode(pkg_info$include_path)

  # -- C++ standard --
  # Default to c++17; c++14 fallback handled in run_bindgen
  cxx_std <- if (mode == "cpp") "c++17" else NULL

  # -- macOS SDK path (for C++ stdlib) --
  isysroot <- detect_sdk_path()

  # -- Blocklist files (known-problematic deep deps) --
  blocklist_files <- resolve_blocklist_files(pkg)

  # -- Allowlist pattern --
  if (is.null(allowlist_pattern)) {
    # Default: match files under the package's own include dir
    pkg_escaped <- gsub("\\.", "\\\\.", pkg)
    allowlist_pattern <- paste0(".*", pkg_escaped, "/.*")
  }

  list(
    pkg = pkg,
    include_paths = include_paths,
    r_include = R.home("include"),
    mode = mode,
    cxx_std = cxx_std,
    isysroot = isysroot,
    blocklist_files = blocklist_files,
    allowlist_pattern = allowlist_pattern,
    headers = headers,
    wrapper_defines = "R_NO_REMAP"
  )
}

#' Resolve include paths: package + recursive LinkingTo deps
#'
#' Walks the LinkingTo dependency tree recursively so that transitive
#' deps (e.g., mlpack -> RcppArmadillo -> Rcpp + BH) are all included.
#' @noRd
resolve_include_paths <- function(pkg, pkg_include_path) {
  paths <- pkg_include_path
  visited <- pkg

  # BFS through LinkingTo deps
  queue <- pkg
  while (length(queue) > 0) {
    current <- queue[[1]]
    queue <- queue[-1]

    desc_path <- system.file("DESCRIPTION", package = current)
    if (!nzchar(desc_path)) next

    desc <- read.dcf(desc_path)
    if (!("LinkingTo" %in% colnames(desc))) next

    lt_raw <- desc[1, "LinkingTo"]
    deps <- trimws(strsplit(lt_raw, ",")[[1]])
    deps <- sub("\\s*\\(.*", "", deps)

    for (dep in deps) {
      if (dep %in% visited) next
      visited <- c(visited, dep)

      dep_include <- system.file("include", package = dep)
      if (nzchar(dep_include)) {
        paths <- c(paths, dep_include)
      }
      # Enqueue for recursive resolution
      queue <- c(queue, dep)
    }
  }

  unique(paths)
}

#' Detect whether headers need C++ mode
#'
#' Returns "c" for pure C headers, "cpp" for C++ headers.
#' Pure C mode enables `--wrap-static-fns` (static inline shim generation),
#' which doesn't work in C++ mode. So we prefer C when possible and
#' fall back to C++ when needed.
#'
#' Strategy: try C first. If bindgen fails, the caller retries in C++ mode.
#' @noRd
detect_header_mode <- function(include_path) {
  # Any .hpp/.hh/.hxx -> must use C++
  cpp_ext <- list.files(include_path, pattern = "\\.(hpp|hh|hxx)$",
                        recursive = TRUE)
  if (length(cpp_ext) > 0) return("cpp")

  # Check .h files for C++ constructs
  h_files <- utils::head(list.files(include_path, pattern = "\\.h$",
                                    recursive = TRUE, full.names = TRUE), 20)

  cxx_indicators <- paste(c(
    "#include\\s*<(string|vector|map|set|list|deque|array|tuple)>",
    "#include\\s*<(iostream|fstream|sstream|ostream)>",
    "#include\\s*<(memory|algorithm|functional|numeric|utility)>",
    "#include\\s*<(cmath|cstddef|cstdint|cstdlib|cstring|cassert)>",
    "#include\\s*<(thread|mutex|atomic|future)>",
    "#include\\s*<(typeinfo|stdexcept|type_traits)>",
    "namespace\\s+\\w+\\s*\\{",
    "template\\s*<",
    "class\\s+\\w+\\s*(:|\\{)"
  ), collapse = "|")

  for (f in h_files) {
    lines <- tryCatch(readLines(f, n = 200, warn = FALSE),
                      error = function(e) character())
    if (any(grepl(cxx_indicators, lines))) return("cpp")
  }

  "c"
}

#' Detect macOS SDK path for C++ stdlib
#' @noRd
detect_sdk_path <- function() {
  if (.Platform$OS.type != "unix") return(NULL)
  if (Sys.info()[["sysname"]] != "Darwin") return(NULL)

  tryCatch({
    path <- system2("xcrun", "--show-sdk-path", stdout = TRUE, stderr = FALSE)
    if (length(path) == 1 && nzchar(path)) path else NULL
  }, error = function(e) NULL)
}

#' Resolve --blocklist-file patterns for known-problematic deep deps
#'
#' Some header libraries contain anonymous types or constructs that crash
#' bindgen. We blocklist their internal headers while still allowing the
#' target package to reference their types opaquely.
#' Uses the same recursive BFS as resolve_include_paths.
#' @noRd
resolve_blocklist_files <- function(pkg) {
  # Collect ALL transitive deps via BFS
  all_deps <- character()
  visited <- pkg
  queue <- pkg
  while (length(queue) > 0) {
    current <- queue[[1]]
    queue <- queue[-1]
    desc_path <- system.file("DESCRIPTION", package = current)
    if (!nzchar(desc_path)) next
    desc <- read.dcf(desc_path)
    if (!("LinkingTo" %in% colnames(desc))) next
    lt_raw <- desc[1, "LinkingTo"]
    deps <- trimws(strsplit(lt_raw, ",")[[1]])
    deps <- sub("\\s*\\(.*", "", deps)
    for (dep in deps) {
      if (dep %in% visited) next
      visited <- c(visited, dep)
      all_deps <- c(all_deps, dep)
      queue <- c(queue, dep)
    }
  }

  blocklist <- character()

  # Boost (BH): anonymous structs in internal headers cause
  # bindgen panic: "/*<unnamed>*/" is not a valid Ident
  if ("BH" %in% all_deps) {
    blocklist <- c(blocklist, ".*/boost/.*")
  }

  # wdm: same issue via boost transitive includes
  if ("wdm" %in% all_deps) {
    blocklist <- c(blocklist, ".*/wdm/.*")
  }

  blocklist
}

# =============================================================================
# Prerequisites
# =============================================================================

#' Assert that bindgen CLI is installed
#' @noRd
assert_bindgen_installed <- function() {
  if (!nzchar(Sys.which("bindgen"))) {
    cli::cli_abort(c(
      "{.strong bindgen} is not installed",
      "i" = "Install it with: {.code cargo install --force --locked bindgen-cli}",
      "i" = "bindgen generates Rust FFI bindings from C/C++ headers"
    ))
  }
}

# =============================================================================
# Known-bad package warnings
# =============================================================================

#' Warn if a package is known not to work with bindgen
#' @noRd
warn_known_bad_package <- function(pkg) {
  # Rcpp/cpp11 ecosystem: headers exist but are C++ framework internals,
  # not standalone C APIs. Using them from Rust requires the full Rcpp runtime.
  rcpp_ecosystem <- c(
    "Rcpp", "RcppArmadillo", "RcppEigen", "RcppParallel", "RcppThread",
    "RcppProgress", "RcppSpdlog", "RcppSimdJson", "RcppFastAD",
    "RcppFastFloat", "RcppMsgPack", "RcppBigIntAlgos", "RcppNumerical",
    "RcppSMC", "RcppZiggurat", "RcppClassic", "RcppDist", "RcppDate",
    "RcppHNSW", "RcppML", "RcppGSL", "RcppAnnoy", "RcppBDT", "RcppCCTZ",
    "RcppTOML", "RcppXsimd", "RcppBessel", "RcppInt64", "RcppHungarian",
    "RcppPlanc", "RcppTN", "RcppColors", "RcppArray", "RcppQuantuccia",
    "RcppEnsmallen", "RcppAlgos", "RcppMagicEnum", "RcppTskit",
    "RcppEigenAD", "RcppCWB",
    "cpp11", "cpp11armadillo", "cpp11eigen", "cpp4r",
    "bindrcpp", "cppcontainers", "tidyCpp", "rcpptimer"
  )
  if (pkg %in% rcpp_ecosystem) {
    cli::cli_abort(c(
      "{.pkg {pkg}} is part of the Rcpp/cpp11 ecosystem",
      "x" = "These packages provide C++ framework headers, not standalone C APIs",
      "i" = "bindgen cannot wrap Rcpp template metaprogramming"
    ))
  }

  # Packages whose headers require Rcpp at #include time but aren't
  # in the Rcpp ecosystem themselves
  rcpp_dependent <- c(
    "RViennaCL", "profoc", "MPCR", "spatialwidget", "ggdmcHeaders",
    "rgen", "PartialNetwork", "magi"
  )
  if (pkg %in% rcpp_dependent) {
    cli::cli_warn(c(
      "{.pkg {pkg}} headers depend on Rcpp internally",
      "!" = "bindgen may fail or produce incomplete bindings",
      "i" = "The package's C++ headers #include <Rcpp.h> directly"
    ))
  }

  # Packages with inst/include/ but no actual header files
  no_headers <- c(
    "armspp", "makemyprior", "multiSA", "noweb",
    "recmap", "rswipl", "salmonMSE", "zigg"
  )
  if (pkg %in% no_headers) {
    cli::cli_abort(c(
      "{.pkg {pkg}} has an {.path include/} directory but no header files",
      "i" = "The directory may contain non-header resources"
    ))
  }

  # Packages needing system libraries not available via R
  needs_system_lib <- c(
    "HighFive",     # HDF5
    "sf",           # GDAL, GEOS, PROJ
    "vapour",       # GDAL
    "tiledb",       # TileDB
    "libimath",     # OpenGL
    "libopenexr"    # Imath
  )
  if (pkg %in% needs_system_lib) {
    cli::cli_warn(c(
      "{.pkg {pkg}} headers require system libraries not available via R",
      "!" = "bindgen may fail with missing header errors",
      "i" = "Install the required system libraries first"
    ))
  }
}

# =============================================================================
# Discovery
# =============================================================================

#' Discover an installed R package's native resources
#' @noRd
discover_native_package <- function(pkg) {
  include_path <- system.file("include", package = pkg)
  libs_path <- system.file("libs", package = pkg)

  list(
    include_path = include_path,
    libs_path = libs_path,
    has_include = nzchar(include_path),
    has_libs = nzchar(libs_path)
  )
}

#' Discover header files in a package's include directory
#' @noRd
discover_native_headers <- function(pkg, include_path) {
  list.files(include_path, pattern = "\\.(h|hpp|hh|hxx)$",
             recursive = TRUE, full.names = FALSE)
}

# =============================================================================
# DESCRIPTION management
# =============================================================================

#' Add a package to LinkingTo in DESCRIPTION
#' @noRd
add_linking_to <- function(pkg) {
  desc_path <- usethis::proj_path("DESCRIPTION")
  if (!fs::file_exists(desc_path)) {
    cli::cli_abort("DESCRIPTION file not found")
  }

  deps <- mx_desc_get_deps(desc_path)

  # Add to LinkingTo if missing
  linking_to <- deps[deps$type == "LinkingTo", ]
  if (!(pkg %in% linking_to$package)) {
    mx_desc_set_dep(desc_path, pkg, type = "LinkingTo")
    cli::cli_alert_success("Added {.pkg {pkg}} to LinkingTo")
  }

  # Add to Imports if missing (needed to load the DLL for R_GetCCallable)
  imports <- deps[deps$type == "Imports", ]
  if (!(pkg %in% imports$package)) {
    mx_desc_set_dep(desc_path, pkg, type = "Imports")
    cli::cli_alert_success("Added {.pkg {pkg}} to Imports")
  }

  invisible(TRUE)
}

# =============================================================================
# configure.ac generation
# =============================================================================

#' Append native package include detection to configure.ac
#'
#' Adds an m4 block that resolves the package's include path at configure time
#' and appends it to NATIVE_PKG_CPPFLAGS.
#' @noRd
add_native_to_configure_ac <- function(pkg) {
  configure_ac <- usethis::proj_path("configure.ac")
  if (!file.exists(configure_ac)) return(invisible())

  lines <- readLines(configure_ac, warn = FALSE)

  # Check if this package is already in configure.ac
  pkg_marker <- paste0("dnl native: ", pkg)
  if (any(grepl(pkg_marker, lines, fixed = TRUE))) {
    cli::cli_alert_info("{.pkg {pkg}} already in configure.ac")
    return(invisible())
  }

  # Build the detection block
  pkg_upper <- toupper(gsub("[.-]", "_", pkg))
  block <- c(
    pkg_marker,
    sprintf('%s_INCLUDE=$("${R_HOME}/bin/Rscript" -e "cat(system.file(\'include\', package=\'%s\'))")',
            pkg_upper, pkg),
    sprintf('if test -n "$%s_INCLUDE" && test -d "$%s_INCLUDE"; then',
            pkg_upper, pkg_upper),
    sprintf('  NATIVE_PKG_CPPFLAGS="$NATIVE_PKG_CPPFLAGS -I$%s_INCLUDE"', pkg_upper),
    sprintf('  AC_MSG_NOTICE([%s include: $%s_INCLUDE])', pkg, pkg_upper),
    "else",
    sprintf('  AC_MSG_WARN([%s package not found])', pkg),
    "fi"
  )

  # Find the insertion point: after the MINIREXTENDR: native-pkg-cppflags
  # insertion marker, or otherwise before AC_SUBST([NATIVE_PKG_CPPFLAGS]).
  marker_idx <- grep("MINIREXTENDR: native-pkg-cppflags", lines, fixed = TRUE)
  subst_idx <- grep("AC_SUBST.*NATIVE_PKG_CPPFLAGS", lines)

  if (length(marker_idx) > 0) {
    insert_at <- marker_idx[1]
  } else if (length(subst_idx) > 0) {
    insert_at <- subst_idx[1] - 1
  } else {
    # No NATIVE_PKG_CPPFLAGS section -- need to add one
    srcdir_idx <- grep("AC_CONFIG_SRCDIR", lines)
    if (length(srcdir_idx) == 0) {
      cli::cli_warn("Could not find insertion point in configure.ac")
      return(invisible())
    }
    # Add the full section before AC_CONFIG_SRCDIR
    section <- c(
      "dnl ---- Native R package include paths ----",
      'NATIVE_PKG_CPPFLAGS=""',
      block,
      "AC_SUBST([NATIVE_PKG_CPPFLAGS])",
      ""
    )
    lines <- append(lines, section, after = srcdir_idx[1] - 1)
    writeLines(lines, configure_ac)
    cli::cli_alert_success("Added {.pkg {pkg}} include detection to configure.ac")
    return(invisible())
  }

  lines <- append(lines, block, after = insert_at)
  writeLines(lines, configure_ac)
  cli::cli_alert_success("Added {.pkg {pkg}} include detection to configure.ac")
}

# =============================================================================
# Wrapper header generation
# =============================================================================

#' Write wrapper header to the project's src/ directory
#' @noRd
write_wrapper_header <- function(pkg, args) {
  pkg_rs <- gsub("[.-]", "_", pkg)
  wrapper_path <- usethis::proj_path("src", paste0(pkg_rs, "_wrapper.h"))
  write_wrapper_header_to(wrapper_path, args)
  cli::cli_alert_success("Created wrapper header: {.path {wrapper_path}}")
  wrapper_path
}

#' Write wrapper header content to a path
#' @noRd
write_wrapper_header_to <- function(path, args) {
  lines <- c(
    paste0("/* Wrapper header for bindgen: ", args$pkg, " */"),
    "/* Generated by minirextendr::use_native_package() */",
    "",
    # R_NO_REMAP must come before Rinternals.h
    paste0("#define ", args$wrapper_defines),
    # Use r_shim.h instead of <Rinternals.h> directly: the shim wraps the
    # include in a scoped #pragma clang diagnostic push/pop to suppress
    # clang 21+'s -Wunknown-warning-option meta-warning for R's Boolean.h.
    # Putting -Wno-unknown-warning-option in PKG_CFLAGS triggers an R CMD
    # check --as-cran WARNING. See issue #443.
    "#include \"r_shim.h\"",
    "",
    vapply(utils::head(args$headers, 20), function(h) {
      paste0("#include <", h, ">")
    }, character(1))
  )
  writeLines(lines, path)
}

# =============================================================================
# Bindgen invocation
# =============================================================================

#' Run bindgen with resolved args, with c++14 fallback
#' @noRd
run_bindgen <- function(pkg, wrapper_path, args) {
  pkg_rs <- gsub("[.-]", "_", pkg)
  ffi_rs <- usethis::proj_path("src", "rust", "native", paste0(pkg_rs, "_ffi.rs"))
  static_c <- usethis::proj_path("src", paste0(pkg_rs, "_static_wrappers.c"))

  native_dir <- dirname(ffi_rs)
  if (!dir.exists(native_dir)) dir.create(native_dir, recursive = TRUE)

  # Try primary mode (C or C++ depending on detection)
  result <- invoke_bindgen(wrapper_path, ffi_rs, static_c, args)
  last_std <- args$cxx_std  # track which standard was last attempted

  # Fallback: C failed -> retry as C++17 (many .h files have C++ includes)
  if (!result$success && args$mode == "c") {
    args_cpp <- args
    args_cpp$mode <- "cpp"
    args_cpp$cxx_std <- "c++17"
    last_std <- "c++17"
    result <- invoke_bindgen(wrapper_path, ffi_rs, static_c, args_cpp)
  }

  # Fallback: C++17 failed -> retry as C++14 (deprecated APIs like auto_ptr)
  if (!result$success && identical(last_std, "c++17")) {
    args_14 <- args
    args_14$mode <- "cpp"
    args_14$cxx_std <- "c++14"
    result <- invoke_bindgen(wrapper_path, ffi_rs, static_c, args_14)
  }

  if (result$success) {
    # Prepend module-level allows and use statement
    # (--raw-line has shell quoting issues with system2, so we prepend here)
    content <- readLines(ffi_rs, warn = FALSE)
    content <- c(
      "// automatically generated by bindgen -- do not edit",
      "#![allow(unused, non_camel_case_types, non_upper_case_globals, clippy::all)]",
      "",
      "use miniextendr_api::ffi::SEXP;",
      "",
      content
    )
    writeLines(content, ffi_rs)
  }

  result$ffi_rs <- if (result$success) ffi_rs else NULL
  result$static_wrappers_c <- if (result$success && file.exists(static_c) &&
                                   file.size(static_c) > 0) static_c else NULL
  result
}

#' Invoke bindgen with specific args
#' @noRd
invoke_bindgen <- function(wrapper_path, ffi_out, static_out, args) {
  bindgen_path <- Sys.which("bindgen")
  if (!nzchar(bindgen_path)) {
    return(list(success = FALSE,
                error = "bindgen not found (cargo install --force --locked bindgen-cli)"))
  }

  # Build bindgen CLI args
  cli_args <- c(
    "--merge-extern-blocks",
    "--no-layout-tests",
    "--no-doc-comments",
    "--wrap-static-fns",
    "--wrap-static-fns-path", static_out,
    "--blocklist-type", "SEXPREC",
    "--blocklist-type", "SEXP"
  )

  if (args$mode == "cpp") {
    cli_args <- c(cli_args, "--enable-cxx-namespaces")
  }

  for (bl in args$blocklist_files) {
    cli_args <- c(cli_args, "--blocklist-file", bl)
  }

  cli_args <- c(cli_args, wrapper_path, "--")

  # Clang args
  if (args$mode == "cpp" && !is.null(args$cxx_std)) {
    cli_args <- c(cli_args, "-x", "c++", paste0("-std=", args$cxx_std))
  }
  if (!is.null(args$isysroot)) {
    cli_args <- c(cli_args, "-isysroot", args$isysroot)
  }

  cli_args <- c(cli_args, paste0("-I", args$r_include))
  for (inc in args$include_paths) {
    cli_args <- c(cli_args, paste0("-I", inc))
  }

  # Run
  output <- tryCatch({
    system2(bindgen_path, args = cli_args, stdout = TRUE, stderr = TRUE)
  }, error = function(e) {
    return(list(success = FALSE, error = conditionMessage(e)))
  })

  if (is.list(output)) return(output)

  status <- attr(output, "status")
  if (!is.null(status) && status != 0) {
    err_lines <- output[grepl("error:|panic", output)]
    err_lines <- err_lines[!grepl("warning:", err_lines)]
    return(list(success = FALSE,
                error = paste(utils::head(err_lines, 3), collapse = "; ")))
  }

  writeLines(output, ffi_out)
  list(success = TRUE, error = NULL)
}

# =============================================================================
# Post-processing
# =============================================================================

#' Fix the C shim's #include to use a relative path
#' @noRd
fix_shim_include <- function(shim_path, wrapper_path) {
  if (!file.exists(shim_path)) return()
  lines <- readLines(shim_path, warn = FALSE)
  wrapper_name <- basename(wrapper_path)
  lines <- sub('^#include ".*"', paste0('#include "', wrapper_name, '"'), lines)
  writeLines(lines, shim_path)
}

#' Add a native FFI module declaration to lib.rs
#' @noRd
add_native_mod_to_lib_rs <- function(pkg) {
  pkg_rs <- gsub("[.-]", "_", pkg)
  mod_name <- paste0(pkg_rs, "_ffi")

  lib_rs <- usethis::proj_path("src", "rust", "lib.rs")
  if (!file.exists(lib_rs)) return(invisible())

  lines <- readLines(lib_rs, warn = FALSE)

  # Check if mod already declared
  if (any(grepl(paste0("mod\\s+", mod_name), lines))) return(invisible())

  # Find or create native.rs module
  native_rs <- usethis::proj_path("src", "rust", "native.rs")
  if (!file.exists(native_rs)) {
    writeLines(c(
      "// Native R package FFI bindings",
      "// Generated by minirextendr::use_native_package()",
      ""
    ), native_rs)

    # Add mod native; to lib.rs if not present
    if (!any(grepl("mod\\s+native;", lines))) {
      native_marker <- "// Native R package FFI bindings"
      lines <- c(lines, "", native_marker, "mod native;")
      writeLines(lines, lib_rs)
      cli::cli_alert_success("Added {.code mod native} to lib.rs")
    }
  }

  # Add mod declaration to native.rs
  native_lines <- readLines(native_rs, warn = FALSE)
  mod_line <- paste0("pub mod ", mod_name, ";")
  if (!any(grepl(mod_line, native_lines, fixed = TRUE))) {
    native_lines <- c(native_lines, mod_line)
    writeLines(native_lines, native_rs)
    cli::cli_alert_success("Added {.code {mod_line}} to native.rs")
  }
}
