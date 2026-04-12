//! NNG socket wrapper.

use super::error::{NngResult, check};
use super::ffi;
use super::msg::NngMsg;
use core::ffi::c_void;
use std::ffi::CString;

/// RAII wrapper for an NNG socket.
///
/// `Drop` calls `nng_close()`. Not `Clone` — each socket is uniquely owned.
/// `Send` because NNG sockets are thread-safe.
pub struct NngSocket {
    inner: ffi::nng_socket,
}

// NNG sockets are thread-safe
unsafe impl Send for NngSocket {}

impl NngSocket {
    /// Internal: wrap a raw nng_socket after successful open.
    fn from_raw(inner: ffi::nng_socket) -> Self {
        Self { inner }
    }

    /// Open helper: calls an NNG protocol open function.
    fn open(f: unsafe extern "C" fn(*mut ffi::nng_socket) -> i32) -> NngResult<Self> {
        let mut sock = ffi::nng_socket { id: 0 };
        check(unsafe { f(&mut sock) })?;
        Ok(Self::from_raw(sock))
    }

    // ─── Protocol constructors ───

    /// Open a REQ (request) socket.
    pub fn req() -> NngResult<Self> {
        Self::open(ffi::nng_req0_open)
    }

    /// Open a REP (reply) socket.
    pub fn rep() -> NngResult<Self> {
        Self::open(ffi::nng_rep0_open)
    }

    /// Open a PAIR v0 socket (1:1 bidirectional).
    pub fn pair() -> NngResult<Self> {
        Self::open(ffi::nng_pair0_open)
    }

    /// Open a PAIR v1 socket (supports polyamorous mode).
    pub fn pair1() -> NngResult<Self> {
        Self::open(ffi::nng_pair1_open)
    }

    /// Open a PUB (publish) socket.
    pub fn pub_() -> NngResult<Self> {
        Self::open(ffi::nng_pub0_open)
    }

    /// Open a SUB (subscribe) socket.
    pub fn sub() -> NngResult<Self> {
        Self::open(ffi::nng_sub0_open)
    }

    /// Open a PUSH socket (pipeline, send-only).
    pub fn push() -> NngResult<Self> {
        Self::open(ffi::nng_push0_open)
    }

    /// Open a PULL socket (pipeline, recv-only).
    pub fn pull() -> NngResult<Self> {
        Self::open(ffi::nng_pull0_open)
    }

    /// Open a BUS socket (mesh, many-to-many).
    pub fn bus() -> NngResult<Self> {
        Self::open(ffi::nng_bus0_open)
    }

    // ─── Connection ───

    /// Dial a remote URL. Blocks until connected (or fails).
    pub fn dial(&self, url: &str) -> NngResult<()> {
        let c_url = CString::new(url).expect("NNG URL must not contain null bytes");
        check(unsafe { ffi::nng_dial(self.inner, c_url.as_ptr(), std::ptr::null_mut(), 0) })
    }

    /// Listen on a URL. Binds and starts accepting connections.
    pub fn listen(&self, url: &str) -> NngResult<()> {
        let c_url = CString::new(url).expect("NNG URL must not contain null bytes");
        check(unsafe { ffi::nng_listen(self.inner, c_url.as_ptr(), std::ptr::null_mut(), 0) })
    }

    // ─── Send / Receive (byte slices) ───

    /// Send bytes synchronously. Blocks until sent.
    ///
    /// NNG copies the data internally, so the caller retains ownership.
    pub fn send(&self, data: &[u8]) -> NngResult<()> {
        check(unsafe { ffi::nng_send(self.inner, data.as_ptr() as *mut c_void, data.len(), 0) })
    }

    /// Receive bytes synchronously. Blocks until a message arrives.
    ///
    /// Returns the received data as an owned `Vec<u8>`.
    pub fn recv(&self) -> NngResult<Vec<u8>> {
        let mut msg_ptr: *mut ffi::nng_msg = std::ptr::null_mut();
        check(unsafe { ffi::nng_recvmsg(self.inner, &mut msg_ptr, 0) })?;
        let msg = unsafe { NngMsg::from_raw(msg_ptr) };
        Ok(msg.body().to_vec())
    }

    // ─── Send / Receive (messages) ───

    /// Send an NNG message. Consumes the message (NNG takes ownership).
    pub fn send_msg(&self, msg: NngMsg) -> NngResult<()> {
        check(unsafe { ffi::nng_sendmsg(self.inner, msg.into_raw(), 0) })
    }

    /// Receive an NNG message. Blocks until a message arrives.
    pub fn recv_msg(&self) -> NngResult<NngMsg> {
        let mut msg_ptr: *mut ffi::nng_msg = std::ptr::null_mut();
        check(unsafe { ffi::nng_recvmsg(self.inner, &mut msg_ptr, 0) })?;
        Ok(unsafe { NngMsg::from_raw(msg_ptr) })
    }

    // ─── Options ───

    /// Set receive timeout in milliseconds. -1 = infinite (default).
    pub fn set_recv_timeout(&self, ms: i32) -> NngResult<()> {
        let opt = c"recv-timeout";
        check(unsafe { ffi::nng_socket_set_ms(self.inner, opt.as_ptr(), ms) })
    }

    /// Set send timeout in milliseconds. -1 = infinite (default).
    pub fn set_send_timeout(&self, ms: i32) -> NngResult<()> {
        let opt = c"send-timeout";
        check(unsafe { ffi::nng_socket_set_ms(self.inner, opt.as_ptr(), ms) })
    }
}

impl Drop for NngSocket {
    fn drop(&mut self) {
        let _ = unsafe { ffi::nng_close(self.inner) };
    }
}
