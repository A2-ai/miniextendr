# String conversion in any locale.
#
# A miniextendr package loads in any locale. Each conversion reads the
# string's encoding mark: UTF-8 and ASCII are borrowed, Latin-1 is translated,
# bytes are refused, and a native (unmarked) string is borrowed in a UTF-8
# session and translated from the session's encoding otherwise. R decides
# per call (`Rf_translateCharUTF8` reads the current locale), so a
# `Sys.setlocale()` mid-session is followed. The first native translation in a
# session raises one `miniextendr_native_translation` warning.
#
# These tests cover:
#   1. Per-string encoding marks in the (UTF-8) test session.
#   2. Native strings after a mid-session `Sys.setlocale("LC_CTYPE", "C")`.
#   3. Package load and conversions in a fresh R under `LC_ALL=C` / `POSIX`
#      (`callr::r()`), where the warning's once-per-session count is visible.
#
# R's docs warn that changing `LC_CTYPE` mid-session may not take effect on
# every platform. Each mid-session test guards with the actual `l10n_info()`
# return so the test only runs when the platform actually flipped the locale.

# region: cached snapshot

test_that("encoding_info_available returns logical", {
  result <- encoding_info_available()
  expect_type(result, "logical")
  # Standard R package builds: encoding_init is disabled (symbols not
  # exported from libR), so this is FALSE. Embedded / nonapi builds may
  # populate the snapshot.
})

# endregion

# region: per-string encoding tags

latin1_facade <- function() {
  value <- rawToChar(as.raw(c(0x66, 0x61, 0xe7, 0x61, 0x64, 0x65)))
  Encoding(value) <- "latin1"
  value
}

test_that("owned String conversion translates Latin-1 in a UTF-8 session", {
  value <- latin1_facade()
  expect_identical(Encoding(value), "latin1")
  expect_identical(conv_string_arg(value), enc2utf8(value))
})

test_that("borrowed str conversion translates Latin-1 in a UTF-8 session", {
  value <- latin1_facade()
  expect_equal(miniextendr:::str_borrow_len(value), 6L)
})

test_that("scalar Cow conversion borrows translated text", {
  value <- latin1_facade()
  expect_true(zero_copy_cow_str_is_borrowed("façade"))
  expect_true(zero_copy_cow_str_is_borrowed(value))
})

test_that("vector Cow strings own elements that require translation", {
  value <- latin1_facade()
  expect_false(zero_copy_vec_cow_str_all_borrowed(c("ascii", value)))
})

test_that("native UTF-8 and ASCII strings remain borrowed in a UTF-8 session", {
  skip_if_not(isTRUE(l10n_info()[["UTF-8"]]), "test session is not UTF-8")
  value <- enc2utf8("façade")
  Encoding(value) <- "unknown"
  expect_identical(Encoding(value), "unknown")
  expect_identical(conv_string_arg(value), enc2utf8(value))
  expect_equal(miniextendr:::str_borrow_len(value), 6L)
  expect_true(zero_copy_cow_str_is_borrowed(value))
  expect_true(zero_copy_cow_str_is_borrowed("ascii"))
})

bytes_value <- function(x) {
  Encoding(x) <- "bytes"
  x
}

# The argument error of a refused string: the conversion's own condition
# (classes, `e$param`, the wrapper's message), not a panic with a location.
expect_string_refusal <- function(expr, message, param = "x") {
  e <- tryCatch(expr, error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, param)
  expect_identical(conditionMessage(e), message)
  expect_false(grepl("from_r", conditionMessage(e), fixed = TRUE))
}

test_that("a scalar string marked as bytes is an argument error", {
  value <- bytes_value("µg")
  expect_identical(Encoding(value), "bytes")
  scalar <- "'x' must be a single string: marked as bytes, not text"
  expect_string_refusal(miniextendr:::encoding_echo_string(value), scalar)
  expect_string_refusal(miniextendr:::encoding_echo_str(value), scalar)
  expect_string_refusal(miniextendr:::encoding_echo_cow(value), scalar)
  expect_string_refusal(
    miniextendr:::encoding_echo_option_string(value),
    "'x' must be NULL or a single string: marked as bytes, not text"
  )
  # The rejection is the error a wrong type gets, with the same classes.
  expect_identical(
    class(tryCatch(miniextendr:::encoding_echo_string(value), error = identity)),
    class(tryCatch(miniextendr:::encoding_echo_string(1), error = identity))
  )
  # The pre-existing fixtures refuse it the same way.
  expect_string_refusal(conv_string_arg(value), scalar)
  expect_string_refusal(
    miniextendr:::str_borrow_len(value),
    "'s' must be a single string: marked as bytes, not text",
    param = "s"
  )
})

test_that("every element marked as bytes is named in one argument error", {
  value <- bytes_value(c("a", "µg", NA, "ñ"))
  expect_identical(Encoding(value), c("unknown", "bytes", "unknown", "bytes"))
  vector <- "'x' must be character: marked as bytes, not text (elements 2, 4)"
  expect_string_refusal(miniextendr:::encoding_echo_vec_string(value), vector)
  expect_string_refusal(miniextendr:::encoding_echo_vec_option_string(value), vector)
  expect_string_refusal(miniextendr:::encoding_echo_vec_str(value), vector)

  one <- bytes_value(c("a", "µg"))
  expect_string_refusal(
    miniextendr:::encoding_echo_vec_string(one),
    "'x' must be character: marked as bytes, not text (element 2)"
  )
})

test_that("NA keeps its handling next to the refusal", {
  expect_identical(
    miniextendr:::encoding_echo_vec_option_string(c("a", NA, "µg")),
    c("a", NA, "µg")
  )
  expect_identical(miniextendr:::encoding_echo_vec_string(c("a", NA)), c("a", ""))
  expect_identical(miniextendr:::encoding_echo_option_string(NA_character_), NA_character_)
})

test_that("a string that is not valid UTF-8 is an argument error", {
  value <- rawToChar(as.raw(0xff))
  Encoding(value) <- "UTF-8"
  expect_string_refusal(
    conv_string_arg(value),
    "'x' must be a single string: not valid UTF-8"
  )
})

test_that("a native string that is not valid UTF-8 is an argument error in a UTF-8 session", {
  skip_if_not(isTRUE(l10n_info()[["UTF-8"]]), "test session is not UTF-8")
  # In a UTF-8 session R reads unmarked (native) bytes untranslated, so they
  # are validated as UTF-8.
  native <- rawToChar(as.raw(c(0x61, 0xff)))
  expect_identical(Encoding(native), "unknown")
  expect_string_refusal(
    miniextendr:::encoding_echo_vec_string(c("ok", native)),
    "'x' must be character: not valid UTF-8 (element 2)"
  )
})

test_that("string_elt_str refuses an element marked as bytes with its position", {
  expect_error(
    miniextendr:::encoding_string_elt_str(bytes_value(c("a", "µg"))),
    "element 2 of a character vector as Rust text: marked as bytes, not text",
    fixed = TRUE
  )
})

test_that("a key marked as bytes is a group_by error naming every element", {
  df <- data.frame(g = bytes_value(c("a", "µg", "ñ")))
  expect_identical(Encoding(df$g), c("unknown", "bytes", "bytes"))
  expect_error(
    group_by_keys(df, "g"),
    "cannot group by column 'g': marked as bytes, not text (elements 2, 3)",
    fixed = TRUE
  )
})

test_that("a grouping variable name marked as bytes is a group metadata error", {
  df <- data.frame(g = c("a", "b"), v = c(1, 2))
  groups <- data.frame(g = c("a", "b"))
  groups[[".rows"]] <- list(1L, 2L)
  names(groups) <- c(bytes_value("µg"), ".rows")
  attr(df, "groups") <- groups
  class(df) <- c("grouped_df", "tbl_df", "tbl", "data.frame")
  msg <- "cannot read the grouping variable names: marked as bytes, not text (element 1)"
  expect_error(group_declaration_of(df), msg, fixed = TRUE)
  expect_error(group_metadata_keys(df), msg, fixed = TRUE)
})

# endregion

# region: Latin-1 names and string reads

latin1_name <- function() iconv("DéV", "UTF-8", "latin1")

test_that("DataFrame::names() translates a Latin-1 name", {
  d <- data.frame(a = 1, b = 2)
  names(d) <- c("a", latin1_name())
  expect_identical(Encoding(names(d)), c("unknown", "latin1"))
  expect_identical(miniextendr:::encoding_df_names(d), c("a", "DéV"))
})

test_that("a Latin-1 name and a Latin-1 argument find each other", {
  d <- data.frame(a = 1)
  names(d) <- latin1_name()
  expect_true(miniextendr:::encoding_df_has_column(d, latin1_name()))
  expect_true(miniextendr:::encoding_df_has_column(d, "DéV"))
  expect_false(miniextendr:::encoding_df_has_column(d, "DeV"))
})

test_that("string_elt_str translates a Latin-1 element and keeps NA", {
  x <- c("plain", latin1_name(), NA)
  expect_identical(Encoding(x)[[2]], "latin1")
  expect_identical(miniextendr:::encoding_string_elt_str(x), c("plain", "DéV", NA))
})

test_that("a Latin-1 String argument still translates", {
  expect_identical(miniextendr:::encoding_echo_string(latin1_name()), "DéV")
  expect_identical(
    miniextendr:::encoding_echo_vec_string(c("a", latin1_name())),
    c("a", "DéV")
  )
})

test_that("group_by reads a Latin-1 key with its encoding", {
  df <- data.frame(g = c(latin1_name(), "a", latin1_name()))
  expect_identical(group_by_keys(df, "g"), c("DéV", "a"))
})

test_that("group metadata reads a Latin-1 grouping variable name", {
  df <- data.frame(g = c("a", "b"), v = c(1, 2))
  groups <- data.frame(g = c("a", "b"))
  groups[[".rows"]] <- list(1L, 2L)
  names(groups) <- c(latin1_name(), ".rows")
  attr(df, "groups") <- groups
  class(df) <- c("grouped_df", "tbl_df", "tbl", "data.frame")
  expect_identical(group_declaration_of(df), list(vars = "DéV", drop = TRUE))
})

test_that("Latin-1 string reads survive allocation", {
  expect_identical(
    miniextendr:::gc_stress_latin1_string_reads(),
    paste0("DéV", 0:5)
  )
})

test_that("Latin-1 string reads survive allocation under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  result <- miniextendr:::gc_stress_latin1_string_reads()
  gctorture(FALSE)
  expect_identical(result, paste0("DéV", 0:5))
})

# endregion

# region: native strings after a mid-session locale flip

# Helper: temporarily switch LC_CTYPE, restoring on exit. Returns the value
# Sys.setlocale() actually returned ("" on failure) so callers can skip when
# the platform refused the requested locale.
with_lc_ctype <- function(loc, code) {
  original <- Sys.getlocale("LC_CTYPE")
  applied <- suppressWarnings(Sys.setlocale("LC_CTYPE", loc))
  on.exit(suppressWarnings(Sys.setlocale("LC_CTYPE", original)), add = TRUE)
  list(applied = applied, value = if (nzchar(applied)) force(code) else NULL)
}

# The warning fires once per session, so whether this session already raised
# it depends on test order: muffle it here; the subprocess tests count it.
muffle_translation <- function(code) {
  withCallingHandlers(
    code,
    miniextendr_native_translation = function(w) invokeRestart("muffleWarning")
  )
}

test_that("native strings are translated after Sys.setlocale to C", {
  res <- with_lc_ctype("C", {
    # Some platforms keep `l10n_info()[['UTF-8']]` TRUE even after
    # `LC_CTYPE=C`. Only check translation when l10n_info actually flipped.
    if (isTRUE(l10n_info()[["UTF-8"]])) {
      skip("Platform did not flip l10n_info()[['UTF-8']] under LC_CTYPE=C")
    }
    native <- rawToChar(as.raw(c(0x66, 0x61, 0xe7, 0x61, 0x64, 0x65)))
    expect_identical(Encoding(native), "unknown")
    expect_identical(muffle_translation(conv_string_arg(native)), "fa<e7>ade")
    # A `Vec<Cow<str>>` element owns its translation.
    expect_false(muffle_translation(zero_copy_vec_cow_str_all_borrowed(c("ascii", native))))
    # ASCII and UTF-8-marked strings are still borrowed.
    expect_true(zero_copy_cow_str_is_borrowed("ascii"))
    expect_true(zero_copy_cow_str_is_borrowed(enc2utf8("fa\u00e7ade")))
  })
  if (!nzchar(res$applied)) {
    skip("Platform refused Sys.setlocale('LC_CTYPE', 'C')")
  }
})

test_that("native UTF-8 strings are borrowed again after restoring UTF-8", {
  original <- Sys.getlocale("LC_CTYPE")
  on.exit(suppressWarnings(Sys.setlocale("LC_CTYPE", original)), add = TRUE)

  flipped <- suppressWarnings(Sys.setlocale("LC_CTYPE", "C"))
  if (!nzchar(flipped) || isTRUE(l10n_info()[["UTF-8"]])) {
    skip("Could not flip locale away from UTF-8 to test restore")
  }
  utf8 <- ""
  for (cand in c("en_US.UTF-8", "C.UTF-8", "en_GB.UTF-8")) {
    utf8 <- suppressWarnings(Sys.setlocale("LC_CTYPE", cand))
    if (nzchar(utf8)) break
  }
  if (!nzchar(utf8) || !isTRUE(l10n_info()[["UTF-8"]])) {
    skip("No UTF-8 locale available to restore")
  }
  value <- enc2utf8("fa\u00e7ade")
  Encoding(value) <- "unknown"
  expect_true(zero_copy_cow_str_is_borrowed(value))
  expect_identical(conv_string_arg(value), enc2utf8(value))
})

# endregion

# region: package load under explicit locale (subprocess)

# These tests spawn a fresh R via callr::r() with the requested locale env
# vars: load the package there and convert strings in a session that starts
# in that locale, where the once-per-session warning count is observable.

skip_on_os("windows")  # Windows locale names + processx orphan handling are
                      # both finicky; the conversion is platform-independent
                      # in implementation, so Linux + macOS cover the
                      # contract.

run_load_in_locale <- function(env) {
  callr::r(
    function() {
      tryCatch(
        {
          library(miniextendr)
          # Cheap call — exercises a real .Call to confirm load succeeded.
          list(ok = TRUE, utf8 = isTRUE(l10n_info()[["UTF-8"]]))
        },
        error = function(e) list(ok = FALSE, message = conditionMessage(e))
      )
    },
    env = env,
    show = FALSE
  )
}

# Helper: pick a locale from candidates that the spawned subprocess actually
# accepts. Returns "" if none work.
probe_locale <- function(var, candidates) {
  for (cand in candidates) {
    env <- c(callr::rcmd_safe_env(), setNames(cand, var))
    res <- tryCatch(
      callr::r(
        function() Sys.getlocale("LC_CTYPE"),
        env = env,
        show = FALSE
      ),
      error = function(e) ""
    )
    if (nzchar(res) && !identical(res, "C") && !identical(res, "POSIX")) {
      # Subprocess successfully picked up the locale.
      return(cand)
    }
  }
  ""
}

test_that("library(miniextendr) succeeds under LC_ALL=C.UTF-8", {
  cand <- probe_locale("LC_ALL", c("C.UTF-8", "en_US.UTF-8"))
  if (!nzchar(cand)) skip("No UTF-8 locale available in subprocess")
  env <- c(callr::rcmd_safe_env(), LC_ALL = cand)
  res <- run_load_in_locale(env)
  expect_true(res$ok, info = res$message %||% "")
  expect_true(res$utf8)
})

test_that("library(miniextendr) succeeds under LC_CTYPE=en_US.UTF-8 with LANG=C", {
  cand <- probe_locale("LC_CTYPE", c("en_US.UTF-8", "C.UTF-8"))
  if (!nzchar(cand)) skip("No UTF-8 LC_CTYPE candidate available")
  env <- c(callr::rcmd_safe_env(), LANG = "C", LC_CTYPE = cand)
  res <- run_load_in_locale(env)
  expect_true(res$ok, info = res$message %||% "")
  expect_true(res$utf8)
})

# Convert ASCII, UTF-8-marked, Latin-1-marked and native non-ASCII strings in
# a fresh session, recording the classes of every warning raised.
run_strings_in_locale <- function(env) {
  callr::r(
    function() {
      library(miniextendr)
      native <- rawToChar(as.raw(c(0x66, 0x61, 0xe7, 0x61, 0x64, 0x65)))
      latin1 <- native
      Encoding(latin1) <- "latin1"
      warnings <- list()
      out <- withCallingHandlers(
        list(
          utf8_session = isTRUE(l10n_info()[["UTF-8"]]),
          ascii = conv_string_arg("ascii"),
          utf8 = conv_string_arg("fa\u00e7ade"),
          latin1 = conv_string_arg(latin1),
          native = conv_string_arg(native),
          native_again = conv_string_arg(native),
          native_vec_cow_borrowed = zero_copy_vec_cow_str_all_borrowed(c("ascii", native))
        ),
        warning = function(w) {
          warnings[[length(warnings) + 1L]] <<- class(w)
          invokeRestart("muffleWarning")
        }
      )
      out$warnings <- warnings
      out
    },
    env = env,
    show = FALSE
  )
}

# Whether a fresh R under `env` reports a UTF-8 session (NA if it failed).
subprocess_utf8 <- function(env) {
  tryCatch(
    callr::r(function() isTRUE(l10n_info()[["UTF-8"]]), env = env, show = FALSE),
    error = function(e) NA
  )
}

for (loc in c("C", "POSIX")) {
  test_that(paste0("strings convert per string under LC_ALL=", loc, ", warning once"), {
    env <- c(callr::rcmd_safe_env(), LC_ALL = loc)
    # Some glibc builds default to C.UTF-8 even when asked for C.
    if (!isFALSE(subprocess_utf8(env))) {
      skip(paste0("Subprocess is not a non-UTF-8 session under LC_ALL=", loc))
    }

    res <- run_strings_in_locale(env)
    expect_false(res$utf8_session)
    expect_identical(res$ascii, "ascii")
    expect_identical(res$utf8, "fa\u00e7ade")
    expect_identical(res$latin1, "fa\u00e7ade")
    # The session's encoding is ASCII: 0xE7 has no character in it.
    expect_identical(res$native, "fa<e7>ade")
    expect_identical(res$native_again, "fa<e7>ade")
    expect_false(res$native_vec_cow_borrowed)
    # Three native translations, one warning.
    expect_length(res$warnings, 1L)
    expect_true("miniextendr_native_translation" %in% res$warnings[[1]])
  })
}

# endregion
