#!/usr/bin/env Rscript
# Single-page HTML reference manual of this R package, built from the Rd
# sources in man/ with base R's tools converters (tools::pkg2HTML, R >= 4.4).
# Nothing is installed or loaded, so the page cannot lag the tree, and it
# builds in seconds even for packages with thousands of generated wrappers.
#
# Usage (from any working directory; base R only, no extra packages):
#   Rscript tools/build-html-reference.R [OUT_DIR]
# OUT_DIR defaults to src/rust/target/doc/r, next to the rustdoc output of a
# per-package cargo target directory, so one tree documents both halves of the
# package. Writes <package>.html, R-nav.css and Rlogo.svg there, plus
# vignettes/*.html when inst/doc holds built vignettes. When a rustdoc crate
# index (rustdoc --enable-index-page) sits one level above OUT_DIR as
# index.html, the manual and the vignettes are added to its crate list.
#
# In-package links stay on the page; cross-package links go to the single-page
# manuals CRAN hosts. This is a maintainer script: R CMD build ignores it and
# nothing at install time depends on it.

if (getRversion() < "4.4.0") stop("tools::pkg2HTML() needs R >= 4.4.0")

script_path <- function() {
  arg <- grep("^--file=", commandArgs(FALSE), value = TRUE)
  if (length(arg) != 1L) stop("run this script with Rscript so its location is known")
  normalizePath(sub("^--file=", "", arg), mustWork = TRUE)
}
pkg_dir <- normalizePath(file.path(dirname(script_path()), ".."), mustWork = TRUE)
pkg <- unname(read.dcf(file.path(pkg_dir, "DESCRIPTION"), fields = "Package")[1L, "Package"])
args <- commandArgs(TRUE)
out_dir <- if (length(args) >= 1L) args[[1L]] else file.path(pkg_dir, "src", "rust", "target", "doc", "r")
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)
out_dir <- normalizePath(out_dir, mustWork = TRUE)

# 1. Validate the Rd sources first; a finding here is an Rd bug to fix, not
#    something to render around. Checking the files (not a parsed Rd_db) makes
#    parse-time problems such as unknown macros findings too. def_enc = TRUE:
#    DESCRIPTION declares the encoding, so non-ASCII text is not a finding.
rd_files <- list.files(file.path(pkg_dir, "man"), pattern = "[.][Rr]d$", full.names = TRUE)
macros <- tools::loadPkgRdMacros(pkg_dir)
problems <- lapply(rd_files, tools::checkRd, def_enc = TRUE, macros = macros)
names(problems) <- basename(rd_files)
problems <- problems[lengths(problems) > 0L]
if (length(problems)) {
  for (nm in names(problems)) {
    cat(nm, ":\n")
    print(problems[[nm]])
  }
  stop(length(problems), " Rd file(s) with checkRd findings; not building the manual")
}

# 2. Link targets for other packages. pkg2HTML appends "#topic+<alias>" to
#    whatever this returns; CRAN serves pkg2HTML output for every package at
#    the refman path, base packages only have per-topic pages.
base_pkgs <- rownames(installed.packages(priority = "base"))
pkg_href <- function(p) {
  if (identical(p, pkg)) return(paste0(pkg, ".html"))
  if (p %in% base_pkgs) {
    return(sprintf("https://search.r-project.org/R/refmans/%s/html/00Index.html", p))
  }
  sprintf("https://cran.r-project.org/web/packages/%s/refman/%s.html", p, p)
}

# 3. The manual. stylesheet is a URL relative to the page; the file is copied
#    next to it below. Code highlighting (prism) and KaTeX load from CDNs, as
#    in pkg2HTML's default output.
out_html <- file.path(out_dir, paste0(pkg, ".html"))
tools::pkg2HTML(
  dir = pkg_dir, out = out_html,
  hooks = list(pkg_href = pkg_href), stylesheet = "R-nav.css"
)

# 4. pkg2HTML hard-codes an absolute path to R's logo; ship a copy instead.
r_html_dir <- file.path(R.home("doc"), "html")
html <- readLines(out_html, encoding = "UTF-8", warn = FALSE)
html <- gsub(file.path(r_html_dir, "Rlogo.svg"), "Rlogo.svg", html, fixed = TRUE)
writeLines(html, out_html, useBytes = TRUE)
for (f in c("R-nav.css", "Rlogo.svg")) {
  stopifnot(file.copy(file.path(r_html_dir, f), out_dir, overwrite = TRUE))
}

# 5. Built vignettes, when present (tools::buildVignettes() or R CMD build).
vig_src <- list.files(file.path(pkg_dir, "inst", "doc"), pattern = "[.]html$", full.names = TRUE)
if (length(vig_src)) {
  vig_dir <- file.path(out_dir, "vignettes")
  dir.create(vig_dir, showWarnings = FALSE)
  stopifnot(file.copy(vig_src, vig_dir, overwrite = TRUE))
}

# 6. Self-check: every in-page link has a target, no machine path leaked.
defined <- sub("^id=.topic[+]", "",
               unlist(regmatches(html, gregexpr("id=.topic[+][^\"'>]+", html))))
referenced <- sub("^href=\"#topic[+]", "",
                  sub("\"$", "", unlist(regmatches(html, gregexpr("href=\"#topic[+][^\"]+\"", html)))))
dangling <- setdiff(referenced, defined)
if (length(dangling)) stop("dangling in-page links: ", paste(dangling, collapse = ", "))
if (any(grepl(R.home(), html, fixed = TRUE))) stop("absolute R.home() path left in ", out_html)
if (any(grepl(sprintf("/refman/%s.html", pkg), html, fixed = TRUE))) {
  stop("links into this package were resolved to CRAN instead of the page")
}

# 7. Register the manual on rustdoc's crate index when one sits one level up.
#    rustdoc rewrites that file on every cargo doc run, so run this script
#    after it. Any other index.html is left alone.
index_html <- file.path(dirname(out_dir), "index.html")
registered <- FALSE
if (file.exists(index_html)) {
  idx <- paste(readLines(index_html, encoding = "UTF-8", warn = FALSE), collapse = "\n")
  m <- regexpr('<ul class="all-items">.*?</ul>', idx, perl = TRUE)
  if (m != -1L) {
    rel <- basename(out_dir)
    items <- regmatches(regmatches(idx, m), gregexpr("<li>.*?</li>", regmatches(idx, m), perl = TRUE))[[1L]]
    items <- items[!grepl(sprintf('href="%s/', rel), items, fixed = TRUE)]
    items <- c(items, sprintf('<li><a href="%s/%s.html">%s (R package reference manual)</a></li>', rel, pkg, pkg))
    if (length(vig_src)) {
      items <- c(items, sprintf('<li><a href="%s/vignettes/%s">%s vignette: %s</a></li>',
                                rel, basename(vig_src), pkg, tools::file_path_sans_ext(basename(vig_src))))
    }
    regmatches(idx, m) <- paste0('<ul class="all-items">', paste(items, collapse = ""), "</ul>")
    title <- paste(pkg, "documentation")
    idx <- sub("<title>Index of crates</title>", sprintf("<title>%s</title>", title), idx, fixed = TRUE)
    idx <- sub('<meta name="description" content="List of crates">',
               sprintf('<meta name="description" content="%s">', title), idx, fixed = TRUE)
    idx <- sub("<h1>List of all crates</h1>", sprintf("<h1>%s</h1>", title), idx, fixed = TRUE)
    writeLines(idx, index_html, useBytes = TRUE)
    registered <- TRUE
  }
}

cat(sprintf("wrote %s (%d topics%s)%s\n", out_html, length(unique(defined)),
            if (length(vig_src)) sprintf(", %d vignette(s)", length(vig_src)) else "",
            if (registered) sprintf("; registered on %s", index_html) else ""))
