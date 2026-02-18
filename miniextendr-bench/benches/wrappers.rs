//! R eval wrapper benchmarks.
//!
//! Measures the overhead of calling R wrapper functions vs direct `.Call`,
//! argument coercion cost, and class-system method dispatch overhead.
//!
//! Benchmark groups:
//! - `wrapper_call_overhead`: R wrapper function -> base R function
//! - `direct_call_overhead`: direct Rf_eval of equivalent expression
//! - `argument_coercion`: wrapper path with type coercion
//! - `class_methods`: Env/R6/S3/S4/S7-style method invocation overhead

use miniextendr_api::ffi;
use std::os::raw::c_char;

// R_ext/Parse.h types not exposed in miniextendr-api FFI.
#[allow(non_camel_case_types, dead_code)]
#[repr(i32)]
#[derive(Debug, PartialEq)]
enum ParseStatus {
    PARSE_NULL,
    PARSE_OK,
    PARSE_INCOMPLETE,
    PARSE_ERROR,
    PARSE_EOF,
}

unsafe extern "C" {
    fn R_ParseVector(
        text: ffi::SEXP,
        n: i32,
        status: *mut ParseStatus,
        srcfile: ffi::SEXP,
    ) -> ffi::SEXP;
}

/// RAII guard that calls `Rf_unprotect(n)` on drop.
struct ProtectGuard {
    count: i32,
}

impl ProtectGuard {
    fn new(count: i32) -> Self {
        Self { count }
    }
}

impl Drop for ProtectGuard {
    fn drop(&mut self) {
        if self.count > 0 {
            unsafe {
                ffi::Rf_unprotect(self.count);
            }
        }
    }
}

/// Pre-parsed R expression with protection.
struct ParsedExpr {
    expr: ffi::SEXP,
    _guard: ProtectGuard,
}

/// Parse an R string into a protected expression ready for `Rf_eval`.
unsafe fn parse_and_protect(code: &str) -> ParsedExpr {
    unsafe {
        let code_sexp = ffi::Rf_protect(ffi::Rf_allocVector(ffi::SEXPTYPE::STRSXP, 1));
        let charsxp = ffi::Rf_mkCharLenCE(
            code.as_ptr().cast::<c_char>(),
            code.len() as i32,
            ffi::CE_UTF8,
        );
        ffi::SET_STRING_ELT(code_sexp, 0, charsxp);

        let mut status = ParseStatus::PARSE_NULL;
        let parsed = ffi::Rf_protect(R_ParseVector(code_sexp, 1, &mut status, ffi::R_NilValue));
        assert_eq!(
            status,
            ParseStatus::PARSE_OK,
            "Failed to parse R code: {code}"
        );

        let expr = ffi::Rf_protect(ffi::VECTOR_ELT(parsed, 0));

        ParsedExpr {
            expr,
            _guard: ProtectGuard::new(3),
        }
    }
}

/// Evaluate an R code string and return the result (unprotected).
unsafe fn r_eval_string(code: &str) -> ffi::SEXP {
    unsafe {
        let parsed = parse_and_protect(code);
        ffi::Rf_eval(parsed.expr, ffi::R_GlobalEnv)
    }
}

fn main() {
    miniextendr_bench::init();

    // Define wrapper functions and fixtures in R's global environment before
    // running benchmarks. These simulate the generated wrapper patterns.
    unsafe {
        r_eval_string(".__bench_noop__ <- function(x) x");
        r_eval_string(".__bench_wrapper_noop__ <- function(x) .__bench_noop__(x)");
        r_eval_string(
            ".__bench_wrapper_coerce_int__ <- function(x) .__bench_noop__(as.integer(x))",
        );
        r_eval_string(".__bench_wrapper_coerce_dbl__ <- function(x) .__bench_noop__(as.double(x))");
        r_eval_string(
            ".__bench_wrapper_coerce_chr__ <- function(x) .__bench_noop__(as.character(x))",
        );

        // Env-style class (like miniextendr's default Env class system).
        // Methods are stored directly in the environment object; R's native
        // `$` for environments handles dispatch.
        r_eval_string(".__bench_env_obj__ <- new.env(parent = emptyenv())");
        r_eval_string(".__bench_env_obj__$value <- function() 42L");
        r_eval_string(".__bench_env_obj__$noop <- function() invisible(NULL)");

        // R6-style class
        r_eval_string(concat!(
            "if (requireNamespace('R6', quietly = TRUE)) {\n",
            "  .__bench_R6Class__ <- R6::R6Class('BenchR6',\n",
            "    public = list(\n",
            "      value = function() 42L,\n",
            "      noop = function() invisible(NULL)\n",
            "    )\n",
            "  )\n",
            "  .__bench_r6_obj__ <- .__bench_R6Class__$new()\n",
            "}\n",
        ));

        // S3-style dispatch
        r_eval_string(".__bench_s3_value__ <- function(x, ...) UseMethod('.__bench_s3_value__')");
        r_eval_string(".__bench_s3_value__.BenchS3 <- function(x, ...) 42L");
        r_eval_string(".__bench_s3_obj__ <- structure(list(), class = 'BenchS3')");

        // S4-style dispatch
        r_eval_string("methods::setClass('BenchS4', slots = c(data = 'integer'))");
        r_eval_string(concat!(
            "methods::setGeneric('.__bench_s4_value__',\n",
            "  function(x, ...) standardGeneric('.__bench_s4_value__'))",
        ));
        r_eval_string(concat!(
            "methods::setMethod('.__bench_s4_value__', 'BenchS4',\n",
            "  function(x, ...) 42L)",
        ));
        r_eval_string(".__bench_s4_obj__ <- methods::new('BenchS4', data = 42L)");

        // S7-style dispatch (if available)
        r_eval_string(concat!(
            "if (requireNamespace('S7', quietly = TRUE)) {\n",
            "  .__bench_S7Class__ <- S7::new_class('BenchS7',\n",
            "    properties = list(data = S7::class_integer))\n",
            "  .__bench_s7_value__ <- S7::new_generic('.__bench_s7_value__', 'x',\n",
            "    function(x, ...) S7::S7_dispatch())\n",
            "  S7::method(.__bench_s7_value__, .__bench_S7Class__) <- function(x, ...) 42L\n",
            "  .__bench_s7_obj__ <- .__bench_S7Class__(data = 42L)\n",
            "}\n",
        ));
    }

    divan::main();
}

// =============================================================================
// wrapper_call_overhead: R wrapper -> base R function
// =============================================================================

/// Baseline: evaluate a pre-built call to sum() directly via Rf_eval.
#[divan::bench]
fn eval_sum(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c"sum".as_ptr());
            let call = ffi::Rf_lang2(sym, miniextendr_bench::fixtures().real_vec(2));
            ffi::Rf_protect(call);
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Call a no-op R function directly (baseline for wrapper overhead measurement).
#[divan::bench]
fn direct_call_noop(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_noop__".as_ptr());
            let arg = ffi::Rf_ScalarInteger(1);
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Call a wrapper R function that delegates to the no-op (measures wrapper overhead).
#[divan::bench]
fn wrapper_call_noop(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_wrapper_noop__".as_ptr());
            let arg = ffi::Rf_ScalarInteger(1);
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Direct call to identity with a real vector argument (no coercion).
#[divan::bench]
fn direct_call_realvec(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_noop__".as_ptr());
            let arg = miniextendr_bench::fixtures().real_vec(2);
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Wrapper call with a real vector argument (no coercion).
#[divan::bench]
fn wrapper_call_realvec(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_wrapper_noop__".as_ptr());
            let arg = miniextendr_bench::fixtures().real_vec(2);
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

// =============================================================================
// argument_coercion: wrapper path with type conversion
// =============================================================================

/// Wrapper with as.integer() coercion on a real scalar.
#[divan::bench]
fn coerce_int_scalar(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_wrapper_coerce_int__".as_ptr());
            let arg = ffi::Rf_ScalarReal(42.0);
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Wrapper with as.double() coercion on an integer scalar.
#[divan::bench]
fn coerce_dbl_scalar(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_wrapper_coerce_dbl__".as_ptr());
            let arg = ffi::Rf_ScalarInteger(42);
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Wrapper with as.character() coercion on an integer scalar.
#[divan::bench]
fn coerce_chr_scalar(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_wrapper_coerce_chr__".as_ptr());
            let arg = ffi::Rf_ScalarInteger(42);
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Wrapper with as.integer() coercion on a 256-element real vector.
#[divan::bench]
fn coerce_int_vec256(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c".__bench_wrapper_coerce_int__".as_ptr());
            let arg = miniextendr_bench::fixtures().real_vec(2); // 256 elements
            let call = ffi::Rf_protect(ffi::Rf_lang2(sym, arg));
            (call, ProtectGuard::new(1))
        })
        .bench_local_refs(|(call, _guard)| unsafe {
            let out = ffi::Rf_eval(*call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

// =============================================================================
// class_methods: compare class system dispatch overhead
// =============================================================================

/// Env-style: obj$value() via `$` dispatch.
#[divan::bench]
fn class_env_method(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe { parse_and_protect(".__bench_env_obj__$value()") })
        .bench_local_refs(|parsed| unsafe {
            let out = ffi::Rf_eval(parsed.expr, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// R6-style: obj$value() via R6 dispatch.
#[divan::bench]
fn class_r6_method(bencher: divan::Bencher) {
    let available =
        unsafe { ffi::Rf_asLogical(r_eval_string("requireNamespace('R6', quietly = TRUE)")) };
    if available != 1 {
        return;
    }

    bencher
        .with_inputs(|| unsafe { parse_and_protect(".__bench_r6_obj__$value()") })
        .bench_local_refs(|parsed| unsafe {
            let out = ffi::Rf_eval(parsed.expr, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// S3-style: generic dispatch via UseMethod.
#[divan::bench]
fn class_s3_method(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe { parse_and_protect(".__bench_s3_value__(.__bench_s3_obj__)") })
        .bench_local_refs(|parsed| unsafe {
            let out = ffi::Rf_eval(parsed.expr, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// S4-style: generic dispatch via setMethod.
#[divan::bench]
fn class_s4_method(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe { parse_and_protect(".__bench_s4_value__(.__bench_s4_obj__)") })
        .bench_local_refs(|parsed| unsafe {
            let out = ffi::Rf_eval(parsed.expr, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// S7-style: generic dispatch via S7.
#[divan::bench]
fn class_s7_method(bencher: divan::Bencher) {
    let available =
        unsafe { ffi::Rf_asLogical(r_eval_string("requireNamespace('S7', quietly = TRUE)")) };
    if available != 1 {
        return;
    }

    bencher
        .with_inputs(|| unsafe { parse_and_protect(".__bench_s7_value__(.__bench_s7_obj__)") })
        .bench_local_refs(|parsed| unsafe {
            let out = ffi::Rf_eval(parsed.expr, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}

/// Baseline: plain R function call with no dispatch overhead.
#[divan::bench]
fn class_baseline_plain_fn(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe { parse_and_protect(".__bench_noop__(42L)") })
        .bench_local_refs(|parsed| unsafe {
            let out = ffi::Rf_eval(parsed.expr, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}
