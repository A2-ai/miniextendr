# usethis replacement review (minirextendr)

Date: 2025-12-30
Scope: identify places where usethis helpers can replace manual scaffolding in minirextendr.
Reference: local usethis 3.1.0 function list/signatures.

## Candidate replacements

| Location | Current behavior | usethis replacement | Notes |
| --- | --- | --- | --- |
| `minirextendr/R/create.R:133` | Hand-builds DESCRIPTION content for the monorepo rpkg subdir. | `usethis::use_description(fields = list(...))` | `use_description()` overwrites DESCRIPTION, so this is safe for new rpkg creation but not for existing packages. Use `withr::with_dir(rpkg_path, ...)` or a temporary `usethis::proj_set()` so it targets `rpkg/`. For extra tweaks (e.g., append to SystemRequirements), keep `desc` and optionally follow with `usethis::use_tidy_description()`. |
| `minirextendr/R/use-r.R:32` | Uses `desc` directly to set Config/* and SystemRequirements. | `usethis::use_description()` + `usethis::use_tidy_description()` | If you want usethis to own the formatting, you can rebuild DESCRIPTION with `use_description(fields = ...)` and/or run `use_tidy_description()` after `desc` updates. Keep `desc` if you need in-place edits without overwriting. |
| `minirextendr/R/use-r.R:69` | Manually reads template and appends to `.Rbuildignore`. | `usethis::use_build_ignore(template_lines, escape = FALSE)` | Use `withr::with_dir()` (or `proj_set`) for monorepo `rpkg/`. `use_build_ignore()` handles deduping and writing. |
| `minirextendr/R/use-r.R:100` | Manually reads template and appends to `.gitignore`. | `usethis::use_git_ignore(template_lines, directory = ".")` | For monorepo, pass `directory = rpkg_name` or run in that directory. If you want the `# miniextendr` header, include it as a line in `template_lines`. |
| `minirextendr/R/use-r.R:10` | Custom template for package doc (adds `@useDynLib`). | `usethis::use_package_doc()` + small patch | `use_package_doc()` creates the package doc but omits `@useDynLib`; you can insert that line after creation. If you prefer to keep your template, consider `usethis::use_template("rpkg/package.R", save_as = ..., data = ..., package = "minirextendr")`. |
| `minirextendr/R/utils.R:140` | Custom `use_template()` with manual mustache replacement. | `usethis::use_template()` | `usethis::use_template(template = file.path(get_template_type(), subdir, template), save_as = save_as, data = data, package = "minirextendr")` uses whisker templating and `write_over()`. This could replace the custom gsub logic and `template_path()`. |
| `minirextendr/R/utils.R:320` | Custom `ensure_dir()` wrapper around `fs::dir_create()`. | `usethis::use_directory()` | `use_directory("src/rust")` (and similar) keeps directory creation within usethis conventions and can optionally add to `.Rbuildignore`. |
| `minirextendr/R/create.R:98` | Manual `git init`. | `usethis::use_git()` | `use_git()` initializes git, adds default `.gitignore` entries, and may prompt for an initial commit (and RStudio restart). Keep manual init if you want to avoid those side effects. |
| `minirextendr/R/create.R:84` and `minirextendr/R/create.R:166` | Writes `.gitignore` / `.Rbuildignore` from templates. | `usethis::use_git_ignore()` / `usethis::use_build_ignore()` | If you want incremental updates instead of replacing files, feed template lines into these helpers. If you still want full-template rendering, `usethis::use_template()` can take over the copy. |

## Dependency suggestions (from request, not applied)

Imports:
    cli (>= 3.0.1),
    clipr (>= 0.3.0),
    crayon,
    curl (>= 2.7),
    desc (>= 1.4.2),
    fs (>= 1.3.0),
    gert (>= 1.4.1),
    gh (>= 1.2.1),
    glue (>= 1.3.0),
    jsonlite,
    lifecycle (>= 1.0.0),
    purrr,
    rappdirs,
    rlang (>= 1.1.0),
    rprojroot (>= 2.1.1),
    rstudioapi,
    stats,
    tools,
    utils,
    whisker,
    withr (>= 2.3.0),
    yaml

Suggests:
    covr,
    knitr,
    magick,
    pkgload (>= 1.3.2.1),
    quarto (>= 1.5.1),
    rmarkdown,
    roxygen2 (>= 7.1.2),
    spelling (>= 1.2),
    testthat (>= 3.1.8)
