# Macro Coverage Completion Plan (Self-Contained Tasks)

## Objective
Make `macro_coverage` the single exhaustive fixture for macro expansion, covering:
1. all macro options
2. all macro item kinds
3. all valid option combinations

with the entire fixture feature-gated.

---

## Task 1: Add Whole-Module Feature Gate
**Context**: You want to enable/disable all macro coverage in one switch.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/Cargo.toml`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/lib.rs`

**Changes**:
```toml
# Cargo.toml
[features]
macro-coverage = []
```

```rust
// lib.rs
#[cfg(feature = "macro-coverage")]
#[doc(hidden)]
pub mod macro_coverage;
```

**Done when**:
- `cargo check -p miniextendr-api --features macro-coverage` compiles.
- `cargo check -p miniextendr-api --no-default-features` compiles (module excluded).

---

## Task 2: Split Fixture Into Matrix Submodules
**Context**: Combinational coverage becomes unmaintainable in one file.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/mod.rs`
- new files under `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/`

**Changes**:
```rust
// mod.rs
mod fn_matrix;
mod impl_matrix;
mod trait_abi_matrix;
mod derive_matrix;
mod helper_macro_matrix;
mod module_matrix;
```

**Done when**:
- Each matrix file compiles independently.
- `mod.rs` has no macro fixtures itself, only orchestration.

---

## Task 3: Function Option Atomic Coverage
**Context**: Ensure every `MiniextendrFnAttrs` option appears at least once.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/fn_matrix.rs`

**Add at least one function per option**:
```rust
use crate::{miniextendr, typed_list};

#[miniextendr(worker)]
pub(crate) fn cov_fn_worker(x: i32) -> i32 { x }

#[miniextendr(rng)]
pub(crate) fn cov_fn_rng(x: i32) -> i32 { x }

#[miniextendr(unwrap_in_r)]
pub(crate) fn cov_fn_unwrap_in_r(x: i32) -> Result<i32, &'static str> { Ok(x) }

#[miniextendr(return = "list")]
pub(crate) fn cov_fn_return_list(x: i32) -> i32 { x }

#[miniextendr(return = "externalptr")]
pub(crate) fn cov_fn_return_externalptr(x: i32) -> i32 { x }

#[miniextendr(return = "vector")]
pub(crate) fn cov_fn_return_vector(x: i32) -> i32 { x }

#[miniextendr(dots = typed_list!(x => integer(), y? => numeric()))]
pub(crate) fn cov_fn_typed_dots(...) -> i32 {
    let _: i32 = dots_typed.get("x").expect("x");
    1
}

#[miniextendr(s3(generic = "vec_proxy", class = "cov_type"))]
pub(crate) fn cov_fn_s3(x: i32) -> i32 { x }

#[miniextendr(lifecycle = "deprecated")]
pub(crate) fn cov_fn_lifecycle_simple(x: i32) -> i32 { x }

#[miniextendr(lifecycle(stage = "deprecated", when = "0.9.0", with = "cov_fn_new()"))]
pub(crate) fn cov_fn_lifecycle_full(x: i32) -> i32 { x }

#[miniextendr]
pub(crate) fn cov_fn_param_default(#[miniextendr(default = "1L")] x: i32) -> i32 { x }
```

**Done when**:
- Each option has one explicit fixture symbol in `fn_matrix.rs`.

---

## Task 4: Function Option Combination Matrix
**Context**: Atomic coverage is not enough; combinations can break expansion.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/fn_matrix.rs`

**Pattern**:
```rust
macro_rules! cov_case {
    ($name:ident, ($($opts:tt)*)) => {
        #[miniextendr($($opts)*)]
        pub(crate) fn $name(x: i32) -> i32 { x }
    };
}

cov_case!(cov_combo_worker_invisible, (worker, invisible));
cov_case!(cov_combo_worker_visible, (worker, visible));
cov_case!(cov_combo_worker_coerce, (worker, coerce));
cov_case!(cov_combo_worker_rng, (worker, rng));
cov_case!(cov_combo_worker_unwrap, (worker, unwrap_in_r));
cov_case!(cov_combo_mainthread_interrupt, (unsafe(main_thread), check_interrupt));
cov_case!(cov_combo_mainthread_visible_interrupt, (unsafe(main_thread), visible, check_interrupt));
```

**Done when**:
- `FN_COMBOS` table exists in code and each row maps 1:1 to a generated fixture.
- No documented valid combo is missing from table.

---

## Task 5: Extern Symbol Variants
**Context**: extern handling has special parser/codegen behavior.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/fn_matrix.rs`

**Add fixtures**:
```rust
use crate::ffi::SEXP;

#[miniextendr]
#[unsafe(no_mangle)]
pub(crate) extern "C-unwind" fn C_cov_no_mangle(x: SEXP) -> SEXP { x }

#[miniextendr]
#[export_name = "C_cov_export_named"]
pub(crate) extern "C-unwind" fn cov_export_named(x: SEXP) -> SEXP { x }
```

**Done when**:
- Expansion includes both direct C symbols.

---

## Task 6: `#[miniextendr]` Impl/Class-System Matrix
**Context**: function-only coverage misses most macro complexity.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/impl_matrix.rs`

**Add minimum fixtures for env/r6/s3/s4/s7**:
```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct CovEnv { v: i32 }

#[miniextendr(env, label = "core")]
impl CovEnv {
    pub fn new(v: i32) -> Self { Self { v } }
    pub fn get(&self) -> i32 { self.v }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovR6 { v: i32 }

#[miniextendr(r6, label = "core")]
impl CovR6 {
    pub fn new(v: i32) -> Self { Self { v } }
    #[miniextendr(r6(active))]
    pub fn value(&self) -> i32 { self.v }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovS7 { v: i32 }

#[miniextendr(s7)]
impl CovS7 {
    pub fn new(v: i32) -> Self { Self { v } }
    #[miniextendr(s7(getter))]
    pub fn value(&self) -> i32 { self.v }
    #[miniextendr(s7(no_dots))]
    pub fn strict(&self) -> i32 { self.v }
}
```

**Done when**:
- All class systems supported by parser are represented by at least one type.

---

## Task 7: Trait ABI Matrix
**Context**: trait definition and trait impl paths have separate expansion logic.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/trait_abi_matrix.rs`

**Add fixtures**:
```rust
#[miniextendr]
pub trait CovTrait {
    fn bump(&mut self);
    fn value(&self) -> i32;
}

#[miniextendr]
impl CovTrait for super::impl_matrix::CovEnv {
    fn bump(&mut self) { self.v += 1; }
    fn value(&self) -> i32 { self.v }
}
```

**Done when**:
- Expansion contains generated trait tag/vtable path and impl vtable path.

---

## Task 8: Derive Macro Matrix
**Context**: derive entrypoints in `miniextendr-macros/src/lib.rs` must be exercised.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/derive_matrix.rs`

**Add one minimal type per derive**:
```rust
#[derive(miniextendr_api::RNativeType, Clone, Copy)]
pub struct CovNative(i32);

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovPtr { pub x: i32 }

#[derive(miniextendr_api::IntoList, miniextendr_api::TryFromList)]
pub struct CovList { pub x: i32 }

#[derive(miniextendr_api::PreferList)]
pub struct CovPreferList { pub x: i32 }

#[derive(miniextendr_api::DataFrameRow)]
pub struct CovRow { pub x: i32 }

#[derive(miniextendr_api::RFactor, Copy, Clone)]
pub enum CovFactor { A, B }

#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs)]
#[vctrs(class = "cov_vctrs", base = "double")]
pub struct CovVctrs { pub data: Vec<f64> }
```

**Done when**:
- Every derive entrypoint has one fixture.

---

## Task 9: Helper Macros + `r_ffi_checked`
**Context**: proc macros `typed_list!`, `list!`, and `r_ffi_checked` are separate entrypoints.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/helper_macro_matrix.rs`

**Add fixtures**:
```rust
use crate::{list, miniextendr, r_ffi_checked, typed_list, IntoR};
use crate::ffi::{SEXP, SEXPTYPE, R_xlen_t};

#[r_ffi_checked]
unsafe extern "C-unwind" {
    pub fn Rf_allocVector(t: SEXPTYPE, n: R_xlen_t) -> SEXP;
    pub fn INTEGER(x: SEXP) -> *mut i32;
}

#[miniextendr]
pub(crate) fn cov_list_macro() -> SEXP {
    list!(a = 1L, b = "x").into_sexp()
}

#[miniextendr]
pub(crate) fn cov_typed_list_macro(...) -> i32 {
    let _ = _dots.typed(typed_list!(a => integer(), b? => character()));
    0
}
```

**Done when**:
- Expansion contains generated wrappers for all three entrypoints.

---

## Task 10: `miniextendr_module!` Item-Variant Matrix
**Context**: parser variants in `miniextendr-macros-core` must all be exercised.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/module_matrix.rs`

**Add module fixture**:
```rust
miniextendr_module! {
    mod macro_coverage;
    use nested_cov;

    fn cov_fn_worker;
    extern "C-unwind" fn C_cov_no_mangle;
    extern "C-unwind" fn C_cov_export_named;

    struct CovAltInt;

    impl CovEnv as "core";
    impl CovR6 as "core";
    impl CovS7;

    impl CovTrait for CovEnv;

    #[cfg(feature = "vctrs")]
    vctrs CovVctrs;
}
```

**Done when**:
- Every `MiniextendrModuleItem` variant appears at least once.
- At least one `#[cfg(...)]` decorated module item is present.

---

## Task 11: ALTREP Wrapper Path
**Context**: `#[miniextendr(class = "...")]` struct path must be present in fixture.

**Files**:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/macro_coverage/impl_matrix.rs`

**Add fixtures**:
```rust
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AltrepInteger)]
pub struct CovAltIntData { len: usize }

#[miniextendr(class = "CovAltInt", pkg = "macro_coverage")]
pub struct CovAltInt(pub CovAltIntData);
```

**Done when**:
- `struct CovAltInt;` registration in module matrix compiles.

---

## Task 12: Validation + Expansion Checks
**Context**: completion is expansion-driven, not runtime-driven.

**Commands**:
```sh
cargo check -p miniextendr-api --features macro-coverage
cargo test -p miniextendr-macros
cargo test -p miniextendr-macros-core
cargo expand --lib -p miniextendr-api --features macro-coverage
cargo expand --lib -p miniextendr-api --features "macro-coverage,vctrs,serde,rayon"
```

**Done when**:
- all commands succeed
- each matrix module’s fixture symbols show up in expansion output
- coverage table in `mod.rs` maps macro path -> fixture symbol(s)

---

## Acceptance Criteria
- Every proc-macro entrypoint in `miniextendr-macros/src/lib.rs` has at least one fixture.
- Every parser item variant in `miniextendr-macros-core/src/miniextendr_module.rs` has at least one fixture.
- Every function-level option in `MiniextendrFnAttrs` has both:
  1. atomic coverage fixture
  2. at least one combination-row fixture
- Entire coverage can be turned off with one feature flag.

---

## Known Risk
S7 fallback generation currently has a semantic mismatch (`class_any` dispatch still assumes `x@.ptr`, and fallback does not apply in one generic-override path). Keep fallback fixtures for expansion coverage, but treat runtime behavior as blocked until generator fix.
