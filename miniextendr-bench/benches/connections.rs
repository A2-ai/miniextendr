//! Connection benchmarks (feature-gated).

#[cfg(feature = "connections")]
use miniextendr_api::connection::{get_connection, write_connection, RConnectionIo};
#[cfg(feature = "connections")]
use miniextendr_api::ffi;
#[cfg(feature = "connections")]
use std::io::Cursor;

#[cfg(feature = "connections")]
struct ProtectedConn {
    sexp: ffi::SEXP,
}

#[cfg(feature = "connections")]
impl Drop for ProtectedConn {
    fn drop(&mut self) {
        unsafe {
            ffi::Rf_unprotect(1);
        }
    }
}

#[cfg(feature = "connections")]
fn make_connection() -> ProtectedConn {
    let data = vec![0u8; 4096];
    let cursor = Cursor::new(data);
    let sexp = RConnectionIo::new(cursor)
        .description("bench connection")
        .mode("rb+")
        .build_read_write_seek();
    unsafe {
        ffi::Rf_protect(sexp);
    }
    ProtectedConn { sexp }
}

#[cfg(feature = "connections")]
fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[cfg(not(feature = "connections"))]
fn main() {}

#[cfg(feature = "connections")]
#[divan::bench]
fn connection_build() {
    let conn = make_connection();
    divan::black_box(conn.sexp);
}

#[cfg(feature = "connections")]
#[divan::bench]
fn connection_write(bencher: divan::Bencher) {
    let buf = [1u8; 128];
    bencher
        .with_inputs(make_connection)
        .bench_local_refs(|conn| unsafe {
            let handle = get_connection(conn.sexp);
            let written = write_connection(handle, &buf);
            divan::black_box(written);
        });
}
