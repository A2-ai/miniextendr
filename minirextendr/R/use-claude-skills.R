# Claude Code skills for scaffolded packages

#' Install the miniextendr Claude Code skill set into a package
#'
#' Copies the bundled agent skills (`.claude/skills/miniextendr-*`) into the
#' project so coding agents working in the package get progressive-loading
#' context about the miniextendr build pipeline, conversions, class systems,
#' data.frame interface, parallelism rules, debugging recipes, and the
#' release/vendoring story -- all framed for the scaffolded-package layout
#' (`src/rust/lib.rs`, `configure`, `tools/`).
#'
#' Re-running replaces the set: every installed `miniextendr-*` skill
#' directory is deleted, including one the bundle no longer ships, and the
#' bundled skills are copied in its place (the same delete-target-first
#' convention as [use_miniextendr()] templates). Local edits to those skills
#' are overwritten; merging them back, from git, is up to you.
#' [upgrade_miniextendr_package()] does the same for a package that has the
#' skills installed. Skill directories you added yourself under
#' `.claude/skills/` (not named `miniextendr-*`) are left untouched.
#'
#' When the project root is an R package (a `DESCRIPTION` is present),
#' `.claude/` and `AGENTS.md` are added to `.Rbuildignore` so agent guidance
#' never ships in the tarball. An `AGENTS.md` stub pointing non-Claude agents at the skill set is
#' written only if the file does not already exist -- it is user-owned content.
#'
#' @param path Path to the project root, or `"."` to use the current directory.
#'   For monorepo layouts call this at the workspace root (where agents run),
#'   not inside the R package subdirectory.
#' @return Invisibly returns TRUE
#' @export
use_claude_skills <- function(path = ".") {
  with_project(path)

  replace_miniextendr_skills(usethis::proj_get())

  # Keep the skills out of the built tarball. Only meaningful when the project
  # root is itself the R package; in a monorepo the R package lives in a
  # subdirectory and never sees the workspace-root .claude/.
  if (fs::file_exists(usethis::proj_path("DESCRIPTION"))) {
    usethis::use_build_ignore(c("^\\.claude$", "^AGENTS\\.md$"), escape = FALSE)
  }

  write_agents_md_stub()

  invisible(TRUE)
}

#' The skill set bundled with minirextendr
#'
#' @return Path of the installed minirextendr's `claude/skills` directory,
#'   which holds one `miniextendr-*` directory per skill.
#' @keywords internal
bundled_skills_dir <- function() {
  system.file("claude", "skills", package = "minirextendr", mustWork = TRUE)
}

#' The miniextendr skills installed in a project
#'
#' @param root Project root whose `.claude/skills/` holds the skills.
#' @return Names of the `miniextendr-*` directories under
#'   `<root>/.claude/skills/`; empty when there is none.
#' @keywords internal
installed_miniextendr_skills <- function(root) {
  slugs <- list.dirs(file.path(root, ".claude", "skills"),
                     full.names = FALSE, recursive = FALSE)
  slugs[startsWith(slugs, "miniextendr-")]
}

#' Replace a project's miniextendr skills with the bundled set
#'
#' Deletes every installed `miniextendr-*` skill directory, including one the
#' bundle no longer ships, then copies the bundled skills. Deleting first
#' makes the copy an overwrite: `fs::dir_copy()` would nest, and usethis-style
#' `write_over()` skips silently in non-interactive mode. The user's own
#' skills under `.claude/skills/` are kept.
#'
#' @param root Project root to install the skills under.
#' @return `NULL` invisibly; called for its side effect on
#'   `<root>/.claude/skills/`.
#' @keywords internal
replace_miniextendr_skills <- function(root) {
  dest <- file.path(root, ".claude", "skills")
  ensure_dir(dest)

  src <- bundled_skills_dir()
  bundled <- list.dirs(src, full.names = FALSE, recursive = FALSE)
  for (slug in installed_miniextendr_skills(root)) {
    fs::dir_delete(file.path(dest, slug))
    if (!slug %in% bundled) {
      cli::cli_alert_success("Removed skill {.path {file.path('.claude', 'skills', slug)}}, which minirextendr no longer ships")
    }
  }
  for (slug in bundled) {
    fs::dir_copy(file.path(src, slug), file.path(dest, slug))
    bullet_created(file.path(".claude", "skills", slug), "Installed skill")
  }
  invisible()
}

#' Write the AGENTS.md stub if absent
#'
#' AGENTS.md is user-owned once created, so this never overwrites.
#'
#' @return Invisibly, `TRUE` if the active project's `AGENTS.md` was
#'   written, `FALSE` if it already existed.
#' @keywords internal
write_agents_md_stub <- function() {
  agents_path <- usethis::proj_path("AGENTS.md")
  if (fs::file_exists(agents_path)) {
    cli::cli_alert_info("{.path AGENTS.md} already exists, leaving it unchanged")
    return(invisible(FALSE))
  }
  writeLines(
    c(
      "# Agent notes",
      "",
      "This R package has a Rust backend built with the",
      "[miniextendr](https://a2-ai.github.io/miniextendr) framework.",
      "",
      "Task-specific context lives in `.claude/skills/miniextendr-*/SKILL.md`",
      "(build pipeline, type conversions, class systems, data.frames,",
      "parallelism, debugging, releasing). Start with",
      "`.claude/skills/miniextendr-guide/SKILL.md` for orientation and a",
      "routing table to the rest.",
      "",
      "Ground rules:",
      "",
      "- Rebuild after Rust changes with `minirextendr::miniextendr_build()` \u2014",
      "  not bare `R CMD INSTALL .` / `devtools::install()` (they skip wrapper",
      "  generation on a fresh tree; the guide skill explains why).",
      "- Never hand-edit generated files (`R/*-wrappers.R`, `src/Makevars`,",
      "  `configure`, `src/rust/wasm_registry.rs`).",
      "- `minirextendr::miniextendr_doctor()` diagnoses most broken states."
    ),
    agents_path
  )
  bullet_created("AGENTS.md")
  invisible(TRUE)
}
