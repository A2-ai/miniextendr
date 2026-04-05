//! Connection benchmarks (feature-gated).
//!
//! `R_new_custom_connection` creates connections in a closed state. We call
//! the `open` callback after building so that `isopen == TRUE` before any
//! read/write operations.

#[cfg(feature = "connections")]
use miniextendr_api::connection::{
    RConnectionIo, Rconn, get_connection, read_connection, write_connection,
};
use miniextendr_api::ffi;
#[cfg(feature = "connections")]
use miniextendr_bench::raw_ffi;
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
            raw_ffi::Rf_unprotect(1);
        }
    }
}

/// Build a connection and open it so read/write operations work.
#[cfg(feature = "connections")]
unsafe fn open_connection(sexp: ffi::SEXP) {
    unsafe {
        let handle = get_connection(sexp).cast::<Rconn>();
        if let Some(open_fn) = (*handle).open {
            open_fn(handle);
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
        open_connection(sexp);
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
        open_connection(sexp);
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
        let handle = get_connection(conn.sexp);
        let written = write_connection(handle, &buf);
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
        let handle = get_connection(conn.sexp);
        let n = read_connection(handle, &mut buf);
        divan::black_box(n);
    });
}

#[cfg(feature = "connections")]
#[divan::bench(args = IO_SIZES)]
fn connection_write_sized(bencher: divan::Bencher, size: usize) {
    let conn = make_connection();
    let buf = vec![1u8; size];
    bencher.bench_local(|| unsafe {
        let handle = get_connection(conn.sexp);
        let written = write_connection(handle, &buf);
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
        let handle = get_connection(conn.sexp);
        let mut total = 0;
        for _ in 0..n_writes {
            total += write_connection(handle, &buf);
        }
        divan::black_box(total);
    });
}
// endregion
