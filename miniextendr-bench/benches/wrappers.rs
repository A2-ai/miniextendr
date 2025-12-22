//! R eval wrapper benchmarks.

use miniextendr_api::ffi;

struct ProtectedCall {
    call: ffi::SEXP,
}

impl Drop for ProtectedCall {
    fn drop(&mut self) {
        unsafe {
            ffi::Rf_unprotect(1);
        }
    }
}

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench]
fn eval_sum(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let sym = ffi::Rf_install(c"sum".as_ptr());
            let call = ffi::Rf_lang2(sym, miniextendr_bench::fixtures().real_vec(2));
            ffi::Rf_protect(call);
            ProtectedCall { call }
        })
        .bench_local_refs(|call| unsafe {
            let out = ffi::Rf_eval(call.call, ffi::R_GlobalEnv);
            divan::black_box(out);
        });
}
