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
  err <- tempfile()
  on.exit(unlink(err), add = TRUE)
  out <- suppressWarnings(system2("cargo", c("metadata", "--no-deps", "--format-version", "1",
    "--offline", "--manifest-path", shQuote(manifest)), stdout = TRUE, stderr = err))
  if (!is.null(attr(out, "status"))) {
    stop("cargo metadata failed for ", manifest, ":\n", paste(readLines(err, warn = FALSE), collapse = "\n"),
         "\nFix the manifest error above, then rerun bootstrap.R.", call. = FALSE)
  }
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
# `all = TRUE` follows every edge, as `cargo revendor --dev` stages them.
path_dependency_closure <- function(paths, all = FALSE) {
  root <- cargo_manifest(paths$manifest)
  nodes <- list()
  patches <- character()
  problems <- character()
  dangling <- character()
  pending <- list(root)
  while (length(pending)) {
    info <- pending[[1L]]
    pending <- pending[-1L]
    sibling <- !identical(info$dir, root$dir)
    for (i in seq_len(nrow(info$deps))) {
      dep <- info$deps[i, ]
      dev <- identical(dep$kind, "dev") && !all
      if (sibling && !dev && !is.na(dep$source) && startsWith(dep$source, "git+")) {
        problems <- c(problems, sprintf("%s: `%s` is a git dependency, which `cargo package` cannot keep",
                                        info$manifest, dep$alias))
      }
      if (is.na(dep$path)) next
      dir <- normalizePath(dep$path, winslash = "/", mustWork = FALSE)
      if (identical(dir, root$dir)) next
      if (!file.exists(file.path(dir, "Cargo.toml"))) {
        dangling <- c(dangling, dir)
        problems <- c(problems, sprintf("%s: path dependency `%s` points at %s, which has no Cargo.toml; fix its `path`",
                                        info$manifest, dep$alias, dir))
        next
      }
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
  list(root = root, nodes = nodes, patches = patches, problems = problems, dangling = dangling)
}

# The staging plan, or NULL when no path in the manifest or its dependency
# closure leaves the package. Every blocking problem is reported in one error
# before staging.
path_dependency_plan <- function(paths) {
  plan <- path_dependency_closure(paths)
  plan$lines <- readLines(paths$manifest, warn = FALSE)
  plan$slots <- vapply(plan$nodes, function(node) paste0(node$name, "-", node$version), "")
  portable <- portable_manifest(paths, plan)
  outside <- !vapply(c(names(plan$nodes), plan$dangling), path_inside, logical(1), root = paths$root)
  if (!any(outside) && !length(portable$outside)) return(NULL)
  problems <- c(plan$problems, portable$outside, portable$missed)
  if (!any(grepl("^\\s*\\[workspace\\]\\s*(#.*)?$", plan$lines)) &&
      any(grepl("(^|[{,.[:space:]])workspace\\s*=\\s*true", plan$lines))) {
    problems <- c(problems, sprintf("%s: inherits from a parent workspace; give it its own [workspace] table",
                                    paths$manifest))
  }
  crates <- vapply(plan$nodes, `[[`, "", "name")
  if (anyDuplicated(crates)) {
    problems <- c(problems, sprintf("path dependencies share a package name: %s; rename one of them",
      paste(plan$slots[crates %in% crates[duplicated(crates)]], collapse = ", ")))
  }
  if (length(problems)) {
    stop(paste(c("Cannot stage path dependencies without cargo-revendor:", paste0("  - ", problems),
      "Fix the entries above, or install cargo-revendor:",
      "  cargo install --git https://github.com/A2-ai/miniextendr cargo-revendor --locked"),
      collapse = "\n"), call. = FALSE)
  }
  plan$portable <- portable$lines
  plan
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
# rewrites its git dependencies. Rewrite only its dependency-table path literals
# that resolve to a staged crate, keep the staged copies out of its workspace,
# and report every other path literal that would dangle once the package leaves
# its checkout (a [patch] or [replace] source, a target outside the package).
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
  outside <- character()
  section <- ""
  for (i in which(!startsWith(trimws(lines), "#"))) {
    s <- trimws(lines[[i]])
    if (startsWith(s, "[")) {
      section <- sub("\\]\\s*#.*$", "]", s)
      next
    }
    m <- gregexpr(literal, lines[[i]], perl = TRUE)
    hits <- regmatches(lines[[i]], m)[[1L]]
    if (!length(hits)) next
    deps <- grepl("dependencies(\\.[^]]+)?\\]$", section) && !startsWith(section, "[patch")
    regmatches(lines[[i]], m) <- list(vapply(hits, function(hit) {
      dir <- resolve(hit)
      if (deps && !is.null(plan$nodes[[dir]])) {
        rewritten <<- c(rewritten, dir)
        return(sprintf('path = "vendor/%s"', plan$slots[[dir]]))
      }
      if (!path_inside(dir, paths$root) && !dir %in% plan$dangling) {
        fix <- if (grepl("^\\[(patch|replace)", section)) {
          "patch sources are not staged, so move that crate inside the package"
        } else {
          "move it inside the package"
        }
        outside <<- c(outside, sprintf("%s: `%s` under %s points outside the package; %s",
                                       paths$manifest, hit, if (nzchar(section)) section else "the top level", fix))
      }
      hit
    }, "", USE.NAMES = FALSE))
  }
  deps <- plan$root$deps[!is.na(plan$root$deps$path), ]
  dirs <- normalizePath(deps$path, winslash = "/", mustWork = FALSE)
  missed <- unique(deps$alias[!dirs %in% c(rewritten, plan$dangling, plan$root$dir)])
  missed <- sprintf("%s: cannot find the `path = \"...\"` entry of `%s` to rewrite; declare it with a plain `path` key",
                    paths$manifest, missed)
  ws <- which(grepl("^\\s*\\[workspace\\]\\s*(#.*)?$", lines))
  if (!length(ws)) {
    lines <- c(lines, "", "[workspace]", 'exclude = ["vendor"]')
  } else {
    heads <- which(grepl("^\\s*\\[", lines))
    end <- c(heads[heads > ws[[1L]]], length(lines) + 1L)[[1L]]
    body <- seq.int(ws[[1L]] + 1L, length.out = end - ws[[1L]] - 1L)
    ex <- body[grepl("^\\s*exclude\\s*=\\s*\\[", lines[body])]
    if (length(ex)) {
      lines[[ex[[1L]]]] <- sub("[", '["vendor", ', lines[[ex[[1L]]]], fixed = TRUE)
    } else {
      lines <- append(lines, 'exclude = ["vendor"]', after = ws[[1L]])
    }
  }
  list(lines = lines, outside = outside, missed = missed)
}

# Stage every node of the plan under src/rust/vendor/<name>-<version>/.
# No .cargo-checksum.json: path crates need none.
stage_path_dependencies <- function(paths, plan) {
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
  if (!file.rename(tree, paths$vendor)) stop("Unable to publish ", paths$vendor, call. = FALSE)
  writeLines(plan$portable, paths$portable)
  message("Staged ", length(plan$slots), " path dependencies under src/rust/vendor without cargo-revendor: ",
          paste(plan$slots, collapse = ", "))
  invisible(TRUE)
}

# Fingerprint the source files behind each staged slot (its packaged file list
# mapped back to the origin, plus an inherited workspace manifest), so the build
# copy can refuse a staging that is older than its path dependencies.
path_dependency_sources <- function(nodes, vendor) {
  sources <- list()
  for (node in nodes) {
    slot <- paste0(node$name, "-", node$version)
    staged <- file.path(vendor, slot)
    if (!dir.exists(staged)) next
    files <- file.path(node$dir, list.files(staged, recursive = TRUE, all.files = TRUE))
    files <- files[file.exists(files)]
    if (!identical(node$workspace, node$dir)) files <- c(files, file.path(node$workspace, "Cargo.toml"))
    sources[[slot]] <- list(name = node$name, dir = node$dir, md5 = tools::md5sum(unique(files)))
  }
  sources
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
  tool <- mode == "dev" && nzchar(Sys.which("cargo-revendor"))
  plan <- if (!tool) path_dependency_plan(paths)
  clear_dev_bootstrap(root)
  if (!tool && is.null(plan)) return(invisible(FALSE))
  if (file.exists(paths$vendor)) {
    stop(paths$vendor, " exists but bootstrap did not create it (no .dev-bootstrap.rds beside Cargo.toml); ",
         "move it out of src/rust, then rerun bootstrap.R.", call. = FALSE)
  }
  # Anything at the staging paths from here on is this run's output.
  done <- FALSE
  on.exit(if (!done) unlink(c(paths$vendor, paths$portable), recursive = TRUE), add = TRUE)
  if (tool) {
    status <- system2("cargo", c("revendor", "--dev", "--manifest-path",
      shQuote(paths$manifest), "--output", shQuote(paths$vendor), "-v"))
    if (status != 0L) stop("Development bootstrap failed (exit ", status, ").", call. = FALSE)
    nodes <- path_dependency_closure(paths, all = TRUE)$nodes
  } else {
    stage_path_dependencies(paths, plan)
    nodes <- plan$nodes
  }
  if (!identical(unname(tools::md5sum(paths$manifest)), original)) {
    stop("Development bootstrap changed the source Cargo.toml.", call. = FALSE)
  }
  saveRDS(list(root = paths$root, manifest = original, mode = mode,
               sources = path_dependency_sources(nodes, paths$vendor)), paths$state)
  done <- TRUE
  invisible(TRUE)
}

# A staging is ours only when its state file exists. It is regenerated on
# every bootstrap, so it is removed rather than retained.
clear_dev_bootstrap <- function(root = ".") {
  paths <- dev_bootstrap_paths(root)
  if (file.exists(paths$state)) {
    unlink(c(paths$vendor, paths$portable, paths$state), recursive = TRUE)
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
  # Sources that no longer exist (a relocated checkout) cannot be compared.
  changed <- character()
  for (source in state$sources) {
    if (!dir.exists(source$dir)) next
    now <- tools::md5sum(names(source$md5))
    if (!identical(unname(now), unname(source$md5))) changed <- c(changed, source$name)
  }
  if (length(changed)) {
    stop(sprintf("path %s %s changed since bootstrap; rerun bootstrap.R.",
                 if (length(changed) == 1L) "dependency" else "dependencies", paste(changed, collapse = ", ")),
         call. = FALSE)
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
