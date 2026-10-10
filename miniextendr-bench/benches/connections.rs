//! Connection benchmarks (feature-gated).
//!
//! `RConnectionIo::build_*` opens the connection it creates, so reads and
//! writes work on it right away.

#[cfg(feature = "connections")]
use miniextendr_api::SEXP;
#[cfg(feature = "connections")]
use miniextendr_api::connection::{RConnectionIo, read_connection, write_connection};
#[cfg(feature = "connections")]
use miniextendr_bench::raw_ffi;
#[cfg(feature = "connections")]
use std::io::Cursor;

#[cfg(feature = "connections")]
struct ProtectedConn {
    sexp: SEXP,
}

#[cfg(feature = "connections")]
impl Drop for ProtectedConn {
    fn drop(&mut self) {
        unsafe {
            raw_ffi::Rf_unprotect(1);
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
        raw_ffi::Rf_protect(sexp);
    }
    ProtectedConn { sexp }
}

/// Create a connection backed by `size` bytes of data, pre-written so reads
/// return actual data. Uses read-write-seek mode.
#[cfg(feature = "connections")]
fn make_readable_connection(size: usize) -> ProtectedConn {
    let data = vec![0xABu8; size];
    let cursor = Cursor::new(data);
    let sexp = RConnectionIo::new(cursor)
        .description("bench readable connection")
        .mode("rb+")
        .build_read_write_seek();
    unsafe {
        raw_ffi::Rf_protect(sexp);
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
    let conn = make_connection();
    let buf = [1u8; 128];
    bencher.bench_local(|| unsafe {
        let written = write_connection(conn.sexp, &buf);
        divan::black_box(written);
    });
}

// region: Parameterized read/write benchmarks

#[cfg(feature = "connections")]
const IO_SIZES: &[usize] = &[64, 256, 1024, 4096, 16384];

/// Read benchmark. We allocate a large backing buffer (size * 10_000) so the
/// cursor does not exhaust across iterations.
#[cfg(feature = "connections")]
#[divan::bench(args = IO_SIZES)]
fn connection_read(bencher: divan::Bencher, size: usize) {
    let conn = make_readable_connection(size * 10_000);
    let mut buf = vec![0u8; size];
    bencher.bench_local(|| unsafe {
        let n = read_connection(conn.sexp, &mut buf);
        divan::black_box(n);
    });
}

#[cfg(feature = "connections")]
#[divan::bench(args = IO_SIZES)]
fn connection_write_sized(bencher: divan::Bencher, size: usize) {
    let conn = make_connection();
    let buf = vec![1u8; size];
    bencher.bench_local(|| unsafe {
        let written = write_connection(conn.sexp, &buf);
        divan::black_box(written);
    });
}
// endregion

// region: Sequential writes — measure throughput under repeated writes

#[cfg(feature = "connections")]
const WRITE_COUNTS: &[usize] = &[1, 10, 50];

#[cfg(feature = "connections")]
#[divan::bench(args = WRITE_COUNTS)]
fn connection_burst_write(bencher: divan::Bencher, n_writes: usize) {
    let conn = make_connection();
    let buf = [1u8; 256];
    bencher.bench_local(|| unsafe {
        let mut total = 0;
        for _ in 0..n_writes {
            total += write_connection(conn.sexp, &buf);
        }
        divan::black_box(total);
    });
}
// endregion
