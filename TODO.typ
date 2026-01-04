 - [x] Add a small assert in the macro to emit a clear compile error if the wrong base is used (e.g., using AltReal flags under base = "Int"). Implemented in `miniextendr-macros/src/lib.rs` via a base-specific trait bound check requiring the corresponding `Alt*` family trait for the selected `base`.
- [x] Only use `static` and not `static mut` for symbols from R.
  - `R_Interactive` is a challenge here.
  - Fix: Changed `static mut` to `static` with raw pointer writes via helper functions.
    - miniextendr-engine: set_r_interactive(), set_r_signal_handlers()
    - miniextendr-api: set_r_cstack_limit(), get_r_cstack_*() (nonapi feature-gated)
- [x] ensure all ffi'd function have the r_ffi macro that provide safe equivalents
- [x] implement proper rayon feature...
  - Generic `with_r_vec<T>` with type inference
  - RNativeType::dataptr_mut() for safe data pointer access
  - Clear documentation on parallel limitations
- [x] make sure that `miniextendr-bench` uses the common `rpkg/src/target` directory...
  - Fix: Added miniextendr-bench to workspace, updated to edition 2024, fixed REngine::new() → build()

== Codex Review Findings (2024) ==

=== CRITICAL: Safety Issues ===
- [x] `SexpExt::as_slice` returns `'static` slices from R memory (unsound)
  - `miniextendr-api/src/ffi.rs:208-220`
  - Fix: Made `as_slice` unsafe with comprehensive safety docs
- [x] Rayon `run_r` panics on Rayon threads (thread-local routing missing)
  - `miniextendr-api/src/rayon_bridge.rs:81-86`, `worker.rs:119-155`
  - Fix: Removed `run_r`, documented limitations. Use `with_r_thread` directly.
    Architecture: R calls happen before/after parallel work, not within.
- [x] Unprotected R vectors in `with_r_*_vec` can be GC'd mid-parallel write
  - `miniextendr-api/src/rayon_bridge.rs:166-209`
  - Fix: Refactored to generic `with_r_vec<T>` using PROTECT/UNPROTECT.
    Added `RNativeType::dataptr_mut()` for type-safe data pointer access.

=== HIGH: Thread Safety ===
- [x] `is_r_main_thread` defaults to true before init
  - `miniextendr-api/src/worker.rs:69-74`
  - Fix: Now returns false when uninitialized (safe default)
- [x] Allocator fallback can call R API on non-main thread
  - `miniextendr-api/src/allocator.rs:70-84`
  - Fix: Removed fallback, now panics with clear error message
- [x] `StackCheckGuard` is not concurrency-safe
  - `miniextendr-api/src/thread.rs:75-94`
  - Fix: Implemented global refcount with atomic operations
- [x] `SendableSexp` is marked `Sync` despite cross-thread mutation risks
  - `miniextendr-api/src/externalptr.rs:69-79`
  - Fix: Removed `Sync` impl, documented why it's unsafe
- [x] Allocator can longjmp across Rust frames
  - `miniextendr-api/src/allocator.rs:209`
  - Fix: Added module-level and call-site documentation about longjmp risk
- [x] Worker init can be called from non-main thread without guard
  - `miniextendr-api/src/worker.rs:332-356`
  - Fix: Added thread consistency check and documentation requirements

=== MEDIUM: Memory/Leaks ===
- [x] R continuation tokens are preserved forever (leak)
  - `miniextendr-api/src/worker.rs:44-51`, `unwind_protect.rs:17-24`
  - Fix: Consolidated to single global token in unwind_protect.rs (no per-thread leak)

=== API/Ergonomics ===
- [x] `REngine::new()` and `shutdown()` shown in docs but not implemented
  - `miniextendr-engine/src/lib.rs:20-33`
  - Fix: Updated example to use `REngine::build()`, documented that shutdown is
    intentionally not provided (Rf_endEmbeddedR is non-reentrant).
- [x] `with_args` default in docs is incorrect
  - `miniextendr-engine/src/lib.rs:96-99`
  - Fix: Updated doc comment to show correct default: `["R", "--quiet", "--vanilla"]`
- [x] Doc claims atexit cleanup is registered, but code does not
  - `miniextendr-engine/src/lib.rs:259-260`
  - Fix: Corrected documentation to explain that cleanup is intentionally NOT
    registered because Rf_endEmbeddedR is non-reentrant.
- [x] Encoding init is documented but disabled in R entrypoint
  - `rpkg/src/entrypoint.c.in:7-10`, `encoding.rs:29-73`
  - Fix: Added documentation explaining that encoding_init only works when
    embedding R (miniextendr-engine), not for R packages where symbols aren't exported.
- [ ] `#[miniextendr]` impl blocks: consuming `self` by value not fully supported
  - `miniextendr-macros/src/miniextendr_impl.rs:79` - "Consuming method (not supported in v1)"
  - Note: `&self` and `&mut self` work correctly
  - `self` by value treated as finalizer unless `#[miniextendr(returns_self)]` is set
  - Future: Allow consuming methods that return a different type
- [x] `miniextendr_module!` treats `extern "C-unwind" fn` and `fn` the same
  - `miniextendr-macros/src/lib.rs:815-816`
  - Fix: Updated documentation to clarify this is intentional. The ABI distinction
    is handled by `#[miniextendr]` at function definition, not in module declaration.
    The `extern "C-unwind" fn` syntax remains accepted for backwards compatibility.
- [x] String NA handling is lossy (`NA_character_` → `""`)
  - `miniextendr-api/src/from_r.rs:298-302`
  - Fix: Added `Option<String>` impl that returns None for NA.
    Updated docs on `String`/`&str` to warn about lossy behavior.
- [x] Missing `IntoR`/`TryFromSexp` conveniences
  - `Vec<String>`, `Vec<&str>` → STRSXP (done)
  - `Vec<Option<T>>` for f64, i32, String → NA-aware vectors (done)
  - [x] Tuple-to-list for small tuples (done - 2-8 element tuples → VECSXP)

=== Build/Packaging ===
- [x] `miniextendr-engine` build script doesn't validate `R RHOME` exit status
  - `miniextendr-engine/build.rs:17-25`
  - Fix: Added exit status check, empty output check, and directory existence check
- [x] Generated build artifacts tracked in git (target/, config.log, etc.)
  - Fix: Updated `.gitignore` with proper entries for config.log, config.status,
    autom4te.cache/, generated Makevars/entrypoint.c/Cargo.toml/.cargo/, vendor/, etc.
- [x] Template/generated files can drift (.in vs generated)
  - Fix: CI `generated-files-check` job verifies generated files not committed
  - R package check jobs run configure which validates templates work correctly
- [x] Vendored set incomplete for `--all-features` (missing rayon crate)
  - `rpkg/src/vendor/`
  - Fix: CRAN builds now use default features (no rayon) to avoid unvendored deps.
    Rayon only enabled for dev builds which use network access.
- [x] `Cargo.lock` doesn't reflect feature-enabled dependency graph
  - Note: Lockfile in workspace root covers all workspace members.
    CRAN builds use default features only, so reduced dep set is fine.
- [x] Vendored `miniextendr-api` dev-dep points outside vendor
  - Note: dev-dependencies only used for tests, not library builds.
    CRAN builds don't run miniextendr-api tests, so this is fine.
- [x] Generated `.cargo/config.toml` contains absolute local paths
  - Note: This is expected - configure regenerates paths for each build env.
    The .cargo/ dir is under src/ and not included in installed package.
- [x] `R RHOME` not error-checked in configure.ac
  - `rpkg/configure.ac:3-5`
  - Fix: Added error checking for R RHOME command and R_HOME directory
- [x] `bootstrap.R` doesn't check exit status of configure/autoconf
  - `rpkg/bootstrap.R:8-27`
  - Fix: Added `run_cmd()` helper that checks exit status and stops on failure
- [x] `rsync` and `sed` required but not validated in configure
  - Fix: Added AC_PATH_PROG checks with error messages, use $RSYNC/$SED variables
- [x] `cargo pkgid --offline` can fail on fresh dev machines
  - Fix: Made `--offline` conditional on NOT_CRAN in configure.ac
- [x] `--all-features` always enabled (CRAN policy risk)
  - Fix: Made feature selection conditional on NOT_CRAN in configure.ac:
    - NOT_CRAN=true (dev): --all-features (nonapi + rayon)
    - NOT_CRAN=false (CRAN): default features only (no warnings, no unvendored deps)
- [x] C preprocessor flags hard-coded to NONAPI
  - Fix: CARGO_FEATURE_CPPFLAGS now derived from feature selection in configure.ac
- [x] No `.Rbuildignore` present
  - Fix: Added comprehensive `.Rbuildignore` to rpkg
- [x] Rust edition 2024 with no minimum rustc check
  - Fix: Added rustc 1.85+ version check in configure.ac
- [x] `cleanup` script removes wrong config path
  - Fix: Changed `.cargo/config.toml` to `src/rust/.cargo`

=== Testing ===
- [x] Rayon integration tests too narrow (missing `with_r_vec`)
  - Fix: Added comprehensive integration tests in `miniextendr-api/tests/rayon.rs`
    using miniextendr-engine for embedded R. Tests cover `with_r_vec` (basic, parallel
    write, i32, empty, large), `RVec` parallel collect, and `IntoR` conversion.
- [ ] No automated regression test for registration bug
  - Note: User indicated this is likely a fluke, low priority.
- [x] Macro compile-fail tests missing (no trybuild/UI tests)
  - Fix: Added trybuild dev-dependency to miniextendr-macros, created tests/ui.rs runner
    and 6 compile-fail test cases: unknown_option, pattern_parameter, option_with_value,
    module_missing_mod, module_duplicate_mod, unsafe_empty
- [ ] Thread-safety assertions not covered by tests
  - Note: Would require embedded R runtime for meaningful tests.
- [ ] Known TODOs not tracked as GitHub issues


== Reviews Findings (December 2024) ==

=== COMPLETED (this session) ===
- [x] Fix autoconf awk quoting bug - `$1`/`$2` eaten by m4 in configure.ac
  - `minirextendr/inst/templates/rpkg/configure.ac`
  - `minirextendr/inst/templates/monorepo/rpkg/configure.ac`
  - `tests/cross-package/producer.pkg/configure.ac`
  - `tests/cross-package/consumer.pkg/configure.ac`
  - Fix: Changed `$1` to `[$]1` in awk expressions so autoconf emits literal `$1`
- [x] Remove proc-macro-error dependency to eliminate dual syn (1.x + 2.x)
  - `miniextendr-macros/Cargo.toml`, `miniextendr-macros/src/lib.rs`
  - `miniextendr-macros/src/roxygen.rs`
  - Fix: Replaced `emit_warning!` with deprecation-based warnings via generated tokens
  - Result: Only syn v2 in dependency tree now
- [x] Add vendor sync check (`just vendor-sync-check`)
  - `justfile`
  - Fix: New recipe verifies rpkg/src/vendor/ matches workspace sources
  - Prevents CRAN build drift from workspace changes
- [x] Fix build.rs stack flags comment inconsistency
  - `miniextendr-api/build.rs:3-12`
  - Fix: Updated misleading comments that said stack flags tied to nonapi feature
  - Reality: Stack flags set unconditionally for R compatibility
- [x] Make rayon feature opt-in in miniextendr-api
  - `miniextendr-api/Cargo.toml`
  - Fix: Changed `default = ["doc-lint", "rayon"]` to `default = ["doc-lint"]`
  - Result: Smaller default dependency footprint for CRAN builds
- [x] Add lint sync check (`just lint-sync-check`)
  - `justfile`
  - Fix: New recipe checks for significant drift between macros and lint parsers
  - Note: Files intentionally differ (lint omits codegen helpers)
- [x] Align CARGO_LOCKED_FLAG handling across all configure.ac files
  - Templates and cross-package tests now derive from NOT_CRAN consistently

=== Documentation (from reviews 01, 02, 08) ===
- [x] Add SAFETY.md documenting FFI/thread safety invariants for Send wrapper types
  - `reviews/01_miniextendr-api.md` section "Invariant documentation"
  - Location: Top-level SAFETY.md
  - Content: Thread model, `Sendable<T>`/`SendablePtr<T>` safety, ExternalPtr safety,
    R_UnwindProtect, StackCheckGuard, allocator thread requirements, FFI categories
- [x] Add macro expansion pipeline documentation
  - Location: miniextendr-macros/src/lib.rs module docs
  - Content: Flow diagrams for fn/impl/trait/module macros, module table,
    symbol naming conventions, return type handling, class systems
- [x] Consolidate `R_init_*` requirements into one doc
  - Location: Top-level ENTRYPOINT.md
  - Content: Initialization order, function explanations, API timing table,
    minimal example, embedding R section, troubleshooting

=== Reference Study Tasks (from background/) ===

==== R Internals & Extensions ====
- [ ] Study `background/R Internals.html` for SEXP type system
  - Document missing SEXP types not yet exposed in miniextendr-api/src/ffi.rs
  - Verify PROTECT/UNPROTECT patterns match R's expectations
  - Check for undocumented API behaviors
- [ ] Study `background/Writing R Extensions.html` for .Call interface
  - Verify R wrapper generation matches documented conventions
  - Check registration patterns against recommended practices
  - Verify NA handling matches R's documented behavior
- [ ] Study ALTREP documentation (`background/ALTREP_ Alternative Representations...html`)
  - Compare miniextendr ALTREP impl against documented patterns
  - Identify any missing ALTREP methods worth implementing
  - Document which ALTREP classes are supported vs. planned

==== R Source Reference ====
- [ ] Use `background/r-source-tags-R-4-5-2/` to verify FFI bindings
  - Location: `src/include/Rinternals.h` - verify SEXP type definitions match
  - Location: `src/include/R_ext/Altrep.h` - verify ALTREP bindings are complete
  - Location: `src/main/memory.c` - study GC behavior for protect patterns
  - Location: `src/main/altclasses.c` - study ALTREP dispatch for reference

==== Class System References ====
- [ ] Study S7 patterns (`background/S7-main/`) for class generation
  - How does S7 handle method dispatch?
  - How does S7 generate constructors?
  - Patterns for property access/validation
  - Reference for improving #[miniextendr] impl block codegen
- [ ] Study R6 patterns (`background/R6-main/`) for reference class generation
  - How does R6 handle private vs. public fields?
  - How does R6 handle inheritance?
  - Reference for R6 class generator in miniextendr-macros
- [ ] Study vctrs patterns (`background/vctrs-main/`) for type coercion
  - How does vctrs handle type casting?
  - Recycling rules for binary operations
  - Patterns for Vec<T> / Option<T> conversions in miniextendr-api

==== ALTREP Implementation References ====
- [ ] Study `background/Rpkg-mutable-master/` for mutable ALTREP patterns
  - How does it handle write barriers?
  - How does it handle copy-on-modify semantics?
- [ ] Study `background/Rpkg-simplemmap-master/` for memory-mapped ALTREP
  - How does it handle lazy loading?
  - How does it handle file descriptor lifecycle?
- [ ] Study `background/vectorwindow-main/` for ALTREP views
  - How does it implement subset views without copying?
  - How does it handle window lifecycle?

==== Documentation & Tooling References ====
- [ ] Study roxygen2 (`background/roxygen2-main/`) for R wrapper generation
  - How does roxygen2 parse @param, @return, @export tags?
  - Patterns for improving miniextendr-macros/src/roxygen.rs
  - Reference for R documentation generation
- [ ] Study mirai (`background/mirai-main/`) for async patterns
  - How does mirai handle clean environment evaluation?
  - Patterns for worker thread communication
  - Reference for potential async miniextendr features

=== Testing (from reviews 02, 06, 08) ===
- [x] Add snapshot/golden tests for R wrapper generation
  - Location: miniextendr-macros/tests/snapshots.rs
  - Uses: expect-test crate for inline snapshot testing
  - Coverage: 21 tests for function wrappers, impl blocks (env/r6/s3),
    roxygen tags, DotCallBuilder, RArgumentBuilder, defaults, dots
- [x] Add CI check for generated file hygiene
  - `reviews/06_rpkg.md` section "Generated file hygiene"
  - `.github/workflows/ci.yml` - `generated-files-check` job
  - Checks: rpkg/src/Makevars, entrypoint.c, rust/Cargo.toml, rust/.cargo/
- [x] Add CI for cross-package trait ABI tests
  - `reviews/06_rpkg.md` section "Suggested next checks"
  - `.github/workflows/ci.yml` - `cross-package-tests` job
  - Builds/tests producer.pkg and consumer.pkg on Linux

=== Build/Infrastructure (from reviews 03, 04, 07) ===
- [x] Add REngineBuilder::r_home(PathBuf) to bypass R RHOME shell-out
  - Location: miniextendr-engine/src/lib.rs
  - Already implemented: `r_home()` method on REngineBuilder
  - Enhanced: `REngineError::RHomeNotFound` now includes stderr for better diagnostics
- [x] Add linking strategy documentation
  - Location: Top-level LINKING.md
  - Content: R package vs standalone linking, build.rs strategy, rpath behavior,
    platform notes, troubleshooting, environment variables
- [ ] Consider processx-based execution in minirextendr
  - `reviews/07_minirextendr.md` section "system2() portability"
  - Location: `minirextendr/R/*.R`
  - Purpose: Better cross-platform command execution with proper quoting/output capture
  - Note: processx is common in R tooling ecosystem

=== Optional Enhancements (lower priority) ===
- [x] Add more lint rules to miniextendr-lint
  - Already implemented in miniextendr-lint/src/lib.rs:
    - "exported item exists but not listed in miniextendr_module!" (lines 308-316)
    - "listed item does not exist / is cfg'd out" (lines 319-329)
    - Multiple impl blocks require labels (lines 663-710)
    - Class system compatibility for trait impls (lines 632-660)
  - Not implemented (not detectable from Rust):
    - "trait ABI: init_ccallables() not called in `R_init_*`" (in C code)
- [x] Add bench environment documentation
  - Location: miniextendr-bench/README.md
  - Content: Recommended setup, environment capture commands, running consistently,
    interpreting results, environment variables, benchmark categories
- [x] Add integration test for minirextendr workflow
  - Implemented: `minirextendr/tests/testthat/test-status-coverage.R`
  - Tests: `has_miniextendr()`, `miniextendr_status()`, `miniextendr_check()` with temp projects

==== minirextendr Dependency Rationalization ====
Source: `reviews/dependency-idiomaticity.md`

Strong fit (replace manual code):
- [x] Replace manual `git init` in `create.R:98-103` with `usethis::use_git()`
  - Already implemented at create.R:99-101
- [x] Replace `jsonlite::fromJSON()` in `vendor.R:12-35` with `gh::gh()` for GitHub API
  - Benefits: automatic pagination, auth token handling, rate limit awareness
  - Implemented: Replaced jsonlite with gh, removed jsonlite from DESCRIPTION
- [x] Replace manual gsub templater in `utils.R:152-179` with `usethis::use_template()`
  - Already implemented at utils.R:163 using usethis::use_template()

Good fit (add functionality):
- [x] Add persistent cache for downloaded tarballs using `rappdirs::user_cache_dir("minirextendr")` in `vendor.R`
  - Implemented: Cache in vendor.R with download_miniextendr_archive()
  - Added: `refresh` param to vendor_miniextendr(), miniextendr_clear_cache(), miniextendr_cache_info()
  - Added rappdirs to DESCRIPTION
- [x] Improve project detection in `utils.R` with `rprojroot::find_root(rprojroot::has_file("Cargo.toml"))`
  - Implemented: Added find_rust_root() helper using rprojroot
  - Updated detect_project_type() and is_in_rust_project() to walk up tree

Optional:
- [ ] Add `miniextendr.yml` config file support for user defaults using `yaml` package
  - Store: crate name, rpkg name, version, features
- [ ] Add `clipr` for copying "next steps" commands to clipboard
- [ ] Add `lifecycle` for deprecation warnings and API evolution

==== minirextendr usethis Replacements ====
Source: `reviews/usethis-replacements.md`

- [x] Keep hand-built DESCRIPTION in `create.R:133` (not using usethis::use_description())
  - Reason: Creating DESCRIPTION in subdirectory (rpkg/) requires project context switching
  - Current sprintf approach is simpler, more direct for scaffolding
  - `desc` package used for updates (use_miniextendr_description)
- [x] Replace manual `.Rbuildignore` append in `use-r.R:69` with `usethis::use_build_ignore(template_lines, escape = FALSE)`
  - Implemented: Uses usethis for deduplication and file creation
- [x] Replace manual `.gitignore` append in `use-r.R:100` with `usethis::use_git_ignore(template_lines, directory = ".")`
  - Implemented: Uses usethis for deduplication and file creation
- [x] Replace custom `use_template()` in `utils.R:140` with `usethis::use_template()`
  - Already implemented: utils.R:163 delegates to usethis::use_template()
- [x] Keep `ensure_dir()` in `utils.R:311` (not replaced)
  - Reason: `usethis::use_directory()` only works for project-relative paths
  - `ensure_dir()` handles arbitrary paths (vendor.R, target_path)
- [x] Keep custom package doc template in `use-r.R:10` (not using usethis::use_package_doc())
  - Reason: Template includes @useDynLib directive; using usethis + patching adds complexity
  - Current approach: Single template with all miniextendr-specific content

checking available recipes (`just --list`) - ALL EXIST
- [x] build, check, clean, clippy, configure, default
- [x] devtools-build, devtools-check, devtools-document, devtools-install, devtools-load, devtools-test
- [x] doc, doc-check, expand, fmt, fmt-check
- [x] r-cmd-build, r-cmd-check, r-cmd-install, test, test-r-build, tree
- [x] vendor-sync-check, lint-sync-check (new recipes added)
- [x] minirextendr-* recipes (build, check, dev, document, install, load, rcmdcheck, test)
- [x] cross-* recipes for cross-package tests
- [x] templates-* recipes for template management

=== Planned: Optional indicatif progress ===
- [x] Add `indicatif` feature to `miniextendr-api` (opt-in, non-default) with `indicatif -> nonapi` dependency
- [x] Implement `RTerm` (`indicatif::TermLike`) that writes to R console via `ptr_R_WriteConsoleEx` and no-ops off main thread
- [x] Provide ANSI cursor/clear defaults in `RTerm` (cursor moves, clear line, write_line)
- [x] Implemented `term_like_stdout()`, `term_like_stderr()` and `into_draw_target()` helpers
- [x] Updated NONAPI.md with `ptr_R_WriteConsoleEx` under feature-gated non-API functions

=== Planned: Feature shortlist from Rust ecosystem ===
Source: `reviews/feature-plans-uuid-time-regex-indexmap.md`, `reviews/feature-shortlist.md`

Common scaffolding for all features:
1. Add optional dep + feature in `miniextendr-api/Cargo.toml` (non-default)
2. Create feature module: `*_impl.rs`
3. Gate module in `lib.rs` with `#[cfg(feature = "...")]`
4. Add doc block per feature in `lib.rs` with example snippets
5. Add feature-gated tests under `miniextendr-api/tests/`

==== uuid feature ====
- [x] Add `uuid = { version = "1", optional = true, features = ["v4"] }` to Cargo.toml
- [x] Create `uuid_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `Uuid`: parse from R `character(1)`
- [x] Implement `IntoR` for `Uuid`: convert to R `character(1)`
- [x] Implement `TryFromSexp` for `Vec<Uuid>`: parse from R `character` vector
- [x] Implement `IntoR` for `Vec<Uuid>`: convert to R `character` vector
- [x] Handle `Option<Uuid>` for NA support: `NA_character_` ⇄ `None`
- [x] Map parse failures to `SexpError::InvalidValue`
- [x] Add feature-gated tests (miniextendr-api/tests/uuid.rs)

==== time feature ====
- [x] Add `time = { version = "0.3", optional = true, features = ["formatting", "parsing", "macros"] }` to Cargo.toml
- [x] Create `time_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `OffsetDateTime`: R `POSIXct` (numeric + tzone attr) → Rust
- [x] Implement `IntoR` for `OffsetDateTime`: Rust → R `POSIXct` with tzone (UTC)
- [x] Implement `TryFromSexp` for `time::Date`: R date (day counts since 1970-01-01)
- [x] Implement `IntoR` for `time::Date`: Rust → R Date
- [x] Fractional seconds policy: truncate (documented in module)
- [x] Add Vec and Option variants for both OffsetDateTime and Date
- [x] Add feature-gated tests (10 tests)

==== regex feature ====
- [x] Add `regex = { version = "1", optional = true }` to Cargo.toml
- [x] Create `regex_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `Regex`: compile from R `character(1)`
- [x] Handle `Option<Regex>` for NA support
- [x] Add `try_compile` helper (users wrap in ExternalPtr themselves for caching)
- [x] Documented `ExternalPtr<Regex>` pattern for loop reuse in module docs
- [x] Add feature-gated tests (5 tests)

==== indexmap feature ====
- [x] Add `indexmap = { version = "2", optional = true }` to Cargo.toml
- [x] Create `indexmap_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `IndexMap<String, T>`: R named list → Rust
- [x] Implement `IntoR` for `IndexMap<String, T>`: Rust → R named list
- [x] Preserve insertion order in both directions
- [x] Auto-name unnamed entries: "V1", "V2", ... when converting R list without names
- [x] Add feature-gated tests (5 tests)

=== Planned: External-trait export strategy ===
Source: `reviews/trait-export-and-numeric-crates.md`

**Key constraint:** Cannot directly export external (non-owned) traits to R.

Solution: Adapter trait pattern
- [x] Document adapter-trait pattern for exporting non-owned traits to R
  - Location: Top-level ADAPTER_TRAITS.md
  - Content: Basic pattern, blanket impl examples, Iterator adapter, newtype alternative
- [x] Provide example wrapper trait + blanket impl pattern in docs/reviews
  - Location: ADAPTER_TRAITS.md - Complete Example section
- [x] Clarify trait ABI constraints:
  - Location: ADAPTER_TRAITS.md - Trait ABI Constraints table
  - No generic parameters, no async, no generic methods, TryFromSexp/IntoR requirements
- [x] Document newtype wrapper as alternative for total control and explicit conversions
  - Location: ADAPTER_TRAITS.md - Alternative: Newtype Wrapper section

=== Planned: Numeric crate feature candidates ===
Source: `reviews/trait-export-and-numeric-crates.md`

Common scaffolding (same as feature shortlist):
1. Add optional dep + feature in `miniextendr-api/Cargo.toml`
2. Create `*_impl.rs` module
3. Gate module with `#[cfg(feature = "...")]`
4. Add doc block + tests

==== num-bigint feature ====
- [x] Add `num-bigint = { version = "0.4", optional = true }` to Cargo.toml
- [x] Create `num_bigint_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `BigInt`: parse from R `character`
- [x] Implement `IntoR` for `BigInt`: convert to R `character` (lossless)
- [x] Implement `TryFromSexp` for `BigUint`: parse from R `character`
- [x] Implement `IntoR` for `BigUint`: convert to R `character` (lossless)
- [x] Add feature-gated tests (miniextendr-api/tests/num_bigint.rs)

==== rust_decimal feature ====
- [x] Add `rust_decimal = { version = "1", optional = true }` to Cargo.toml
- [x] Create `rust_decimal_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `Decimal`: parse from R `character` (lossless)
- [x] Implement `IntoR` for `Decimal`: convert to R `character` (lossless)
- [x] Add `numeric` fast path with precision warning in docs
  - Now accepts REALSXP (f64), INTSXP (i32), and STRSXP (character)
  - Comprehensive docs explain precision trade-offs
  - Output always goes to character for lossless storage
- [x] Add feature-gated tests (miniextendr-api/tests/rust_decimal.rs)
  - 7 tests including numeric and integer fast paths

==== ordered-float feature ====
- [x] Add `ordered-float = { version = "4", optional = true }` to Cargo.toml
- [x] Create `ordered_float_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `OrderedFloat<f64>`: R `numeric` → Rust
- [x] Implement `IntoR` for `OrderedFloat<f64>`: Rust → R `numeric`
- [x] Implement `TryFromSexp` for `OrderedFloat<f32>`: R `numeric` → Rust
- [x] Implement `IntoR` for `OrderedFloat<f32>`: Rust → R `numeric`
- [x] Implement vector conversions: `Vec<OrderedFloat<T>>`, `Vec<Option<OrderedFloat<T>>>`
- [x] Add feature-gated tests (miniextendr-api/tests/ordered_float.rs)

==== num-traits (internal only) ====
- [ ] Optional helper for generic implementations
- [ ] NOT a public R-facing feature (internal use only)
- [ ] Consider for implementing generic numeric helpers

==== rug (LGPL + system GMP) ====
- [ ] Keep out of defaults due to LGPL license and system GMP dependency
- [ ] Document as advanced/opt-in if ever added
- [ ] Include clear license notes if implemented

=== Planned: Additional Adapter Trait Candidates ===
Source: ADAPTER_TRAITS.md pattern - applicable to many external traits

The adapter trait pattern (local trait + blanket impl) enables exporting external traits to R.
Each candidate below can follow the pattern documented in ADAPTER_TRAITS.md.

==== std library traits ====

Iterator adapter:
- [ ] Create `RIterator` adapter trait for `Iterator` (documented example in ADAPTER_TRAITS.md)
  - `next() -> Option<T>` where T: IntoR
  - `size_hint() -> (usize, Option<usize>)` as R integer vector
  - Wrap `ExactSizeIterator::len()` when available
  - Use case: Expose Rust iterators as R generator-like objects

Display/FromStr adapters:
- [x] Create `RDisplay` adapter trait for `Display`
  - `to_r_string(&self) -> String` delegating to Display::fmt
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root
- [x] Create `RFromStr` adapter trait for `FromStr`
  - `r_from_str(s: &str) -> Option<Self>` delegating to FromStr::from_str
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root

Debug adapter:
- [x] Create `RDebug` adapter trait for `Debug`
  - `debug_str(&self) -> String` and `debug_str_pretty(&self) -> String`
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root

==== Comparison trait adapters ====

- [x] Create `RPartialOrd` adapter trait for `PartialOrd`
  - `r_partial_cmp(&self, other: &Self) -> Option<i32>` returning -1/0/1/None
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
- [x] Create `ROrd` adapter trait for `Ord`
  - `r_cmp(&self, other: &Self) -> i32` returning -1/0/1
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
- [x] Create `RHash` adapter trait for `Hash`
  - `r_hash(&self) -> i64` using DefaultHasher
  - Implemented in `miniextendr-api/src/adapter_traits.rs`

==== serde trait adapters (with serde feature) ====

- [ ] Create `RSerialize` adapter trait for `serde::Serialize`
  - `r_to_json(&self) -> String` using serde_json
  - `r_to_list(&self) -> SEXP` for direct R list output (future)
  - Use case: Serialize Rust structs to R-consumable JSON
- [ ] Create `RDeserialize` adapter trait for `serde::Deserialize`
  - `r_from_json(s: &str) -> Result<Self, SexpError>`
  - Use case: Parse R character JSON into Rust types
- [ ] Consider serde_json R list bridge
  - Direct SEXP serialization without JSON intermediate
  - Similar to jsonlite's R ↔ JSON model

==== num-traits adapters (internal helpers) ====

- [ ] Create `RNum` adapter trait for common numeric operations
  - Blanket impl for `T: num_traits::Num + Clone + ToString`
  - Methods: `r_zero()`, `r_one()`, `r_is_zero()`, `r_abs()` (where applicable)
  - Use case: Generic numeric type R interfaces
- [ ] Create `RFloat` adapter trait for floating point ops
  - Blanket impl for `T: num_traits::Float`
  - Methods: `r_is_nan()`, `r_is_infinite()`, `r_floor()`, `r_ceil()`, etc.
  - Use case: Generic float operations exposed to R

==== Error trait adapters ====

- [x] Create `RError` adapter trait for `std::error::Error`
  - `error_message(&self) -> String` from Error::to_string()
  - `error_chain(&self) -> Vec<String>` walking source() chain
  - `error_chain_length(&self) -> i32` for chain length
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root

==== IO trait adapters (with connections feature) ====

- [ ] Create `RRead` adapter trait for `std::io::Read`
  - `r_read_bytes(&mut self, n: usize) -> Vec<u8>`
  - `r_read_to_end(&mut self) -> Vec<u8>`
  - Use case: R-accessible readers from Rust IO sources
- [ ] Create `RWrite` adapter trait for `std::io::Write`
  - `r_write_bytes(&mut self, data: &[u8]) -> usize`
  - `r_flush(&mut self)`
  - Use case: R-accessible writers to Rust IO sinks
- [ ] Create `RBufRead` adapter trait for `std::io::BufRead`
  - `r_read_line(&mut self) -> Option<String>`
  - `r_lines(&mut self) -> impl Iterator<Item = String>` (combined with RIterator)
  - Use case: Line-by-line reading in R

==== Collection trait adapters ====

- [ ] Create `RExtend` adapter trait for `Extend`
  - `r_extend_from_vec(&mut self, items: Vec<T>)`
  - Use case: Append R vectors to Rust collections
- [ ] Create `RIntoIterator` adapter trait for `IntoIterator`
  - Returns wrapped `RIterator` from `into_iter()`
  - Use case: Convert Rust collections into R-iterable objects

==== rand trait adapters (with rand feature) ====

- [ ] Create `RRng` adapter for `rand::Rng`
  - `r_gen_range(low: f64, high: f64) -> f64`
  - `r_gen_bool(p: f64) -> bool`
  - Use case: Access custom RNGs from R
- [ ] Create `RDistribution` adapter for `rand_distr::Distribution`
  - `r_sample(&self, rng: &mut dyn Rng) -> T`
  - Use case: Sample from Rust distributions in R

==== Documentation tasks ====

- [x] Add adapter trait examples to ADAPTER_TRAITS.md for each major category
  - Added: Display/FromStr, Debug, Comparison (Ord/PartialOrd), Hash
  - Added: Serde (Serialize/Deserialize with JSON)
  - Added: IO (Read/Write/BufRead)
  - Added: Error (with error chain walking)
- [x] Create cookbook with common adapter patterns (ADAPTER_COOKBOOK.md)
  - Recipe 1: Expose a custom iterator to R
  - Recipe 2: Serialize/deserialize custom types with serde
  - Recipe 3: Use Rust IO with R connections
  - Recipe 4: Wrap comparison for R sorting
  - Recipe 5: Expose hash for deduplication

==== rayon trait adapters (with rayon feature) ====

- [ ] Create `RParallelIterator` adapter trait for `rayon::iter::ParallelIterator`
  - `r_par_for_each(&self, f: impl Fn(T) + Sync)` - parallel iteration
  - `r_par_map(&self, f: impl Fn(T) -> U + Sync) -> Vec<U>` - parallel transform
  - `r_par_filter(&self, f: impl Fn(&T) -> bool + Sync) -> Vec<T>` - parallel filter
  - `r_par_reduce(&self, identity: T, op: impl Fn(T, T) -> T + Sync) -> T`
  - Use case: Expose Rayon's parallel iteration to R for vectorized operations
- [ ] Create `RParallelExtend` adapter trait for `rayon::iter::ParallelExtend`
  - `r_par_extend(&mut self, items: Vec<T>)` - parallel bulk insert
  - Use case: Efficient parallel collection building from R vectors

==== ndarray trait adapters (with ndarray feature) ====

- [ ] Create `RArrayBase` adapter trait for common `ndarray` operations
  - `r_shape(&self) -> Vec<usize>` - get dimensions as R integer vector
  - `r_ndim(&self) -> i32` - number of dimensions
  - `r_is_contiguous(&self) -> bool` - check memory layout
  - `r_sum(&self) -> T` where T: Sum - reduce to scalar
  - Use case: Expose array metadata and operations to R
- [ ] Create `RNdIndex` adapter for ndarray indexing
  - `r_slice(&self, start: Vec<usize>, end: Vec<usize>) -> Self` - subarray view
  - Use case: R-style array subsetting for ndarray types

==== nalgebra trait adapters (with nalgebra feature) ====

- [ ] Create `RMatrix` adapter trait for nalgebra matrix operations
  - `r_nrows(&self) -> i32`, `r_ncols(&self) -> i32`
  - `r_transpose(&self) -> Self` - matrix transpose
  - `r_determinant(&self) -> f64` where applicable
  - `r_inverse(&self) -> Option<Self>` where applicable
  - Use case: Linear algebra operations accessible from R
- [ ] Create `RVector` adapter trait for nalgebra vector operations
  - `r_norm(&self) -> f64` - Euclidean norm
  - `r_dot(&self, other: &Self) -> f64` - dot product
  - `r_normalize(&self) -> Self` - unit vector
  - Use case: Vector math operations from R

==== regex trait adapters (with regex feature) ====

- [ ] Create `RReplacer` adapter trait for `regex::Replacer`
  - `r_replace_all(&self, text: &str, replacement: &str) -> String`
  - `r_replace_first(&self, text: &str, replacement: &str) -> String`
  - Use case: Expose regex replacement API to R
- [ ] Create `RCaptures` adapter for `regex::Captures`
  - `r_group(&self, i: usize) -> Option<String>` - get capture group by index
  - `r_named_group(&self, name: &str) -> Option<String>` - get by name
  - `r_len(&self) -> i32` - number of groups
  - Use case: Access regex capture groups from R

==== time trait adapters (with time feature) ====

- [ ] Create `RDuration` adapter trait for `time::Duration`
  - `r_as_seconds(&self) -> f64` - total seconds as float
  - `r_as_millis(&self) -> i64` - total milliseconds
  - `r_components(&self) -> (i64, i32, i32, i32)` - (days, hours, minutes, seconds)
  - Use case: Time duration operations from R
- [ ] Create `RDateTimeFormat` adapter for formatting/parsing
  - `r_format(&self, fmt: &str) -> String` - format with pattern
  - `r_parse(s: &str, fmt: &str) -> Result<Self, SexpError>` - parse with pattern
  - Use case: Custom datetime formatting in R

==== bytes crate adapters (potential new feature) ====

- [ ] Add `bytes = { version = "1", optional = true }` feature (if useful)
- [ ] Create `RBuf` adapter trait for `bytes::Buf`
  - `r_remaining(&self) -> usize` - bytes remaining
  - `r_chunk(&self) -> Vec<u8>` - get current chunk
  - `r_advance(&mut self, n: usize)` - advance cursor
  - Use case: Zero-copy byte buffer access from R
- [ ] Create `RBufMut` adapter trait for `bytes::BufMut`
  - `r_put_slice(&mut self, data: &[u8])` - write bytes
  - `r_remaining_mut(&self) -> usize` - writable space
  - Use case: Efficient byte buffer writing from R

==== crossbeam channel adapters (potential new feature) ====

- [ ] Add `crossbeam-channel = { version = "0.5", optional = true }` feature (if useful)
- [ ] Create `RSender` adapter trait for channel senders
  - `r_send(&self, item: T) -> bool` - send item, return success
  - `r_try_send(&self, item: T) -> Option<T>` - non-blocking send
  - `r_is_full(&self) -> bool` - check if channel full
  - Use case: Inter-thread communication from R
- [ ] Create `RReceiver` adapter trait for channel receivers
  - `r_recv(&self) -> Option<T>` - blocking receive
  - `r_try_recv(&self) -> Option<T>` - non-blocking receive
  - `r_is_empty(&self) -> bool` - check if channel empty
  - Use case: Receive from background threads in R

==== Future/async adapters (long-term, if async support added) ====

- [ ] Create `RFuture` adapter trait for `std::future::Future`
  - `r_poll(&mut self) -> Option<T>` - check if ready (simplified poll)
  - `r_block_on(&mut self) -> T` - blocking wait (using tokio/async-std runtime)
  - Use case: Basic async/await integration with R
  - Note: Requires careful design around R's single-threaded nature

==== Clone/Copy/Default adapters ====

- [x] Create `RClone` adapter trait for `Clone`
  - `r_clone(&self) -> Self` - explicit deep copy for R
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root
- [x] Create `RCopy` adapter trait for `Copy`
  - `r_copy(&self) -> Self` - cheap bitwise copy (O(1), no heap)
  - `is_copy(&self) -> bool` - runtime type check
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root
- [x] Create `RDefault` adapter trait for `Default`
  - `r_default() -> Self` - construct default instance
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root

==== num-bigint trait adapters (with num-bigint feature) ====

- [ ] Create `RBigIntOps` adapter trait for BigInt/BigUint arithmetic
  - `r_add(&self, other: &Self) -> Self`
  - `r_sub(&self, other: &Self) -> Self`
  - `r_mul(&self, other: &Self) -> Self`
  - `r_div(&self, other: &Self) -> Self`
  - `r_rem(&self, other: &Self) -> Self`
  - `r_pow(&self, exp: u32) -> Self`
  - `r_gcd(&self, other: &Self) -> Self`
  - `r_lcm(&self, other: &Self) -> Self`
  - Use case: Arbitrary-precision arithmetic from R
- [ ] Create `RBigIntBitOps` adapter for bitwise operations
  - `r_bit_and(&self, other: &Self) -> Self`
  - `r_bit_or(&self, other: &Self) -> Self`
  - `r_bit_xor(&self, other: &Self) -> Self`
  - `r_shl(&self, n: u32) -> Self`
  - `r_shr(&self, n: u32) -> Self`
  - Use case: Bitwise operations on big integers from R

==== rust_decimal trait adapters (with rust_decimal feature) ====

- [ ] Create `RDecimalOps` adapter trait for Decimal arithmetic
  - `r_add(&self, other: &Self) -> Self`
  - `r_sub(&self, other: &Self) -> Self`
  - `r_mul(&self, other: &Self) -> Self`
  - `r_div(&self, other: &Self) -> Self`
  - `r_rem(&self, other: &Self) -> Self`
  - `r_round(&self, dp: i32) -> Self` - round to decimal places
  - `r_floor(&self) -> Self`, `r_ceil(&self) -> Self`
  - `r_trunc(&self) -> Self` - truncate toward zero
  - Use case: Fixed-precision decimal arithmetic from R
- [ ] Create `RDecimalConversions` adapter for type conversions
  - `r_to_f64(&self) -> f64` - lossy conversion to f64
  - `r_to_i64(&self) -> Option<i64>` - try conversion to i64
  - `r_scale(&self) -> i32` - number of decimal places
  - `r_mantissa(&self) -> i128` (or string for R compatibility)
  - Use case: Inspect decimal internals from R

==== uuid trait adapters (with uuid feature) ====

- [ ] Create `RUuidOps` adapter trait for UUID operations
  - `r_new_v4() -> Self` - generate random UUID
  - `r_nil() -> Self` - nil UUID
  - `r_max() -> Self` - max UUID (all 1s)
  - `r_version(&self) -> i32` - UUID version number
  - `r_variant(&self) -> i32` - UUID variant
  - `r_is_nil(&self) -> bool`
  - Use case: UUID generation and inspection from R
- [ ] Create `RUuidParse` adapter for parsing variants
  - `r_from_str(s: &str) -> Result<Self, SexpError>` - parse any format
  - `r_from_bytes(bytes: &[u8]) -> Result<Self, SexpError>` - parse from raw
  - `r_as_bytes(&self) -> Vec<u8>` - convert to raw bytes
  - Use case: UUID parsing and serialization from R

==== ordered-float trait adapters (with ordered-float feature) ====

- [ ] Create `ROrderedFloatOps` adapter trait for NaN-safe operations
  - `r_min(&self, other: &Self) -> Self` - NaN-safe minimum
  - `r_max(&self, other: &Self) -> Self` - NaN-safe maximum
  - `r_clamp(&self, min: f64, max: f64) -> Self` - clamp to range
  - `r_is_nan(&self) -> bool`, `r_is_infinite(&self) -> bool`
  - Use case: NaN-safe numeric operations from R

==== indexmap trait adapters (with indexmap feature) ====

- [ ] Create `RIndexMapOps` adapter trait for IndexMap operations
  - `r_insert(&mut self, key: String, value: T) -> Option<T>` - insert/update
  - `r_remove(&mut self, key: &str) -> Option<T>` - remove by key
  - `r_get(&self, key: &str) -> Option<&T>` - get by key
  - `r_get_index(&self, i: usize) -> Option<(&String, &T)>` - get by position
  - `r_keys(&self) -> Vec<String>` - all keys in order
  - `r_len(&self) -> i32`, `r_is_empty(&self) -> bool`
  - `r_shift_remove(&mut self, key: &str) -> Option<T>` - remove preserving order
  - Use case: Ordered dictionary operations from R
