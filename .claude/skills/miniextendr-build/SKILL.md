---
name: miniextendr-build
description: Use when debugging configure.ac failures, Makevars issues, Cargo.lock mismatches, vendor tarball problems, build mode confusion, or the cdylib-to-staticlib double-link pipeline. Also use when changing a Makevars value, diagnosing a latch-leak, understanding the bash-vs-sh configure requirement, or working with m4 quoting in configure.ac.
---

# miniextendr Build System

miniextendr bridges R's package build system and Cargo via a configure script,
Makevars template, and a double-link pipeline that generates R wrappers during
the build itself. This skill covers the full build pipeline and the install-mode
latch that switches between source and CRAN tarball builds.

## When to use this skill

- "Why does `./configure` fail but `bash ./configure` work?"
- "My build picks the tarball vendor when it shouldn't."
- "I'm getting a Cargo.lock mismatch error."
- "I changed a Makevars value — what's the regen flow?"
- "What are the cdylib and staticlib builds and why are there two?"
- "Where does `.cargo/config.toml` come from?"
- "What is the install-mode latch and how does it leak?"
- "How do I add a cargo feature flag to the build?"
- "What are the m4 quoting pitfalls in configure.ac?"
- "Why does `just check`/`build`/`test` keep rewriting `rpkg/src/rust/Cargo.lock`?"
- "How do I verify the committed R wrappers / NAMESPACE / man are in sync?"

## Key concepts

### The double-link pipeline

The most unusual aspect of the build: two sequential cargo invocations produce
two different artifacts, and R wrapper generation happens between them.

```
configure.ac  →  configure  →  src/Makevars
                                    |
             cargo rustc --crate-type staticlib  (warm cache first)
                                    |
             cargo rustc --crate-type cdylib
                                    |
                   Makevars: dyn.load cdylib in R
                                    |
             miniextendr_write_wrappers()    ← miniextendr-api/src/registry.rs
             miniextendr_write_wasm_registry()
                                    |
                   R/miniextendr-wrappers.R  (generated, do not hand-edit)
                                    |
                        final .so  (links staticlib)
```

The cdylib phase boots enough of the Rust runtime inside R to walk the
`MX_R_WRAPPERS` distributed_slice (declared in `miniextendr-api/src/registry.rs`)
and emit the R-side `.Call()` wrappers. The staticlib phase then relinks the
same code as the final installed shared object. `miniextendr_write_wrappers`
lives in `miniextendr-api/src/registry.rs` — not in miniextendr-engine.

`stub.c` provides the minimal C translation unit that R's build system requires.
It declares `extern const char miniextendr_force_link`, which references a symbol
emitted by `miniextendr_init!()`. With `codegen-units = 1` in `Cargo.toml`, this
pulls the entire user crate out of the staticlib archive, carrying all
`#[distributed_slice]` entries — no `-force_load` or `--whole-archive` needed
in user packages.

### configure.ac mechanics

#### `bash ./configure`, not `./configure`

The generated configure script uses `#!/bin/sh` as its shebang (required by
autoconf's portability model). The `AC_CONFIG_COMMANDS` blocks write
`.cargo/config.toml` inline by emitting shell commands via `echo`. Under some
shells invoked as `sh`, the passthrough of those commands produces spurious
errors or mis-executes. Invoking the script explicitly with `bash` avoids the
problem on all platforms. Always use `bash ./configure` or the `just configure`
recipe.

This is a shell-passthrough issue with `AC_CONFIG_COMMANDS`, not a dash
incompatibility — macOS `/bin/sh` is bash. The symptom is spurious failures in
the `cargo-config` or `unpack-vendor-tarball` command blocks.

#### What configure does, in order

1. Verifies `DESCRIPTION` and `NAMESPACE` exist (guards against running from
   the monorepo root instead of `rpkg/`).
2. Self-repair: if `inst/vendor.tar.xz` is absent, no `.git` ancestor exists,
   and `cargo-revendor` is on PATH, invokes `cargo revendor` to produce the
   tarball. Skipped in git-tracked source trees to keep `just configure` fast.
3. Detects install mode from `[ -f inst/vendor.tar.xz ]`.
4. Discovers `cargo`, `rustc`, `sed`. Enforces rustc 1.85+ (edition 2024).
5. Detects webR/wasm32 via `CC=emcc`.
6. Resolves monorepo siblings by walking parent directories for
   `miniextendr-api/Cargo.toml`.
7. Writes `src/Makevars` from `src/Makevars.in` via `AC_CONFIG_FILES`.
8. Writes `src/miniextendr-win.def` from `src/win.def.in` via `AC_CONFIG_FILES`.
9. Writes `src/rust/.cargo/config.toml` **inline** via `AC_CONFIG_COMMANDS`
   (three variants: tarball / source+monorepo / source-only). There is no
   `.in` template for this file — configure emits it directly.
10. Unpacks `inst/vendor.tar.xz` in tarball mode.
11. Validates Cargo.lock shape and compatibility.

#### m4 quoting pitfalls

m4 treats `[` and `]` as quoting characters throughout `configure.ac`. Literal
brackets in shell code or sed expressions inside `AC_CONFIG_COMMANDS` must be
escaped as `@<:@` (for `[`) and `@:>@` (for `]`). This applies everywhere in
the file, not only in m4 macro arguments. Forgetting this in a sed character
class produces silent wrong output or m4 parse errors.

Also: inside `AC_CONFIG_COMMANDS`, `$1` is empty (it refers to the command
name, not a shell positional). Use `$0` or avoid `sh -c` constructs.

#### PACKAGE_NAME vs PACKAGE_TARNAME

`PACKAGE_NAME` preserves dots (e.g., `producer.pkg`). `PACKAGE_TARNAME`
lowercases and normalizes. When deriving C or Rust identifiers (e.g.,
`CARGO_STATICLIB_NAME`), configure converts both hyphens and dots to
underscores. If only hyphens are replaced, package names with dots produce
broken symbol names.

### .in templates that exist on disk

`AC_CONFIG_FILES` in `configure.ac` drives substitution for:

- `rpkg/src/Makevars.in` → `rpkg/src/Makevars`
- `rpkg/src/win.def.in` → `rpkg/src/miniextendr-win.def`

The `configure` script itself is generated from `rpkg/configure.ac` by running
`autoconf`. If you edit `configure.ac`, re-run `autoconf` from the `rpkg/`
directory to regenerate `configure`.

`rpkg/src/stub.c` is static — no substitution, no `.in` counterpart.

`rpkg/src/rust/.cargo/config.toml` is written inline by `AC_CONFIG_COMMANDS`
in configure.ac. There is no `cargo-config.toml.in` file in the repository.
Do not edit `config.toml` directly; re-run `bash ./configure` to regenerate it.

### The install-mode latch

`rpkg/inst/vendor.tar.xz` is the single signal that flips configure into
tarball (CRAN offline) mode.

| Mode | Triggered when | What configure writes to .cargo/config.toml |
|------|----------------|---------------------------------------------|
| Source (monorepo) | tarball absent, `.git` ancestor found, monorepo siblings detected | `[patch."git+url"]` → local workspace crates |
| Source (standalone) | tarball absent, no monorepo siblings | `[build] target-dir = ...` only; cargo follows git URL |
| Tarball | tarball present | `[source.crates-io] replace-with = "vendored-sources"` + git-source replacements + `[source.vendored-sources] directory = ...` |

The tarball has been gitignored since 2026-04-18. CI regenerates it per-build
via `just vendor`. Locally, `just r-cmd-build` and `just r-cmd-check` produce
the tarball transiently and trap-clean on exit.

Three layered triggers converge on the latch:
1. Maintainer's explicit `just vendor` / `miniextendr_vendor()`.
2. `bootstrap.R` (invoked by pkgbuild during `devtools::build()`, `rcmdcheck`,
   `r-lib/actions/check-r-package`) — runs configure in a staging directory
   that has no `.git` ancestor, triggering auto-vendor.
3. End-user install of a tarball that shipped without vendored dependencies —
   configure auto-vendors at install time.

CRAN's offline build farm does not have `cargo-revendor`, so the auto-vendor
branch is short-circuited there. A maintainer who ships a tarball without
`inst/vendor.tar.xz` inside fails CRAN's offline check loudly; this is the
intended canary. There is no `NOT_CRAN`, `FORCE_VENDOR`, or `PREPARE_CRAN`
escape — see `docs/CRAN_COMPATIBILITY.md`.

### The latch leak (#441)

If `inst/vendor.tar.xz` is present during local development (left behind by a
failed `just r-cmd-build` or `just r-cmd-check` that did not trap-clean),
configure switches to tarball mode. In that mode:

- `.cargo/config.toml` contains `[source.vendored-sources]` instead of
  `[patch."git+url"]`.
- Edits to `miniextendr-api`, `miniextendr-macros`, or `miniextendr-lint` are
  silently ignored — cargo resolves those crates from the stale tarball, not
  the working tree.
- Cargo.lock mismatches may appear when the tarball lock diverges from the
  current workspace state.

Recipes that produce the tarball (`just r-cmd-build`, `just r-cmd-check`,
`just devtools-build`) trap-clean on exit. Recipes that consume configure state
(`just rcmdinstall`, `just devtools-test`, `just devtools-load`,
`just devtools-install`) refuse to run if the tarball is present.

Detection: `minirextendr_doctor()` reports both stale-latch and missing
`.cargo/config.toml`.

Fix: `just clean-vendor-leak` (safe, idempotent). Then re-run `just configure`.

Regression test: `just test-bootstrap-vendor`.

## How it works

### Makevars.in and the double-link

`rpkg/src/Makevars.in` defines `make` rules for both cargo invocations. The
key targets are:

- `$(SHLIB)` — depends on `$(CARGO_AR)` (the staticlib archive).
- `$(CARGO_AR)` — invokes `cargo build --lib --profile $(CARGO_PROFILE)`.
  This is the staticlib. A `FORCE_CARGO` phony target ensures cargo is always
  invoked so that cargo's own incremental logic decides what to rebuild.
- `$(WRAPPERS_R)` — depends on `$(CARGO_CDYLIB)` in source mode. In tarball
  mode, the pre-shipped `R/*-wrappers.R` is used and the cdylib build is
  skipped (the 10–30 s cdylib build is a no-op in tarball installs). Set
  `MINIEXTENDR_FORCE_WRAPPER_GEN` to override for debugging.
- `$(CARGO_CDYLIB)` — invokes `cargo rustc --crate-type cdylib`.

In tarball mode, Makevars also scrubs `vendor/`, `rust-target/`, and
`.cargo/` after a successful build to save installed-package size.

The generated wrappers (`R/miniextendr-wrappers.R`) and
`src/rust/wasm_registry.rs` are **gitignored** (regenerated on every host
install; they ship in the tarball from disk), while `NAMESPACE` and `man/`
stay tracked and must be committed in sync with the `#[miniextendr]` Rust
source. `just wrappers-sync-check` installs rpkg, runs `devtools::document`,
fails on any git diff in `NAMESPACE` / `man/`, and asserts the regenerated
`wasm_registry.rs` is a real (non-stub) snapshot. Regenerate with
`just rcmdinstall && just force-document`.

### configure → cargo feature flags

`configure` auto-detects optional features via `tools/detect-features.R`. The
result is passed to cargo as `--features=$(CARGO_FEATURES)` via `Makevars`.
Override with the `CARGO_FEATURES` environment variable.

### macOS tar and xattrs

When producing `inst/vendor.tar.xz` on macOS, set `COPYFILE_DISABLE=1` to
suppress Apple extended attribute metadata. PAX headers injected by macOS tar
cause warnings on Linux/Windows GNU tar and can confuse the unpacking step in
configure.

### cargo-revendor standalone workspace

`cargo-revendor/` is excluded from the main miniextendr workspace (see root
`Cargo.toml`). Build and test it via `just revendor-build` / `just revendor-test`.
It is the tool that vendors, strips, and compresses the dependency tree into
`inst/vendor.tar.xz`. `--freeze` mode resolves `Cargo.toml` against `vendor/`
only (no network). Windows paths in the generated `config.toml` must use
forward slashes; the `\\?\` prefix from `canonicalize()` must be stripped.

## Decision trees

### I changed a Makevars value — what is the regen flow?

1. Edit `rpkg/src/Makevars.in` (not `rpkg/src/Makevars`).
2. `cd rpkg && bash ./configure` (or `just configure` from the monorepo root).
3. Verify `rpkg/src/Makevars` reflects your change.
4. Commit `Makevars.in`. Do not commit the generated `Makevars` — it is
   generated at install time and its path depends on the local system.

If the generated `configure` script itself needs to change, edit
`rpkg/configure.ac` and run `autoconf` in `rpkg/` to regenerate `configure`.

### Build is picking tarball mode when it shouldn't

1. Check: `ls rpkg/inst/vendor.tar.xz` — if present, the latch is tripped.
2. Run: `just clean-vendor-leak`.
3. Run: `just configure`.
4. Verify: `cat rpkg/src/rust/.cargo/config.toml` shows `[patch."git+url"]`
   entries (monorepo) or only `[build] target-dir` (standalone).
5. If `config.toml` is missing entirely: `minirextendr_doctor()` detects this
   condition; run `bash ./configure` to regenerate.

### Cargo.lock mismatch during build

Source mode (no tarball):
- Cargo rewrites `rpkg/src/rust/Cargo.lock` freely; drift is tolerated at build
  time. But the dev-loop recipes `just check / build / clippy / doc / doc-check
  / test` invoke cargo with
  `--config "patch.'https://github.com/A2-ai/miniextendr'.<crate>.path=…"`,
  which makes cargo *drop* the `source = "git+…#<sha>"` line for each workspace
  sibling (`miniextendr-{api,lint,macros}`). The committed lock must keep those
  lines — CRAN/tarball builds resolve `vendor/` against them. Each of those
  recipes auto-chains `just cargo-lock-restore` (a `git restore --worktree` of
  `rpkg/src/rust/Cargo.lock` from the index/HEAD) as its last line. If a recipe
  aborts mid-way and leaves the lock drifted, run `just cargo-lock-restore`
  manually (#709).

Tarball mode:
- configure runs `tools/lock-shape-check.R` to verify framework crates carry
  `source = "git+https://github.com/A2-ai/miniextendr#<sha>"` (not `path+`).
  `checksum = "..."` lines ARE allowed post-#408 — cargo-revendor recomputes a
  matching `.cargo-checksum.json`. `[[patch.unused]]` is not checked here.
- If the check fails, the Cargo.lock in the tarball drifted from the vendored
  sources. Run `just vendor` from a clean source state to regenerate.

Committed-lock check (pre-commit + CI): `just lock-shape-check` asserts the
checked-in `rpkg/src/rust/Cargo.lock` keeps canonical `git+url#<sha>` sources
for `miniextendr-{api,lint,macros}` and has no `[[patch.unused]]` blocks
(over-broad `[patch.crates-io]`). It prints two recovery paths: `just
cargo-lock-restore` (restore from HEAD) or `just vendor` (regenerate canonical
shape). This is the source-shape counterpart to the tarball-shape
`tools/lock-shape-check.R` above.

### Why is my configure failing with an error in AC_CONFIG_COMMANDS?

Almost always one of:
- Called as `./configure` instead of `bash ./configure` — the `#!/bin/sh`
  passthrough produces spurious errors in `AC_CONFIG_COMMANDS`.
- m4 quoting issue: literal `[` or `]` inside a `AC_CONFIG_COMMANDS` block
  without `@<:@` / `@:>@` escaping. Check the configure.ac line that defines
  the failing command.
- `$1` used inside `AC_CONFIG_COMMANDS` — it is empty there. Use `$0` or
  eliminate the reference.

## Key files

- `rpkg/configure.ac` — autoconf source for install-mode detection and cargo
  config generation.
- `rpkg/configure` — generated configure script (do not edit; regenerate with
  `autoconf` in `rpkg/`).
- `rpkg/src/Makevars.in` — Makevars template driving the double-link pipeline.
- `rpkg/src/win.def.in` → `rpkg/src/miniextendr-win.def` — Windows symbol
  export definitions.
- `rpkg/src/stub.c` — static C translation unit; declares `miniextendr_force_link`.
- `rpkg/src/rust/.cargo/config.toml` — generated inline by configure; three
  variants per install mode. No `.in` template exists for this file.
- `rpkg/tools/` — `Rscript`-invoked helpers for configure (lock-shape-check.R,
  detect-features.R, etc.).
- `miniextendr-api/src/registry.rs` — `miniextendr_write_wrappers` cdylib
  entry + `collect_r_wrappers` ordering logic.
- `cargo-revendor/` — standalone vendoring tool (separate workspace).
- `docs/CRAN_COMPATIBILITY.md` — vendoring requirements and offline build
  verification.
- `docs/RELEASE_WORKFLOW.md` — AlmaLinux 8 / macOS arm64 release CI details.

## Common pitfalls

- **Never edit generated files directly.** `rpkg/src/Makevars`,
  `rpkg/src/miniextendr-win.def`, and `rpkg/src/rust/.cargo/config.toml` are
  all generated by configure. Edit the `.in` templates or `configure.ac` and
  re-run configure.

- **`configure.ac` must not mutate sources.** Writing to `Cargo.toml`,
  `Cargo.lock`, or `.rs` files during `./configure` dirties the VCS working
  tree. Vendoring belongs in `just vendor`, not in configure.

- **`configure.ac` must not call `minirextendr::*`.** Configure runs in a
  minimal environment where the R library may not be available. Put helpers
  in `tools/*.R` and invoke via `Rscript tools/foo.R`.

- **`PACKAGE_NAME` includes dots; `PACKAGE_TARNAME` does not.** When deriving
  C or Rust identifiers, convert both hyphens and dots to underscores.

- **macOS `/bin/sh` is not dash.** The `bash ./configure` requirement is about
  `AC_CONFIG_COMMANDS` passthrough behavior, not about a dash-incompatible
  script.

- **`R CMD build --debug` is invalid.** R silently ignores the flag. The
  `r-cmd-build` justfile recipe passes it; this is a pre-existing harmless quirk.

- **Trap-cleanup in justfile recipes.** Recipes that produce a transient
  tarball use `trap 'rm -f X' EXIT` joined on one logical shell line (via `\`
  continuation). A recipe where the trap and the command run as separate `-c`
  invocations will fire the trap immediately after defining it, before the
  command runs.

## Related skills

- `miniextendr-architecture` — the install-mode latch, distributed_slice tables,
  and the double-link pipeline at a higher level.
- `miniextendr-scaffolding` — minirextendr templates, doctor checks, and the
  `use_release_workflow()` helper for CI scaffolding.
- `miniextendr-lint` — the MXL300 / MXL301 rules that `build.rs` enforces
  during `cargo check`.
- `miniextendr-getting-started` — end-user walkthrough of `bash ./configure &&
  R CMD INSTALL .`.
