//! Advanced ALTREP benchmarks: guard modes, materialization, string/complex,
//! and zero-allocation (constant-value) ALTREP.
//!
//! Extends the basic altrep.rs with deeper coverage of ALTREP performance
//! characteristics. Guard mode benchmarks use constant-value ALTREP to isolate
//! guard overhead from data-access patterns.

use miniextendr_api::ffi;
use miniextendr_api::ffi::Rcomplex;
use miniextendr_api::ffi::SexpExt;
use miniextendr_api::{IntoR, miniextendr};
use miniextendr_bench::raw_ffi;

const SIZE_INDICES: &[usize] = &[0, 2, 4];

// region: Guard mode types: constant-value integer ALTREP with different guard modes. (Using constant elt isolates guard overhead from data access cost.)

#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt = "value")]
pub struct GuardDefaultData {
    value: i32,
    len: usize,
}

#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt = "value", r#unsafe)]
pub struct GuardUnsafeData {
    value: i32,
    len: usize,
}

#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt = "value", r_unwind)]
pub struct GuardRUnwindData {
    value: i32,
    len: usize,
}

#[miniextendr(class = "BenchGuardDefault", pkg = "miniextendr.bench")]
struct BenchGuardDefault(GuardDefaultData);

#[miniextendr(class = "BenchGuardUnsafe", pkg = "miniextendr.bench")]
struct BenchGuardUnsafe(GuardUnsafeData);

#[miniextendr(class = "BenchGuardRUnwind", pkg = "miniextendr.bench")]
struct BenchGuardRUnwind(GuardRUnwindData);
// endregion

// region: Zero-allocation real ALTREP (constant value, no backing Vec).

#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AltrepReal)]
#[altrep(len = "len", elt = "value")]
pub struct ConstantRealData {
    value: f64,
    len: usize,
}

#[miniextendr(class = "BenchConstantReal", pkg = "miniextendr.bench")]
struct BenchConstantReal(ConstantRealData);
// endregion

// region: Vec-backed ALTREP types (simple newtype pattern, default guard).

#[miniextendr(class = "BenchIntVec", pkg = "miniextendr.bench")]
struct BenchIntVec(Vec<i32>);

#[miniextendr(class = "BenchRealVec", pkg = "miniextendr.bench")]
struct BenchRealVec(Vec<f64>);

#[miniextendr(class = "BenchString", pkg = "miniextendr.bench")]
struct BenchString(Vec<Option<String>>);

#[miniextendr(class = "BenchComplex", pkg = "miniextendr.bench")]
struct BenchComplex(Vec<Rcomplex>);

fn main() {
    miniextendr_bench::init();
    divan::main();
}
// endregion

// region: Group 1: Guard mode comparison (full scan to amortize creation cost)

mod guard_modes {
    use super::*;

    /// Default guard (catch_unwind) — full scan via INTEGER_ELT.
    #[divan::bench(args = SIZE_INDICES)]
    fn default_guard(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data = GuardDefaultData { value: 42, len };
        let sexp = BenchGuardDefault(data).into_sexp();
        unsafe {
            let mut sum = 0i64;
            for i in 0..len {
                sum += sexp.integer_elt(i as ffi::R_xlen_t) as i64;
            }
            divan::black_box(sum);
        }
    }

    /// Unsafe guard (no protection, fastest path) — full scan via INTEGER_ELT.
    #[divan::bench(args = SIZE_INDICES)]
    fn unsafe_guard(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data = GuardUnsafeData { value: 42, len };
        let sexp = BenchGuardUnsafe(data).into_sexp();
        unsafe {
            let mut sum = 0i64;
            for i in 0..len {
                sum += sexp.integer_elt(i as ffi::R_xlen_t) as i64;
            }
            divan::black_box(sum);
        }
    }

    /// R-unwind guard (with_r_unwind_protect, catches R longjmps) — full scan.
    #[divan::bench(args = SIZE_INDICES)]
    fn r_unwind_guard(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data = GuardRUnwindData { value: 42, len };
        let sexp = BenchGuardRUnwind(data).into_sexp();
        unsafe {
            let mut sum = 0i64;
            for i in 0..len {
                sum += sexp.integer_elt(i as ffi::R_xlen_t) as i64;
            }
            divan::black_box(sum);
        }
    }

    /// Plain INTSXP full scan via INTEGER_ELT (baseline, no ALTREP dispatch).
    #[divan::bench(args = SIZE_INDICES)]
    fn plain_intsxp(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        unsafe {
            let mut sum = 0i64;
            for i in 0..len {
                sum += sexp.integer_elt(i as ffi::R_xlen_t) as i64;
            }
            divan::black_box(sum);
        }
    }
}
// endregion

// region: Group 2: Access patterns (Vec-backed integer ALTREP vs plain INTSXP)

mod materialization {
    use super::*;

    /// DATAPTR_RO on Vec-backed ALTREP (pointer extraction, no copy).
    #[divan::bench(args = SIZE_INDICES)]
    fn altrep_dataptr_ro(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<i32> = (0..len as i32).collect();
        let sexp = BenchIntVec::from(data).into_sexp();
        unsafe {
            divan::black_box(ffi::DATAPTR_RO(sexp));
        }
    }

    /// Full scan via elt (no materialization, per-element dispatch).
    #[divan::bench(args = SIZE_INDICES)]
    fn altrep_full_scan_elt(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<i32> = (0..len as i32).collect();
        let sexp = BenchIntVec::from(data).into_sexp();
        unsafe {
            let mut sum: i64 = 0;
            for i in 0..len {
                sum += sexp.integer_elt(i as ffi::R_xlen_t) as i64;
            }
            divan::black_box(sum);
        }
    }

    /// Full scan via DATAPTR (pointer extraction + pointer scan).
    #[divan::bench(args = SIZE_INDICES)]
    fn altrep_full_scan_dataptr(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<i32> = (0..len as i32).collect();
        let sexp = BenchIntVec::from(data).into_sexp();
        unsafe {
            let ptr = ffi::DATAPTR_RO(sexp).cast::<i32>();
            let mut sum: i64 = 0;
            for i in 0..len {
                sum += *ptr.add(i) as i64;
            }
            divan::black_box(sum);
        }
    }

    /// Plain INTSXP full scan via pointer (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn plain_full_scan(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        unsafe {
            let ptr = raw_ffi::INTEGER(sexp);
            let mut sum: i64 = 0;
            for i in 0..len {
                sum += *ptr.add(i) as i64;
            }
            divan::black_box(sum);
        }
    }

    /// Plain INTSXP DATAPTR_RO (direct data pointer, no dispatch).
    #[divan::bench(args = SIZE_INDICES)]
    fn plain_dataptr_ro(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        unsafe {
            divan::black_box(ffi::DATAPTR_RO(sexp));
        }
    }
}
// endregion

// region: Group 3: String ALTREP

mod string_altrep {
    use super::*;

    /// String ALTREP creation (wraps Vec<Option<String>> in ExternalPtr).
    #[divan::bench(args = SIZE_INDICES)]
    fn create(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Option<String>> = (0..len).map(|i| Some(format!("str_{i}"))).collect();
        divan::black_box(BenchString::from(data).into_sexp());
    }

    /// String ALTREP elt access (returns CHARSXP, converts from Rust String).
    #[divan::bench(args = SIZE_INDICES)]
    fn elt(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Option<String>> = (0..len).map(|i| Some(format!("str_{i}"))).collect();
        let sexp = BenchString::from(data).into_sexp();
        unsafe {
            divan::black_box(raw_ffi::STRING_ELT(sexp, 0));
        }
    }

    /// String ALTREP elt access with 10% NA density.
    #[divan::bench(args = SIZE_INDICES)]
    fn elt_with_na(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Option<String>> = (0..len)
            .map(|i| {
                if i % 10 == 0 {
                    None
                } else {
                    Some(format!("str_{i}"))
                }
            })
            .collect();
        let sexp = BenchString::from(data).into_sexp();
        unsafe {
            divan::black_box(raw_ffi::STRING_ELT(sexp, (len - 1) as isize));
        }
    }

    /// Forces materialization into STRSXP cached in data2 slot.
    #[divan::bench(args = SIZE_INDICES)]
    fn force_materialize(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Option<String>> = (0..len).map(|i| Some(format!("str_{i}"))).collect();
        let sexp = BenchString::from(data).into_sexp();
        unsafe {
            divan::black_box(ffi::DATAPTR_RO(sexp));
        }
    }

    /// Plain STRSXP elt access (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn plain_strsxp_elt(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let strings: Vec<String> = (0..len).map(|i| format!("str_{i}")).collect();
        let sexp = strings.into_sexp();
        unsafe {
            divan::black_box(raw_ffi::STRING_ELT(sexp, 0));
        }
    }
}
// endregion

// region: Group 4: Complex ALTREP

mod complex_altrep {
    use super::*;

    /// Complex ALTREP elt access (unit circle values).
    #[divan::bench(args = SIZE_INDICES)]
    fn elt(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Rcomplex> = (0..len)
            .map(|i| {
                let theta = 2.0 * std::f64::consts::PI * (i as f64) / (len as f64);
                Rcomplex {
                    r: theta.cos(),
                    i: theta.sin(),
                }
            })
            .collect();
        let sexp = BenchComplex::from(data).into_sexp();
        unsafe {
            divan::black_box(sexp.complex_elt(0));
        }
    }

    /// Complex ALTREP DATAPTR_RO (pointer extraction, no copy).
    #[divan::bench(args = SIZE_INDICES)]
    fn dataptr(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Rcomplex> = (0..len)
            .map(|i| Rcomplex {
                r: i as f64,
                i: 0.0,
            })
            .collect();
        let sexp = BenchComplex::from(data).into_sexp();
        unsafe {
            divan::black_box(ffi::DATAPTR_RO(sexp));
        }
    }

    /// Complex ALTREP full scan via COMPLEX_ELT (per-element dispatch).
    #[divan::bench(args = SIZE_INDICES)]
    fn full_scan_elt(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Rcomplex> = (0..len)
            .map(|i| Rcomplex {
                r: i as f64,
                i: 0.0,
            })
            .collect();
        let sexp = BenchComplex::from(data).into_sexp();
        unsafe {
            let mut sum_r = 0.0f64;
            for i in 0..len {
                sum_r += sexp.complex_elt(i as ffi::R_xlen_t).r;
            }
            divan::black_box(sum_r);
        }
    }
}
// endregion

// region: Group 5: Zero-allocation ALTREP (constant value, no backing Vec)

mod zero_alloc {
    use super::*;

    /// Constant-value ALTREP creation (no Vec, just 2 fields).
    #[divan::bench(args = SIZE_INDICES)]
    fn create_constant(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data = ConstantRealData { value: 1.0, len };
        divan::black_box(BenchConstantReal(data).into_sexp());
    }

    /// Vec-backed real ALTREP creation (allocates Vec<f64>).
    #[divan::bench(args = SIZE_INDICES)]
    fn create_vec_backed(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<f64> = (0..len).map(|i| i as f64).collect();
        divan::black_box(BenchRealVec::from(data).into_sexp());
    }

    /// Plain REALSXP creation (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn create_plain(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<f64> = (0..len).map(|i| i as f64).collect();
        divan::black_box(data.into_sexp());
    }

    /// Constant ALTREP elt access.
    #[divan::bench(args = SIZE_INDICES)]
    fn constant_elt(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data = ConstantRealData { value: 1.0, len };
        let sexp = BenchConstantReal(data).into_sexp();
        unsafe {
            divan::black_box(sexp.real_elt(0));
        }
    }

    /// Vec-backed ALTREP elt access.
    #[divan::bench(args = SIZE_INDICES)]
    fn vec_backed_elt(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<f64> = (0..len).map(|i| i as f64).collect();
        let sexp = BenchRealVec::from(data).into_sexp();
        unsafe {
            divan::black_box(sexp.real_elt(0));
        }
    }

    /// Constant ALTREP full scan (no backing allocation, returns stored field).
    #[divan::bench(args = SIZE_INDICES)]
    fn constant_full_scan(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data = ConstantRealData { value: 1.0, len };
        let sexp = BenchConstantReal(data).into_sexp();
        unsafe {
            let mut sum = 0.0f64;
            for i in 0..len {
                sum += sexp.real_elt(i as ffi::R_xlen_t);
            }
            divan::black_box(sum);
        }
    }

    /// Vec-backed ALTREP full scan.
    #[divan::bench(args = SIZE_INDICES)]
    fn vec_backed_full_scan(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<f64> = (0..len).map(|i| i as f64).collect();
        let sexp = BenchRealVec::from(data).into_sexp();
        unsafe {
            let mut sum = 0.0f64;
            for i in 0..len {
                sum += sexp.real_elt(i as ffi::R_xlen_t);
            }
            divan::black_box(sum);
        }
    }

    /// Plain REALSXP full scan via pointer (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn plain_full_scan(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
        unsafe {
            let ptr = raw_ffi::REAL(sexp);
            let mut sum = 0.0f64;
            for i in 0..len {
                sum += *ptr.add(i);
            }
            divan::black_box(sum);
        }
    }
}
// endregion

// region: Group 6: ALTREP creation cost comparison

mod creation {
    use super::*;

    /// Vec-backed integer ALTREP creation (wraps Vec<i32> in ExternalPtr).
    #[divan::bench(args = SIZE_INDICES)]
    fn altrep_int(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<i32> = (0..len as i32).collect();
        divan::black_box(BenchIntVec::from(data).into_sexp());
    }

    /// Plain INTSXP creation via into_sexp (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn plain_int(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<i32> = (0..len as i32).collect();
        divan::black_box(data.into_sexp());
    }

    /// String ALTREP creation (wraps Vec<Option<String>> in ExternalPtr).
    #[divan::bench(args = SIZE_INDICES)]
    fn altrep_string(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Option<String>> = (0..len).map(|i| Some(format!("s{i}"))).collect();
        divan::black_box(BenchString::from(data).into_sexp());
    }

    /// Plain STRSXP creation via into_sexp (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn plain_string(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<String> = (0..len).map(|i| format!("s{i}")).collect();
        divan::black_box(data.into_sexp());
    }

    /// Complex ALTREP creation (wraps Vec<Rcomplex> in ExternalPtr).
    #[divan::bench(args = SIZE_INDICES)]
    fn altrep_complex(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data: Vec<Rcomplex> = (0..len)
            .map(|i| Rcomplex {
                r: i as f64,
                i: 0.0,
            })
            .collect();
        divan::black_box(BenchComplex::from(data).into_sexp());
    }

    /// Zero-allocation constant ALTREP — cost should be O(1) regardless of len.
    #[divan::bench(args = SIZE_INDICES)]
    fn constant_real(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let data = ConstantRealData { value: 1.0, len };
        divan::black_box(BenchConstantReal(data).into_sexp());
    }
}
// endregion
