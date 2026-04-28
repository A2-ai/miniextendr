+++
title = "Vendoring: full lifecycle review + test plan"
description = "Exhaustive review and test plan for the entire vendoring system: cargo-revendor, configure.ac, bootstrap.R, Makevars.in, templates, minirextendr, CI, and every user/install pathway"
+++

# Vendoring — full lifecycle review + test plan

Execute on a fresh checkout of `origin/main`. All commands from repo root unless noted.

---

## Scope

Every component that touches vendoring:

| Layer | Files |
|-------|-------|
| cargo-revendor binary | `cargo-revendor/src/{main,cache,metadata,package,strip,vendor,verify,manifest_guard}.rs` + `tests/` |
| R build system | `rpkg/configure.ac`, `rpkg/configure`, `rpkg/src/Makevars.in`, `rpkg/src/Makevars`, `rpkg/src/rust/.cargo/config.toml` |
| bootstrap | `rpkg/bootstrap.R`, `tests/model_project/bootstrap.R`, `minirextendr/inst/templates/rpkg/bootstrap.R` |
| templates | `minirextendr/inst/templates/rpkg/configure.ac`, `Makevars.in`, `Cargo.toml.tmpl`, `justfile`, all cleanup scripts |
| minirextendr R | `workflow.R` (`miniextendr_vendor`), `vendor.R` (`vendor_crates_io`), `vendor-lib.R`, `status.R` (validate/status), `doctor.R`, `upgrade.R`, `create.R` |
| justfile | `vendor`, `r-cmd-build`, `r-cmd-check`, `devtools-build`, `configure`, `vendor-sync-check`, `lock-shape-check` |
| hooks | `.githooks/pre-commit` |
| CI | `.github/workflows/ci.yml` (vendor cache, r-cmd-check job) |

---

## Part 1 — Code review

### R1. The single signal — no legacy env vars anywhere

```bash
grep -rn "NOT_CRAN\|FORCE_VENDOR\|PREPARE_CRAN\|BUILD_CONTEXT" \
  rpkg/ minirextendr/R/ minirextendr/inst/templates/ .github/
```
Expected: zero matches (except comments explaining they were removed).

```bash
grep -rn "vendor_miniextendr\|cache.info\|cache-info" minirextendr/R/
```
Expected: zero matches — legacy API fully removed.

### R2. configure.ac — the single detection block

File: `rpkg/configure.ac`

- Exactly one `if [ -f inst/vendor.tar.xz ]` detection block, nothing else drives mode selection
- **Source branch**: writes `[patch."git+url"]` overrides into `.cargo/config.toml` (monorepo) OR leaves config minimal (standalone). No `[source.vendored-sources]`.
- **Tarball branch**: unpacks `inst/vendor.tar.xz` into `vendor/`, writes `[source.vendored-sources] directory = "..."` into `.cargo/config.toml`. No `[patch.*]`.
- No writes to `Cargo.toml`, `Cargo.lock`, or any `.rs` file
- m4 bracket escaping: all `[` / `]` in sed/grep patterns use `@<:@` / `@:>@`
- `PACKAGE_NAME` vs `PACKAGE_TARNAME`: C/Rust identifiers derived from `PACKAGE_NAME` convert BOTH hyphens AND dots to underscores
- `AC_CONFIG_COMMANDS`: `$1` is empty; `$0` used or avoided. Shell `trap … EXIT` is on one logical line if present.
- Generated `rpkg/configure` is committed and matches `autoconf < configure.ac`

### R3. Makevars.in and generated Makevars

File: `rpkg/src/Makevars.in`

- `--print link-args` and `--print native-static-libs` flags are present (permanent, per feedback)
- Source mode: `CARGO_FLAGS` does not include `--offline`
- Tarball mode: `CARGO_FLAGS` includes `--offline` (set by configure when tarball detected)
- `-Wl,--whole-archive` (Linux) / `-Wl,-force_load` (macOS) for linkme `distributed_slice`
- Template `Makevars.in` in `minirextendr/inst/templates/rpkg/Makevars.in` matches rpkg (modulo template placeholders)

### R4. bootstrap.R — all three copies

Files: `rpkg/bootstrap.R`, `tests/model_project/bootstrap.R`, `minirextendr/inst/templates/rpkg/bootstrap.R`

All three must:
- NOT call any R function from minirextendr or any other package
- NOT reference `NOT_CRAN`, `miniextendr_vendor`, or any env var
- Use `system2("bash", "./configure")` on Unix, `system("sh configure.ucrt")` / `system("sh configure.win")` on Windows
- Work when called by `R CMD build` with `Config/build/bootstrap: TRUE` in DESCRIPTION

```bash
diff rpkg/bootstrap.R tests/model_project/bootstrap.R
diff rpkg/bootstrap.R minirextendr/inst/templates/rpkg/bootstrap.R
# Any diff must be intentional and documented
```

### R5. vendor.R — vendor_crates_io

File: `minirextendr/R/vendor.R`

- `vendor_crates_io(path)`: calls `cargo revendor` with `--compress inst/vendor.tar.xz`, `--strip-all`, `--source-root <path>`, `--source-marker`
- Does NOT strip checksums itself (delegated to cargo-revendor since #321 fix)
- `check_cargo_revendor()`: verifies cargo-revendor binary is available, helpful error if not
- `strip_vendored_dir()`: verify this is dead code or still needed; if dead, flag it

### R6. workflow.R — miniextendr_vendor

File: `minirextendr/R/workflow.R`

- `miniextendr_vendor(path)`:
  - Calls `vendor_crates_io()` (which runs cargo-revendor with `--compress`)
  - Emits `cli_alert_warning` (not `cli_alert_info`) telling user to **delete `inst/vendor.tar.xz`** before resuming development
  - The warning text must mention the file path explicitly
  - Does NOT run configure, does NOT call `R CMD build`
- `miniextendr_configure(path)`: runs `bash ./configure`, not bare `./configure`
- `miniextendr_autoconf(path)`: runs `autoconf` in the package dir
- `miniextendr_build(path)`: calls autoconf → configure → install → document. Does NOT vendor.
- `miniextendr_check(path)`: check function; does it call `miniextendr_vendor()` first? Should it?

### R7. status.R / doctor.R — no workspace-crate noise

Files: `minirextendr/R/status.R`, `minirextendr/R/doctor.R`

```bash
grep -n "miniextendr-macros\|miniextendr-lint\|miniextendr-engine\|not unpacked\|Vendored Crates" \
  minirextendr/R/status.R minirextendr/R/doctor.R
```
Expected: zero matches.

- `miniextendr_validate(path)`: checks `src/rust/Cargo.toml` declares `miniextendr-api` (the only useful dep check)
- `miniextendr_status(path)`: does NOT include "Vendored Crates" in the expected file list
- `miniextendr_doctor(path)`: Rust dep check references only `miniextendr-api`, not all 4 workspace crates

### R8. upgrade.R — upgrade_miniextendr_package

File: `minirextendr/R/upgrade.R`

- `upgrade_miniextendr_package()`: updates templates to latest version — does it trigger `miniextendr_vendor()`? Should not run it automatically.
- `check_configure_ac_drift()`: detects stale configure.ac vs template — check the comparison logic
- After upgrade, does the user need to re-run `miniextendr_vendor()`? Is this documented in the function output?

### R9. Templates — configure.ac and Makevars.in

Files: `minirextendr/inst/templates/rpkg/configure.ac`, `minirextendr/inst/templates/rpkg/Makevars.in`

- Template configure.ac must use same tarball-detection logic as rpkg: `[ -f inst/vendor.tar.xz ]`
- No references to `NOT_CRAN`, `FORCE_VENDOR`, etc.
- Template drift check: `just templates-check` must pass
- `patches/templates.patch` reflects approved delta between rpkg and templates

```bash
just templates-check
# Expected: no unexpected drift
```

### R10. justfile — vendor recipes

```bash
grep -A 30 "^vendor:" justfile
grep -A 30 "^r-cmd-build:" justfile
grep -A 30 "^r-cmd-check:" justfile
grep -A 30 "^devtools-build:" justfile
grep -A 5 "^lock-shape-check:" justfile
grep -A 5 "^vendor-sync-check:" justfile
```

Check for each build/check recipe:
- `just vendor` is called before `R CMD build` / `R CMD check`
- `trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT` is present
- Trap and the subsequent command are on ONE logical line via `\` continuation (justfile runs each line in its own `bash -c` — a trap on a separate line fires immediately and is useless)
- `rm -rf rpkg/vendor` happens after `just vendor` and before `R CMD build` (vendor dir must not be in the source tarball, only `inst/vendor.tar.xz`)
- `lock-shape-check` recipe correctly greps the committed file (not just staged lines)

### R11. CI — ci.yml vendor steps

```bash
grep -n "vendor\|NOT_CRAN\|tarball\|bootstrap\|configure" .github/workflows/ci.yml | grep -v "^.*#" | head -40
```

Check:
- `just vendor` runs before `R CMD build` / `R CMD check` in the tarball job
- No `NOT_CRAN=true` environment variable in any job
- After `just vendor`, `rm -rf rpkg/vendor` before `R CMD build` (to keep tarball lean)
- The tarball job comment says "configure.ac detects tarball install mode from the presence of inst/vendor.tar.xz"
- Vendor cache key includes `Cargo.lock` + `**/Cargo.toml` + `rpkg/src/rust/Cargo.lock`
- Source job (no tarball) runs without any vendor step

### R12. cargo-revendor — all CLI flags documented and correct

File: `cargo-revendor/src/main.rs`

For every flag in `Cli`:
- `--manifest-path`: auto-discovery fallback order is `src/rust/Cargo.toml` → `./Cargo.toml` → `*/src/rust/Cargo.toml`
- `--output`: relative to CWD, not manifest dir
- `--source-root`: passed as `git_overrides` to `partition_packages` in all three `run_*` functions
- `--strip-all`: implies strip-tests + strip-benches + strip-examples + strip-bins
- `--freeze` + `--strict-freeze`: only in `run_full` — gated with error in phase modes
- `--compress` + `--blank-md`: only in `run_full` (or `run_local_only` when externals present)
- `--verify`: dispatched before mode selection, works on whatever vendor/ contains
- `--sync`: included in external cache hash, not local cache hash
- `--flat-dirs`: reverts to old cargo vendor flat layout
- `--external-only` / `--local-only`: mutually exclusive; see phase mode correctness below
- `--force`: bypasses cache in all modes

### R13. cargo-revendor — partition_packages + git_overrides

File: `cargo-revendor/src/metadata.rs`

- `resolve_git_override(name, git_version, overrides)`: name match + version equality check; errors with clear diagnostic on mismatch
- All 3 call sites pass `&source_root_members` as third argument:
  ```bash
  grep -n "partition_packages" cargo-revendor/src/main.rs
  # Expected: 3 lines, all with 3 args
  ```
- 6 unit tests present covering: empty, no-match, match, version-mismatch, duplicate-names, same-version unrelated

### R14. cargo-revendor — strip.rs

File: `cargo-revendor/src/strip.rs`

- `strip_toml_sections()`: removes `[dev-dependencies]`, `[[bench]]`, `[[test]]`, `[[example]]`, `[[bin]]` as configured — does NOT remove `[features]` entries that don't reference stripped deps
- `prune_dangling_feature_refs(content, removed_deps)`: removes feature array entries matching `"<dep>"`, `"<dep>/..."`, `"<dep>?/..."` for each removed dep; empty arrays kept as `[]`
- `strip_crate_dir()`: calls `prune_dangling_feature_refs` with the set of removed dev-dep names
- Test: `strip_crate_dir_prunes_dangling_features_after_dev_dep_strip` passes

### R15. cargo-revendor — vendor.rs

File: `cargo-revendor/src/vendor.rs`

- `strip_lockfile_inplace(lockfile, v)`: strips `^checksum = ` lines, overwrites in-place, preserves trailing newline
- Called in `run_full` when `--compress` is given without `--freeze`
- `freeze_manifest()`: rewrites git deps to vendor paths, strips `[patch.*]`, adds `[patch.crates-io]` for transitive locals, sorted alphabetically
- `generate_cargo_config()`: rescans all of `vendor/` dir — produces correct config regardless of run order (external-only, local-only, or full)
- `strip_vendor_path_deps()`: removes relative path deps from vendored Cargo.toml files (only needed after `cargo vendor`, not after local crate extraction)

### R16. cargo-revendor — cache.rs

File: `cargo-revendor/src/cache.rs`

- `compute_hash_external`: hashes Cargo.lock + Cargo.toml + sync manifests ONLY — does NOT hash local source trees
- `compute_hash_local`: hashes local crate source trees ONLY — does NOT hash Cargo.lock
- These are fully independent: a local source edit must NOT invalidate the external cache; a Cargo.lock change must NOT invalidate the local cache
- Legacy `.revendor-cache` still written by `run_full` for backward compat
- FNV-1a vectors test still passes (stability guarantee for cross-toolchain cache keys)

### R17. cargo-revendor — phase modes correctness

File: `cargo-revendor/src/main.rs`

`run_external_only`:
- Has bootstrap-seed step before metadata call (without it, `cargo metadata` fails on fresh clone with frozen path deps)
- Calls `remove_flat_dirs` (or equivalent) to remove local-crate flat dirs from staging after `cargo vendor` (cargo vendor may emit placeholder dirs for git/path deps)
- Uses `merge_copy_vendor` (not `remove_dir_all + rename`) for step 8
- Does NOT call `save_cache()` (legacy full) or `save_cache_local()`

`run_local_only`:
- Cache check runs BEFORE bootstrap-seed and metadata (not after — avoid unnecessary I/O on cache hit)
- Does NOT call `save_cache()` (legacy full) — writing the full cache after only local work would cause false hits when external deps later change
- Uses `merge_copy_vendor` for step 8
- Does NOT run `strip_vendor_path_deps` (that's only needed after `cargo vendor`)

`merge_copy_vendor`:
- Does NOT `remove_dir_all(output)` — only removes entries it's replacing
- Works even when `output/` doesn't exist yet (creates it)

### R18. .githooks/pre-commit

File: `.githooks/pre-commit`

- `$STAGED` variable defined before the Cargo.lock block
- Block checks staged diff (added lines only: `grep '^+'`, exclude `'^+++'`)
- Error messages point to `just vendor`
- Does NOT check unstaged changes (only staged)
- `bash -n .githooks/pre-commit` passes

---

## Part 2 — Automated test suites

```bash
# cargo-revendor: unit + offline integration tests
just revendor-test 2>&1 | grep "^test result"
# Expected: 76 passed, 0 failed across all bins

# minirextendr: R tests
just minirextendr-test 2>&1 > /tmp/minirextendr-test.log
grep "\[ FAIL\|\[ PASS\|\[ WARN\|\[ SKIP" /tmp/minirextendr-test.log | tail -5
# Expected: [ FAIL 0 | WARN 4 | SKIP 3 | PASS >= 375 ]

# template sync
just templates-check
# Expected: no drift

# vendor sync (workspace sources vs vendor/)
just vendor-sync-check
# Expected: pass (or "vendor/ absent — run just vendor first" is acceptable)

# lock shape
just lock-shape-check
# Expected: OK

# hook syntax
bash -n .githooks/pre-commit && echo "PASS: hook syntax"
```

---

## Part 3 — Install pathway matrix

### P1. Source mode — R CMD INSTALL (dev, no tarball)

```bash
[ ! -f rpkg/inst/vendor.tar.xz ] || { echo "SETUP: remove tarball first"; rm rpkg/inst/vendor.tar.xz; }

cd rpkg && bash ./configure && cd ..
grep -q "vendored-sources" rpkg/src/rust/.cargo/config.toml \
  && echo "FAIL: source mode has vendored-sources" || echo "PASS: source mode config"

R CMD INSTALL rpkg 2>&1 > /tmp/p1-source-install.log
grep -c "Downloading crates" /tmp/p1-source-install.log   # OK if > 0 (network, expected in source mode)
grep "ERROR\|error\[E" /tmp/p1-source-install.log | grep -v "^#" | head -5
Rscript -e 'library(miniextendr); cat("PASS: P1 source R CMD INSTALL\n")'
```

### P2. Source mode — devtools::install()

```bash
[ ! -f rpkg/inst/vendor.tar.xz ] || rm rpkg/inst/vendor.tar.xz
Rscript -e '
  devtools::install("rpkg")
  library(miniextendr)
  cat("PASS: P2 devtools source\n")
' 2>&1 > /tmp/p2-devtools.log
tail -3 /tmp/p2-devtools.log
```

### P3. Source mode — just configure + just rcmdinstall (developer workflow)

```bash
[ ! -f rpkg/inst/vendor.tar.xz ] || rm rpkg/inst/vendor.tar.xz
just configure
just rcmdinstall 2>&1 > /tmp/p3-rcmdinstall.log
Rscript -e 'library(miniextendr); cat("PASS: P3 just rcmdinstall\n")'
```

### P4. Tarball mode — full CRAN submission pipeline

```bash
# Vendor (creates inst/vendor.tar.xz)
just vendor 2>&1 > /tmp/p4-vendor.log
[ -f rpkg/inst/vendor.tar.xz ] || { echo "FAIL: no tarball"; exit 1; }
echo "Tarball size: $(du -h rpkg/inst/vendor.tar.xz | cut -f1)"

# Configure detects tarball mode
cd rpkg && bash ./configure && cd ..
grep -q "vendored-sources" rpkg/src/rust/.cargo/config.toml \
  && echo "PASS: tarball mode config" || echo "FAIL: not in tarball mode"
grep -q 'patch\."git+' rpkg/src/rust/.cargo/config.toml \
  && echo "FAIL: patch stanza still present in tarball mode" || echo "PASS: no patch in tarball mode"

# R CMD build (removes vendor/, packages inst/vendor.tar.xz into the tarball)
just r-cmd-build 2>&1 > /tmp/p4-build.log

# Trap must have cleaned up
[ ! -f rpkg/inst/vendor.tar.xz ] \
  && echo "PASS: tarball trap-cleanup worked" \
  || echo "FAIL: inst/vendor.tar.xz still present after build"

# The built tarball must contain inst/vendor.tar.xz
tarball=$(ls miniextendr_*.tar.gz 2>/dev/null | sort -V | tail -1)
[ -n "$tarball" ] || { echo "FAIL: no source tarball built"; exit 1; }
tar tf "$tarball" | grep -q "vendor.tar.xz" \
  && echo "PASS: vendor tarball embedded in source tarball" \
  || echo "FAIL: vendor tarball missing from source tarball"
tar tf "$tarball" | grep -q "^miniextendr[^/]*/vendor/" \
  && echo "FAIL: expanded vendor/ dir leaked into source tarball" \
  || echo "PASS: no raw vendor/ in source tarball"

# R CMD check on the tarball (offline-capable)
just r-cmd-check 2>&1 > /tmp/p4-check.log
grep -E "^Status:|^ERROR|^WARNING" /tmp/p4-check.log | head -10

# Install from the built tarball
R CMD INSTALL "$tarball" 2>&1 | tail -5
Rscript -e 'library(miniextendr); cat("PASS: P4 tarball install\n")'
grep -c "Downloading crates" /tmp/p4-check.log \
  && echo "FAIL: cargo downloaded during R CMD check (not fully offline)" \
  || echo "PASS: no cargo downloads during R CMD check"
```

### P5. Stale tarball footgun

```bash
just vendor > /dev/null 2>&1
[ -f rpkg/inst/vendor.tar.xz ] || { echo "SKIP"; exit 0; }

# Dev forgets to delete tarball → configure enters tarball mode
cd rpkg && bash ./configure && cd ..
grep -q "vendored-sources" rpkg/src/rust/.cargo/config.toml && echo "Footgun confirmed: in tarball mode"

# devtools::install() in this state uses stale vendor
# The fix: miniextendr_vendor() warning must tell user to delete it
Rscript -e 'minirextendr::miniextendr_vendor()' 2>&1 | grep -iE "delete|remove|cleanup|rm"
# Expected: warning text mentioning deletion

# Simulate user following the warning
rm rpkg/inst/vendor.tar.xz
cd rpkg && bash ./configure && cd ..
grep -q "vendored-sources" rpkg/src/rust/.cargo/config.toml \
  && echo "FAIL: still in tarball mode after cleanup" || echo "PASS: back to source mode"
```

### P6. remotes::install_github (package consumer, source mode)

```bash
# Confirm vendor tarball is not tracked in git (would corrupt remote installs)
git ls-files rpkg/inst/vendor.tar.xz | grep -q . \
  && echo "FAIL: vendor tarball tracked in git" || echo "PASS: vendor tarball not in git"

# Install from GitHub (network required)
Rscript -e '
  tryCatch({
    remotes::install_github("A2-ai/miniextendr", subdir = "rpkg", quiet = FALSE)
    library(miniextendr)
    cat("PASS: P6 remotes::install_github\n")
  }, error = function(e) cat("SKIP/FAIL:", conditionMessage(e), "\n"))
' 2>&1 | tail -5
```

### P7. install.packages from CRAN tarball (consumer, offline)

Simulates what CRAN check machines do: install from the built tarball, no network.

```bash
tarball=$(ls miniextendr_*.tar.gz 2>/dev/null | sort -V | tail -1)
[ -z "$tarball" ] && { echo "SKIP: build P4 first"; exit 0; }

# Install with no network (CRAN check simulation)
R_CRAN_WEB="" R CMD INSTALL --no-multiarch "$tarball" 2>&1 > /tmp/p7-cran-install.log
grep "Downloading crates\|error\[E\|ERROR" /tmp/p7-cran-install.log | head -5
# Expected: no Downloading, no errors
Rscript -e 'library(miniextendr); cat("PASS: P7 CRAN install\n")'
```

### P8. NOT_CRAN env var must be ignored

```bash
NOT_CRAN=true R CMD INSTALL rpkg 2>&1 > /tmp/p8-notcran.log
# Should behave identically to P1 (source mode)
grep -i "NOT_CRAN\|FORCE_VENDOR" /tmp/p8-notcran.log | head -5
# Expected: no mentions in configure output
Rscript -e 'library(miniextendr); cat("PASS: P8 NOT_CRAN ignored\n")'
```

### P9. New package via create_miniextendr_package + full CRAN prep

```bash
tmp=$(mktemp -d)
Rscript -e "
  library(minirextendr)
  create_miniextendr_package('$tmp/mypkg', open = FALSE)
" 2>&1

# Structural checks
for f in configure.ac bootstrap.R src/rust/Cargo.toml src/Makevars.in; do
  [ -f "$tmp/mypkg/$f" ] && echo "PASS: $f" || echo "FAIL: $f missing"
done

# bootstrap.R must not reference R functions or env vars
grep -iE "miniextendr_vendor|NOT_CRAN|library\(|require\(" "$tmp/mypkg/bootstrap.R" \
  && echo "FAIL: bootstrap.R has forbidden content" || echo "PASS: bootstrap.R clean"

# configure.ac must use tarball-detection, not env vars
grep -q "vendor.tar.xz" "$tmp/mypkg/configure.ac" \
  && echo "PASS: configure.ac uses tarball detection" || echo "FAIL"
grep -iE "NOT_CRAN|FORCE_VENDOR" "$tmp/mypkg/configure.ac" \
  && echo "FAIL: configure.ac has legacy env var" || echo "PASS: no legacy env vars"

# Source install
cd "$tmp/mypkg"
autoconf && bash ./configure
R CMD INSTALL . 2>&1 | tail -3

# CRAN prep via minirextendr_vendor
Rscript -e "minirextendr::miniextendr_vendor('$tmp/mypkg')" 2>&1 | tee /tmp/p9-vendor.log
grep -iE "delete|remove|cleanup" /tmp/p9-vendor.log \
  && echo "PASS: cleanup warning present" || echo "FAIL: no cleanup warning"
[ -f "$tmp/mypkg/inst/vendor.tar.xz" ] && echo "PASS: tarball created" || echo "FAIL"

# R CMD build (bootstrap.R runs configure which detects the tarball)
cd "$tmp/mypkg"
R CMD build . 2>&1 | tail -3
tarball=$(ls *.tar.gz 2>/dev/null | tail -1)
[ -n "$tarball" ] && tar tf "$tarball" | grep vendor.tar.xz \
  && echo "PASS: vendor tarball in built package" || echo "FAIL"

# User cleanup (the warning told them to)
rm -f inst/vendor.tar.xz
bash ./configure
grep -q "vendored-sources" src/rust/.cargo/config.toml \
  && echo "FAIL: still in tarball mode" || echo "PASS: back to source after cleanup"

cd -
rm -rf "$tmp"
```

### P10. Monorepo via create_miniextendr_monorepo

```bash
tmp=$(mktemp -d)
Rscript -e "
  library(minirextendr)
  create_miniextendr_monorepo('$tmp/myproject', open = FALSE)
" 2>&1

[ -f "$tmp/myproject/Cargo.toml" ]        && echo "PASS: workspace Cargo.toml" || echo "FAIL"
[ -f "$tmp/myproject/rpkg/configure.ac" ] && echo "PASS: rpkg/configure.ac"    || echo "FAIL"
[ -f "$tmp/myproject/rpkg/bootstrap.R" ]  && echo "PASS: rpkg/bootstrap.R"     || echo "FAIL"

# Source install of the embedded R package
cd "$tmp/myproject/rpkg"
autoconf && bash ./configure
R CMD INSTALL . 2>&1 | tail -3
cd -

rm -rf "$tmp"
```

### P11. upgrade_miniextendr_package

```bash
tmp=$(mktemp -d)
Rscript -e "
  library(minirextendr)
  create_miniextendr_package('$tmp/mypkg', open = FALSE)
" 2>&1

# Simulate old configure.ac by backdating a field
# Then upgrade and verify it's refreshed
Rscript -e "
  setwd('$tmp/mypkg')
  minirextendr::upgrade_miniextendr_package(path = '$tmp/mypkg')
" 2>&1 | head -10

# After upgrade: configure.ac must still use tarball detection
grep -q "vendor.tar.xz" "$tmp/mypkg/configure.ac" \
  && echo "PASS: upgraded configure.ac still uses tarball detection" || echo "FAIL"
grep -iE "NOT_CRAN|FORCE_VENDOR" "$tmp/mypkg/configure.ac" \
  && echo "FAIL: upgraded configure.ac has legacy env var" || echo "PASS"

rm -rf "$tmp"
```

### P12. miniextendr_validate / doctor / status on a real package

```bash
# On the rpkg itself (source mode, no tarball)
[ -f rpkg/inst/vendor.tar.xz ] && rm rpkg/inst/vendor.tar.xz

Rscript -e "
  library(minirextendr)
  result <- miniextendr_validate('rpkg')
  cat('validate:', if (isTRUE(result)) 'PASS' else 'FAIL', '\n')
"

Rscript -e "
  library(minirextendr)
  r <- miniextendr_doctor('rpkg')
  cat('doctor fail:', length(r[['fail']]), '\n')
  if (length(r[['fail']]) == 0) cat('PASS\n') else { cat('FAIL\n'); print(r[['fail']]) }
"

Rscript -e "
  library(minirextendr)
  r <- miniextendr_status('rpkg')
  cat('missing:', length(r[['missing']]), '\n')
  # In source mode, vendor/ absent is acceptable
  cat('PASS: status ran without error\n')
"
```

---

## Part 4 — cargo-revendor feature tests

### C1. git-source seeding — local edits propagate

```bash
[ ! -f rpkg/inst/vendor.tar.xz ] || rm rpkg/inst/vendor.tar.xz
echo "// test-marker-$(date +%s)" >> miniextendr-api/src/lib.rs

just vendor 2>&1 | grep -E "Local packages|miniextendr-api"
# Expected: Local packages includes miniextendr-api

marker=$(grep "test-marker" miniextendr-api/src/lib.rs | tail -1 | tr -d '/')
grep -r "$marker" rpkg/vendor/miniextendr-api/ 2>/dev/null | head -2
# Expected: match found (local edit in vendor)

grep -c "test-marker" rpkg/vendor/miniextendr-api/src/lib.rs 2>/dev/null \
  && echo "PASS: local edit propagated to vendor" || echo "FAIL: local edit not in vendor"

git checkout miniextendr-api/src/lib.rs
rm -f rpkg/inst/vendor.tar.xz
```

### C2. version mismatch error

```bash
# cargo-revendor must error when --source-root crate version != lockfile version
# Test via the unit test (avoids needing a real workspace setup):
cargo test --manifest-path cargo-revendor/Cargo.toml \
  resolve_git_override 2>&1 | grep -E "ok|FAILED"
# Expected: all resolve_git_override tests ok
```

### C3. strip-all removes dev dirs, prunes dangling features

```bash
# Verified by unit tests — run specifically:
cargo test --manifest-path cargo-revendor/Cargo.toml \
  strip_crate_dir_prunes_dangling 2>&1 | grep -E "ok|FAILED"
# Expected: ok
```

### C4. Lockfile checksum stripping — canonical Cargo.lock

```bash
# After just vendor, the canonical Cargo.lock must have no checksum lines
just vendor > /dev/null 2>&1
grep -c "^checksum = " rpkg/src/rust/Cargo.lock \
  && echo "FAIL: canonical Cargo.lock has checksums after vendor" \
  || echo "PASS: no checksums in canonical Cargo.lock"
rm -f rpkg/inst/vendor.tar.xz
```

### C5. Phase modes — external-only then local-only compose to full

```bash
tmp=$(mktemp -d)

# Full run baseline
cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . --output "$tmp/full" \
  --strip-all -v 2>&1 | grep "vendored"
full=$(ls "$tmp/full" | grep -v "^\." | wc -l | tr -d ' ')

# Phase run
cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . --output "$tmp/phase" \
  --external-only --strip-all 2>&1 | tail -2

cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . --output "$tmp/phase" \
  --local-only 2>&1 | tail -2

phase=$(ls "$tmp/phase" | grep -v "^\." | wc -l | tr -d ' ')

[ "$full" -eq "$phase" ] \
  && echo "PASS: full=$full == phase=$phase crate dirs" \
  || echo "FAIL: full=$full != phase=$phase"

# Cache files written
[ -f "$tmp/phase/.revendor-cache-external" ] && echo "PASS: external cache" || echo "FAIL"
[ -f "$tmp/phase/.revendor-cache-local" ]    && echo "PASS: local cache"    || echo "FAIL"

rm -rf "$tmp"
```

### C6. External cache hit on unchanged lockfile

```bash
tmp=$(mktemp -d)
cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . --output "$tmp/v" \
  --external-only --strip-all 2>&1 | tail -2

# Second run must be a no-op
out=$(cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . --output "$tmp/v" \
  --external-only --strip-all -v 2>&1)
echo "$out" | grep -iE "up.to.date|cached|unchanged" \
  && echo "PASS: external cache hit" || echo "FAIL: no cache hit"

rm -rf "$tmp"
```

### C7. Phase-mode flag compatibility errors

```bash
cargo revendor --external-only --local-only \
  --manifest-path rpkg/src/rust/Cargo.toml 2>&1 | grep -iE "conflict|cannot"
echo "Exit $? (expected non-zero)"

cargo revendor --external-only --freeze \
  --manifest-path rpkg/src/rust/Cargo.toml 2>&1 | grep -iE "incompatible|cannot|error"
echo "Exit $? (expected non-zero)"

tmp=$(mktemp -d)
cargo revendor --local-only --compress "$tmp/x.tar.xz" \
  --manifest-path rpkg/src/rust/Cargo.toml --output "$tmp/v" 2>&1 | grep -iE "external|first|incompatible"
echo "Exit $? (expected non-zero)"
rm -rf "$tmp"
```

### C8. --verify catches drift

```bash
tmp=$(mktemp -d)
# Vendor then corrupt a crate dir
cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . --output "$tmp/v" \
  --strip-all 2>&1 | tail -2

# Corrupt by removing a crate
first_crate=$(ls "$tmp/v" | grep -v "^\." | head -1)
rm -rf "$tmp/v/$first_crate"

cargo revendor --verify \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --output "$tmp/v" 2>&1 | grep -iE "mismatch|missing|drift|error"
echo "Exit $? (expected non-zero)"

rm -rf "$tmp"
```

### C9. --freeze produces an offline-buildable manifest

```bash
tmp=$(mktemp -d)
cp rpkg/src/rust/Cargo.toml "$tmp/Cargo.toml.orig"

cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . --output "$tmp/v" \
  --strip-all --freeze -v 2>&1 | grep -E "freeze|Rewriting|patch"

# After freeze: Cargo.toml must not contain git = "" deps for workspace crates
grep -E 'git = "http' rpkg/src/rust/Cargo.toml | head -5
# Expected: none (frozen to vendor paths)

# Restore
cp "$tmp/Cargo.toml.orig" rpkg/src/rust/Cargo.toml
cd rpkg && bash ./configure && cd ..
rm -rf "$tmp"
```

### C10. --sync merges disjoint workspaces

```bash
tmp=$(mktemp -d)
cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --source-root . \
  --output "$tmp/v" \
  --sync miniextendr-bench/Cargo.toml \
  --strip-all -v 2>&1 | tail -5
# Expected: vendored deps from both manifests in $tmp/v
rm -rf "$tmp"
```

---

## Part 5 — Pre-commit hook + lock-shape

### H1. Hook syntax

```bash
bash -n .githooks/pre-commit && echo "PASS"
```

### H2. Hook blocks checksum lines

```bash
git config core.hooksPath .githooks
cp rpkg/src/rust/Cargo.lock /tmp/lock.bak
printf '\nchecksum = "abc123"\n' >> rpkg/src/rust/Cargo.lock
git add rpkg/src/rust/Cargo.lock
git commit -m "test should fail" 2>&1 | grep -E "checksum|pre-commit|wrong shape"
# Expected: blocked (exit non-zero, message about checksums)
git restore --staged rpkg/src/rust/Cargo.lock
cp /tmp/lock.bak rpkg/src/rust/Cargo.lock
```

### H3. Hook blocks path+ sources

```bash
sed 's|source = "git+|source = "path+|' /tmp/lock.bak > rpkg/src/rust/Cargo.lock
git add rpkg/src/rust/Cargo.lock
git commit -m "test should fail" 2>&1 | grep -E 'path\+|pre-commit|wrong shape'
# Expected: blocked
git restore --staged rpkg/src/rust/Cargo.lock
cp /tmp/lock.bak rpkg/src/rust/Cargo.lock
rm /tmp/lock.bak
```

### H4. lock-shape-check on current repo

```bash
just lock-shape-check
# Expected: OK, exit 0
```

---

## Pass criteria

| ID | Check | Target |
|----|-------|--------|
| R1 | `NOT_CRAN`/`FORCE_VENDOR` grep | 0 matches |
| R1 | `vendor_miniextendr` grep | 0 matches |
| R2 | configure source mode | no `vendored-sources` in `.cargo/config.toml` |
| R2 | configure tarball mode | `vendored-sources` present, no `[patch."git+..."]` |
| R2 | configure no source mutations | no Cargo.toml/Cargo.lock/.rs writes |
| R4 | bootstrap.R copies match | `diff` shows only intentional deltas |
| R4 | bootstrap.R no R calls | no `library(`, `require(`, `miniextendr_vendor` |
| R6 | `miniextendr_vendor()` warns | warning mentions deleting `inst/vendor.tar.xz` |
| R7 | no workspace-crate noise | `grep "not unpacked"` → 0 matches |
| R9 | templates-check | passes, no unexpected drift |
| R10 | trap-cleanup in recipes | `inst/vendor.tar.xz` absent after build/check |
| R10 | no raw vendor/ in tarball | `tar tf *.tar.gz \| grep "^.*/vendor/"` → 0 |
| R11 | CI no `NOT_CRAN=true` | 0 matches in ci.yml |
| R13 | `partition_packages` call sites | all 3 have 3 args |
| R17 | `run_local_only` no legacy cache write | no `save_cache()` call in run_local_only |
| R17 | `run_external_only` has bootstrap | bootstrap-seed step present |
| P1 | source R CMD INSTALL | library loads |
| P2 | devtools source | library loads |
| P3 | just rcmdinstall | library loads |
| P4 | tarball R CMD check | Status: OK, 0 `Downloading crates` |
| P4 | tarball trap-cleanup | `inst/vendor.tar.xz` absent after `just r-cmd-build` |
| P4 | vendor.tar.xz in source tarball | `tar tf *.tar.gz \| grep vendor.tar.xz` matches |
| P5 | stale tarball → warning → cleanup | re-configure enters source mode |
| P6 | `remotes::install_github` | library loads (requires network) |
| P8 | `NOT_CRAN=true` ignored | same result as P1 |
| P9 | new package scaffold | all required files present |
| P9 | new package configure.ac | uses `vendor.tar.xz` detection, no `NOT_CRAN` |
| P12 | validate/doctor/status | no "not unpacked" output |
| C1 | git-source seeding | local marker in `vendor/miniextendr-api/` |
| C4 | canonical Cargo.lock | 0 `checksum =` lines after `just vendor` |
| C5 | phase modes compose | `full_count == phase_count` |
| C6 | external cache hit | second run prints cache-hit message |
| C7 | `--external-only --freeze` | exits non-zero |
| C7 | `--external-only --local-only` | clap conflict error |
| C8 | `--verify` catches drift | exits non-zero on corrupted vendor |
| H1 | hook syntax | `bash -n` exit 0 |
| H2 | hook blocks checksums | commit blocked |
| H3 | hook blocks path+ | commit blocked |
| H4 | lock-shape-check | OK |
| Suite | `just revendor-test` | 76 passed, 0 failed |
| Suite | `just minirextendr-test` | FAIL 0, WARN 4, SKIP 3, PASS ≥ 375 |
