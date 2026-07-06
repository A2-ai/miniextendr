# pak-driven installs under R CMD check (#1154).
#
# devtools::install() (>= 2.5.0) unconditionally runs pak::local_install_deps()
# before R CMD INSTALL, and pak resolves through pkgcache. pkgcache hard-errors
# inside the pak subprocess when it detects R CMD check (via
# _R_CHECK_PACKAGE_NAME_) and no explicit cache location:
#
#   "R_USER_CACHE_DIR env var not set during package check"
#
# -- a deliberate guard so a check run cannot write into the user's real cache.
# That killed every install-driven template test under `just minirextendr-check`
# while the same tests pass under `just minirextendr-test` (no check env, so
# pkgcache falls back to the user cache as usual).
#
# Remedy prescribed by pkgcache's README: point R_USER_CACHE_DIR at a throwaway
# directory for the check run. It must happen HERE, in a setup file, before the
# suite's first pak call: pak keeps a persistent worker subprocess that
# snapshots its environment at creation, so a per-test withr::with_envvar()
# comes too late once any earlier test has touched pak (empirically verified:
# setting the var after one failed pak call still reproduces the error).
#
# Outside R CMD check this is a no-op, so dev runs keep pak's warm user-level
# cache and its speed.
if (testthat::is_checking() && !nzchar(Sys.getenv("R_USER_CACHE_DIR"))) {
  check_cache <- file.path(tempdir(), "minirextendr-check-r-user-cache")
  dir.create(check_cache, recursive = TRUE, showWarnings = FALSE)
  Sys.setenv(R_USER_CACHE_DIR = check_cache)
}
