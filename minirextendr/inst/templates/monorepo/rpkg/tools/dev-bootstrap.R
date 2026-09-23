# Path-dependency staging. Base R only; no minirextendr runtime dependency.
# bootstrap runs in the checkout; cleanup activates only in R CMD build's copy.
# Development builds stage via `cargo revendor --dev`; without that tool, and for
# distribution builds without it, the base-R stager below produces the same shape.
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

# region: base-R stager

# `cargo metadata` without a JSON parser: dependency objects are flat, and each
# package object opens with name/version/id, which delimits the packages.
cargo_json_field <- function(json, key) {
  m <- regmatches(json, regexec(sprintf('"%s":(null|"((\\\\.|[^"\\\\])*)")', key), json, perl = TRUE))[[1L]]
  if (!length(m) || m[[2L]] == "null") return(NA_character_)
  gsub("\\\\(.)", "\\1", m[[3L]], perl = TRUE)
}

cargo_manifest <- function(manifest) {
  manifest <- normalizePath(manifest, winslash = "/", mustWork = TRUE)
  out <- suppressWarnings(system2("cargo", c("metadata", "--no-deps", "--format-version", "1",
    "--offline", "--manifest-path", shQuote(manifest)), stdout = TRUE))
  if (!is.null(attr(out, "status"))) stop("cargo metadata failed for ", manifest, call. = FALSE)
  json <- paste(out, collapse = "")
  starts <- gregexpr('\\{"name":"[^"]*","version":"[^"]*","id":', json, perl = TRUE)[[1L]]
  pkgs <- substring(json, starts, c(starts[-1L] - 1L, nchar(json)))
  own <- vapply(pkgs, function(pkg) identical(normalizePath(cargo_json_field(pkg, "manifest_path"),
    winslash = "/"), manifest), logical(1), USE.NAMES = FALSE)
  pkg <- pkgs[own][[1L]]
  deps <- sub('^.*?"dependencies":\\[(.*?)\\],"targets":.*$', "\\1", pkg, perl = TRUE)
  objs <- regmatches(deps, gregexpr("\\{[^{}]*\\}", deps, perl = TRUE))[[1L]]
  field <- function(key) vapply(objs, cargo_json_field, "", key = key, USE.NAMES = FALSE)
  deps <- data.frame(name = field("name"), rename = field("rename"), kind = field("kind"),
    req = field("req"), target = field("target"), path = field("path"), source = field("source"))
  deps$alias <- ifelse(is.na(deps$rename), deps$name, deps$rename)
  list(name = cargo_json_field(pkg, "name"), version = cargo_json_field(pkg, "version"),
       manifest = manifest, dir = dirname(manifest), deps = deps,
       workspace = normalizePath(cargo_json_field(json, "workspace_root"), winslash = "/"))
}

path_inside <- function(path, root) startsWith(paste0(path, "/"), paste0(root, "/"))

# Path-dependency closure of the R crate. Every root edge ships (R builds the
# root as a workspace member, dev-dependencies included); siblings ship only
# their normal/build edges. `cargo package` re-resolves each sibling's
# normalized manifest, so its versioned path edges (dev ones too) also need a
# patch entry; unversioned dev edges are stripped by cargo and need nothing.
path_dependency_plan <- function(paths) {
  root <- cargo_manifest(paths$manifest)
  nodes <- list()
  patches <- character()
  problems <- character()
  pending <- list(root)
  while (length(pending)) {
    info <- pending[[1L]]
    pending <- pending[-1L]
    sibling <- !identical(info$dir, root$dir)
    for (i in seq_len(nrow(info$deps))) {
      dep <- info$deps[i, ]
      dev <- identical(dep$kind, "dev")
      if (sibling && !dev && !is.na(dep$source) && startsWith(dep$source, "git+")) {
        problems <- c(problems, sprintf("%s: `%s` is a git dependency, which `cargo package` cannot keep",
                                        info$manifest, dep$alias))
      }
      if (is.na(dep$path)) next
      dir <- normalizePath(dep$path, winslash = "/")
      if (identical(dir, root$dir)) next
      if (sibling && dev) {
        if (dep$req != "*") patches[[dep$name]] <- dir
        next
      }
      if (is.null(nodes[[dir]])) {
        nodes[[dir]] <- cargo_manifest(file.path(dir, "Cargo.toml"))
        pending <- c(pending, list(nodes[[dir]]))
      }
      if (!sibling) next
      patches[[dep$name]] <- dir
      if (dep$req == "*") {
        problems <- c(problems, sprintf("%s: path dependency `%s` has no version requirement; add `version = \"%s\"`%s",
          info$manifest, dep$alias, nodes[[dir]]$version,
          if (identical(info$workspace, info$dir)) "" else
            sprintf(" (or to [workspace.dependencies] in %s/Cargo.toml when inherited)", info$workspace)))
      }
    }
  }
  if (all(vapply(names(nodes), path_inside, logical(1), root = paths$root))) return(NULL)
  lines <- readLines(paths$manifest, warn = FALSE)
  if (!any(grepl("^\\s*\\[workspace\\]\\s*(#.*)?$", lines)) &&
      any(grepl("(^|[{,.[:space:]])workspace\\s*=\\s*true", lines))) {
    problems <- c(problems, sprintf("%s: inherits from a parent workspace; give it its own [workspace] table",
                                    paths$manifest))
  }
  if (length(problems)) {
    stop(paste(c("Cannot stage path dependencies without cargo-revendor:", paste0("  - ", problems),
      "Fix the entries above, or install cargo-revendor:",
      "  cargo install --git https://github.com/A2-ai/miniextendr cargo-revendor --locked"),
      collapse = "\n"), call. = FALSE)
  }
  slots <- vapply(nodes, function(node) paste0(node$name, "-", node$version), "")
  if (anyDuplicated(slots) || anyDuplicated(vapply(nodes, `[[`, "", "name"))) {
    stop("Path dependencies must have distinct package names: ", paste(slots, collapse = ", "), call. = FALSE)
  }
  list(root = root, nodes = nodes, slots = slots, patches = patches, lines = lines)
}

# `cargo package` rewrites a nested path dependency to a version-only
# [<kind>.<alias>] table; point it back at its staged sibling slot.
restore_sibling_paths <- function(node, lines, slots) {
  deps <- node$deps[!is.na(node$deps$path) & !node$deps$kind %in% "dev", ]
  at <- integer()
  add <- character()
  for (i in seq_len(nrow(deps))) {
    dep <- deps[i, ]
    section <- if (identical(dep$kind, "build")) "build-dependencies" else "dependencies"
    heads <- if (is.na(dep$target)) sprintf("[%s.%s]", section, dep$alias) else
      sprintf("[target.%s.%s.%s]", c(dep$target, sprintf('"%s"', gsub('"', '\\"', dep$target, fixed = TRUE)),
                                     sprintf("'%s'", dep$target)), section, dep$alias)
    hit <- which(trimws(lines) %in% heads)
    if (length(hit) != 1L) {
      stop("Cannot locate ", heads[[1L]], " in the packaged manifest of ", node$name, call. = FALSE)
    }
    at <- c(at, hit)
    add <- c(add, sprintf('path = "../%s"', slots[[normalizePath(dep$path, winslash = "/")]]))
  }
  for (j in order(at, decreasing = TRUE)) lines <- append(lines, add[[j]], after = at[[j]])
  lines
}

# The R crate's own manifest cannot come from `cargo package`, which rejects or
# rewrites its git dependencies. Rewrite only its path literals that resolve to
# a staged crate, and keep the staged copies out of its workspace.
portable_manifest <- function(paths, plan) {
  lines <- plan$lines
  base <- dirname(paths$manifest)
  literal <- "(?<![A-Za-z0-9_-])path\\s*=\\s*(\"(?:[^\"\\\\]|\\\\.)*\"|'[^']*')"
  resolve <- function(hit) {
    quoted <- sub(literal, "\\1", hit, perl = TRUE)
    value <- substr(quoted, 2L, nchar(quoted) - 1L)
    if (startsWith(quoted, '"')) value <- gsub("\\\\(.)", "\\1", value, perl = TRUE)
    if (!grepl("^([A-Za-z]:)?[/\\\\]", value)) value <- file.path(base, value)
    normalizePath(value, winslash = "/", mustWork = FALSE)
  }
  rewritten <- character()
  in_deps <- FALSE
  code <- !startsWith(trimws(lines), "#")
  for (i in which(code)) {
    s <- trimws(lines[[i]])
    if (startsWith(s, "[")) {
      s <- sub("\\]\\s*#.*$", "]", s)
      in_deps <- grepl("dependencies(\\.[^]]+)?\\]$", s) && !startsWith(s, "[patch")
      next
    }
    if (!in_deps) next
    m <- gregexpr(literal, lines[[i]], perl = TRUE)
    hits <- regmatches(lines[[i]], m)[[1L]]
    if (!length(hits)) next
    regmatches(lines[[i]], m) <- list(vapply(hits, function(hit) {
      dir <- resolve(hit)
      if (is.null(plan$nodes[[dir]])) return(hit)
      rewritten <<- c(rewritten, dir)
      sprintf('path = "vendor/%s"', plan$slots[[dir]])
    }, "", USE.NAMES = FALSE))
  }
  direct <- unique(normalizePath(plan$root$deps$path[!is.na(plan$root$deps$path)], winslash = "/"))
  outside <- unlist(lapply(regmatches(lines[code], gregexpr(literal, lines[code], perl = TRUE)), function(hits) {
    hits[!vapply(hits, function(hit) path_inside(resolve(hit), paths$root), logical(1))]
  }))
  if (length(setdiff(direct, rewritten)) || length(outside)) {
    stop("Cannot rewrite the path dependencies of ", paths$manifest, " for staging: ",
         paste(c(setdiff(direct, rewritten), outside), collapse = ", "), call. = FALSE)
  }
  ws <- which(grepl("^\\s*\\[workspace\\]\\s*(#.*)?$", lines))
  if (!length(ws)) return(c(lines, "", "[workspace]", 'exclude = ["vendor"]'))
  heads <- which(grepl("^\\s*\\[", lines))
  end <- c(heads[heads > ws[[1L]]], length(lines) + 1L)[[1L]]
  body <- seq.int(ws[[1L]] + 1L, length.out = end - ws[[1L]] - 1L)
  ex <- body[grepl("^\\s*exclude\\s*=\\s*\\[", lines[body])]
  if (length(ex)) {
    lines[[ex[[1L]]]] <- sub("[", '["vendor", ', lines[[ex[[1L]]]], fixed = TRUE)
    lines
  } else {
    append(lines, 'exclude = ["vendor"]', after = ws[[1L]])
  }
}

# Stage every path dependency under src/rust/vendor/<name>-<version>/ when one
# lies outside the package. No .cargo-checksum.json: path crates need none.
stage_path_dependencies <- function(paths) {
  plan <- path_dependency_plan(paths)
  if (is.null(plan)) return(FALSE)
  # Same prefix as the backups, so an interrupted run stays ignored.
  stage <- tempfile(".dev-vendor-backup-", dirname(paths$manifest))
  tree <- file.path(stage, "vendor")
  dir.create(tree, recursive = TRUE)
  on.exit(unlink(stage, recursive = TRUE), add = TRUE)
  config <- unlist(lapply(names(plan$patches), function(name) {
    c("--config", shQuote(sprintf('patch.crates-io.%s.path="%s"', name, plan$patches[[name]])))
  }))
  for (dir in names(plan$nodes)) {
    node <- plan$nodes[[dir]]
    status <- system2("cargo", c("package", "--quiet", "--no-verify", "--allow-dirty", "--manifest-path",
      shQuote(node$manifest), "--target-dir", shQuote(file.path(stage, "target")), config))
    if (status != 0L) stop("cargo package failed for ", node$name, " (exit ", status, ").", call. = FALSE)
    if (utils::untar(file.path(stage, "target/package", paste0(plan$slots[[dir]], ".crate")), exdir = tree) != 0L) {
      stop("Unable to unpack the packaged ", node$name, call. = FALSE)
    }
    manifest <- file.path(tree, plan$slots[[dir]], "Cargo.toml")
    writeLines(restore_sibling_paths(node, readLines(manifest, warn = FALSE), plan$slots), manifest)
  }
  portable <- portable_manifest(paths, plan)
  dev_bootstrap_backup(paths, paths$vendor)
  if (!file.rename(tree, paths$vendor)) stop("Unable to publish ", paths$vendor, call. = FALSE)
  writeLines(portable, paths$portable)
  message("Staged ", length(plan$slots), " path dependencies under src/rust/vendor without cargo-revendor: ",
          paste(plan$slots, collapse = ", "))
  invisible(TRUE)
}

# endregion

prepare_dev_bootstrap <- function(root = ".", mode = "dev") {
  paths <- dev_bootstrap_paths(root)
  if (file.exists(file.path(paths$root, "inst/vendor.tar.xz"))) {
    stop("Development bootstrap requires a source tree without inst/vendor.tar.xz; ",
         "restore the source tree with minirextendr::miniextendr_clean_vendor_leak() first.",
         call. = FALSE)
  }
  original <- unname(tools::md5sum(paths$manifest))
  if (mode == "dev" && nzchar(Sys.which("cargo-revendor"))) {
    status <- system2("cargo", c("revendor", "--dev", "--manifest-path",
      shQuote(paths$manifest), "--output", shQuote(paths$vendor), "-v"))
    if (status != 0L) stop("Development bootstrap failed (exit ", status, ").", call. = FALSE)
  } else if (!stage_path_dependencies(paths)) {
    clear_dev_bootstrap(root)
    return(invisible(FALSE))
  }
  if (!identical(unname(tools::md5sum(paths$manifest)), original)) {
    stop("Development bootstrap changed the source Cargo.toml.", call. = FALSE)
  }
  saveRDS(list(root = paths$root, manifest = original, mode = mode), paths$state)
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
  if (!identical(Sys.getenv("MINIEXTENDR_BOOTSTRAP_MODE", "dist"), state$mode)) {
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
