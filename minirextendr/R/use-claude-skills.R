# Claude Code skills for scaffolded packages

#' Install the miniextendr Claude Code skill set into a package
#'
#' Copies the bundled agent skills (`.claude/skills/miniextendr-*`) into the
#' project so coding agents working in the package get progressive-loading
#' context about the miniextendr build pipeline, conversions, class systems,
#' data.frame interface, parallelism rules, debugging recipes, and the
#' release/vendoring story — all framed for the scaffolded-package layout
#' (`src/rust/lib.rs`, `configure`, `tools/`).
#'
#' Re-running upgrades in place: each bundled skill directory is deleted and
#' rewritten (the same delete-target-first convention as [use_miniextendr()]
#' templates), so stale copies never linger. Skill directories you added
#' yourself under `.claude/skills/` are left untouched.
#'
#' When the project root is an R package (a `DESCRIPTION` is present),
#' `^\.claude$` is added to `.Rbuildignore` so the skills never ship in the
#' tarball. An `AGENTS.md` stub pointing non-Claude agents at the skill set is
#' written only if the file does not already exist — it is user-owned content.
#'
#' @param path Path to the project root, or `"."` to use the current directory.
#'   For monorepo layouts call this at the workspace root (where agents run),
#'   not inside the R package subdirectory.
#' @return Invisibly returns TRUE
#' @export
use_claude_skills <- function(path = ".") {
  with_project(path)

  src <- system.file("claude", "skills", package = "minirextendr", mustWork = TRUE)
  dest <- usethis::proj_path(".claude", "skills")
  ensure_dir(dest)

  for (slug_dir in fs::dir_ls(src, type = "directory")) {
    slug <- fs::path_file(slug_dir)
    target <- fs::path(dest, slug)
    # Delete-first so re-runs actually overwrite (fs::dir_copy() would nest,
    # and usethis-style write_over() skips silently in non-interactive mode).
    if (fs::dir_exists(target)) {
      fs::dir_delete(target)
    }
    fs::dir_copy(slug_dir, target)
    bullet_created(file.path(".claude", "skills", slug), "Installed skill")
  }

  # Keep the skills out of the built tarball. Only meaningful when the project
  # root is itself the R package; in a monorepo the R package lives in a
  # subdirectory and never sees the workspace-root .claude/.
  if (fs::file_exists(usethis::proj_path("DESCRIPTION"))) {
    usethis::use_build_ignore("^\\.claude$", escape = FALSE)
  }

  write_agents_md_stub()

  invisible(TRUE)
}

#' Write the AGENTS.md stub if absent
#'
#' AGENTS.md is user-owned once created, so this never overwrites.
#' @noRd
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
      "- Rebuild after Rust changes with `minirextendr::miniextendr_build()` —",
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
