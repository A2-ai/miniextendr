# Tests for custom R connections backed by Rust types.
#
# Requires the `connections` feature.

skip_if_missing_feature("connections")

# =============================================================================
# String input connection
# =============================================================================

test_that("string_input_connection reads multi-line content", {
  con <- string_input_connection("line1\nline2\nline3")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_length(lines, 3)
  expect_equal(lines, c("line1", "line2", "line3"))
})

test_that("string_input_connection reads single line", {
  con <- string_input_connection("hello")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_length(lines, 1)
  expect_equal(lines, "hello")
})

test_that("string_input_connection reads empty string", {
  con <- string_input_connection("")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_length(lines, 0)
})

# =============================================================================
# Counter connection
# =============================================================================

test_that("counter_connection generates sequential integers", {
  con <- counter_connection(1L, 5L)
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, as.character(1:5))
})

test_that("counter_connection with single value", {
  con <- counter_connection(42L, 42L)
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "42")
})

# =============================================================================
# Memory connection (read + write + seek)
# =============================================================================

test_that("memory_connection write then read roundtrip", {
  con <- memory_connection()
  on.exit(close(con))
  writeLines("Hello, World!", con)
  seek(con, 0)
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "Hello, World!")
})

test_that("memory_connection write multiple lines", {
  con <- memory_connection()
  on.exit(close(con))
  writeLines(c("foo", "bar", "baz"), con)
  seek(con, 0)
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, c("foo", "bar", "baz"))
})

# =============================================================================
# Uppercase transform connection
# =============================================================================

test_that("uppercase_connection transforms to uppercase", {
  con <- uppercase_connection("hello world")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "HELLO WORLD")
})

test_that("uppercase_connection preserves non-alpha characters", {
  con <- uppercase_connection("abc 123 !@#")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "ABC 123 !@#")
})

# =============================================================================
# ROT13 connection
# =============================================================================

test_that("rot13_connection encodes text", {
  con <- rot13_connection("hello")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "uryyb")
})

test_that("rot13 double-application returns original", {
  # ROT13 is its own inverse
  con1 <- rot13_connection("hello world")
  on.exit(close(con1), add = TRUE)
  encoded <- readLines(con1, warn = FALSE)

  con2 <- rot13_connection(encoded)
  on.exit(close(con2), add = TRUE)
  decoded <- readLines(con2, warn = FALSE)

  expect_equal(decoded, "hello world")
})

# =============================================================================
# Cursor connection (binary read/write/seek)
# =============================================================================

test_that("empty_cursor_connection binary roundtrip", {
  con <- empty_cursor_connection()
  on.exit(close(con))
  data <- as.raw(1:10)
  writeBin(data, con)
  seek(con, 0)
  result <- readBin(con, "raw", 10)
  expect_equal(result, data)
})

test_that("cursor_connection reads pre-filled data", {
  data <- charToRaw("Hello")
  con <- cursor_connection(data)
  on.exit(close(con))
  result <- readBin(con, "raw", 5)
  expect_equal(rawToChar(result), "Hello")
})

# =============================================================================
# I/O failures and seek positions — issue #1923
# =============================================================================

test_that("a failing read, write, flush or seek raises a classed condition", {
  con <- failing_io_connection()
  on.exit(close(con))
  catch <- function(expr) {
    tryCatch(expr, miniextendr_connection_io_error = function(e) e)
  }
  read <- catch(readBin(con, "raw", 1L))
  expect_s3_class(read, "miniextendr_connection_io_error")
  expect_s3_class(read, "rust_error")
  expect_identical(read$operation, "read")
  expect_identical(read$io_kind, "PermissionDenied")
  expect_identical(read$os_error, NA_integer_)
  expect_match(conditionMessage(read), "read refused", fixed = TRUE)
  expect_identical(catch(writeBin(as.raw(1L), con))$operation, "write")
  expect_identical(catch(flush(con))$operation, "flush")
  expect_identical(catch(seek(con, 0))$operation, "seek")
  expect_error(
    tryCatch(readBin(con, "raw", 1L), rust_error = function(e) stop("caught")),
    "caught"
  )
})

test_that("seek() returns the position before the move, as rawConnection's does", {
  x <- cursor_connection(charToRaw("abcde"))
  y <- rawConnection(charToRaw("abcde"), "rb")
  on.exit({
    close(x)
    close(y)
  })
  positions <- function(con) {
    c(
      seek(con, 2L), seek(con), seek(con, 1L, "current"),
      seek(con, -1L, "end"), seek(con, 2.7), seek(con)
    )
  }
  expect_identical(positions(y), c(0, 2, 2, 3, 4, 2))
  expect_identical(positions(x), c(0, 2, 2, 3, 4, 2))

  mem <- memory_connection()
  on.exit(close(mem), add = TRUE)
  writeBin(charToRaw("abcde"), mem)
  expect_identical(seek(mem, 0), 5)
  expect_identical(seek(mem), 0)
})

test_that("seek() before the start stays put, as file()'s does", {
  positions <- function(con) c(seek(con, 3L), seek(con, -1L), seek(con))
  x <- cursor_connection(charToRaw("abcde"))
  on.exit(close(x))
  expect_identical(positions(x), c(0, 3, 3))

  # ?seek: file positioning on Windows is unreliable.
  skip_on_os("windows")
  path <- tempfile()
  writeBin(charToRaw("abcde"), path)
  f <- file(path, "rb")
  on.exit({
    close(f)
    unlink(path)
  }, add = TRUE)
  expect_identical(positions(f), c(0, 3, 3))
})

test_that("seek() and truncate() refuse a connection that supports neither", {
  con <- counter_connection(1L, 3L)
  on.exit(close(con))
  expect_error(seek(con), "'seek' not enabled for this connection", fixed = TRUE)
  mem <- memory_connection()
  on.exit(close(mem), add = TRUE)
  expect_error(
    truncate(mem),
    "truncation not enabled for this connection",
    fixed = TRUE
  )
})

# =============================================================================
# Standard stream connections — issue #175
# =============================================================================

test_that("rust_get_stdout returns R's stdout connection", {
  con <- rust_get_stdout()
  expect_true(inherits(con, "terminal"))
  expect_true(inherits(con, "connection"))
  expect_equal(summary(con)$description, "stdout")
})

test_that("rust_get_stderr returns R's stderr connection", {
  con <- rust_get_stderr()
  expect_true(inherits(con, "terminal"))
  expect_true(inherits(con, "connection"))
  expect_equal(summary(con)$description, "stderr")
})

test_that("rust_write_to_stderr captured by capture.output(type='message')", {
  out <- capture.output(rust_write_to_stderr("hello-from-rust"), type = "message")
  expect_true(any(grepl("hello-from-rust", out)))
})

test_that("rust_write_to_stderr captured by sink(type='message')", {
  buf <- character(0)
  tc <- textConnection("buf", "w", local = TRUE)
  sink(tc, type = "message")
  rust_write_to_stderr("sink-captured")
  sink(NULL, type = "message")
  close(tc)
  expect_true(any(grepl("sink-captured", buf)))
})

# =============================================================================
# Reading R's console input through RStdin — issue #1912
# =============================================================================

# Evaluate `call` (R code, as text) in a child R whose console input goes on
# with `input` (exact bytes), and return the call's value.
#
# Under Rscript, R's stdin() is the script, not the process's standard input,
# so `input` follows the code in the script. The code line ends with q(), so
# input the call leaves unread is never parsed as R.
console_input_child <- function(call, input) {
  script <- withr::local_tempfile(fileext = ".R")
  code <- sprintf("dput(%s); q(save = \"no\")\n", call)
  writeBin(charToRaw(paste0(code, input)), script)
  out <- callr::rscript(script, libpath = .libPaths(), show = FALSE)
  eval(parse(text = out$stdout))
}

test_that("RStdin reads R's console input one line per read", {
  skip_on_cran()
  skip_if_not_installed("callr")
  expect_identical(
    console_input_child("miniextendr::rust_stdin_read_chunks(64L)", "hello\nworld\n"),
    c("hello\n", "world\n")
  )
})

test_that("RStdin ends every line with \\n, a last line without one too", {
  skip_on_cran()
  skip_if_not_installed("callr")
  expect_identical(
    console_input_child("miniextendr::rust_stdin_read_chunks(64L)", "crlf\r\ncr\rlast"),
    c("crlf\n", "cr\n", "last\n")
  )
})

test_that("RStdin reads nothing from empty input", {
  skip_on_cran()
  skip_if_not_installed("callr")
  expect_identical(
    console_input_child("miniextendr::rust_stdin_read_chunks(64L)", ""),
    character(0)
  )
  expect_identical(
    console_input_child("miniextendr::rust_stdin_lines(5L)", ""),
    character(0)
  )
})

test_that("RStdin reads a line longer than the buffer in pieces", {
  skip_on_cran()
  skip_if_not_installed("callr")
  expect_identical(
    console_input_child("miniextendr::rust_stdin_read_chunks(3L)", "hello\nhi\n"),
    c("hel", "lo\n", "hi\n")
  )
})

test_that("RStdin::lines() returns the lines without their terminators", {
  skip_on_cran()
  skip_if_not_installed("callr")
  expect_identical(
    console_input_child("miniextendr::rust_stdin_lines(10L)", "one\ntwo\r\nthree"),
    c("one", "two", "three")
  )
  # take(2) leaves the third line unread.
  expect_identical(
    console_input_child("miniextendr::rust_stdin_lines(2L)", "a\nb\nc\n"),
    c("a", "b")
  )
})

test_that("Read and BufRead on separate RStdin handles read one stream", {
  skip_on_cran()
  skip_if_not_installed("callr")
  expect_identical(
    console_input_child("miniextendr::rust_stdin_read_then_lines(2L)", "hello\nworld\nrest\n"),
    c("he", "llo\n", "world\n")
  )
})

# =============================================================================
# Null connection — issue #176
# =============================================================================

test_that("rust_get_null_connection returns an open write-capable connection", {
  con <- rust_get_null_connection()
  on.exit(close(con))
  caps <- summary(con)
  expect_equal(caps$mode, "w")
  expect_equal(caps$opened, "opened")
  expect_equal(caps[["can read"]], "no")
  expect_equal(caps[["can write"]], "yes")
})

test_that("rust_write_to_null succeeds silently (no error)", {
  expect_no_error(rust_write_to_null("discarded message"))
})

test_that("close(rust_get_null_connection()) does not error", {
  con <- rust_get_null_connection()
  expect_no_error(close(con))
})

test_that("rust_get_null_connection double-close is safe", {
  con <- rust_get_null_connection()
  close(con)
  rm(con)
  gc()
  succeed()
})

test_that("an owned RNullConnection closes its own connection on drop (#1917)", {
  before <- nrow(showConnections(all = TRUE))
  rust_write_to_null("discarded")
  expect_equal(nrow(showConnections(all = TRUE)), before)
})

test_that("a panic with an open RNullConnection is an R error, and the handle closes (#1917)", {
  before <- nrow(showConnections(all = TRUE))
  expect_error(rust_null_conn_panic_while_open("boom-while-open"), "boom-while-open")
  expect_equal(nrow(showConnections(all = TRUE)), before)
})

test_that("a connection passed in as RNullConnection stays open after the call (#1917)", {
  tmp <- tempfile()
  con <- file(tmp, "w")
  on.exit({
    try(close(con), silent = TRUE)
    unlink(tmp)
  })
  rust_null_conn_write(con, "from-rust")
  expect_true(isOpen(con))
  expect_no_error(writeLines("after", con))
  close(con)
  expect_equal(readLines(tmp), c("from-rust", "after"))
})

test_that("close() on a borrowed RNullConnection leaves the R connection open (#1917)", {
  tmp <- tempfile()
  con <- file(tmp, "w")
  on.exit({
    try(close(con), silent = TRUE)
    unlink(tmp)
  })
  expect_true(rust_null_conn_close_then_write(con))
  expect_true(isOpen(con))
  writeLines("after", con)
  close(con)
  expect_equal(readLines(tmp), "after")
})

# =============================================================================
# txtProgressBar (PR B / #177)
# =============================================================================

test_that("rust_run_progress emits a style-3 progress bar", {
  out <- capture.output(rust_run_progress(5L))
  # style 3 looks like: |====      | 80% or |==========| 100%
  expect_true(any(grepl("\\|.+\\|.+%", out)))
  # set() advances the bar past its initial 0% draw (#1917).
  expect_true(any(grepl("100%", out, fixed = TRUE)))
})

test_that("rust_run_progress_to_stderr captures via sink(message)", {
  buf <- character(0)
  con <- textConnection("buf", "w", local = TRUE)
  sink(con, type = "message")
  rust_run_progress_to_stderr(5L)
  sink(NULL, type = "message")
  close(con)
  expect_true(any(grepl("%", buf)))
})

test_that("RTxtProgressBar Drop runs cleanly", {
  # Exercises the auto-close Drop path end-to-end without error.
  expect_no_error(rust_run_progress(3L))
})

test_that("RTxtProgressBar explicit close() succeeds and Drop is no-op", {
  expect_true(rust_run_progress_explicit_close(5L))
})

test_that("gc_stress_txt_progress_bar does not segfault", {
  expect_no_error(miniextendr:::gc_stress_txt_progress_bar())
})

test_that("a txtProgressBar passed in as RTxtProgressBar stays open after the call (#1917)", {
  buf <- character(0)
  tc <- textConnection("buf", "w", local = TRUE)
  on.exit(close(tc))
  pb <- txtProgressBar(min = 0, max = 10, style = 3, file = tc)
  expect_equal(rust_txt_progress_bar_set(pb, 4), 4)
  expect_false(environment(pb$kill)$.killed)
  setTxtProgressBar(pb, 7)
  expect_equal(getTxtProgressBar(pb), 7)
  close(pb)
  expect_true(environment(pb$kill)$.killed)
})

test_that("close() on a borrowed RTxtProgressBar leaves the bar open (#1917)", {
  buf <- character(0)
  tc <- textConnection("buf", "w", local = TRUE)
  on.exit(close(tc))
  pb <- txtProgressBar(min = 0, max = 10, style = 3, file = tc)
  expect_true(rust_txt_progress_bar_close(pb))
  expect_false(environment(pb$kill)$.killed)
  close(pb)
})
