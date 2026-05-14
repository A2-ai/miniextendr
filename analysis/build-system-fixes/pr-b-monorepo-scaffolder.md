# PR-B: Monorepo scaffolder gaps (r_shim.h + upgrade + release-workflow)

**Context**: `analysis/build-system-investigation-2026-05-11.md` §2.4, §2.5, §12.4, §12.5.

## Problems

Three monorepo-related gaps in minirextendr:

### B1. `r_shim.h` missing from monorepo template

`use_miniextendr_stub()` (`minirextendr/R/use-rust.R:54-62`) copies
both `stub.c` and `r_shim.h`. But `create_rpkg_subdirectory()`
(`minirextendr/R/create.R:234-236`) — the monorepo's rpkg-subdir
creator — only copies `stub.c`. The monorepo template directory
`minirextendr/inst/templates/monorepo/rpkg/` does not contain
`r_shim.h` either.

Impact: monorepo packages that call `use_native_package()` (which writes
C shims including `r_shim.h`) hit a compile-time "file not found."

### B2. `upgrade_miniextendr_package()` is rpkg-only

`upgrade_miniextendr_package()` (`minirextendr/R/upgrade.R:27-95`) calls
`use_miniextendr_stub()`, `use_miniextendr_makevars()`, etc., all of
which use `usethis::proj_get()` to resolve paths. In a monorepo,
`proj_get()` is the workspace root, not the rpkg subdir, so upgrade
writes files at the wrong level.

### B3. `use_release_workflow()` not monorepo-aware

`use_release_workflow()` (`minirextendr/R/use-release-workflow.R:26-50`)
ships `r-release.yml` that runs `bash ./configure` and `R CMD build .`
at the repo root. In a monorepo the correct invocation is from within
the rpkg subdir.

## Files to change

### B1

- `minirextendr/inst/templates/monorepo/rpkg/r_shim.h` — **new file**.
  Copy the contents of `minirextendr/inst/templates/rpkg/r_shim.h`
  byte-for-byte (it's identical regardless of structure).
- `minirextendr/R/create.R:234-236` (`create_rpkg_subdirectory`) — add
  a `fs::file_copy` call for `r_shim.h` paralleling the existing
  `stub.c` copy.

### B2

- `minirextendr/R/upgrade.R` — add a `path` argument that defaults to
  `usethis::proj_get()`. Detect monorepo via `detect_project_type(path)`
  (already in `minirextendr/R/utils.R` per investigation §2). If
  monorepo, resolve to the rpkg subdir (read `miniextendr.yml` or pass
  `rpkg_subdir` argument). All subsequent `use_*` calls take this
  resolved path.
- Document the new `path` / `rpkg_subdir` argument in roxygen.
- Update tests (if any exist in `minirextendr/tests/testthat/test-upgrade*.R`).

### B3

- `minirextendr/R/use-release-workflow.R` — add `rpkg_subdir = NULL`
  argument. If `NULL`, auto-detect via `detect_project_type()`. If
  monorepo, set to default rpkg name from `miniextendr.yml` (or fail
  with a clear "pass `rpkg_subdir = '<name>'`" message).
- `minirextendr/inst/templates/r-release.yml` — parameterize the
  `working-directory` for `bash ./configure` and `R CMD build`
  steps. Use a mustache `{{rpkg_subdir}}` variable that defaults to
  empty (standalone case).
- The template currently does:
  ```yaml
  - run: bash ./configure
  - run: R CMD build .
  ```
  Change to:
  ```yaml
  - run: bash ./configure
    working-directory: {{rpkg_subdir}}
  - run: R CMD build .
    working-directory: {{rpkg_subdir}}
  ```
  For standalone case `{{rpkg_subdir}}` is empty and `working-directory:`
  with empty value is invalid YAML. Use `working-directory: ${{ inputs.rpkg_subdir || '.' }}`
  or render two variants. Simpler: add the keys only when `rpkg_subdir`
  is non-empty in the renderer.

  Pick whichever is cleanest in the existing `use_template` machinery.

## Tests / verification

### B1

1. `Rscript -e 'minirextendr::create_miniextendr_monorepo("/tmp/m_test_b1", rpkg_name="rpkg", crate_name="mycrate")'`
2. Assert `/tmp/m_test_b1/rpkg/src/r_shim.h` exists and is byte-identical
   to `minirextendr/inst/templates/rpkg/r_shim.h`.
3. From `/tmp/m_test_b1/rpkg/`: `bash ./configure && R CMD INSTALL .`
   should succeed (the install doesn't actually need `r_shim.h` unless
   native bindings are used; we just want to confirm the file is in place).

### B2

1. Scaffold a monorepo (per B1).
2. `Rscript -e 'minirextendr::upgrade_miniextendr_package(path="/tmp/m_test_b1")'`
   should write to `/tmp/m_test_b1/rpkg/src/` (NOT to `/tmp/m_test_b1/src/`).
3. Standalone case: `create_miniextendr_package("/tmp/s_test_b2")` +
   `upgrade_miniextendr_package(path="/tmp/s_test_b2")` should still write
   to `/tmp/s_test_b2/src/` as before.

### B3

1. Scaffold a monorepo.
2. `Rscript -e 'minirextendr::use_release_workflow(path="/tmp/m_test_b1")'`
   (or via auto-detect). Assert the generated
   `/tmp/m_test_b1/.github/workflows/r-release.yml` has
   `working-directory: rpkg` (or the user's rpkg_name) on the configure
   and build steps.
3. Standalone case: `use_release_workflow(path="/tmp/s_test")` produces
   a workflow with no `working-directory` (or with `.`) — unchanged
   from current behavior.

## Not in scope

- Tracking down every `use_*` helper that may also be monorepo-blind.
  The investigation only audited `upgrade_miniextendr_package` and
  `use_release_workflow`. Other helpers may need similar treatment —
  file follow-up issues if discovered.

## PR title

`fix(minirextendr): monorepo scaffolding gaps — r_shim.h, upgrade path, release workflow`

## PR body

Reference §2.4, §2.5 of investigation. Note that the three changes
are bundled because they share the same root cause (scaffolding
assumes single-structure layout) and the same affected files.

## Branch

`fix/minirextendr-monorepo-gaps`
