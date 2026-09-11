test_that("monorepo examples render parseable Rust with either prefix", {
  skip_if_not(nzchar(Sys.which("rustfmt")), "rustfmt not available")
  root <- withr::local_tempdir()
  usethis::local_project(root, force = TRUE, setwd = FALSE)
  previous_type <- minirextendr:::get_template_type()
  minirextendr:::set_template_type("monorepo")
  withr::defer(minirextendr:::set_template_type(previous_type))

  for (prefix in c("", "// ")) {
    suppressMessages(minirextendr:::use_template(
      "lib.rs", save_as = "lib.rs", subdir = "rpkg",
      data = list(crate_name = "example-core", core_example_prefix = prefix)
    ))
    source <- file.path(root, "lib.rs")
    lines <- readLines(source)
    expect_true(any(startsWith(lines, paste0(prefix, "pub fn core_greeting()"))))
    log <- file.path(root, "rustfmt.log")
    status <- system2("rustfmt", c("--edition", "2024", "--emit", "stdout",
                                   shQuote(source)), stdout = log, stderr = log)
    expect_identical(status, 0L, info = paste(readLines(log), collapse = "\n"))
  }
})
