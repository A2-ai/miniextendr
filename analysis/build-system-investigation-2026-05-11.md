# Build System: Cases, Corners, and Decisions

**Date**: 2026-05-11
**Scope**: end-to-end build/install/check pipeline for miniextendr-built R packages —
project structures, configure/Makevars, wrapper generation, vendoring, Cargo.lock,
installer surfaces, R staging, incremental compilation.

**Status**: investigation only. Decisions flagged ⚖️. Confirmed bugs flagged 🐞.
Optimization opportunities flagged ⚡. Open questions flagged ❓.

Companion test plan: `analysis/build-system-tests-2026-05-11.md`.

Raw source notes (read-only, /tmp): `01-build-entry-wrappers.md`,
`02-vendor-lockfile.md`, `03-project-structures-templates.md`,
`04-installers-staging-incremental.md`.

---

## TL;DR

The build system is **largely correct** but carries several real defects and one
clear optimization opportunity:

- **One single signal** controls all install-mode behavior: `inst/vendor.tar.xz`.
  Present → tarball install (offline, vendored). Absent → source install (network
  or monorepo-patched). No `NOT_CRAN`, no `BUILD_CONTEXT`, no env-var overrides.
  This is good.

- **Two project structures** (monorepo vs. standalone rpkg) produce **byte-identical
  Makevars** and behave identically at install time. They differ only in scaffolding,
  bootstrap.R richness, and configure.ac's auto-vendor self-repair block.

- ⚡ **Wrapper generation runs unconditionally on every install**, even for tarball
  installs where the wrappers are pre-shipped and the write is already a no-op. The
  cdylib build (~10–30 s) can be skipped with a 4-line Makevars `ifeq`.
  See §6.6.

- 🐞 **`miniextendr_vendor()` (R-side) and `just vendor` produce inconsistent
  vendor trees.** The R path clears `.cargo-checksum.json` to `{"files":{}}` and
  strips `checksum =` lines from Cargo.lock; the just path uses cargo-revendor's
  recompute and preserves checksums. PR #408 made the strip-and-clear path wrong.
  See §7.6.

- 🐞 **`bootstrap.R` does not regenerate Cargo.lock**, so a developer with a
  source-shape (dirty) lock who runs `devtools::build()` ships a tarball with a
  source-shape lock. Offline install of that tarball fails. `just vendor` does the
  right thing; bootstrap.R does not. See §7.5.

- 🐞 **Monorepo template lacks `r_shim.h`**, so monorepo-scaffolded packages that
  add native C bindings (`use_native_package()`) fail at compile time. See §2.4.

- 🐞 **`upgrade_miniextendr_package()` and `use_release_workflow()` are not
  monorepo-aware** — they assume the R package is at `usethis::proj_get()`. See §2.5.

- ⚖️ **`--freeze` is documented in CLAUDE.md but never invoked**; the "stale
  frozen-vendor recovery" section is historical. Should be deleted or marked
  archived. See §7.3.

- ⚖️ **`devtools::install` always full-recompiles** because tarball staging puts
  `CARGO_TARGET_DIR` inside the ephemeral temp dir. This is the design's
  iteration-friction floor; the documented workaround is `devtools::load_all` for
  iterative dev. Acknowledged trade-off, no fix proposed here. See §11.2.

- ❓ **CLAUDE.md still describes `-Wl,--whole-archive` / `-force_load` for the
  cdylib→staticlib link**, but current `Makevars.in` does not use them. The
  `stub.c` anchor + `codegen-units = 1` design replaced them. CLAUDE.md is stale
  on this point. See §6.4.

---

## 1. Vocabulary

| Term | Meaning |
|---|---|
| **rpkg** | The example R package at `rpkg/` in this repo. Also the name of one of the two scaffolded structures. Context disambiguates. |
| **monorepo (structure)** | Project layout with a Rust workspace at the top and the R package as a named subdirectory. Scaffolded by `create_miniextendr_monorepo()`. |
| **standalone (structure)** | Project layout where the R package is at the top and the Rust crate is embedded in `src/rust/`. Scaffolded by `create_miniextendr_package()`. |
| **source mode** | Install mode where cargo resolves deps either from a monorepo `[patch."git+url"]` (dev) or from the network (end-user via `install_github`). Selected when `inst/vendor.tar.xz` is absent. |
| **tarball mode** | Install mode where cargo resolves deps offline from a vendored `vendor/` tree. Selected when `inst/vendor.tar.xz` is present. |
| **install-mode latch** | The file `rpkg/inst/vendor.tar.xz`. Its presence flips the install into tarball mode. Single signal, no other inputs. |
| **staging** | When R copies the package source to a fresh temp directory before installing. Always happens for tarball installs and for builds; never for `R CMD INSTALL <path>` or `devtools::load_all`. |
| **wrapper gen** | The `cdylib → dyn.load → write R/<pkg>-wrappers.R` pass driven from Makevars. |
| **cargo-revendor** | Standalone cargo subcommand (in `cargo-revendor/`) that vendors workspace + git deps in a way `cargo vendor` alone cannot. |
| **the lock** (unqualified) | `rpkg/src/rust/Cargo.lock` — the CRAN-relevant lockfile. The repo-root `Cargo.lock` is the workspace lock, separate. |

---

## 2. The Two Project Structures

### 2.1 Monorepo (rust lib + rpkg subdir)

```
my-project/
├── Cargo.toml               # workspace manifest; excludes rpkg/src/rust
├── justfile                 # workspace-level recipes
├── tools/bump-version.R
├── <crate_name>/            # main Rust library (pure Rust, no miniextendr)
│   ├── Cargo.toml
│   └── src/lib.rs
└── <rpkg_name>/             # the R package
    ├── DESCRIPTION
    ├── configure.ac
    ├── bootstrap.R          # lean: just runs ./configure
    ├── src/
    │   ├── Makevars.in
    │   ├── stub.c
    │   └── rust/
    │       ├── Cargo.toml   # standalone [workspace]
    │       └── lib.rs
    └── tools/lock-shape-check.R
```

Created by `create_miniextendr_monorepo()` (`minirextendr/R/create.R:67`).

The rpkg's `src/rust/Cargo.toml` declares its own `[workspace]` so cargo treats
it standalone — the parent workspace explicitly excludes it
(`Cargo.toml.tmpl:1-16`).

### 2.2 Standalone rpkg (rust crate in src/rust)

```
my.package/
├── DESCRIPTION
├── configure.ac             # has self-repair auto-vendor block
├── bootstrap.R              # rich: runs cargo-revendor then ./configure
├── R/
├── src/
│   ├── Makevars.in
│   ├── stub.c
│   ├── r_shim.h             # <-- present here
│   └── rust/
│       ├── Cargo.toml       # standalone [workspace]
│       └── lib.rs
└── tools/
```

Created by `create_miniextendr_package()` (`minirextendr/R/create.R:14`).

### 2.3 What's actually different

| Aspect | monorepo | standalone |
|---|---|---|
| `Makevars.in` | identical | identical |
| `Cargo.toml` (rpkg crate) | identical | identical |
| `configure.ac` | no self-repair block | has self-repair (49 lines) |
| `bootstrap.R` | only runs ./configure (lean) | auto-vendors + ./configure (rich) |
| `r_shim.h` | 🐞 **absent** | present |
| `justfile` | at workspace root | at package root |
| `miniextendr.yml` | at workspace root | at package root |
| Configure run-from | `cd rpkg && ./configure` | `./configure` from pkg root |

`monorepo/rpkg/configure.ac` is `rpkg/configure.ac` (exemplar) with the
self-repair block stripped — see `patches/templates.patch:77-125`.

The patch templates also reset `CARGO_FEATURES=""` (the exemplar has 31+ features
turned on for demonstration; scaffolded packages start lean) and substitute
`{{package}}` for `miniextendr` (`patches/templates.patch:173-178, 393-400`).

### 2.4 🐞 `r_shim.h` gap in monorepo

`use_miniextendr_stub()` (`minirextendr/R/use-rust.R:54-62`) copies both
`stub.c` and `r_shim.h`. But `create_rpkg_subdirectory()`
(`minirextendr/R/create.R:234-236`) — the monorepo's rpkg-subdir creator — only
copies `stub.c`. The monorepo template directory
`minirextendr/inst/templates/monorepo/rpkg/` does not contain `r_shim.h`.

Fix: copy `r_shim.h` in `create_rpkg_subdirectory()`. Trivial change.

### 2.5 🐞 Upgrade / release-workflow gaps for monorepo

- `upgrade_miniextendr_package()` (`upgrade.R:27`) calls `use_miniextendr_stub()`
  etc., which all use `usethis::proj_get()`. In a monorepo, `proj_get()` is the
  workspace root, not the rpkg subdir — so upgrade writes files at the wrong level.
- `use_release_workflow()` (`use-release-workflow.R:26`) ships `r-release.yml`
  that runs `bash ./configure` and `R CMD build .` at the repo root. For a
  monorepo, the working dir should be the rpkg subdir.
- `miniextendr.yml` is written at `usethis::proj_get()` — workspace root in
  monorepo — but logically it's R-package-level config. ❓ Decision needed:
  workspace root or rpkg subdir?

### 2.6 ❓ Where does `miniextendr.yml` belong?

Two reasonable answers. Workspace root: there's one config for the whole
project. rpkg subdir: it's R-package-level config and follows the package if
the rpkg subdir moves. **Recommendation**: rpkg subdir (logical home, follows
the package). Verify with maintainer.

---

## 3. The Install-Mode Latch

**Single signal**: `rpkg/inst/vendor.tar.xz`.

- **Present** → `IS_TARBALL_INSTALL=true` → tarball mode (offline, vendored).
- **Absent** → `IS_TARBALL_INSTALL=false` → source mode.

No other inputs decide install mode. Specifically NOT inputs:
`NOT_CRAN`, `BUILD_CONTEXT`, `--with-vendor`, `FORCE_VENDOR`, env vars,
`CRAN_RELEASE`. The single-bit design is intentional and load-bearing — every
shortcut around it has historically been removed (per `docs/CRAN_COMPATIBILITY.md`).

### 3.1 Latch lifecycle

| Phase | Who creates it | Who consumes it | Who deletes it |
|---|---|---|---|
| Maintainer prep | `just vendor` (`justfile:394-428`) | next configure | `just r-cmd-build`/`r-cmd-check`/`devtools-build` (trap EXIT, `justfile:539-587`); `just clean-vendor-leak` (`justfile:437-444`); `miniextendr_clean_vendor_leak()` (R) |
| Build via pkgbuild | `bootstrap.R` (`rpkg/bootstrap.R:26-51`) | next configure (inside the staged dir) | sealed into the tarball; original source dir is then cleaned by the trap |
| End-user tarball install | already in tarball | configure on extraction | n/a (in temp dir) |
| End-user GitHub install | configure auto-vendor block (`configure.ac:62-87`) creates it if `cargo-revendor` is on PATH | same run's configure | n/a (in temp dir) |
| Tarball install no cargo-revendor | n/a | n/a | falls through to source mode; cargo network-fetches → fails on CRAN offline farm. **Intended canary.** |

### 3.2 🐞 The latch-leak failure (#441)

If `inst/vendor.tar.xz` is left in the source tree after an interrupted
`r-cmd-build`/`r-cmd-check`, the next dev `configure` flips into tarball mode.
The Makevars cleanup then deletes `src/rust/.cargo/` from the live source tree
permanently until the next `just configure`. Subsequent edits to monorepo
siblings (`miniextendr-api/`, etc.) are silently ignored.

**Guards in place**:

- `_assert-no-vendor-leak` recipe (`justfile:452-471`) blocks dev recipes if
  the tarball is present.
- `minirextendr_doctor()` (`doctor.R:87-138`) detects both stale tarball and
  missing `.cargo/config.toml`.
- `just clean-vendor-leak` recovers.

**Holes**:

- The guard is `just`-only. Bare `R CMD INSTALL rpkg/` from the shell, or
  `devtools::install()` invoked outside `just`, has no guard.
- SIGKILL bypasses the EXIT trap.

---

## 4. configure.ac State Machine

`rpkg/configure.ac` (563 lines) is the gatekeeper. It runs once at install/dev
time and bakes all decisions into `src/Makevars` and `src/rust/.cargo/config.toml`.
Nothing reconsidered later.

Must be invoked as `bash ./configure` (not bare `./configure` — `#!/bin/sh`
breaks `AC_CONFIG_COMMANDS` passthrough on some sh).

### 4.1 Decisions, in order

| # | Decision | Inputs | Output |
|---|---|---|---|
| 1 | **SOURCE_IS_GIT** (`62-70`) | walks up from rpkg dir looking for `.git` | bool; gates only step 2 |
| 2 | **Auto-vendor** (`72-87`) | no tarball + no `.git` + cargo-revendor on PATH | runs cargo-revendor → produces `inst/vendor.tar.xz` |
| 3 | **IS_TARBALL_INSTALL** (`102-119`) | `inst/vendor.tar.xz` presence | bool; the latch |
| 4 | **CARGO_FEATURES** (`121-148`) | env var, else `tools/detect-features.R` | `--features=...` string |
| 5 | **Tool discovery** (`151-157`) | `AC_PATH_TOOL` | `CARGO`, `RUSTC`, `SED` |
| 6 | **wasm32/webR** (`193-224`) | `$CC` (emcc → wasm32) | `CARGO_BUILD_TARGET`, `RUST_TOOLCHAIN=+nightly`, `-Z build-std` |
| 7 | **Cross-compile** (`254-273`) | host ≠ build | `CARGO_BUILD_TARGET=$host` |
| 8 | **Monorepo siblings** (`327-360`) | walks up ≤5 levels for `miniextendr-api/Cargo.toml` | `MONOREPO_ROOT` (ignored if tarball mode) |

### 4.2 Output files

- `src/Makevars` (from `src/Makevars.in` via `@VAR@` substitution).
- `src/<pkg>-win.def` (from `src/win.def.in`).
- `src/rust/.cargo/config.toml` (written by `AC_CONFIG_COMMANDS` — three mutually
  exclusive branches; see §4.3).
- `src/rust/Cargo.lock` (generated via `cargo generate-lockfile` if absent;
  potentially regenerated for lockfile-v4 compat).

`.cargo/config.toml` is gitignored — it's regenerated on every configure.

### 4.3 `.cargo/config.toml` — three mutually exclusive branches

| Branch | Condition | What it writes |
|---|---|---|
| **Tarball** | `IS_TARBALL_INSTALL=true` | `[source.crates-io] → vendored-sources`, one `[source."git+<url>"] → vendored-sources` per git dep, `[source.vendored-sources] directory = <abs>/vendor`, `[build] target-dir` |
| **Source + monorepo** | tarball=false AND `MONOREPO_ROOT` found | `[patch."https://github.com/A2-ai/miniextendr"]` with `path = "<abs>/miniextendr-api"` etc, plus `[build] target-dir` |
| **Source + no monorepo** | tarball=false AND no monorepo | only `[build] target-dir`; cargo follows the bare git URL → fetches from network |

⚠️ The git-URL extraction for tarball mode uses **sed line-regex**
(`configure.ac:450`):

```sh
sed -n 's/.*git = "\(https:\/\/[^"]*\)".*/\1/p' src/rust/Cargo.toml
```

This works only for `git = "https://..."` with exactly that punctuation.
`cargo-revendor` uses structural toml_edit parsing in the same role
(`vendor.rs:649-715`). ❓ Should configure.ac call out to a small Rscript
helper instead of using sed?

### 4.4 ❓ configure runs autoconf every time it's a dep of a just recipe

`just devtools-document` depends on `just configure`, which runs `bash ./configure`,
which checks if `configure.ac` has been touched and re-runs `autoconf` if so. In
practice this is fast (≪ 1 s on modern hardware), but it's also unconditional.
If `configure.ac` is fresh, autoconf is a no-op; if not, it re-runs.

Not a bug; just observe that every `just devtools-document` re-runs the full
configure pipeline regardless of whether anything changed.

---

## 5. The Build Pipeline (Makevars)

`rpkg/src/Makevars.in` is annotated below. After `configure` substitutes
`@VAR@`, the result is `src/Makevars`.

### 5.1 Targets

```
all: $(SHLIB) $(WRAPPERS_R)
```

`$(SHLIB)` (R's variable for the final `.so`) depends on `$(OBJECTS)` and
`$(CARGO_AR)`. `$(WRAPPERS_R)` depends on `$(CARGO_CDYLIB)`. The cdylib has an
order-only prereq `| $(CARGO_AR)`.

### 5.2 Build order (effective)

1. Compile `stub.c` → `stub.o`. (R's build system invokes the C compiler.)
2. `$(CARGO_AR)`: `cargo build --lib --profile release` → `libpkg.a`
   (staticlib; warms cargo's incremental cache).
3. `$(SHLIB)`: linker links `stub.o + libpkg.a` → the final `.so`.
4. `$(CARGO_CDYLIB)`: `cargo rustc --crate-type cdylib` → temp shared library
   (fast, cache warm from step 2).
5. `$(WRAPPERS_R)`: Rscript loads cdylib, calls `miniextendr_write_wrappers`
   and `miniextendr_write_wasm_registry`, unloads, deletes cdylib.

### 5.3 Post-build cleanup

- **Source mode**: `touch src/rust/Cargo.toml` (force pkgbuild to consider
  sources stale next time).
- **Tarball mode**: `rm -rf vendor/ rust-target/ ra-target/ src/rust/.cargo/
  src/rust/target/` — saves disk in installed package.

### 5.4 Why no `--whole-archive` / `-force_load`

CLAUDE.md says the build uses these. Current Makevars.in does **not**.

The current design uses an *anchor* trick: `stub.c` declares

```c
extern const char miniextendr_force_link;
const void *miniextendr_anchor = &miniextendr_force_link;
```

The symbol `miniextendr_force_link` is defined in the Rust crate by
`miniextendr_init!()` (`miniextendr-macros/src/lib.rs:2589`). When the linker
sees `stub.o` referencing it, it must extract the archive member containing it.
With `codegen-units = 1` in Cargo.toml, the **entire user crate** compiles into
a single `.o` inside the archive. Extracting that one member drags every
`#[distributed_slice]` entry along.

This is more portable than per-platform linker flags. ❓ Update CLAUDE.md
to reflect the anchor design (see §13.6).

### 5.5 Force-cargo + cargo's incremental

`$(CARGO_AR)` depends on `FORCE_CARGO` (a phony target with no recipe). Every
make run invokes cargo. Cargo's own incremental compilation then decides what
to rebuild — make never tries to second-guess cargo. This is correct.

---

## 6. Wrapper Generation

### 6.1 Pipeline

1. **Compile-time registration**: every `#[miniextendr]` proc-macro invocation
   emits a `#[distributed_slice(MX_R_WRAPPERS)]` entry. Eight distributed slices
   total (`miniextendr-api/src/registry.rs:32-104`).

2. **cdylib build**: `cargo rustc --crate-type cdylib` produces a temporary
   shared library. The linker gathers all linkme entries into contiguous arrays.

3. **dyn.load + .Call**: `Rscript` invokes:

   ```sh
   MINIEXTENDR_CDYLIB_WRAPPERS=1 Rscript -e "invisible(local({
       lib <- dyn.load('libpkg.dylib')
       .Call(getNativeSymbolInfo('miniextendr_write_wrappers', lib), '<path>')
       .Call(getNativeSymbolInfo('miniextendr_write_wasm_registry', lib), '<path>')
       dyn.unload('libpkg.dylib')
   }))"
   ```

4. **Skip-ALTREP during wrapper gen**:
   `MINIEXTENDR_CDYLIB_WRAPPERS=1` is read in `registry.rs:393`:

   ```rust
   let wrapper_gen = std::env::var_os("MINIEXTENDR_CDYLIB_WRAPPERS").is_some();
   if !wrapper_gen { for reg in altrep_regs().iter() { (reg.register)(); } }
   ```

   ALTREP registration is **skipped** in wrapper-gen mode. Critical: ALTREP
   creates R-global entries with method pointers into the cdylib. After
   `dyn.unload`, those pointers dangle, and the subsequent staticlib's
   `R_init_*` would encounter stale entries → heap corruption.

5. **Assembly** (`registry.rs:721-936`):
   - Fixed preamble (`.miniextendr_raise_condition` helper).
   - `collect_r_wrappers()` (`468-497`): sort by priority (Sidecar=0,
     Class=1, Function=2, TraitImpl=3, Vctrs=4), dedup, inject `@rdname`,
     topo-sort S7 classes.
   - Resolve placeholders: `match.arg` choices, `match.arg` `@param` docs,
     S7 sidecar property docs, class-name refs (loud and silent variants).
   - **Write-if-changed** (`918-935`): if existing file is byte-identical,
     return without writing — avoids triggering devtools::document churn.

6. **Cleanup**: `dyn.unload` + `rm -f <cdylib>`.

### 6.2 Eight distributed slices

| Slice | Purpose |
|---|---|
| `MX_CALL_DEFS` | `R_CallMethodDef` table (C entry points registered via `R_registerRoutines`) |
| `MX_R_WRAPPERS` | R source fragments + priority + originating file |
| `MX_MATCH_ARG_CHOICES` | placeholder → `c("a","b","c")` substitution for R formals |
| `MX_MATCH_ARG_PARAM_DOCS` | placeholder → `@param` docstring text |
| `MX_CLASS_NAMES` | rust type → R class name resolution |
| `MX_S7_SIDECAR_PROPS` | property docs for S7 sidecars |
| `MX_ALTREP_REGISTRATIONS` | ALTREP class registration (skipped in wrapper-gen mode) |
| `MX_TRAIT_DISPATCH` | trait-ABI vtable shims |

### 6.3 The artifact

`rpkg/R/miniextendr-wrappers.R`: **27,287 lines** (1.1 MB). Tracked in git.
Must be committed in sync with the Rust changes that produced it; pre-commit
hook (`.githooks/pre-commit`) enforces NAMESPACE staged whenever
`*-wrappers.R` is staged.

### 6.4 ⚡ Skip wrapper gen in tarball mode

**The opportunity**: when installing from a built tarball, `R/<pkg>-wrappers.R`
is already in the tarball. The wrapper gen pass rebuilds the same file (and the
write is already a no-op because of write-if-changed). The cdylib build itself
is the wasted work — typically 10–30 s.

**Why it's safe**:

1. The tarball was built from a commit where `R/<pkg>-wrappers.R` matched the
   Rust code (enforced by pre-commit hook).
2. The same Rust code is what gets compiled at install time.
3. Therefore the committed wrappers are exactly what wrapper gen would
   regenerate — the cdylib build is pure waste.
4. The write-if-changed already shows this: the bytes match, the write is a
   no-op. The cost is the cdylib + dyn.load round-trip, not the write itself.

**Detection**: `IS_TARBALL_INSTALL` is already substituted into Makevars.

**Implementation sketch** (`src/Makevars.in:115-133`):

```make
ifeq ($(IS_WASM_INSTALL),true)
$(WRAPPERS_R):
 @touch $(WRAPPERS_R)
else ifeq ($(IS_TARBALL_INSTALL),true)
$(WRAPPERS_R):
 @echo "tarball install: using pre-shipped R/$(PACKAGE_NAME)-wrappers.R"
 @touch $(WRAPPERS_R)
else
$(WRAPPERS_R): $(CARGO_CDYLIB)
 @... existing recipe ...
endif
```

When `IS_TARBALL_INSTALL=true`, `$(CARGO_CDYLIB)` is never targeted →
cdylib build skipped entirely.

**Risks**: none material. The only theoretical concern is "tarball ships
out-of-sync wrappers.R" — prevented by the pre-commit hook by construction.
`wasm_registry.rs` is also already in the tarball and similarly stable.

**Time saved per tarball install**: 10–30 s (cdylib rustc link + R process
startup + dyn.load round-trip). On CRAN's offline farm this directly reduces
build time and resource use.

**Recommendation**: implement, with a guard that asserts wrappers.R exists in
the tarball-mode case. Track as a follow-up issue.

### 6.5 Tests this needs

See companion test plan §T5 — must verify:

- skip path is taken for tarball install
- wrappers.R loaded by the installed `.so` matches expectations (no drift)
- non-tarball installs unchanged (skip does not fire)
- wasm path unchanged

### 6.6 ⚡ `just devtools-document` re-runs configure on every call

`just devtools-document` lists `configure` as a prereq → every roxygen-only edit
re-runs configure. Configure is fast (~1 s) but unconditional. Not a defect, but
in heavy-iteration workflows the accumulated time is noticeable.

Possible mitigations:

- Skip configure if `src/Makevars` and `src/rust/.cargo/config.toml` are both
  newer than `configure.ac`.
- Inline a fast guard: `if [ Makevars -nt configure.ac ]; then exit 0; fi`.

Trade-off: harder to debug if a stale Makevars goes undetected. ❓ Worth fixing?

---

## 7. Vendoring + cargo-revendor

### 7.1 Why plain `cargo vendor` doesn't work

The rpkg crate depends on `miniextendr-{api,lint,macros}` as **git URLs**, but
these crates are in the same monorepo. Plain `cargo vendor`:

- Cannot package workspace path deps with `*.workspace = true` inheritance
  resolved.
- Does not rewrite `path = "../sibling"` cross-refs in vendored Cargo.toml
  files, which are valid inside the workspace but broken inside `vendor/`.
- Does not recompute `.cargo-checksum.json` after CRAN-trim, leading to
  cargo verification failures.

`cargo-revendor` (`cargo-revendor/src/main.rs:1-32`) solves all three.

### 7.2 cargo-revendor pipeline (16 steps)

(See raw notes `/tmp/build-investigation/02-vendor-lockfile.md:30-65` for
line-anchored detail.) The headline steps:

1. **Bootstrap-seed** for frozen manifests so `cargo metadata` works.
2. **Auto-detect** `[patch."git+url"]` config to find monorepo siblings.
3. **`cargo metadata`** for graph.
4. **`cargo package`** for local crates (resolves `*.workspace = true`).
5. **`cargo vendor`** for external deps (with temporary `[patch.crates-io]`
   pointing at local crates so vendor can resolve them).
6–9. Extract local crate archives, strip CRAN-disallowed dirs, rewrite path
   cross-refs to flat sibling layout, recompute `.cargo-checksum.json`
   (preserving the lockfile-matching `package` hash).
10–12. Move staging → `vendor/`, write `.cargo/config.toml`, copy Cargo.lock
   into `vendor/Cargo.lock` for `--freeze` reuse.
6. **Freeze** (optional, **not used in current flow**).
7. **Compress** with `COPYFILE_DISABLE=1` (skip macOS xattrs).
8. Save cache files.

### 7.3 ⚖️ `--freeze`, `--sync`, `--versioned-dirs`

| Flag | Status | What it does |
|---|---|---|
| `--freeze` | **Not used** anywhere | Rewrites Cargo.toml `[dependencies]` to `path = "../../vendor/..."`. Was the old design; mutates source files, causes VCS churn. `docs/CRAN_COMPATIBILITY.md:221-228` explicitly lists this as removed. |
| `--sync` | **Not used** anywhere | Mirrors `cargo vendor --sync` — unions multiple disjoint workspaces. Available but no current consumer. |
| `--versioned-dirs` | **On by default** since 2026-04-18 | Vendor entries as `vendor/<name>-<version>/` instead of flat `vendor/<name>/`. Tracked by #239. Use `--flat-dirs` to revert. |

**Cleanup**: CLAUDE.md's "Stale frozen-vendor recovery" section
is historical (recovery from a flow we don't use). Should be deleted or
marked archived to prevent confusion.

### 7.4 Two Cargo.lock files

| File | Tracked? | What for |
|---|---|---|
| **`Cargo.lock` (repo root)** | YES | Workspace lock for framework dev (miniextendr-api, miniextendr-macros, etc.). Not relevant to CRAN. |
| **`rpkg/src/rust/Cargo.lock`** | YES | CRAN-relevant lock for the rpkg's rust crate. Must be "tarball-shape". |

### 7.5 "Tarball-shape" lock

The rpkg lock must satisfy:

- **No `source = "path+..."` lines** for framework crates. (Path sources are
  dev-mode artifacts — they encode the developer's local filesystem path.)
- `source = "git+https://github.com/A2-ai/miniextendr#<sha>"` or absent for
  framework crates.
- `checksum = "..."` lines are **allowed** since PR #408 (which made
  `.cargo-checksum.json` files carry valid checksums).

Enforcement: `tools/lock-shape-check.R` (called by configure in tarball mode);
`just lock-shape-check`; pre-commit hook.

### 7.6 🐞 `bootstrap.R` does not regenerate the lock

`just vendor` does three things:

1. Move `.cargo/config.toml` aside (so the patch override isn't active).
2. Delete `Cargo.lock` and run `cargo generate-lockfile` (bare URLs).
3. Restore `.cargo/config.toml` and run `cargo-revendor`.

`bootstrap.R` does only step 3. **If a developer has a source-shape (dirty)
lock on disk and runs `devtools::build()`**, bootstrap.R runs cargo-revendor
against that dirty lock. The resulting tarball ships a source-shape lock with
`path+file:///...` entries for miniextendr crates. Offline install of that
tarball fails with the "requires a lock file" error.

Why it doesn't break in practice: the pre-commit hook blocks source-shape locks,
so PRs always have tarball-shape locks; `just r-cmd-build` depends on `just vendor`
which regenerates. But the path `devtools::build()` → `rcmdcheck()` →
`r-lib/actions/check-r-package` goes through bootstrap.R and is exposed.

**Fix**: bootstrap.R should perform the same regenerate-the-lock dance as
`just vendor` before invoking cargo-revendor. Or delegate to a shared shell
helper invoked by both.

### 7.7 🐞 `miniextendr_vendor()` inconsistent with `just vendor`

The R-side `miniextendr_vendor()` (`minirextendr/R/workflow.R:155-`, `vendor.R:`)
clears `.cargo-checksum.json` to `{"files":{}}` (`vendor.R:137-142`) and strips
`checksum =` lines from Cargo.lock (`workflow.R:169-173`).

After PR #408, both behaviors are wrong. cargo-revendor recomputes valid
checksums in `.cargo-checksum.json` *and* the lock retains checksums. Two paths
producing different vendor trees is a recipe for "works on maintainer machine,
fails for scaffolded package user."

**Fix**: align `miniextendr_vendor()` with `just vendor` — either by
shelling out to `cargo-revendor` and not post-processing, or by removing the
clear-and-strip steps. Likely the former.

### 7.8 Who creates, consumes, deletes the tarball

| Phase | Creator | Consumer | Deleter |
|---|---|---|---|
| Maintainer prep | `just vendor` | next configure | trap EXIT on `r-cmd-build`/`r-cmd-check`/`devtools-build` |
| Build via pkgbuild | `bootstrap.R` (in source dir) | `R CMD build` (seals it into tarball) | trap EXIT |
| End-user GitHub install | configure auto-vendor (if cargo-revendor) | same configure | n/a (temp dir) |
| End-user tarball install | (already in tarball) | configure | n/a (temp dir) |
| CRAN offline farm install | auto-vendor canNOT fire (no cargo-revendor) | falls to source mode | n/a — fails loudly. **Intended canary.** |

### 7.9 ❓ Vendor-sync only checks `src/`, not `Cargo.toml`

`just vendor-sync-check` (`justfile:935-972`) diffs `<crate>/src` only, not
`Cargo.toml`. A Cargo.toml change (new dep) wouldn't be caught — but a new dep
forces a Cargo.lock change which forces a full re-vendor, so the next CI run
catches it. Defensible but worth knowing.

---

## 8. Installer Surfaces

Every installer ends at `./configure` + `make`. They differ in: whether they
stage to a temp dir, whether they call bootstrap.R, and whether `.git` is
present in the working tree.

### 8.1 Call-chain matrix

| Installer | Stage? | bootstrap.R? | `.git` in workdir? | Resulting mode |
|---|---|---|---|---|
| `R CMD INSTALL <path>` | No | No | Yes (dev) | Source + monorepo patch |
| `R CMD INSTALL <tarball>` | Yes | No | No | Tarball (offline) |
| `R CMD build <path>` | Yes (build stage) | Yes | Yes (bootstrap runs in source dir) | bootstrap produces tarball |
| `R CMD check <tarball>` | Yes | No | No | Tarball (offline) |
| `devtools::install(path)` | Yes (tarball stage) | Yes | Yes (bootstrap) → No (install) | Tarball (offline) |
| `devtools::load_all(path)` | No | No | Yes | Source + monorepo patch |
| `devtools::document(path)` | No | No | Yes | Source + monorepo patch |
| `devtools::build(path)` | Yes | Yes | Yes (bootstrap) | Tarball produced |
| `devtools::check(path)` | Yes | Yes | Yes (bootstrap) → No (check) | Tarball check |
| `pkgbuild::build(path)` | Yes | Yes | same as `devtools::build` | Tarball produced |
| `remotes::install_local(path)` | Yes (tarball stage) | Yes | Yes → No | Tarball (offline) |
| `remotes::install_local(tarball)` | Yes (extract) | No | No | Tarball (offline) |
| `remotes::install_github("user/repo")` | Yes (archive download, no .git) | n/a (archive lacks bootstrap trigger) | No | Auto-vendor if cargo-revendor on PATH, else source-network |
| `pak::pkg_install("github::...")` | Yes (callr subprocess) | n/a | No | same as install_github |
| `pak::local_install(path)` | Yes (tarball stage) | Yes | Yes → No | Tarball (offline) |
| `install.packages(repos=NULL, pkgs=tarball)` | Yes (extract) | No | No | Tarball (offline) |

### 8.2 What R staging strips

When `R CMD INSTALL <tarball>` extracts:

- Temp dir is typically `$TMPDIR/RtmpXXX/R.INSTALL<hash>/<pkg>/`.
- Includes everything in the tarball (`inst/vendor.tar.xz`, `tools/`,
  `configure`, `src/`).
- Excludes (because never in the tarball): `.git`, `target/`, `rust-target/`.

When `R CMD build` stages:

- Temp dir is `$TMPDIR/RtmpXXX/R.build<hash>/<pkg>/`.
- Excludes per `.Rbuildignore` + R defaults: `.git`, `*.o`, `*.so`, `target/`.

### 8.3 Why staging breaks `.git`-ancestor probe — by design

`configure.ac:62-70` walks up from the package source dir looking for `.git`.
Under staging, no `.git` ancestor exists → `SOURCE_IS_GIT=false`. This is the
correct trigger for auto-vendor: archives extracted into temp dirs don't have
`.git`. Source trees do.

### 8.4 ❓ pak subprocess env propagation

pak runs in a callr subprocess. `CARGO_HOME` and similar set in the parent R
session don't reliably propagate. Not known to be broken in practice but
untested. ❓ Worth a smoke test (see test plan §T9).

### 8.5 rv — orthogonal

`rv` is a renv-analog written in Rust. The `rv/` directory in this repo is the
rv-managed R library for *this repo's R dependencies*, not a build-system
component. It does not touch the Rust toolchain. End-user packages are
unaffected. The `rproject.toml` at root defines dependencies for the dev
environment.

---

## 9. Staging — what works and what doesn't

### 9.1 What's preserved across staging

- Environment variables (CARGO_HOME, PATH, R_HOME).
- The full tarball contents.

### 9.2 What's lost across staging

- `.git` ancestry. Used by SOURCE_IS_GIT — but its loss is the intent.
- Any `target/` or cargo build state — tarball install always full-recompiles.
- Symlinks (R CMD build follows or omits them based on `.Rbuildignore`).

### 9.3 The `CARGO_TARGET_DIR` ephemerality

Configure bakes `CARGO_TARGET_DIR=<abs_top_srcdir>/rust-target` into Makevars
(`configure.ac:229-233`). Under staging, `<abs_top_srcdir>` is the temp dir →
target/ lives inside the temp dir → discarded when install completes. Tarball
installs cannot benefit from a warm target/ between calls.

❓ Possible mitigation: configure could honor a user-set `CARGO_TARGET_DIR`
env var pointing to a stable user-level cache (e.g., `tools::R_user_dir()`).
Requires care because the cleanup `rm -rf "$(CARGO_TARGET_DIR)"` in tarball
mode would clobber a shared cache.

### 9.4 Bootstrap.R runs in source dir, not staging

`Config/build/copy-method: "none"` (default) means pkgbuild's `callr::rscript`
runs against the **original source directory**, not the build staging dir.
That's why bootstrap.R sees `.git` in dev — and why it must explicitly run
`cargo-revendor` instead of relying on configure's auto-vendor block (which
would never fire because `SOURCE_IS_GIT=true`).

---

## 10. Incremental Compilation

### 10.1 The cargo target dir

| Context | `CARGO_TARGET_DIR` |
|---|---|
| Dev (`R CMD INSTALL <path>`, `devtools::load_all`, `devtools::document`) | `<package_root>/rust-target` — persistent across calls |
| Tarball install (always) | `<temp_dir>/rust-target` — ephemeral |

Persistent target/ across dev calls means `devtools::load_all` and
`devtools::document` are **truly incremental**: cargo fingerprints in
`rust-target/release/.fingerprint/` detect no source changes → no recompile.

### 10.2 The cargo home

`CARGO_HOME` is **not** set by configure. Default is `~/.cargo`. Shared across
all projects on the machine. Registry downloads cached once.

❓ Configure doesn't accept a `CARGO_HOME` override. End users on locked-down
or read-only `$HOME` have no standard path. Adding `AC_ARG_VAR([CARGO_HOME])`
- propagating to make env would close this.

### 10.3 What incrementality looks like in practice

- **`devtools::load_all` repeat call, no Rust changes**: cargo runs (because
  FORCE_CARGO is phony), checks fingerprints, exits in ~1 s. No recompile.
- **`devtools::document` repeat call**: same — but **roxygen2 always re-parses
  all 27k lines of `R/<pkg>-wrappers.R`**. ❓ This is a roxygen2 limitation
  (no per-file cache), not a build-system one. Mention but don't fix here.
- **`devtools::install` repeat call**: each call → fresh tarball staging dir
  → full cargo recompile. The "warm" case is no faster than the "cold" case.
  **By design**; documented workaround is `load_all` for iterative dev.
- **`devtools::test`**: calls `compile_dll` (same as `load_all`) → incremental.
- **`rcmdcheck` / CI check**: always tarball; always full recompile. sccache
  saves CI repeats; doesn't help local.

### 10.4 sccache (CI only)

CI sets `RUSTC_WRAPPER=sccache` and `CARGO_INCREMENTAL=0`. Without incremental,
rustc outputs are deterministic → sccache hits approach 100%. Not configured
for local dev.

### 10.5 ❓ What would `devtools::install` incrementality look like?

Difficult: tarball staging is R's own behavior, not the build system's. Three
plausible mitigations:

1. **Stable `CARGO_TARGET_DIR` via env**: end user exports
   `CARGO_TARGET_DIR=~/.cache/miniextendr-target` before install. Configure
   already honors this if set. The cleanup `rm -rf "$(CARGO_TARGET_DIR)"`
   would still clobber it — guard with "skip cleanup if CARGO_TARGET_DIR
   not under abs_top_srcdir."

2. **Skip wrapper-gen cdylib**: §6.4 — saves 10–30 s per install regardless.

3. **Configure-driven persistent cache**: opt-in flag `--with-persistent-cache`
   that picks a path under `tools::R_user_dir("miniextendr-cache")` and skips
   the cleanup. Complexity not obviously worth it.

---

## 11. Test Surface — what we need to be able to verify

(Full test plan in `analysis/build-system-tests-2026-05-11.md`.) Headline cases:

- **Fresh-clone smoke**: every installer × every project structure → success.
- **Warm dev iteration**: `load_all` / `document` / `install` repeat → no
  unnecessary recompile, no re-download.
- **Tarball install simulates CRAN**: works offline, vendor.tar.xz inside,
  no cargo-revendor required.
- **No-cargo-revendor end-user**: source-mode network install works.
- **Latch leak detection**: vendor.tar.xz left in source tree blocks `just`
  recipes; bare `R CMD INSTALL` produces a clearly-failed install or warns.
- **Bootstrap.R lock regeneration**: dirty lock + `devtools::build` ships a
  working tarball (currently fails — see §7.6).
- **Inconsistency**: `miniextendr_vendor()` and `just vendor` produce the
  same vendor tree, byte-for-byte (currently fails — see §7.7).
- **Wrapper-gen skip** (when implemented): tarball install does not invoke
  the cdylib build path.
- **Cross-installer parity**: same inputs through `devtools::install` /
  `remotes::install_local` / `pak::local_install` produce identical installed
  packages.
- **Both project structures pass the same matrix** — no path is "monorepo
  only" or "standalone only."

---

## 12. Decisions Required

### 12.1 ⚡ Skip wrapper gen in tarball mode

**Decision**: implement. Pre-commit hook + atomic tarball construction make
this safe by construction. Saves 10–30 s per tarball install.

**Open**: should there be an env-var escape hatch (`MINIEXTENDR_FORCE_WRAPPER_GEN=1`)
to regenerate during debug? Probably yes, costs nothing.

### 12.2 🐞 Fix bootstrap.R Cargo.lock regeneration

**Decision**: bootstrap.R should run the same regenerate-lock dance as
`just vendor`. Extract into a shared helper (Rscript or shell) called by
both. Without this, `devtools::build()` from a dirty workspace produces a
broken tarball.

### 12.3 🐞 Reconcile `miniextendr_vendor()` with `just vendor`

**Decision**: either delegate `miniextendr_vendor()` to `cargo-revendor`
directly (preferred — single source of truth), or remove the
`clear-checksum.json` and `strip-checksums-from-lock` steps. Document the
chosen approach.

Decision: Delegate to cargfo-revendor

### 12.4 🐞 r_shim.h gap in monorepo template

**Decision**: copy `r_shim.h` in `create_rpkg_subdirectory()`. Trivial.

### 12.5 🐞 Monorepo-aware upgrade + release workflow

**Decision**: `upgrade_miniextendr_package(path)` should resolve to the rpkg
subdir; `use_release_workflow(path)` should accept a `rpkg_subdir` parameter
and parameterize the YAML.

### 12.6 ⚖️ Delete or archive "stale frozen-vendor recovery" docs

**Decision**: delete from CLAUDE.md (it describes recovery from a flow we
don't use). Reduces confusion for new contributors.

### 12.7 ⚖️ CLAUDE.md `-Wl,--whole-archive` / `-force_load` claim

**Decision**: update to describe the `stub.c` anchor + `codegen-units = 1`
design.

### 12.8 ❓ `miniextendr.yml` placement in monorepo

**Open**: workspace root vs. rpkg subdir? Logical home is rpkg subdir.
Verify with maintainer.

### 12.9 ❓ `CARGO_HOME` override in configure

**Open**: add `AC_ARG_VAR([CARGO_HOME])` for end users on restrictive systems?
Low cost, occasionally requested.

### 12.10 ❓ Skip configure on no-op runs

**Open**: gate `just devtools-document`'s configure dependency on Makevars
mtime? Trade-off: harder to debug stale state if guard fails.

### 12.11 ❓ Replace configure.ac sed-based git-URL extraction

**Open**: `configure.ac:450` uses sed line-regex. cargo-revendor uses
toml_edit. Fragile but no known failure in current Cargo.toml shape.

### 12.12 ❓ `cran-prep` template trap-cleanup (#454)

**Open**: scaffolded template lags root justfile. Should be aligned.

---

## 13. Reference: file:line index

Critical sites referenced throughout this doc:

| Where | What |
|---|---|
| `rpkg/configure.ac:62-70` | `.git` ancestor walk (SOURCE_IS_GIT) |
| `rpkg/configure.ac:72-87` | Auto-vendor self-repair (configure-time) |
| `rpkg/configure.ac:102-119` | IS_TARBALL_INSTALL latch detection |
| `rpkg/configure.ac:229-233` | CARGO_TARGET_DIR baking |
| `rpkg/configure.ac:327-360` | MONOREPO_ROOT detection (5-level walk) |
| `rpkg/configure.ac:433-498` | `.cargo/config.toml` three-branch write |
| `rpkg/configure.ac:450` | sed-based git-URL extraction (fragile) |
| `rpkg/configure.ac:551-562` | `cargo generate-lockfile` fallback |
| `rpkg/src/Makevars.in:43-71` | `all` target and post-build cleanup |
| `rpkg/src/Makevars.in:84-98` | `$(CARGO_AR)` staticlib rule |
| `rpkg/src/Makevars.in:100-133` | `$(WRAPPERS_R)` rule (skip-target for §6.4 opt) |
| `rpkg/src/Makevars.in:137-153` | `$(CARGO_CDYLIB)` rule |
| `rpkg/src/stub.c` | force-link anchor for staticlib |
| `rpkg/bootstrap.R:26-51` | pkgbuild-time auto-vendor (does not regen lock) |
| `miniextendr-api/src/registry.rs:32-104` | Eight distributed_slice declarations |
| `miniextendr-api/src/registry.rs:393` | ALTREP-skip during wrapper-gen |
| `miniextendr-api/src/registry.rs:468-497` | `collect_r_wrappers()` |
| `miniextendr-api/src/registry.rs:721-936` | `write_r_wrappers_to_file()` |
| `miniextendr-api/src/registry.rs:918-921` | Write-if-changed (early-exit no-op) |
| `miniextendr-macros/src/lib.rs:2589` | `miniextendr_force_link` static (anchor target) |
| `cargo-revendor/src/main.rs:442-766` | Full revendor pipeline |
| `cargo-revendor/src/vendor.rs:781-818` | `.cargo/config.toml` writer |
| `cargo-revendor/Cargo.toml:11-12` | Standalone workspace decision |
| `minirextendr/R/workflow.R:155-` | `miniextendr_vendor()` (the inconsistent R path) |
| `minirextendr/R/workflow.R:169-173` | Lock checksum stripping (legacy, wrong post-#408) |
| `minirextendr/R/vendor.R:137-142` | `.cargo-checksum.json` cleared to `{"files":{}}` |
| `minirextendr/R/create.R:178-270` | `create_rpkg_subdirectory()` (the monorepo path missing r_shim.h) |
| `minirextendr/R/use-rust.R:54-62` | `use_miniextendr_stub()` (the standalone path that copies r_shim.h) |
| `minirextendr/R/upgrade.R:27` | `upgrade_miniextendr_package()` (rpkg-only) |
| `minirextendr/R/use-release-workflow.R:26` | `use_release_workflow()` (not monorepo-aware) |
| `minirextendr/R/doctor.R:87-138` | latch-leak detection |
| `minirextendr/R/clean-vendor-leak.R:26-42` | R-side latch-leak recovery |
| `justfile:367-370` | `configure` recipe |
| `justfile:393-428` | `vendor` recipe (regenerate-lock + cargo-revendor) |
| `justfile:437-444` | `clean-vendor-leak` recipe |
| `justfile:452-471` | `_assert-no-vendor-leak` (dev-recipe guard) |
| `justfile:499-506` | `devtools-load`, `devtools-install` |
| `justfile:538-551` | `devtools-build`, `devtools-check` (trap cleanup) |
| `justfile:568-616` | `r-cmd-install`, `r-cmd-build`, `r-cmd-check` |
| `justfile:935-972` | `vendor-sync-check` (src-only diff) |
| `patches/templates.patch:1-65` | bootstrap.R deltas (monorepo template) |
| `patches/templates.patch:77-125` | self-repair block stripped from monorepo |
| `patches/templates.patch:173-178, 393-400` | CARGO_FEATURES defaults |
| `.githooks/pre-commit` | wrappers.R + NAMESPACE staged-together enforcement |
| `docs/CRAN_COMPATIBILITY.md:113-116` | "requires a lock file" error condition |
| `docs/CRAN_COMPATIBILITY.md:221-228` | `--freeze` removed from codebase |

---

## 14. Things NOT investigated here

- WebR/wasm specifics beyond noting wasm path skips wrapper gen (`Makevars.in:116-118`).
- The `tests/cross-package/` trait-ABI tests (producer.pkg / consumer.pkg) — relevant
  for vctrs/trait dispatch correctness, not for the build-system question proper.
- Per-class-system codegen details (R6/S3/S4/S7/Env/Vctrs) — covered by
  `docs/CONVERSION_MATRIX.md` and per-subtree `CLAUDE.md`.
- Site / docs sync (`scripts/docs-to-site.sh`) — separate pipeline.
- The `cargo-revendor` cache layer (`cache.rs`) beyond noting it exists.

---

## 15. Out-of-scope opinions (flag for discussion)

- **`Config/build/bootstrap: TRUE` is load-bearing infrastructure** and the
  pipeline assumes pkgbuild ≥ a certain version. Worth pinning in
  `SystemRequirements` or DESCRIPTION? Not currently checked.
- **The `_assert-no-vendor-leak` guard exists only for `just` recipes.**
  Adding an equivalent at the R level (called from `miniextendr_build()` /
  `miniextendr_check()`) would cover the bare-devtools path too.
- **`R CMD check <path>` is allowed but disrecommended.** No code blocks it.
  Worth a `tools/` script warning if invoked? Trade-off: surprise for users
  who didn't read CLAUDE.md.
