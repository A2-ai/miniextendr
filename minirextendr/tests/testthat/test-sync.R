# Tests for sync, knitr, rmarkdown, and quarto integration

# =============================================================================
# compute_source_hash tests
# =============================================================================

test_that("compute_source_hash returns empty string for empty project", {
  tmp <- withr::local_tempdir()
  hash <- compute_source_hash(tmp)
  expect_equal(hash, "")
})

test_that("compute_source_hash returns consistent hash", {
  tmp <- withr::local_tempdir()
  rust_dir <- file.path(tmp, "src", "rust")
  dir.create(rust_dir, recursive = TRUE)
  writeLines("fn main() {}", file.path(rust_dir, "lib.rs"))
  writeLines('[package]\nname = "test"', file.path(rust_dir, "Cargo.toml"))

  h1 <- compute_source_hash(tmp)
  h2 <- compute_source_hash(tmp)
  expect_identical(h1, h2)
  expect_true(nchar(h1) > 0)
})

test_that("compute_source_hash changes when source changes", {
  tmp <- withr::local_tempdir()
  rust_dir <- file.path(tmp, "src", "rust")
  dir.create(rust_dir, recursive = TRUE)
  writeLines("fn main() {}", file.path(rust_dir, "lib.rs"))

  h1 <- compute_source_hash(tmp)

  writeLines("fn main() { println!(\"hello\"); }", file.path(rust_dir, "lib.rs"))
  h2 <- compute_source_hash(tmp)

  expect_false(identical(h1, h2))
})

# =============================================================================
# render state read/write tests
# =============================================================================

test_that("read_render_state returns NULL for missing state", {
  tmp <- withr::local_tempdir()
  expect_null(read_render_state(tmp))
})

test_that("write/read_render_state round-trips", {
  tmp <- withr::local_tempdir()
  write_render_state("abc123", "install", tmp)

  state <- read_render_state(tmp)
  expect_equal(state$hash, "abc123")
  expect_equal(state$stage_run, "install")
  expect_s3_class(state$timestamp, "POSIXct")
})

test_that("read_render_state handles corrupted file", {
  tmp <- withr::local_tempdir()
  state_file <- file.path(tmp, "tools", ".miniextendr", "render-state.rds")
  dir.create(dirname(state_file), recursive = TRUE)
  writeLines("not an RDS file", state_file)

  expect_null(read_render_state(tmp))
  # Corrupted file should be removed

  expect_false(file.exists(state_file))
})

# =============================================================================
# miniextendr_sync tests (mode dispatch)
# =============================================================================

test_that("miniextendr_sync with mode='never' skips rebuild", {
  tmp <- withr::local_tempdir()
  # Set up minimal project
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  result <- miniextendr_sync(path = tmp, mode = "never", quiet = TRUE)
  expect_true(result$fresh)
  expect_equal(result$stage_run, "none")
})

test_that("miniextendr_sync argument validation works", {
  tmp <- withr::local_tempdir()
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  expect_error(miniextendr_sync(path = tmp, mode = "invalid"),
               "should be one of")
  expect_error(miniextendr_sync(path = tmp, stage = "invalid"),
               "should be one of")
})

test_that("miniextendr_sync force=TRUE overrides mode", {
  tmp <- withr::local_tempdir()
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  # force=TRUE with mode="never" should still try to rebuild
  # (will error because there's no real Rust project, but the mode override works)
  expect_error(
    miniextendr_sync(path = tmp, mode = "never", force = TRUE, quiet = TRUE)
  )
})

test_that("miniextendr_sync mode='always' always rebuilds", {
  tmp <- withr::local_tempdir()
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  # mode="always" should attempt rebuild even with saved state
  # (will error because there's no real Rust project, but mode dispatch works)
  expect_error(
    miniextendr_sync(path = tmp, mode = "always", quiet = TRUE)
  )
})

test_that("miniextendr_sync mode='if_stale' skips when hash matches", {
  tmp <- withr::local_tempdir()
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  # Write saved state with current hash
  current_hash <- compute_source_hash(tmp)
  write_render_state(current_hash, "install", tmp)

  result <- miniextendr_sync(path = tmp, mode = "if_stale", quiet = TRUE)
  expect_true(result$fresh)
  expect_equal(result$stage_run, "none")
})

test_that("miniextendr_sync mode='if_stale' rebuilds when hash differs", {
  tmp <- withr::local_tempdir()
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  # Write saved state with stale hash
  write_render_state("stale-hash-value", "install", tmp)

  # Should attempt rebuild because hash differs
  # (will error because there's no real Rust project)
  expect_error(
    miniextendr_sync(path = tmp, mode = "if_stale", quiet = TRUE)
  )
})

test_that("miniextendr_sync mode='if_stale' rebuilds when no saved state", {
  tmp <- withr::local_tempdir()
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  # No saved state means stale
  expect_error(
    miniextendr_sync(path = tmp, mode = "if_stale", quiet = TRUE)
  )
})

# =============================================================================
# knitr helper tests
# =============================================================================

test_that("miniextendr_knitr_setup requires knitr", {
  # knitr is in Suggests, so this tests the check
  skip_if_not_installed("knitr")
  # Can't fully test without a real project, but we can verify it exists
  expect_true(is.function(miniextendr_knitr_setup))
})

test_that("eng_miniextendr is a valid engine function", {
  expect_true(is.function(eng_miniextendr))
  # Engine functions take a single 'options' argument
  expect_equal(names(formals(eng_miniextendr)), "options")
})

# =============================================================================
# rmarkdown format tests
# =============================================================================

test_that("miniextendr_html_document creates valid format", {
  skip_if_not_installed("rmarkdown")

  fmt <- miniextendr_html_document()
  expect_true(is.function(fmt$pre_knit))
})

test_that("miniextendr_pdf_document creates valid format", {
  skip_if_not_installed("rmarkdown")

  fmt <- miniextendr_pdf_document()
  expect_true(is.function(fmt$pre_knit))
})

test_that("miniextendr_word_document creates valid format", {
  skip_if_not_installed("rmarkdown")

  fmt <- miniextendr_word_document()
  expect_true(is.function(fmt$pre_knit))
})

test_that("miniextendr_quarto_pre_render is exported", {
  expect_true(is.function(miniextendr_quarto_pre_render))
})

# =============================================================================
# scaffolding helper tests
# =============================================================================

test_that("use_miniextendr_knitr adds setup chunk", {
  tmp <- withr::local_tempdir()
  rmd <- file.path(tmp, "test.Rmd")
  writeLines(c(
    "---",
    "title: Test",
    "---",
    "",
    "Some content"
  ), rmd)

  result <- use_miniextendr_knitr(rmd)
  expect_true(result)

  lines <- readLines(rmd)
  expect_true(any(grepl("miniextendr_knitr_setup", lines)))

  # Running again should be idempotent
  result2 <- use_miniextendr_knitr(rmd)
  expect_false(result2)
})

test_that("use_miniextendr_knitr validates arguments", {
  expect_error(use_miniextendr_knitr(), "Provide a single path")
  expect_error(use_miniextendr_knitr(c("a", "b")), "Provide a single path")
  expect_error(use_miniextendr_knitr("nonexistent.Rmd"), "File not found")
})

test_that("use_miniextendr_rmarkdown sets output format", {
  tmp <- withr::local_tempdir()
  rmd <- file.path(tmp, "test.Rmd")
  writeLines(c(
    "---",
    "title: Test",
    "output: html_document",
    "---",
    "",
    "Some content"
  ), rmd)

  result <- use_miniextendr_rmarkdown(rmd)
  expect_true(result)

  lines <- readLines(rmd)
  expect_true(any(grepl("minirextendr::miniextendr_html_document", lines)))

  # Running again should be idempotent
  result2 <- use_miniextendr_rmarkdown(rmd)
  expect_false(result2)
})

test_that("use_miniextendr_rmarkdown adds output when missing", {
  tmp <- withr::local_tempdir()
  rmd <- file.path(tmp, "test.Rmd")
  writeLines(c(
    "---",
    "title: Test",
    "---",
    "",
    "Some content"
  ), rmd)

  result <- use_miniextendr_rmarkdown(rmd, format = "pdf_document")
  expect_true(result)

  lines <- readLines(rmd)
  expect_true(any(grepl("minirextendr::miniextendr_pdf_document", lines)))
})

test_that("use_miniextendr_rmarkdown validates arguments", {
  expect_error(use_miniextendr_rmarkdown(), "Provide a single path")
  expect_error(use_miniextendr_rmarkdown("nonexistent.Rmd"), "File not found")
})

test_that("use_miniextendr_quarto creates _quarto.yml", {
  tmp <- withr::local_tempdir()

  result <- use_miniextendr_quarto(tmp)
  expect_true(result)

  qmd <- file.path(tmp, "_quarto.yml")
  expect_true(file.exists(qmd))
  lines <- readLines(qmd)
  expect_true(any(grepl("miniextendr_quarto_pre_render", lines)))

  # Running again should be idempotent
  result2 <- use_miniextendr_quarto(tmp)
  expect_false(result2)
})

test_that("use_miniextendr_quarto appends to existing file", {
  tmp <- withr::local_tempdir()
  qmd <- file.path(tmp, "_quarto.yml")
  writeLines(c(
    "project:",
    "  type: website"
  ), qmd)

  result <- use_miniextendr_quarto(tmp)
  expect_true(result)

  lines <- readLines(qmd)
  expect_true(any(grepl("miniextendr_quarto_pre_render", lines)))
  # Original content preserved
  expect_true(any(grepl("type: website", lines)))
})
