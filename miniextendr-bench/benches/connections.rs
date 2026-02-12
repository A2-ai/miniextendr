//! Connection benchmarks (feature-gated).

#[cfg(feature = "connections")]
use miniextendr_api::connection::{
    RConnectionIo, get_connection, read_connection, write_connection,
};
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

/// Create a connection backed by `size` bytes of data, pre-written so reads
/// return actual data. Uses read-write-seek mode for rewind between iterations.
#[cfg(feature = "connections")]
fn make_readable_connection(size: usize) -> ProtectedConn {
    let data = vec![0xABu8; size];
    let cursor = Cursor::new(data);
    let sexp = RConnectionIo::new(cursor)
        .description("bench readable connection")
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

// =============================================================================
// Read benchmarks at different sizes
// =============================================================================

/// Read 128 bytes from a small connection.
#[cfg(feature = "connections")]
#[divan::bench]
fn connection_read_small(bencher: divan::Bencher) {
    let mut buf = [0u8; 128];
    bencher
        .with_inputs(|| make_readable_connection(128))
        .bench_local_refs(|conn| unsafe {
            let handle = get_connection(conn.sexp);
            let n = read_connection(handle, &mut buf);
            divan::black_box(n);
        });
}

/// Read 4096 bytes from a larger connection.
#[cfg(feature = "connections")]
#[divan::bench]
fn connection_read_large(bencher: divan::Bencher) {
    let mut buf = [0u8; 4096];
    bencher
        .with_inputs(|| make_readable_connection(4096))
        .bench_local_refs(|conn| unsafe {
            let handle = get_connection(conn.sexp);
            let n = read_connection(handle, &mut buf);
            divan::black_box(n);
        });
}

// =============================================================================
// Write benchmark with larger payload
// =============================================================================

/// Write 4096 bytes to a connection (vs the existing 128-byte write).
#[cfg(feature = "connections")]
#[divan::bench]
fn connection_write_large(bencher: divan::Bencher) {
    let buf = [1u8; 4096];
    bencher
        .with_inputs(make_connection)
        .bench_local_refs(|conn| unsafe {
            let handle = get_connection(conn.sexp);
            let written = write_connection(handle, &buf);
            divan::black_box(written);
        });
}
