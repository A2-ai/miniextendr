//! The framework's own access to an R connection given as a SEXP: its state
//! and its bytes.
//!
//! Built against R >= 4.6 (`r_4_6`), it goes through `R_GetConnection` and
//! the [`Rconn`](super::Rconn) struct, `R_ReadConnection` and
//! `R_WriteConnection`, which R 4.6 lists as experimental API. R 4.4 and 4.5
//! export the same entry points, but `R CMD check` notes them as non-API
//! there, so before R 4.6 the same state and bytes come from R functions
//! (#1905): `summary.connection()`, `isSeekable()`, `readBin()` /
//! `writeBin()` on a binary connection and `readChar()` / `writeChar()` with
//! `useBytes = TRUE` on a text-mode one. Each of those calls the
//! connection's `read` / `write` callback, as the C entry points do.
//!
//! R has no R-level getter for the blocking flag, so [`is_blocking`] is
//! `None` before R 4.6.
//!
//! [`write`] raises R's error (before R 4.6, panics with its message);
//! [`write_caught`] returns it as an `io::Error`, for a caller that must not
//! unwind, such as a `std::io::Write` impl or a progress-bar draw.

/// A connection's state, the fields `summary()` reports.
pub(crate) struct ConnState {
    /// The description (`""` when the connection has none).
    pub(crate) description: String,
    /// The connection class, e.g. `"file"` or `"terminal"`.
    pub(crate) class: String,
    /// The mode string, e.g. `"r"` or `"wb"`.
    pub(crate) mode: String,
    /// Text mode (not binary).
    pub(crate) is_text: bool,
    /// Open.
    pub(crate) is_open: bool,
    /// Can be read.
    pub(crate) can_read: bool,
    /// Can be written.
    pub(crate) can_write: bool,
}

pub(crate) use imp::{can_seek, is_blocking, read, state, write, write_caught};

#[cfg(r_4_6)]
mod imp {
    use std::ffi::CStr;
    use std::io;
    use std::os::raw::{c_char, c_void};

    use super::ConnState;
    use crate::connection::Rconn;
    use crate::{Rboolean, SEXP};

    // # Safety
    // `sexp` must be an R connection; R errors (longjmp) on anything else.
    unsafe fn rconn(sexp: SEXP) -> *const Rconn {
        unsafe { crate::sys::R_GetConnection(sexp) }
            .cast::<Rconn>()
            .cast_const()
    }

    // # Safety
    // `ptr` is null or a NUL-terminated string.
    unsafe fn string(ptr: *const c_char) -> String {
        if ptr.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned()
        }
    }

    /// The connection's state.
    ///
    /// # Safety
    ///
    /// `sexp` is an R connection; called on R's main thread.
    pub(crate) unsafe fn state(sexp: SEXP) -> ConnState {
        unsafe {
            let conn = rconn(sexp);
            ConnState {
                description: string((*conn).description),
                class: string((*conn).class),
                mode: string((*conn).mode.as_ptr()),
                is_text: (*conn).text != Rboolean::FALSE,
                is_open: (*conn).isopen != Rboolean::FALSE,
                can_read: (*conn).canread != Rboolean::FALSE,
                can_write: (*conn).canwrite != Rboolean::FALSE,
            }
        }
    }

    /// Whether the connection can seek.
    ///
    /// # Safety
    ///
    /// As [`state`].
    pub(crate) unsafe fn can_seek(sexp: SEXP) -> bool {
        unsafe { (*rconn(sexp)).canseek != Rboolean::FALSE }
    }

    /// Whether the connection blocks.
    ///
    /// # Safety
    ///
    /// As [`state`].
    pub(crate) unsafe fn is_blocking(sexp: SEXP) -> Option<bool> {
        Some(unsafe { (*rconn(sexp)).blocking != Rboolean::FALSE })
    }

    /// Read up to `buf.len()` bytes; R errors if the connection is not open
    /// or cannot read.
    ///
    /// # Safety
    ///
    /// As [`state`].
    pub(crate) unsafe fn read(sexp: SEXP, buf: &mut [u8]) -> usize {
        unsafe {
            let conn = crate::sys::R_GetConnection(sexp);
            crate::sys::R_ReadConnection(conn, buf.as_mut_ptr().cast::<c_void>(), buf.len())
        }
    }

    /// Write `buf`; R errors if the connection is not open or cannot write.
    ///
    /// # Safety
    ///
    /// As [`state`].
    pub(crate) unsafe fn write(sexp: SEXP, buf: &[u8]) -> usize {
        unsafe {
            let conn = crate::sys::R_GetConnection(sexp);
            crate::sys::R_WriteConnection(conn, buf.as_ptr().cast::<c_void>(), buf.len())
        }
    }

    /// [`write`] with R's error (no connection, not open, cannot write, or
    /// a write callback that raises) returned as an `io::Error`. The count
    /// is the write callback's, which can be short.
    ///
    /// # Safety
    ///
    /// `sexp` rooted; called on R's main thread.
    pub(crate) unsafe fn write_caught(sexp: SEXP, buf: &[u8]) -> io::Result<usize> {
        struct Write<'a> {
            sexp: SEXP,
            buf: &'a [u8],
            written: usize,
        }

        // An R error jumps out of here, over nothing to drop.
        unsafe extern "C-unwind" fn body(data: *mut c_void) -> SEXP {
            let data = data.cast::<Write<'_>>();
            unsafe { (*data).written = write((*data).sexp, (*data).buf) };
            SEXP::nil()
        }

        let mut data = Write {
            sexp,
            buf,
            written: 0,
        };
        unsafe { crate::expression::try_call(body, (&raw mut data).cast()) }
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(data.written)
    }
}

#[cfg(not(r_4_6))]
mod imp {
    use std::io;

    use super::ConnState;
    use crate::expression::RCall;
    use crate::gc_protect::OwnedProtect;
    use crate::{SEXP, SexpExt};

    // Evaluate `call` in base, with R's error as an `io::Error`.
    unsafe fn try_eval(call: RCall) -> io::Result<SEXP> {
        unsafe { call.eval_base() }.map_err(|e| io::Error::other(e.to_string()))
    }

    // Evaluate `call` in base, panicking with R's message on an R error (a
    // panic becomes an R error at the `#[miniextendr]` boundary).
    unsafe fn eval(call: RCall) -> SEXP {
        unsafe { try_eval(call) }.unwrap_or_else(|e| panic!("{e}"))
    }

    // An `n` / `nchars` argument, which R takes as an integer.
    fn r_int(n: usize) -> SEXP {
        SEXP::scalar_integer(
            i32::try_from(n).expect("a connection read or write of over i32::MAX bytes"),
        )
    }

    /// The connection's state.
    ///
    /// # Safety
    ///
    /// Called on R's main thread.
    ///
    /// # Panics
    ///
    /// With R's message when `sexp` is no connection.
    pub(crate) unsafe fn state(sexp: SEXP) -> ConnState {
        unsafe { try_state(sexp) }.unwrap_or_else(|e| panic!("{e}"))
    }

    // [`state`] with R's error as an `io::Error`.
    unsafe fn try_state(sexp: SEXP) -> io::Result<ConnState> {
        unsafe {
            // A list of seven strings, always in this order: description,
            // class, mode, text, opened, can read, can write.
            let summary = OwnedProtect::new(try_eval(RCall::new("summary.connection").arg(sexp))?);
            // Read lossily: the description is a file path or URL the user
            // chose, which need not be text, and only goes into messages.
            let field = |i: isize| {
                crate::from_r::charsxp_to_string_lossy(summary.get().vector_elt(i).string_elt(0))
                    .unwrap_or_default()
            };
            Ok(ConnState {
                description: field(0),
                class: field(1),
                mode: field(2),
                is_text: field(3) == "text",
                is_open: field(4) == "opened",
                can_read: field(5) == "yes",
                can_write: field(6) == "yes",
            })
        }
    }

    /// Whether the connection can seek: `isSeekable()`.
    ///
    /// # Safety
    ///
    /// As [`state`].
    pub(crate) unsafe fn can_seek(sexp: SEXP) -> bool {
        unsafe { eval(RCall::new("isSeekable").arg(sexp)) }
            .as_logical()
            .unwrap_or(false)
    }

    /// `None`: R has no R-level getter for the blocking flag.
    ///
    /// # Safety
    ///
    /// None; the signature matches the R >= 4.6 one.
    pub(crate) unsafe fn is_blocking(_sexp: SEXP) -> Option<bool> {
        None
    }

    // `R_ReadConnection` / `R_WriteConnection` refuse a closed connection,
    // where `readBin()` and friends would open it for the call.
    unsafe fn checked_state(sexp: SEXP, write: bool) -> io::Result<ConnState> {
        let state = unsafe { try_state(sexp) }?;
        let refuse = |message| Err(io::Error::other(message));
        if !state.is_open {
            return refuse("connection is not open");
        }
        if write && !state.can_write {
            return refuse("cannot write to this connection");
        }
        if !write && !state.can_read {
            return refuse("cannot read from this connection");
        }
        Ok(state)
    }

    /// Read up to `buf.len()` bytes, at most `i32::MAX`.
    ///
    /// # Safety
    ///
    /// As [`state`].
    ///
    /// # Panics
    ///
    /// When the connection is not open or cannot read, or with R's message
    /// when the read fails.
    pub(crate) unsafe fn read(sexp: SEXP, buf: &mut [u8]) -> usize {
        unsafe {
            let n = r_int(buf.len().min(i32::MAX as usize));
            let state = checked_state(sexp, false).unwrap_or_else(|e| panic!("{e}"));
            let value = if state.is_text {
                // `readChar()` warns that it skips a text-mode connection's
                // pushback and re-encoding; so does `R_ReadConnection`.
                let read = RCall::new("readChar")
                    .arg(sexp)
                    .arg(n)
                    .named_arg("useBytes", SEXP::scalar_logical(true));
                OwnedProtect::new(eval(RCall::new("suppressWarnings").arg(read.build())))
            } else {
                OwnedProtect::new(eval(
                    RCall::new("readBin")
                        .arg(sexp)
                        .arg(SEXP::scalar_string_from_str("raw"))
                        .arg(n),
                ))
            };
            let value = value.get();
            let bytes: &[u8] = if value.is_raw() {
                value.as_slice::<u8>()
            } else if value.len() == 0 {
                // `readChar()` gives `character(0)` at the end of the input.
                &[]
            } else {
                let charsxp = value.string_elt(0);
                std::slice::from_raw_parts(charsxp.r_char().cast::<u8>(), charsxp.len())
            };
            buf[..bytes.len()].copy_from_slice(bytes);
            bytes.len()
        }
    }

    /// Write `buf`; returns the bytes written.
    ///
    /// # Safety
    ///
    /// As [`state`].
    ///
    /// # Panics
    ///
    /// With [`write_caught`]'s error.
    pub(crate) unsafe fn write(sexp: SEXP, buf: &[u8]) -> usize {
        unsafe { write_caught(sexp, buf) }.unwrap_or_else(|e| panic!("{e}"))
    }

    /// Write `buf`, with R's error returned as an `io::Error`; returns the
    /// bytes written.
    ///
    /// A text-mode connection takes a string, so it gets the longest prefix
    /// of `buf` that is UTF-8 without a NUL byte; `0` when there is none, and
    /// `write_all()` then fails with `WriteZero`.
    ///
    /// # Safety
    ///
    /// As [`state`].
    ///
    /// # Errors
    ///
    /// When the connection is not open or cannot write, with R's message
    /// when the write fails, and for non-ASCII text in a multibyte locale
    /// other than UTF-8 (see [`nchars`]).
    pub(crate) unsafe fn write_caught(sexp: SEXP, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            if !checked_state(sexp, true)?.is_text {
                try_eval(RCall::new("writeBin").arg_value(buf).arg(sexp))?;
                return Ok(buf.len());
            }
            let text = match std::str::from_utf8(buf) {
                Ok(text) => text,
                Err(e) => std::str::from_utf8(&buf[..e.valid_up_to()]).expect("a UTF-8 prefix"),
            };
            let text = text.split('\0').next().unwrap_or_default();
            if text.is_empty() {
                return Ok(0);
            }
            let nchars = i32::try_from(nchars(text)?)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            // `eos = NULL` writes no terminator.
            try_eval(
                RCall::new("writeChar")
                    .arg_value(text)
                    .arg(sexp)
                    .named_value("nchars", nchars)
                    .named_arg("eos", SEXP::nil())
                    .named_value("useBytes", true),
            )?;
            Ok(text.len())
        }
    }

    /// `writeChar()`'s `nchars` for all of `text`. It counts characters in a
    /// multibyte locale and bytes in any other, even with `useBytes = TRUE`;
    /// a character count in a single-byte locale writes only that many bytes.
    /// UTF-8 bytes are characters only in a UTF-8 locale, so another
    /// multibyte locale is refused.
    ///
    /// # Safety
    ///
    /// Called on R's main thread.
    unsafe fn nchars(text: &str) -> io::Result<usize> {
        if text.is_ascii() {
            return Ok(text.len());
        }
        // A list whose first two elements are `MBCS` and `UTF-8`.
        let info = unsafe { OwnedProtect::new(try_eval(RCall::new("l10n_info"))?) };
        let flag = |i: isize| info.get().vector_elt(i).as_logical() == Some(true);
        match (flag(0), flag(1)) {
            (false, _) => Ok(text.len()),
            (true, true) => Ok(text.chars().count()),
            (true, false) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "a text-mode connection takes non-ASCII UTF-8 only in a UTF-8 or single-byte locale",
            )),
        }
    }
}
