+++
title = "minirextendr R CMD check cleanup"
description = "Fix three pre-existing R CMD check lints in minirextendr: non-ASCII characters, undeclared test dependencies, and unqualified head() calls."
+++

# minirextendr R CMD check cleanup

Surfaced during audit of PR #349; unrelated to that PR — own branch off `origin/main`.

## Items (flat priority order)

### 1. Non-ASCII characters in R sources (WARNING)

`R CMD check` warns on non-ASCII bytes in R source files. Three files affected:

- `minirextendr/R/git-hooks.R` — em-dashes (`—`, `<e2><80><94>`) and box-drawing chars (`─`, `<e2><94><80>`)
- `minirextendr/R/use-native.R` — em-dashes (`—`) and right-arrows (`→`, `<e2><86><92>`)
- `minirextendr/R/vendor-lib.R` — em-dash (`—`)

**Fix**: replace with ASCII equivalents in comments and strings:
- em-dash (`—`) → ` -- ` (double-hyphen with spaces)
- right-arrow (`→`) → `->`
- box-drawing `──` → `--`

Validate with `Rscript -e 'tools::showNonASCIIfile("minirextendr/R/git-hooks.R")'` etc. — must show no output.

### 2. Undeclared test dependencies (WARNING)

`R CMD check` warns: "package 'desc' used but not declared" and "package 'rprojroot' used but not declared" (both are `Suggests`-level: used in tests but not in runtime code).

**Fix**: add both to `Suggests` in `minirextendr/DESCRIPTION`:
```
Suggests:
    desc,
    devtools,
    knitr,
    rcmdcheck,
    rmarkdown,
    roxygen2,
    rprojroot,
    testthat (>= 3.0.0),
    withr,
    yaml
```

### 3. Undefined global function `head` (NOTE)

`R CMD check --as-cran` NOTE: "no visible global function definition for 'head'" — fired when `head` is called without namespace qualification in three locations:

- `use-native.R:275` — inside `detect_header_mode`
- `use-native.R:600` — inside `write_wrapper_header_to`
- `use-native.R:720` — inside `invoke_bindgen` (run_bindgen)

**Fix**: qualify each as `utils::head(...)`.

Rationale: `utils` is a base package always available; `@importFrom utils head` also works but adds NAMESPACE churn. Prefer explicit qualification per project conventions.

## Validation

```bash
just minirextendr-test 2>&1 > /tmp/minirextendr-test.log   # 0 FAIL
just minirextendr-check 2>&1 > /tmp/minirextendr-check.log  # Status: OK
```
