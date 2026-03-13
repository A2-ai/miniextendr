//! List benchmarks.
//!
//! Focuses on:
//! - named lookup cost (`List::get_named`)
//! - positional lookup baseline (`List::get_index`)
//! - derive-driven struct ↔ list conversions (`IntoList` / `TryFromList`)

use miniextendr_api::ffi;
use miniextendr_api::list::{IntoList as IntoListTrait, List, TryFromList as TryFromListTrait};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[inline(always)]
fn fixtures() -> miniextendr_bench::Fixtures {
    miniextendr_bench::fixtures()
}

const END_KEYS: [&str; 3] = ["k15", "k255", "k4095"];

#[divan::bench(args = [0usize, 1, 2])]
fn list_get_named_first_i32(size_idx: usize) {
    let sexp = fixtures().named_list_i32(size_idx);
    let list = unsafe { List::from_raw(sexp) };
    let val: i32 = list.get_named("k0").unwrap();
    divan::black_box(val);
}

#[divan::bench(args = [0usize, 1, 2])]
fn list_get_named_last_i32(size_idx: usize) {
    let sexp = fixtures().named_list_i32(size_idx);
    let list = unsafe { List::from_raw(sexp) };
    let val: i32 = list.get_named(END_KEYS[size_idx]).unwrap();
    divan::black_box(val);
}

#[divan::bench(args = [0usize, 1, 2])]
fn list_get_index_first_i32(size_idx: usize) {
    let sexp = fixtures().named_list_i32(size_idx);
    let list = unsafe { List::from_raw(sexp) };
    let val: i32 = list.get_index(0).unwrap();
    divan::black_box(val);
}

#[divan::bench(args = [0usize, 1, 2])]
fn list_get_index_last_i32(size_idx: usize) {
    let sexp = fixtures().named_list_i32(size_idx);
    let list = unsafe { List::from_raw(sexp) };
    let last = (miniextendr_bench::NAMED_LIST_SIZES[size_idx] - 1) as isize;
    let val: i32 = list.get_index(last).unwrap();
    divan::black_box(val);
}

// region: Derive-based conversions

#[derive(miniextendr_api::IntoList, miniextendr_api::TryFromList)]
struct Named4 {
    a: i32,
    b: f64,
    c: bool,
    #[into_list(ignore)]
    _d: i32,
}

#[derive(miniextendr_api::IntoList, miniextendr_api::TryFromList)]
#[allow(dead_code)] // Tuple field 1 is ignored by derive but needed for struct layout
struct Tuple3(i32, #[into_list(ignore)] i32, i32);

#[divan::bench]
fn derive_into_list_named() {
    let v = Named4 {
        a: 1,
        b: 2.0,
        c: true,
        _d: 999,
    };
    let list = v.into_list();
    divan::black_box(list.as_sexp());
}

#[divan::bench]
fn derive_into_list_tuple() {
    let v = Tuple3(1, 2, 3);
    let list = v.into_list();
    divan::black_box(list.as_sexp());
}

struct ProtectedSexp {
    sexp: ffi::SEXP,
}

impl Drop for ProtectedSexp {
    fn drop(&mut self) {
        unsafe { ffi::Rf_unprotect(1) };
    }
}

#[divan::bench]
fn derive_try_from_list_named(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let list = Named4 {
                a: 1,
                b: 2.0,
                c: true,
                _d: 999,
            }
            .into_list();
            let sexp = list.as_sexp();
            ffi::Rf_protect(sexp);
            ProtectedSexp { sexp }
        })
        .bench_local_refs(|protected| {
            let list = unsafe { List::from_raw(protected.sexp) };
            let out = Named4::try_from_list(list).unwrap();
            divan::black_box(out.a);
        });
}

#[divan::bench]
fn derive_try_from_list_tuple(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let list = Tuple3(1, 2, 3).into_list();
            let sexp = list.as_sexp();
            ffi::Rf_protect(sexp);
            ProtectedSexp { sexp }
        })
        .bench_local_refs(|protected| {
            let list = unsafe { List::from_raw(protected.sexp) };
            let out = Tuple3::try_from_list(list).unwrap();
            divan::black_box(out.0);
        });
}
// endregion
