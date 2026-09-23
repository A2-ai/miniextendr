# Robustness matrix for the base-R path-dependency stager (#1580). Each case is
# a git-tracked monorepo whose R crate lives at pkg/src/rust. A supported case
# stages, relocates under an unrelated ancestor workspace, and must build
# offline against the checkout's lockfile; an unsupported one must stop before
# staging with an error that names the fix.

# region: fixture builders

stager_helper <- function() {
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  helper
}

# A library crate at `dir`; `...` are manifest lines after the package keys.
crate <- function(dir, ..., name = basename(dir), version = "0.1.0", lib = "pub fn value() -> i32 { 1 }",
                  head = c(sprintf('name = "%s"', name), sprintf('version = "%s"', version), 'edition = "2021"')) {
  stats::setNames(list(c("[package]", head, ...), lib), file.path(dir, c("Cargo.toml", "src/lib.rs")))
}

# The R crate; `...` are manifest lines after its [lib] table.
rcrate <- function(..., lib = "pub fn g() -> i32 { alpha::value() }", workspace = "[workspace]") {
  stats::setNames(list(c("[package]", 'name = "pkg-r"', 'version = "0.1.0"', 'edition = "2021"',
                         "publish = false", workspace, "[lib]", 'path = "lib.rs"', ...), lib),
                  c("pkg/src/rust/Cargo.toml", "pkg/src/rust/lib.rs"))
}

featured <- 'pub fn value() -> i32 { 1 }\n#[cfg(feature = "extra")]\npub fn extra() -> i32 { 2 }'
features <- c("[features]", 'default = ["std"]', "std = []", "extra = []")
member_head <- function(name) c(sprintf('name = "%s"', name), "version.workspace = true", "edition.workspace = true")

git_fixture <- function(repo, ...) {
  status <- system2("git", c("-C", shQuote(repo), "-c", "user.name=test", "-c", "user.email=test@example.com",
                             "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", ...),
                    stdout = TRUE, stderr = TRUE)
  stopifnot(is.null(attr(status, "status")))
  status
}

# The .gitignore a scaffold ships for the staging outputs.
staging_ignores <- c("target/", "**/src/rust/vendor/", "**/src/rust/.Cargo.toml.dev",
                     "**/src/rust/.dev-bootstrap.rds", "**/src/rust/.dev-vendor-backup-*/")

stager_fixture <- function(case, env = parent.frame()) {
  repo <- normalizePath(withr::local_tempdir(.local_envir = env), winslash = "/")
  files <- c(case$files(repo), list(".gitignore" = c(staging_ignores, case$ignore)))
  for (file in names(files)) {
    dir.create(dirname(file.path(repo, file)), recursive = TRUE, showWarnings = FALSE)
    writeLines(files[[file]], file.path(repo, file))
  }
  manifest <- file.path(repo, "pkg/src/rust/Cargo.toml")
  if (is.null(case$error)) {
    status <- system2("cargo", c("generate-lockfile", "--offline", "--quiet", "--manifest-path", shQuote(manifest)),
                      stdout = FALSE, stderr = FALSE)
    if (status != 0L) {
      if (isTRUE(case$registry)) skip("registry crate not in the local cargo cache")
      stop("fixture lockfile generation failed for ", case$id)
    }
  }
  git_fixture(repo, "init", "-q")
  git_fixture(repo, "add", "-A")
  git_fixture(repo, "commit", "-q", "-m", "fixture")
  for (file in names(case$untracked)) writeLines(case$untracked[[file]], file.path(repo, file))
  list(repo = repo, pkg = file.path(repo, "pkg"), manifest = manifest)
}

# Paths of every path-source package cargo resolves for `manifest`.
cargo_path_packages <- function(manifest) {
  json <- paste(system2("cargo", c("metadata", "--format-version", "1", "--offline", "--locked",
                                   "--manifest-path", shQuote(manifest)), stdout = TRUE), collapse = "")
  ids <- unique(regmatches(json, gregexpr('"id":"path\\+file://[^"#]+', json, perl = TRUE))[[1L]])
  normalizePath(sub('^"id":"path\\+file://', "", ids), winslash = "/")
}

cargo_workspace_members <- function(manifest) {
  json <- paste(system2("cargo", c("metadata", "--no-deps", "--format-version", "1", "--offline",
                                   "--manifest-path", shQuote(manifest)), stdout = TRUE), collapse = "")
  members <- regmatches(json, regexpr('"workspace_members":\\[[^]]*\\]', json))
  regmatches(members, gregexpr("path\\+file://[^\"#]+", members))[[1L]]
}

# endregion

# region: cases

stager_cases <- list(
  # a. How the R crate declares its sibling.
  list(id = "a1", name = "inline table", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'), crate("alpha"))),
  list(id = "a2", name = "inline table, path last, with version/features/default-features",
       files = function(repo) c(
    rcrate("[dependencies]",
           'alpha = { version = "0.1", default-features = false, features = ["extra"], path = "../../../alpha" }',
           lib = "pub fn g() -> i32 { alpha::extra() }"),
    crate("alpha", features, lib = featured))),
  list(id = "a3", name = "renamed with package =", files = function(repo) c(
    rcrate("[dependencies]", 'engine = { package = "alpha", path = "../../../alpha" }',
           lib = "pub fn g() -> i32 { engine::value() }"), crate("alpha"))),
  list(id = "a4", name = "table form [dependencies.alpha]", files = function(repo) c(
    rcrate("[dependencies.alpha]", 'version = "0.1"', 'path = "../../../alpha"'), crate("alpha"))),
  list(id = "a5", name = "dotted key alpha.path", files = function(repo) c(
    rcrate("[dependencies]", 'alpha.path = "../../../alpha"'), crate("alpha"))),
  list(id = "a6", name = "single-quoted literal string", files = function(repo) c(
    rcrate("[dependencies]", "alpha = { path = '../../../alpha' }"), crate("alpha"))),
  list(id = "a7", name = "no spaces around =, trailing comment", files = function(repo) c(
    rcrate("[dependencies] # path siblings", 'alpha={path="../../../alpha"} # the sibling'), crate("alpha"))),
  list(id = "a8", name = "absolute path", files = function(repo) c(
    rcrate("[dependencies]", sprintf('alpha = { path = "%s/alpha" }', repo)), crate("alpha"))),
  list(id = "a9", name = "./ prefix, trailing slash, redundant ..", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "./../../../alpha/" }', 'beta = { path = "../../../alpha/../beta" }',
           lib = "pub fn g() -> i32 { alpha::value() + beta::value() }"),
    crate("alpha"), crate("beta"))),
  list(id = "a10", name = "own [workspace.dependencies], inherited plain and with features",
       files = function(repo) c(
    rcrate("[dependencies]", "alpha.workspace = true", 'beta = { workspace = true, features = ["extra"] }',
           workspace = c("[workspace]", "", "[workspace.dependencies]", 'alpha = { path = "../../../alpha" }',
                         'beta = { path = "../../../beta" }', ""),
           lib = "pub fn g() -> i32 { alpha::value() + beta::extra() }"),
    crate("alpha"), crate("beta", features, lib = featured))),
  list(id = "a11", name = "target tables, single- and double-quoted cfg", files = function(repo) c(
    rcrate("[target.'cfg(unix)'.dependencies]", 'alpha = { path = "../../../alpha" }',
           '[target."cfg(unix)".dependencies.beta]', 'path = "../../../beta"',
           lib = "#[cfg(unix)]\npub fn g() -> i32 { alpha::value() + beta::value() }"),
    crate("alpha"), crate("beta"))),
  list(id = "a12", name = "build- and dev-dependencies", files = function(repo) c(
    rcrate("[build-dependencies]", 'alpha = { path = "../../../alpha" }',
           "[dev-dependencies]", 'beta = { path = "../../../beta" }', lib = "pub fn g() -> i32 { 1 }"),
    list("pkg/src/rust/build.rs" = "fn main() { assert_eq!(alpha::value(), 1); }"),
    crate("alpha"), crate("beta"))),
  list(id = "a13", name = "optional dependency enabled by a feature", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha", optional = true }',
           "[features]", 'default = ["fast"]', 'fast = ["dep:alpha"]',
           lib = '#[cfg(feature = "fast")]\npub fn g() -> i32 { alpha::value() }'),
    crate("alpha"))),
  list(id = "a14", name = "same sibling as normal and build dependency", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }',
           "[build-dependencies]", 'alpha = { path = "../../../alpha" }'),
    list("pkg/src/rust/build.rs" = "fn main() { assert_eq!(alpha::value(), 1); }"),
    crate("alpha"))),
  list(id = "a15", name = "multi-line inline table (TOML 1.1)", files = function(repo) c(
    rcrate("[dependencies]", "alpha = {", '  path = "../../../alpha",', '  version = "0.1",', "}"),
    crate("alpha"))),
  list(id = "a16", name = "[patch.crates-io] pointing outside is rejected", files = function(repo) c(
    rcrate("[patch.crates-io]", 'beta = { path = "../../../beta" }', lib = "pub fn g() -> i32 { 1 }"),
    crate("beta")),
    error = c("under [patch.crates-io] points outside the package",
              "patch sources are not staged, so move that crate inside the package")),
  list(id = "a17", name = "[replace] pointing outside is rejected", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }',
           "[replace]", '"beta:0.1.0" = { path = "../../../beta" }'),
    crate("alpha"), crate("beta")),
    error = "under [replace] points outside the package"),

  # b. Sibling shapes. A standalone crate is a1.
  list(id = "b2", name = "root package with its own [workspace] members", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "", "[workspace]", 'members = ["tool"]'), crate("alpha/tool", name = "alpha-tool"))),
  list(id = "b3", name = "workspace member inheriting package keys and a registry dependency",
       registry = TRUE, files = function(repo) c(
    list("Cargo.toml" = c("[workspace]", 'resolver = "2"', 'members = ["alpha"]', 'exclude = ["pkg"]',
                          "[workspace.package]", 'version = "0.1.0"', 'edition = "2021"',
                          "[workspace.dependencies]", 'cfg-if = "1"')),
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", "cfg-if.workspace = true", head = member_head("alpha"),
          lib = "cfg_if::cfg_if! { if #[cfg(unix)] { pub fn value() -> i32 { 1 } } else { pub fn value() -> i32 { 1 } } }"))),
  list(id = "b4", name = "standalone crate with a nested path dependency", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", 'inner = { path = "../inner", version = "0.2" }',
          lib = "pub fn value() -> i32 { inner::value() }"),
    crate("inner", version = "0.2.0"))),
  list(id = "b5", name = "workspace member depending on a member by path", files = function(repo) c(
    list("Cargo.toml" = c("[workspace]", 'resolver = "2"', 'members = ["alpha", "inner"]', 'exclude = ["pkg"]',
                          "[workspace.package]", 'version = "0.1.0"', 'edition = "2021"')),
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", 'inner = { path = "../inner", version = "0.1" }', head = member_head("alpha"),
          lib = "pub fn value() -> i32 { inner::value() }"),
    crate("inner", head = member_head("inner")))),
  list(id = "b6", name = "workspace member inheriting a member dependency", files = function(repo) c(
    list("Cargo.toml" = c("[workspace]", 'resolver = "2"', 'members = ["alpha", "inner"]', 'exclude = ["pkg"]',
                          "[workspace.package]", 'version = "0.1.0"', 'edition = "2021"',
                          "[workspace.dependencies]", 'inner = { path = "inner", version = "0.1" }')),
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", "inner.workspace = true", head = member_head("alpha"),
          lib = "pub fn value() -> i32 { inner::value() }"),
    crate("inner", head = member_head("inner")))),
  list(id = "b7", name = "diamond: root to A and B, A to B", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }', 'beta = { path = "../../../beta" }',
           lib = "pub fn g() -> i32 { alpha::value() + beta::value() }"),
    crate("alpha", "[dependencies]", 'beta = { path = "../beta", version = "0.1" }',
          lib = "pub fn value() -> i32 { beta::value() }"),
    crate("beta"))),
  list(id = "b8", name = "three levels deep", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", 'beta = { path = "../beta", version = "0.1" }',
          lib = "pub fn value() -> i32 { beta::value() }"),
    crate("beta", "[dependencies]", 'gamma = { path = "../gamma", version = "0.1" }',
          lib = "pub fn value() -> i32 { gamma::value() }"),
    crate("gamma"))),
  list(id = "b9", name = "build.rs, include/exclude, publish = false", absent = c(
    "alpha-0.1.0/notes/private.txt", "beta-0.1.0/data/big.txt"), files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }', 'beta = { path = "../../../beta" }',
           lib = "pub fn g() -> i32 { alpha::value() + beta::value() }"),
    crate("alpha", "publish = false", 'include = ["/src", "/build.rs"]',
          lib = 'pub fn value() -> i32 { env!("ALPHA_BUILT").len() as i32 }'),
    list("alpha/build.rs" = 'fn main() { println!("cargo:rustc-env=ALPHA_BUILT=y"); }',
         "alpha/notes/private.txt" = "not packaged"),
    crate("beta", 'exclude = ["/data"]'), list("beta/data/big.txt" = "not packaged"))),
  list(id = "b10", name = "sibling optional dependency selected by a root feature", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha", features = ["extra"] }'),
    crate("alpha", "[dependencies]", 'opt = { path = "../opt", version = "0.1", optional = true }',
          "[features]", 'extra = ["dep:opt"]',
          lib = '#[cfg(feature = "extra")]\npub fn value() -> i32 { opt::value() }'),
    crate("opt"))),
  list(id = "b11", name = "in-tree path crate mixed with an external one", files = function(repo) c(
    rcrate("[dependencies]", 'satellite = { path = "satellite" }', 'alpha = { path = "../../../alpha" }',
           lib = "pub fn g() -> i32 { alpha::value() + satellite::value() }"),
    crate("pkg/src/rust/satellite", name = "satellite"), crate("alpha"))),
  list(id = "b12", name = "untracked source packaged, ignored file left out", absent = "alpha-0.1.0/secret.txt",
       untracked = list("alpha/src/fresh.rs" = "pub fn f() -> i32 { 1 }", "alpha/secret.txt" = "ignored"),
       ignore = "alpha/secret.txt", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", lib = "mod fresh;\npub fn value() -> i32 { fresh::f() }"))),

  # Unsupported shapes stop before staging.
  list(id = "e1", name = "nested path dependency without a version", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", 'inner = { path = "../inner" }'), crate("inner", version = "0.2.0")),
    error = "`inner` has no version requirement; add `version = \"0.2.0\"`"),
  list(id = "e2", name = "git dependency in a sibling", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", 'remote = { git = "https://example.invalid/remote.git" }')),
    error = "`remote` is a git dependency, which `cargo package` cannot keep"),
  # A workspace root loads its path dependencies, so cargo itself reports this one.
  list(id = "e3", name = "dangling path in the R crate", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }', 'ghost = { path = "../../../ghost" }'),
    crate("alpha")),
    header = "cargo metadata failed for",
    error = c("failed to load manifest for dependency `ghost`", "Fix the manifest error above")),
  list(id = "e4", name = "dangling path in a sibling", files = function(repo) c(
    rcrate("[dependencies]", 'alpha = { path = "../../../alpha" }'),
    crate("alpha", "[dependencies]", 'ghost = { path = "../ghost", version = "0.1" }')),
    error = "path dependency `ghost` points at"),
  list(id = "e5", name = "R crate inheriting from a parent workspace", files = function(repo) c(
    list("Cargo.toml" = c("[workspace]", 'resolver = "2"', 'members = ["pkg/src/rust"]',
                          "[workspace.dependencies]", 'alpha = { path = "alpha" }')),
    rcrate("[dependencies]", "alpha.workspace = true", workspace = character()), crate("alpha")),
    error = "inherits from a parent workspace; give it its own [workspace] table"),
  list(id = "e6", name = "two path crates with one package name", files = function(repo) c(
    rcrate("[dependencies]", 'one = { package = "alpha", path = "../../../alpha" }',
           'two = { package = "alpha", path = "../../../other" }'),
    crate("alpha"), crate("other", name = "alpha", version = "0.2.0")),
    error = "path dependencies share a package name")
)

# endregion

skip_without_stager_tools <- function() {
  skip_on_cran()
  for (tool in c("cargo", "git")) skip_if_not(nzchar(Sys.which(tool)), paste(tool, "not available"))
}

for (case in Filter(function(case) is.null(case$error), stager_cases)) {
  test_that(sprintf("stager matrix %s: %s relocates and builds offline", case$id, case$name), {
    skip_without_stager_tools()
    helper <- stager_helper()
    withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA, CARGO_NET_OFFLINE = "true", CARGO_TARGET_DIR = NA))
    fx <- stager_fixture(case)
    locks <- list.files(fx$repo, "^Cargo\\.(toml|lock)$", recursive = TRUE, full.names = TRUE)
    before <- tools::md5sum(locks)
    expect_true(suppressMessages(helper$prepare_dev_bootstrap(fx$pkg, mode = "dist")))
    expect_identical(tools::md5sum(locks), before)
    expect_identical(git_fixture(fx$repo, "status", "--porcelain", "--untracked-files=all"),
                     sort(sprintf("?? %s", setdiff(names(case$untracked), case$ignore))))
    for (file in case$absent) expect_false(file.exists(file.path(fx$pkg, "src/rust/vendor", file)), label = file)

    # What R CMD build's cleanup sees: a copy under an unrelated ancestor workspace.
    outer <- normalizePath(withr::local_tempdir(), winslash = "/")
    writeLines(c("[workspace]", "members = []"), file.path(outer, "Cargo.toml"))
    expect_true(file.copy(fx$pkg, outer, recursive = TRUE))
    copy <- file.path(outer, "pkg")
    manifest <- file.path(copy, "src/rust/Cargo.toml")
    expect_true(helper$activate_dev_bootstrap(copy))
    log <- tempfile(fileext = ".log")
    status <- system2("cargo", c("check", "--locked", "--offline", "--quiet", "--manifest-path", shQuote(manifest)),
                      stdout = log, stderr = log, env = paste0("CARGO_TARGET_DIR=", shQuote(file.path(outer, "target"))))
    expect_identical(status, 0L, info = paste(readLines(log), collapse = "\n"))
    expect_identical(unname(tools::md5sum(file.path(copy, "src/rust/Cargo.lock"))),
                     unname(tools::md5sum(file.path(fx$pkg, "src/rust/Cargo.lock"))))
    expect_length(cargo_workspace_members(manifest), 1L)
    resolved <- cargo_path_packages(manifest)
    expect_true(all(startsWith(resolved, paste0(copy, "/"))), info = paste(resolved, collapse = "\n"))
  })
}

for (case in Filter(function(case) !is.null(case$error), stager_cases)) {
  test_that(sprintf("stager matrix %s: %s stops before staging", case$id, case$name), {
    skip_without_stager_tools()
    helper <- stager_helper()
    withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA, CARGO_NET_OFFLINE = "true"))
    fx <- stager_fixture(case)
    header <- if (is.null(case$header)) "Cannot stage path dependencies without cargo-revendor" else case$header
    err <- expect_error(helper$prepare_dev_bootstrap(fx$pkg, mode = "dist"), header, fixed = TRUE)
    for (pattern in case$error) expect_match(conditionMessage(err), pattern, fixed = TRUE)
    if (is.null(case$header)) expect_match(conditionMessage(err), "cargo install --git", fixed = TRUE)
    expect_false(file.exists(file.path(fx$pkg, "src/rust/vendor")))
    expect_false(file.exists(file.path(fx$pkg, "src/rust/.dev-bootstrap.rds")))
    expect_identical(git_fixture(fx$repo, "status", "--porcelain", "--untracked-files=all"), character())
  })
}

test_that("re-staging replaces the previous staging and leaves no backups", {
  skip_without_stager_tools()
  helper <- stager_helper()
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA, CARGO_NET_OFFLINE = "true"))
  fx <- stager_fixture(stager_cases[[which(vapply(stager_cases, `[[`, "", "id") == "b8")]])
  rust <- file.path(fx$pkg, "src/rust")
  for (i in 1:3) expect_true(suppressMessages(helper$prepare_dev_bootstrap(fx$pkg, mode = "dist")))
  expect_setequal(list.files(file.path(rust, "vendor")), c("alpha-0.1.0", "beta-0.1.0", "gamma-0.1.0"))
  expect_length(list.files(rust, "^\\.dev-vendor-backup-", all.files = TRUE), 0L)
  expect_identical(git_fixture(fx$repo, "status", "--porcelain", "--untracked-files=all"), character())
  helper$clear_dev_bootstrap(fx$pkg)
  expect_false(any(file.exists(file.path(rust, c("vendor", ".Cargo.toml.dev", ".dev-bootstrap.rds")))))
  expect_length(list.files(rust, "^\\.dev-vendor-backup-", all.files = TRUE), 0L)

  # A src/rust/vendor without bootstrap's state file is not ours: never touch it.
  dir.create(file.path(rust, "vendor/mine"), recursive = TRUE)
  writeLines("keep", file.path(rust, "vendor/mine/file.txt"))
  expect_error(helper$prepare_dev_bootstrap(fx$pkg, mode = "dist"), "exists but bootstrap did not create it")
  expect_identical(readLines(file.path(rust, "vendor/mine/file.txt")), "keep")
  expect_false(file.exists(file.path(rust, ".dev-bootstrap.rds")))
})

test_that("activation refuses a staging older than its path dependencies", {
  skip_without_stager_tools()
  helper <- stager_helper()
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA, CARGO_NET_OFFLINE = "true"))
  fx <- stager_fixture(stager_cases[[which(vapply(stager_cases, `[[`, "", "id") == "b6")]])
  expect_true(suppressMessages(helper$prepare_dev_bootstrap(fx$pkg, mode = "dist")))
  sources <- readRDS(file.path(fx$pkg, "src/rust/.dev-bootstrap.rds"))$sources
  expect_setequal(names(sources), c("alpha-0.1.0", "inner-0.1.0"))
  # The inherited workspace manifest is part of each member's fingerprint.
  expect_true(file.path(fx$repo, "Cargo.toml") %in% names(sources[["alpha-0.1.0"]]$md5))
  relocate <- function() {
    outer <- withr::local_tempdir(.local_envir = parent.frame())
    expect_true(file.copy(fx$pkg, outer, recursive = TRUE))
    file.path(outer, "pkg")
  }
  expect_true(helper$activate_dev_bootstrap(relocate()))

  lib <- file.path(fx$repo, "inner/src/lib.rs")
  writeLines("pub fn value() -> i32 { 2 }", lib)
  copy <- relocate()
  expect_error(helper$activate_dev_bootstrap(copy),
               "path dependency inner changed since bootstrap; rerun bootstrap.R.", fixed = TRUE)
  expect_true(file.exists(file.path(copy, "src/rust/.dev-bootstrap.rds")))
  git_fixture(fx$repo, "checkout", "--", "inner/src/lib.rs")
  unlink(file.path(fx$repo, "alpha/src/lib.rs"))
  writeLines(c(readLines(file.path(fx$repo, "Cargo.toml")), "# edited"), file.path(fx$repo, "Cargo.toml"))
  expect_error(helper$activate_dev_bootstrap(relocate()),
               "path dependencies alpha, inner changed since bootstrap", fixed = TRUE)
  # A source that no longer exists cannot be compared, so it is not a mismatch.
  unlink(file.path(fx$repo, c("alpha", "inner", "Cargo.toml")), recursive = TRUE)
  expect_true(helper$activate_dev_bootstrap(relocate()))
})

test_that("cargo revendor --dev output is replaced, not backed up, and fingerprinted", {
  skip_without_stager_tools()
  skip_if_not(nzchar(Sys.which("cargo-revendor")), "cargo-revendor not available")
  helper <- stager_helper()
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA, CARGO_NET_OFFLINE = "true"))
  fx <- stager_fixture(stager_cases[[which(vapply(stager_cases, `[[`, "", "id") == "b4")]])
  rust <- file.path(fx$pkg, "src/rust")
  for (i in 1:3) expect_true(helper$prepare_dev_bootstrap(fx$pkg, mode = "dev"))
  expect_setequal(list.files(file.path(rust, "vendor")), c("alpha-0.1.0", "inner-0.2.0"))
  expect_length(list.files(rust, "^\\.dev-vendor-backup-", all.files = TRUE), 0L)
  expect_identical(git_fixture(fx$repo, "status", "--porcelain", "--untracked-files=all"), character())
  sources <- readRDS(file.path(rust, ".dev-bootstrap.rds"))$sources
  expect_setequal(names(sources), c("alpha-0.1.0", "inner-0.2.0"))
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = "dev"))
  writeLines("pub fn value() -> i32 { 9 }", file.path(fx$repo, "inner/src/lib.rs"))
  outer <- withr::local_tempdir()
  expect_true(file.copy(fx$pkg, outer, recursive = TRUE))
  expect_error(helper$activate_dev_bootstrap(file.path(outer, "pkg")),
               "path dependency inner changed since bootstrap", fixed = TRUE)
})
