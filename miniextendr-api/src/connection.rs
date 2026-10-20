//! Custom R connection framework.
//!
//! This module provides a safe Rust interface for creating custom R connections.
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | `io_adapters` | `std::io::{Read, Write, Seek}` adapters: `IoRead`, `IoWrite`, `IoReadWrite`, etc. |
//!
//! # Core Types
//!
//! - [`RConnectionImpl`] — trait users implement for custom connection behavior
//! - [`RCustomConnection`] — builder for registering connections with R
//! - [`ConnectionCapabilities`] — query connection state (readable, writable, seekable)
//! - [`Rconn`] — C-compatible struct mirroring R's internal `struct Rconn`
//!
//! # WARNING
//!
//! **The R connection API is explicitly unstable.** From R's `R_ext/Connections.h`:
//!
//! > "IMPORTANT: we do not expect future connection APIs to be backward-compatible
//! > so if you use this, you *must* check the version and proceed only if it matches
//! > what you expect. We explicitly reserve the right to change the connection
//! > implementation without a compatibility layer."
//!
//! This module is gated behind the `connections` feature and should be used with caution.
//! Always verify [`sys::R_CONNECTIONS_VERSION`](crate::sys::R_CONNECTIONS_VERSION)
//! matches the expected version before using these APIs (see [`check_connections_version`]).
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::connection::{RConnectionImpl, RCustomConnection};
//!
//! struct MyConnection {
//!     data: Vec<u8>,
//!     position: usize,
//! }
//!
//! impl RConnectionImpl for MyConnection {
//!     fn open(&mut self) -> bool { true }
//!     fn close(&mut self) {}
//!     fn read(&mut self, buf: &mut [u8]) -> usize {
//!         let available = self.data.len() - self.position;
//!         let to_read = buf.len().min(available);
//!         buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
//!         self.position += to_read;
//!         to_read
//!     }
//! }
//!
//! let conn = RCustomConnection::new()
//!     .description("my connection")
//!     .mode("r")
//!     .class_name("myconn")
//!     .can_read(true)
//!     .build(MyConnection { data: vec![1, 2, 3], position: 0 });
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

use crate::sys::{R_CONNECTIONS_VERSION, Rconnection};
use crate::{Rboolean, SEXP};

/// The expected R connections API version this module is compatible with.
///
/// This should match `R_CONNECTIONS_VERSION` from R. If they don't match,
/// connection operations may behave incorrectly or crash.
pub const EXPECTED_CONNECTIONS_VERSION: c_int = 1;

/// Compile-time compatibility assertion for the R connections ABI.
///
/// Compares [`EXPECTED_CONNECTIONS_VERSION`] (what this crate was written for)
/// against [`sys::R_CONNECTIONS_VERSION`](crate::sys::R_CONNECTIONS_VERSION)
/// (the version from R's headers when the FFI bindings were compiled).
/// Both values are compile-time constants, so this is a static consistency
/// check, not a dynamic probe of the running R session.
///
/// # Panics
///
/// Panics if `R_CONNECTIONS_VERSION` doesn't match `EXPECTED_CONNECTIONS_VERSION`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::check_connections_version;
///
/// // Call early, e.g. in R_init_<pkg>, to fail fast on version mismatch
/// check_connections_version();
/// ```
#[inline]
pub fn check_connections_version() {
    assert_eq!(
        R_CONNECTIONS_VERSION, EXPECTED_CONNECTIONS_VERSION,
        "R_CONNECTIONS_VERSION mismatch: expected {}, got {}. \
         The R connections API may have changed incompatibly.",
        EXPECTED_CONNECTIONS_VERSION, R_CONNECTIONS_VERSION
    );
}

/// Check if the running R version supports custom connections.
///
/// The custom connections API (`R_new_custom_connection`) requires R >= 4.3.0,
/// when the API was stabilized with `R_CONNECTIONS_VERSION = 1`.
///
/// This is a **runtime** check using `R.Version()`, complementing the
/// compile-time [`check_connections_version`] assertion.
///
/// # Safety
///
/// Must be called from the R main thread (uses R API calls).
///
/// # Returns
///
/// - `Ok(())` if the running R version supports custom connections
/// - `Err(message)` if the version is too old or could not be determined
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::check_connections_runtime;
///
/// unsafe {
///     if let Err(msg) = check_connections_runtime() {
///         panic!("Connections not supported: {msg}");
///     }
/// }
/// ```
pub unsafe fn check_connections_runtime() -> Result<(), String> {
    // SAFETY: R's main thread (caller contract).
    let (major, minor) = unsafe { running_r_version() }
        .map_err(|e| format!("could not read the running R version: {e}"))?;

    // R_new_custom_connection requires R >= 4.3.0
    if major > 4 || (major == 4 && minor >= 3) {
        Ok(())
    } else {
        Err(format!(
            "Custom connections require R >= 4.3.0, but running R {major}.{minor}"
        ))
    }
}

/// The running R's major and minor version, from `R.Version()`.
///
/// # Safety
///
/// Must be called from the R main thread.
unsafe fn running_r_version() -> Result<(i32, i32), crate::expression::REvalError> {
    use crate::SexpExt;
    use crate::expression::{RCall, dollar_extract};
    use crate::gc_protect::OwnedProtect;
    use crate::sys::R_BaseEnv;

    unsafe {
        // Evaluate R.Version() in base env.
        let version_list = OwnedProtect::new(RCall::new("R.Version").eval(R_BaseEnv)?);

        // Extract $major / $minor — each guarded by an OwnedProtect that
        // releases automatically if a later step short-circuits with `?`.
        let major_sexp = OwnedProtect::new(dollar_extract(version_list.get(), "major")?);
        let minor_sexp = OwnedProtect::new(dollar_extract(version_list.get(), "minor")?);

        // Convert major (character) to integer via as.integer().
        let major_int = RCall::new("as.integer")
            .arg(major_sexp.get())
            .eval(R_BaseEnv)?;
        let major = major_int.as_integer().expect("R.Version()$major is not NA");

        // Parse minor: it's a string like "3.1" — only the part before the dot.
        let minor_stripped = RCall::new("sub")
            .arg(crate::sys::Rf_mkString(c"\\..*".as_ptr()))
            .arg(crate::sys::Rf_mkString(c"".as_ptr()))
            .arg(minor_sexp.get())
            .eval(R_BaseEnv)?;
        let minor_int = RCall::new("as.integer")
            .arg(minor_stripped)
            .eval(R_BaseEnv)?;
        let minor = minor_int.as_integer().expect("R.Version()$minor is not NA");

        Ok((major, minor))
    }
}

// region: ConnectionCapabilities - query connection state

/// Capabilities and state of an R connection.
///
/// Obtained by probing the fields of an R connection struct.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::ConnectionCapabilities;
///
/// unsafe {
///     let caps = ConnectionCapabilities::from_sexp(conn_sexp);
///     if caps.can_read && caps.is_open {
///         // safe to read
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionCapabilities {
    /// Whether the connection supports reading.
    pub can_read: bool,
    /// Whether the connection supports writing.
    pub can_write: bool,
    /// Whether the connection supports seeking.
    pub can_seek: bool,
    /// Whether the connection is in text mode (vs binary).
    pub is_text: bool,
    /// Whether the connection is currently open.
    pub is_open: bool,
    /// Whether the connection is blocking: `None` from
    /// [`from_sexp`](Self::from_sexp) built against R < 4.6, which has no
    /// R-level getter for the flag (#1905).
    pub is_blocking: Option<bool>,
}

impl ConnectionCapabilities {
    /// Probe the capabilities of a connection from its SEXP.
    ///
    /// Built against R >= 4.6, reads the flags from the `Rconn` struct;
    /// before, from `summary()` and `isSeekable()` (#1905).
    ///
    /// # Safety
    ///
    /// - `conn_sexp` must be a valid R connection object
    /// - Must be called from the R main thread
    pub unsafe fn from_sexp(conn_sexp: SEXP) -> Self {
        unsafe {
            let state = access::state(conn_sexp);
            ConnectionCapabilities {
                can_read: state.can_read,
                can_write: state.can_write,
                can_seek: access::can_seek(conn_sexp),
                is_text: state.is_text,
                is_open: state.is_open,
                is_blocking: access::is_blocking(conn_sexp),
            }
        }
    }

    /// Probe the capabilities of a connection from its handle.
    ///
    /// Only built against R >= 4.6, like [`get_connection`], the handle's
    /// source (#1905).
    ///
    /// # Safety
    ///
    /// - `handle` must be a valid `Rconnection` obtained from [`get_connection`]
    #[cfg(r_4_6)]
    pub unsafe fn from_handle(handle: Rconnection) -> Self {
        let conn = handle.cast::<Rconn>().cast_const();
        unsafe {
            ConnectionCapabilities {
                can_read: (*conn).canread != Rboolean::FALSE,
                can_write: (*conn).canwrite != Rboolean::FALSE,
                can_seek: (*conn).canseek != Rboolean::FALSE,
                is_text: (*conn).text != Rboolean::FALSE,
                is_open: (*conn).isopen != Rboolean::FALSE,
                is_blocking: Some((*conn).blocking != Rboolean::FALSE),
            }
        }
    }
}

/// Check if a connection is in binary mode by inspecting its mode string.
///
/// Returns `true` if the mode contains `'b'` (e.g., `"rb"`, `"wb"`, `"r+b"`).
///
/// # Safety
///
/// - `conn_sexp` must be a valid R connection object
/// - Must be called from the R main thread
pub unsafe fn is_binary_mode(conn_sexp: SEXP) -> bool {
    unsafe { access::state(conn_sexp) }.mode.contains('b')
}

/// Get the mode string of a connection.
///
/// Returns the mode string (e.g., `"r"`, `"wb"`, `"r+b"`).
///
/// # Safety
///
/// - `conn_sexp` must be a valid R connection object
/// - Must be called from the R main thread
pub unsafe fn connection_mode(conn_sexp: SEXP) -> String {
    unsafe { access::state(conn_sexp) }.mode
}

/// Get the description string of a connection.
///
/// Returns the description string (shown in `summary(conn)`).
///
/// # Safety
///
/// - `conn_sexp` must be a valid R connection object
/// - Must be called from the R main thread
pub unsafe fn connection_description(conn_sexp: SEXP) -> String {
    unsafe { access::state(conn_sexp) }.description
}
// endregion

// region: Rconn struct - mirrors R's struct Rconn from R_ext/Connections.h

/// Callback type: open connection.
/// Returns TRUE on success, FALSE on failure.
pub type OpenCallback = unsafe extern "C-unwind" fn(*mut Rconn) -> Rboolean;

/// Callback type: close connection (after auto-open).
pub type CloseCallback = unsafe extern "C-unwind" fn(*mut Rconn);

/// Callback type: destroy connection (called when connection is being freed).
pub type DestroyCallback = unsafe extern "C-unwind" fn(*mut Rconn);

/// Callback type: vfprintf (formatted output).
pub type VfprintfCallback =
    unsafe extern "C-unwind" fn(*mut Rconn, *const c_char, *mut c_void) -> c_int;

/// Callback type: fgetc (read single character).
pub type FgetcCallback = unsafe extern "C-unwind" fn(*mut Rconn) -> c_int;

/// Callback type: seek.
pub type SeekCallback = unsafe extern "C-unwind" fn(*mut Rconn, f64, c_int, c_int) -> f64;

/// Callback type: truncate.
pub type TruncateCallback = unsafe extern "C-unwind" fn(*mut Rconn);

/// Callback type: flush.
pub type FlushCallback = unsafe extern "C-unwind" fn(*mut Rconn) -> c_int;

/// Callback type: read (fread-style: buf, size, nitems, conn).
pub type ReadCallback = unsafe extern "C-unwind" fn(*mut c_void, usize, usize, *mut Rconn) -> usize;

/// Callback type: write (fwrite-style: buf, size, nitems, conn).
pub type WriteCallback =
    unsafe extern "C-unwind" fn(*const c_void, usize, usize, *mut Rconn) -> usize;

/// R connection structure (mirrors `struct Rconn` from R_ext/Connections.h).
///
/// # WARNING
///
/// This struct layout must exactly match R's `struct Rconn`. Any mismatch
/// will cause undefined behavior. The layout may change between R versions.
///
/// Only modify the callback pointers and flags through the builder API.
/// Do not modify other fields directly.
#[repr(C)]
#[allow(non_snake_case)] // Match R's C field names exactly
pub struct Rconn {
    /// Connection class name (allocated by R).
    pub class: *mut c_char,
    /// Connection description (allocated by R).
    pub description: *mut c_char,
    /// Encoding of description.
    pub enc: c_int,
    /// Mode string (e.g., "r", "w", "rb").
    pub mode: [c_char; 5],

    // Boolean flags
    /// TRUE if text mode, FALSE if binary.
    pub text: Rboolean,
    /// TRUE if connection is open.
    pub isopen: Rboolean,
    /// TRUE if last read was incomplete.
    pub incomplete: Rboolean,
    /// TRUE if connection supports reading.
    pub canread: Rboolean,
    /// TRUE if connection supports writing.
    pub canwrite: Rboolean,
    /// TRUE if connection supports seeking.
    pub canseek: Rboolean,
    /// TRUE if connection is blocking.
    pub blocking: Rboolean,
    /// TRUE if connection is a gzcon wrapper.
    #[allow(non_snake_case)]
    pub isGzcon: Rboolean,

    // Callbacks
    /// Called to open the connection.
    pub open: Option<OpenCallback>,
    /// Called when connection closes (after auto-open).
    pub close: Option<CloseCallback>,
    /// Called when connection is destroyed (must free private data).
    pub destroy: Option<DestroyCallback>,
    /// Formatted print function.
    pub vfprintf: Option<VfprintfCallback>,
    /// Read single character.
    pub fgetc: Option<FgetcCallback>,
    /// Internal fgetc (usually same as fgetc).
    pub fgetc_internal: Option<FgetcCallback>,
    /// Seek within connection.
    pub seek: Option<SeekCallback>,
    /// Truncate connection.
    pub truncate: Option<TruncateCallback>,
    /// Flush connection.
    pub fflush: Option<FlushCallback>,
    /// Read data (fread-style).
    pub read: Option<ReadCallback>,
    /// Write data (fwrite-style).
    pub write: Option<WriteCallback>,

    // Pushback buffer (for ungetc)
    #[allow(non_snake_case)]
    pub nPushBack: c_int,
    #[allow(non_snake_case)]
    pub posPushBack: c_int,
    #[allow(non_snake_case)]
    pub PushBack: *mut *mut c_char,

    // State
    pub save: c_int,
    pub save2: c_int,

    // Encoding conversion
    pub encname: [c_char; 101],
    pub inconv: *mut c_void,
    pub outconv: *mut c_void,
    pub iconvbuff: [c_char; 25],
    pub oconvbuff: [c_char; 50],
    pub next: *mut c_char,
    pub init_out: [c_char; 25],
    pub navail: i16,
    pub inavail: i16,
    #[allow(non_snake_case)]
    pub EOF_signalled: Rboolean,
    #[allow(non_snake_case)]
    pub UTF8out: Rboolean,

    // Identifiers
    pub id: *mut c_void,
    pub ex_ptr: *mut c_void,

    /// Private data pointer - store your Rust state here.
    ///
    /// This is where you store a pointer to your boxed Rust state.
    /// **IMPORTANT:** You must free this in your `destroy` callback.
    /// R will not free it for you.
    pub private: *mut c_void,

    // Connection status and buffer
    pub status: c_int,
    pub buff: *mut u8,
    pub buff_len: usize,
    pub buff_stored_len: usize,
    pub buff_pos: usize,
}
// endregion

// region: RConnectionImpl trait - user-facing trait for implementing connections

/// Trait for implementing custom R connections.
///
/// Implement this trait on your type to create a custom R connection.
/// The default implementations provide sensible defaults that match R's
/// `null_*` and `dummy_*` callback behaviors.
///
/// # Required Methods
///
/// All methods have default implementations, but you'll typically want to
/// implement at least `open`, `close`, and either `read` or `write`.
///
/// # Errors
///
/// Ordinary I/O callbacks (`read`, `write`, `fgetc`, `seek`, `truncate`,
/// `flush`) run under R unwind protection. Panics and
/// [`crate::error!`] become R errors after Rust destructors run. The std::io
/// adapters preserve failures as `miniextendr_connection_io_error`, including
/// the operation, I/O error kind and optional OS error number.
/// Close and destroy may run during garbage collection, so their panic payloads
/// and deferred conditions are suppressed. A close panic sets status to `-1`:
/// R then warns "Problem closing connection" and returns the negative status.
/// Opening failures return `false` so the builder can tear down.
///
/// # Memory Management
///
/// Your type will be boxed and stored in the connection's `private` field.
/// It will be automatically dropped when the connection is destroyed.
/// You don't need to implement `destroy` unless you have additional cleanup.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::{RConnectionImpl, RCustomConnection};
///
/// struct MemorySource {
///     data: Vec<u8>,
///     pos: usize,
/// }
///
/// impl RConnectionImpl for MemorySource {
///     fn open(&mut self) -> bool {
///         self.pos = 0;
///         true
///     }
///
///     fn read(&mut self, buf: &mut [u8]) -> usize {
///         let remaining = self.data.len() - self.pos;
///         let n = buf.len().min(remaining);
///         buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
///         self.pos += n;
///         n
///     }
/// }
///
/// let conn = RCustomConnection::new()
///     .description("memory source")
///     .mode("rb")
///     .can_read(true)
///     .build(MemorySource { data: vec![1, 2, 3], pos: 0 });
/// ```
pub trait RConnectionImpl: Sized + 'static {
    /// Called when the connection is opened.
    ///
    /// Return `true` for success, `false` for failure.
    /// On failure, R will signal an error.
    fn open(&mut self) -> bool {
        true
    }

    /// Called when the connection is closed.
    ///
    /// This is called when `close()` is called on the R connection
    /// or when it's auto-closed after being opened. Flush buffered output here,
    /// not in `Drop`, whose errors cannot be reported. A panic is contained and
    /// sets a negative close status; R issues its usual close warning after this
    /// callback returns, including when it is called by R's finalizer.
    fn close(&mut self) {}

    /// Called when the connection is being destroyed.
    ///
    /// This is called when the connection object is garbage collected.
    /// The default implementation does nothing - your type will be dropped
    /// automatically after this returns.
    ///
    /// Override this if you need to perform cleanup before your type is dropped.
    fn destroy(&mut self) {}

    /// Read data from the connection.
    ///
    /// Fill `buf` with data and return the number of bytes actually read.
    /// Return 0 on EOF.
    ///
    /// The default returns 0 (EOF).
    fn read(&mut self, _buf: &mut [u8]) -> usize {
        0
    }

    /// Write data to the connection.
    ///
    /// Write the data in `buf` and return the number of bytes actually written.
    ///
    /// The default returns 0 (no bytes written).
    fn write(&mut self, _buf: &[u8]) -> usize {
        0
    }

    /// Read a single character.
    ///
    /// Return the character as an `i32`, or -1 on EOF.
    ///
    /// The default reads one byte via `read()` or returns -1.
    fn fgetc(&mut self) -> i32 {
        let mut buf = [0u8; 1];
        if self.read(&mut buf) == 1 {
            buf[0] as i32
        } else {
            -1
        }
    }

    /// Seek to a position in the connection.
    ///
    /// - `where_`: The position to seek to (or NA to query current position)
    /// - `origin`: 1 = start, 2 = current, 3 = end
    /// - `rw`: 0 = the last used, 1 = read, 2 = write
    ///
    /// Return the previous position. A query returns the current position
    /// without moving. This is R's contract, unlike `std::io::Seek`, which
    /// returns the new position. R hands the value to `seek()`'s caller as it
    /// is, so report a failure with [`crate::error!`] or a panic, not -1.
    ///
    /// The default raises R's own error for a connection without seek
    /// ("'seek' not enabled for this connection").
    fn seek(&mut self, _where: f64, _origin: i32, _rw: i32) -> f64 {
        crate::error!("'seek' not enabled for this connection")
    }

    /// Truncate the connection at the current position.
    ///
    /// The default raises R's own error for a connection without truncation
    /// ("truncation not enabled for this connection").
    fn truncate(&mut self) {
        crate::error!("truncation not enabled for this connection")
    }

    /// Flush buffered output.
    ///
    /// Return 0. R ignores the value, so report a failure with
    /// [`crate::error!`] or a panic.
    ///
    /// The default returns 0 (success/no-op).
    fn flush(&mut self) -> i32 {
        0
    }
}
// endregion

// region: Callback trampolines - bridge between C callbacks and Rust trait

/// Extract the Rust state from a connection's private pointer.
///
/// # Safety
///
/// The `private` field must point to a valid `Box<T>`.
#[inline]
unsafe fn get_state<T: RConnectionImpl>(conn: *mut Rconn) -> &'static mut T {
    let private = unsafe { (*conn).private };
    assert!(!private.is_null(), "Connection private pointer is null");
    unsafe { &mut *private.cast::<T>() }
}

/// Catches panics in connection callbacks, returning `fallback` on panic.
///
/// Wraps the closure in [`catch_unwind`](std::panic::catch_unwind), fires
/// [`PanicSource::Connection`](crate::panic_telemetry::PanicSource::Connection)
/// telemetry on panic, and returns `fallback` instead of unwinding into C.
#[inline]
fn catch_connection_panic<F, R>(fallback: R, f: F) -> R
where
    F: FnOnce() -> R,
{
    crate::ffi_guard::guarded_ffi_call_with_fallback(
        f,
        fallback,
        crate::panic_telemetry::PanicSource::Connection,
    )
}

/// Protect ordinary I/O callbacks against both Rust panics and R longjmps.
/// Raise the original structured condition after Rust locals have unwound;
/// EOF and short writes are successful results, not error substitutes.
fn connection_callback<F: FnOnce() -> R, R>(f: F) -> R {
    crate::ffi_guard::guarded_ffi_call(
        f,
        crate::ffi_guard::GuardMode::RUnwind,
        crate::panic_telemetry::PanicSource::Connection,
    )
}

/// Generate simple I/O trampolines with the shared R-unwind guard.
macro_rules! simple_trampoline {
    ($name:ident, $ret:ty $(, $arg:ident: $arg_ty:ty)* => $method:ident($($call_arg:expr),*)) => {
        unsafe extern "C-unwind" fn $name<T: RConnectionImpl>(
            conn: *mut Rconn,
            $($arg: $arg_ty),*
        ) -> $ret {
            connection_callback(|| {
                let state = unsafe { get_state::<T>(conn) };
                state.$method($($call_arg),*)
            })
        }
    };
}

/// Open callback trampoline.
unsafe extern "C-unwind" fn open_trampoline<T: RConnectionImpl>(conn: *mut Rconn) -> Rboolean {
    catch_connection_panic(Rboolean::FALSE, || {
        let state = unsafe { get_state::<T>(conn) };
        if state.open() {
            unsafe { (*conn).isopen = Rboolean::TRUE };
            Rboolean::TRUE
        } else {
            Rboolean::FALSE
        }
    })
}

/// Close callback trampoline.
unsafe extern "C-unwind" fn close_trampoline<T: RConnectionImpl>(conn: *mut Rconn) {
    // R also calls close from its connection finalizer, without caller handlers.
    let _conditions = crate::deferred_condition::Suppress::new();
    let closed = catch_connection_panic(false, || {
        let state = unsafe { get_state::<T>(conn) };
        state.close();
        true
    });
    unsafe {
        // R owns warning delivery (checkClose), including from its finalizer.
        // A failed flush must not leave the connection marked open.
        (*conn).isopen = Rboolean::FALSE;
        if !closed {
            (*conn).status = -1;
        }
    }
}

/// Destroy callback trampoline - drops the Rust state.
unsafe extern "C-unwind" fn destroy_trampoline<T: RConnectionImpl>(conn: *mut Rconn) {
    crate::externalptr::drop_catching_panic(|| {
        let private = unsafe { (*conn).private };
        if !private.is_null() {
            // Finalization suppresses deferred conditions through destroy AND Drop.
            catch_connection_panic((), || {
                let state = unsafe { &mut *private.cast::<T>() };
                state.destroy();
            });
            unsafe { (*conn).private = std::ptr::null_mut() };
            drop(unsafe { Box::from_raw(private.cast::<T>()) });
        }
    });
}

/// Read callback trampoline.
unsafe extern "C-unwind" fn read_trampoline<T: RConnectionImpl>(
    buf: *mut c_void,
    size: usize,
    nitems: usize,
    conn: *mut Rconn,
) -> usize {
    connection_callback(|| {
        let total_bytes = match size.checked_mul(nitems) {
            Some(n) => n,
            None => return 0, // overflow
        };
        if total_bytes == 0 {
            return 0;
        }
        let state = unsafe { get_state::<T>(conn) };
        let slice = unsafe { std::slice::from_raw_parts_mut(buf.cast::<u8>(), total_bytes) };
        let bytes_read = state.read(slice);
        // Return number of items read
        bytes_read.checked_div(size).unwrap_or(0)
    })
}

/// Write callback trampoline.
unsafe extern "C-unwind" fn write_trampoline<T: RConnectionImpl>(
    buf: *const c_void,
    size: usize,
    nitems: usize,
    conn: *mut Rconn,
) -> usize {
    connection_callback(|| {
        let total_bytes = match size.checked_mul(nitems) {
            Some(n) => n,
            None => return 0, // overflow
        };
        if total_bytes == 0 {
            return 0;
        }
        let state = unsafe { get_state::<T>(conn) };
        let slice = unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), total_bytes) };
        let bytes_written = state.write(slice);
        // Return number of items written
        bytes_written.checked_div(size).unwrap_or(0)
    })
}

// Generate simple trampolines using macro
simple_trampoline!(fgetc_trampoline, c_int => fgetc());
simple_trampoline!(seek_trampoline, f64, where_: f64, origin: c_int, rw: c_int => seek(where_, origin, rw));
simple_trampoline!(truncate_trampoline, () => truncate());
simple_trampoline!(flush_trampoline, c_int => flush());
// endregion

// region: RCustomConnection builder

/// Builder for creating custom R connections.
///
/// # Example
///
/// ```ignore
/// let conn_sexp = RCustomConnection::new()
///     .description("my data source")
///     .mode("rb")
///     .class_name("myconn")
///     .text(false)
///     .can_read(true)
///     .can_write(false)
///     .build(MyConnectionState::new());
/// ```
pub struct RCustomConnection {
    description: CString,
    mode: CString,
    class_name: CString,
    text: Option<bool>,
    can_read: Option<bool>,
    can_write: Option<bool>,
    can_seek: Option<bool>,
    blocking: Option<bool>,
}

impl Default for RCustomConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl RCustomConnection {
    /// Create a new connection builder with defaults.
    ///
    /// Returns a builder with sensible defaults:
    /// - description: `"custom connection"`
    /// - mode: `"r"` (read text)
    /// - class_name: `"customConnection"`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::RCustomConnection;
    ///
    /// let conn = RCustomConnection::new()
    ///     .description("my source")
    ///     .mode("rb")
    ///     .can_read(true)
    ///     .build(MyState::new());
    /// ```
    pub fn new() -> Self {
        Self {
            description: CString::new("custom connection").expect("no null bytes in literal"),
            mode: CString::new("r").expect("no null bytes in literal"),
            class_name: CString::new("customConnection").expect("no null bytes in literal"),
            text: None,
            can_read: None,
            can_write: None,
            can_seek: None,
            blocking: None,
        }
    }

    /// Set the connection description (shown in `summary()`).
    pub fn description(mut self, desc: &str) -> Self {
        self.description = CString::new(desc).expect("description contains null byte");
        self
    }

    /// Set the connection mode.
    ///
    /// Common modes:
    /// - `"r"` - read text
    /// - `"w"` - write text
    /// - `"a"` - append text
    /// - `"rb"` - read binary
    /// - `"wb"` - write binary
    /// - `"ab"` - append binary
    /// - `"r+"` - read/write text
    /// - `"rb+"` or `"r+b"` - read/write binary
    pub fn mode(mut self, mode: &str) -> Self {
        assert!(mode.len() <= 4, "mode must be at most 4 characters");
        self.mode = CString::new(mode).expect("mode contains null byte");
        self
    }

    /// Set the connection class name.
    ///
    /// This becomes part of the connection's class vector (along with "connection").
    pub fn class_name(mut self, name: &str) -> Self {
        self.class_name = CString::new(name).expect("class_name contains null byte");
        self
    }

    /// Set whether this is a text connection (vs binary).
    ///
    /// If not set, R infers from the mode string.
    pub fn text(mut self, is_text: bool) -> Self {
        self.text = Some(is_text);
        self
    }

    /// Set whether the connection supports reading.
    ///
    /// If not set, R infers from the mode string.
    pub fn can_read(mut self, can_read: bool) -> Self {
        self.can_read = Some(can_read);
        self
    }

    /// Set whether the connection supports writing.
    ///
    /// If not set, R infers from the mode string.
    pub fn can_write(mut self, can_write: bool) -> Self {
        self.can_write = Some(can_write);
        self
    }

    /// Set whether the connection supports seeking.
    ///
    /// Default is `false`.
    pub fn can_seek(mut self, can_seek: bool) -> Self {
        self.can_seek = Some(can_seek);
        self
    }

    /// Set whether the connection is blocking.
    ///
    /// Default is `true`. R's underlying `R_new_custom_connection` defaults
    /// this to FALSE, which makes `readLines` push back the final incomplete
    /// line on EOF (silently dropping a trailing line without `\n`). Most
    /// in-memory Rust impls want synchronous blocking behavior, so we
    /// flip the default to match R's stock `file()` connections.
    pub fn blocking(mut self, blocking: bool) -> Self {
        self.blocking = Some(blocking);
        self
    }

    /// Build the connection with the given state.
    ///
    /// Returns an **open** R connection SEXP that can be returned to R.
    /// The state is boxed and stored in the connection's `private` field.
    /// It will be automatically dropped when the connection is destroyed.
    ///
    /// `R_new_custom_connection` returns the connection in a CLOSED state
    /// with `text = TRUE` as defaults. This builder fixes both:
    /// - `text` is inferred from the mode string when not explicitly set
    ///   (`'b'` in the mode → binary; otherwise text). This matches the
    ///   behavior of R's built-in `file()` / `url()` connections.
    /// - The `open` callback is invoked before returning, so the resulting
    ///   SEXP is immediately usable for `readLines` / `writeBin` / `seek`
    ///   without relying on R's per-call auto-open path (which doesn't
    ///   fire for write/seek operations).
    ///
    /// If the `open` callback fails, the boxed state is dropped via the
    /// destroy trampoline and `SEXP::nil()` is returned.
    ///
    /// # Panics
    ///
    /// Panics if `R_CONNECTIONS_VERSION` doesn't match the expected version.
    ///
    /// # Safety
    ///
    /// This function is safe to call, but the returned SEXP must be properly
    /// protected from R's garbage collector.
    pub fn build<T: RConnectionImpl>(self, state: T) -> SEXP {
        // Verify API version
        check_connections_version();

        // Infer text vs binary from the mode string when the caller did not
        // explicitly set `text`. R's `R_new_custom_connection` defaults
        // `text = TRUE` regardless of mode, so a builder like
        // `.mode("r+b")` without `.text(false)` would land as a text
        // connection that rejects `readBin` / `writeBin`.
        let text_default = !self.mode.as_bytes().contains(&b'b');
        let text = self.text.unwrap_or(text_default);

        // Default `blocking = true`. R's `init_con` defaults blocking to
        // FALSE, but for in-memory / synchronous custom connections that
        // is wrong — non-blocking text connections push back the final
        // incomplete line on EOF, so `readLines` silently drops a trailing
        // line without a `\n`. R's stock `file()`/`url()` paths flip this
        // to TRUE later in their own builders; we do the same.
        let blocking = self.blocking.unwrap_or(true);

        unsafe {
            // Box the state
            let boxed_state = Box::new(state);
            let state_ptr = Box::into_raw(boxed_state).cast::<c_void>();

            // Create the R connection object.
            //
            // SAFETY: self.{description,mode,class_name} are owned `CString`
            // field bindings on the builder, alive for the entirety of this
            // call. R `strcpy`s them into its own malloc'd buffers inside
            // `init_con` (r-source `src/main/connections.c` ~line 710) and
            // frees those copies in `con_close1`; we only need the borrow
            // to outlive this single FFI call. Do not refactor the builder
            // to hold `&str` views or to compute these via
            // `CString::new(...).as_ptr()` without a binding — that would
            // free the temporary before R reads it (silent UAF, not caught
            // by basic tests because R has already copied the bytes by then).
            let mut conn_ptr: Rconnection = std::ptr::null_mut();
            let sexp = crate::sys::R_new_custom_connection(
                self.description.as_ptr(),
                self.mode.as_ptr(),
                self.class_name.as_ptr(),
                &mut conn_ptr,
            );
            // open() can defer a condition; signalling may allocate or run GC
            // before this connection is returned to an R stack frame.
            let _connection_root = crate::OwnedProtect::new(sexp);

            if conn_ptr.is_null() {
                // Clean up the boxed state
                let _ = Box::from_raw(state_ptr.cast::<T>());
                // Return R_NilValue on failure (R will have signaled an error)
                return SEXP::nil();
            }

            // Cast to our Rconn struct
            let conn = conn_ptr.cast::<Rconn>();

            // Store state pointer
            (*conn).private = state_ptr;

            // Set callbacks
            (*conn).open = Some(open_trampoline::<T>);
            (*conn).close = Some(close_trampoline::<T>);
            (*conn).destroy = Some(destroy_trampoline::<T>);
            (*conn).read = Some(read_trampoline::<T>);
            (*conn).write = Some(write_trampoline::<T>);
            (*conn).fgetc = Some(fgetc_trampoline::<T>);
            (*conn).fgetc_internal = Some(fgetc_trampoline::<T>);
            (*conn).seek = Some(seek_trampoline::<T>);
            (*conn).truncate = Some(truncate_trampoline::<T>);
            (*conn).fflush = Some(flush_trampoline::<T>);

            // Keep the `dummy_vfprintf` callback installed by
            // `R_new_custom_connection`. R owns the platform-specific
            // `va_list`, grows its formatting buffer as needed, and forwards
            // the complete output through our `write` trampoline.

            // Set text/binary mode (always overrides R's default of TRUE).
            (*conn).text = if text {
                Rboolean::TRUE
            } else {
                Rboolean::FALSE
            };
            if let Some(can_read) = self.can_read {
                (*conn).canread = if can_read {
                    Rboolean::TRUE
                } else {
                    Rboolean::FALSE
                };
            }
            if let Some(can_write) = self.can_write {
                (*conn).canwrite = if can_write {
                    Rboolean::TRUE
                } else {
                    Rboolean::FALSE
                };
            }
            if let Some(can_seek) = self.can_seek {
                (*conn).canseek = if can_seek {
                    Rboolean::TRUE
                } else {
                    Rboolean::FALSE
                };
            }
            (*conn).blocking = if blocking {
                Rboolean::TRUE
            } else {
                Rboolean::FALSE
            };

            // Open the connection. `R_new_custom_connection` returns a
            // connection in CLOSED state (`isopen = FALSE`). R auto-opens
            // on some paths (e.g. `readLines`) but not on `writeBin` /
            // `writeLines` / direct `seek`, which would then bail with
            // "Error writing to connection". Pre-open here so callers get
            // a uniformly usable connection. R's auto-open path sees
            // `isopen == TRUE` and short-circuits — no double-open.
            let opened = open_trampoline::<T>(conn);
            if !matches!(opened, Rboolean::TRUE) {
                // Open failed — tear down via the destroy trampoline,
                // which drops the boxed state and nulls out `private`.
                if let Some(destroy) = (*conn).destroy {
                    destroy(conn);
                }
                return SEXP::nil();
            }

            sexp
        }
    }
}
// endregion

// region: Helper functions for working with connections

/// Read up to `buf.len()` bytes from an R connection; returns the count read.
///
/// Built against R >= 4.6 this is `R_ReadConnection`; before, `readBin()` on
/// a binary connection and `readChar(useBytes = TRUE)` on a text-mode one,
/// which call the same `read` callback (#1905). R 4.4 and 4.5 export the C
/// entry point too, but `R CMD check` notes it there as non-API.
///
/// # Safety
///
/// - `conn` must be an R connection object
/// - Must be called from the R main thread
///
/// # Panics
///
/// R errors (or, before R 4.6, panics) when the connection is not open or
/// cannot read.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::read_connection;
///
/// // Given an R connection SEXP:
/// let mut buf = [0u8; 1024];
/// let n = unsafe { read_connection(conn_sexp, &mut buf) };
/// let data = &buf[..n];
/// ```
#[must_use = "the count of bytes read"]
pub unsafe fn read_connection(conn: SEXP, buf: &mut [u8]) -> usize {
    unsafe { access::read(conn, buf) }
}

/// Write `buf` to an R connection; returns the count written.
///
/// Built against R >= 4.6 this is `R_WriteConnection`, which writes all of
/// `buf`. Before, `writeBin()` on a binary connection and
/// `writeChar(useBytes = TRUE)` on a text-mode one (#1905); a text-mode
/// connection then gets the longest prefix of `buf` that is UTF-8 without a
/// NUL byte, so the count can be short. Loop on it, or write through
/// [`std::io::Write::write_all`].
///
/// # Safety
///
/// - `conn` must be an R connection object
/// - Must be called from the R main thread
///
/// # Panics
///
/// R errors (or, before R 4.6, panics) when the connection is not open or
/// cannot write.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::write_connection;
///
/// let data = b"hello, world\n";
/// let written = unsafe { write_connection(conn_sexp, data) };
/// ```
#[must_use = "the count of bytes written, which can be short before R 4.6"]
pub unsafe fn write_connection(conn: SEXP, buf: &[u8]) -> usize {
    unsafe { access::write(conn, buf) }
}

/// Get the raw connection handle from an R connection SEXP.
///
/// Only built against R >= 4.6 (`r_4_6`), whose `R CMD check` treats
/// `R_GetConnection` as API; R 4.4 and 4.5 note it as non-API and offer no
/// R-level route to the handle (#1905). For reading and writing, use
/// [`read_connection`] / [`write_connection`], which work on every R.
///
/// # Safety
///
/// - `sexp` must be a valid R connection object
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::{Rconn, get_connection};
///
/// let handle = unsafe { get_connection(conn_sexp) };
/// let conn = handle.cast::<Rconn>();
/// ```
#[cfg(r_4_6)]
#[inline]
pub unsafe fn get_connection(sexp: SEXP) -> Rconnection {
    unsafe { crate::sys::R_GetConnection(sexp) }
}
// endregion

pub(crate) mod access;
mod io_adapters;
mod iterator;
pub use io_adapters::*;
pub use iterator::IteratorReader;

// region: standard streams — issue #175

/// R's `stdin()` terminal connection (read-only): R's console input.
///
/// A zero-sized handle, like [`std::io::Stdin`]: every `RStdin` reads the
/// same stream through the same line buffer. It implements
/// [`Read`](std::io::Read). [`lock`](Self::lock) returns an [`RStdinLock`],
/// which implements [`BufRead`](std::io::BufRead), and
/// [`read_line`](Self::read_line) / [`lines`](Self::lines) go through one.
///
/// Input is read the way R reads it: one line at a time, with
/// `readLines(stdin(), n = 1L, warn = FALSE)`. R's terminal `stdin()` has
/// no `read` callback, so `R_ReadConnection` can't read it.
///
/// - Every line comes back ending in `"\n"`. `readLines()` drops the line
///   terminator (`"\n"`, `"\r\n"` or `"\r"`), and `RStdin` adds `"\n"`,
///   also to a last line that had none.
/// - The bytes are the ones `readLines()` returns, in the native encoding.
/// - At the end of the input, a read returns `Ok(0)`.
/// - R's console is not always the process's standard input. Under
///   `Rscript` or `R --file`, `stdin()` reads the script being run, from the
///   line after the code being evaluated. For the process's standard input,
///   use R's `file("stdin")`.
/// - Rust holds at most one line. An R-side read of `stdin()` while part of
///   a line is still unread in Rust gets the next line.
/// - Input is read on R's main thread only. Off it, a read that needs a new
///   line returns an [`io::Error`](std::io::Error).
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::RStdin;
/// use std::io::Read;
///
/// // One line per read, with its "\n".
/// let mut buf = [0u8; 64];
/// let n = RStdin.read(&mut buf)?;
///
/// // Lines without their "\n".
/// for line in RStdin.lines() {
///     let line = line?;
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RStdin;

/// R's `stdout()` terminal connection (write-only).
///
/// A zero-sized type representing R's standard output stream. Use it as a
/// `std::io::Write` sink when you want Rust code to emit text through R's
/// normal output path (obeying `sink()` redirection etc.).
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::RStdout;
/// use std::io::Write;
///
/// let mut out = RStdout;
/// writeln!(out, "hello from Rust").unwrap();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RStdout;

/// R's `stderr()` terminal connection (write-only).
///
/// A zero-sized type representing R's standard error stream. Writes go
/// through R's message channel, which respects `sink(type="message")`
/// redirection.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::RStderr;
/// use std::io::Write;
///
/// let mut err = RStderr;
/// writeln!(err, "diagnostic info").unwrap();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RStderr;

/// The [`RStdin`] line buffer, locked: [`BufRead`](std::io::BufRead) access
/// to R's console input.
///
/// Returned by [`RStdin::lock`]. It holds the line buffer every `RStdin` on
/// this thread shares, and gives it back when dropped. `fill_buf()` returns
/// the unread rest of the current line and reads the next one when that is
/// used up; it is empty at the end of the input. It also implements
/// [`Read`](std::io::Read), on top of `fill_buf()` and `consume()`.
#[derive(Debug)]
pub struct RStdinLock {
    line: StdinLine,
    /// `!Send`: the buffer belongs to the thread that locked it.
    _not_send: std::marker::PhantomData<*const ()>,
}

/// The unread rest of the last line [`RStdin`] read.
#[derive(Debug, Default)]
struct StdinLine {
    bytes: Vec<u8>,
    pos: usize,
}

impl StdinLine {
    const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            pos: 0,
        }
    }

    /// The unread rest of the line. Once it is used up, `read_line` appends
    /// the next line to the emptied buffer (nothing at the end of the input).
    fn fill_with(
        &mut self,
        read_line: impl FnOnce(&mut Vec<u8>) -> std::io::Result<()>,
    ) -> std::io::Result<&[u8]> {
        if self.pos == self.bytes.len() {
            self.bytes.clear();
            self.pos = 0;
            read_line(&mut self.bytes)?;
        }
        Ok(&self.bytes[self.pos..])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_add(amt).min(self.bytes.len());
    }
}

thread_local! {
    /// The line buffer every `RStdin` on this thread reads through. `None`
    /// while an `RStdinLock` holds it.
    static STDIN_LINE: std::cell::Cell<Option<StdinLine>> =
        const { std::cell::Cell::new(Some(StdinLine::new())) };
}

/// Append the next line of R's console input to `line`, with `"\n"` for its
/// terminator, through `readLines(stdin(), n = 1L, warn = FALSE)`. Appends
/// nothing at the end of the input (`character(0)`).
fn read_console_line(line: &mut Vec<u8>) -> std::io::Result<()> {
    use crate::SexpExt;
    use crate::expression::RCall;

    if !crate::worker::is_r_main_thread() {
        return Err(std::io::Error::other(
            "RStdin can only be read on R's main thread",
        ));
    }
    // SAFETY: R's main thread (checked above). The `stdin()` call is an
    // argument of the `readLines()` call, which evaluates it. `eval_base`
    // returns an R error as `Err`.
    let lines = unsafe {
        RCall::from_cstr(c"readLines")
            .arg(RCall::from_cstr(c"stdin").build())
            .named_arg("n", SEXP::scalar_integer(1))
            .named_arg("warn", SEXP::scalar_logical(false))
            .eval_base()
    }
    .map_err(|e| std::io::Error::other(format!("reading R's stdin() failed: {e}")))?;
    if !lines.is_empty() {
        // `lines` is unprotected: copy the bytes out before anything allocates.
        let text = unsafe { CStr::from_ptr(lines.string_elt(0).r_char()) };
        line.extend_from_slice(text.to_bytes());
        line.push(b'\n');
    }
    Ok(())
}

impl RStdin {
    /// Lock the line buffer every `RStdin` on this thread shares, for
    /// [`BufRead`](std::io::BufRead) access. Like [`std::io::Stdin::lock`].
    ///
    /// Reading through the lock off R's main thread returns an
    /// [`io::Error`](std::io::Error) as soon as it needs a new line.
    ///
    /// # Panics
    ///
    /// If an [`RStdinLock`] is already alive on this thread ("RStdin is
    /// already locked on this thread"). That includes reading through
    /// another `RStdin` handle while a lock is held.
    pub fn lock(&self) -> RStdinLock {
        let line = STDIN_LINE
            .with(std::cell::Cell::take)
            .expect("RStdin is already locked on this thread");
        RStdinLock {
            line,
            _not_send: std::marker::PhantomData,
        }
    }

    /// Read the next line, with its `"\n"`, into `buf`; see
    /// [`BufRead::read_line`](std::io::BufRead::read_line). Returns `Ok(0)`
    /// at the end of the input.
    ///
    /// # Panics
    ///
    /// Like [`lock`](Self::lock).
    pub fn read_line(&self, buf: &mut String) -> std::io::Result<usize> {
        std::io::BufRead::read_line(&mut self.lock(), buf)
    }

    /// The lines that are left, without their `"\n"`, holding the lock
    /// until the iterator is dropped; see
    /// [`BufRead::lines`](std::io::BufRead::lines).
    ///
    /// # Panics
    ///
    /// Like [`lock`](Self::lock).
    pub fn lines(self) -> std::io::Lines<RStdinLock> {
        std::io::BufRead::lines(self.lock())
    }
}

impl Drop for RStdinLock {
    fn drop(&mut self) {
        let line = std::mem::take(&mut self.line);
        // During thread teardown the buffer goes away with the thread.
        let _ = STDIN_LINE.try_with(|slot| slot.set(Some(line)));
    }
}

impl std::io::BufRead for RStdinLock {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.line.fill_with(read_console_line)
    }

    fn consume(&mut self, amt: usize) {
        self.line.consume(amt);
    }
}

impl std::io::Read for RStdinLock {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::BufRead;

        if buf.is_empty() {
            return Ok(0);
        }
        let available = self.fill_buf()?;
        let n = available.len().min(buf.len());
        buf[..n].copy_from_slice(&available[..n]);
        self.consume(n);
        Ok(n)
    }
}

impl std::io::Read for RStdin {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        std::io::Read::read(&mut self.lock(), buf)
    }
}

// R's terminal connections route writes through the console hook
// (ptr_R_WriteConsoleEx / Rprintf / REprintf), not through
// R_WriteConnection.
//
// otype semantics (matches R_WriteConsoleEx convention):
//   0 = stdout (regular output)
//   1 = stderr (message/diagnostic output)
//
// Calls from a non-main thread are silently dropped — writing to the R
// console from a worker thread is unsound (R API is not thread-safe).
//
// This is `pub(crate)` so [`crate::progress`] can route its terminal-connection
// branch through the same console hook (the `indicatif` integration dispatches
// on `inherits("terminal")` and uses this for terminal targets).
pub(crate) fn write_console_to(buf: &[u8], otype: std::os::raw::c_int) -> usize {
    if buf.is_empty() {
        return 0;
    }
    if !crate::worker::is_r_main_thread() {
        return 0;
    }
    // Construct a temporary null-terminated copy for %s printing.
    // For write() we can safely truncate at the first embedded NUL, which
    // matches what R's own writeLines() does (it errors on embedded NUL).
    let truncated = match buf.iter().position(|&b| b == 0) {
        Some(pos) => &buf[..pos],
        None => buf,
    };
    if truncated.is_empty() {
        return buf.len(); // all NUL bytes — nothing to print
    }
    // Build a null-terminated temporary string.
    let mut tmp = Vec::with_capacity(truncated.len() + 1);
    tmp.extend_from_slice(truncated);
    tmp.push(0u8);
    unsafe {
        let ptr = tmp.as_ptr().cast::<std::os::raw::c_char>();
        if otype == 0 {
            crate::sys::Rprintf(c"%s".as_ptr(), ptr);
        } else {
            crate::sys::REprintf(c"%s".as_ptr(), ptr);
        }
    }
    buf.len()
}

impl std::io::Write for RStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(write_console_to(buf, 0))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Write for RStderr {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(write_console_to(buf, 1))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// endregion

// region: null connection — issue #176

/// A write-only connection to the platform null device
/// (`/dev/null` on Unix, `NUL` on Windows).
///
/// Writes are discarded but still routed through R's connection machinery,
/// so active `sink()` redirections see the writes. A write R refuses (the
/// connection was closed, or its write callback raised) returns R's message
/// as an `io::Error`.
///
/// # Ownership
///
/// A handle closes only the connection it opened, like std's `OwnedFd`
/// versus `BorrowedFd`:
///
/// - **Owned**: [`RNullConnection::new`] (and `Default`) opens the null
///   device. Dropping the handle closes that connection.
/// - **Borrowed**: a `#[miniextendr]` parameter of this type accepts any
///   open, write-capable R connection (`file("out.txt", "w")`, a
///   `textConnection`, ...). The R caller owns that connection. Dropping the
///   handle leaves it open, so the caller can keep writing to it and closes
///   it when they're done.
///
/// Either way the handle keeps the connection object on R's precious list
/// while it lives, and dropping it releases that pin.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::RNullConnection;
/// use std::io::Write;
///
/// let mut null = RNullConnection::new();
/// writeln!(null, "discarded").unwrap();
/// // closes automatically when dropped: this handle opened it
/// ```
///
/// ```ignore
/// #[miniextendr]
/// pub fn log_to(con: RNullConnection, text: &str) {
///     let mut con = con;
///     writeln!(con, "{text}").unwrap();
///     // `con` is borrowed from the R caller: it stays open after this returns
/// }
/// ```
pub struct RNullConnection {
    /// The R connection object: `file(nullfile(), open = "w")` for an owned
    /// handle, the caller's connection for a borrowed one.
    ///
    /// Stored on the precious list so R's GC cannot collect it while Rust
    /// holds the struct. `Drop` (or `IntoR`) releases it exactly once.
    sexp: SEXP,
    /// Whether this handle may still write (i.e., `close()` hasn't been
    /// called yet). Guards the `Drop` impl against double-close.
    open: bool,
    /// Whether this handle opened the connection ([`RNullConnection::new`])
    /// and so closes it. `false` for a connection borrowed from R.
    owned: bool,
}

impl std::fmt::Debug for RNullConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RNullConnection")
            .field("open", &self.open)
            .field("owned", &self.owned)
            .finish_non_exhaustive()
    }
}

impl RNullConnection {
    /// Open a connection to the null device.
    ///
    /// Evaluates `file(nullfile(), open = "w")` in the base environment.
    ///
    /// # Panics
    ///
    /// Panics if R cannot open the null device (highly unlikely).
    ///
    /// # Safety (caller)
    ///
    /// Must be called from the R main thread.
    pub fn new() -> Self {
        unsafe {
            use crate::expression::RCall;
            use crate::gc_protect::OwnedProtect;
            use crate::sys::R_PreserveObject;

            // nullfile() — returns the platform null device path. Protect it
            // while we build the file() call that references it as an arg.
            let null_path = OwnedProtect::new(
                RCall::new("nullfile")
                    .eval_base()
                    .unwrap_or_else(|_| panic!("nullfile() failed")),
            );

            // file(null_path, open = "w")
            let sexp = RCall::new("file")
                .arg(null_path.get())
                .named_arg("open", crate::sys::Rf_mkString(c"w".as_ptr()))
                .eval_base()
                .unwrap_or_else(|_| panic!("file(nullfile(), open=\"w\") failed"));

            // Pin on the precious list so GC cannot collect it while we hold it.
            R_PreserveObject(sexp);

            RNullConnection {
                sexp,
                open: true,
                owned: true,
            }
        }
    }

    /// The underlying R connection SEXP.
    #[inline]
    pub fn sexp(&self) -> SEXP {
        self.sexp
    }

    /// Borrow a connection the R caller owns, from a SEXP that has
    /// **already been added to R's precious list**.
    ///
    /// Used by `TryFromSexp for RNullConnection` which calls
    /// `R_PreserveObject` before handing the SEXP to this constructor.
    /// The handle never closes the R connection: `Drop` only calls
    /// `R_ReleaseObject`, exactly once.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid, open R connection SEXP.
    /// - Caller must have called `R_PreserveObject(sexp)` before this call.
    pub(crate) unsafe fn from_preserved_sexp(sexp: SEXP) -> Self {
        RNullConnection {
            sexp,
            open: true,
            owned: false,
        }
    }

    /// Stop writing through this handle.
    ///
    /// After calling this, `write` returns an error. A second close is a
    /// no-op, and dropping the handle afterwards closes nothing more.
    ///
    /// - On an **owned** handle ([`RNullConnection::new`]) this closes the
    ///   null-device connection now instead of at `Drop`.
    /// - On a **borrowed** handle (a connection passed in from R) the R
    ///   connection stays open: the R caller owns it and closes it. Only this
    ///   handle stops writing.
    pub fn close(&mut self) {
        if self.open {
            self.open = false;
            if self.owned {
                unsafe { close_r_connection(self.sexp) };
            }
        }
    }
}

// Evaluate R's `close(con)` on a connection an owned handle opened. An R error
// is ignored: there is nothing the caller could do about a failed close.
//
// # Safety
// Must be called from the R main thread, with `sexp` rooted.
unsafe fn close_r_connection(sexp: SEXP) {
    use crate::expression::RCall;
    let _ = unsafe { RCall::new("close").arg(sexp).eval_base() };
}

impl Default for RNullConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl std::io::Write for RNullConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.open {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "RNullConnection is closed",
            ));
        }
        // A borrowed connection's write can fail: return R's error rather
        // than jump over the caller's frames.
        unsafe { access::write_caught(self.sexp, buf) }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for RNullConnection {
    fn drop(&mut self) {
        // Close only a connection this handle opened and hasn't closed yet; a
        // borrowed one belongs to the R caller. Release the pin either way.
        let close = self.open && self.owned;
        self.open = false;
        let sexp = self.sexp;
        crate::externalptr::drop_catching_panic(|| unsafe {
            if close {
                close_r_connection(sexp);
            }
            crate::sys::R_ReleaseObject(sexp);
        });
    }
}

// endregion

#[cfg(test)]
mod tests;
