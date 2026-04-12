//! Raw FFI declarations for the NNG C library.
//!
//! NNG is compiled by R's Makevars and linked as a static archive.
//! These are just `extern "C"` declarations that the linker resolves.

use core::ffi::{c_char, c_int, c_void};

/// NNG socket handle (opaque ID).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nng_socket {
    pub id: u32,
}

/// NNG dialer handle.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nng_dialer {
    pub id: u32,
}

/// NNG listener handle.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nng_listener {
    pub id: u32,
}

/// NNG context handle (for multiplexing on REQ/REP sockets).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nng_ctx {
    pub id: u32,
}

/// Opaque NNG message type.
#[repr(C)]
pub struct nng_msg {
    _opaque: [u8; 0],
}

// NNG flags
pub const NNG_FLAG_NONBLOCK: c_int = 1;

unsafe extern "C" {
    // Version
    pub fn nng_version() -> *const c_char;

    // Error
    pub fn nng_strerror(err: c_int) -> *const c_char;

    // Socket protocol openers
    pub fn nng_req0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_rep0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_pair0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_pair1_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_pub0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_sub0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_push0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_pull0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_bus0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_surveyor0_open(socket: *mut nng_socket) -> c_int;
    pub fn nng_respondent0_open(socket: *mut nng_socket) -> c_int;

    // Socket lifecycle
    pub fn nng_close(socket: nng_socket) -> c_int;

    // Dialer / Listener
    pub fn nng_dial(
        socket: nng_socket,
        url: *const c_char,
        dialer: *mut nng_dialer,
        flags: c_int,
    ) -> c_int;
    pub fn nng_listen(
        socket: nng_socket,
        url: *const c_char,
        listener: *mut nng_listener,
        flags: c_int,
    ) -> c_int;

    // Synchronous send/recv (raw bytes)
    pub fn nng_send(socket: nng_socket, data: *mut c_void, size: usize, flags: c_int) -> c_int;
    pub fn nng_recv(socket: nng_socket, data: *mut c_void, size: *mut usize, flags: c_int)
    -> c_int;

    // Message-based send/recv
    pub fn nng_sendmsg(socket: nng_socket, msg: *mut nng_msg, flags: c_int) -> c_int;
    pub fn nng_recvmsg(socket: nng_socket, msg: *mut *mut nng_msg, flags: c_int) -> c_int;

    // Message allocation and manipulation
    pub fn nng_msg_alloc(msg: *mut *mut nng_msg, size: usize) -> c_int;
    pub fn nng_msg_free(msg: *mut nng_msg);
    pub fn nng_msg_body(msg: *mut nng_msg) -> *mut c_void;
    pub fn nng_msg_len(msg: *const nng_msg) -> usize;
    pub fn nng_msg_clear(msg: *mut nng_msg);
    pub fn nng_msg_append(msg: *mut nng_msg, data: *const c_void, size: usize) -> c_int;
    pub fn nng_msg_header(msg: *mut nng_msg) -> *mut c_void;
    pub fn nng_msg_header_len(msg: *const nng_msg) -> usize;
    pub fn nng_msg_header_append(msg: *mut nng_msg, data: *const c_void, size: usize) -> c_int;

    // Socket options
    pub fn nng_socket_set_ms(
        socket: nng_socket,
        opt: *const c_char,
        val: i32, // nng_duration = int32_t (milliseconds)
    ) -> c_int;
}
