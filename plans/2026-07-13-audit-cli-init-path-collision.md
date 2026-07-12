# Plan: audit 2026-07-12 P0 — CLI `init package` cannot create a package (+ CLI contract lies)

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `fix/audit-cli-init-path-collision`.

Covers 2026-07-12 audit worklist items 2, 11, 12, 13. Items 1 and 3 (the
adopt-or-demote product decision and deleting the duplicated CLI scaffold
templates) are **maintainer decisions** — see "Deferred" below and the
pre-existing on-disk plan `plans/2026-07-01-cli-adopt-or-demote.md`
(untracked in the main checkout). This plan only makes the advertised
commands stop lying; it does not grow the CLI's scaffold surface.

## Verified defects (all re-verified on 656e5cdd)

**A. `init package <dest>` is a deterministic dead end.**
- `miniextendr-cli/src/main.rs:48-51`: `run()` calls
  `ProjectContext::discover(Path::new(&cli.path))` before dispatch for
  *every* command (only `Completions` is exempted earlier, main.rs:27-35).
- `miniextendr-cli/src/project.rs:78-79`: `discover` starts with
  `std::fs::canonicalize(path)` → errors on any nonexistent path.
- `miniextendr-cli/src/commands/init.rs:36-40`: `init_package` *requires*
  the destination to not exist (`bail!("Directory already exists: ...")`).
- `miniextendr-cli/src/cli.rs:11-14`: global `#[arg(long, global = true,
  default_value = ".")] pub path: String` shares the clap arg id `path` with
  the `InitCmd::Package` positional (`cli.rs:117-123`) and the
  `InitCmd::Monorepo` positional (`cli.rs:124-127`). The audit's black-box
  reproduction shows the positional destination surfacing as `cli.path`
  ("Error: path does not exist: .../demoPkg") — so the pre-dispatch
  canonicalize runs on the not-yet-existing destination. Two independent
  bugs: the id collision AND the unconditional pre-dispatch discovery.
- There are zero CLI integration tests (`miniextendr-cli/` has no `tests/`
  dir; only a few unit tests inside `src/`).

**B. `--json` promise is partial.**
`miniextendr-cli/src/commands.rs:24-41`: dispatch passes `json` to
status/vendor/feature/config only; workflow, init, cargo, render, rust,
lint, clean silently ignore it. `miniextendr --json workflow doctor` prints
human text (audit evidence).

**C. README advertises a removed flag.**
`miniextendr-cli/README.md:48` documents `workflow configure --cran`; no
`--cran` exists anywhere in `cli.rs`/`workflow.rs` (grep verified — the only
hit for `cran` in the crate is that README line). The parser rejects it.

**D. `status show/validate` contradict `workflow doctor` on source trees.**
- `miniextendr-cli/src/commands/status.rs:100-104`: vendored-crate paths
  (`vendor/<crate>` from `MINIEXTENDR_CRATES`, `project.rs:6`) are checked
  unconditionally → reported "missing" on a healthy source-mode tree.
- `status.rs:151`: `R/<pkg>-wrappers.R` (via `wrapper_file`, `status.rs:70`)
  checked unconditionally → "missing" although it is generated-on-build.
- `status.rs:224` (`status_validate`) + `status.rs:333` repeat the vendor
  check as warnings.
- `miniextendr-cli/src/commands/workflow.rs:222` (doctor) already has the
  correct semantics: "not unpacked (normal in source mode)".

## Work items (flat order)

1. **Fix the arg-id collision.** Rename the `Init` positional fields so
   they no longer share clap id `path` with the global option: `dest:
   String` on `InitCmd::Package` (cli.rs:117-123) and `InitCmd::Monorepo`
   (cli.rs:124-127), with `value_name = "DEST"`. Update the destructuring
   in `commands/init.rs:9-33`.
2. **Stop discovering a project for commands that create one.** In
   `main.rs`, dispatch `Command::Init` with `InitCmd::Package`/`Monorepo`
   *before* `ProjectContext::discover` (same pattern as `Completions`,
   main.rs:27-35), or make `commands::dispatch` take a lazy ctx. `init use`
   still needs the discovered ctx (it mutates an existing package,
   init.rs:26-31) — keep discovery for it.
3. **Black-box tests** — `miniextendr-cli/tests/cli_init.rs` driving
   `env!("CARGO_BIN_EXE_miniextendr")` via `std::process::Command` (zero
   new dependencies; same rationale as `plans/2026-07-01-cli-adopt-or-demote.md`
   item 2). Cases (per worklist item 2): `init package` with a nonexistent
   relative dest, nonexistent absolute dest, existing dest (clean error,
   exit != 0, no panic), `--path <elsewhere>` combined with a positional
   dest, `init monorepo <dest>`, and `--help` for every `init` subcommand.
   Assert `init package` actually creates DESCRIPTION + `src/rust/lib.rs`.
4. **Remove the stale README flag** (`miniextendr-cli/README.md:48`) and
   add a drift guard: a test that runs `Cli::command().try_get_matches_from`
   over every fenced command line in the README (or, cheaper, a curated
   array of them) so a removed flag fails CI instead of shipping.
5. **Make `--json` honest** (worklist 12). Recommended: *scope* the flag —
   move `json` off the global `Cli` struct onto the subcommands that
   implement it (status/vendor/feature/config), so `--json workflow doctor`
   is a parse error instead of a silent lie. Plumbing JSON through
   workflow/init is bigger and belongs to the adopt-or-demote decision.
   Add one JSON parse test per supported subcommand
   (`serde_json::from_str` on captured stdout).
6. **Unify status/doctor install-mode semantics** (worklist 11). Add a
   single mode probe on `ProjectContext` (source vs tarball, keyed on
   `inst/vendor.tar.xz` presence — mirror configure's latch, and the
   unpacked-vendor state doctor already distinguishes at workflow.rs:222).
   `status_show` (status.rs:100-104, :151) and `status_validate`
   (status.rs:224, :333) report generated/mode-dependent files as
   "absent (expected in source mode)" — a third category, not `missing`,
   in both human and JSON output.

## Deferred (maintainer decision — do NOT fold into this PR)

- Adopt-or-demote and the duplicated template source
  (`init.rs:52-102` writes its own configure.ac/Makevars/bootstrap copies;
  it omits files its own `status show` expects — config.guess, config.sub,
  configure.win, cleanup.win, r_shim.h — compare
  `minirextendr/inst/templates/rpkg/`). Per the audit: do not repair the
  duplicated templates independently. If the maintainer chooses demotion,
  items 1-3 shrink to "delete `init package`/`init monorepo` + README";
  items 4-6 survive either way. Whoever implements: ask before starting if
  the decision is still open, and reference
  `plans/2026-07-01-cli-adopt-or-demote.md`.

## Exact commands (worktree)

```bash
cargo test -p miniextendr-cli 2>&1 > /tmp/audit-cli-test.log      # Read it
cargo build -p miniextendr-cli --features dev                     # dev feature still builds
cargo clippy --workspace --all-targets --locked -- -D warnings    # + all/all_s7 legs per ci.yml
cargo fmt --all
# manual smoke (must succeed from a clean shell):
cargo run -q -p miniextendr-cli -- init package /tmp/mx-audit-demoPkg
```

## Must NOT touch

- `minirextendr/inst/templates/**`, `patches/templates.patch` (no template
  work in this PR; `just templates-check` must stay green untouched).
- The content of the CLI's embedded templates (init.rs:52-102 writers) —
  routing only; template consolidation is the deferred decision.
- `justfile` recipes.

## Done criteria

- `miniextendr init package <new-path>` creates a package skeleton;
  `init monorepo` likewise; black-box tests cover both plus the error
  paths and run in CI's existing `cargo test --workspace` (verify the cli
  crate is a workspace member the recipe picks up).
- `--json` is either honoured or rejected by the parser — never ignored.
- README contains no command the parser rejects; drift guard test in place.
- `status show`/`status validate`/`workflow doctor` agree on a healthy
  source-mode `rpkg` tree (no false "missing").
- Three clippy legs + `just test` green.

## Escalation rule

If reality diverges from this plan — the clap id collision behaves
differently than described, `init use` turns out to also be broken, or the
maintainer has already made the adopt-or-demote call — **stop, commit
nothing further, and report back. Do not improvise.**
