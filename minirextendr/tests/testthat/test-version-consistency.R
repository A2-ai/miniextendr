# Mirror of CI's version-check job: Cargo.toml [workspace.package] version must
# equal rpkg/DESCRIPTION Version (X.Y.Z base, ignoring R dev suffixes like .9000).
# Skipped when run outside the monorepo (e.g. installed-package context).
# Closes #914.

test_that("Cargo.toml workspace version matches rpkg/DESCRIPTION Version", {
  skip_if_no_local_repo()

  repo <- find_miniextendr_repo()

  # --- Read Cargo.toml workspace.package version ----------------------------
  cargo_toml <- file.path(repo, "Cargo.toml")
  expect_true(file.exists(cargo_toml), info = "Cargo.toml not found at repo root")

  lines <- readLines(cargo_toml)

  # Find the [workspace.package] section and extract its version = "..." line.
  # Replicate CI: sed -n '/\[workspace\.package\]/,/^\[/{ s/^version = "(.*)"/\1/p; }'
  in_section <- FALSE
  cargo_version <- NULL
  for (line in lines) {
    if (grepl("^\\[workspace\\.package\\]", line)) {
      in_section <- TRUE
      next
    }
    if (in_section && grepl("^\\[", line)) {
      break  # reached next section
    }
    if (in_section) {
      m <- regmatches(line, regexpr('^version = "([^"]+)"', line, perl = TRUE))
      if (length(m) == 1L && nzchar(m)) {
        cargo_version <- sub('^version = "([^"]+)"', "\\1", m, perl = TRUE)
        break
      }
    }
  }
  expect_false(is.null(cargo_version),
               info = "Could not find version in [workspace.package] section of Cargo.toml")

  # --- Read rpkg/DESCRIPTION Version ----------------------------------------
  desc_path <- file.path(repo, "rpkg", "DESCRIPTION")
  expect_true(file.exists(desc_path), info = "rpkg/DESCRIPTION not found")

  desc <- read.dcf(desc_path, fields = "Version")
  r_version <- as.character(desc[1, "Version"])
  expect_false(is.na(r_version), info = "Version field missing from rpkg/DESCRIPTION")

  # --- Compare base X.Y.Z (allow R dev suffix X.Y.Z.9000 to match X.Y.Z) ---
  strip_base <- function(v) {
    m <- regmatches(v, regexpr("^[0-9]+\\.[0-9]+\\.[0-9]+", v, perl = TRUE))
    if (length(m) == 0L || !nzchar(m)) NA_character_ else m
  }

  cargo_base <- strip_base(cargo_version)
  r_base <- strip_base(r_version)

  expect_false(is.na(cargo_base),
               info = sprintf("Cargo.toml version '%s' is not X.Y.Z format", cargo_version))
  expect_false(is.na(r_base),
               info = sprintf("DESCRIPTION Version '%s' is not X.Y.Z format", r_version))

  expect_equal(
    cargo_base, r_base,
    info = sprintf(
      "Version mismatch: Cargo.toml [workspace.package] = %s, rpkg/DESCRIPTION = %s.\nRun: ./scripts/bump-version.sh %s",
      cargo_version, r_version, cargo_version
    )
  )
})
