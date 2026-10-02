# r-deps-check.R - check that rv installs every R package the repository uses.
#
# Run from the repo root (`just r-deps-check`), where `.Rprofile` activates rv.
# Scans the R scripts and R Markdown files git tracks with attachment, then
# compares each package they reference with rv.lock. Untracked trees such as
# background/ are never scanned: R's own sources there hold deliberately
# unparsable test scripts.
#
# Exits 1 when a package is referenced but missing from rv.lock (add it with
# `rv add <pkg>`). A package that rv.lock only has because something else
# depends on it is reported, but doesn't fail the check, when files outside
# the R packages use it. Declaring it keeps it installed if that dependency
# goes away.

files <- system2("git", c("ls-files", "*.R", "*.r", "*.Rmd", "*.rmd"), stdout = TRUE)
# rv's own activation scripts (rv/scripts/) are rv's, not ours.
files <- files[!startsWith(files, "rv/")]

deps_of <- function(f) {
  # attachment warns when a script doesn't parse (minirextendr's {{ }}
  # templates) and falls back to text matching, which is fine here.
  suppressWarnings(
    if (grepl("[.]rmd$", f, ignore.case = TRUE)) {
      attachment::att_from_rmds(f, inline = FALSE)
    } else {
      attachment::att_from_rscripts(f)
    }
  )
}
used <- lapply(files, deps_of)
names(used) <- files
pkgs <- sort(unique(unlist(used)))

lock <- readLines("rv.lock", warn = FALSE)
locked <- sub('^name = "([^"]+)"$', "\\1", grep('^name = "', lock, value = TRUE))

toml <- readLines("rproject.toml", warn = FALSE)
toml <- sub("#.*$", "", toml)
declared <- unique(c(
  sub('^\\s*"([^"]+)".*$', "\\1", grep('^\\s*"[A-Za-z][A-Za-z0-9.]*"', toml, value = TRUE)),
  regmatches(toml, regexpr('(?<=name = ")[^"]+', toml, perl = TRUE))
))

# Packages declared by the R packages rv installs dependencies for
# (`dependencies_only` in rproject.toml). R CMD check holds those packages to
# their own DESCRIPTION.
pkg_dirs <- c("rpkg", "minirextendr")
description_deps <- unique(unlist(lapply(pkg_dirs, function(d) {
  desc <- read.dcf(file.path(d, "DESCRIPTION"),
                   fields = c("Depends", "Imports", "Suggests", "LinkingTo"))
  fields <- desc[!is.na(desc)]
  trimws(sub("\\(.*$", "", unlist(strsplit(fields, ","))))
})))

base_pkgs <- rownames(installed.packages(priority = "base"))
own <- c("miniextendr", "minirextendr", "producer.pkg", "consumer.pkg")
# Not packages. attachment reads the argument of library() / requireNamespace()
# literally, so a variable holding a package name looks like one:
# `requireNamespace(pkg)` (scripts/gctorture-full-sweep.R) and
# `library(pkg_name, character.only = TRUE)` (minirextendr). `mypackage` is the
# example package in minirextendr's vignettes (eval = FALSE chunks).
not_packages <- c("pkg", "pkg_name", "mypackage")
pkgs <- setdiff(pkgs, c(base_pkgs, own, not_packages))

users <- function(p) names(Filter(function(d) p %in% d, used))
show <- function(p) {
  u <- users(p)
  more <- if (length(u) > 3) sprintf(" (+%d more)", length(u) - 3) else ""
  sprintf("  %-14s %s%s", p, paste(head(u, 3), collapse = ", "), more)
}

cat(sprintf("Scanned %d tracked R files; %d packages referenced.\n",
            length(files), length(pkgs)))

missing <- setdiff(pkgs, locked)
package_prefixes <- c(paste0(pkg_dirs, "/"),
                      "tests/cross-package/producer.pkg/",
                      "tests/cross-package/consumer.pkg/")
in_package <- function(f) any(startsWith(f, package_prefixes))
transitive <- Filter(function(p) {
  !p %in% c(declared, description_deps) && !all(vapply(users(p), in_package, logical(1)))
}, setdiff(pkgs, missing))

if (length(transitive)) {
  cat("\nIn rv.lock only as a dependency of something else, but used outside the R packages:\n")
  cat(vapply(transitive, show, ""), sep = "\n")
  cat(sprintf("Declare them with: rv add %s\n", paste(transitive, collapse = " ")))
}

if (length(missing)) {
  cat("\nReferenced but missing from rv.lock:\n")
  cat(vapply(missing, show, ""), sep = "\n")
  cat(sprintf("Add them with: rv add %s\n", paste(missing, collapse = " ")))
  quit(status = 1)
}

cat("\nEvery referenced package is in rv.lock.\n")
