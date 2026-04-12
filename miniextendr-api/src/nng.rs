//! Safe Rust wrappers for the NNG (nanomsg next generation) C library.
//!
//! NNG provides lightweight, broker-less messaging with multiple transport types
//! (inproc, IPC, TCP, TLS) and scalability protocols (req/rep, pub/sub, push/pull, etc.).
//!
//! These wrappers are RAII-based: `Drop` ensures cleanup. NNG types are `Send`
//! (NNG is thread-safe) but not `Sync` (concurrent operations on the same socket
//! require external synchronization or separate contexts).
//!
//! # Requires
//!
//! The `nng` cargo feature AND the NNG C library linked via Makevars.
//! Without the C library, you'll get linker errors.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::nng::*;
//!
//! let rep = NngSocket::rep()?;
//! rep.listen("inproc://test")?;
//!
//! let req = NngSocket::req()?;
//! req.dial("inproc://test")?;
//!
//! req.send(b"hello")?;
//! let msg = rep.recv()?;
//! assert_eq!(&msg, b"hello");
//! ```

mod error;
pub mod ffi;
mod msg;
mod socket;

pub use error::{NngError, NngResult};
pub use msg::NngMsg;
pub use socket::NngSocket;
