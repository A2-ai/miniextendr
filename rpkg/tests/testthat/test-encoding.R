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

test_that("bytes-encoded strings are rejected as non-text", {
  value <- rawToChar(as.raw(0xff))
  Encoding(value) <- "bytes"
  expect_error(conv_string_arg(value), "marked as bytes")
  expect_error(miniextendr:::str_borrow_len(value), "marked as bytes")
  expect_error(zero_copy_cow_str_is_borrowed(value), "marked as bytes")
})

test_that("malformed strings tagged as UTF-8 are rejected before creating str", {
  value <- rawToChar(as.raw(0xff))
  Encoding(value) <- "UTF-8"
  expect_error(conv_string_arg(value), "marked as UTF-8 contains invalid UTF-8")
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
