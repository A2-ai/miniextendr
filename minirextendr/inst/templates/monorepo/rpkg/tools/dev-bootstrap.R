# Development artifact staging. Base R only; no minirextendr runtime dependency.
# bootstrap runs in the checkout; cleanup activates only in R CMD build's copy.
dev_bootstrap_paths <- function(root = ".") {
  root <- normalizePath(root, winslash = "/", mustWork = TRUE)
  list(root = root, manifest = file.path(root, "src/rust/Cargo.toml"),
       portable = file.path(root, "src/rust/.Cargo.toml.dev"),
       state = file.path(root, "src/rust/.dev-bootstrap.rds"),
       vendor = file.path(root, "src/rust/vendor"))
}

dev_bootstrap_backup <- function(paths, files) {
  files <- files[file.exists(files)]
  if (!length(files)) return(invisible(NULL))
  backup <- tempfile(".dev-vendor-backup-", dirname(paths$manifest))
  dir.create(backup)
  for (file in files) {
    if (!file.rename(file, file.path(backup, basename(file)))) {
      stop("Unable to retain development staging file: ", file, call. = FALSE)
    }
  }
  invisible(backup)
}

prepare_dev_bootstrap <- function(root = ".") {
  paths <- dev_bootstrap_paths(root)
  if (file.exists(file.path(paths$root, "inst/vendor.tar.xz"))) {
    stop("Development bootstrap requires a source tree without inst/vendor.tar.xz; ",
         "restore the source tree with minirextendr::miniextendr_clean_vendor_leak() first.",
         call. = FALSE)
  }
  if (!nzchar(Sys.which("cargo-revendor"))) {
    stop("Development bootstrap requires cargo-revendor on PATH.", call. = FALSE)
  }
  original <- unname(tools::md5sum(paths$manifest))
  status <- system2("cargo", c("revendor", "--dev", "--manifest-path",
    shQuote(paths$manifest), "--output", shQuote(paths$vendor), "-v"))
  if (status != 0L) stop("Development bootstrap failed (exit ", status, ").", call. = FALSE)
  if (!identical(unname(tools::md5sum(paths$manifest)), original)) {
    stop("Development bootstrap changed the source Cargo.toml.", call. = FALSE)
  }
  saveRDS(list(root = paths$root, manifest = original), paths$state)
  invisible(TRUE)
}

clear_dev_bootstrap <- function(root = ".") {
  paths <- dev_bootstrap_paths(root)
  if (file.exists(paths$state)) {
    dev_bootstrap_backup(paths, c(paths$portable, paths$state, paths$vendor))
  }
  invisible(TRUE)
}

activate_dev_bootstrap <- function(root = ".") {
  paths <- dev_bootstrap_paths(root)
  if (!file.exists(paths$state)) return(invisible(FALSE))
  state <- readRDS(paths$state)
  if (identical(paths$root, state$root)) return(invisible(FALSE))
  if (!identical(Sys.getenv("MINIEXTENDR_BOOTSTRAP_MODE", "dist"), "dev")) {
    clear_dev_bootstrap(root)
    return(invisible(FALSE))
  }
  if (!identical(unname(tools::md5sum(paths$manifest)), state$manifest)) {
    stop("Source Cargo.toml changed after development bootstrap; rerun bootstrap.R.", call. = FALSE)
  }
  # Preserve the staged original too. The checkout was never rewritten.
  dev_bootstrap_backup(paths, c(paths$manifest, paths$state))
  if (!file.rename(paths$portable, paths$manifest)) {
    stop("Unable to activate the development manifest in the staged package.", call. = FALSE)
  }
  invisible(TRUE)
}

if (sys.nframe() == 0L) {
  action <- match.arg(commandArgs(trailingOnly = TRUE)[[1L]], c("prepare", "activate", "clear"))
  switch(action, prepare = prepare_dev_bootstrap(), activate = activate_dev_bootstrap(),
         clear = clear_dev_bootstrap())
}
