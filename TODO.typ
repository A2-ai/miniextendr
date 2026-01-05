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

=== Safety Issues (from project-review-2026-01-04) ===
- [x] DOCUMENT: `charsxp_to_str` assumes UTF-8 encoding
  - `miniextendr-api/src/from_r.rs:30`
  - Added "Encoding Assumption" section documenting UTF-8 requirement
  - Suggests `Rf_translateCharUTF8()` or `from_utf8()` for external data
- [x] DOCUMENT: `Vec<String>` conversion maps NA/invalid UTF-8 to empty strings
  - `miniextendr-api/src/from_r.rs:1002`
  - Added "NA and Encoding Handling" warning in doc comment
  - Recommends `Vec<Option<String>>` for NA-aware paths
- [x] DOCUMENT: Named list → map drops elements with NA/empty names
  - `miniextendr-api/src/from_r.rs:893`
  - Added "NA and Empty Name Handling" warning with example of data loss
  - Recommends `Vec<(String, V)>` for full preservation

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
- [x] Create `RIterator` adapter trait for `Iterator`
  - `r_next() -> Option<T>` where T: IntoR
  - `r_size_hint() -> (i64, Option<i64>)` - lower and upper bounds
  - `r_count()`, `r_collect_n(n)`, `r_skip(n)`, `r_nth(n)` - convenience methods
  - Note: No blanket impl because Iterator::next() requires &mut self
  - Users implement manually with interior mutability (RefCell, Mutex)
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root

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

- [x] Create `RSerialize` adapter trait for `serde::Serialize`
  - `r_to_json(&self) -> Result<String, String>` - compact JSON
  - `r_to_json_pretty(&self) -> Result<String, String>` - pretty-printed JSON
  - Implemented in `miniextendr-api/src/serde_impl.rs`
  - Re-exported from crate root
- [x] Create `RDeserialize` adapter trait for `serde::Deserialize`
  - `r_from_json(s: &str) -> Option<Self>` - returns None on failure
  - `r_from_json_result(s: &str) -> Result<Self, String>` - with error details
  - Implemented in `miniextendr-api/src/serde_impl.rs`
  - Re-exported from crate root
- [x] Added serde_json to serde feature dependencies
  - `serde` feature now includes `serde_json` automatically
- [ ] Consider serde_json R list bridge
  - Direct SEXP serialization without JSON intermediate
  - Similar to jsonlite's R ↔ JSON model

==== num-traits adapters (with num-traits feature) ====

- [x] Create `RNum` adapter trait for common numeric operations
  - Blanket impl for `T: num_traits::Num + Clone`
  - Methods: `r_zero()`, `r_one()`, `r_is_zero()`, `r_is_one()`
  - Implemented in `miniextendr-api/src/num_traits_impl.rs`
  - Re-exported from crate root
- [x] Create `RSigned` adapter trait for signed number operations
  - Blanket impl for `T: num_traits::Signed + Clone`
  - Methods: `r_abs()`, `r_signum()`, `r_is_positive()`, `r_is_negative()`
  - Implemented in `miniextendr-api/src/num_traits_impl.rs`
  - Re-exported from crate root
- [x] Create `RFloat` adapter trait for floating point ops
  - Blanket impl for `T: num_traits::Float`
  - Classification: `r_is_nan()`, `r_is_infinite()`, `r_is_finite()`, `r_is_normal()`, etc.
  - Rounding: `r_floor()`, `r_ceil()`, `r_round()`, `r_trunc()`, `r_fract()`
  - Math: `r_abs()`, `r_signum()`, `r_sqrt()`, `r_cbrt()`
  - Exp/Log: `r_exp()`, `r_exp2()`, `r_ln()`, `r_log2()`, `r_log10()`
  - Trig: `r_sin()`, `r_cos()`, `r_tan()`, `r_asin()`, `r_acos()`, `r_atan()`
  - Hyperbolic: `r_sinh()`, `r_cosh()`, `r_tanh()`, `r_asinh()`, `r_acosh()`, `r_atanh()`
  - Special: `r_infinity()`, `r_neg_infinity()`, `r_nan()`, `r_min_value()`, `r_max_value()`, `r_epsilon()`
  - Power: `r_powi()`, `r_powf()`, `r_recip()`
  - Implemented in `miniextendr-api/src/num_traits_impl.rs`
  - Re-exported from crate root

==== Error trait adapters ====

- [x] Create `RError` adapter trait for `std::error::Error`
  - `error_message(&self) -> String` from Error::to_string()
  - `error_chain(&self) -> Vec<String>` walking source() chain
  - `error_chain_length(&self) -> i32` for chain length
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root

==== IO trait adapters (with connections feature) ====

NOTE: IO adapters are provided by the connection module (`miniextendr-api/src/connection.rs`):
- `IoRead<T>` for `T: std::io::Read`
- `IoWrite<T>` for `T: std::io::Write`
- `IoBufRead<T>` for `T: std::io::BufRead`
- `IoReadWrite<T>`, `IoReadSeek<T>`, `IoWriteSeek<T>`, `IoReadWriteSeek<T>`
- Use `RConnectionIo` builder for easy creation

Standalone adapter traits not needed - use connection framework instead.

==== Collection trait adapters ====

- [x] Create `RExtend` adapter trait for `Extend`
  - `r_extend_from_vec(&self, items: Vec<T>)` - extend with items from vector
  - `r_extend_from_slice(&self, items: &[T])` - extend from slice (Clone items)
  - `r_len(&self) -> i64` - get current length (optional, -1 if unknown)
  - No blanket impl (requires interior mutability like RIterator)
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root
- [x] Create `RFromIter` adapter trait for `FromIterator`
  - `r_from_vec(items: Vec<T>) -> Self` - create collection from vector
  - Blanket impl for all `FromIterator` types
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root
- [x] Create `RToVec` adapter trait for collection-to-vector conversion
  - `r_to_vec(&self) -> Vec<T>` - collect elements by cloning (non-consuming)
  - `r_len(&self) -> i64` - get element count
  - `r_is_empty(&self) -> bool` - check if empty
  - Blanket impl using HRTB for `&C: IntoIterator<Item = &T>` where `T: Clone`
  - Complement to `RFromIter` (create from vec vs extract to vec)
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root
  - Added 4 unit tests
- [x] Create `RMakeIter` adapter trait for iterator factory
  - `r_make_iter(&self) -> I` - create new iterator wrapper (I: RIterator)
  - Use case: Create independent iterators from collections
  - No blanket impl (requires user-defined iterator type with interior mutability)
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root
  - Added 2 unit tests

==== rand trait adapters (with rand feature) ====

- [x] Create `RRngOps` adapter trait for exposing custom RNGs to R
  - `r_random_f64()` - Random float in [0, 1)
  - `r_random_i32()` - Random i32 (full range)
  - `r_random_bool()` - Random boolean (50/50)
  - `r_gen_range_f64(low, high)` - Random float in [low, high)
  - `r_gen_range_i32(low, high)` - Random integer in [low, high)
  - `r_gen_bool(p)` - Bernoulli trial with probability p
  - `r_random_f64_vec(n)` - Generate n random floats
  - `r_gen_range_f64_vec(n, low, high)` - Generate n floats in range
  - `r_gen_range_i32_vec(n, low, high)` - Generate n integers in range
  - `r_gen_bool_vec(n, p)` - Generate n booleans
  - No blanket impl (requires interior mutability like RIterator)
  - Implemented in `miniextendr-api/src/rand_impl.rs`
  - Re-exported from crate root
- [x] Create `RDistributionOps` adapter for probability distributions
  - `r_sample(&self) -> T` - draw single sample (distribution owns RNG)
  - `r_sample_n(&self, n) -> Vec<T>` - draw n samples
  - `r_sample_vec(&self, n) -> Vec<T>` - alias for r_sample_n
  - `r_mean(&self) -> Option<f64>` - theoretical mean
  - `r_variance(&self) -> Option<f64>` - theoretical variance
  - `r_std_dev(&self) -> Option<f64>` - derived from variance
  - No blanket impl (requires interior mutability for RNG)
  - Implemented in `miniextendr-api/src/rand_impl.rs`
  - Re-exported from crate root
  - Added 7 unit tests

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
- [x] Add `miniextendr_module!` registration examples to all adapter trait docs
  - Updated: adapter_traits.rs (13 traits), num_traits_impl.rs (3 traits), rand_impl.rs (2 traits)
  - Updated: serde_impl.rs (2 traits), time_impl.rs (1 trait), regex_impl.rs (1 trait)
  - Updated: rust_decimal_impl.rs, ordered_float_impl.rs, indexmap_impl.rs, uuid_impl.rs
  - Updated: num_bigint_impl.rs (2 traits), ndarray_impl.rs (3 traits), nalgebra_impl.rs (2 traits)
  - Each trait example now shows the required `miniextendr_module! { impl Trait for Type; }` block

==== rayon trait adapters (with rayon feature) ====

- [x] Create `RParallelIterator` adapter trait for parallel iteration
  - Associated type `Item: Send + Sync + Copy` for element type
  - `r_par_iter(&self) -> impl ParallelIterator<Item = Self::Item>` - core method
  - `r_par_len(&self) -> i32` - element count
  - Aggregations: `r_par_sum()`, `r_par_sum_int()`, `r_par_sum_i64()`, `r_par_mean()`
  - Min/Max: `r_par_min()`, `r_par_max()`, `r_par_min_f64()`, `r_par_max_f64()`
  - Statistics: `r_par_count()`, `r_par_product()`, `r_par_variance()`, `r_par_std_dev()`
  - Predicates: `r_par_any_gt()`, `r_par_all_gt()`, `r_par_any_lt()`, `r_par_all_lt()`
  - Counting: `r_par_count_gt()`, `r_par_count_lt()`, `r_par_count_eq()`
  - Filtering: `r_par_filter_gt()`, `r_par_filter_lt()`
  - Transforms: `r_par_scale()`, `r_par_offset()`, `r_par_clamp()`
  - Math: `r_par_abs()`, `r_par_sqrt()`, `r_par_pow()`, `r_par_ln()`, `r_par_exp()`
  - No blanket impl (requires `r_par_iter()` implementation)
  - Implemented in `miniextendr-api/src/rayon_bridge.rs`
  - Re-exported from crate root
  - Added 17 unit tests
- [x] Create `RParallelExtend` adapter trait for parallel collection extension
  - `r_par_extend(&self, items: Vec<T>)` - extend collection in parallel
  - `r_par_extend_from_slice(&self, items: &[T])` - extend from slice
  - `r_par_len(&self) -> i32`, `r_par_is_empty(&self) -> bool`
  - `r_par_clear(&self)`, `r_par_reserve(&self, additional: i32)`
  - No blanket impl (requires interior mutability)
  - Implemented in `miniextendr-api/src/rayon_bridge.rs`
  - Re-exported from crate root
  - Added 4 unit tests

==== ndarray trait adapters (with ndarray feature) ====

- [x] Create `RNdArrayOps` adapter trait for common `ndarray` operations
  - `len()`, `is_empty()`, `ndim()`, `shape()` - array metadata
  - `sum()`, `mean()`, `min()`, `max()`, `product()` - reductions
  - `var()`, `std()` - statistical operations
  - Implemented for `Array1<f64>`, `Array2<f64>`, `ArrayD<f64>`
  - Implemented in `miniextendr-api/src/ndarray_impl.rs`
  - Re-exported from crate root
- [x] Create `RNdSlice` adapter trait for 1D array element access
  - `r_get(&self, index) -> Option<T>` - get element by index
  - `r_first(&self) -> Option<T>`, `r_last(&self) -> Option<T>` - endpoints
  - `r_slice_1d(&self, start, end) -> Vec<T>` - extract range
  - `r_get_many(&self, indices) -> Vec<Option<T>>` - batch access
  - `r_is_valid_index(&self, index) -> bool` - bounds check
  - Implemented for `Array1<f64>`, `Array1<i32>`
  - Implemented in `miniextendr-api/src/ndarray_impl.rs`
  - Re-exported from crate root
  - Added 5 unit tests
- [x] Create `RNdSlice2D` adapter trait for 2D array row/column access
  - `r_get_2d(&self, row, col) -> Option<T>` - get element
  - `r_row(&self, row) -> Vec<T>` - extract row
  - `r_col(&self, col) -> Vec<T>` - extract column
  - `r_diag(&self) -> Vec<T>` - extract diagonal
  - `r_nrows(&self) -> i32`, `r_ncols(&self) -> i32` - dimensions
  - Implemented for `Array2<f64>`, `Array2<i32>`
  - Implemented in `miniextendr-api/src/ndarray_impl.rs`
  - Re-exported from crate root
  - Added 4 unit tests
- [x] Create `RNdIndex` adapter for n-dimensional ndarray indexing
  - `r_get_nd(&self, indices: Vec<i32>) -> Option<T>` - element at n-dimensional index
  - `r_slice_nd(&self, start: Vec<i32>, end: Vec<i32>) -> Option<Vec<T>>` - subarray extraction
  - `r_shape_nd(&self) -> Vec<i32>`, `r_ndim(&self) -> i32`, `r_len_nd(&self) -> i32`
  - `r_flatten(&self) -> Vec<T>` - Fortran (column-major) order for R compatibility
  - `r_flatten_c(&self) -> Vec<T>` - C (row-major) order
  - `r_axis_slice(&self, axis: i32, index: i32) -> Vec<T>` - slice along axis
  - `r_reshape(&self, new_shape: Vec<i32>) -> Option<Vec<T>>`
  - Implemented for `ArrayD<f64>`, `ArrayD<i32>`
  - Implemented in `miniextendr-api/src/ndarray_impl.rs`
  - Added 8 unit tests
- [ ] Expand ndarray support to all dimension types
  - Currently only Array1, Array2, ArrayD supported
  - Add support for Array0 through Array6 (fixed dimensions)
  - Add ArrayView types: ArrayView0 through ArrayViewD (read-only views)
  - Add ArrayViewMut types: ArrayViewMut0 through ArrayViewMutD (mutable views)
  - Add ArcArray types: ArcArray1, ArcArray2 (shared ownership)
  - Index helper functions: Ix0() through Ix6(), IxDyn()
  - All type aliases defined in ndarray's `type_aliases.rs`

==== nalgebra trait adapters (with nalgebra feature) ====

- [x] Create `RMatrixOps` adapter trait for nalgebra DMatrix operations
  - `nrows()`, `ncols()`, `shape()`, `is_square()`, `is_empty()`
  - `transpose()`, `determinant()`, `trace()`, `diagonal()`, `norm()`
  - `try_inverse()` - returns Option for singular matrices
  - `sum()`, `mean()`, `min()`, `max()`, `scale()`
  - `add()`, `sub()`, `mul()`, `component_mul()`
  - `row_sum()`, `column_sum()`, `row_mean()`, `column_mean()`
  - Implemented in `miniextendr-api/src/nalgebra_impl.rs`
  - Re-exported from crate root
- [x] Create `RVectorOps` adapter trait for nalgebra DVector operations
  - `len()`, `is_empty()`
  - `norm()`, `norm_squared()`, `norm_l1()`, `norm_linf()`
  - `sum()`, `mean()`, `min()`, `max()`, `argmin()`, `argmax()`
  - `dot()`, `normalize()`, `scale()`
  - `add()`, `sub()`, `component_mul()`
  - Implemented in `miniextendr-api/src/nalgebra_impl.rs`
  - Re-exported from crate root

==== regex trait adapters (with regex feature) ====

- [x] Create `RRegexOps` adapter trait for `regex::Regex`
  - `replace_first(&self, text, replacement) -> String`
  - `replace_all(&self, text, replacement) -> String`
  - `is_match(&self, text) -> bool`
  - `find(&self, text) -> Option<String>`
  - `find_all(&self, text) -> Vec<String>`
  - `split(&self, text) -> Vec<String>`
  - `captures_len(&self) -> i32`
  - Implemented in `miniextendr-api/src/regex_impl.rs`
  - Re-exported from crate root
- [x] Create `RCaptureGroups` adapter for capture group access
  - `CaptureGroups::capture(re, text) -> Option<Self>` - capture from regex
  - `get(&self, i) -> Option<String>` - get capture group by index
  - `get_named(&self, name) -> Option<String>` - get by name
  - `len(&self) -> i32`, `is_empty(&self) -> bool`
  - `all_groups(&self) -> Vec<Option<String>>`
  - Implemented in `miniextendr-api/src/regex_impl.rs`
  - Re-exported from crate root

==== time trait adapters (with time feature) ====

- [x] Create `RDuration` adapter trait for `time::Duration`
  - `as_seconds_f64(&self) -> f64` - total seconds as float
  - `as_milliseconds(&self) -> i64` - total milliseconds
  - `whole_days(&self) -> i64`, `whole_hours(&self) -> i64`, `whole_minutes(&self) -> i64`, `whole_seconds(&self) -> i64`
  - `subsec_nanoseconds(&self) -> i32` - nanosecond component
  - `is_negative(&self) -> bool`, `is_zero(&self) -> bool`, `abs(&self) -> Duration`
  - Implemented in `miniextendr-api/src/time_impl.rs`
  - Use case: Time duration operations from R
- [x] Create `RDateTimeFormat` adapter for formatting/parsing
  - `r_format(&self, fmt: &str) -> Result<String, String>` - format with pattern
  - `r_parse(s: &str, fmt: &str) -> Result<Self, String>` - parse with pattern
  - Implemented for both `OffsetDateTime` and `Date`
  - Located in `miniextendr-api/src/time_impl.rs`
  - Re-exported from crate root
  - Use case: Custom datetime formatting in R

==== bytes crate adapters (with bytes feature) ====

- [x] Add `bytes = { version = "1", optional = true }` feature
- [x] Create `RBuf` adapter trait for `bytes::Buf`
  - `r_remaining(&self) -> i32` - bytes remaining
  - `r_has_remaining(&self) -> bool` - any bytes remaining
  - `r_get_u8()`, `r_get_i8()` - read single byte
  - `r_get_u16()`, `r_get_u16_le()`, `r_get_i16()`, `r_get_i16_le()` - read 16-bit
  - `r_get_u32()`, `r_get_u32_le()`, `r_get_i32()`, `r_get_i32_le()` - read 32-bit
  - `r_get_u64()`, `r_get_u64_le()`, `r_get_i64()`, `r_get_i64_le()` - read 64-bit
  - `r_get_f32()`, `r_get_f32_le()`, `r_get_f64()`, `r_get_f64_le()` - read floats
  - `r_chunk(&self) -> Vec<u8>` - get current chunk
  - `r_copy_to_vec(&self, len) -> Vec<u8>` - copy bytes advancing cursor
  - `r_advance(&self, cnt)` - advance cursor
  - `r_to_vec(&self) -> Vec<u8>` - read all remaining
  - No blanket impl (requires interior mutability like RIterator)
  - Implemented in `miniextendr-api/src/bytes_impl.rs`
  - Re-exported from crate root
- [x] Create `RBufMut` adapter trait for `bytes::BufMut`
  - `r_remaining_mut(&self) -> i32` - writable space
  - `r_has_remaining_mut(&self) -> bool` - any space remaining
  - `r_put_u8()`, `r_put_i8()` - write single byte
  - `r_put_u16()`, `r_put_u16_le()`, `r_put_i16()`, `r_put_i16_le()` - write 16-bit
  - `r_put_u32()`, `r_put_u32_le()`, `r_put_i32()`, `r_put_i32_le()` - write 32-bit
  - `r_put_u64()`, `r_put_u64_le()`, `r_put_i64()`, `r_put_i64_le()` - write 64-bit
  - `r_put_f32()`, `r_put_f32_le()`, `r_put_f64()`, `r_put_f64_le()` - write floats
  - `r_put_slice(&self, src: Vec<u8>)` - write bytes
  - `r_put_bytes(&self, val, n)` - write n copies of byte
  - `r_reserve(&self, additional)` - reserve capacity
  - `r_len(&self) -> i32`, `r_is_empty(&self) -> bool`, `r_clear(&self)`
  - No blanket impl (requires interior mutability like RIterator)
  - Implemented in `miniextendr-api/src/bytes_impl.rs`
  - Re-exported from crate root
- [x] Re-exports `Bytes`, `BytesMut`, `Buf`, `BufMut` from bytes crate
- [x] Added 14 unit tests covering read/write operations

==== crossbeam channel adapters (potential new feature) - POSTPONED ====

**POSTPONED:** Do these last - complex concurrency patterns require careful design.

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

==== Future/async adapters (long-term, if async support added) - POSTPONED ====

**POSTPONED:** Do these last - requires async runtime integration and careful design around R's single-threaded nature.

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

- [x] Create `RBigIntOps` adapter trait for BigInt arithmetic
  - `as_string()`, `is_zero()`, `is_positive()`, `is_negative()`, `sign()`
  - `bit_length()`, `abs()`, `neg()`
  - `add_str()`, `sub_str()`, `mul_str()`, `div_str()`, `rem_str()` (string-based operands)
  - `pow(exp: u32)`, `gcd_str()`
  - `to_bytes_be()`, `to_bytes_le()`
  - Implemented in `miniextendr-api/src/num_bigint_impl.rs`
  - Re-exported from crate root
- [x] Create `RBigUintOps` adapter trait for BigUint arithmetic
  - Same operations as RBigIntOps but for unsigned integers
  - Additional: `is_one()`
  - Implemented in `miniextendr-api/src/num_bigint_impl.rs`
  - Re-exported from crate root
- [x] Create `RBigIntBitOps` adapter for BigInt bitwise operations
  - `bit_and_str()`, `bit_or_str()`, `bit_xor_str()` (string-based operands)
  - `bit_not()`, `shl()`, `shr()`
  - `count_ones()`, `trailing_zeros()`
  - `bit()`, `set_bit()`, `clear_bit()`
  - Implemented in `miniextendr-api/src/num_bigint_impl.rs`
  - Re-exported from crate root
- [x] Create `RBigUintBitOps` adapter for BigUint bitwise operations
  - Same operations as RBigIntBitOps (except bit_not)
  - Implemented in `miniextendr-api/src/num_bigint_impl.rs`
  - Re-exported from crate root

==== rust_decimal trait adapters (with rust_decimal feature) ====

- [x] Create `RDecimalOps` adapter trait for Decimal operations
  - `as_string()`, `is_zero()`, `is_positive()`, `is_negative()`, `sign()`
  - `scale()`, `abs()`, `neg()`
  - `add_str()`, `sub_str()`, `mul_str()`, `div_str()`, `rem_str()` (string-based operands)
  - `round(dp)`, `floor()`, `ceil()`, `trunc()`, `fract()`
  - `as_f64()`, `as_i64()`, `normalize()`, `is_integer()`
  - Implemented in `miniextendr-api/src/rust_decimal_impl.rs`
  - Re-exported from crate root

==== uuid trait adapters (with uuid feature) ====

- [x] Create `RUuidOps` adapter trait for UUID operations
  - `version(&self) -> i32` - UUID version number
  - `variant(&self) -> String` - UUID variant
  - `is_nil(&self) -> bool`, `is_max(&self) -> bool`
  - `as_bytes(&self) -> Vec<u8>` - convert to raw bytes
  - `to_hyphenated(&self) -> String`, `to_simple(&self) -> String`, `to_urn(&self) -> String`
  - Implemented in `miniextendr-api/src/uuid_impl.rs`
  - Re-exported from crate root
- [x] Create `uuid_helpers` module for UUID creation
  - `new_v4() -> Uuid` - generate random UUID
  - `nil() -> Uuid`, `max() -> Uuid` - special UUIDs
  - `from_bytes(bytes) -> Result<Uuid, String>` - parse from raw
  - `parse_str(s) -> Result<Uuid, String>` - parse any format
  - Implemented in `miniextendr-api/src/uuid_impl.rs`
  - Re-exported from crate root

==== ordered-float trait adapters (with ordered-float feature) ====

- [x] Create `ROrderedFloatOps` adapter trait for NaN-safe operations
  - `into_inner()`, `is_nan()`, `is_infinite()`, `is_finite()`
  - `is_positive()`, `is_negative()`
  - `floor()`, `ceil()`, `round()`, `trunc()`, `fract()`
  - `abs()`, `signum()`
  - `min_with(other)`, `max_with(other)`, `clamp_to(min, max)`
  - Implemented in `miniextendr-api/src/ordered_float_impl.rs`
  - Re-exported from crate root

==== indexmap trait adapters (with indexmap feature) ====

- [x] Create `RIndexMapOps<T>` adapter trait for IndexMap operations
  - `len()`, `is_empty()`, `keys()`, `contains_key()`
  - `get_index(index) -> Option<(String, T)>` - get by position
  - `get_key_at(index) -> Option<String>` - get key at position
  - `first() -> Option<(String, T)>`, `last() -> Option<(String, T)>`
  - `get_index_of(key) -> i32` - find position of key (-1 if not found)
  - Implemented in `miniextendr-api/src/indexmap_impl.rs`
  - Re-exported from crate root

== New Optional Features (from reviews/ plans 2026-01-04) ==

==== aho-corasick feature ====

- [ ] Add `aho-corasick` optional feature for multi-pattern search
  - `aho-corasick = { version = "1.1", optional = true }`
  - Create `miniextendr-api/src/aho_corasick_impl.rs`
  - `TryFromSexp for AhoCorasick` - build from `Vec<String>` patterns
  - `IntoR for AhoCorasick` - wrap in ExternalPtr via `impl_typed_external!`
  - Helpers: `aho_compile(patterns)`, `aho_find_all(ac, haystack) -> Vec<(pattern_id, start, end)>`
  - Optional builder for `ascii_case_insensitive` and `match_kind` options
  - R wrappers: `aho_compile()`, `aho_find_all_df()`, `aho_find_all_mat()`
  - Pattern IDs 1-based in R, byte offsets documented
  - Plan: `reviews/aho-corasick-plan.md`

==== bitflags feature ====

- [ ] Add `bitflags` optional feature for flag ↔ integer conversions
  - `bitflags = { version = "2", optional = true }`
  - Create `miniextendr-api/src/bitflags_impl.rs`
  - Wrapper types: `RFlags<T>`, `RFlagsVec<T>` to avoid blanket impl conflicts
  - `TryFromSexp for RFlags<T>` - read integer, use `T::from_bits`
  - `IntoR for RFlags<T>` - return integer with `flags.bits()`
  - Default: strict bits (unknown bits cause error)
  - Optional truncating helper: `flags_from_bits_truncate()`
  - Bit width policy: require values fit in `i32`; RFlags64 optional for u64
  - Plan: `reviews/bitflags-plan.md`

==== bitvec feature ====

- [ ] Add `bitvec` optional feature for bit vectors ↔ logical vectors
  - `bitvec = { version = "1", optional = true }`
  - Create `miniextendr-api/src/bitvec_impl.rs`
  - Type alias: `pub type RBitVec = BitVec<u8, Lsb0>` (stable order)
  - `TryFromSexp for RBitVec` - accept LGLSXP, TRUE→1, FALSE→0, NA→error
  - `IntoR for RBitVec` - produce LGLSXP
  - Optional raw mapping (deferred): RAWSXP with `bit_length` attribute
  - Plan: `reviews/bitvec-plan.md`

==== borsh feature ====

- [ ] Add `borsh` optional feature for binary serialization
  - `borsh = { version = "1", optional = true }`
  - Create `miniextendr-api/src/borsh_impl.rs`
  - Wrapper type: `Borsh<T>` to avoid trait conflicts
  - `IntoR for Borsh<T>` where `T: BorshSerialize` → RAWSXP
  - `TryFromSexp for Borsh<T>` where `T: BorshDeserialize` → decode RAWSXP
  - Helpers: `borsh_to_raw()`, `borsh_from_raw()`
  - Map decode errors to `SexpError::InvalidValue`
  - Optional `mx_version` attribute for versioning
  - Plan: `reviews/borsh-rkyv-plan.md`

==== rkyv feature ====

- [ ] Add `rkyv` optional feature for zero-copy serialization
  - `rkyv = { version = "0.7", optional = true }`
  - `bytecheck = { version = "0.6", optional = true }` (required for safety)
  - Create `miniextendr-api/src/rkyv_impl.rs`
  - Wrapper type: `Rkyv<T>` to avoid trait conflicts
  - `IntoR for Rkyv<T>` → RAWSXP
  - `TryFromSexp for Rkyv<T>` → validate with bytecheck before deserialize
  - Always use `rkyv::check_archived_root` to avoid UB
  - Plan: `reviews/borsh-rkyv-plan.md`

==== num-complex feature ====

- [x] Add `num-complex` optional feature for complex number support
  - `num-complex = { version = "0.4", optional = true }`
  - Create `miniextendr-api/src/num_complex_impl.rs`
  - Helpers: `to_rcomplex()`, `from_rcomplex()`, `na_rcomplex()`, `is_na_rcomplex()`, `is_na_complex()`
  - `TryFromSexp for Complex<f64>` - from CPLXSXP scalar
  - `TryFromSexp for Vec<Complex<f64>>` - from CPLXSXP vector
  - `TryFromSexp for Option<Complex<f64>>` - NA handling
  - `TryFromSexp for Vec<Option<Complex<f64>>>` - NA-aware vectors
  - `IntoR` impls for all above
  - NA detection: either part is `NA_REAL` (use `to_bits()` comparison)
  - `RComplexOps` adapter trait (re, im, norm, norm_sqr, arg, is_finite, is_infinite, is_nan, is_normal, conj, inv)
  - Plan: `reviews/num-complex-plan.md`

==== serde-json Value bridge ====

- [ ] Add `serde-json` feature for direct Value ↔ R list conversion
  - Feature already exists; enhance with Value ↔ SEXP bridge
  - Create/update `miniextendr-api/src/serde_json_impl.rs`
  - Type alias: `pub type JsonValue = serde_json::Value`
  - Functions: `json_from_sexp(sexp)`, `json_into_sexp(value)`
  - `TryFromSexp for serde_json::Value`, `IntoR for serde_json::Value`
  - R → JSON mapping:
    - NULL → Null, scalars → primitives, vectors → Arrays, named lists → Objects
    - NA → Null (default), NaN/Inf → error (default)
    - Factors → String via levels
  - JSON → R mapping:
    - Null → R_NilValue, Bool → LGLSXP, Number → INTSXP/REALSXP
    - String → STRSXP, Array → VECSXP, Object → named VECSXP
  - Optional helpers: `json_from_sexp_strict()`, `json_from_sexp_permissive()`
  - Plan: `reviews/serde-json-plan.md`

==== sha2 feature ====

- [ ] Add `sha2` optional feature for hashing helpers
  - `sha2 = { version = "0.10", optional = true }`
  - Create `miniextendr-api/src/sha2_impl.rs`
  - Helpers:
    - `sha256_raw(bytes) -> String` (hex, lowercase)
    - `sha256_str(s) -> String` (UTF-8)
    - `sha512_raw(bytes) -> String`
    - `sha512_str(s) -> String`
  - Optional vector helpers: `sha256_raw_vec()`, `sha256_str_vec()`
  - R wrappers: `sha256_raw()`, `sha256_str()`, `sha512_*`
  - NA strings → error (no hashing of NA)
  - Plan: `reviews/sha2-plan.md`

==== tabled feature ====

- [ ] Add `tabled` optional feature for table formatting
  - `tabled = { version = "0.20", optional = true, default-features = true }`
  - Create `miniextendr-api/src/tabled_impl.rs`
  - Helpers:
    - `table_to_string<T: Tabled>(rows)` → String
    - `table_builder_to_string(builder)` → String
  - Optional: `impl IntoR for tabled::Table` → STRSXP
  - Optional formatting options: `max_width`, `align`, `trim`
  - Default: no ANSI styling (R consoles may not render)
  - Plan: `reviews/tabled-plan.md`

==== toml feature ====

- [ ] Add `toml` optional feature for TOML value conversions
  - `toml = { version = "0.8", optional = true }`
  - Create `miniextendr-api/src/toml_impl.rs`
  - Type alias: `pub type TomlValue = toml::Value`
  - Functions: `toml_from_str(s)`, `toml_to_string(v)`
  - `TryFromSexp for TomlValue` - from character(1) or list
  - `IntoR for TomlValue` - to list or character(1)
  - R → TOML mapping:
    - NULL → error (TOML has no null)
    - Scalars → primitives, lists → Tables (require names)
    - Vectors → Arrays (must be homogeneous)
    - NA → error (default), mixed arrays → error
  - TOML → R mapping:
    - Primitives → scalars, Arrays → vectors/lists, Tables → named lists
    - Datetime → character(1)
  - Plan: `reviews/toml-plan.md`

==== url feature ====

- [x] Add `url` optional feature for URL parsing/validation
  - `url = { version = "2", optional = true }`
  - Create `miniextendr-api/src/url_impl.rs`
  - `TryFromSexp for url::Url` - from character(1), strict validation
  - `TryFromSexp for Option<url::Url>` - NA → None
  - `TryFromSexp for Vec<url::Url>` and `Vec<Option<url::Url>>`
  - `IntoR for url::Url` - character(1) via `url.as_str()`
  - `RUrlOps` adapter trait (scheme, host, port, path, query, fragment, etc.)
  - Helpers in `url_helpers`: `parse()`, `join()`, `is_valid()`
  - Strict validation: invalid URLs error
  - Plan: `reviews/url-plan.md`

==== raw_conversions feature (bytemuck-based) ====

- [ ] Add `raw_conversions` optional feature for POD ↔ raw vector
  - `bytemuck = { version = "1", optional = true, features = ["derive"] }`
  - Create `miniextendr-api/src/raw_conversions.rs`
  - Core types:
    - `Raw<T>` - validated typed wrapper
    - `RawSlice<T>` - validated vector wrapper
    - `RawTagged<T>` / `RawSliceTagged<T>` - with header metadata
  - Traits: `IntoRawBytes`, `TryFromRawBytes`
  - Error type: `RawError` (LengthMismatch, AlignmentMismatch, InvalidHeader, TypeMismatch)
  - Safety: alignment checks mandatory, length checks mandatory
  - Fast format: headerless native layout (not portable)
  - Tagged format: header with magic/version/elem_size/elem_count
  - Plan: `reviews/raw-conversions-plan.md`

==== enum-as-factors (proc-macro) ====

- [ ] Add `#[derive(RFactor)]` for Rust enums ↔ R factors
  - Add derive macro in `miniextendr-macros`
  - Create `miniextendr-api/src/factor.rs` module
  - Global `FACTOR_SYMBOL` (cached SYMSXP)
  - Trait: `pub trait RFactor: Copy + 'static`
    - `const LEVELS: &'static [&'static str]`
    - `fn to_level_index(self) -> i32` (1-based)
    - `fn from_level_index(idx: i32) -> Option<Self>`
  - Helpers: `build_levels_sexp()`, `factor_class_vec()`, `build_factor()`
  - Trait impls: `IntoR for T`, `IntoR for Vec<T>`, `IntoR for Vec<Option<T>>`
  - `TryFromSexp` impls with `Rf_isFactor` validation
  - Option<T>: NA_INTEGER → None
  - Derive attributes: `#[r_factor(rename = "...")]`, `#[r_factor(rename_all = "...")]`
  - Validation: only fieldless (C-style) enums
  - Plan: `reviews/enum-as-factors-plan.md`

== Test Infrastructure (from reviews/ plans) ==

==== rpkg adapter trait tests ====

- [x] Add feature pass-throughs in `rpkg/src/rust/Cargo.toml.in`
  - Pass-through for all optional features to miniextendr-api
  - Completed: rayon, rand, rand_distr, either, ndarray, nalgebra, serde, num-bigint,
    rust_decimal, ordered-float, uuid, regex, indexmap, time, num-traits, bytes
- [ ] Add `rpkg_enabled_features()` function to return compiled feature list
- [ ] Add R helper `rpkg_has_feature(name)` and `skip_if_missing_feature(name)`
- [ ] Create `rpkg/src/rust/adapter_traits_tests.rs` with test types:
  - RDebug/RDisplay: `DebugType`, `DisplayType`
  - RHash/ROrd/RPartialOrd: `HashType`, `OrdType`, `PartialOrdType`
  - RError: `MyError` with source chain
  - RFromStr: `IpAddrType`
  - RClone/RCopy/RDefault: `CloneType`, `CopyType`, `DefaultType`
  - RIterator: `IterVec(RefCell<...>)`
  - RExtend/RFromIter: `ExtendVec`, `FromIterSet`
- [ ] Add R tests in `rpkg/tests/testthat/test-adapter-traits.R`
- [ ] Add per-feature adapter demos (cfg-gated):
  - ordered-float, num-bigint, rust_decimal, uuid, regex, indexmap, time
- [ ] Add per-feature R tests using `skip_if_missing_feature()`
- Plan: `reviews/rpkg-adapter-tests-plan.md`

== API Cleanup ==

==== Remove `r_` prefix from adapter trait methods ====

- [x] Remove `r_` prefix from adapter trait method names
  - Changed: `r_clone()` → `clone()`, `r_to_vec()` → `to_vec()`, `r_next()` → `next()`, etc.
  - Completed in `miniextendr-api/src/adapter_traits.rs`
  - Note: `RClone::clone` and `RDefault::default` have same signatures as std traits, so
    ambiguity requires `Clone::clone(&x)` syntax when calling std trait in scope
  - Remaining: Update feature `*_impl.rs` files, rpkg wrappers, and documentation

== Coerce Integration (from coerce-coverage-review-2026-01-04) ==

==== Feature module Coerce/TryCoerce integration ====

- [ ] Add `Coerce`/`TryCoerce` impls for feature types
  - `impl Coerce<OrderedFloat<f64>> for f64`
  - `impl TryCoerce<OrderedFloat<f32>> for f64` (precision-loss check)
  - `impl Coerce<Decimal> for i32`
  - `impl TryCoerce<Decimal> for f64` (precision loss/error)
  - `impl Coerce<BigInt> for i32`
  - `impl TryCoerce<BigInt> for i64` (if needed)
- [ ] Use `Coerced<T, R>` in feature `TryFromSexp` impls
  - Standardize error messages and NA handling
  - Replace manual parsing in `ordered_float_impl`, `rust_decimal_impl`, `num_bigint_impl`
- [ ] Document per-feature coercion policy
  - Clarify integer inputs for float-centric types
  - Document truncation/rounding behavior
  - Note lossy vs strict conversions
- [ ] Add TryCoerce tests for feature types
  - `f64 -> OrderedFloat<f32>` errors on precision loss
  - `i32 -> Decimal` succeeds; `f64 -> Decimal` errors on NaN/Inf
  - `i32 -> BigInt` succeeds; `f64 -> BigInt` errors on fractional/NaN
