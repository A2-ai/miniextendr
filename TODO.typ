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
- [ ] `#[miniextendr]` has no support for methods (`self`)
  - `miniextendr-macros/src/lib.rs:203-206`
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
- [ ] Template/generated files can drift (.in vs generated)
  - Fix: CI check to ensure generated files up-to-date
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
- [ ] Add SAFETY.md documenting FFI/thread safety invariants for Send wrapper types
  - `reviews/01_miniextendr-api.md` section "Invariant documentation"
  - Location: Top-level SAFETY.md or miniextendr-api/SAFETY.md
  - Content needed:
    - Document why `SendWrapper<Sexp>` is safe (what invariants it upholds)
    - Document `ExternalPtr<T>` Send/Sync story
    - Document worker thread model and when R APIs can be called
    - Document unwind protection requirements
    - Cross-reference from relevant code with `// SAFETY:` comments
- [ ] Add macro expansion pipeline documentation
  - `reviews/02_miniextendr-macros.md` section "Macro expansion pipeline"
  - Location: miniextendr-macros/src/lib.rs module docs or separate ARCHITECTURE.md
  - Content needed:
    - Flow diagram: `#[miniextendr]` → ParsedFn → C wrapper + R wrapper + registration
    - Document the 5 class system generators (Env, R6, S3, S4, S7)
    - Document `miniextendr_module!` expansion stages
    - Document ALTREP trait derivation pipeline
- [ ] Consolidate R_init_* requirements into one doc
  - `reviews/08_repo-tooling-docs-tests.md` section "Docs improvements"
  - Location: Top-level ENTRYPOINT.md or in miniextendr-api docs
  - Content needed:
    - What must be called in R_init_* (miniextendr_worker_init, trait_abi::init_ccallables)
    - When it's safe to call which APIs (main thread only vs worker-routed)
    - Order of initialization requirements
    - Example minimal R_init_* function

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
- [ ] Add snapshot/golden tests for R wrapper generation
  - `reviews/02_miniextendr-macros.md` section "Testing gaps"
  - Location: miniextendr-macros/tests/snapshots/ or similar
  - Purpose: Catch unintentional R wrapper shape regressions
  - Implementation:
    - Use `expect_test` or similar snapshot testing crate
    - Test cases for: basic function, impl block methods, all 5 class systems
    - Test cases for: roxygen tag propagation, parameter defaults, generic overrides
    - Compare generated R wrapper strings against golden files
- [ ] Add CI check for generated file hygiene
  - `reviews/06_rpkg.md` section "Generated file hygiene"
  - `reviews/08_repo-tooling-docs-tests.md` section "Repo hygiene task"
  - Purpose: Ensure generated build outputs (Makevars, entrypoint.c, Cargo.toml) not committed
  - Implementation:
    - CI script that fails if certain generated paths are tracked in git
    - Check: rpkg/src/Makevars, rpkg/src/entrypoint.c, rpkg/src/rust/Cargo.toml
    - Check: rpkg/src/rust/.cargo/config.toml, rpkg/src/rust/target/
- [ ] Add CI for cross-package trait ABI tests
  - `reviews/06_rpkg.md` section "Suggested next checks"
  - `reviews/08_repo-tooling-docs-tests.md` section "cross-package tests"
  - Purpose: Validate trait ABI stability across package boundaries
  - Implementation:
    - GitHub Actions workflow that builds/installs/tests producer.pkg and consumer.pkg
    - Run on at least one OS (Linux recommended for CI speed)
    - Triggered on changes to miniextendr-api trait_abi module

=== Build/Infrastructure (from reviews 03, 04, 07) ===
- [ ] Add REngineBuilder::r_home(PathBuf) to bypass R RHOME shell-out
  - `reviews/03_miniextendr-engine.md` section "R discovery policy"
  - Location: miniextendr-engine/src/lib.rs
  - Purpose: More flexible R home resolution for custom installs/CI
  - Also: Surface stderr on all R command failures for better diagnostics
- [ ] Add linking strategy documentation
  - `reviews/03_miniextendr-engine.md` section "Linking/rpath policy"
  - Location: miniextendr-engine/README.md or top-level LINKING.md
  - Content: Document "mirror R CMD LINK" behavior, rpath decisions per platform
- [ ] Consider processx-based execution in minirextendr
  - `reviews/07_minirextendr.md` section "system2() portability"
  - Location: minirextendr/R/*.R
  - Purpose: Better cross-platform command execution with proper quoting/output capture
  - Note: processx is common in R tooling ecosystem

=== Optional Enhancements (lower priority) ===
- [ ] Add more lint rules to miniextendr-lint
  - `reviews/04_miniextendr-lint.md` section "Lint surface area"
  - Candidates:
    - "exported item exists but not listed in miniextendr_module!"
    - "listed item does not exist / is cfg'd out"
    - "trait ABI: init_ccallables() not called in R_init_*" (if detectable)
- [ ] Add bench environment documentation
  - `reviews/05_miniextendr-bench.md` section "Bench reproducibility"
  - Location: miniextendr-bench/README.md
  - Content: Recommended bench environment, R version capture, sessionInfo logging
- [ ] Add integration test for minirextendr workflow
  - `reviews/07_minirextendr.md` section "Tests focus on templates"
  - Purpose: Test "configure ran but didn't generate expected files" scenarios
  - Implementation: Create temp project, run workflow subset, verify outputs

==== minirextendr Dependency Rationalization ====
Source: `reviews/dependency-idiomaticity.md`

Strong fit (replace manual code):
- [ ] Replace manual `git init` in `create.R:98-103` with `gert::git_init()` or `usethis::use_git()`
- [ ] Replace `jsonlite::fromJSON()` in `vendor.R:12-35` with `gh::gh()` for GitHub API
  - Benefits: automatic pagination, auth token handling, rate limit awareness
  - Enables removal of jsonlite dependency
- [ ] Replace manual gsub templater in `utils.R:152-179` with `whisker::whisker.render()` or `usethis::use_template()`

Good fit (add functionality):
- [ ] Add persistent cache for downloaded tarballs using `rappdirs::user_cache_dir("minirextendr")` in `vendor.R:52-65`
  - Reduces repeated downloads of same crate versions
- [ ] Improve project detection in `utils.R:23-63` with `rprojroot::find_root(rprojroot::has_file("Cargo.toml"))`
  - Current: only checks current/parent dir
  - Improved: walks up tree to find project root

Optional:
- [ ] Add `miniextendr.yml` config file support for user defaults using `yaml` package
  - Store: crate name, rpkg name, version, features
- [ ] Add `clipr` for copying "next steps" commands to clipboard
- [ ] Add `lifecycle` for deprecation warnings and API evolution

==== minirextendr usethis Replacements ====
Source: `reviews/usethis-replacements.md`

- [ ] Replace hand-built DESCRIPTION in `create.R:133` with `usethis::use_description(fields = list(...))`
  - Use `withr::with_dir(rpkg_path)` for monorepo targeting
  - Keep `desc` for in-place edits if needed
- [ ] Replace manual `.Rbuildignore` append in `use-r.R:69` with `usethis::use_build_ignore(template_lines, escape = FALSE)`
- [ ] Replace manual `.gitignore` append in `use-r.R:100` with `usethis::use_git_ignore(template_lines, directory = ".")`
- [ ] Replace custom `use_template()` in `utils.R:140` with `usethis::use_template()`
  - Replace custom gsub logic with whisker templating
- [ ] Replace custom `ensure_dir()` in `utils.R:320` with `usethis::use_directory()`
- [ ] Update package doc template in `use-r.R:10` to use `usethis::use_package_doc()` + patch for `@useDynLib`

checking available recipes (`just --list`)
- [x] build \*cargo_flags           // # [alias: cargo-build]
- [x] check \*cargo_flags           // # [alias: cargo-check]
- [x] clean \*cargo_flags           // # [alias: cargo-clean]
- [x] clippy \*cargo_flags          // # [alias: cargo-clippy]
- [ ] configure                    // # Run ./configure and vendor rpkg deps
- [ ] default 
- [ ] devtools-build               // # Build rpkg with devtools::build
- [ ] devtools-check               // # Check rpkg with devtools::check
- [ ] devtools-document            // # Document rpkg with devtools::document
- [ ] devtools-install             // # Install rpkg with devtools::install
- [ ] devtools-load                // # [alias: devtools-load_all]
- [ ] devtools-test FILTER=""      // # Load and test rpkg with devtools
- [ ] doc \*cargo_flags             // # [alias: cargo-doc]
- [ ] doc-check \*cargo_flags       // # [alias: cargo-doc-check]
- [ ] expand \*cargo_flags          // # [alias: cargo-expand]
- [ ] fmt \*cargo_flags             // # [alias: cargo-fmt]
- [ ] fmt-check \*cargo_flags       // # [alias: cargo-fmt-check]
- [ ] r-cmd-build \*args            // # [alias: rcmdbuild]
- [ ] r-cmd-check \*args            // # [alias: rcmdcheck]
- [ ] r-cmd-install \*args          // # [alias: rcmdinstall]
- [ ] test \*args                   // # [alias: cargo-test]
- [ ] test-r-build                 // # Build R package tarball
- [ ] tree \*cargo_flags            // # [alias: cargo-tree]
- [ ] vendor                       // # [alias: cargo-vendor]
- [ ] vendor-rpkg                  // # - bootstrap.R (CRAN tarball builds)

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
- [ ] Document adapter-trait pattern for exporting non-owned traits to R
  - Define local wrapper trait mirroring desired subset of external trait
  - Provide blanket impl for types implementing the external trait
  - Example pattern:
    ```rust
    #[miniextendr]
    pub trait RNum {
        fn add(&self, other: &Self) -> Self;
        fn to_string(&self) -> String;
    }
    impl<T: Num + Clone + ToString> RNum for T { ... }
    ```
- [ ] Provide example wrapper trait + blanket impl pattern in docs/reviews
- [ ] Clarify trait ABI constraints:
  - No generic parameters on traits
  - No async methods
  - No generic methods
  - Argument/return types must implement `TryFromSexp`/`IntoR`
  - Static methods allowed (but don't go through vtable)
- [ ] Document newtype wrapper as alternative for total control and explicit conversions

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
- [ ] Optional: Add `numeric` fast path with precision warning in docs
- [x] Add feature-gated tests (miniextendr-api/tests/rust_decimal.rs)

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
