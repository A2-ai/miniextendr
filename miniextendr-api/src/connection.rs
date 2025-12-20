//! Custom R connection framework.
//!
//! This module provides a safe Rust interface for creating custom R connections.
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
//! Always check [`ffi::R_CONNECTIONS_VERSION`](crate::ffi::R_CONNECTIONS_VERSION) at runtime
//! before using these APIs.
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

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

use crate::ffi::{Rboolean, Rconnection, SEXP, R_CONNECTIONS_VERSION, R_NilValue};

/// The expected R connections API version this module is compatible with.
///
/// This should match `R_CONNECTIONS_VERSION` from R. If they don't match,
/// connection operations may behave incorrectly or crash.
pub const EXPECTED_CONNECTIONS_VERSION: c_int = 1;

/// Check that the R connections API version matches what we expect.
///
/// # Panics
///
/// Panics if `R_CONNECTIONS_VERSION` doesn't match `EXPECTED_CONNECTIONS_VERSION`.
#[inline]
pub fn check_connections_version() {
    assert_eq!(
        R_CONNECTIONS_VERSION, EXPECTED_CONNECTIONS_VERSION,
        "R_CONNECTIONS_VERSION mismatch: expected {}, got {}. \
         The R connections API may have changed incompatibly.",
        EXPECTED_CONNECTIONS_VERSION, R_CONNECTIONS_VERSION
    );
}

// =============================================================================
// Rconn struct - mirrors R's struct Rconn from R_ext/Connections.h
// =============================================================================

/// Callback type: open connection.
/// Returns TRUE on success, FALSE on failure.
pub type OpenCallback = unsafe extern "C-unwind" fn(*mut Rconn) -> Rboolean;

/// Callback type: close connection (after auto-open).
pub type CloseCallback = unsafe extern "C-unwind" fn(*mut Rconn);

/// Callback type: destroy connection (called when connection is being freed).
pub type DestroyCallback = unsafe extern "C-unwind" fn(*mut Rconn);

/// Callback type: vfprintf (formatted output).
pub type VfprintfCallback = unsafe extern "C-unwind" fn(*mut Rconn, *const c_char, *mut c_void) -> c_int;

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
pub type WriteCallback = unsafe extern "C-unwind" fn(*const c_void, usize, usize, *mut Rconn) -> usize;

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

// =============================================================================
// RConnectionImpl trait - user-facing trait for implementing connections
// =============================================================================

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
/// # Memory Management
///
/// Your type will be boxed and stored in the connection's `private` field.
/// It will be automatically dropped when the connection is destroyed.
/// You don't need to implement `destroy` unless you have additional cleanup.
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
    /// or when it's auto-closed after being opened.
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
    /// - `rw`: 1 = read, 2 = write
    ///
    /// Return the new position, or -1 on failure, or current position if `where_` is NA.
    ///
    /// The default returns -1 (seek not supported).
    fn seek(&mut self, _where: f64, _origin: i32, _rw: i32) -> f64 {
        -1.0
    }

    /// Truncate the connection at the current position.
    ///
    /// The default does nothing.
    fn truncate(&mut self) {}

    /// Flush buffered output.
    ///
    /// Return 0 on success, non-zero on failure.
    ///
    /// The default returns 0 (success/no-op).
    fn flush(&mut self) -> i32 {
        0
    }

    /// Formatted print (vfprintf-style).
    ///
    /// This is rarely needed - R typically uses `write` for output.
    /// Return the number of characters written, or -1 on error.
    ///
    /// The default returns -1 (not implemented).
    fn vfprintf(&mut self, _fmt: *const c_char, _ap: *mut c_void) -> i32 {
        -1
    }
}

// =============================================================================
// Callback trampolines - bridge between C callbacks and Rust trait
// =============================================================================

/// Extract the Rust state from a connection's private pointer.
///
/// # Safety
///
/// The `private` field must point to a valid `Box<T>`.
#[inline]
unsafe fn get_state<T: RConnectionImpl>(conn: *mut Rconn) -> &'static mut T {
    let private = unsafe { (*conn).private };
    debug_assert!(!private.is_null(), "Connection private pointer is null");
    unsafe { &mut *(private as *mut T) }
}

/// Open callback trampoline.
unsafe extern "C-unwind" fn open_trampoline<T: RConnectionImpl>(conn: *mut Rconn) -> Rboolean {
    let state = unsafe { get_state::<T>(conn) };
    if state.open() {
        unsafe { (*conn).isopen = Rboolean::TRUE };
        Rboolean::TRUE
    } else {
        Rboolean::FALSE
    }
}

/// Close callback trampoline.
unsafe extern "C-unwind" fn close_trampoline<T: RConnectionImpl>(conn: *mut Rconn) {
    let state = unsafe { get_state::<T>(conn) };
    state.close();
    unsafe { (*conn).isopen = Rboolean::FALSE };
}

/// Destroy callback trampoline - drops the Rust state.
unsafe extern "C-unwind" fn destroy_trampoline<T: RConnectionImpl>(conn: *mut Rconn) {
    let private = unsafe { (*conn).private };
    if !private.is_null() {
        // Give the implementation a chance to do cleanup
        let state = unsafe { &mut *(private as *mut T) };
        state.destroy();

        // Now drop the boxed state
        let _ = unsafe { Box::from_raw(private as *mut T) };
        unsafe { (*conn).private = std::ptr::null_mut() };
    }
}

/// Read callback trampoline.
unsafe extern "C-unwind" fn read_trampoline<T: RConnectionImpl>(
    buf: *mut c_void,
    size: usize,
    nitems: usize,
    conn: *mut Rconn,
) -> usize {
    let state = unsafe { get_state::<T>(conn) };
    let total_bytes = size * nitems;
    if total_bytes == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, total_bytes) };
    let bytes_read = state.read(slice);
    // Return number of items read
    if size > 0 {
        bytes_read / size
    } else {
        0
    }
}

/// Write callback trampoline.
unsafe extern "C-unwind" fn write_trampoline<T: RConnectionImpl>(
    buf: *const c_void,
    size: usize,
    nitems: usize,
    conn: *mut Rconn,
) -> usize {
    let state = unsafe { get_state::<T>(conn) };
    let total_bytes = size * nitems;
    if total_bytes == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(buf as *const u8, total_bytes) };
    let bytes_written = state.write(slice);
    // Return number of items written
    if size > 0 {
        bytes_written / size
    } else {
        0
    }
}

/// Fgetc callback trampoline.
unsafe extern "C-unwind" fn fgetc_trampoline<T: RConnectionImpl>(conn: *mut Rconn) -> c_int {
    let state = unsafe { get_state::<T>(conn) };
    state.fgetc()
}

/// Seek callback trampoline.
unsafe extern "C-unwind" fn seek_trampoline<T: RConnectionImpl>(
    conn: *mut Rconn,
    where_: f64,
    origin: c_int,
    rw: c_int,
) -> f64 {
    let state = unsafe { get_state::<T>(conn) };
    state.seek(where_, origin, rw)
}

/// Truncate callback trampoline.
unsafe extern "C-unwind" fn truncate_trampoline<T: RConnectionImpl>(conn: *mut Rconn) {
    let state = unsafe { get_state::<T>(conn) };
    state.truncate();
}

/// Flush callback trampoline.
unsafe extern "C-unwind" fn flush_trampoline<T: RConnectionImpl>(conn: *mut Rconn) -> c_int {
    let state = unsafe { get_state::<T>(conn) };
    state.flush()
}

/// Vfprintf callback trampoline.
unsafe extern "C-unwind" fn vfprintf_trampoline<T: RConnectionImpl>(
    conn: *mut Rconn,
    fmt: *const c_char,
    ap: *mut c_void,
) -> c_int {
    let state = unsafe { get_state::<T>(conn) };
    state.vfprintf(fmt, ap)
}

// =============================================================================
// RCustomConnection builder
// =============================================================================

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
    pub fn new() -> Self {
        Self {
            description: CString::new("custom connection").unwrap(),
            mode: CString::new("r").unwrap(),
            class_name: CString::new("customConnection").unwrap(),
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
    /// Default is `true`.
    pub fn blocking(mut self, blocking: bool) -> Self {
        self.blocking = Some(blocking);
        self
    }

    /// Build the connection with the given state.
    ///
    /// Returns an R connection SEXP that can be returned to R.
    /// The state is boxed and stored in the connection's `private` field.
    /// It will be automatically dropped when the connection is destroyed.
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

        unsafe {
            // Box the state
            let boxed_state = Box::new(state);
            let state_ptr = Box::into_raw(boxed_state) as *mut c_void;

            // Create the R connection object
            let mut conn_ptr: Rconnection = std::ptr::null_mut();
            let sexp = crate::ffi::R_new_custom_connection(
                self.description.as_ptr(),
                self.mode.as_ptr(),
                self.class_name.as_ptr(),
                &mut conn_ptr,
            );

            if conn_ptr.is_null() {
                // Clean up the boxed state
                let _ = Box::from_raw(state_ptr as *mut T);
                // Return R_NilValue on failure (R will have signaled an error)
                return R_NilValue;
            }

            // Cast to our Rconn struct
            let conn = conn_ptr as *mut Rconn;

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
            (*conn).vfprintf = Some(vfprintf_trampoline::<T>);

            // Set optional flags
            if let Some(text) = self.text {
                (*conn).text = if text { Rboolean::TRUE } else { Rboolean::FALSE };
            }
            if let Some(can_read) = self.can_read {
                (*conn).canread = if can_read { Rboolean::TRUE } else { Rboolean::FALSE };
            }
            if let Some(can_write) = self.can_write {
                (*conn).canwrite = if can_write { Rboolean::TRUE } else { Rboolean::FALSE };
            }
            if let Some(can_seek) = self.can_seek {
                (*conn).canseek = if can_seek { Rboolean::TRUE } else { Rboolean::FALSE };
            }
            if let Some(blocking) = self.blocking {
                (*conn).blocking = if blocking { Rboolean::TRUE } else { Rboolean::FALSE };
            }

            sexp
        }
    }
}

// =============================================================================
// Helper functions for working with connections
// =============================================================================

/// Read data from an R connection.
///
/// # Safety
///
/// - `conn` must be a valid, open connection handle
/// - `buf` must be a valid buffer with at least `n` bytes
#[inline]
pub unsafe fn read_connection(conn: Rconnection, buf: &mut [u8]) -> usize {
    unsafe {
        crate::ffi::R_ReadConnection(conn, buf.as_mut_ptr() as *mut c_void, buf.len())
    }
}

/// Write data to an R connection.
///
/// # Safety
///
/// - `conn` must be a valid, open connection handle
#[inline]
pub unsafe fn write_connection(conn: Rconnection, buf: &[u8]) -> usize {
    unsafe {
        crate::ffi::R_WriteConnection(conn, buf.as_ptr() as *const c_void, buf.len())
    }
}

/// Get a connection handle from an R connection SEXP.
///
/// # Safety
///
/// - `sexp` must be a valid R connection object
#[inline]
pub unsafe fn get_connection(sexp: SEXP) -> Rconnection {
    unsafe { crate::ffi::R_GetConnection(sexp) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connections_version() {
        // This just verifies the constant is defined correctly
        assert_eq!(R_CONNECTIONS_VERSION, 1);
        assert_eq!(EXPECTED_CONNECTIONS_VERSION, 1);
    }

    #[test]
    fn test_rconn_struct_size() {
        // Sanity check that the struct is reasonably sized
        // The actual size depends on platform alignment
        let size = std::mem::size_of::<Rconn>();
        // Should be at least several hundred bytes
        assert!(size > 200, "Rconn struct seems too small: {} bytes", size);
        // Should be less than 2KB
        assert!(size < 2048, "Rconn struct seems too large: {} bytes", size);
    }
}
