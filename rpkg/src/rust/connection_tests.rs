//! Examples demonstrating R custom connections from Rust.
//!
//! This module shows how to use the connections API to create R connections
//! backed by Rust types. Requires the `connections` feature.

#[cfg(feature = "connections")]
use miniextendr_api::connection::{RConnectionImpl, RConnectionIo, RCustomConnection};
#[cfg(feature = "connections")]
use miniextendr_api::ffi::SEXP;
#[cfg(feature = "connections")]
use miniextendr_api::miniextendr;
#[cfg(feature = "connections")]
use std::io::Cursor;

// region: Example 1: In-memory buffer connection (Read + Write + Seek)

/// In-memory buffer that can be used as an R connection.
#[cfg(feature = "connections")]
struct MemoryBuffer {
    data: Vec<u8>,
    position: usize,
}

#[cfg(feature = "connections")]
impl MemoryBuffer {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            position: 0,
        }
    }

    fn with_data(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }
}

#[cfg(feature = "connections")]
impl RConnectionImpl for MemoryBuffer {
    fn open(&mut self) -> bool {
        self.position = 0;
        true
    }

    fn close(&mut self) {
        // Nothing to clean up
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        let available = self.data.len().saturating_sub(self.position);
        let to_read = buf.len().min(available);
        if to_read > 0 {
            buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
            self.position += to_read;
        }
        to_read
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        // Extend buffer if necessary
        let end_pos = self.position + buf.len();
        if end_pos > self.data.len() {
            self.data.resize(end_pos, 0);
        }
        self.data[self.position..end_pos].copy_from_slice(buf);
        self.position = end_pos;
        buf.len()
    }

    fn seek(&mut self, where_: f64, origin: i32, _rw: i32) -> f64 {
        // Handle position query
        if where_.is_nan() {
            return self.position as f64;
        }

        let new_pos = match origin {
            1 => where_.max(0.0) as usize, // Start
            2 => {
                // Current
                let offset = where_ as isize;
                if offset < 0 && (-offset as usize) > self.position {
                    0
                } else {
                    (self.position as isize + offset) as usize
                }
            }
            3 => {
                // End
                let offset = where_ as isize;
                if offset < 0 && (-offset as usize) > self.data.len() {
                    0
                } else {
                    (self.data.len() as isize + offset) as usize
                }
            }
            _ => return -1.0,
        };

        // Clamp to valid range
        self.position = new_pos.min(self.data.len());
        self.position as f64
    }

    fn flush(&mut self) -> i32 {
        0 // Success (nothing to flush for in-memory buffer)
    }
}

/// Create an empty in-memory connection for reading and writing.
///
/// @return A custom R connection backed by an in-memory buffer
/// @examples
/// \dontrun{
/// conn <- memory_connection()
/// writeLines("Hello, World!", conn)
/// seek(conn, 0)
/// readLines(conn)  # "Hello, World!"
/// close(conn)
/// }
#[cfg(feature = "connections")]
/// Test creating an empty in-memory read-write connection.
#[miniextendr]
pub fn memory_connection() -> SEXP {
    RCustomConnection::new()
        .description("memory buffer")
        .mode("r+b")
        .class_name("memoryConnection")
        .can_read(true)
        .can_write(true)
        .can_seek(true)
        .text(false)
        .build(MemoryBuffer::new())
}

/// Create a memory connection pre-filled with the given string.
///
/// @param content Initial content for the buffer
/// @return A custom R connection for reading the content
/// @examples
/// \dontrun{
/// conn <- string_input_connection("line1\nline2\nline3")
/// readLines(conn)  # c("line1", "line2", "line3")
/// close(conn)
/// }
#[cfg(feature = "connections")]
/// Test creating a read-only connection pre-filled with a string.
/// @param content Character string for initial buffer content.
#[miniextendr]
pub fn string_input_connection(content: &str) -> SEXP {
    RCustomConnection::new()
        .description("string input")
        .mode("r")
        .class_name("stringInputConnection")
        .can_read(true)
        .can_write(false)
        .can_seek(true)
        .text(true)
        .build(MemoryBuffer::with_data(content.as_bytes().to_vec()))
}
// endregion

// region: Example 2: Counter connection (generates numeric data on-the-fly)

/// A connection that generates sequential integers as text lines.
#[cfg(feature = "connections")]
struct CounterConnection {
    current: i32,
    max: i32,
    buffer: Vec<u8>,
    buffer_pos: usize,
}

#[cfg(feature = "connections")]
impl CounterConnection {
    fn new(start: i32, end: i32) -> Self {
        Self {
            current: start,
            max: end,
            buffer: Vec::new(),
            buffer_pos: 0,
        }
    }

    fn fill_buffer(&mut self) {
        if self.current <= self.max && self.buffer_pos >= self.buffer.len() {
            // Generate next line
            self.buffer = format!("{}\n", self.current).into_bytes();
            self.buffer_pos = 0;
            self.current += 1;
        }
    }
}

#[cfg(feature = "connections")]
impl RConnectionImpl for CounterConnection {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        self.fill_buffer();

        let available = self.buffer.len().saturating_sub(self.buffer_pos);
        if available == 0 {
            return 0; // EOF
        }

        let to_read = buf.len().min(available);
        buf[..to_read].copy_from_slice(&self.buffer[self.buffer_pos..self.buffer_pos + to_read]);
        self.buffer_pos += to_read;
        to_read
    }
}

/// Create a connection that generates sequential integers.
///
/// @param start First integer to generate
/// @param end Last integer to generate (inclusive)
/// @return A read-only connection that generates one integer per line
/// @examples
/// \dontrun{
/// conn <- counter_connection(1L, 5L)
/// readLines(conn)  # c("1", "2", "3", "4", "5")
/// close(conn)
/// }
#[cfg(feature = "connections")]
/// Test creating a connection that generates sequential integers.
/// @param start Integer first value to generate.
/// @param end Integer last value to generate (inclusive).
#[miniextendr]
pub fn counter_connection(start: i32, end: i32) -> SEXP {
    RCustomConnection::new()
        .description(format!("counter {}:{}", start, end).as_str())
        .mode("r")
        .class_name("counterConnection")
        .can_read(true)
        .can_write(false)
        .can_seek(false)
        .text(true)
        .build(CounterConnection::new(start, end))
}
// endregion

// region: Example 3: std::io adapter (using Cursor for in-memory Read+Write+Seek)

/// Create a connection using Rust's std::io Cursor.
///
/// Demonstrates the RConnectionIo adapter for wrapping std::io types.
///
/// @param data Initial data for the cursor
/// @return A read-write-seekable connection backed by a Cursor
/// @examples
/// \dontrun{
/// conn <- cursor_connection(charToRaw("Hello"))
/// rawToChar(readBin(conn, "raw", 5))  # "Hello"
/// seek(conn, 0)
/// writeBin(charToRaw("Hi!!!"), conn)
/// seek(conn, 0)
/// rawToChar(readBin(conn, "raw", 5))  # "Hi!!!"
/// close(conn)
/// }
#[cfg(feature = "connections")]
/// Test creating a read-write-seekable Cursor-backed connection.
/// @param data Raw vector of initial data for the cursor.
#[miniextendr]
pub fn cursor_connection(data: Vec<u8>) -> SEXP {
    let cursor = Cursor::new(data);
    RConnectionIo::new(cursor)
        .description("Rust Cursor")
        .mode("r+b")
        .class_name("cursorConnection")
        .build_read_write_seek()
}

/// Create an empty cursor connection for writing.
///
/// @return A write-seekable connection starting empty
/// @examples
/// \dontrun{
/// conn <- empty_cursor_connection()
/// writeBin(as.raw(1:10), conn)
/// seek(conn, 0)
/// readBin(conn, "raw", 10)  # raw(10): 01 02 03 04 05 06 07 08 09 0a
/// close(conn)
/// }
#[cfg(feature = "connections")]
/// Test creating an empty write-seekable Cursor-backed connection.
#[miniextendr]
pub fn empty_cursor_connection() -> SEXP {
    let cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    RConnectionIo::new(cursor)
        .description("empty cursor")
        .mode("w+b")
        .class_name("cursorConnection")
        .build_read_write_seek()
}
// endregion

// region: Example 4: Transform connection (applies a transformation on read)

/// A connection that transforms input data.
#[cfg(feature = "connections")]
struct TransformConnection<F>
where
    F: Fn(u8) -> u8 + 'static,
{
    data: Vec<u8>,
    position: usize,
    transform: F,
}

#[cfg(feature = "connections")]
impl<F: Fn(u8) -> u8 + 'static> TransformConnection<F> {
    fn new(data: Vec<u8>, transform: F) -> Self {
        Self {
            data,
            position: 0,
            transform,
        }
    }
}

#[cfg(feature = "connections")]
impl<F: Fn(u8) -> u8 + 'static> RConnectionImpl for TransformConnection<F> {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        let available = self.data.len().saturating_sub(self.position);
        let to_read = buf.len().min(available);

        for (i, slot) in buf.iter_mut().enumerate().take(to_read) {
            *slot = (self.transform)(self.data[self.position + i]);
        }
        self.position += to_read;
        to_read
    }
}

/// Create a connection that returns uppercase text.
///
/// @param text Input text to transform
/// @return A read-only connection that returns uppercase text
/// @examples
/// \dontrun{
/// conn <- uppercase_connection("hello world")
/// readLines(conn)  # "HELLO WORLD"
/// close(conn)
/// }
#[cfg(feature = "connections")]
/// Test creating a connection that transforms text to uppercase on read.
/// @param text Character string of input text to transform.
#[miniextendr]
pub fn uppercase_connection(text: &str) -> SEXP {
    let data = text.as_bytes().to_vec();
    let transform = |b: u8| {
        if b.is_ascii_lowercase() {
            b.to_ascii_uppercase()
        } else {
            b
        }
    };
    RCustomConnection::new()
        .description("uppercase transform")
        .mode("r")
        .class_name("uppercaseConnection")
        .can_read(true)
        .can_write(false)
        .can_seek(false)
        .text(true)
        .build(TransformConnection::new(data, transform))
}

/// Create a connection that ROT13-encodes text.
///
/// @param text Input text to encode
/// @return A read-only connection that returns ROT13-encoded text
/// @examples
/// \dontrun{
/// conn <- rot13_connection("hello")
/// readLines(conn)  # "uryyb"
/// close(conn)
/// # Applying ROT13 twice returns the original
/// conn2 <- rot13_connection("uryyb")
/// readLines(conn2)  # "hello"
/// }
#[cfg(feature = "connections")]
/// Test creating a connection that ROT13-encodes text on read.
/// @param text Character string of input text to encode.
#[miniextendr]
pub fn rot13_connection(text: &str) -> SEXP {
    let data = text.as_bytes().to_vec();
    let transform = |b: u8| {
        if b.is_ascii_lowercase() {
            b'a' + (b - b'a' + 13) % 26
        } else if b.is_ascii_uppercase() {
            b'A' + (b - b'A' + 13) % 26
        } else {
            b
        }
    };
    RCustomConnection::new()
        .description("ROT13 transform")
        .mode("r")
        .class_name("rot13Connection")
        .can_read(true)
        .can_write(false)
        .can_seek(false)
        .text(true)
        .build(TransformConnection::new(data, transform))
}
// endregion

// region: Standard stream and null connection fixtures — issues #175 + #176

/// Return R's stdout() terminal connection from Rust.
///
/// Used in tests to verify the type round-trips correctly.
/// @return R's stdout connection object.
#[cfg(feature = "connections")]
#[miniextendr]
pub fn rust_get_stdout() -> miniextendr_api::connection::RStdout {
    miniextendr_api::connection::RStdout
}

/// Return R's stderr() terminal connection from Rust.
///
/// Used in tests to verify the type round-trips correctly.
/// @return R's stderr connection object.
#[cfg(feature = "connections")]
#[miniextendr]
pub fn rust_get_stderr() -> miniextendr_api::connection::RStderr {
    miniextendr_api::connection::RStderr
}

/// Write a message to R's stderr connection from Rust.
///
/// Useful for capturing stderr output from Rust code via `capture.output()`.
/// @param message Character string to emit on stderr.
#[cfg(feature = "connections")]
#[miniextendr]
pub fn rust_write_to_stderr(message: String) {
    use miniextendr_api::connection::RStderr;
    use std::io::Write;
    let mut e = RStderr;
    writeln!(e, "{message}").expect("write to RStderr failed");
}

/// Open a null device connection from Rust and return it to R.
///
/// The no-arg signature makes this eligible for the gctorture GC sweep.
/// The returned connection is open; R will close it when it is GC'd or
/// when the caller calls `close()` on it.
/// @return An open write-only connection to the null device.
#[cfg(feature = "connections")]
#[miniextendr]
pub fn rust_get_null_connection() -> miniextendr_api::connection::RNullConnection {
    miniextendr_api::connection::RNullConnection::new()
}

/// Write a message to a new null connection from Rust (output discarded).
///
/// Exercises the full write path: allocate → write → auto-close on drop.
/// @param message Character string to write (discarded by null device).
#[cfg(feature = "connections")]
#[miniextendr]
pub fn rust_write_to_null(message: String) {
    use miniextendr_api::connection::RNullConnection;
    use std::io::Write;
    let mut n = RNullConnection::new();
    writeln!(n, "{message}").expect("write to RNullConnection failed");
}

// endregion
