//! Benchmark harness helpers for `miniextendr`.
//!
//! This crate is **not** part of the main workspace build. It's intended to be
//! invoked directly via `cargo bench` from `miniextendr-bench/`.

use std::os::raw::c_char;
use std::sync::OnceLock;

use miniextendr_api::ffi::{self, SEXP};

type SexpAddr = usize;

#[derive(Clone, Copy)]
pub struct Fixtures {
    utf8_charsxp: SexpAddr,
    latin1_charsxp: SexpAddr,
    utf8_strsxp: SexpAddr,
    latin1_strsxp: SexpAddr,
}

impl Fixtures {
    #[inline(always)]
    pub fn utf8_charsxp(self) -> SEXP {
        self.utf8_charsxp as SEXP
    }

    #[inline(always)]
    pub fn latin1_charsxp(self) -> SEXP {
        self.latin1_charsxp as SEXP
    }

    #[inline(always)]
    pub fn utf8_strsxp(self) -> SEXP {
        self.utf8_strsxp as SEXP
    }

    #[inline(always)]
    pub fn latin1_strsxp(self) -> SEXP {
        self.latin1_strsxp as SEXP
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
    let init = INIT_THREAD.get().expect("miniextendr_bench::init not called");
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
        let engine = miniextendr_engine::REngine::new()
            .with_args(&["R", "--quiet", "--vanilla"])
            .interactive(false)
            .signal_handlers(false)
            .init()
            .expect("Failed to initialize embedded R");

        let _ = ENGINE.set(engine);
    });
}

unsafe fn init_fixtures_once() {
    let _ = FIXTURES.get_or_init(|| unsafe {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_mkCharLenCE, Rf_protect, SET_STRING_ELT};

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
        let utf8_strsxp = Rf_protect(Rf_allocVector(ffi::SEXPTYPE::STRSXP, 1));
        SET_STRING_ELT(utf8_strsxp, 0, utf8_charsxp);

        let latin1_strsxp = Rf_protect(Rf_allocVector(ffi::SEXPTYPE::STRSXP, 1));
        SET_STRING_ELT(latin1_strsxp, 0, latin1_charsxp);

        Fixtures {
            utf8_charsxp: utf8_charsxp as SexpAddr,
            latin1_charsxp: latin1_charsxp as SexpAddr,
            utf8_strsxp: utf8_strsxp as SexpAddr,
            latin1_strsxp: latin1_strsxp as SexpAddr,
        }
    });
}
