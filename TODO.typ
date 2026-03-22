#import "@preview/cheq:0.3.0": checklist
#show: checklist

#set page(numbering: "1 of 1", paper: "a5", margin: (top: 2em, left: 1em, right: 2em, bottom: 2.2em))

== R6 Deep Integration

Basic R6 support exists (`#[miniextendr(r6)]`, active bindings, public/private).
See `plans/r6-deep-integration-plan.md` for full spec.

- [x] `lock_objects` / `lock_class` flags — fully working + tested
- [x] `cloneable` flag with deep clone hook — `#[miniextendr(r6(deep_clone))]` works
- [x] Finalizer as private member — `#[miniextendr(r6(finalize))]` works
- [x] Active binding getter+setter pairs — `#[miniextendr(r6(active))]` + `setter` works
- [x] Inheritance (`inherit = "ParentType"`) — 3-level chain tested (R6Animal → R6Dog → R6GoldenRetriever)
- [x] `portable` flag — tested (`portable = false` works)
- [~] Field-level `#[r6(public|private|skip)]` annotations
  - Architecturally impossible: proc macros can't access struct field definitions
  - Workaround: use `#[r_data]` sidecar pattern (documented in CLASS_SYSTEMS.md)

== API Gaps

- [x] Safe mutable input helpers — `CopySliceMut<T>` copy-in/copy-out + `&mut [T]` compile error
  - `&mut [T]` rejected at macro boundary with helpful error message
  - `CopySliceMut<T>`: TryFromSexp copies in, DerefMut for mutation, IntoR copies out
  - UI test: `fn_mut_slice_rejected.rs`; docs: `GAPS.md` §2.1
- [~] String ndarray/matrix conversion
  - ndarray is designed for numeric/Copy types; `String` doesn't fit the model
  - `Vec<String>` / `Vec<Vec<String>>` are the natural Rust representations
- [x] Quoted-expression evaluation helpers — `RSymbol`, `RCall`, `REnv`
  - `RCall` builder: `.arg()`, `.named_arg()`, `.eval()` via `R_tryEvalSilent`
  - `RSymbol`: interned SYMSXP wrapper; `REnv`: GlobalEnv/BaseEnv/EmptyEnv handles
- [x] S4 compatibility helpers — `s4_helpers` module with slot access wrappers
  - `s4_is`, `s4_class_name`, `s4_has_slot`, `s4_get_slot`, `s4_set_slot`

== Performance

- [x] Worker batching/context reuse API — `with_r_thread_batch` + `RThreadScope`
  - `with_r_thread_batch`: send multiple closures in one round-trip
  - `RThreadScope`: RAII context for multiple `call()` invocations on one channel open
- [~] Direct/no-wrapper export mode for hot functions
  - Already covered by `extern "C-unwind"` + `#[no_mangle]` support
  - Function IS the C symbol; R wrapper is `unsafe_` prefixed convenience only
- [x] Name-indexed list API — `NamedList` wrapper with `HashMap<String, usize>`
  - O(1) lookup via `get()`, `contains()`, `get_raw()`; `TryFromSexp` for use as fn param

== Testing

- [x] Property-based roundtrip tests — 24 proptest tests for all scalar/vector/option types
- [x] Macro codegen snapshot tests — 12 expect-test snapshots for R wrappers and class systems
- [x] Thread-safety assertions — 198 R-level worker thread tests, checked FFI wrapper
  panic tests, RAII cleanup across thread boundaries (test-worker.R, panic_tests.rs)
  - Remaining gap: RThreadBuilder direct tests skipped (crashes R runtime)
- [x] String ALTREP NA serialization — fixed in cc115a7 (use `Vec<Option<String>>`,
  register `Vec_Option_String` ALTREP class)
- [~] Worker thread test re-enablement
  - `test-thread.R` — disabled anti-pattern tests (R API calls from worker thread); correctly disabled
  - `test-thread-broken.R` — `RThreadBuilder` crashes R's stack checking; needs R-level API changes
  - Both files serve as documentation; `test-worker.R` (198 tests) covers the correct patterns
- [x] Cross-package test expansion — `Resettable` trait + `StatefulCounter` + 163 new tests
  - Multi-trait dispatch (Counter + Resettable on same type across packages)
  - Type discrimination (DoubleCounter is Counter but NOT Resettable)
  - Combined trait operations (increment-then-reset, get-reset-get)

== Build / Infrastructure

- [x] processx-based execution in minirextendr — migrated from system2()
- [ ] Windows CI debugging
  - The `-l` (login) flag in bash might change working directory
  - Path format when passing Windows paths to bash
- [x] Module `#[cfg]` friction reduction — `#[cfg(...)] use module;` in `miniextendr_module!`
  - `MiniextendrModuleUse` now parses outer attributes
  - cfg attrs applied at all 5 expansion points (CALL_ENTRIES, altrep, wrappers)

== Optional Features

=== Serialization

- [x] `borsh` optional feature for binary serialization
  - `Borsh<T>` wrapper: `IntoR` → RAWSXP, `TryFromSexp` → decode RAWSXP
  - `RBorshOps` adapter trait with blanket impl
- [ ] `rkyv` optional feature for zero-copy serialization (DEFERRED)
  - Complex: lifetime/validation issues with R's GC model

=== Adapter Traits

- [x] serde_json R list bridge — direct SEXP ↔ JsonValue conversion
  - `JsonValue` IntoR/TryFromSexp, homogeneous array optimization
  - NA/NaN/Inf handling via `JsonOptions`, factor support

=== Connections

- [x] Connections API stability — capability probing + runtime version check
  - `ConnectionCapabilities::from_sexp()`: probe can_read/write/seek, is_open/text/blocking
  - `check_connections_runtime()`: R.Version() probe for R >= 4.3.0
  - `is_binary_mode()`, `connection_mode()`, `connection_description()` helpers

=== Concurrency (POSTPONED)

- [ ] crossbeam channel adapters (`RSender`, `RReceiver`)
- [ ] Future/async adapters (`RFuture`) — requires async runtime integration
- [ ] Async-like handle model for background tasks (poll/wait/cancel)

== Documentation

- [x] Connection & progress bar guides — `docs/CONNECTIONS.md` + `docs/PROGRESS.md`
- [x] Intermediate minirextendr vignettes — `adding-rust-functions.Rmd` + `altrep-quick-start.Rmd`

== Macro Consolidation

Make both `#[miniextendr]` and derive paths available for every feature.

- [x] `#[miniextendr]` on multi-field struct → ExternalPtr
- [x] `#[miniextendr(list)]` on struct → IntoList + TryFromList + PreferList
- [x] `#[miniextendr(dataframe)]` on struct → IntoList + DataFrameRow + IntoR on companion
- [x] `#[miniextendr]` on fieldless enum → RFactor
- [x] `#[miniextendr(match_arg)]` on fieldless enum → MatchArg
- [x] `#[miniextendr(prefer = "...")]` on struct → Prefer\* markers
- [x] `#[derive(Altrep)]` on 1-field struct → ALTREP registration

== Trait Adapter Wrappers (2026-03-22)

- [x] `AsDisplay<T>` / `AsDisplayVec<T>` — `T: Display` → R character
- [x] `AsFromStr<T>` / `AsFromStrVec<T>` — R character → `T: FromStr`
- [x] `Collect<I>` / `CollectStrings<I>` — zero-alloc iterator → R vector
- [x] `AsJson<T>` / `FromJson<T>` / `AsJsonPretty<T>` / `AsJsonVec<T>` — JSON string ↔ serde
- [x] `RCondition<E>` — `std::error::Error` cause chain in R error messages
- [x] `log` feature — route `log::info!`/`warn!`/`error!` to R console
- [x] `tools/detect-features.R` — configure-time feature auto-detection
- [x] `sync_feature_rules()` in minirextendr — auto-update detect script from Cargo.toml
- [x] `alloc_r_vector<T>()` — centralized R vector allocation, no more `ptr.add` loops

== Remaining Plans

- [ ] `lazy-altrep-materialization` — `Lazy<T>` opt-in ALTREP for Arrow/ndarray/nalgebra
- [ ] `datafusion-full-integration` — DataFrame API, file I/O, UDF bridge
- [ ] `extract-feature-crates` — split optional integrations into `miniextendr-*` crates
- [ ] `par-column-chunks-two` — builder-style parallel DataFrame column processing
- [ ] `lfs-history-cleanup` — rewrite git history to use LFS for vendor.tar.xz

== Low Priority / Nice to Have

- [x] `miniextendr.yml` config file support — `mx_config()` + `mx_config_defaults()`
  - Reads YAML with fallback to defaults; warns on unknown keys / parse errors
  - Template scaffolded by `use_miniextendr_config()`; yaml package optional
- [x] `lifecycle` package for deprecation warnings — `@importFrom` tag injection
  - `import_from_fn()` maps stages to R functions (deprecate_warn, deprecate_soft, etc.)
  - `inject_lifecycle_imports()` adds `@importFrom lifecycle` roxygen tags with dedup
- [x] `num-traits` as internal helper for generic numeric implementations
  - `RNum`, `RFloat`, `RSigned` adapter traits with blanket impls
  - Feature-gated: `miniextendr-api = { features = ["num-traits"] }`
