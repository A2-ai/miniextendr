//! Benchmark harness helpers for `miniextendr`.
//!
//! This crate is **not** part of the main workspace build. It's intended to be
//! invoked directly via `cargo bench` from `miniextendr-bench/`.
//!
//! The benchmark plan is documented in `bench_plan` (module-level docs only).

use std::os::raw::c_char;
use std::sync::OnceLock;

use miniextendr_api::ffi::{self, SEXP};

pub mod bench_plan;

// =============================================================================
// Size matrix for parameterized benchmarks
// =============================================================================

/// Standard size constants for benchmark parameterization.
pub const SIZES: &[usize] = &[1, 16, 256, 4096, 65536];

/// Size labels for divan output.
pub const SIZE_LABELS: &[&str] = &["1", "16", "256", "4K", "64K"];

/// Sizes for named list fixtures (used for map/list benchmarks).
pub const NAMED_LIST_SIZES: &[usize] = &[16, 256, 4096];

/// Labels for named list sizes.
pub const NAMED_LIST_SIZE_LABELS: &[&str] = &["16", "256", "4K"];

/// Matrix dimensions (nrow, ncol) for matrix/rarray benchmarks.
pub const MATRIX_DIMS: &[(usize, usize)] = &[(64, 64), (256, 256)];

/// Labels for matrix dimensions.
pub const MATRIX_DIM_LABELS: &[&str] = &["64x64", "256x256"];

// =============================================================================
// Fixtures
// =============================================================================

#[derive(Clone, Copy)]
pub struct Fixtures {
    utf8_charsxp: SEXP,
    latin1_charsxp: SEXP,
    utf8_strsxp: SEXP,
    latin1_strsxp: SEXP,
    // Pre-allocated vectors for each size
    int_vecs: [SEXP; 5],
    real_vecs: [SEXP; 5],
    lgl_vecs: [SEXP; 5],
    raw_vecs: [SEXP; 5],
    // STRSXP with single string element of various sizes
    str_vecs: [SEXP; 5],
    // Named lists (VECSXP + names) used for list/map benchmarks
    named_list_i32: [SEXP; 3],
    // REAL matrices used for rarray benchmarks
    real_mats: [SEXP; 2],
}

impl Fixtures {
    #[inline(always)]
    pub fn utf8_charsxp(self) -> SEXP {
        self.utf8_charsxp
    }

    #[inline(always)]
    pub fn latin1_charsxp(self) -> SEXP {
        self.latin1_charsxp
    }

    #[inline(always)]
    pub fn utf8_strsxp(self) -> SEXP {
        self.utf8_strsxp
    }

    #[inline(always)]
    pub fn latin1_strsxp(self) -> SEXP {
        self.latin1_strsxp
    }

    /// Get pre-allocated INTSXP of given size index (0-4 maps to SIZES).
    #[inline(always)]
    pub fn int_vec(self, size_idx: usize) -> SEXP {
        self.int_vecs[size_idx]
    }

    /// Get pre-allocated REALSXP of given size index.
    #[inline(always)]
    pub fn real_vec(self, size_idx: usize) -> SEXP {
        self.real_vecs[size_idx]
    }

    /// Get pre-allocated LGLSXP of given size index.
    #[inline(always)]
    pub fn lgl_vec(self, size_idx: usize) -> SEXP {
        self.lgl_vecs[size_idx]
    }

    /// Get pre-allocated RAWSXP of given size index.
    #[inline(always)]
    pub fn raw_vec(self, size_idx: usize) -> SEXP {
        self.raw_vecs[size_idx]
    }

    /// Get pre-allocated STRSXP(1) with string of given size index.
    #[inline(always)]
    pub fn str_vec(self, size_idx: usize) -> SEXP {
        self.str_vecs[size_idx]
    }

    /// Get pre-allocated named list (VECSXP) of given size index (0-2 maps to NAMED_LIST_SIZES).
    #[inline(always)]
    pub fn named_list_i32(self, size_idx: usize) -> SEXP {
        self.named_list_i32[size_idx]
    }

    /// Get pre-allocated REAL matrix of given size index (0-1 maps to MATRIX_DIMS).
    #[inline(always)]
    pub fn real_matrix(self, size_idx: usize) -> SEXP {
        self.real_mats[size_idx]
    }
}

static INIT_THREAD: OnceLock<std::thread::ThreadId> = OnceLock::new();
static ENGINE: OnceLock<miniextendr_engine::REngine> = OnceLock::new();
static FIXTURES: OnceLock<Fixtures> = OnceLock::new();

/// Initialize the embedded R runtime and benchmark fixtures.
///
/// This must be called once, and all subsequent benchmark code should run on
/// the same thread.
pub fn init() {
    unsafe {
        init_r_once();
        init_fixtures_once();
    }
}

#[inline(always)]
pub fn assert_on_init_thread() {
    let init = INIT_THREAD
        .get()
        .expect("miniextendr_bench::init not called");
    assert_eq!(
        *init,
        std::thread::current().id(),
        "R must be called from the init thread"
    );
}

#[inline(always)]
pub fn fixtures() -> Fixtures {
    assert_on_init_thread();
    *FIXTURES.get().expect("fixtures not initialized")
}

unsafe fn init_r_once() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| unsafe {
        let _ = INIT_THREAD.set(std::thread::current().id());

        // Initialize embedded R via `miniextendr-engine` (kept alive for the
        // lifetime of the benchmark process).
        let engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .interactive(false)
            .signal_handlers(false)
            .init()
            .expect("Failed to initialize embedded R");

        let _ = ENGINE.set(engine);

        // Register this thread as the R main thread for miniextendr-api's
        // thread safety checks. This must be called after R is initialized.
        miniextendr_api::miniextendr_worker_init();
    });
}

unsafe fn init_fixtures_once() {
    let _ = FIXTURES.get_or_init(|| unsafe {
        use miniextendr_api::ffi::{
            INTEGER,
            LOGICAL,
            RAW,
            REAL,
            R_NamesSymbol,
            Rf_allocMatrix,
            Rf_allocVector,
            Rf_mkCharLenCE,
            Rf_protect,
            Rf_setAttrib,
            Rf_ScalarInteger,
            SET_STRING_ELT,
            SET_VECTOR_ELT,
            SEXPTYPE,
        };

        // UTF-8 string.
        let utf8_bytes: &[u8] = b"hello";
        let utf8_charsxp = Rf_mkCharLenCE(
            utf8_bytes.as_ptr().cast::<c_char>(),
            utf8_bytes.len() as i32,
            ffi::CE_UTF8,
        );

        // Latin-1 "café" (0xE9).
        let latin1_bytes: &[u8] = &[0x63, 0x61, 0x66, 0xE9];
        let latin1_charsxp = Rf_mkCharLenCE(
            latin1_bytes.as_ptr().cast::<c_char>(),
            latin1_bytes.len() as i32,
            ffi::cetype_t::CE_LATIN1,
        );

        // STRSXP(1) wrappers to mirror the `TryFromSexp` path.
        let utf8_strsxp = Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, 1));
        SET_STRING_ELT(utf8_strsxp, 0, utf8_charsxp);

        let latin1_strsxp = Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, 1));
        SET_STRING_ELT(latin1_strsxp, 0, latin1_charsxp);

        // Pre-allocate vectors for each size in SIZES
        let mut int_vecs = [SEXP::null(); 5];
        let mut real_vecs = [SEXP::null(); 5];
        let mut lgl_vecs = [SEXP::null(); 5];
        let mut raw_vecs = [SEXP::null(); 5];

        for (i, &size) in SIZES.iter().enumerate() {
            // Integer vector filled with 0..size
            let int_vec = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, size as ffi::R_xlen_t));
            let int_ptr = INTEGER(int_vec);
            for j in 0..size {
                *int_ptr.add(j) = j as i32;
            }
            int_vecs[i] = int_vec;

            // Real vector filled with 0.0..size as f64
            let real_vec = Rf_protect(Rf_allocVector(SEXPTYPE::REALSXP, size as ffi::R_xlen_t));
            let real_ptr = REAL(real_vec);
            for j in 0..size {
                *real_ptr.add(j) = j as f64;
            }
            real_vecs[i] = real_vec;

            // Logical vector with alternating TRUE/FALSE
            let lgl_vec = Rf_protect(Rf_allocVector(SEXPTYPE::LGLSXP, size as ffi::R_xlen_t));
            let lgl_ptr = LOGICAL(lgl_vec);
            for j in 0..size {
                *lgl_ptr.add(j) = (j % 2) as i32;
            }
            lgl_vecs[i] = lgl_vec;

            // Raw vector filled with 0..255 cycling
            let raw_vec = Rf_protect(Rf_allocVector(SEXPTYPE::RAWSXP, size as ffi::R_xlen_t));
            let raw_ptr = RAW(raw_vec);
            for j in 0..size {
                *raw_ptr.add(j) = (j % 256) as u8;
            }
            raw_vecs[i] = raw_vec;
        }

        // String vectors: STRSXP(1) with string of various lengths
        let mut str_vecs = [SEXP::null(); 5];
        for (i, &size) in SIZES.iter().enumerate() {
            let s: String = "x".repeat(size);
            let strsxp = Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, 1));
            let charsxp = Rf_mkCharLenCE(s.as_ptr().cast::<c_char>(), size as i32, ffi::CE_UTF8);
            SET_STRING_ELT(strsxp, 0, charsxp);
            str_vecs[i] = strsxp;
        }

        // Named lists for list/map conversion benchmarks
        let mut named_list_i32 = [SEXP::null(); 3];
        for (idx, &len) in NAMED_LIST_SIZES.iter().enumerate() {
            let list = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, len as ffi::R_xlen_t));
            let names = Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, len as ffi::R_xlen_t));

            for i in 0..len {
                let val = Rf_ScalarInteger(i as i32);
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, val);

                let key = format!("k{i}");
                let chars = Rf_mkCharLenCE(
                    key.as_ptr().cast::<c_char>(),
                    key.len() as i32,
                    ffi::CE_UTF8,
                );
                SET_STRING_ELT(names, i as ffi::R_xlen_t, chars);
            }

            Rf_setAttrib(list, R_NamesSymbol, names);
            named_list_i32[idx] = list;
        }

        // REAL matrices for rarray benchmarks
        let mut real_mats = [SEXP::null(); 2];
        for (idx, &(nrow, ncol)) in MATRIX_DIMS.iter().enumerate() {
            let mat = Rf_protect(Rf_allocMatrix(
                SEXPTYPE::REALSXP,
                nrow as i32,
                ncol as i32,
            ));

            let ptr = REAL(mat);
            let total = nrow * ncol;
            for i in 0..total {
                *ptr.add(i) = i as f64;
            }

            real_mats[idx] = mat;
        }

        Fixtures {
            utf8_charsxp,
            latin1_charsxp,
            utf8_strsxp,
            latin1_strsxp,
            int_vecs,
            real_vecs,
            lgl_vecs,
            raw_vecs,
            str_vecs,
            named_list_i32,
            real_mats,
        }
    });
}
