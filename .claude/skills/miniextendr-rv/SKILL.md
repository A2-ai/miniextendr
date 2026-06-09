---
name: miniextendr-rv
description: Use when R/test setup misbehaves (e.g. `testthat` is "not installed" mid-test, `__rv_R_mismatch` temp library appears, `just devtools-test` skips tests silently, `rv sync` complains, or you need to bump the pinned R version). Also use when adding a new R dependency to the dev workflow, understanding why `library()` works in one shell but not another, or coaching contributors through their first `rig` / `rv` setup for this repo.
---

# miniextendr R version + dependency management (rv + rig)

This repo pins its R version and dev-dependency set in `rproject.toml` at the
repo root and uses **rv** to materialize them into `rv/library/<r-version>/<arch>/`.
Switching the R on `PATH` is **rig**'s job. Mismatching the two puts rv into safe
mode and quietly breaks every test recipe.

## When to use this skill

- "`testthat` not installed" or "no package called 'X'" appears mid-test even though deps look correct.
- Logs contain `WARNING: R version specified in config (4.X) does not match session version (4.Y)` or `creating temporary library: …/__rv_R_mismatch`.
- `just devtools-test` exits 0 but no test summary appears in the log.
- `rv sync` complains about missing or incompatible packages.
- A contributor's `R --version` doesn't match `rproject.toml`'s `r_version`.
- You want to bump the pinned R version (e.g. 4.5 → 4.6 after R-devel ships a release).
- You're adding a new dev/test dependency and need it picked up by everyone.

## Layout

```
rproject.toml                # Pin: r_version, repositories, dependencies
rv/
  library/<r-version>/<arch>/   # rv-installed packages (gitignored)
  scripts/
    activate.R               # sourced via .Rprofile-style mechanism
    rvr.R                    # rv R-side helpers
  .gitignore
```

`rproject.toml` fields that matter here:

```toml
[project]
name = "miniextendr"
r_version = "4.6"          # MUST match `R --version` on PATH

repositories = [
    {alias = "CRAN", url = "https://cloud.r-project.org/"},
]

dependencies = [
    {name = "minirextendr", dependencies_only = true, path = "minirextendr"},
    {name = "miniextendr",  dependencies_only = true, path = "rpkg"},
    "testthat",
    "devtools",
    {name = "roxygen2", url = "https://cran.r-project.org/src/contrib/roxygen2_8.0.0.tar.gz"},
    # …
]
```

`dependencies_only = true` on the local packages means rv installs *their*
deps but not the local package itself (we install that via
`just rcmdinstall` / `just minirextendr-install`).

The pinned `roxygen2_8.0.0.tar.gz` URL is intentional — current code targets
roxygen2 8.0.0; the CRAN release moved on and would re-render man pages
differently. The pin is the source of truth in `rproject.toml`, mirrored by
`Config/roxygen2/version: 8.0.0` in `rpkg/DESCRIPTION`.

## The R-version contract (the only thing that goes wrong here)

`r_version` in `rproject.toml` must equal `R --version` on `PATH`. If they
diverge, rv aborts activation and creates a temporary stub library at
`/var/folders/.../__rv_R_mismatch`. R then runs against that empty library
and every `library(testthat)` / `library(devtools)` fails with
`no package called …`.

The error you'll see in `just devtools-test` output:

```
WARNING: R version specified in config (4.5) does not match session version (4.6.0).
rv library will not be activated until the issue is resolved. Entering safe mode...

creating temporary library: /var/folders/_x/…/__rv_R_mismatch

Error in loadNamespace(x) : there is no package called 'testthat'
Calls: loadNamespace -> withRestarts -> withOneRestart -> doWithOneRestart
Execution halted
```

**The `just` recipe still exits 0** even though no tests ran. CI catches this
because CI installs R deliberately, but locally it's easy to miss.

### How to fix

Pick whichever side reflects reality:

**Option A — switch R to match `rproject.toml`** (most common):

```bash
rig default 4.6        # or `rig default 4.6-arm64` on macOS arm64
R --version | head -1  # verify
just devtools-test     # should now show "[ FAIL 0 | WARN N | SKIP M | PASS K ]"
```

**Option B — bump `rproject.toml` to match a newer R you've adopted**:

```bash
# 1. Edit rproject.toml
sed -i '' 's/r_version = "4.5"/r_version = "4.6"/' rproject.toml
# 2. Sync the library against the new R
rv sync
# 3. Re-run tests
just devtools-test
```

If you choose B, mirror the change in the **R version (rv + rig)** section of
the root `CLAUDE.md` so future contributors know the current pin.

## rig cheat-sheet (R installer)

`rig` (the **R installation manager**, not to be confused with rv) installs
and switches between R versions. Not in this repo — it's a global tool.

```bash
rig list                            # show installed versions, marks default
rig default 4.6-arm64               # set system default R
rig add 4.6                         # install R 4.6 (arch auto-detected)
rig add 4.5                         # install another version side-by-side
rig system create-lib               # idempotent: ensure user lib exists
```

The `*` next to a `rig list` row indicates the current default. Subshells
and CI use that R unless they override `PATH`.

## rv cheat-sheet (R-side venv-equivalent)

`rv` is the project-local dependency manager.

```bash
rv info --library --r-version --repositories  # what activate.R inspects
rv sync                                        # install / update everything in rproject.toml
rv add <pkg>                                   # add a dep + persist to rproject.toml
rv add --no-sync <pkg>                         # write to rproject.toml only
rv remove <pkg>                                # opposite of add
rv status                                      # show drift between toml and library
```

The justfile has wrapper recipes for the **CRAN corpus** workflow (bulk add/remove
of a curated list of CRAN packages used for adapter-tests); for everyday dev
deps just edit `rproject.toml` and run `rv sync`.

## Adding a new dependency for tests

1. Edit `rproject.toml`:
   ```toml
   dependencies = [
       # …existing…
       "new-package",
   ]
   ```
2. `rv sync` — installs `new-package` and its deps into `rv/library/...`.
3. Commit `rproject.toml`. The library directory is gitignored; CI's `rv sync`
   step will recreate it.

If the new package needs a non-CRAN source (pinned URL, git ref, local path),
the toml supports the same syntax as the existing `roxygen2`:

```toml
{name = "roxygen2", url = "https://cran.r-project.org/src/contrib/roxygen2_8.0.0.tar.gz"},
{name = "minirextendr", dependencies_only = true, path = "minirextendr"},
{name = "somelib", git = "https://github.com/org/somelib.git", tag = "v1.2.3"},
```

## Bumping the pinned R version

Trigger: a new R release lands and we want to track it.

1. `rig add <new-version>` if you don't have it locally.
2. `rig default <new-version>` to switch your session.
3. Edit `rproject.toml` → `r_version = "<new-version>"`.
4. Update the **R version (rv + rig)** section in the root `CLAUDE.md` to
   match.
5. `rv sync` — rebuilds the library under the new R version.
6. `just devtools-test` — confirms the new env is usable end-to-end.
7. If you bump R minor versions on CI workflows (e.g. `r-lib/actions/setup-r`
   with explicit version), update those YAMLs too. The release workflow
   template (`minirextendr/inst/templates/r-release.yml`) currently pins
   `r-version: release` so it auto-tracks; check anyway.

Commit `rproject.toml` and `CLAUDE.md` together. The library dir stays
gitignored.

## Doctor checks for this skill

- `R --version | head -1` matches `r_version` in `rproject.toml`.
- `rig list` shows the matching version marked `*`.
- `which rv` resolves; `rv info` doesn't error.
- `rv sync` exits clean (no missing deps, no version conflicts).
- `R -e 'find.package("testthat")'` resolves under `rv/library/<r>/<arch>/`,
  NOT a path containing `__rv_R_mismatch`.

If any of these fail, you're in safe mode — see the fix recipes above.

## What this skill does NOT cover

- Installing `rv` itself or `rig` itself (both are global / system installs;
  see their upstream docs).
- CRAN corpus management (`just bindgen-corpus-add-packages` / `just bindgen-corpus-remove-packages`)
  — see comments in `justfile`.
- `renv` migration — this repo is rv-native and has no `renv.lock`.

## Cross-references

- Root `CLAUDE.md` § "R version (rv + rig)" — the abbreviated version of this
  doc, intended for first-touch contributors.
- `rproject.toml` (the `roxygen2` dependency entry) and
  `rpkg/DESCRIPTION` (`Config/roxygen2/version`) — the source of truth for the
  pinned `roxygen2_8.0.0.tar.gz` URL.
- `[[miniextendr-build]]` skill — what to do *after* you've sorted R/rv.
