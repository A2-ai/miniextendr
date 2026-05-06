# miniextendr

Rust-R interoperability framework for building R packages with Rust backends.

## Principles

- **No backwards compatibility**: unreleased project — remove deprecated code, don't shim around it.
- **Simple over complex**: only make changes directly requested or clearly necessary. Trust the framework — no defensive error handling for internal invariants.
- **Fix warnings you see**: no "known issues". Every warning, lint, or test failure gets fixed, even if pre-existing and unrelated. Leave the code cleaner than you found it.
- **Deferred items = GitHub issues**: any scope cut, known limitation, or partial fix needs `gh issue create` referenced in the PR. No "out of scope" without a tracked issue.
- **`just` is maintainer-only**: end-user packages must build via `configure.ac` / `tools/*.R` / standard R mechanisms. Never require `just` in scaffolded packages.
- **`configure.ac` never mutates sources**: don't rewrite Cargo.toml / Cargo.lock / .rs during `./configure` — dirties VCS. Use `cargo revendor --freeze` at vendor time.
- **`configure.ac` must not call `minirextendr::*`**: put helpers in `tools/` and invoke via `Rscript tools/foo.R`.
- **Edit `.in` templates, not generated files**:
  - `rpkg/src/rust/.cargo/config.toml` ← `rpkg/src/rust/cargo-config.toml.in`
  - `rpkg/src/Makevars` ← `rpkg/src/Makevars.in`
  - `rpkg/src/miniextendr-win.def` ← `rpkg/src/win.def.in`
  - `rpkg/configure` ← `rpkg/configure.ac` (then `autoconf`)
  - `rpkg/src/stub.c` — static, no substitution.
- **Collect all errors in vectorized ops**: don't bail on first failure — give users one batched diagnostic.
- **Flat plans, no phases**: list work in flat priority order, not "Phase 1/2/3".
- **Prefer `From`/`TryFrom` over `as` casts**: propagate the error rather than silently truncating.

### Rust/FFI gotchas

- **Pointer provenance**: cache `*mut T` via a mutable path (`&mut T`, `Box::into_raw`, `downcast_mut`, `ptr::from_mut`). Never write through a `cached_ptr` derived from `&T` / `downcast_ref` — UB under Stacked Borrows.
- **`cargo package` for workspace resolution**: when vendoring, let `cargo package` expand workspace inheritance — never hard-code workspace dependency replacements.
- **m4 in `AC_CONFIG_COMMANDS`**: `$1` is empty (use `$0` or avoid `sh -c`). Escape `[` / `]` in sed/grep as `@<:@` / `@:>@`.
- **Windows paths in TOML**: forward slashes only. Strip `\\?\` prefix from `canonicalize()` output before writing.
- **macOS tar xattrs**: set `COPYFILE_DISABLE=1` when creating tarballs to avoid Apple metadata warnings on Linux/Windows GNU tar.
- **`cargo-revendor`**: standalone workspace (excluded from miniextendr workspace). Build/test via `just revendor-build`/`just revendor-test`. `--freeze` resolves `Cargo.toml` against `vendor/` only.
- **`ColumnarDataFrame::from_rows`: all-None columns**: columns where every row had `Option<T> = None` land as a logical NA column (LGLSXP), not `list(NULL, NULL, …)`. R coerces logical NA to the surrounding type on first use (`c(NA, 1L)` → integer, `c(NA, "x")` → character). Mixed Some/None columns are unaffected.

### `just` vs raw `cargo`

Always use `just check/clippy/test/fmt/vendor/lint` — the recipes iterate every manifest in the multi-crate workspace (root, `rpkg/src/rust`, `tests/cross-package/*`) with the correct `[patch.crates-io]` overrides. Raw `cargo --workspace` only sees the top-level manifest. If a recipe doesn't exist, say so and fall back — don't invent shortcuts.

**Interactive iteration**: prefer `cargo-limit` aliases (`cargo lcheck/lclippy/ltest/lbuild`) — they truncate output to the first few errors. Install with `just dev-tools-install`. CI and `just` recipes keep plain `cargo` (CI needs full output + `-D warnings`).

## Project Structure

```
miniextendr-api/      # Runtime (FFI, ExternalPtr, ALTREP, worker thread)
miniextendr-macros/   # Proc macros (#[miniextendr], derives; naming in src/naming.rs)
miniextendr-bench/    # Benchmarks (separate workspace member)
miniextendr-lint/     # Static analysis
miniextendr-engine/   # Code generation engine
cargo-revendor/       # Standalone cargo subcommand (not in workspace)
rpkg/                 # Example R package (installed as `miniextendr`)
minirextendr/         # Pure R scaffolding helper
tests/cross-package/  # producer.pkg / consumer.pkg — trait ABI tests
site/                 # Zola docs → GitHub Pages
background/           # Reference docs (gitignored)
```

## Build Commands

```bash
# Rust
just check / test / clippy / fmt / lint

# rpkg (R package)
just configure           # REQUIRED before any R CMD operation (dev mode)
just rcmdinstall         # Build + install (compiles Rust, auto-generates R wrappers)
just devtools-document   # roxygen2 → NAMESPACE + man/
just devtools-test       # R tests
just r-cmd-build         # Build tarball
just r-cmd-check         # Check built tarball (save to log — see Capturing Output)
just devtools-check      # Check, preserving output in rpkg-check-output/

# CRAN release prep (only step needed; configure auto-detects tarball mode)
just vendor              # regen Cargo.lock in tarball-shape, vendor deps, compress to inst/vendor.tar.xz

# Cross-package
just cross-install / cross-test / cross-check

# minirextendr
just minirextendr-install / minirextendr-test / minirextendr-check

# Site
just site-build / site-serve   # http://127.0.0.1:1111
```

### Configure is mandatory

Always `bash ./configure` (not bare `./configure` — `#!/bin/sh` causes spurious errors in `AC_CONFIG_COMMANDS` passthrough). Configure:
1. Generates `Makevars` from `.in` templates
2. Auto-detects install mode (source vs tarball) from `[ -f inst/vendor.tar.xz ]`
3. Writes `.cargo/config.toml` per mode (source: `[patch."git+url"]` for monorepo siblings or empty; tarball: `[source]` replacement to `vendored-sources`)
4. Does **not** create `inst/vendor.tar.xz` — that's `just vendor`

Package loads as `library(miniextendr)`, not `library(rpkg)`. Always check the **built tarball**, not the source dir (R CMD check on a source dir skips `Authors@R` → `Author/Maintainer` conversion).

### Install modes (source vs tarball)

| Mode | Triggered when | What configure does |
|---|---|---|
| **Source** | `inst/vendor.tar.xz` absent | No vendoring. In monorepo: writes `[patch."git+url"]` → workspace siblings. Otherwise: minimal config, cargo follows git URL. |
| **Tarball** | `inst/vendor.tar.xz` present | Unpacks tarball, writes `[source]` replacement to `vendored-sources`, build is offline. |

That's the entire decision. No `NOT_CRAN`, no `FORCE_VENDOR`, no `PREPARE_CRAN`, no `BUILD_CONTEXT`. See `docs/CRAN_COMPATIBILITY.md`.

## Development Workflow

### After Rust changes (especially macro changes)

```bash
just configure && just rcmdinstall && just devtools-document
```

Run `just devtools-document` after **anything** that affects R wrapper output: proc-macro roxygen changes, `r_wrappers.rs` / class-system codegen changes, adding/removing `#[miniextendr]` functions. Generated files (`rpkg/R/miniextendr-wrappers.R`, `NAMESPACE`, `man/*.Rd`) must be committed in sync with the Rust changes that produced them.

The pre-commit hook (`.githooks/pre-commit`) blocks commits where `*-wrappers.R` is staged without matching `NAMESPACE`. Enable once per clone: `git config core.hooksPath .githooks`.

### Adding a `#[miniextendr]` function

1. `pub fn` with `#[miniextendr]` — registration is automatic via linkme `#[distributed_slice]`, no module macro needed
2. Module must be reachable via `mod` from `lib.rs` (`#[cfg(feature = "foo")]` on `mod` declaration is sufficient for feature-gated modules)
3. `just configure && just rcmdinstall && just devtools-document`

Build sequence: `Makevars` → `cargo rustc --crate-type cdylib` → `dyn.load` + `miniextendr_write_wrappers` → `R/miniextendr-wrappers.R` → `cargo rustc --crate-type staticlib` → final `.so`.

### Reproducing CI clippy before PR

`just clippy` ≠ CI. Two CI jobs must both pass `-D warnings`:

- `clippy_default`: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `clippy_all`: same + `--features rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,vctrs,tinyvec,borsh,connections,nonapi,default-strict,default-coerce,default-r6,default-worker`

`--all-features` fails (`default-r6` and `default-s7` are mutually exclusive). CI runs a newer toolchain, so lints like `collapsible_match`, `manual_checked_ops` can fire on CI with green local. Reproduce both before pushing.

### sccache + `[profile.dev] incremental`

CI uses sccache with `CARGO_INCREMENTAL=0`. Setting `[profile.dev] incremental = false` in a workspace's `Cargo.toml` jumps sccache hit rate to ~100% (incremental builds poison cache keys with per-invocation hashes). Trade-off: loses local incremental compilation — usually worth it for CI and agent worktrees (`cargo clean` often). Not set project-wide currently — flag in PR description if changing.

### `inst/vendor.tar.xz` is not tracked

Gitignored — generated in CI (every R CMD check runs `just vendor` first) and at release time. Tracked tarballs caused binary merge conflicts, 22 MB/commit bloat, and stale-after-rebase drift. Regenerate locally with `just vendor` — cheap and deterministic from `Cargo.lock` + workspace sources.

## Key Concepts

- **Worker thread**: Rust runs on a worker thread for panic safety
- **ExternalPtr**: Box-like owned pointer over `EXTPTRSXP`. Stores `Box<Box<dyn Any>>` — thin ptr in `R_ExternalPtrAddr` → fat ptr on heap (carries `Any` vtable). Type safety via `Any::downcast`, not R symbols. Non-generic `release_any` finalizer. `cached_ptr` must have mutable provenance.
- **TypedExternal**: R-visible type name (`TYPE_NAME_CSTR` display tag, `TYPE_ID_CSTR` errors). Not authoritative for type safety — `Any::downcast` is.
- **ALTREP**: Single-struct pattern, no wrapper. Two paths:
  - *Field-based derive* — `#[derive(AltrepInteger)]` with `#[altrep(len = "field", elt = "field", class = "Name")]` generates everything
  - *Manual* — `#[altrep(manual)]` generates lowlevel traits + registration; user writes `AltrepLen` and `Alt*Data`
  - `AltrepExtract` trait abstracts data extraction (blanket impl for `TypedExternal`; override for custom storage)
  - `#[miniextendr]` on 1-field structs is **removed** — use derives
- **R_UnwindProtect**: runs Rust destructors on R errors
- **GC**: `OwnedProtect` / `ProtectScope` for RAII protect/unprotect
- **Dots (`...`)**: `_dots: &Dots`, or `name @ ...` for custom name. See `docs/DOTS_TYPED_LIST.md`.
- **typed_list!**: `#[miniextendr(dots = typed_list!(...))]` validates and creates `dots_typed`
- **`impl Trait`**: return position only (`-> impl IntoR`). Argument position fails type inference (E0283 across `let` bindings for `TryFromSexp + Trait`).
- **S4 helpers**: `slot()`/`slot<-()` live in `methods` — resolve via `getNamespace("methods")`, not `R_BaseEnv`.

### FFI thread checking (`#[r_ffi_checked]`)

On `unsafe extern` blocks, generates both checked and `*_unchecked` variants. Checked routes through `with_r_thread()` (debug assert); unchecked bypasses. Use `*_unchecked` inside ALTREP callbacks, `with_r_unwind_protect`, or `with_r_thread` blocks to skip the debug assertion. Statics pass through unchanged. `^nonapi^` variants require `#[cfg(feature = "nonapi")]`.

### Adding a new conversion type (e.g., `Box<[T]>`)

Modify in order:
1. `miniextendr-api/src/from_r.rs` — `TryFromSexp` impls (native, NA-aware, bool, String variants)
2. `miniextendr-api/src/into_r.rs` — `IntoR` impls (`Box<[T: RNativeType]>` blanket + explicit `Box<[bool]>` / `Box<[String]>`)
3. `miniextendr-api/src/coerce.rs` — `Coerce<Box<[R]>> for Box<[T]>`
4. Serde docs — `src/serde.rs`, `src/serde/de.rs`, `src/serde/traits.rs`
5. rpkg fixtures — `rpkg/src/rust/<type>_tests.rs` + `rpkg/tests/testthat/test-<type>.R`
6. `vendor_miniextendr(path = "rpkg", local_path = ".")` to sync vendor/

`bool` ≠ `RNativeType` (R uses `i32` for logicals) → separate impls. Proc-macro handles `Box<[T]>` generically via `TryFromSexp`/`IntoR` — no macro changes.

## Code Style

- **Never `mod.rs`** — use `foo.rs` + `foo/` directory. Migrate if touched.
- **Section comments**: `// region:` / `// endregion` (IDE-foldable). Migrate `// =====`, `// ──`, `// ---` when touched.

## miniextendr-lint

Build-time static analysis (runs via `build.rs` during `cargo build`/`check`). Disable with `MINIEXTENDR_LINT=0`.

- **MXL008**: trait-impl class-system compat with inherent impl
- **MXL009**: multiple impl blocks missing distinct labels → add `#[miniextendr(label = "...")]`
- **MXL010**: duplicate labels
- **MXL106**: non-`pub` function that would get `@export` → make `pub` or add `#[miniextendr(noexport)]`
- **MXL110**: parameter name is an R reserved word → codegen will break
- **MXL111**: `s4_*` method name on `#[miniextendr(s4)]` impl — codegen auto-prepends, you'll get `s4_s4_*`
- **MXL203**: redundant `internal` + `noexport`
- **MXL300**: direct `Rf_error`/`Rf_errorcall` → replace with `panic!()` (framework converts to R error)
- **MXL301**: `_unchecked` FFI outside known-safe contexts

## Common Issues

- **"could not find function"**: check `#[miniextendr]` + `pub`, module reachable from `lib.rs`, then `just configure && just rcmdinstall && just devtools-document`.
- **"configure: command not found"**: `cd rpkg && autoconf && bash ./configure`.
- **Permission errors installing**: `R_LIBS=/tmp/claude/R_lib R CMD INSTALL rpkg` or `just devtools-install`. `/tmp/claude/` is writable in sandboxes.
- **Stale `.snap.new`**: diff vs `.snap`; if expected, `mv` over the old snapshot. Re-run `just test`.
- **Segfaults**: `R -d lldb -e '…'`; at `(lldb)` type `run`, then `bt` / `frame select` / `p`.
- **Missing `.cargo/config.toml` / cargo resolves framework crates from git instead of local siblings**: stale `rpkg/inst/vendor.tar.xz` left by an interrupted `just r-cmd-build` or `r-cmd-check`. Fix: `rm rpkg/inst/vendor.tar.xz && just configure`. `minirextendr_doctor()` detects both conditions.

## Capturing Command Output

Redirect long-running R/cargo commands to a log, then **Read tool** the log — never `tail` / `head`:

```bash
just <recipe> 2>&1 > /tmp/<name>.log
```

Common: `devtools-doc.log`, `rcmdinstall.log`, `rcmdcheck.log`, `devtools-test.log`, `vendor.log`, `devtools-check.log`, `minirextendr-test.log`. `rpkg-check-output/miniextendr.Rcheck/` has `00check.log`, `00install.out`, `tests/`.

## Sandbox Restrictions

Claude Code sandbox blocks compilation. For any compiling command (`just devtools-document`, `rcmdinstall`, `cargo build`, `R CMD INSTALL/check`), pass `dangerouslyDisableSandbox: true`.

## File Deletion Safety

- **Never `rm`** or permanent-delete in automated workflows.
- Use trash (`trash`, `gio trash`, platform equivalent).
- No trash utility available? Stop and ask.

## Agent Worktrees

- Run agents in worktrees (`isolation: "worktree"`) to avoid collisions.
- **Merge**: rebase worktree onto current main, *then* merge. Rebase must happen immediately before the merge, not when the agent finishes.
- **Sequential merging** of multiple worktrees: rebase → merge → rebase next → merge. Each rebase must see prior merge commits on main, otherwise changes get silently overwritten.
- **Never copy whole files** worktree → main — rebase/merge is the only correct flow.
- If agent didn't commit, commit in the worktree first.
- Don't delete a worktree until its branch is pushed or merged.
- **Clean up after push**: `git worktree remove -f -f .claude/worktrees/<name>` + `git worktree prune`. Each worktree holds a full `target/` (2–3 GB/agent).
- **Rebase conflicts**: plain `git rebase origin/main`. NEVER `-X theirs` blanket — drops main's changes to shared files (justfile, lockfiles, etc.). Only use `git checkout --theirs rpkg/inst/vendor.tar.xz && git add` for the binary tarball. Resolve everything else by hand. Regenerate tarball with `just vendor` and amend into the vendor-refresh commit.

## Sync Checks

### Vendor sync

`just vendor-sync-check` verifies vendored copies match workspace sources; `just vendor` refreshes.

**Stale vendor freeze recovery**: `just vendor --freeze` writes `path = "../../vendor/..."` into `rpkg/src/rust/Cargo.toml` `[dependencies]` and `[patch.crates-io]`. After merging main, the frozen vendor/ can go stale and `cargo metadata` fails. Fix: reset frozen path deps back to `"*"`, delete `rpkg/vendor/` + `rpkg/src/rust/Cargo.lock`, run `just configure`.

### Template sync (rpkg → templates)

`minirextendr/inst/templates/` are **derived from rpkg** (master source).

1. Apply changes to `rpkg/` first
2. Port to `minirextendr/inst/templates/`
3. `just templates-approve` locks the delta

`just templates-check` verifies no unexpected drift. Approved delta is recorded in `patches/templates.patch` — templates may have extra standalone-project logic (checking for miniextendr-api before using path overrides, running `cargo vendor` for transitive deps).

## Documentation Site

Zola static site in `site/` → GitHub Pages at `https://a2-ai.github.io/miniextendr/`. Content pages are TOML-frontmatter markdown (`+++`). `weight` controls sort order.

GitHub Actions auto-deploys on push to `main` when `site/**`, `docs/**`, or `*/src/**` changes: runs `scripts/docs-to-site.sh` → builds nightly rustdoc (`--document-private-items --document-hidden-items --show-type-layout --enable-index-page --generate-link-to-definition -Z rustdoc-map`) → builds Zola → copies rustdoc to `site/public/rustdoc/` → deploys. Rustdoc index at `.../rustdoc/`; individual crates at `.../rustdoc/miniextendr_api/` etc.

`site/content/manual/` is **auto-generated from `docs/`** by `scripts/docs-to-site.sh` (1:1 conversion, not curated summaries). Edit `docs/*.md` only — never edit `site/content/manual/*.md` directly. The generator derives frontmatter (title + description) from each doc's `# Heading` and first paragraph. `site/content/_index.md` and anything outside `manual/` are hand-authored and must be edited directly.

**After editing `docs/`, regenerate and commit both together**: `just site-docs && git add docs/ site/content/manual/ site/data/plans.json`. CI runs `docs-to-site.sh` itself before each deploy, so the live site is always correct — but the in-repo `site/content/manual/` drifts out of sync if you skip the regenerate step, making subsequent diffs noisy and masking unrelated site edits.

### Site scripts

| Script | What it does |
|--------|-------------|
| `scripts/docs-to-site.sh` | Converts `docs/*.md` → `site/content/manual/` (frontmatter derived from heading + first paragraph) |
| `scripts/plans-to-json.sh` | Emits `site/data/plans.json` — title/summary/URL for each `plans/*.md`, consumed by the Zola roadmap template |
| `scripts/bump-version.sh <version>` | Bumps `version =` / `Version:` across all Cargo.toml and DESCRIPTION files (workspace + rpkg + minirextendr + cross-package) |

### Just recipes

| Recipe | What it runs |
|--------|-------------|
| `just site-docs` | `docs-to-site.sh` + `plans-to-json.sh` — run before `site-build`/`site-serve` when previewing doc or plan changes locally |
| `just site-build` | `zola build` only (output in `site/public/`) |
| `just site-serve` | `zola serve` only (http://127.0.0.1:1111) |
| `just bump-version <v>` | `bump-version.sh <v>` — use before a release commit |

## Reference Docs (`background/`, gitignored)

- **Official R**: `R Internals.html`, `Writing R Extensions.html`, `ALTREP_...html`, `Autoconf.html`, `GNU make.html`
- **R source**: `r-source-tags-R-4-5-2/` — key paths `src/include/Rinternals.h`, `src/include/R_ext/Altrep.h`, `src/main/altclasses.c`, `src/main/memory.c`
- **Reference R packages**: `S7-main/`, `R6-main/`, `vctrs-main/`, `roxygen2-main/`, `mirai-main/`
- **ALTREP examples**: `Rpkg-mutable-master/`, `Rpkg-simplemmap-master/`, `vectorwindow-main/`

**Always check `background/` for R API details before guessing.**

## Reviews & Plans

- **Reviews** (`reviews/*.md`): when things go wrong (test/CI failure, runtime error, unexpected behavior), write a short file: *what was attempted*, *what went wrong*, *root cause*, *fix*. Accumulates institutional knowledge on non-obvious failure modes.
- **Vendor audit**: when deps change, audit `vendor/` for crates worth integrating — write a `plans/` file for any candidate (e.g., R-relevant error types, serialization, data structures).

## Using Codex for Reviews

Use `codex exec` for non-interactive (bare `codex` needs a TTY and fails under Bash tool):

```bash
codex exec -m gpt-5.3-codex --full-auto "prompt"
codex exec -m gpt-5.3-codex review "review these changes"
```

## Compaction

Between **200k–400k tokens**, run `/compact` before auto-compaction forces it. Compact sooner during exploration; later during mid-refactor when recent turns are load-bearing.

**Preserve**: current goal, active PRs/branches, file:line for uncommitted in-progress edits, unresolved review comments / test failures, session feedback not yet in `CLAUDE.md` or memory.

**Discard**: tool-output dumps, file contents already at their final edited form, search results, anything already in a commit message or memory file.
