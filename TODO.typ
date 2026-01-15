#import "@preview/cheq:0.3.0": checklist
#show: checklist

#set page(numbering: "1 of 1", paper: "a5", margin: (top: 2em, left: 1em, right: 2em, bottom: 2.2em))


- [x] Add a small assert in the macro to emit a clear compile error if the wrong base is used (e.g., using AltReal flags under base = "Int"). Implemented in `miniextendr-macros/src/lib.rs` via a base-specific trait bound check requiring the corresponding `Alt*` family trait for the selected `base`.
- [x] Only use `static` and not `static mut` for symbols from R.
  - `R_Interactive` is a challenge here.
  - Fix: Changed `static mut` to `static` with raw pointer writes via helper functions.
    - miniextendr-engine: set_r_interactive(), set_r_signal_handlers()
    - miniextendr-api: set_r_cstack_limit(), `get_r_cstack_*()` (nonapi feature-gated)
- [x] ensure all ffi'd function have the r_ffi macro that provide safe equivalents
- [x] implement proper rayon feature...
  - Generic `with_r_vec<T>` with type inference
  - RNativeType::dataptr_mut() for safe data pointer access
  - Clear documentation on parallel limitations
- [x] make sure that `miniextendr-bench` uses the common `rpkg/src/target` directory...
  - Fix: Added miniextendr-bench to workspace, updated to edition 2024, fixed REngine::new() → build()
- [x] Add storage-directed conversion helpers (value-based) that compose `TryCoerce` + `IntoR`
  - Goal: user picks storage (integer/numeric/logical/raw/character), conversions happen automatically
  - Implemented `IntoRAs<Target>` trait in `miniextendr-api/src/into_r_as.rs`
  - No lossy escape hatch - users cast manually if they want lossy behavior
  - Semantics documented in `docs/CONVERSION_SEMANTICS.md`
- [x] Allow numeric → character conversions via stringification (including NaN/Inf)
  - NaN → "NaN", Inf → "Inf", -Inf → "-Inf"
  - Logical: TRUE → "TRUE", FALSE → "FALSE", NA → "NA"
  - Applied to scalar + Vec/slice conversion paths via `IntoRAs<String>`

== Codex Review Findings (2024)

=== CRITICAL: Safety Issues
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

=== HIGH: Thread Safety
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

=== MEDIUM: Memory/Leaks
- [x] R continuation tokens are preserved forever (leak)
  - `miniextendr-api/src/worker.rs:44-51`, `unwind_protect.rs:17-24`
  - Fix: Consolidated to single global token in unwind_protect.rs (no per-thread leak)

=== API/Ergonomics
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

=== Build/Packaging
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
  - Fix: Added AC_PATH_PROG checks with error messages, use `$RSYNC/$SED` variables
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

=== Testing
- [x] Rayon integration tests too narrow (missing `with_r_vec`)
  - Fix: Added comprehensive integration tests in `miniextendr-api/tests/rayon.rs`
    using miniextendr-engine for embedded R. Tests cover `with_r_vec` (basic, parallel
    write, i32, empty, large), `Vec` parallel collect, and `IntoR` conversion.
- [ ] No automated regression test for registration bug
  - Note: User indicated this is likely a fluke, low priority.
- [x] Macro compile-fail tests missing (no trybuild/UI tests)
  - Fix: Added trybuild dev-dependency to miniextendr-macros, created tests/ui.rs runner
    and 6 compile-fail test cases: unknown_option, pattern_parameter, option_with_value,
    module_missing_mod, module_duplicate_mod, unsafe_empty
- [ ] Thread-safety assertions not covered by tests
  - Note: Would require embedded R runtime for meaningful tests.
- [x] Known TODOs not tracked as GitHub issues
  - Cleaned up: removed stale TODOs from justfile, conversions.rs, test-thread.R
  - Remaining TODOs are informational notes (lint enhancement idea, configure.ac upstream issue)

== Codex ALTREP Review (2026-01-12)

=== Correctness/Safety
- [x] Fix `Range<i32>`/`Range<i64>` `no_na()` to account for `i32::MIN` (NA sentinel)
  - `Range<i32>::no_na()` now checks if range contains NA sentinel
  - `Range<i64>::no_na()` now checks for NA sentinel and out-of-bounds values
  - `sum()`/`min()`/`max()` now properly handle NA with `na_rm` parameter
  - Fixed in `miniextendr-api/src/altrep_data/builtins.rs`
- [x] Decide overflow/NA semantics for `LazyIntSeqData`
  - `no_na()` now computes actual bounds to detect if NA sentinel is in range
  - Checks for saturation at i32::MIN in first/last elements
  - `sum()`/`min()`/`max()` return None when NAs present (let R compute)
  - Fixed in `rpkg/src/rust/lib.rs`
- [x] Confirm R ALTREP contract for NULL return values from installed methods
  - Confirmed: NULL (R_NilValue) is valid return from sum/min/max methods
  - NULL signals R to fall back to default computation (materialize vector)
  - This is documented R ALTREP behavior - no changes needed

=== Robustness
- [x] Avoid panicking on iterator length mismatch in iterator-backed ALTREP
  - Removed `assert_eq!` panic in `materialize_all()`
  - Now handles mismatch gracefully: truncates if too many, returns NA for missing
  - Prints warning to stderr when mismatch detected
  - Fixed in `miniextendr-api/src/altrep_data/iter.rs`

=== Testing
- [x] Add tests for NA sentinel handling in `Range<i32>`
  - Added tests in `rpkg/tests/testthat/test-altrep-builtins.R`
  - Tests normal ranges, negative ranges, and NA detection
- [x] Add tests for out-of-range `Range<i64>` behavior
  - Added test for Range<i64> normal case
- [x] Add tests for `LazyIntSeqData` overflow edge cases (large ranges, negative step)
  - Added tests for normal sequences, descending sequences, and near-max-int sequences

=== Safety Issues (from project-review-2026-01-04)
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

== Reviews Findings (December 2024)

=== COMPLETED (this session)
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

=== Documentation (from reviews 01, 02, 08)
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

== GC Protect Review Findings (2026-01-12)
See `reviews/gc_protect_review.md` for full context.

=== HIGH: Soundness Issues
- [x] `ReprotectSlot::set` returns stale `Root<'a>` - use-after-unprotect risk
  - `miniextendr-api/src/gc_protect.rs:629`
  - Fix: Changed `set()` to return raw `SEXP` instead of `Root<'a>`
  - Added doc comment warning that SEXP is only protected until next `set()` call
- [x] `ReprotectSlot::Deref` violates `Cell` aliasing rules - potential UB
  - `miniextendr-api/src/gc_protect.rs:663`
  - Fix: Removed `Deref` impl entirely, added comment explaining why
  - Users should use `get()` which returns SEXP by value

=== MEDIUM: Panic Safety
- [x] `tls::with_protect_scope` not panic-safe - dangling TLS pointer
  - `miniextendr-api/src/gc_protect.rs:741`
  - Fix: Added `TlsScopeGuard` struct that pops TLS stack in `Drop` impl
  - Guard ensures cleanup even during panic unwinding

=== MEDIUM: Benchmark Issues
- [x] Unprotected CHARSXP in manual string benchmarks
  - `strvec_manual_construction`: Added explicit "intentionally unsafe" warning
  - `build_named_list_realistic`: Fixed by protecting CHARSXP before use

=== Testing Gaps
- [ ] Add tests for TLS panic cleanup behavior
- [ ] Add tests for `ReprotectSlot::set` invalidation semantics

== ExternalPtr Sidecar R Wrappers (Planned Feature)
See `plans/externalptr_sidecar_r_wrappers.md` for full design.

=== Overview
Expose `#[r_data]` sidecar fields from `ExternalPtr` structs to R as getter/setter functions.

=== Implementation Tasks
- [x] Add `RSidecar` and `RData` marker types (ZST) to miniextendr-api
  - Added to `miniextendr-api/src/externalptr.rs` with full documentation
- [x] Update `#[derive(ExternalPtr)]` to emit:
  - `RDATA_CALL_DEFS_<TYPE>`: `&[R_CallMethodDef]` for sidecar accessors
  - `R_WRAPPERS_RDATA_<TYPE>`: `&str` R wrapper code
  - Parses `#[r_data]` attributes on struct fields
  - Generates getter/setter C functions for pub RData fields
- [x] Update `miniextendr_module!` to auto-include sidecar accessors when type registered
  - Added `rdata_call_defs_const_ident()` and `rdata_r_wrappers_const_ident()` to MiniextendrModuleImpl
  - Sidecar call defs included in CALL_ENTRIES array
  - Sidecar R wrappers included in R_WRAPPERS_IMPLS array
- [x] Generate R functions: `<type>_get_<field>(x)` and `<type>_set_<field>(x, value)`
  - Getter returns SEXP from prot slot
  - Setter updates prot slot and returns invisible(x)
- [x] Add compile error for generic types with `pub` `#[r_data]` fields
  - Error: "generic types with pub #[r_data] fields are not supported; .Call entrypoints cannot be generic"
- [ ] The `rdname` is not default to file-name for the sidecar impls.

=== Tests
- [ ] UI tests: multiple selector fields error, non-marker type error, generic type error
- [ ] Runtime tests: getter returns stored SEXP, setter updates and returns invisible(x)

=== Reference Study Tasks (from background/)

==== R Internals & Extensions
- [x] Study `background/R Internals.html` for SEXP type system (2026-01-12)
  - Compared against R 4.5.2 `src/include/Rinternals.h`
  - *FINDING: SEXPTYPE enum is complete* - all 22 types match exactly:
    - NILSXP(0) through LGLSXP(10), skip 11-12, INTSXP(13) through S4SXP(25)
    - Plus NEWSXP(30), FREESXP(31), FUNSXP(99)
    - OBJSXP/S4SXP aliasing (both value 25) correctly handled
  - *PROTECT patterns verified *:
    - `gc_protect` module: RAII wrappers for Rf_protect/Rf_unprotect
    - `preserve` module: R_PreserveObject/R_ReleaseObject for cross-.Call objects
    - `ExternalPtr`: R-owned Rust data with finalizers
    - ProtectScope, OwnedProtect, ReprotectSlot follow R's LIFO stack semantics
- [x] Study `background/Writing R Extensions.html` for .Call interface (2026-01-12)
  - *Registration patterns verified correct*:
    - NAMESPACE: `useDynLib(miniextendr, .registration = TRUE)` ✓
    - entrypoint.c: `R_useDynamicSymbols(dll, FALSE)` ✓
    - entrypoint.c: `R_forceSymbols(dll, TRUE)` ✓
    - `R_init_*_miniextendr(dll)` calls `R_registerRoutines()` internally ✓
  - R wrapper generation: roxygen2 generates exports from `@export` tags
  - NA handling: documented in `altrep_traits.rs` (NA_INTEGER, NA_REAL, NA_LOGICAL)
- [x] Study ALTREP documentation (`background/ALTREP_ Alternative Representations...html`)
  - Compared miniextendr ALTREP impl against R 4.5.2 `src/include/R_ext/Altrep.h`
  - *FINDING: All R ALTREP methods are implemented* in miniextendr:
    - Base: Length (required), Serialize/Unserialize, Duplicate, Coerce, Inspect
    - Vector: Dataptr, Dataptr_or_null, Extract_subset
    - Integer/Real: Elt, Get_region, Is_sorted, No_NA, Sum, Min, Max
    - Logical: Elt, Get_region, Is_sorted, No_NA, Sum (no Min/Max per R spec)
    - Raw/Complex: Elt, Get_region
    - String: Elt (required), Set_elt, Is_sorted, No_NA
    - List: Elt (required), Set_elt
  - FFI bindings in `ffi/altrep.rs` match R's ALTREP API exactly
  - `HAS_*` const gating correctly prevents unused methods from being installed

==== R Source Reference
- [x] Use `background/r-source-tags-R-4-5-2/` to verify FFI bindings (2026-01-12)
  - [x] Location: `src/include/Rinternals.h` - SEXP types verified complete
  - [x] Location: `src/include/R_ext/Altrep.h` - ALTREP bindings verified complete
  - [x] Location: `src/main/memory.c` - GC behavior studied
    - PROTECT stack: `R_PPStack` array with `R_PPStackTop` index
    - PROTECT() pushes to `R_PPStack[R_PPStackTop++]`, UNPROTECT(n) decrements by n
    - R_PreserveObject/R_ReleaseObject use `R_PreciousList` (linked list or hash table)
    - R_PreserveObject: adds via `CONS(object, R_PreciousList)` (O(1))
    - R_ReleaseObject: removes via `DeleteFromList` (O(n) traversal)
    - miniextendr's `gc_protect.rs` correctly models these patterns
  - [x] Location: `src/main/altclasses.c` - ALTREP dispatch studied
    - Concrete implementations: compact_intseq, compact_realseq, deferred_string, mmap, wrapper
    - Pattern: define methods (Length, Elt, Get_region, etc.) then register with `R_set_alt*_method()`
    - Coerce method returns NULL to signal "use default coercion" (matches miniextendr)
    - Duplicate method creates materialized copy
    - Is_sorted returns: SORTED_DECR_NA_1ST, SORTED_INCR, UNKNOWN_SORTEDNESS, etc.
    - No_NA returns TRUE/FALSE for guaranteed NA-free status
    - miniextendr's altrep_traits.rs matches these patterns correctly

==== Class System References
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

==== ALTREP Implementation References
- [ ] Study `background/Rpkg-mutable-master/` for mutable ALTREP patterns
  - How does it handle write barriers?
  - How does it handle copy-on-modify semantics?
- [ ] Study `background/Rpkg-simplemmap-master/` for memory-mapped ALTREP
  - How does it handle lazy loading?
  - How does it handle file descriptor lifecycle?
- [ ] Study `background/vectorwindow-main/` for ALTREP views
  - How does it implement subset views without copying?
  - How does it handle window lifecycle?

==== Documentation & Tooling References
- [ ] Study roxygen2 (`background/roxygen2-main/`) for R wrapper generation
  - How does roxygen2 parse `@param`, `@return`, `@export` tags?
  - Patterns for improving miniextendr-macros/src/roxygen.rs
  - Reference for R documentation generation
- [ ] Study mirai (`background/mirai-main/`) for async patterns
  - How does mirai handle clean environment evaluation?
  - Patterns for worker thread communication
  - Reference for potential async miniextendr features

=== Testing (from reviews 02, 06, 08)
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

=== Build/Infrastructure (from reviews 03, 04, 07)
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

=== Optional Enhancements (lower priority)
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

==== minirextendr Dependency Rationalization
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

==== minirextendr usethis Replacements
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
  - Reason: Template includes `@useDynLib` directive; using usethis + patching adds complexity
  - Current approach: Single template with all miniextendr-specific content

checking available recipes (`just --list`) - ALL EXIST
- [x] build, check, clean, clippy, configure, default
- [x] devtools-build, devtools-check, devtools-document, devtools-install, devtools-load, devtools-test
- [x] doc, doc-check, expand, fmt, fmt-check
- [x] r-cmd-build, r-cmd-check, r-cmd-install, test, test-r-build, tree
- [x] vendor-sync-check, lint-sync-check (new recipes added)
- [x] `minirextendr-*` recipes (build, check, dev, document, install, load, rcmdcheck, test)
- [x] `cross-*` recipes for cross-package tests
- [x] `templates-*` recipes for template management

=== Planned: Optional indicatif progress
- [x] Add `indicatif` feature to `miniextendr-api` (opt-in, non-default) with `indicatif -> nonapi` dependency
- [x] Implement `RTerm` (`indicatif::TermLike`) that writes to R console via `ptr_R_WriteConsoleEx` and no-ops off main thread
- [x] Provide ANSI cursor/clear defaults in `RTerm` (cursor moves, clear line, write_line)
- [x] Implemented `term_like_stdout()`, `term_like_stderr()` and `into_draw_target()` helpers
- [x] Updated NONAPI.md with `ptr_R_WriteConsoleEx` under feature-gated non-API functions

=== Planned: Feature shortlist from Rust ecosystem
Source: `reviews/feature-plans-uuid-time-regex-indexmap.md`, `reviews/feature-shortlist.md`

Common scaffolding for all features:
1. Add optional dep + feature in `miniextendr-api/Cargo.toml` (non-default)
2. Create feature module: `*_impl.rs`
3. Gate module in `lib.rs` with `#[cfg(feature = "...")]`
4. Add doc block per feature in `lib.rs` with example snippets
5. Add feature-gated tests under `miniextendr-api/tests/`

==== uuid feature
- [x] Add `uuid = { version = "1", optional = true, features = ["v4"] }` to Cargo.toml
- [x] Create `uuid_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `Uuid`: parse from R `character(1)`
- [x] Implement `IntoR` for `Uuid`: convert to R `character(1)`
- [x] Implement `TryFromSexp` for `Vec<Uuid>`: parse from R `character` vector
- [x] Implement `IntoR` for `Vec<Uuid>`: convert to R `character` vector
- [x] Handle `Option<Uuid>` for NA support: `NA_character_` ⇄ `None`
- [x] Map parse failures to `SexpError::InvalidValue`
- [x] Add feature-gated tests (miniextendr-api/tests/uuid.rs)

==== time feature
- [x] Add `time = { version = "0.3", optional = true, features = ["formatting", "parsing", "macros"] }` to Cargo.toml
- [x] Create `time_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `OffsetDateTime`: R `POSIXct` (numeric + tzone attr) → Rust
- [x] Implement `IntoR` for `OffsetDateTime`: Rust → R `POSIXct` with tzone (UTC)
- [x] Implement `TryFromSexp` for `time::Date`: R date (day counts since 1970-01-01)
- [x] Implement `IntoR` for `time::Date`: Rust → R Date
- [x] Fractional seconds policy: truncate (documented in module)
- [x] Add Vec and Option variants for both OffsetDateTime and Date
- [x] Add feature-gated tests (10 tests)

==== regex feature
- [x] Add `regex = { version = "1", optional = true }` to Cargo.toml
- [x] Create `regex_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `Regex`: compile from R `character(1)`
- [x] Handle `Option<Regex>` for NA support
- [x] Add `try_compile` helper (users wrap in ExternalPtr themselves for caching)
- [x] Documented `ExternalPtr<Regex>` pattern for loop reuse in module docs
- [x] Add feature-gated tests (5 tests)

==== indexmap feature
- [x] Add `indexmap = { version = "2", optional = true }` to Cargo.toml
- [x] Create `indexmap_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `IndexMap<String, T>`: R named list → Rust
- [x] Implement `IntoR` for `IndexMap<String, T>`: Rust → R named list
- [x] Preserve insertion order in both directions
- [x] Auto-name unnamed entries: "V1", "V2", ... when converting R list without names
- [x] Add feature-gated tests (5 tests)

=== Planned: External-trait export strategy
Source: `reviews/trait-export-and-numeric-crates.md`

\*Key constraint:\* Cannot directly export external (non-owned) traits to R.

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

=== Planned: Numeric crate feature candidates
Source: `reviews/trait-export-and-numeric-crates.md`

Common scaffolding (same as feature shortlist):
1. Add optional dep + feature in `miniextendr-api/Cargo.toml`
2. Create `*_impl.rs` module
3. Gate module with `#[cfg(feature = "...")]`
4. Add doc block + tests

==== num-bigint feature
- [x] Add `num-bigint = { version = "0.4", optional = true }` to Cargo.toml
- [x] Create `num_bigint_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `BigInt`: parse from R `character`
- [x] Implement `IntoR` for `BigInt`: convert to R `character` (lossless)
- [x] Implement `TryFromSexp` for `BigUint`: parse from R `character`
- [x] Implement `IntoR` for `BigUint`: convert to R `character` (lossless)
- [x] Add feature-gated tests (miniextendr-api/tests/num_bigint.rs)

==== rust_decimal feature
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

==== ordered-float feature
- [x] Add `ordered-float = { version = "4", optional = true }` to Cargo.toml
- [x] Create `ordered_float_impl.rs` in miniextendr-api/src/
- [x] Implement `TryFromSexp` for `OrderedFloat<f64>`: R `numeric` → Rust
- [x] Implement `IntoR` for `OrderedFloat<f64>`: Rust → R `numeric`
- [x] Implement `TryFromSexp` for `OrderedFloat<f32>`: R `numeric` → Rust
- [x] Implement `IntoR` for `OrderedFloat<f32>`: Rust → R `numeric`
- [x] Implement vector conversions: `Vec<OrderedFloat<T>>`, `Vec<Option<OrderedFloat<T>>>`
- [x] Add feature-gated tests (miniextendr-api/tests/ordered_float.rs)

==== num-traits (internal only)
- [ ] Optional helper for generic implementations
- [ ] NOT a public R-facing feature (internal use only)
- [ ] Consider for implementing generic numeric helpers

==== rug (LGPL + system GMP)
- [ ] Keep out of defaults due to LGPL license and system GMP dependency
- [ ] Document as advanced/opt-in if ever added
- [ ] Include clear license notes if implemented

=== Planned: Additional Adapter Trait Candidates
Source: ADAPTER_TRAITS.md pattern - applicable to many external traits

The adapter trait pattern (local trait + blanket impl) enables exporting external traits to R.
Each candidate below can follow the pattern documented in ADAPTER_TRAITS.md.

==== std library traits

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
  - `as_r_string(&self) -> String` delegating to Display::fmt
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

==== Comparison trait adapters

- [x] Create `RPartialOrd` adapter trait for `PartialOrd`
  - `r_partial_cmp(&self, other: &Self) -> Option<i32>` returning -1/0/1/None
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
- [x] Create `ROrd` adapter trait for `Ord`
  - `r_cmp(&self, other: &Self) -> i32` returning -1/0/1
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
- [x] Create `RHash` adapter trait for `Hash`
  - `r_hash(&self) -> i64` using DefaultHasher
  - Implemented in `miniextendr-api/src/adapter_traits.rs`

==== serde trait adapters (with serde feature)

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

==== num-traits adapters (with num-traits feature)

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

==== Error trait adapters

- [x] Create `RError` adapter trait for `std::error::Error`
  - `error_message(&self) -> String` from Error::to_string()
  - `error_chain(&self) -> Vec<String>` walking source() chain
  - `error_chain_length(&self) -> i32` for chain length
  - Implemented in `miniextendr-api/src/adapter_traits.rs`
  - Re-exported from crate root

==== IO trait adapters (with connections feature)

NOTE: IO adapters are provided by the connection module (`miniextendr-api/src/connection.rs`):
- `IoRead<T>` for `T: std::io::Read`
- `IoWrite<T>` for `T: std::io::Write`
- `IoBufRead<T>` for `T: std::io::BufRead`
- `IoReadWrite<T>`, `IoReadSeek<T>`, `IoWriteSeek<T>`, `IoReadWriteSeek<T>`
- Use `RConnectionIo` builder for easy creation

Standalone adapter traits not needed - use connection framework instead.

==== Collection trait adapters

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

==== rand trait adapters (with rand feature)

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

==== Documentation tasks

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

==== rayon trait adapters (with rayon feature)

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

==== ndarray trait adapters (with ndarray feature)

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
- [x] Expand ndarray support to all dimension types
  - Array0 through Array6 and ArrayD: TryFromSexp, IntoR, TypedExternal - DONE
  - ArrayView1-3, ArrayViewD: IntoR (copies to R native) - DONE
  - ArrayViewMut types: re-exported (no IntoR - mutable views rarely returned)
  - ArcArray1, ArcArray2: TryFromSexp, IntoR - DONE
  - Index types Ix0-Ix6, IxDyn: re-exported from crate root - DONE
  - RNdArrayOps for Array1/2/D<i32> in addition to f64 - DONE
  - rpkg test types: NdVec, NdMatrix, NdArrayDyn, NdIntVec - DONE
  - R test suite: rpkg/tests/testthat/test-ndarray.R - DONE

==== nalgebra trait adapters (with nalgebra feature)

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

==== regex trait adapters (with regex feature)

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

==== time trait adapters (with time feature)

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

==== bytes crate adapters (with bytes feature)

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

==== crossbeam channel adapters (potential new feature) - POSTPONED

*POSTPONED:* Do these last - complex concurrency patterns require careful design.

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

==== Future/async adapters (long-term, if async support added) - POSTPONED

*POSTPONED:* Do these last - requires async runtime integration and careful design around R's single-threaded nature.

- [ ] Create `RFuture` adapter trait for `std::future::Future`
  - `r_poll(&mut self) -> Option<T>` - check if ready (simplified poll)
  - `r_block_on(&mut self) -> T` - blocking wait (using tokio/async-std runtime)
  - Use case: Basic async/await integration with R
  - Note: Requires careful design around R's single-threaded nature

==== Clone/Copy/Default adapters

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

==== num-bigint trait adapters (with num-bigint feature)

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

==== rust_decimal trait adapters (with rust_decimal feature)

- [x] Create `RDecimalOps` adapter trait for Decimal operations
  - `as_string()`, `is_zero()`, `is_positive()`, `is_negative()`, `sign()`
  - `scale()`, `abs()`, `neg()`
  - `add_str()`, `sub_str()`, `mul_str()`, `div_str()`, `rem_str()` (string-based operands)
  - `round(dp)`, `floor()`, `ceil()`, `trunc()`, `fract()`
  - `as_f64()`, `as_i64()`, `normalize()`, `is_integer()`
  - Implemented in `miniextendr-api/src/rust_decimal_impl.rs`
  - Re-exported from crate root

==== uuid trait adapters (with uuid feature)

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

==== ordered-float trait adapters (with ordered-float feature)

- [x] Create `ROrderedFloatOps` adapter trait for NaN-safe operations
  - `into_inner()`, `is_nan()`, `is_infinite()`, `is_finite()`
  - `is_positive()`, `is_negative()`
  - `floor()`, `ceil()`, `round()`, `trunc()`, `fract()`
  - `abs()`, `signum()`
  - `min_with(other)`, `max_with(other)`, `clamp_to(min, max)`
  - Implemented in `miniextendr-api/src/ordered_float_impl.rs`
  - Re-exported from crate root

==== indexmap trait adapters (with indexmap feature)

- [x] Create `RIndexMapOps<T>` adapter trait for IndexMap operations
  - `len()`, `is_empty()`, `keys()`, `contains_key()`
  - `get_index(index) -> Option<(String, T)>` - get by position
  - `get_key_at(index) -> Option<String>` - get key at position
  - `first() -> Option<(String, T)>`, `last() -> Option<(String, T)>`
  - `get_index_of(key) -> i32` - find position of key (-1 if not found)
  - Implemented in `miniextendr-api/src/indexmap_impl.rs`
  - Re-exported from crate root

== New Optional Features (from reviews/ plans 2026-01-04)

==== aho-corasick feature

- [x] Add `aho-corasick` optional feature for multi-pattern search
  - `aho-corasick = { version = "1.1", optional = true }`
  - Create `miniextendr-api/src/aho_corasick_impl.rs`
  - `TryFromSexp for AhoCorasick` - build from `Vec<String>` patterns
  - Helpers: `aho_compile(patterns)`, `aho_find_all(ac, haystack) -> Vec<(pattern_id, start, end)>`
  - Builder: `aho_builder(patterns, ascii_case_insensitive, match_kind)`
  - `RAhoCorasickOps` adapter trait for R interop
  - Pattern IDs 1-based in R, byte offsets documented
  - Plan: `reviews/aho-corasick-plan.md`

==== bitflags feature

- [x] Add `bitflags` optional feature for flag ↔ integer conversions
  - `bitflags = { version = "2", optional = true }`
  - Create `miniextendr-api/src/bitflags_impl.rs`
  - Wrapper type: `RFlags<T>` implementing `TryFromSexp` and `IntoR`
  - `TryFromSexp for RFlags<T>` - read integer, use `T::from_bits` (strict)
  - `IntoR for RFlags<T>` - return integer with `flags.bits()`
  - Helper functions:
    - `flags_from_i32_strict(v)` - rejects unknown bits
    - `flags_from_i32_truncate(v)` - ignores unknown bits
    - `flags_to_i32(flags)` - convert to integer
  - Bit width policy: require values fit in `i32`
  - Plan: `reviews/bitflags-plan.md`

==== bitvec feature

- [x] Add `bitvec` optional feature for bit vectors ↔ logical vectors
  - `bitvec = { version = "1", optional = true }`
  - Create `miniextendr-api/src/bitvec_impl.rs`
  - Type alias: `pub type RBitVec = BitVec<u8, Lsb0>` (stable order)
  - `TryFromSexp for RBitVec` - accept LGLSXP, TRUE→1, FALSE→0, NA→error
  - `TryFromSexp for BitVec<u8, Msb0>` - MSB-first variant
  - `IntoR for RBitVec` and `BitVec<u8, Msb0>` - produce LGLSXP
  - Helper functions: `bitvec_from_bools()`, `bitvec_to_bools()`, `bitvec_count_ones/zeros()`
  - Plan: `reviews/bitvec-plan.md`

==== borsh feature

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

==== rkyv feature

- [ ] Add `rkyv` optional feature for zero-copy serialization
  - `rkyv = { version = "0.7", optional = true }`
  - `bytecheck = { version = "0.6", optional = true }` (required for safety)
  - Create `miniextendr-api/src/rkyv_impl.rs`
  - Wrapper type: `Rkyv<T>` to avoid trait conflicts
  - `IntoR for Rkyv<T>` → RAWSXP
  - `TryFromSexp for Rkyv<T>` → validate with bytecheck before deserialize
  - Always use `rkyv::check_archived_root` to avoid UB
  - Plan: `reviews/borsh-rkyv-plan.md`

==== num-complex feature

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

==== serde-json Value bridge

- [x] Add `serde-json` feature for direct Value ↔ R list conversion
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

==== sha2 feature

- [x] Add `sha2` optional feature for hashing helpers
  - `sha2 = { version = "0.10", optional = true }`
  - Create `miniextendr-api/src/sha2_impl.rs`
  - Helpers:
    - `sha256_bytes(data) -> String` (64-char hex, lowercase)
    - `sha256_str(s) -> String` (UTF-8)
    - `sha512_bytes(data) -> String` (128-char hex)
    - `sha512_str(s) -> String`
  - Vector helpers: `sha256_bytes_vec()`, `sha256_str_vec()`, `sha512_*_vec()`
  - Re-exports `Sha256`, `Sha512`, `Digest` for advanced usage
  - Plan: `reviews/sha2-plan.md`

==== tabled feature

- [x] Add `tabled` optional feature for table formatting
  - `tabled = { version = "0.20", optional = true }`
  - Create `miniextendr-api/src/tabled_impl.rs`
  - Helpers:
    - `table_to_string(rows)`, `table_to_string_opts(rows, max_width, align, trim)`
    - `table_to_string_styled(rows, style)`, `builder_to_string(builder)`
    - `table_from_vecs(headers, rows)`
  - `impl IntoR for tabled::Table` → STRSXP
  - Plan: `reviews/tabled-plan.md`

==== toml feature

- [x] Add `toml` optional feature for TOML value conversions
  - `toml = { version = "0.8", optional = true }`
  - Create `miniextendr-api/src/toml_impl.rs`
  - Type alias: `pub type TomlValue = toml::Value`
  - Functions: `toml_from_str(s)`, `toml_to_string(v)`, `toml_to_string_pretty(v)`
  - `TryFromSexp for TomlValue` - from character(1)
  - `IntoR for TomlValue` - to list/vector
  - `RTomlOps` adapter trait for value inspection
  - Plan: `reviews/toml-plan.md`

==== url feature

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

==== raw_conversions feature (bytemuck-based)

- [x] Add `raw_conversions` optional feature for POD ↔ raw vector
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

==== enum-as-factors (proc-macro)

- [x] Add `#[derive(RFactor)]` for Rust enums ↔ R factors
  - Added derive macro in `miniextendr-macros/src/factor_derive.rs`
  - Created `miniextendr-api/src/factor.rs` module
  - Trait: `pub trait RFactor: Copy + 'static`
    - `const LEVELS: &'static [&'static str]`
    - `fn to_level_index(self) -> i32` (1-based)
    - `fn from_level_index(idx: i32) -> Option<Self>`
  - Helpers: `build_factor()`, `build_levels_sexp()`, `build_levels_sexp_cached()`, `factor_from_sexp()`
  - Derive generates: `impl RFactor`, `impl IntoR`, `impl TryFromSexp`
  - Newtype wrappers: `FactorVec<T>`, `FactorOptionVec<T>` for Vec conversions
  - NA handling: NA_INTEGER → None via Option<T>
  - Derive attributes:
    - `#[r_factor(rename = "...")]` - rename variant level string
    - `#[r_factor(rename_all = "...")]` - rename all variants (snake_case, SCREAMING_SNAKE_CASE, etc.)
    - `#[r_factor(interaction = ["A", "B", ...])]` - interaction factor with inner RFactor type
    - `#[r_factor(sep = "_")]` - custom separator for interaction levels (default ".")
  - Interaction factors:
    - Outer enum wraps inner RFactor type: `enum Outer { A(Inner), B(Inner) }`
    - Combined levels: `["A.Inner1", "A.Inner2", ..., "B.Inner1", "B.Inner2", ...]`
    - Lex-order indexing: `outer_idx * n_inner + inner_idx + 1`
    - Compile-time const assertion validates specified levels match inner type's LEVELS
  - Slice access types:
    - `Factor<'a>` - immutable view into factor integer array (`Deref<Target=[i32]>`)
    - `FactorMut<'a>` - mutable view with validation on set
  - Performance: Symbol-based CHARSXP caching with OnceLock (~6-8x speedup)
    - `build_levels_sexp_cached()` caches levels STRSXP in static OnceLock
    - Uses `Rf_install()` for permanent GC protection (no R_ReleaseObject needed)
  - Validation: only fieldless (C-style) enums
  - Benchmarks: `miniextendr-bench/benches/factor.rs`
  - Tests: `rpkg/src/rust/factor_tests.rs`
  - Plan: `reviews/enum-as-factors-plan.md`

== Test Infrastructure (from reviews/ plans)

==== rpkg adapter trait tests

- [x] Add feature pass-throughs in `rpkg/src/rust/Cargo.toml.in`
  - Pass-through for all optional features to miniextendr-api
  - Completed: rayon, rand, rand_distr, either, ndarray, nalgebra, serde, num-bigint,
    rust_decimal, ordered-float, uuid, regex, indexmap, time, num-traits, bytes,
    bitvec, bitflags, num-complex, sha2, tabled, toml, url, aho-corasick, raw_conversions
- [x] Enable all features by default in dev mode (NOT_CRAN=true)
  - Updated `configure.ac` to set MINIEXTENDR_FEATURES automatically
- [x] Add `rpkg_enabled_features()` function to return compiled feature list
  - Added to `rpkg/src/rust/lib.rs`
- [x] Add R helper `rpkg_has_feature(name)` and `skip_if_missing_feature(name)`
  - Added `rpkg/R/feature_helpers.R`
- [x] Create `rpkg/src/rust/adapter_traits_tests.rs` with test types:
  - Point: RDebug, RDisplay, RHash, ROrd, RClone, RDefault, RFromStr, RCopy
  - MyFloat: RPartialOrd (NaN handling)
  - ChainedError: RError (error chain walking)
  - IntVecIter: RIterator (with RefCell interior mutability)
  - GrowableVec: RExtend
  - IntSet: RFromIter, RToVec
  - IterableVec/IterableVecIter: RMakeIter
- [x] Add R tests in `rpkg/tests/testthat/test-adapter-traits.R`
- [x] Add per-feature adapter demos (cfg-gated):
  - ordered-float, num-bigint, rust_decimal, uuid, regex, indexmap, time
  - Rust: `rpkg/src/rust/*_adapter_tests.rs`
  - R tests: `rpkg/tests/testthat/test-feature-adapters.R`
- [x] Add per-feature R tests using `skip_if_missing_feature()`
- Plan: `reviews/rpkg-adapter-tests-plan.md`

== API Cleanup

==== Remove `r_` prefix from adapter trait methods

- [x] Remove `r_` prefix from adapter trait method names
  - Changed: `r_clone()` → `clone()`, `r_to_vec()` → `to_vec()`, `r_next()` → `next()`, etc.
  - Completed in `miniextendr-api/src/adapter_traits.rs`
  - Note: `RClone::clone` and `RDefault::default` have same signatures as std traits, so
    ambiguity requires `Clone::clone(&x)` syntax when calling std trait in scope
  - Remaining: Update feature `*_impl.rs` files, rpkg wrappers, and documentation

== Coerce Integration (from coerce-coverage-review-2026-01-04)

==== Feature module Coerce/TryCoerce integration

- [x] Add `Coerce`/`TryCoerce` impls for feature types
  - `impl Coerce<OrderedFloat<f64>> for f64`
  - `impl TryCoerce<OrderedFloat<f32>> for f64` (precision-loss check)
  - `impl Coerce<Decimal> for i32`
  - `impl TryCoerce<Decimal> for f64` (precision loss/error)
  - `impl Coerce<BigInt> for i32`
  - `impl TryCoerce<BigInt> for i64` (if needed)
  - Implemented in: `ordered_float_impl.rs`, `rust_decimal_impl.rs`, `num_bigint_impl.rs`
- [ ] Use `Coerced<T, R>` in feature `TryFromSexp` impls
  - Standardize error messages and NA handling
  - Replace manual parsing in `ordered_float_impl`, `rust_decimal_impl`, `num_bigint_impl`
- [ ] Document per-feature coercion policy
  - Clarify integer inputs for float-centric types
  - Document truncation/rounding behavior
  - Note lossy vs strict conversions
- [x] Add TryCoerce tests for feature types
  - `f64 -> OrderedFloat<f32>` errors on precision loss (miniextendr-api/tests/ordered_float.rs)
  - `i32 -> Decimal` succeeds; `f64 -> Decimal` errors on NaN/Inf (miniextendr-api/tests/rust_decimal.rs)
  - `i32 -> BigInt` succeeds; `f64 -> BigInt` errors on fractional/NaN (miniextendr-api/tests/num_bigint.rs)

== minirextendr Enhancements

==== Feature Detection Generator

- [x] Add `rust_enabled_features()` generator to minirextendr
  - Create function that generates `rpkg_enabled_features()` equivalent for user packages
  - Function should scan Cargo.toml features and generate matching Rust code
  - Allow re-running to sync with Cargo.toml feature changes
  - R wrapper: `minirextendr::update_feature_detection()`
  - Should generate both Rust code and R helper (`has_feature()`, `skip_if_missing_feature()`)
  - Implemented in `minirextendr/R/use-feature.R`: `use_feature_detection()`, `update_feature_detection()`


== minirextendr

- [x] Add a `cargo_new` command. The manifest-path argument isn't defined for `cargo new `. Instead, you'll have to navigate to the `src/rust` directory to execute a `cargo new` that is workspace aware, and so on.
  - Implemented in `minirextendr/R/cargo.R`: `cargo_new()`, `find_workspace_root()`, `add_crate_to_workspace()`

== ALTREP Serialization Gaps - FIXED

=== Problem (RESOLVED)

The `AltrepSerialize` trait was implemented on data types (Vec, Box, Range) but the bridge to R's `Serialized_state` ALTREP method was not connected.

=== Solution Implemented

1. Added `serialize` option to macros: `impl_altlogical_from_data!`, `impl_altraw_from_data!`, `impl_altstring_from_data!`
2. Added `AltrepSerialize` implementations for Range<i32>, Range<i64>, Range<f64>
3. Updated all builtin type macro invocations to use the `serialize` option

Types now correctly serialize via `Serialized_state` method:
- [x] Vec<i32>, Vec<f64>, Vec<String>, Vec<Rcomplex> - dataptr + serialize
- [x] Box<[i32]>, Box<[f64]>, Box<[String]>, Box<[Rcomplex]> - dataptr + serialize
- [x] Vec<bool>, Vec<u8> - serialize (no DATAPTR needed)
- [x] Box<[bool]>, Box<[u8]> - serialize (no DATAPTR needed)
- [x] Range<i32>, Range<i64>, Range<f64> - serialize (lazy ranges)

=== Test Coverage

Tests exist in `rpkg/tests/testthat/test-altrep-serialization.R`:
- All tests should pass now that serialization bridge is implemented
- Remove any remaining `skip()` calls after verification

== Windows CI Debugging (In Progress)

=== Current Status
- Pushed fix to `rpkg/bootstrap.R` that captures configure.win output (commit 06be75c)
- CI run 21034458320 in progress, Windows job pending

=== What to Check
- [ ] Look at Windows job logs from run 21034458320 for actual configure.win error
- [ ] The error was previously hidden due to `stdout = ""` suppressing output

=== Potential Issues to Investigate
- The `-l` (login) flag in bash might change working directory
- Path format when passing Windows paths to bash
- Something in configure script itself failing

=== To Revert Debugging Changes

After finding the issue, revert the verbose debugging in `rpkg/bootstrap.R`:

```bash
git revert 06be75c
```

Or manually simplify back to minimal output:
- In `run_cmd` function: change `stdout = TRUE, stderr = TRUE` back to `stdout = "", stderr = ""`
- In Windows section: remove the extra `message()` calls and simplify `system2()` call

== Deep Integration Plans (See plans/ directory)

Three comprehensive plans for deeper R class system integration. Each has detailed implementation specs in `plans/`.

=== vctrs Integration (`plans/vctrs-deep-integration-plan.md`)
*Status: Runtime support complete, proc-macro derive not started*

*Goal:* `#[derive(Vctrs)]` to auto-generate vctrs-compatible S3 classes from Rust types.

*Current state:*
- Runtime: `new_vctr()`, `new_rcrd()`, `new_list_of()` in `miniextendr-api/src/vctrs.rs`
- Traits: `VctrsClass`, `IntoVctrs`, `VctrsRecord`, `VctrsListOf`

*To implement:*
- [ ] `#[derive(Vctrs)]` proc-macro
- [ ] Auto-generate R methods: `format.<class>`, `vec_ptype_abbr.<class>`
- [ ] Auto-generate `vec_proxy.<class>`, `vec_restore.<class>`
- [ ] Auto-generate `vec_ptype2.<class>.<other>`, `vec_cast.<class>.<other>`
- [ ] Record type support (`base = "record"`)
- [ ] List-of type support (`base = "list"`)
- [ ] Proxy equal/compare/order methods
- [ ] Arithmetic/math method generation

=== R6 Integration (`plans/r6-deep-integration-plan.md`)
*Status: Basic support exists, advanced features not started*

*Goal:* Generate full R6 classes from Rust with minimal annotations.

*Current state:*
- Basic R6Class generation from `#[miniextendr(r6)]`
- Active bindings with `#[miniextendr(r6(active))]`
- Public/private member support

*To implement:*
- [ ] Inheritance (`inherit = ParentType`)
- [ ] `portable` / `non_portable` flags
- [ ] `lock_objects` / `lock_class` flags
- [ ] `cloneable` flag with deep clone hook
- [ ] Finalizer as private member
- [ ] Active binding getter+setter pairs
- [ ] Field-level `#[r6(public|private|skip)]` annotations

=== S7 Integration (`plans/s7-computed-properties-plan.md`)
*Status: Basic support exists, comprehensive features not started*

*Goal:* Full S7 class generation with property inference, generics, and validation.

*Current state:*
- Basic `new_class()` generation from `#[miniextendr(s7)]`
- Property support via `#[externalptr(s7)]`

*To implement (5 phases):*
- [ ] Phase 1: Property inference + accessor wiring
- [ ] Phase 2: Validation, defaults, required/frozen/deprecated patterns
- [ ] Phase 3: Generics (multi-dispatch, no-dots, optional/required args)
- [ ] Phase 4: `convert()` from Rust `From`/`TryFrom`, S3/S4 interop
- [ ] Phase 5: Docs, tests, stabilization

=== Recommended Starting Point

*vctrs `#[derive(Vctrs)]`* is the best first target because:
1. Runtime foundation already exists
2. Self-contained proc-macro addition
3. Doesn't require modifying existing class system code
4. Clear test coverage via `rpkg/tests/testthat/test-vctrs-*.R`
