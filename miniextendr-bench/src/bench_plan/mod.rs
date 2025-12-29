//! Comprehensive benchmark plan for `miniextendr-bench`.
//!
//! This module is documentation-only. It lays out the bench files, modules,
//! fixtures, and parameter matrices that should exist, but does not include any
//! executable benchmark code.
//!
//! ---------------------------------------------------------------------------
//! Planned bench targets (files under `miniextendr-bench/benches/`)
//!
//! Each target should call `miniextendr_bench::init()` and run on the init
//! thread. Targets should be organized by topic to keep iteration times low and
//! to allow selective runs (e.g., `cargo bench --bench from_r`).
//!
//! - `ffi_calls.rs`        Raw R API calls vs checked wrappers
//! - `sexp_ext.rs`         `SexpExt` helpers vs raw pointers
//! - `into_r.rs`           Rust -> R conversions (scalars, vectors, strings)
//! - `from_r.rs`           R -> Rust conversions (scalars, slices, maps, sets)
//! - `strings.rs`          Encoding and string extraction variants
//! - `coerce.rs`           Coerce / TryCoerce cost and error paths
//! - `altrep.rs`           ALTREP callbacks and data access patterns
//! - `altrep_iter.rs`      Iterator-backed ALTREP performance
//! - `externalptr.rs`      ExternalPtr creation, access, tagging
//! - `trait_abi.rs`        mx_erased / trait vtable query and dispatch
//! - `preserve.rs`         Preserve-list vs PROTECT/UNPROTECT patterns
//! - `unwind_protect.rs`   with_r_unwind_protect overhead (normal and error)
//! - `worker.rs`           worker-thread dispatch overhead vs direct calls
//! - `allocator.rs`        RAllocator vs System allocator (when applicable)
//! - `rayon.rs`            rayon_bridge parallel helpers (feature-gated)
//! - `connections.rs`      Custom connections (feature-gated)
//! - `wrappers.rs`         R wrapper call overhead (optional, via R eval)
//!
//! ---------------------------------------------------------------------------
//! Shared harness expectations
//!
//! - Use `miniextendr_bench::init()` once per process.
//! - Assert `miniextendr_bench::assert_on_init_thread()` for any R calls.
//! - Reuse fixtures where possible; avoid allocating per-iteration unless that
//!   is what is being measured.
//! - Use `divan` groups with clear parameter sets and labels.
//! - For allocation-heavy benchmarks, separate “allocation included” and
//!   “allocation excluded” cases.
//! - Keep NA density and size fixed within a benchmark to avoid noisy results.
//!
//! ---------------------------------------------------------------------------
//! Standard size matrix
//!
//! Use a consistent set of sizes across benches:
//! - tiny:   1, 4, 16
//! - small:  64, 256
//! - medium: 1_024, 4_096
//! - large:  65_536, 1_000_000
//!
//! Standard NA densities (for logical/real/int/string where applicable):
//! - none (0%)
//! - sparse (~1%)
//! - moderate (~10%)
//! - heavy (~50%)
//!
//! ---------------------------------------------------------------------------
//! Fixtures to provide from the harness
//!
//! - UTF-8 and Latin-1 CHARSXP and STRSXP fixtures (already in lib.rs).
//! - Pre-allocated vectors for each type/size matrix (INTSXP, REALSXP,
//!   LGLSXP, RAWSXP, STRSXP, VECSXP).
//! - Rust-side `Vec<T>` inputs mirroring the same sizes.
//! - Named list fixtures for map conversions.
//! - ExternalPtr fixtures for tagging/protection tests.
//! - ALTREP class fixtures for each data type and iterator variant.
//!
//! ---------------------------------------------------------------------------
//! Module map (documentation only)
//!
//! - `harness`: shared fixture and parameter design
//! - `ffi_calls`: raw R API calls, checked vs unchecked
//! - `sexp_ext`: `SexpExt` helpers vs raw pointer access
//! - `into_r`: conversion costs for IntoR
//! - `from_r`: conversion costs for TryFromSexp
//! - `strings`: encoding and string extraction costs
//! - `coerce`: Coerce / TryCoerce / Coerced
//! - `altrep`: ALTREP class access and callbacks
//! - `altrep_iter`: iterator-backed ALTREP
//! - `externalptr`: ExternalPtr creation/access/protection
//! - `trait_abi`: trait ABI dispatch (mx_erased query + vtable calls)
//! - `preserve`: preserve list insert/release vs PROTECT
//! - `unwind_protect`: with_r_unwind_protect overhead
//! - `worker`: worker thread dispatch overhead
//! - `allocator`: RAllocator behavior
//! - `rayon`: parallel helpers
//! - `connections`: custom connections
//! - `wrappers`: generated R wrapper overhead
//! - `rffi_checked`: checked wrapper overhead
//!
//! Each submodule contains a detailed plan for its bench cases.

pub mod allocator;
pub mod altrep;
pub mod altrep_iter;
pub mod coerce;
pub mod connections;
pub mod externalptr;
pub mod ffi_calls;
pub mod from_r;
pub mod harness;
pub mod into_r;
pub mod preserve;
pub mod rayon;
pub mod rffi_checked;
pub mod sexp_ext;
pub mod strings;
pub mod trait_abi;
pub mod unwind_protect;
pub mod worker;
pub mod wrappers;
