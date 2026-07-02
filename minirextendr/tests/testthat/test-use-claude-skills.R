# use_claude_skills(): bundled agent skills installed into scaffolded packages

expected_skill_slugs <- c(
  "miniextendr-guide", "miniextendr-debugging", "miniextendr-conversions",
  "miniextendr-classes", "miniextendr-dataframe", "miniextendr-parallel",
  "miniextendr-release"
)

test_that("the bundled skill set ships complete, with frontmatter", {
  src <- system.file("claude", "skills", package = "minirextendr")
  skip_if(!nzchar(src), "inst/claude/skills not installed (devtools shim?)")

  slugs <- basename(list.dirs(src, recursive = FALSE))
  expect_setequal(slugs, expected_skill_slugs)

  for (slug in slugs) {
    skill_md <- file.path(src, slug, "SKILL.md")
    expect_true(file.exists(skill_md), info = slug)
    lines <- readLines(skill_md, warn = FALSE)
    # frontmatter with name + description ("Use when..." trigger)
    expect_identical(lines[[1]], "---", info = slug)
    expect_true(any(grepl(paste0("^name: ", slug, "$"), lines)), info = slug)
    expect_true(any(grepl("^description: Use when", lines)), info = slug)
    # self-contained budget from the plan: <= 250 lines per skill
    expect_lte(length(lines), 250)
  }
})

test_that("use_claude_skills() installs skills, .Rbuildignore, and AGENTS.md", {
  root <- tempfile("claude-skills-")
  on.exit(unlink(root, recursive = TRUE), add = TRUE)
  dir.create(root)
  tmp <- file.path(root, "testpkg")
  suppressMessages(usethis::create_package(tmp, open = FALSE))

  suppressMessages(use_claude_skills(path = tmp))

  installed <- basename(list.dirs(file.path(tmp, ".claude", "skills"),
                                  recursive = FALSE))
  expect_setequal(installed, expected_skill_slugs)

  rbuildignore <- readLines(file.path(tmp, ".Rbuildignore"), warn = FALSE)
  expect_true("^\\.claude$" %in% rbuildignore)

  agents <- file.path(tmp, "AGENTS.md")
  expect_true(file.exists(agents))
  expect_true(any(grepl("miniextendr-guide", readLines(agents, warn = FALSE))))
})

test_that("use_claude_skills() re-run overwrites stale copies, keeps user files", {
  root <- tempfile("claude-skills-rerun-")
  on.exit(unlink(root, recursive = TRUE), add = TRUE)
  dir.create(root)
  tmp <- file.path(root, "testpkg")
  suppressMessages(usethis::create_package(tmp, open = FALSE))
  suppressMessages(use_claude_skills(path = tmp))

  guide <- file.path(tmp, ".claude", "skills", "miniextendr-guide", "SKILL.md")
  writeLines("stale local edit", guide)
  custom <- file.path(tmp, ".claude", "skills", "my-own-skill", "SKILL.md")
  dir.create(dirname(custom), recursive = TRUE)
  writeLines("user-owned", custom)
  agents <- file.path(tmp, "AGENTS.md")
  writeLines("user-owned agents notes", agents)

  suppressMessages(use_claude_skills(path = tmp))

  # bundled skill restored from the package copy
  expect_true(any(grepl("^name: miniextendr-guide$",
                        readLines(guide, warn = FALSE))))
  # user-owned files untouched
  expect_identical(readLines(custom, warn = FALSE), "user-owned")
  expect_identical(readLines(agents, warn = FALSE), "user-owned agents notes")
  # .Rbuildignore not duplicated
  rbuildignore <- readLines(file.path(tmp, ".Rbuildignore"), warn = FALSE)
  expect_identical(sum(rbuildignore == "^\\.claude$"), 1L)
})

test_that("use_miniextendr(claude_skills = FALSE) opts out", {
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")

  root <- tempfile("claude-skills-optout-")
  on.exit(unlink(root, recursive = TRUE), add = TRUE)
  dir.create(root)
  tmp <- file.path(root, "testpkg")
  suppressMessages(usethis::create_package(tmp, open = FALSE))

  suppressWarnings(suppressMessages(
    use_miniextendr(path = tmp, claude_skills = FALSE)
  ))

  expect_false(dir.exists(file.path(tmp, ".claude")))
})

test_that("scaffolded package excludes .claude from the built tarball", {
  skip_on_cran()
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")

  tmp <- tempfile("claude-skills-tarball-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  pkg_path <- file.path(tmp, "tarpkg")
  suppressMessages(usethis::create_package(pkg_path, open = FALSE))
  suppressWarnings(suppressMessages(use_miniextendr(path = pkg_path)))

  expect_true(dir.exists(file.path(pkg_path, ".claude", "skills")))

  # R CMD build only stages files; the bootstrap/vendor machinery is
  # pkgbuild-driven and does not run here, so this stays fast and offline.
  result <- withr::with_dir(tmp, {
    system2("R", c("CMD", "build", "--no-build-vignettes", "tarpkg"),
            stdout = TRUE, stderr = TRUE)
  })
  status <- attr(result, "status")
  expect_true(is.null(status) || status == 0,
              info = paste(result, collapse = "\n"))

  tarball <- list.files(tmp, pattern = "^tarpkg_.*\\.tar\\.gz$",
                        full.names = TRUE)
  expect_length(tarball, 1L)
  contents <- untar(tarball, list = TRUE)
  expect_false(any(grepl("\\.claude", contents)))
})
