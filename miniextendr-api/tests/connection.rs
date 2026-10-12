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

// region: I/O failures and R's seek return contract

struct BrokenIo;
impl Read for BrokenIo {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("read refused"))
    }
}
impl Write for BrokenIo {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("write refused"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("flush refused"))
    }
}
impl BufRead for BrokenIo {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Err(std::io::Error::other("read refused"))
    }
    fn consume(&mut self, _: usize) {}
}
impl std::io::Seek for BrokenIo {
    fn seek(&mut self, _: std::io::SeekFrom) -> std::io::Result<u64> {
        Err(std::io::Error::other("seek refused"))
    }
}

fn expect_io_error<T>(operation: &str, f: impl FnOnce() -> T) {
    use miniextendr_api::condition::{RCondition, catch_condition};
    use miniextendr_api::rvalue::RValue;
    let condition = match catch_condition(f) {
        Err(condition) => condition,
        Ok(_) => panic!("I/O failure was swallowed"),
    };
    let RCondition::Error {
        message,
        class,
        data,
        ..
    } = condition
    else {
        panic!("expected error condition")
    };
    assert_eq!(class, ["miniextendr_connection_io_error"]);
    assert!(message.contains(operation), "{message}");
    let data = data.unwrap();
    assert!(data.iter().any(|(name, value)| name == "operation"
        && matches!(value, RValue::Character(v) if v == &[Some(operation.to_string())])));
    assert!(data.iter().any(|(name, _)| name == "io_kind"));
    assert!(data.iter().any(|(name, _)| name == "os_error"));
}

#[test]
fn every_io_adapter_preserves_failures() {
    use miniextendr_api::connection::*;
    macro_rules! read_failure {
        ($($adapter:ident),*) => {$(
            let mut adapter = $adapter::new(BrokenIo);
            expect_io_error("read", || adapter.read(&mut [0; 1]));
            expect_io_error("read", || adapter.fgetc());
        )*};
    }
    read_failure!(IoRead, IoReadWrite, IoReadSeek, IoReadWriteSeek, IoBufRead);
    macro_rules! write_failure {
        ($($adapter:ident),*) => {$(
            let mut adapter = $adapter::new(BrokenIo);
            expect_io_error("write", || adapter.write(&[0]));
            expect_io_error("flush", || adapter.flush());
        )*};
    }
    write_failure!(IoWrite, IoReadWrite, IoWriteSeek, IoReadWriteSeek);
    macro_rules! seek_failure {
        ($($adapter:ident),*) => {$(
            let mut adapter = $adapter::new(BrokenIo);
            expect_io_error("seek", || adapter.seek(0.0, 1, 0));
        )*};
    }
    seek_failure!(IoReadSeek, IoWriteSeek, IoReadWriteSeek);
}

#[test]
fn seek_adapters_return_old_positions_and_reject_invalid_offsets() {
    use miniextendr_api::connection::*;
    macro_rules! check {
        ($($adapter:ident),*) => {$(
            let mut adapter = $adapter::new(Cursor::new(vec![0_u8; 8]));
            assert_eq!(adapter.seek(3.0, 1, 0), 0.0);
            assert_eq!(adapter.seek(f64::NAN, 1, 0), 3.0);
            assert_eq!(adapter.seek(2.0, 2, 0), 3.0);
            assert_eq!(adapter.seek(-1.0, 3, 0), 5.0);
            // R truncates a fractional offset.
            assert_eq!(adapter.seek(2.9, 1, 0), 7.0);
            assert_eq!(adapter.seek(-1.5, 2, 0), 2.0);
            // A position before the start stays put, as in R's file connections.
            assert_eq!(adapter.seek(-1.0, 1, 0), 1.0);
            assert_eq!(adapter.seek(f64::NAN, 1, 0), 1.0);
            for offset in [f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
                expect_io_error("seek", || adapter.seek(offset, 1, 0));
                assert_eq!(adapter.seek(f64::NAN, 1, 0), 1.0);
            }
            // The stream's own refusal.
            expect_io_error("seek", || adapter.seek(-5.0, 2, 0));
            expect_io_error("seek", || adapter.seek(0.0, 99, 0));
        )*};
    }
    check!(IoReadSeek, IoWriteSeek, IoReadWriteSeek);
}

/// A stream that moves one byte per call and reports `Interrupted` before
/// each one, as `Read` / `Write` may.
#[derive(Default)]
struct Trickle {
    data: Rc<RefCell<Vec<u8>>>,
    position: usize,
    interrupted: bool,
}
impl Trickle {
    fn interrupt(&mut self) -> std::io::Result<()> {
        self.interrupted = !self.interrupted;
        if self.interrupted {
            Err(std::io::ErrorKind::Interrupted.into())
        } else {
            Ok(())
        }
    }
}
impl Read for Trickle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.interrupt()?;
        let Some(&byte) = self.data.borrow().get(self.position) else {
            return Ok(0);
        };
        buf[0] = byte;
        self.position += 1;
        Ok(1)
    }
}
impl Write for Trickle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.interrupt()?;
        self.data.borrow_mut().push(buf[0]);
        Ok(1)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.interrupt()
    }
}

#[test]
fn short_and_interrupted_transfers_move_every_byte() {
    r_test_utils::with_r_thread(|| unsafe {
        let data = Rc::new(RefCell::new(Vec::new()));
        let writer = Trickle {
            data: data.clone(),
            ..Trickle::default()
        };
        let conn = OwnedProtect::new(RConnectionIo::new(writer).build_write());
        RCall::new("writeBin")
            .arg_value(vec![1_i32, 2])
            .arg(conn.get())
            .eval_base()
            .unwrap();
        RCall::new("flush").arg(conn.get()).eval_base().unwrap();
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
        assert_eq!(data.borrow().len(), 8);

        let reader = Trickle {
            data,
            ..Trickle::default()
        };
        let conn = OwnedProtect::new(RConnectionIo::new(reader).build_read());
        let value = OwnedProtect::new(
            RCall::new("readBin")
                .arg(conn.get())
                .arg_value("integer")
                .arg_value(3_i32)
                .eval_base()
                .unwrap(),
        );
        assert_eq!(Vec::<i32>::try_from_sexp(value.get()).unwrap(), [1, 2]);
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
    });
}

/// Hands out its chunks one read at a time; between them it has nothing yet,
/// like a non-blocking stream.
struct Pending(Vec<&'static [u8]>);
impl Read for Pending {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.0.pop() {
            Some([]) => Err(std::io::ErrorKind::WouldBlock.into()),
            Some(chunk) => {
                buf[..chunk.len()].copy_from_slice(chunk);
                Ok(chunk.len())
            }
            None => Ok(0),
        }
    }
}
impl BufRead for Pending {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self.0.last() {
            Some([]) => {
                self.0.pop();
                Err(std::io::ErrorKind::WouldBlock.into())
            }
            Some(chunk) => Ok(chunk),
            None => Ok(&[]),
        }
    }
    fn consume(&mut self, amount: usize) {
        let chunk = self.0.pop().unwrap();
        if amount < chunk.len() {
            self.0.push(&chunk[amount..]);
        }
    }
}

#[test]
fn a_stream_with_nothing_more_now_returns_what_it_has() {
    use miniextendr_api::connection::*;
    let mut adapter = IoRead::new(Pending(vec![b"cd", b"", b"ab"]));
    let mut buf = [0; 4];
    assert_eq!(adapter.read(&mut buf), 2);
    assert_eq!(&buf[..2], b"ab");
    assert_eq!(adapter.read(&mut buf), 2);
    assert_eq!(&buf[..2], b"cd");
    assert_eq!(adapter.read(&mut buf), 0);

    let mut adapter = IoBufRead::new(Pending(vec![b"b", b"", b"a"]));
    assert_eq!(adapter.fgetc(), i32::from(b'a'));
    assert_eq!(adapter.fgetc(), -1);
    assert_eq!(adapter.fgetc(), i32::from(b'b'));
    assert_eq!(adapter.fgetc(), -1);
}

#[test]
fn r_receives_io_conditions_and_can_close_after_failure() {
    r_test_utils::with_r_thread(|| unsafe {
        let conn = OwnedProtect::new(RConnectionIo::new(BrokenIo).build_read_write_seek());
        for error in [
            RCall::new("readBin")
                .arg(conn.get())
                .arg_value("raw")
                .arg_value(1_i32)
                .eval_base()
                .unwrap_err(),
            RCall::new("writeBin")
                .arg_value(vec![1_u8])
                .arg(conn.get())
                .eval_base()
                .unwrap_err(),
            RCall::new("flush").arg(conn.get()).eval_base().unwrap_err(),
            RCall::new("seek")
                .arg(conn.get())
                .arg_value(0_f64)
                .eval_base()
                .unwrap_err(),
        ] {
            assert!(error.inherits("miniextendr_connection_io_error"), "{error}");
            assert!(error.message().contains("refused"), "{error}");
        }
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
        let conn = OwnedProtect::new(RConnectionIo::new(BrokenIo).mode("r").build_bufread());
        let error = RCall::new("readLines")
            .arg(conn.get())
            .eval_base()
            .unwrap_err();
        assert!(error.inherits("miniextendr_connection_io_error"), "{error}");
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
    });
}

#[test]
fn r_seek_returns_the_position_before_the_move() {
    r_test_utils::with_r_thread(|| unsafe {
        let conn = OwnedProtect::new(
            RConnectionIo::new(Cursor::new(vec![0_u8; 8])).build_read_write_seek(),
        );
        for (where_, previous) in [(3.0, 0.0), (5.0, 3.0), (f64::NAN, 5.0)] {
            let value = OwnedProtect::new(
                RCall::new("seek")
                    .arg(conn.get())
                    .arg_value(where_)
                    .eval_base()
                    .unwrap(),
            );
            assert_eq!(f64::try_from_sexp(value.get()).unwrap(), previous);
        }
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
    });
}

#[test]
fn r_rereads_a_non_blocking_seekable_connection() {
    // readLines() re-positions an open non-blocking connection with
    // seek(con, seek(con, -1, 1, 1), 1, 1) before reading on.
    r_test_utils::with_r_thread(|| unsafe {
        let conn = OwnedProtect::new(
            RConnectionIo::new(Cursor::new(b"a\nb\nc\n".to_vec()))
                .mode("r")
                .blocking(false)
                .build_read_seek(),
        );
        let read_lines = |n: i32| {
            let lines = OwnedProtect::new(
                RCall::new("readLines")
                    .arg(conn.get())
                    .arg_value(n)
                    .eval_base()
                    .unwrap(),
            );
            Vec::<String>::try_from_sexp(lines.get()).unwrap()
        };
        assert_eq!(read_lines(1), ["a"]);
        assert_eq!(read_lines(-1), ["b", "c"]);
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
    });
}

#[test]
fn r_refuses_seek_and_truncate_where_the_stream_has_none() {
    r_test_utils::with_r_thread(|| unsafe {
        let conn = OwnedProtect::new(RConnectionIo::new(Vec::<u8>::new()).build_write());
        let error = RCall::new("seek").arg(conn.get()).eval_base().unwrap_err();
        assert!(
            error
                .message()
                .contains("'seek' not enabled for this connection"),
            "{error}"
        );
        let error = RCall::new("truncate")
            .arg(conn.get())
            .eval_base()
            .unwrap_err();
        assert!(
            error
                .message()
                .contains("truncation not enabled for this connection"),
            "{error}"
        );
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
    });
}

/// Queues a warning on every read; warning!() would end the callback.
struct DeferringSource;
impl miniextendr_api::connection::RConnectionImpl for DeferringSource {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        miniextendr_api::defer_warning!("deferred from read");
        buf[0] = 7;
        1
    }
}

#[test]
fn a_warning_deferred_in_a_callback_reaches_r_handlers() {
    use miniextendr_api::sys::R_GlobalEnv;
    r_test_utils::with_r_thread(|| unsafe {
        let conn = OwnedProtect::new(RCustomConnection::new().mode("rb").build(DeferringSource));
        RCall::new("assign")
            .arg_value("con")
            .arg(conn.get())
            .arg(R_GlobalEnv)
            .eval_base()
            .unwrap();
        let seen = OwnedProtect::new(
            miniextendr_api::r_eval_str_global(
                r#"local({
                    seen <- NULL
                    value <- withCallingHandlers(
                        readBin(con, "raw", 1L),
                        warning = function(w) {
                            seen <<- conditionMessage(w)
                            invokeRestart("muffleWarning")
                        }
                    )
                    c(seen, as.character(value))
                })"#,
            )
            .unwrap(),
        );
        assert_eq!(
            Vec::<String>::try_from_sexp(seen.get()).unwrap(),
            ["deferred from read", "07"]
        );
        RCall::new("close").arg(conn.get()).eval_base().unwrap();
        miniextendr_api::r_eval_str_global("rm(con)").unwrap();
    });
}
// endregion

// region: Writes that return R's error

#[test]
fn null_connection_writes_return_r_errors() {
    use miniextendr_api::connection::RNullConnection;
    use miniextendr_api::expression::r_eval_str_global;

    r_test_utils::with_r_thread(|| unsafe {
        // A connection whose write callback raises: every write is an `Err`,
        // and the handle keeps working.
        let failing = OwnedProtect::new(RConnectionIo::new(BrokenIo).build_write());
        let mut con = RNullConnection::try_from_sexp(failing.get()).unwrap();
        for _ in 0..2 {
            let error = con.write_all(b"x").unwrap_err();
            assert!(error.to_string().contains("write refused"), "{error}");
        }
        drop(con);
        RCall::new("suppressWarnings")
            .arg(RCall::new("close").arg(failing.get()).build())
            .eval_base()
            .unwrap();

        // A connection R closed while the handle borrowed it.
        let file = OwnedProtect::new(r_eval_str_global("file(nullfile(), \"w\")").unwrap());
        let mut con = RNullConnection::try_from_sexp(file.get()).unwrap();
        con.write_all(b"open\n").unwrap();
        RCall::new("close").arg(file.get()).eval_base().unwrap();
        let error = con.write_all(b"closed\n").unwrap_err();
        assert!(error.to_string().contains("invalid connection"), "{error}");
    });
}
// endregion
