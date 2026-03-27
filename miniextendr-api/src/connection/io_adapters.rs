//! std::io integration — capability-aware adapters for R connections.
//!
//! Provides adapter types that wrap `std::io::{Read, Write, Seek}` implementations
//! and expose them as R connections via [`RConnectionImpl`](super::RConnectionImpl).

use crate::ffi::SEXP;

use super::{RConnectionImpl, RCustomConnection};

/// Capability flags for I/O operations.
///
/// Used by adapter types to declare which operations they support,
/// enabling automatic configuration of R connection flags.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::connection::{IoCaps, IoRead};
///
/// // IoRead<T> automatically declares read capability
/// assert!(IoRead::<std::io::Cursor<Vec<u8>>>::HAS_READ);
/// assert!(!IoRead::<std::io::Cursor<Vec<u8>>>::HAS_WRITE);
/// ```
pub trait IoCaps {
    /// True if the connection supports reading.
    const HAS_READ: bool = false;
    /// True if the connection supports writing.
    const HAS_WRITE: bool = false;
    /// True if the connection supports seeking.
    const HAS_SEEK: bool = false;
    /// True if the connection uses BufRead (optimized buffered reading).
    const HAS_BUFREAD: bool = false;
    /// True if the connection is a terminal (affects text/blocking defaults).
    const HAS_TERMINAL: bool = false;
}

/// Macro to generate std::io adapter types with automatic capability detection.
///
/// This reduces ~400 lines of boilerplate to ~50 lines of macro invocations.
macro_rules! define_io_adapter {
    (
        $(#[$meta:meta])*
        $adapter_name:ident<$t:ident: $($trait_bound:path),+>
        {
            caps: { $($cap_name:ident = $cap_value:expr),* $(,)? },
            methods: {
                $($method_impl:tt)*
            }
        }
    ) => {
        $(#[$meta])*
        pub struct $adapter_name<$t: $($trait_bound +)+ 'static> {
            inner: $t,
        }

        impl<$t: $($trait_bound +)+ 'static> $adapter_name<$t> {
            /// Create a new adapter.
            pub fn new(inner: $t) -> Self {
                Self { inner }
            }
        }

        impl<$t: $($trait_bound +)+ 'static> IoCaps for $adapter_name<$t> {
            $(const $cap_name: bool = $cap_value;)*
        }

        impl<$t: $($trait_bound +)+ 'static> RConnectionImpl for $adapter_name<$t> {
            $($method_impl)*
        }
    };
}

// Helper macro for seek implementation (shared across all seekable adapters)
macro_rules! impl_seek {
    () => {
        fn seek(&mut self, where_: f64, origin: i32, _rw: i32) -> f64 {
            // Handle position query (where_ is NA/NaN)
            if where_.is_nan() {
                return self
                    .inner
                    .stream_position()
                    .map(|pos| pos as f64)
                    .unwrap_or(-1.0);
            }

            // Map R's origin to SeekFrom
            let seek_from = match origin {
                1 => std::io::SeekFrom::Start(where_.max(0.0) as u64),
                2 => std::io::SeekFrom::Current(where_ as i64),
                3 => std::io::SeekFrom::End(where_ as i64),
                _ => return -1.0,
            };

            self.inner
                .seek(seek_from)
                .map(|pos| pos as f64)
                .unwrap_or(-1.0)
        }
    };
}

define_io_adapter! {
    /// Adapter for types implementing `std::io::Read`.
    ///
    /// Provides read-only connection with automatic capability detection.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::{IoRead, RCustomConnection};
    ///
    /// let data = std::io::Cursor::new(vec![1u8, 2, 3]);
    /// let adapter = IoRead::new(data);
    /// let conn = RCustomConnection::new()
    ///     .description("byte reader")
    ///     .mode("rb")
    ///     .can_read(true)
    ///     .build(adapter);
    /// ```
    IoRead<T: std::io::Read> {
        caps: { HAS_READ = true },
        methods: {
            fn read(&mut self, buf: &mut [u8]) -> usize {
                self.inner.read(buf).unwrap_or(0)
            }
        }
    }
}

define_io_adapter! {
    /// Adapter for types implementing `std::io::Write`.
    ///
    /// Provides write-only connection with automatic capability detection.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::{IoWrite, RCustomConnection};
    ///
    /// let sink = Vec::<u8>::new();
    /// let adapter = IoWrite::new(sink);
    /// let conn = RCustomConnection::new()
    ///     .description("byte writer")
    ///     .mode("wb")
    ///     .can_write(true)
    ///     .build(adapter);
    /// ```
    IoWrite<T: std::io::Write> {
        caps: { HAS_WRITE = true },
        methods: {
            fn write(&mut self, buf: &[u8]) -> usize {
                self.inner.write(buf).unwrap_or(0)
            }

            fn flush(&mut self) -> i32 {
                if self.inner.flush().is_ok() { 0 } else { -1 }
            }
        }
    }
}

define_io_adapter! {
    /// Adapter for types implementing both `Read` and `Write`.
    ///
    /// Provides read-write connection with automatic capability detection.
    IoReadWrite<T: std::io::Read, std::io::Write> {
        caps: { HAS_READ = true, HAS_WRITE = true },
        methods: {
            fn read(&mut self, buf: &mut [u8]) -> usize {
                self.inner.read(buf).unwrap_or(0)
            }

            fn write(&mut self, buf: &[u8]) -> usize {
                self.inner.write(buf).unwrap_or(0)
            }

            fn flush(&mut self) -> i32 {
                if self.inner.flush().is_ok() { 0 } else { -1 }
            }
        }
    }
}

define_io_adapter! {
    /// Adapter for types implementing `Read + Seek`.
    ///
    /// Provides seekable read connection with automatic capability detection.
    IoReadSeek<T: std::io::Read, std::io::Seek> {
        caps: { HAS_READ = true, HAS_SEEK = true },
        methods: {
            fn read(&mut self, buf: &mut [u8]) -> usize {
                self.inner.read(buf).unwrap_or(0)
            }

            impl_seek!();
        }
    }
}

define_io_adapter! {
    /// Adapter for types implementing `Write + Seek`.
    ///
    /// Provides seekable write connection with automatic capability detection.
    IoWriteSeek<T: std::io::Write, std::io::Seek> {
        caps: { HAS_WRITE = true, HAS_SEEK = true },
        methods: {
            fn write(&mut self, buf: &[u8]) -> usize {
                self.inner.write(buf).unwrap_or(0)
            }

            fn flush(&mut self) -> i32 {
                if self.inner.flush().is_ok() { 0 } else { -1 }
            }

            impl_seek!();
        }
    }
}

define_io_adapter! {
    /// Adapter for types implementing `Read + Write + Seek`.
    ///
    /// Provides full bidirectional seekable connection with automatic capability detection.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::{IoReadWriteSeek, RCustomConnection};
    ///
    /// let cursor = std::io::Cursor::new(Vec::<u8>::new());
    /// let adapter = IoReadWriteSeek::new(cursor);
    /// let conn = RCustomConnection::new()
    ///     .description("memory buffer")
    ///     .mode("r+b")
    ///     .can_read(true)
    ///     .can_write(true)
    ///     .can_seek(true)
    ///     .build(adapter);
    /// ```
    IoReadWriteSeek<T: std::io::Read, std::io::Write, std::io::Seek> {
        caps: { HAS_READ = true, HAS_WRITE = true, HAS_SEEK = true },
        methods: {
            fn read(&mut self, buf: &mut [u8]) -> usize {
                self.inner.read(buf).unwrap_or(0)
            }

            fn write(&mut self, buf: &[u8]) -> usize {
                self.inner.write(buf).unwrap_or(0)
            }

            fn flush(&mut self) -> i32 {
                if self.inner.flush().is_ok() { 0 } else { -1 }
            }

            impl_seek!();
        }
    }
}

define_io_adapter! {
    /// Adapter for types implementing `BufRead`.
    ///
    /// Provides buffered reading with optimized `fgetc` implementation.
    IoBufRead<T: std::io::BufRead> {
        caps: { HAS_READ = true, HAS_BUFREAD = true },
        methods: {
            fn read(&mut self, buf: &mut [u8]) -> usize {
                self.inner.read(buf).unwrap_or(0)
            }

            fn fgetc(&mut self) -> i32 {
                // Use fill_buf for optimized buffered reading
                match self.inner.fill_buf() {
                    Ok(buffer) if !buffer.is_empty() => {
                        let byte = buffer[0];
                        self.inner.consume(1);
                        byte as i32
                    }
                    _ => -1, // EOF or error
                }
            }
        }
    }
}

/// Builder for creating R connections from std::io types.
///
/// Automatically selects the appropriate adapter based on trait bounds
/// and configures connection capabilities accordingly.
///
/// # Example
///
/// ```ignore
/// use std::io::Cursor;
///
/// let data = vec![1u8, 2, 3, 4, 5];
/// let cursor = Cursor::new(data);
///
/// // Auto-detects Read + Write + Seek capabilities
/// let conn_sexp = RConnectionIo::new(cursor)
///     .description("memory buffer")
///     .mode("rb+")
///     .build_read_write_seek();
/// ```
pub struct RConnectionIo<T> {
    inner: T,
    description: Option<String>,
    mode: Option<String>,
    class_name: Option<String>,
    text: Option<bool>,
    blocking: Option<bool>,
    // Capability overrides (if None, auto-detected from adapter)
    can_read_override: Option<bool>,
    can_write_override: Option<bool>,
    can_seek_override: Option<bool>,
}

impl<T> RConnectionIo<T> {
    /// Create a new connection builder from an I/O type.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::RConnectionIo;
    /// use std::io::Cursor;
    ///
    /// let cursor = Cursor::new(vec![1u8, 2, 3]);
    /// let conn = RConnectionIo::new(cursor)
    ///     .description("in-memory data")
    ///     .mode("rb")
    ///     .build_read();
    /// ```
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            description: None,
            mode: None,
            class_name: None,
            text: None,
            blocking: None,
            can_read_override: None,
            can_write_override: None,
            can_seek_override: None,
        }
    }

    /// Set the connection description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the connection mode (e.g., "r", "w", "rb", "wb", "r+").
    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }

    /// Set the connection class name.
    pub fn class_name(mut self, name: impl Into<String>) -> Self {
        self.class_name = Some(name.into());
        self
    }

    /// Set whether this is a text connection (vs binary).
    pub fn text(mut self, is_text: bool) -> Self {
        self.text = Some(is_text);
        self
    }

    /// Set whether the connection is blocking.
    pub fn blocking(mut self, blocking: bool) -> Self {
        self.blocking = Some(blocking);
        self
    }

    /// Override the can_read capability (auto-detected by default).
    pub fn can_read(mut self, can_read: bool) -> Self {
        self.can_read_override = Some(can_read);
        self
    }

    /// Override the can_write capability (auto-detected by default).
    pub fn can_write(mut self, can_write: bool) -> Self {
        self.can_write_override = Some(can_write);
        self
    }

    /// Override the can_seek capability (auto-detected by default).
    pub fn can_seek(mut self, can_seek: bool) -> Self {
        self.can_seek_override = Some(can_seek);
        self
    }
}

// Specialized build methods for different adapter types
// Internal macro to reduce duplication in build methods
macro_rules! build_io_connection {
    (
        $adapter:expr,
        $caps:ty,
        description = $desc:expr,
        mode = $mode:expr,
        class_name = $class:expr,
        text = $text:expr,
        blocking = $blocking:expr,
        can_read_override = $read_ovr:expr,
        can_write_override = $write_ovr:expr,
        can_seek_override = $seek_ovr:expr
    ) => {{
        let mut builder = RCustomConnection::new();

        // Set description
        if let Some(desc) = $desc {
            builder = builder.description(&desc);
        }

        // Set mode
        if let Some(mode) = $mode {
            builder = builder.mode(&mode);
        } else {
            // Auto-detect mode from capabilities (binary by default for byte streams)
            let mode = match (<$caps>::HAS_READ, <$caps>::HAS_WRITE) {
                (true, true) => "rb+",
                (true, false) => "rb",
                (false, true) => "wb",
                (false, false) => "rb",
            };
            builder = builder.mode(mode);
        }

        // Set class name
        if let Some(class_name) = $class {
            builder = builder.class_name(&class_name);
        } else {
            builder = builder.class_name("ioConnection");
        }

        // Set text mode
        if let Some(text) = $text {
            builder = builder.text(text);
        }

        // Set blocking
        if let Some(blocking) = $blocking {
            builder = builder.blocking(blocking);
        } else {
            builder = builder.blocking(true);
        }

        // Set capabilities (use overrides if provided, otherwise auto-detect)
        builder = builder.can_read($read_ovr.unwrap_or(<$caps>::HAS_READ));
        builder = builder.can_write($write_ovr.unwrap_or(<$caps>::HAS_WRITE));
        builder = builder.can_seek($seek_ovr.unwrap_or(<$caps>::HAS_SEEK));

        builder.build($adapter)
    }};
}

impl<T: std::io::Read + 'static> RConnectionIo<T> {
    /// Build a read-only connection.
    ///
    /// Automatically detects `Read` capability and configures the connection accordingly.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::RConnectionIo;
    /// use std::fs::File;
    ///
    /// let file = File::open("/path/to/data.bin").unwrap();
    /// let conn = RConnectionIo::new(file)
    ///     .description("data file")
    ///     .mode("rb")
    ///     .build_read();
    /// ```
    pub fn build_read(self) -> SEXP {
        let adapter = IoRead::new(self.inner);
        build_io_connection!(
            adapter,
            IoRead<T>,
            description = self.description,
            mode = self.mode,
            class_name = self.class_name,
            text = self.text,
            blocking = self.blocking,
            can_read_override = self.can_read_override,
            can_write_override = self.can_write_override,
            can_seek_override = self.can_seek_override
        )
    }
}

impl<T: std::io::Write + 'static> RConnectionIo<T> {
    /// Build a write-only connection.
    ///
    /// Automatically detects `Write` capability and configures the connection accordingly.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::RConnectionIo;
    ///
    /// let output = Vec::<u8>::new();
    /// let conn = RConnectionIo::new(output)
    ///     .description("output buffer")
    ///     .mode("wb")
    ///     .build_write();
    /// ```
    pub fn build_write(self) -> SEXP {
        let adapter = IoWrite::new(self.inner);
        build_io_connection!(
            adapter,
            IoWrite<T>,
            description = self.description,
            mode = self.mode,
            class_name = self.class_name,
            text = self.text,
            blocking = self.blocking,
            can_read_override = self.can_read_override,
            can_write_override = self.can_write_override,
            can_seek_override = self.can_seek_override
        )
    }
}

impl<T: std::io::Read + std::io::Write + 'static> RConnectionIo<T> {
    /// Build a read-write connection.
    ///
    /// Automatically detects `Read + Write` capabilities and configures the connection accordingly.
    pub fn build_read_write(self) -> SEXP {
        let adapter = IoReadWrite::new(self.inner);
        build_io_connection!(
            adapter,
            IoReadWrite<T>,
            description = self.description,
            mode = self.mode,
            class_name = self.class_name,
            text = self.text,
            blocking = self.blocking,
            can_read_override = self.can_read_override,
            can_write_override = self.can_write_override,
            can_seek_override = self.can_seek_override
        )
    }
}

impl<T: std::io::Read + std::io::Seek + 'static> RConnectionIo<T> {
    /// Build a read+seek connection.
    ///
    /// Automatically detects `Read + Seek` capabilities and configures the connection accordingly.
    pub fn build_read_seek(self) -> SEXP {
        let adapter = IoReadSeek::new(self.inner);
        build_io_connection!(
            adapter,
            IoReadSeek<T>,
            description = self.description,
            mode = self.mode,
            class_name = self.class_name,
            text = self.text,
            blocking = self.blocking,
            can_read_override = self.can_read_override,
            can_write_override = self.can_write_override,
            can_seek_override = self.can_seek_override
        )
    }
}

impl<T: std::io::Write + std::io::Seek + 'static> RConnectionIo<T> {
    /// Build a write+seek connection.
    ///
    /// Automatically detects `Write + Seek` capabilities and configures the connection accordingly.
    pub fn build_write_seek(self) -> SEXP {
        let adapter = IoWriteSeek::new(self.inner);
        build_io_connection!(
            adapter,
            IoWriteSeek<T>,
            description = self.description,
            mode = self.mode,
            class_name = self.class_name,
            text = self.text,
            blocking = self.blocking,
            can_read_override = self.can_read_override,
            can_write_override = self.can_write_override,
            can_seek_override = self.can_seek_override
        )
    }
}

impl<T: std::io::Read + std::io::Write + std::io::Seek + 'static> RConnectionIo<T> {
    /// Build a read+write+seek connection (full capabilities).
    ///
    /// Automatically detects all capabilities and configures the connection accordingly.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::RConnectionIo;
    /// use std::io::Cursor;
    ///
    /// let buffer = Cursor::new(Vec::<u8>::new());
    /// let conn = RConnectionIo::new(buffer)
    ///     .description("seekable buffer")
    ///     .mode("r+b")
    ///     .class_name("memoryConn")
    ///     .build_read_write_seek();
    /// ```
    pub fn build_read_write_seek(self) -> SEXP {
        let adapter = IoReadWriteSeek::new(self.inner);
        build_io_connection!(
            adapter,
            IoReadWriteSeek<T>,
            description = self.description,
            mode = self.mode,
            class_name = self.class_name,
            text = self.text,
            blocking = self.blocking,
            can_read_override = self.can_read_override,
            can_write_override = self.can_write_override,
            can_seek_override = self.can_seek_override
        )
    }
}

impl<T: std::io::BufRead + 'static> RConnectionIo<T> {
    /// Build a buffered read connection with optimized character reading.
    ///
    /// Uses `BufRead::fill_buf` for efficient `fgetc` implementation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use miniextendr_api::connection::RConnectionIo;
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// let file = File::open("/path/to/text.csv").unwrap();
    /// let reader = BufReader::new(file);
    /// let conn = RConnectionIo::new(reader)
    ///     .description("CSV reader")
    ///     .mode("r")
    ///     .text(true)
    ///     .build_bufread();
    /// ```
    pub fn build_bufread(self) -> SEXP {
        let adapter = IoBufRead::new(self.inner);
        build_io_connection!(
            adapter,
            IoBufRead<T>,
            description = self.description,
            mode = self.mode,
            class_name = self.class_name,
            text = self.text,
            blocking = self.blocking,
            can_read_override = self.can_read_override,
            can_write_override = self.can_write_override,
            can_seek_override = self.can_seek_override
        )
    }
}

#[cfg(test)]
mod tests;
// endregion
