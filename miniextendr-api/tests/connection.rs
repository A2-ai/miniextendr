#![cfg(feature = "connections")]

mod r_test_utils;

use miniextendr_api::connection::{
    EXPECTED_CONNECTIONS_VERSION, RConnectionIo, RCustomConnection, check_connections_version,
};
use miniextendr_api::expression::RCall;
use miniextendr_api::sys::R_CONNECTIONS_VERSION;
use miniextendr_api::{IntoR, OwnedProtect, TryFromSexp};
use std::cell::{Cell, RefCell};
use std::io::{BufRead, Cursor, Read, Write};
use std::rc::Rc;

#[test]
fn connections_version_constant_matches() {
    assert_eq!(EXPECTED_CONNECTIONS_VERSION, R_CONNECTIONS_VERSION);
    check_connections_version();
}

#[test]
fn mode_panics_when_too_long() {
    let result = std::panic::catch_unwind(|| {
        let _ = RCustomConnection::new().mode("abcdef"); // >4 chars should panic
    });
    assert!(result.is_err(), "mode longer than 4 chars should panic");
}

// region: Boxed trait-object streams
//
// `RConnectionIo<Box<dyn Read>>` and friends: the Box is the sized owner R's
// connection holds, and calls go through the stream's vtable. These run R's
// own readBin / readLines / writeBin / close on the connection.

struct Tracked<T> {
    inner: T,
    drops: Rc<Cell<usize>>,
}

impl<T> Drop for Tracked<T> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
impl<T: Read> Read for Tracked<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}
impl<T: BufRead> BufRead for Tracked<T> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }
    fn consume(&mut self, amount: usize) {
        self.inner.consume(amount);
    }
}
impl<T: Write> Write for Tracked<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[test]
fn boxed_read_stream_preserves_position_eof_and_drop() {
    r_test_utils::with_r_thread(|| unsafe {
        let drops = Rc::new(Cell::new(0));
        let reader: Box<dyn Read> = Box::new(Tracked {
            inner: Cursor::new(vec![1_u8, 2, 3]),
            drops: drops.clone(),
        });
        let conn = OwnedProtect::new(RConnectionIo::new(reader).mode("rb").build_read());
        let raw = OwnedProtect::new("raw".into_sexp());
        let count = OwnedProtect::new(2_i32.into_sexp());
        let read = || {
            let x = OwnedProtect::new(
                RCall::new("readBin")
                    .arg(conn.get())
                    .arg(raw.get())
                    .arg(count.get())
                    .eval_base()
                    .unwrap(),
            );
            Vec::<u8>::try_from_sexp(x.get()).unwrap()
        };
        assert_eq!(read(), [1, 2]);
        assert_eq!(read(), [3]);
        assert!(read().is_empty());
        assert_eq!(drops.get(), 0);
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
        assert_eq!(drops.get(), 1);
        RCall::new("gc").eval_base().unwrap();
        assert_eq!(drops.get(), 1);
    });
}

#[test]
fn boxed_buffered_stream_uses_character_reads() {
    r_test_utils::with_r_thread(|| unsafe {
        let drops = Rc::new(Cell::new(0));
        let reader: Box<dyn BufRead> = Box::new(Tracked {
            inner: Cursor::new(b"one\ntwo\n".to_vec()),
            drops: drops.clone(),
        });
        let conn = OwnedProtect::new(RConnectionIo::new(reader).mode("r").build_bufread());
        let lines = OwnedProtect::new(RCall::new("readLines").arg(conn.get()).eval_base().unwrap());
        assert_eq!(
            Vec::<String>::try_from_sexp(lines.get()).unwrap(),
            ["one", "two"]
        );
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
        assert_eq!(drops.get(), 1);
    });
}

struct SharedSink {
    bytes: Rc<RefCell<Vec<u8>>>,
    flushes: Rc<Cell<usize>>,
}
impl Write for SharedSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.bytes.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.flushes.set(self.flushes.get() + 1);
        Ok(())
    }
}

#[test]
fn boxed_writer_receives_bytes_flush_and_one_drop() {
    r_test_utils::with_r_thread(|| unsafe {
        let bytes = Rc::new(RefCell::new(Vec::new()));
        let flushes = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let writer: Box<dyn Write> = Box::new(Tracked {
            inner: SharedSink {
                bytes: bytes.clone(),
                flushes: flushes.clone(),
            },
            drops: drops.clone(),
        });
        let conn = OwnedProtect::new(RConnectionIo::new(writer).mode("wb").build_write());
        let data = OwnedProtect::new(vec![1_u8, 2, 3].into_sexp());
        RCall::new("writeBin")
            .arg(data.get())
            .arg(conn.get())
            .eval_base()
            .unwrap();
        RCall::new("flush").arg(conn.get()).eval_base().unwrap();
        assert_eq!(&*bytes.borrow(), &[1, 2, 3]);
        assert!(flushes.get() > 0);
        assert_eq!(drops.get(), 0);
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
        assert_eq!(drops.get(), 1);
    });
}
// endregion

// region: Byte iterators as readers

#[cfg(feature = "rand_distr")]
#[test]
fn distribution_iterator_is_readable_as_r_doubles() {
    use miniextendr_api::connection::IteratorReader;
    use miniextendr_api::rand::{SeedableRng, rngs::StdRng};
    use miniextendr_api::rand_distr::{Distribution, Normal};

    r_test_utils::with_r_thread(|| unsafe {
        let distribution = Normal::new(0.0_f64, 1.0).unwrap();
        let expected: Vec<_> = distribution
            .sample_iter(StdRng::seed_from_u64(17))
            .take(4)
            .collect();
        let samples = distribution.sample_iter(StdRng::seed_from_u64(17)).take(4);
        let conn = OwnedProtect::new(
            RConnectionIo::new(IteratorReader::new(samples.flat_map(f64::to_le_bytes)))
                .build_read(),
        );
        let what = OwnedProtect::new("double".into_sexp());
        let n = OwnedProtect::new(2_i32.into_sexp());
        let size = OwnedProtect::new(8_i32.into_sexp());
        let endian = OwnedProtect::new("little".into_sexp());
        let read = || {
            let value = OwnedProtect::new(
                RCall::new("readBin")
                    .arg(conn.get())
                    .arg(what.get())
                    .arg(n.get())
                    .arg(size.get())
                    .named_arg("endian", endian.get())
                    .eval_base()
                    .unwrap(),
            );
            Vec::<f64>::try_from_sexp(value.get()).unwrap()
        };
        assert_eq!(read(), expected[..2]);
        assert_eq!(read(), expected[2..]);
        assert!(read().is_empty());
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
    });
}
// endregion
