# Dependency investigation for idiomatic/expanded minirextendr

Date: 2025-12-30
Scope: evaluate the proposed dependencies for places they can replace manual code
or unlock more idiomatic workflows in minirextendr.

## Quick take

- Strong fits: gh, gert, rappdirs, whisker, lifecycle
- Optional: rprojroot, yaml, purrr, rstudioapi, clipr
- Likely unnecessary: crayon (cli already used), stats/tools/utils (base)

## Detailed notes by dependency

### cli (>= 3.0.1)
Already used throughout. Version bump is reasonable.

### clipr (>= 0.3.0)
Optional: could copy "next steps" commands or install instructions to clipboard.
No current code path clearly benefits; add only if you want this UX.

### crayon
Probably unnecessary because cli already handles styling and colors consistently.

### curl (>= 2.7)
Already used in `minirextendr/R/vendor.R` for tarball downloads.
Version pin is fine.

### desc (>= 1.4.2)
Already used in `minirextendr/R/use-r.R` and `minirextendr/R/status.R`.
Version pin is fine.

### fs (>= 1.3.0)
Already used across scaffolding and vendoring.

### gert (>= 1.4.1)
Good fit to make git operations idiomatic and consistent with usethis:
- `minirextendr/R/create.R:98-103` manually runs `git init`.
  Replace with `gert::git_init()` (or `usethis::use_git()` if you want
  .gitignore defaults and optional commit prompts).
- Could also use `gert::git_ls()` or `gert::git_status()` in future status checks.

### gh (>= 1.2.1)
Strong fit for GitHub API usage:
- `minirextendr/R/vendor.R:12-35` uses `jsonlite::fromJSON()` on the tags API.
  `gh::gh("/repos/{owner}/{repo}/tags")` gives pagination, auth, and error handling.
- Could add a `latest` helper via `gh::gh("/repos/{owner}/{repo}/releases/latest")`
  and use that for `vendor_miniextendr(version = "latest")`.
If you adopt gh, you can likely drop jsonlite for tag fetching (unless used elsewhere).

### glue (>= 1.3.0)
Already used. Version pin OK.

### jsonlite
Currently used for GitHub tag listing. If replaced with gh, this can become optional.

### lifecycle (>= 1.0.0)
Optional but idiomatic for public API evolution:
- Use `lifecycle::deprecate_warn()` for renamed functions.
- Add `@lifecycle` tags in docs for experimental helpers (e.g., vendoring helpers).

### purrr
Optional: could replace for-loops with `purrr::walk()` to simplify templating
and file operations, but not strictly necessary.

### rappdirs
Good fit for caching downloaded tarballs:
- `minirextendr/R/vendor.R:52-65` always downloads into temp.
  `rappdirs::user_cache_dir("minirextendr")` would enable persistent cache
  by version (reduces repeated downloads and GitHub rate-limit risk).

### rlang (>= 1.1.0)
Already used.

### rprojroot (>= 2.1.1)
Potential improvement to project detection:
- `minirextendr/R/utils.R:23-63` only checks current/parent dir for Cargo.toml/DESCRIPTION.
  `rprojroot::find_root(rprojroot::has_file("Cargo.toml"))` would let you run
  from nested subdirectories more reliably.

### rstudioapi
Optional: direct IDE integration (open project, open file) beyond usethis.
Given you already depend on usethis, this is a "nice to have."

### stats / tools / utils
Base packages. No need to add to Imports unless you want to drop `tools::` and
`utils::` namespace usage.

### whisker
Good fit for templates:
- `minirextendr/R/utils.R:152-179` implements a simple `gsub`-style templater.
  `whisker::whisker.render()` supports real mustache, matches usethis expectations,
  and handles escaping.
- Alternatively, delegate to `usethis::use_template()` and let it call whisker.

### withr (>= 2.3.0)
Already used.

### yaml
Optional expansion: add a `miniextendr.yml` config file so users can set
defaults (crate name, rpkg name, miniextendr version, features), then read
via `yaml::read_yaml()` to populate `template_data()`.

## Suggested "idiomatic" swaps tied to current code

- Git init: `minirextendr/R/create.R:98-103` -> `gert::git_init()` or `usethis::use_git()`.
- GitHub tags: `minirextendr/R/vendor.R:12-35` -> `gh::gh()` (droppable jsonlite).
- Templating: `minirextendr/R/utils.R:152-179` -> `whisker::whisker.render()` or `usethis::use_template()`.
- Download cache: `minirextendr/R/vendor.R:52-65` -> `rappdirs::user_cache_dir()`.
- Project detection: `minirextendr/R/utils.R:23-63` -> `rprojroot::find_root()`.

## Suggests list

The Suggests list you provided is reasonable for testing, docs, and reporting.
No code currently calls these packages directly, so they remain optional.
