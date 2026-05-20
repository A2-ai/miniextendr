//! `indicatif` integration that routes progress-bar output through R
//! connections (issue #178).
//!
//! [`RTerm`] is a `TermLike` adapter holding an R connection SEXP. On each
//! write it dispatches:
//!
//! - If the connection inherits the R class `"terminal"` (i.e. `stdout()` /
//!   `stderr()` / `stdin()`), the bytes are written through R's console hook
//!   (`ptr_R_WriteConsoleEx`, falling back to `Rprintf` / `REprintf`). This is
//!   required because R's terminal connections do **not** wire a `write`
//!   callback — calling `R_WriteConnection` on them raises an R error.
//! - For any other connection (`file()`, `textConnection()`, `sink()` targets,
//!   custom connections), the bytes are handed to `R_WriteConnection`. This is
//!   what makes `sink(file)` / `capture.output()` / file or text connections
//!   capture indicatif progress-bar output.
//!
//! # `Send` / `Sync`
//!
//! `indicatif::TermLike` is declared as `Debug + Send + Sync`, so `RTerm`
//! must also be `Send + Sync` — there is no compile-time guard against
//! off-thread use. Safety is enforced at runtime: every method
//! short-circuits when [`crate::worker::is_r_main_thread`] returns `false`.
//! Off-main-thread writes are silently dropped, mirroring the contract of
//! the pre-existing `RStdout` / `RStderr` `std::io::Write` impls.
//!
//! # Terminal connections bypass `sink("output")`
//!
//! Users redirecting `stdout()` / `stderr()` via `sink()` will still see no
//! bar when the bar targets [`term_like_stdout`] / [`term_like_stderr`] — R's
//! console hook does not consult the `sink()` stack. To capture indicatif
//! output through `sink()`, target a non-terminal connection explicitly via
//! [`term_like_connection`] or build an `RTerm` from a `file()` /
//! `textConnection()` SEXP.
//!
//! # GC / lifetime
//!
//! `RTerm` pins the underlying connection SEXP on R's precious list via
//! `R_PreserveObject` for its lifetime. The corresponding `R_ReleaseObject`
//! runs inside [`crate::externalptr::drop_catching_panic`] so a panic during
//! release aborts rather than unwinding through `Drop` (UB).
//!
//! # `nonapi`
//!
//! The terminal-connection branch still depends on `ptr_R_WriteConsoleEx` /
//! `Rprintf` / `REprintf`, which are gated under the `nonapi` Cargo feature.
//! That is why `indicatif` continues to pull in `nonapi`. Dropping the
//! requirement would need R itself to expose a `write` callback on terminal
//! connections (out of scope).

use indicatif::{ProgressDrawTarget, TermLike};
use std::ffi::CStr;
use std::fmt;
use std::io;

use crate::connection::{RStderr, RStdout, Rconn};
use crate::ffi::{R_PreserveObject, R_ReleaseObject, SEXP, SexpExt};
use crate::into_r::IntoR;

// region: TermKind — precomputed dispatch kind

/// Where an [`RTerm`] writes its bytes.
///
/// Resolved once at construction by inspecting the connection's class and
/// description so per-write dispatch is a branch on a tiny enum rather than
/// repeated `Rf_inherits` / string-compare calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TermKind {
    /// `stdout()` terminal — write via the console hook with otype 0.
    TerminalStdout,
    /// `stderr()` terminal — write via the console hook with otype 1.
    TerminalStderr,
    /// Any other R connection — write via `R_WriteConnection`.
    Connection,
}

// endregion

// region: RTerm

/// `TermLike` adapter that writes indicatif progress-bar output through an R
/// connection SEXP.
///
/// See the [module docs](self) for the dispatch rules (terminal connections
/// go through the console hook, everything else through `R_WriteConnection`)
/// and the `Send` / `Sync` story.
///
/// Construct via [`RTerm::new`] from any R connection SEXP (e.g. the SEXP
/// produced by [`RStdout`] / [`RStderr`] via [`IntoR::into_sexp`], or a
/// user-supplied `textConnection` / `file()` / custom connection). The
/// connection is pinned on R's precious list for the lifetime of the struct
/// and released on `Drop`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::progress::{RTerm, term_like_stderr};
/// use miniextendr_api::indicatif::{ProgressBar, ProgressStyle};
///
/// // Convenience: 80-column stderr terminal.
/// let pb = ProgressBar::with_draw_target(Some(100), term_like_stderr(80));
/// pb.set_style(ProgressStyle::with_template("{bar:40} {pos}/{len}").unwrap());
/// for _ in 0..100 {
///     pb.inc(1);
/// }
/// pb.finish_with_message("done");
/// ```
pub struct RTerm {
    /// Connection SEXP, preserved via `R_PreserveObject`.
    sexp: SEXP,
    width: u16,
    kind: TermKind,
}

// `indicatif::TermLike: Send + Sync`, so `RTerm` must be too. We rely on the
// runtime `is_r_main_thread()` guard inside every method to enforce thread
// safety; off-main-thread calls are silent no-ops.
//
// Safety: the SEXP is only dereferenced inside methods that first check
// `is_r_main_thread()`. `Drop` runs inside `drop_catching_panic`, which is
// invoked whenever the box-holding context drops — in practice that is
// always on the R main thread because `RTerm` is constructed from
// `#[miniextendr]` fns. The `R_ReleaseObject` call inside `Drop` happens
// without an explicit thread check (matching `RTxtProgressBar::drop`); if
// callers ever construct an `RTerm` on the main thread and then move it to
// a non-main thread that drops it, that is unsound — but that pattern
// requires explicit `unsafe impl Send` use which this module does not
// expose externally.
unsafe impl Send for RTerm {}
unsafe impl Sync for RTerm {}

impl RTerm {
    /// Build an `RTerm` from any R connection SEXP.
    ///
    /// The connection is pinned on R's precious list via `R_PreserveObject`
    /// for the lifetime of the struct. The connection kind (terminal vs
    /// non-terminal) is resolved once here, so per-write dispatch is cheap.
    ///
    /// `conn` accepts anything that converts into a SEXP through [`IntoR`] —
    /// most commonly [`RStdout`](crate::connection::RStdout) or
    /// [`RStderr`](crate::connection::RStderr) (which evaluate `base::stdout()`
    /// / `base::stderr()`), or a raw SEXP obtained from an R-side
    /// `textConnection()` / `file()` / custom connection.
    ///
    /// # Panics
    ///
    /// Panics if called from a non-R-main thread — constructing an `RTerm`
    /// needs to evaluate `inherits()` / `R_PreserveObject`, both of which
    /// require the main thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::progress::RTerm;
    /// use miniextendr_api::connection::RStderr;
    ///
    /// let term = RTerm::new(RStderr, 80);
    /// ```
    pub fn new(conn: impl IntoR, width: u16) -> Self {
        assert!(
            crate::worker::is_r_main_thread(),
            "RTerm::new must be called from the R main thread"
        );
        let sexp = conn.into_sexp();
        // PROTECT discipline: `conn.into_sexp()` may return a freshly-allocated
        // unprotected SEXP (e.g. `RStdout` / `RStderr` evaluate `base::stdout()`,
        // which mints a fresh `ScalarInteger(R_OutputCon)` + STRSXP class vec).
        // `classify_connection` calls `R_GetConnection` + a `CStr` deref over the
        // description string; both are GC points. `R_PreserveObject` itself
        // allocates a CONSXP and is also a GC point. Preserve first, classify
        // second to close the window. (Same class of bug as PR #344 commit
        // af6b4875 — R-release CI can pass while R-devel's stricter GC trips.)
        unsafe { R_PreserveObject(sexp) };
        let kind = unsafe { classify_connection(sexp) };
        RTerm { sexp, width, kind }
    }

    /// The underlying connection SEXP. Still preserved on R's precious list.
    #[inline]
    pub fn sexp(&self) -> SEXP {
        self.sexp
    }

    #[inline]
    fn write_bytes(&self, buf: &[u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        if !crate::worker::is_r_main_thread() {
            return Ok(());
        }
        match self.kind {
            TermKind::TerminalStdout => {
                crate::connection::write_console_to(buf, 0);
            }
            TermKind::TerminalStderr => {
                crate::connection::write_console_to(buf, 1);
            }
            TermKind::Connection => unsafe {
                let handle = crate::connection::get_connection(self.sexp);
                let _ = crate::connection::write_connection(handle, buf);
            },
        }
        Ok(())
    }

    #[inline]
    fn write_ansi(&self, seq: &str) -> io::Result<()> {
        self.write_bytes(seq.as_bytes())
    }
}

impl fmt::Debug for RTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Reading the description dereferences the SEXP — only do that on the
        // main thread.
        let description = if crate::worker::is_r_main_thread() {
            unsafe { conn_description(self.sexp) }.unwrap_or_else(|| "<unknown>".to_string())
        } else {
            "<off-main-thread>".to_string()
        };
        let kind = match self.kind {
            TermKind::TerminalStdout => "terminal-stdout",
            TermKind::TerminalStderr => "terminal-stderr",
            TermKind::Connection => "connection",
        };
        f.debug_struct("RTerm")
            .field("description", &description)
            .field("width", &self.width)
            .field("kind", &kind)
            .finish()
    }
}

impl Drop for RTerm {
    fn drop(&mut self) {
        let sexp = self.sexp;
        // Use `drop_catching_panic` so a panic during release aborts rather
        // than unwinding through the destructor stack (UB).
        crate::externalptr::drop_catching_panic(|| unsafe {
            R_ReleaseObject(sexp);
        });
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
        self.write_bytes(s.as_bytes())
    }

    fn clear_line(&self) -> io::Result<()> {
        self.write_str("\r\x1b[2K")
    }

    fn flush(&self) -> io::Result<()> {
        if !crate::worker::is_r_main_thread() {
            return Ok(());
        }
        match self.kind {
            TermKind::TerminalStdout | TermKind::TerminalStderr => {
                unsafe { crate::ffi::R_FlushConsole() };
            }
            // Non-terminal connections flush on close or on the next write;
            // R's connection API doesn't expose a portable generic flush
            // entry point. Treating this as a no-op matches `RStdout` /
            // `RStderr`'s `std::io::Write::flush` semantics.
            TermKind::Connection => {}
        }
        Ok(())
    }
}

// endregion

// region: Connection classification helpers

// Decide which dispatch path a connection SEXP should use.
//
// # Safety
// - `sexp` must be a valid R connection SEXP.
// - Must be called from the R main thread.
unsafe fn classify_connection(sexp: SEXP) -> TermKind {
    if !sexp.inherits_class(c"terminal") {
        return TermKind::Connection;
    }
    // Terminal connection — distinguish stdout vs stderr by the
    // connection's description string.
    let desc = unsafe { conn_description(sexp) }.unwrap_or_default();
    match desc.as_str() {
        "stderr" => TermKind::TerminalStderr,
        // Default: anything else (including "stdin", "stdout", or unknown)
        // routes through the stdout console path. `stdin` cannot meaningfully
        // be a write target — `TermLike::write_str` on a stdin-backed `RTerm`
        // would be a user error, but routing through `Rprintf` is the
        // least-surprise behaviour rather than longjmping.
        _ => TermKind::TerminalStdout,
    }
}

// Read the `description` field from a connection SEXP. Returns `None` if the
// description pointer is null.
//
// # Safety
// - `sexp` must be a valid R connection SEXP.
// - Must be called from the R main thread.
unsafe fn conn_description(sexp: SEXP) -> Option<String> {
    unsafe {
        let handle = crate::connection::get_connection(sexp);
        let conn = handle.cast::<Rconn>().cast_const();
        if conn.is_null() || (*conn).description.is_null() {
            None
        } else {
            Some(
                CStr::from_ptr((*conn).description)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

// endregion

// region: Public factories

/// Convenience: an indicatif draw target backed by R's `stdout()`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::progress::term_like_stdout;
/// use miniextendr_api::indicatif::ProgressBar;
///
/// let pb = ProgressBar::with_draw_target(Some(100), term_like_stdout(80));
/// ```
pub fn term_like_stdout(width: u16) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like(Box::new(RTerm::new(RStdout, width)))
}

/// Convenience: an indicatif draw target backed by R's `stderr()`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::progress::term_like_stderr;
/// use miniextendr_api::indicatif::ProgressBar;
///
/// let pb = ProgressBar::with_draw_target(Some(100), term_like_stderr(80));
/// ```
pub fn term_like_stderr(width: u16) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like(Box::new(RTerm::new(RStderr, width)))
}

/// Convenience: a stdout draw target with a custom refresh rate (Hz).
pub fn term_like_stdout_with_hz(width: u16, refresh_rate: u8) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like_with_hz(Box::new(RTerm::new(RStdout, width)), refresh_rate)
}

/// Convenience: a stderr draw target with a custom refresh rate (Hz).
pub fn term_like_stderr_with_hz(width: u16, refresh_rate: u8) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like_with_hz(Box::new(RTerm::new(RStderr, width)), refresh_rate)
}

/// Build an indicatif draw target from an arbitrary R connection SEXP.
///
/// This is the path that makes `sink()` / `capture.output()` / file / text /
/// custom R connections capture progress-bar output. Bytes are routed via
/// `R_WriteConnection` when the connection isn't a terminal; terminal
/// connections fall back to the console hook (see module docs).
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::progress::term_like_connection;
/// use miniextendr_api::indicatif::ProgressBar;
///
/// // `conn_sexp` is a textConnection / file / custom connection from R.
/// let pb = ProgressBar::with_draw_target(Some(100), term_like_connection(conn_sexp, 80));
/// ```
pub fn term_like_connection(conn: impl IntoR, width: u16) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like(Box::new(RTerm::new(conn, width)))
}

/// Build an indicatif draw target from an arbitrary R connection SEXP with a
/// custom refresh rate (Hz).
pub fn term_like_connection_with_hz(
    conn: impl IntoR,
    width: u16,
    refresh_rate: u8,
) -> ProgressDrawTarget {
    ProgressDrawTarget::term_like_with_hz(Box::new(RTerm::new(conn, width)), refresh_rate)
}

// endregion
