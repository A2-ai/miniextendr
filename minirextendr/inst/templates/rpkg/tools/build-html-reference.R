#!/usr/bin/env Rscript
# Single-page HTML reference manual of this R package, built from the Rd
# sources in man/ with base R's tools converters (tools::pkg2HTML, R >= 4.4).
# Nothing is installed or loaded, so the page cannot lag the tree, and it
# builds in seconds even for packages with thousands of generated wrappers.
#
# Usage (from any working directory; base R only, no extra packages):
#   Rscript tools/build-html-reference.R [OUT_DIR]
# OUT_DIR defaults to rust-target/doc/r, next to the rustdoc output in the
# package's cargo target directory (configure's default CARGO_TARGET_DIR), so
# one tree documents both halves of the package. Writes <package>.html,
# R-nav.css and Rlogo.svg there, plus
# vignettes/*.html when inst/doc holds built vignettes. When a rustdoc crate
# index (rustdoc --enable-index-page) sits one level above OUT_DIR as
# index.html, the manual and the vignettes are added to its crate list.
#
# In-package links stay on the page; cross-package links go to the single-page
# manuals CRAN hosts, or for base packages to the topic's own page. Nothing at install time depends on this script; it ships
# with the other tools/ scripts so an unpacked tarball can build the manual too.
# MINIEXTENDR_HTML_STRICT=0 turns checkRd findings from an error into a message.
# Topics with \keyword{internal} are left out of the page, as R leaves them
# out of a package's help index. MINIEXTENDR_HTML_INTERNAL=1 keeps them.

if (getRversion() < "4.4.0") stop("tools::pkg2HTML() needs R >= 4.4.0")

script_path <- function() {
  arg <- grep("^--file=", commandArgs(FALSE), value = TRUE)
  if (length(arg) != 1L) stop("run this script with Rscript so its location is known")
  normalizePath(sub("^--file=", "", arg), mustWork = TRUE)
}
pkg_dir <- normalizePath(file.path(dirname(script_path()), ".."), mustWork = TRUE)
pkg <- unname(read.dcf(file.path(pkg_dir, "DESCRIPTION"), fields = "Package")[1L, "Package"])
args <- commandArgs(TRUE)
out_dir <- if (length(args) >= 1L) args[[1L]] else file.path(pkg_dir, "rust-target", "doc", "r")
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)
out_dir <- normalizePath(out_dir, mustWork = TRUE)

# 1. Validate the Rd sources first; a finding here is an Rd bug to fix, not
#    something to render around (MINIEXTENDR_HTML_STRICT=0 renders anyway). Checking the files (not a parsed Rd_db) makes
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
  found <- paste(length(problems), "Rd file(s) with checkRd findings")
  if (!identical(Sys.getenv("MINIEXTENDR_HTML_STRICT"), "0")) {
    stop(found, "; not building the manual (MINIEXTENDR_HTML_STRICT=0 builds anyway)")
  }
  message(found, "; building anyway (MINIEXTENDR_HTML_STRICT=0)")
}

# 2. Link targets for other packages. pkg2HTML appends "#topic+<alias>" to
#    whatever this returns; CRAN serves pkg2HTML output for every package at
#    the refman path, base packages only have per-topic pages (step 5 points
#    their links at the right page).
base_pkgs <- rownames(installed.packages(priority = "base"))
pkg_href <- function(p) {
  if (identical(p, pkg)) return(paste0(pkg, ".html"))
  if (p %in% base_pkgs) {
    return(sprintf("https://search.r-project.org/R/refmans/%s/html/00Index.html", p))
  }
  sprintf("https://cran.r-project.org/web/packages/%s/refman/%s.html", p, p)
}

# 3. The topics on the page. A topic with \keyword{internal} is left out, as
#    R's help index leaves it out, unless it is the package overview
#    (\docType{package}, the page pkg2HTML puts first) or a kept topic links
#    to it, so every in-page link keeps its target.
#    MINIEXTENDR_HTML_INTERNAL=1 keeps every topic.
rd_tag_text <- function(rd, tag) {
  hits <- Filter(function(e) identical(attr(e, "Rd_tag"), tag), rd)
  trimws(vapply(hits, function(e) paste(unlist(e), collapse = ""), ""))
}
# The topics in this package that `rd` links to: \link{topic},
# \link[=topic]{text}, \link[<pkg>]{topic} and \link[<pkg>:file]{text}.
rd_own_links <- function(rd) {
  found <- character()
  walk <- function(e) {
    if (identical(attr(e, "Rd_tag"), "\\link")) {
      text <- paste(unlist(e), collapse = "")
      opt <- attr(e, "Rd_option")
      opt <- if (is.null(opt)) "" else paste(unlist(opt), collapse = "")
      target <- if (!nzchar(opt)) {
        text
      } else if (startsWith(opt, "=")) {
        substring(opt, 2L)
      } else if (identical(opt, pkg)) {
        text
      } else if (startsWith(opt, paste0(pkg, ":"))) {
        substring(opt, nchar(pkg) + 2L)
      }
      found <<- c(found, target)
    }
    if (is.list(e)) for (child in e) walk(child)
  }
  walk(rd)
  unique(found)
}
rd <- lapply(rd_files, tools::parse_Rd, macros = macros, encoding = "UTF-8")
keep <- vapply(rd, function(r) {
  !("internal" %in% rd_tag_text(r, "\\keyword")) || "package" %in% rd_tag_text(r, "\\docType")
}, NA)
if (identical(Sys.getenv("MINIEXTENDR_HTML_INTERNAL"), "1")) keep[] <- TRUE
# A link names an alias, or for \link[<pkg>:file] the Rd file.
topic_index <- c(
  unlist(lapply(seq_along(rd), function(i) {
    a <- rd_tag_text(rd[[i]], "\\alias")
    structure(rep(i, length(a)), names = a)
  })),
  structure(seq_along(rd_files), names = sub("[.][Rr]d$", "", basename(rd_files)))
)
repeat {
  linked <- topic_index[unique(unlist(lapply(rd[keep], rd_own_links)))]
  linked <- unique(linked[!is.na(linked)])
  more <- setdiff(linked, which(keep))
  if (!length(more)) break
  keep[more] <- TRUE
}

# 4. The manual. pkg2HTML renders every Rd file under dir, so with topics left
#    out it renders a copy of DESCRIPTION and man/ without them. stylesheet is
#    a URL relative to the page; the file is copied next to it below. Code
#    highlighting (prism) and KaTeX load from CDNs, as in pkg2HTML's default
#    output.
render_dir <- pkg_dir
if (!all(keep)) {
  render_dir <- file.path(tempfile("html-reference-"), pkg)
  dir.create(render_dir, recursive = TRUE)
  stopifnot(
    file.copy(file.path(pkg_dir, "DESCRIPTION"), render_dir),
    file.copy(file.path(pkg_dir, "man"), render_dir, recursive = TRUE)
  )
  unlink(file.path(render_dir, "man", basename(rd_files[!keep])))
}
out_html <- file.path(out_dir, paste0(pkg, ".html"))
tools::pkg2HTML(
  dir = render_dir, out = out_html,
  hooks = list(pkg_href = pkg_href), stylesheet = "R-nav.css"
)
if (!identical(render_dir, pkg_dir)) unlink(dirname(render_dir), recursive = TRUE)

# 5. pkg2HTML hard-codes an absolute path to R's logo; ship a copy instead.
#    A base package's index page has no "#topic+<alias>" anchors, so each such
#    link goes to the page of the Rd file that has the alias, looked up in the
#    package's installed help index (stderr is on showConnections.html). An
#    alias missing from it keeps the index page, without the fragment.
r_html_dir <- file.path(R.home("doc"), "html")
html <- readLines(out_html, encoding = "UTF-8", warn = FALSE)
html <- gsub(file.path(r_html_dir, "Rlogo.svg"), "Rlogo.svg", html, fixed = TRUE)
base_links <- unique(unlist(regmatches(html, gregexpr(
  'https://search[.]r-project[.]org/R/refmans/[^/]+/html/00Index[.]html#topic[+][^"]+', html))))
base_rd <- list()
for (link in base_links) {
  p <- sub("^.*/refmans/([^/]+)/html/.*$", "\\1", link)
  if (is.null(base_rd[[p]])) {
    aliases <- readRDS(file.path(find.package(p), "help", "aliases.rds"))
    # pkg2HTML's id for an alias (tools:::string2id).
    ids <- gsub("%", "+", vapply(names(aliases), utils::URLencode, "", reserved = TRUE))
    base_rd[[p]] <- structure(unname(aliases), names = ids)
  }
  rd <- base_rd[[p]][sub("^.*#topic[+]", "", link)]
  page <- if (is.na(rd)) "00Index.html" else paste0(rd, ".html")
  html <- gsub(link, sub("00Index[.]html#.*$", page, link), html, fixed = TRUE)
}
writeLines(html, out_html, useBytes = TRUE)
for (f in c("R-nav.css", "Rlogo.svg")) {
  stopifnot(file.copy(file.path(r_html_dir, f), out_dir, overwrite = TRUE))
}

# 6. Built vignettes, when present (tools::buildVignettes() or R CMD build).
vig_src <- list.files(file.path(pkg_dir, "inst", "doc"), pattern = "[.]html$", full.names = TRUE)
if (length(vig_src)) {
  vig_dir <- file.path(out_dir, "vignettes")
  dir.create(vig_dir, showWarnings = FALSE)
  stopifnot(file.copy(vig_src, vig_dir, overwrite = TRUE))
}

# 7. Self-check: every in-page link has a target, no machine path leaked.
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

# 8. Register the manual on rustdoc's crate index when one sits one level up.
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

cat(sprintf("wrote %s (%d topics%s%s)%s\n", out_html, length(unique(defined)),
            if (any(!keep)) sprintf(", %d internal Rd page(s) left out", sum(!keep)) else "",
            if (length(vig_src)) sprintf(", %d vignette(s)", length(vig_src)) else "",
            if (registered) sprintf("; registered on %s", index_html) else ""))
