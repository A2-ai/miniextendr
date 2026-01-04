//! Built-in adapter traits for common Rust standard library traits.
//!
//! These traits provide blanket implementations that allow any Rust type
//! implementing standard traits to be exposed to R without boilerplate.
//!
//! # Example
//!
//! ```rust,ignore
//! use miniextendr_api::prelude::*;
//! use miniextendr_api::adapter_traits::RDebug;
//!
//! #[derive(Debug, ExternalPtr)]
//! struct MyData {
//!     values: Vec<i32>,
//! }
//!
//! // RDebug is automatically available for any Debug type
//! #[miniextendr]
//! impl RDebug for MyData {}
//!
//! miniextendr_module! {
//!     mod mymod;
//!     impl RDebug for MyData;
//! }
//! ```
//!
//! In R:
//! ```r
//! data <- MyData$new(...)
//! data$debug_str()        # "MyData { values: [1, 2, 3] }"
//! data$debug_str_pretty() # Pretty-printed with newlines
//! ```

use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::str::FromStr;

/// Adapter trait for [`std::fmt::Debug`].
///
/// Provides string representations for debugging and inspection in R.
/// Automatically implemented for any type that implements `Debug`.
///
/// # Methods
///
/// - `debug_str()` - Returns compact debug string (`:?` format)
/// - `debug_str_pretty()` - Returns pretty-printed debug string (`:#?` format)
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Debug, ExternalPtr)]
/// struct Config { name: String, value: i32 }
///
/// #[miniextendr]
/// impl RDebug for Config {}
/// ```
pub trait RDebug {
    /// Get a compact debug string representation.
    fn debug_str(&self) -> String;

    /// Get a pretty-printed debug string with indentation.
    fn debug_str_pretty(&self) -> String;
}

impl<T: Debug> RDebug for T {
    fn debug_str(&self) -> String {
        format!("{:?}", self)
    }

    fn debug_str_pretty(&self) -> String {
        format!("{:#?}", self)
    }
}

/// Adapter trait for [`std::fmt::Display`].
///
/// Provides user-friendly string conversion for R.
/// Automatically implemented for any type that implements `Display`.
///
/// # Methods
///
/// - `to_r_string()` - Returns the Display representation
///
/// # Example
///
/// ```rust,ignore
/// struct Version(u32, u32, u32);
///
/// impl Display for Version {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "{}.{}.{}", self.0, self.1, self.2)
///     }
/// }
///
/// #[miniextendr]
/// impl RDisplay for Version {}
/// ```
pub trait RDisplay {
    /// Convert to a user-friendly string.
    fn to_r_string(&self) -> String;
}

impl<T: Display> RDisplay for T {
    fn to_r_string(&self) -> String {
        self.to_string()
    }
}

/// Adapter trait for [`std::hash::Hash`].
///
/// Provides hashing for deduplication and environment keys in R.
/// Automatically implemented for any type that implements `Hash`.
///
/// # Methods
///
/// - `r_hash()` - Returns a 64-bit hash as i64
///
/// # Note
///
/// Hash values are deterministic within a single R session but may vary
/// between sessions due to Rust's hasher implementation.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Hash, ExternalPtr)]
/// struct Record { id: String, value: i64 }
///
/// #[miniextendr]
/// impl RHash for Record {}
/// ```
pub trait RHash {
    /// Compute a hash of this value.
    fn r_hash(&self) -> i64;
}

impl<T: Hash> RHash for T {
    fn r_hash(&self) -> i64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as i64
    }
}

/// Adapter trait for [`std::cmp::Ord`].
///
/// Provides total ordering comparison for R sorting operations.
/// Automatically implemented for any type that implements `Ord`.
///
/// # Methods
///
/// - `r_cmp(&self, other: &Self)` - Returns -1, 0, or 1
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Ord, PartialOrd, Eq, PartialEq, ExternalPtr)]
/// struct Priority(u32);
///
/// #[miniextendr]
/// impl ROrd for Priority {}
/// ```
pub trait ROrd {
    /// Compare with another value.
    ///
    /// Returns:
    /// - `-1` if `self < other`
    /// - `0` if `self == other`
    /// - `1` if `self > other`
    fn r_cmp(&self, other: &Self) -> i32;
}

impl<T: Ord> ROrd for T {
    fn r_cmp(&self, other: &Self) -> i32 {
        match self.cmp(other) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}

/// Adapter trait for [`std::cmp::PartialOrd`].
///
/// Provides partial ordering comparison for R, handling incomparable values.
/// Automatically implemented for any type that implements `PartialOrd`.
///
/// # Methods
///
/// - `r_partial_cmp(&self, other: &Self)` - Returns Some(-1/0/1) or None
///
/// # Example
///
/// ```rust,ignore
/// // f64 has partial ordering (NaN is not comparable)
/// #[miniextendr]
/// impl RPartialOrd for MyFloat {}
/// ```
pub trait RPartialOrd {
    /// Compare with another value, returning None if incomparable.
    ///
    /// Returns:
    /// - `Some(-1)` if `self < other`
    /// - `Some(0)` if `self == other`
    /// - `Some(1)` if `self > other`
    /// - `None` if values are incomparable (maps to NA in R)
    fn r_partial_cmp(&self, other: &Self) -> Option<i32>;
}

impl<T: PartialOrd> RPartialOrd for T {
    fn r_partial_cmp(&self, other: &Self) -> Option<i32> {
        self.partial_cmp(other).map(|ord| match ord {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }
}

/// Adapter trait for [`std::error::Error`].
///
/// Provides error message extraction and error chain walking for R.
/// Automatically implemented for any type that implements `Error`.
///
/// # Methods
///
/// - `error_message()` - Returns the error's display message
/// - `error_chain()` - Returns all messages in the error chain
///
/// # Example
///
/// ```rust,ignore
/// use std::error::Error;
/// use std::fmt;
///
/// #[derive(Debug)]
/// struct MyError { msg: String, source: Option<Box<dyn Error + Send + Sync>> }
///
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "{}", self.msg)
///     }
/// }
///
/// impl Error for MyError {
///     fn source(&self) -> Option<&(dyn Error + 'static)> {
///         self.source.as_ref().map(|e| e.as_ref() as _)
///     }
/// }
///
/// // Wrap in ExternalPtr for R access
/// #[derive(ExternalPtr)]
/// struct MyErrorWrapper(MyError);
///
/// #[miniextendr]
/// impl RError for MyErrorWrapper {}
/// ```
pub trait RError {
    /// Get the error message (Display representation).
    fn error_message(&self) -> String;

    /// Get all error messages in the chain, from outermost to innermost.
    fn error_chain(&self) -> Vec<String>;

    /// Get the number of errors in the chain.
    fn error_chain_length(&self) -> i32;
}

impl<T: std::error::Error> RError for T {
    fn error_message(&self) -> String {
        self.to_string()
    }

    fn error_chain(&self) -> Vec<String> {
        let mut chain = vec![self.to_string()];
        let mut current: &dyn std::error::Error = self;
        while let Some(source) = current.source() {
            chain.push(source.to_string());
            current = source;
        }
        chain
    }

    fn error_chain_length(&self) -> i32 {
        let mut count = 1i32;
        let mut current: &dyn std::error::Error = self;
        while let Some(source) = current.source() {
            count += 1;
            current = source;
        }
        count
    }
}

/// Adapter trait for [`std::str::FromStr`].
///
/// Provides string parsing for R, allowing R strings to be parsed into Rust types.
/// Automatically implemented for any type that implements `FromStr`.
///
/// # Methods
///
/// - `r_from_str(s: &str)` - Parse a string into this type, returning None on failure
///
/// # Example
///
/// ```rust,ignore
/// use std::net::IpAddr;
///
/// // IpAddr implements FromStr
/// #[derive(ExternalPtr)]
/// struct IpAddress(IpAddr);
///
/// #[miniextendr]
/// impl RFromStr for IpAddress {}
/// ```
///
/// In R:
/// ```r
/// ip <- IpAddress$r_from_str("192.168.1.1")
/// ```
pub trait RFromStr: Sized {
    /// Parse a string into this type.
    ///
    /// Returns `Some(value)` on success, `None` on parse failure.
    /// The None case maps to NULL in R.
    fn r_from_str(s: &str) -> Option<Self>;
}

impl<T: FromStr> RFromStr for T {
    fn r_from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

/// Adapter trait for [`std::clone::Clone`].
///
/// Provides explicit deep copying for R. This is useful when R users need
/// to create independent copies of Rust objects (which normally use reference
/// semantics via `ExternalPtr`).
///
/// # Methods
///
/// - `r_clone()` - Create a deep copy of this value
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Clone, ExternalPtr)]
/// struct Buffer { data: Vec<u8> }
///
/// #[miniextendr]
/// impl RClone for Buffer {}
/// ```
///
/// In R:
/// ```r
/// buf1 <- Buffer$new(...)
/// buf2 <- buf1$r_clone()  # Independent copy
/// ```
pub trait RClone {
    /// Create a deep copy of this value.
    fn r_clone(&self) -> Self;
}

impl<T: Clone> RClone for T {
    fn r_clone(&self) -> Self {
        self.clone()
    }
}

/// Adapter trait for [`std::default::Default`].
///
/// Provides default value construction for R. This allows R users to create
/// instances with default values without needing to specify all parameters.
///
/// # Methods
///
/// - `r_default()` - Create a new instance with default values
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Default, ExternalPtr)]
/// struct Config {
///     timeout: u32,     // defaults to 0
///     retries: u32,     // defaults to 0
///     verbose: bool,    // defaults to false
/// }
///
/// #[miniextendr]
/// impl RDefault for Config {}
/// ```
///
/// In R:
/// ```r
/// config <- Config$r_default()  # All fields have default values
/// ```
pub trait RDefault {
    /// Create a new instance with default values.
    fn r_default() -> Self;
}

impl<T: Default> RDefault for T {
    fn r_default() -> Self {
        Self::default()
    }
}

/// Adapter trait for [`std::marker::Copy`].
///
/// Indicates that a type can be cheaply copied (bitwise copy, no heap allocation).
/// This is useful for R users to know that copying is O(1) and doesn't involve
/// deep cloning of heap data.
///
/// # Methods
///
/// - `r_copy()` - Create a bitwise copy of this value
/// - `is_copy()` - Returns true (useful for runtime type checking in R)
///
/// # Difference from RClone
///
/// Both `RCopy` and `RClone` create copies, but:
/// - `RCopy`: Only for types where copying is cheap (stack-only, no heap)
/// - `RClone`: For any clonable type (may involve heap allocation)
///
/// If a type implements both, prefer `r_copy()` when you know copies are frequent.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Copy, Clone, ExternalPtr)]
/// struct Point { x: f64, y: f64 }
///
/// #[miniextendr]
/// impl RCopy for Point {}
/// ```
///
/// In R:
/// ```r
/// p1 <- Point$new(1.0, 2.0)
/// p2 <- p1$r_copy()  # Cheap bitwise copy
/// p1$is_copy()       # TRUE
/// ```
pub trait RCopy {
    /// Create a bitwise copy of this value.
    ///
    /// For Copy types, this is always cheap (O(1), no heap allocation).
    fn r_copy(&self) -> Self;

    /// Check if this type implements Copy.
    ///
    /// Always returns true for types implementing this trait.
    /// Useful for runtime type checking in R.
    fn is_copy(&self) -> bool;
}

impl<T: Copy> RCopy for T {
    fn r_copy(&self) -> Self {
        *self
    }

    fn is_copy(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdebug() {
        let v = vec![1, 2, 3];
        assert_eq!(v.debug_str(), "[1, 2, 3]");
        assert!(v.debug_str_pretty().contains('\n') || v.debug_str_pretty() == "[1, 2, 3]");
    }

    #[test]
    fn test_rdisplay() {
        let s = "hello";
        assert_eq!(s.to_r_string(), "hello");

        let n = 42i32;
        assert_eq!(n.to_r_string(), "42");
    }

    #[test]
    fn test_rhash() {
        let a = "test";
        let b = "test";
        let c = "other";

        assert_eq!(a.r_hash(), b.r_hash());
        assert_ne!(a.r_hash(), c.r_hash());
    }

    #[test]
    fn test_rord() {
        assert_eq!(1i32.r_cmp(&2), -1);
        assert_eq!(2i32.r_cmp(&2), 0);
        assert_eq!(3i32.r_cmp(&2), 1);
    }

    #[test]
    fn test_rpartialord() {
        assert_eq!(1.0f64.r_partial_cmp(&2.0), Some(-1));
        assert_eq!(2.0f64.r_partial_cmp(&2.0), Some(0));
        assert_eq!(3.0f64.r_partial_cmp(&2.0), Some(1));
        assert_eq!(f64::NAN.r_partial_cmp(&1.0), None);
    }

    #[test]
    fn test_rerror_simple() {
        use std::io;
        let err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        assert_eq!(err.error_message(), "file not found");
        assert_eq!(err.error_chain().len(), 1);
        assert_eq!(err.error_chain_length(), 1);
    }

    #[test]
    fn test_rerror_chain() {
        use std::fmt;

        #[derive(Debug)]
        struct OuterError {
            msg: &'static str,
            source: InnerError,
        }

        #[derive(Debug)]
        struct InnerError {
            msg: &'static str,
        }

        impl fmt::Display for OuterError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.msg)
            }
        }

        impl fmt::Display for InnerError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.msg)
            }
        }

        impl std::error::Error for InnerError {}

        impl std::error::Error for OuterError {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.source)
            }
        }

        let err = OuterError {
            msg: "outer error",
            source: InnerError { msg: "inner error" },
        };

        assert_eq!(err.error_message(), "outer error");
        let chain = err.error_chain();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], "outer error");
        assert_eq!(chain[1], "inner error");
        assert_eq!(err.error_chain_length(), 2);
    }

    #[test]
    fn test_rfromstr_success() {
        let result: Option<i32> = RFromStr::r_from_str("42");
        assert_eq!(result, Some(42));

        let result: Option<f64> = RFromStr::r_from_str("3.14");
        assert_eq!(result, Some(3.14));

        let result: Option<bool> = RFromStr::r_from_str("true");
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_rfromstr_failure() {
        let result: Option<i32> = RFromStr::r_from_str("not a number");
        assert_eq!(result, None);

        let result: Option<f64> = RFromStr::r_from_str("abc");
        assert_eq!(result, None);
    }

    #[test]
    fn test_rclone() {
        let v = vec![1, 2, 3];
        let cloned = v.r_clone();
        assert_eq!(v, cloned);

        // Verify it's a deep copy
        let s = String::from("hello");
        let cloned_s = s.r_clone();
        assert_eq!(s, cloned_s);
    }

    #[test]
    fn test_rdefault() {
        let default_i32: i32 = RDefault::r_default();
        assert_eq!(default_i32, 0);

        let default_vec: Vec<i32> = RDefault::r_default();
        assert!(default_vec.is_empty());

        let default_string: String = RDefault::r_default();
        assert_eq!(default_string, "");

        let default_bool: bool = RDefault::r_default();
        assert!(!default_bool);
    }

    #[test]
    fn test_rcopy() {
        // Primitives are Copy
        let x = 42i32;
        let y = x.r_copy();
        assert_eq!(x, y);
        assert!(x.is_copy());

        // Tuples of Copy types are Copy
        let point = (1.0f64, 2.0f64);
        let point2 = point.r_copy();
        assert_eq!(point, point2);
        assert!(point.is_copy());

        // Arrays of Copy types are Copy
        let arr = [1, 2, 3];
        let arr2 = arr.r_copy();
        assert_eq!(arr, arr2);
    }
}
