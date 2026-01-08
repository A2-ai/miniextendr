//! indicatif integration for R console output (optional feature).
//!
//! This module provides a `TermLike` implementation that writes to the R
//! console via `ptr_R_WriteConsoleEx`, with ANSI cursor defaults to reduce
//! boilerplate. All output is a no-op off the R main thread.

use indicatif::{ProgressDrawTarget, TermLike};
use std::fmt;
use std::io;
use std::os::raw::{c_char, c_int};

/// Target stream for R console output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

impl RStream {
    #[inline]
    fn otype(self) -> c_int {
        match self {
            Self::Stdout => 0,
            Self::Stderr => 1,
        }
    }
}

/// R console-backed terminal for indicatif.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RTerm {
    stream: RStream,
    width: u16,
}

impl fmt::Debug for RTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RTerm")
            .field("stream", &self.stream)
            .field("width", &self.width)
            .finish()
    }
}

impl RTerm {
    /// Create a new R console terminal with a fixed width.
    pub fn new(stream: RStream, width: u16) -> Self {
        Self { stream, width }
    }

    #[inline]
    fn write_console(&self, bytes: &[u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }

        if !crate::worker::is_r_main_thread() {
            return Ok(());
        }

        unsafe {
            if let Some(write) = crate::ffi::ptr_R_WriteConsoleEx {
                let mut offset = 0;
                while offset < bytes.len() {
                    let remaining = bytes.len() - offset;
                    let chunk = remaining.min(i32::MAX as usize);
                    let ptr = bytes[offset..].as_ptr() as *const c_char;
                    write(ptr, chunk as c_int, self.stream.otype());
                    offset += chunk;
                }
                return Ok(());
            }
        }

        // Fallback to Rprintf/REprintf if console hook is unavailable.
        let cleaned = String::from_utf8_lossy(bytes).replace('\0', "");
        let cstr = std::ffi::CString::new(cleaned)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in output"))?;

        unsafe {
            match self.stream {
                RStream::Stdout => crate::ffi::Rprintf(c"%s".as_ptr(), cstr.as_ptr()),
                RStream::Stderr => crate::ffi::REprintf(c"%s".as_ptr(), cstr.as_ptr()),
            };
        }

        Ok(())
    }

    #[inline]
    fn write_ansi(&self, seq: &str) -> io::Result<()> {
        self.write_console(seq.as_bytes())
    }
}

impl TermLike for RTerm {
    fn width(&self) -> u16 {
        self.width
    }

    fn move_cursor_up(&self, n: usize) -> io::Result<()> {
        if n == 0 {
            return Ok(());
        }
        self.write_ansi(&format!("\x1b[{n}A"))
    }

    fn move_cursor_down(&self, n: usize) -> io::Result<()> {
        if n == 0 {
            return Ok(());
        }
        self.write_ansi(&format!("\x1b[{n}B"))
    }

    fn move_cursor_right(&self, n: usize) -> io::Result<()> {
        if n == 0 {
            return Ok(());
        }
        self.write_ansi(&format!("\x1b[{n}C"))
    }

    fn move_cursor_left(&self, n: usize) -> io::Result<()> {
        if n == 0 {
            return Ok(());
        }
        self.write_ansi(&format!("\x1b[{n}D"))
    }

    fn write_line(&self, s: &str) -> io::Result<()> {
        self.write_str(s)?;
        self.write_str("\n")
    }

    fn write_str(&self, s: &str) -> io::Result<()> {
        self.write_console(s.as_bytes())
    }

    fn clear_line(&self) -> io::Result<()> {
        self.write_str("\r\x1b[2K")
    }

    fn flush(&self) -> io::Result<()> {
        if crate::worker::is_r_main_thread() {
            unsafe { crate::ffi::R_FlushConsole() };
        }
        Ok(())
    }
}

/// Construct a term-like draw target with a stream hint.
pub fn term_like_with_hz_and_stream(
    stream: RStream,
    width: u16,
    refresh_rate: u8,
) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like_with_hz(Box::new(RTerm::new(stream, width)), refresh_rate)
}

/// Convenience: stdout draw target.
pub fn term_like_stdout(width: u16) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like(Box::new(RTerm::new(RStream::Stdout, width)))
}

/// Convenience: stderr draw target.
pub fn term_like_stderr(width: u16) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like(Box::new(RTerm::new(RStream::Stderr, width)))
}

/// Convenience: stdout draw target with custom refresh rate.
pub fn term_like_stdout_with_hz(width: u16, refresh_rate: u8) -> ProgressDrawTarget {
    term_like_with_hz_and_stream(RStream::Stdout, width, refresh_rate)
}

/// Convenience: stderr draw target with custom refresh rate.
pub fn term_like_stderr_with_hz(width: u16, refresh_rate: u8) -> ProgressDrawTarget {
    term_like_with_hz_and_stream(RStream::Stderr, width, refresh_rate)
}
