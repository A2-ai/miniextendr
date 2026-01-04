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
}
