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
- [ ] Add `indicatif` feature to `miniextendr-api` (opt-in, non-default) with `indicatif -> nonapi` dependency
- [ ] Implement `RTerm` (`indicatif::TermLike`) that writes to R console via `ptr_R_WriteConsoleEx` and no-ops off main thread
- [ ] Provide ANSI cursor/clear defaults in `RTerm` (cursor moves, clear line, write_line)
- [ ] Add convenience constructors (`term_like_{stdout,stderr}[_with_hz]`) for stream routing
- [ ] Update NONAPI.md with new console hook usage

=== Planned: Feature shortlist from Rust ecosystem ===
- [ ] `uuid` feature: `uuid::Uuid` <-> R `character` (scalar + vector), plus ExternalPtr cache option
- [ ] `time` feature: `time::OffsetDateTime` / `time::Date` <-> R `POSIXct` / `Date`
- [ ] `regex` feature: `Regex` from R `character` + optional compiled cache via ExternalPtr
- [ ] `indexmap` feature: `IndexMap<String, T>` <-> R named list (order-preserving; auto-name when missing)
