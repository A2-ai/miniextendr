# Upgrade fidelity: what upgrade_miniextendr_package() keeps, writes and
# reports (#1711, #1712, #1669, #1670).

# region: #1711 — config.guess / config.sub timestamps ----------------------

# A copy of the bundled script with its timestamp line replaced.
write_config_script <- function(dest, script, timestamp) {
  lines <- readLines(minirextendr:::script_path(script), warn = FALSE)
  stamp <- grep("^timestamp='", lines)[[1L]]
  if (is.null(timestamp)) {
    lines <- lines[-stamp]
  } else {
    lines[[stamp]] <- sprintf("timestamp='%s'", timestamp)
  }
  writeLines(lines, dest)
  dest
}

test_that("bundled config scripts carry a parseable timestamp", {
  for (script in c("config.guess", "config.sub")) {
    ts <- minirextendr:::config_script_timestamp(minirextendr:::script_path(script))
    expect_match(ts, "^[0-9]{4}-[0-9]{2}-[0-9]{2}$", info = script)
  }
})

test_that("copy_config_scripts() keeps a newer script and replaces an older one", {
  tmp <- withr::local_tempdir()
  newer <- write_config_script(file.path(tmp, "config.guess"), "config.guess", "2999-01-01")
  older <- write_config_script(file.path(tmp, "config.sub"), "config.sub", "2000-01-01")
  newer_bytes <- readBin(newer, "raw", file.size(newer))

  msgs <- capture_messages(minirextendr:::copy_config_scripts(tmp, display_prefix = "tools"))

  expect_identical(readBin(newer, "raw", file.size(newer)), newer_bytes)
  expect_match(paste(msgs, collapse = ""), "Kept.*tools/config\\.guess.*2999-01-01.*newer than the bundled")
  bundled_sub <- minirextendr:::script_path("config.sub")
  expect_identical(unname(tools::md5sum(older)), unname(tools::md5sum(bundled_sub)))
  expect_false(any(grepl("Kept.*config\\.sub", msgs)))
})

test_that("copy_config_scripts() replaces a script without a timestamp", {
  tmp <- withr::local_tempdir()
  for (script in c("config.guess", "config.sub")) {
    write_config_script(file.path(tmp, script), script, NULL)
  }
  suppressMessages(minirextendr:::copy_config_scripts(tmp, display_prefix = NULL))
  for (script in c("config.guess", "config.sub")) {
    expect_identical(unname(tools::md5sum(file.path(tmp, script))),
                     unname(tools::md5sum(minirextendr:::script_path(script))),
                     info = script)
  }
})

# endregion -------------------------------------------------------------------
