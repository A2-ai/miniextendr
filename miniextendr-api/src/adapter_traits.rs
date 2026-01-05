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

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RDebug for Config;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RDisplay for Version;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RHash for Record;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl ROrd for Priority;
/// }
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
/// #[derive(PartialOrd, PartialEq, ExternalPtr)]
/// struct MyFloat(f64);
///
/// #[miniextendr]
/// impl RPartialOrd for MyFloat {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RPartialOrd for MyFloat;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RError for MyErrorWrapper;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RFromStr for IpAddress;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RClone for Buffer;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RDefault for Config;
/// }
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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RCopy for Point;
/// }
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

/// Adapter trait for [`std::iter::Iterator`].
///
/// Provides iterator operations for R, allowing Rust iterators to be consumed
/// element-by-element from R code. Since iterators are stateful, the wrapper
/// type should use interior mutability (e.g., `RefCell`).
///
/// # Methods
///
/// - `r_next()` - Get the next element, or None if exhausted
/// - `r_size_hint()` - Get estimated remaining elements as `c(lower, upper)`
/// - `r_count()` - Consume and count remaining elements
/// - `r_collect_n(n)` - Collect up to n elements into a vector
/// - `r_skip(n)` - Skip n elements
/// - `r_nth(n)` - Get the nth element (0-indexed)
///
/// # Example
///
/// ```rust,ignore
/// use std::cell::RefCell;
///
/// #[derive(ExternalPtr)]
/// struct MyIter(RefCell<std::vec::IntoIter<i32>>);
///
/// impl MyIter {
///     fn new(data: Vec<i32>) -> Self {
///         Self(RefCell::new(data.into_iter()))
///     }
/// }
///
/// impl RIterator for MyIter {
///     type Item = i32;
///
///     fn r_next(&self) -> Option<Self::Item> {
///         self.0.borrow_mut().next()
///     }
///
///     fn r_size_hint(&self) -> (i64, Option<i64>) {
///         let (lo, hi) = self.0.borrow().size_hint();
///         (lo as i64, hi.map(|h| h as i64))
///     }
/// }
///
/// #[miniextendr]
/// impl RIterator for MyIter {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RIterator for MyIter;
/// }
/// ```
///
/// In R:
/// ```r
/// it <- MyIter$new(c(1L, 2L, 3L))
/// it$r_next()        # 1L
/// it$r_next()        # 2L
/// it$r_size_hint()   # c(1, 1) - one element remaining
/// it$r_next()        # 3L
/// it$r_next()        # NULL (exhausted)
/// ```
///
/// # Design Note
///
/// Unlike other adapter traits, `RIterator` does NOT have a blanket impl
/// because iterators require `&mut self` for `next()`, but R's ExternalPtr
/// pattern typically provides `&self`. Users must implement this trait
/// manually using interior mutability (RefCell, Mutex, etc.).
pub trait RIterator {
    /// The type of elements yielded by this iterator.
    type Item;

    /// Get the next element from the iterator.
    ///
    /// Returns `Some(item)` if there are more elements, `None` if exhausted.
    /// None maps to NULL in R.
    fn r_next(&self) -> Option<Self::Item>;

    /// Get the estimated number of remaining elements.
    ///
    /// Returns `(lower_bound, upper_bound)` where upper_bound is None if unknown.
    /// In R, this becomes `c(lower, upper)` where upper is NA if unknown.
    fn r_size_hint(&self) -> (i64, Option<i64>);

    /// Consume the iterator and count remaining elements.
    ///
    /// **Warning:** This exhausts the iterator.
    fn r_count(&self) -> i64 {
        let mut count = 0i64;
        while self.r_next().is_some() {
            count += 1;
        }
        count
    }

    /// Collect up to `n` elements into a vector.
    ///
    /// Returns fewer than `n` elements if the iterator is exhausted first.
    fn r_collect_n(&self, n: i32) -> Vec<Self::Item> {
        let mut result = Vec::with_capacity(n.max(0) as usize);
        for _ in 0..n {
            match self.r_next() {
                Some(item) => result.push(item),
                None => break,
            }
        }
        result
    }

    /// Skip `n` elements from the iterator.
    ///
    /// Returns the number of elements actually skipped (may be less than `n`
    /// if the iterator is exhausted).
    fn r_skip(&self, n: i32) -> i32 {
        let mut skipped = 0i32;
        for _ in 0..n {
            if self.r_next().is_none() {
                break;
            }
            skipped += 1;
        }
        skipped
    }

    /// Get the `n`th element (0-indexed), consuming elements up to and including it.
    ///
    /// Returns None if the iterator has fewer than `n + 1` elements.
    fn r_nth(&self, n: i32) -> Option<Self::Item> {
        if n < 0 {
            return None;
        }
        for _ in 0..n {
            if self.r_next().is_none() {
                return None;
            }
        }
        self.r_next()
    }
}

// Note: No blanket impl because Iterator::next() requires &mut self,
// but ExternalPtr methods receive &self. Users must use interior mutability.

/// Adapter trait for [`std::iter::Extend`].
///
/// Provides collection extension operations for R, allowing Rust collections
/// to be extended with R vectors. Since extension requires mutation, the
/// wrapper type should use interior mutability (e.g., `RefCell`).
///
/// # Methods
///
/// - `r_extend_from_vec(items)` - Extend the collection with items from a vector
/// - `r_extend_from_slice(items)` - Extend from a slice (for Clone items)
///
/// # Example
///
/// ```rust,ignore
/// use std::cell::RefCell;
///
/// #[derive(ExternalPtr)]
/// struct MyVec(RefCell<Vec<i32>>);
///
/// impl MyVec {
///     fn new() -> Self {
///         Self(RefCell::new(Vec::new()))
///     }
/// }
///
/// impl RExtend<i32> for MyVec {
///     fn r_extend_from_vec(&self, items: Vec<i32>) {
///         self.0.borrow_mut().extend(items);
///     }
/// }
///
/// #[miniextendr]
/// impl RExtend<i32> for MyVec {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RExtend<i32> for MyVec;
/// }
/// ```
///
/// In R:
/// ```r
/// v <- MyVec$new()
/// v$r_extend_from_vec(c(1L, 2L, 3L))  # Add items
/// v$r_extend_from_vec(c(4L, 5L))      # Add more items
/// ```
///
/// # Design Note
///
/// Like `RIterator`, `RExtend` does NOT have a blanket impl because `Extend::extend()`
/// requires `&mut self`, but R's ExternalPtr pattern provides `&self`. Users must
/// implement this trait manually using interior mutability (RefCell, Mutex, etc.).
pub trait RExtend<T> {
    /// Extend the collection with items from a vector.
    ///
    /// The items are moved into the collection.
    fn r_extend_from_vec(&self, items: Vec<T>);

    /// Extend the collection with cloned items from a slice.
    ///
    /// Default implementation clones items into a Vec and calls `r_extend_from_vec`.
    fn r_extend_from_slice(&self, items: &[T])
    where
        T: Clone,
    {
        self.r_extend_from_vec(items.to_vec());
    }

    /// Get the current length of the collection.
    ///
    /// Optional - returns -1 if not implemented.
    fn r_len(&self) -> i64 {
        -1 // Indicates "unknown" - implementers can override
    }
}

// Note: No blanket impl because Extend::extend() requires &mut self,
// but ExternalPtr methods receive &self. Users must use interior mutability.

/// Adapter trait for [`std::iter::FromIterator`].
///
/// Provides collection construction from iterators/vectors for R.
/// Unlike `RExtend`, this creates a new collection from items.
///
/// # Methods
///
/// - `r_from_vec(items)` - Create a new collection from a vector
///
/// # Example
///
/// ```rust,ignore
/// #[derive(ExternalPtr)]
/// struct MySet(std::collections::HashSet<i32>);
///
/// impl RFromIter<i32> for MySet {
///     fn r_from_vec(items: Vec<i32>) -> Self {
///         Self(items.into_iter().collect())
///     }
/// }
///
/// #[miniextendr]
/// impl RFromIter<i32> for MySet {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RFromIter<i32> for MySet;
/// }
/// ```
///
/// In R:
/// ```r
/// set <- MySet$r_from_vec(c(1L, 2L, 2L, 3L))  # Creates {1, 2, 3}
/// ```
pub trait RFromIter<T>: Sized {
    /// Create a new collection from a vector of items.
    fn r_from_vec(items: Vec<T>) -> Self;
}

impl<C, T> RFromIter<T> for C
where
    C: FromIterator<T>,
{
    fn r_from_vec(items: Vec<T>) -> Self {
        items.into_iter().collect()
    }
}

/// Adapter trait for collections that can be converted to vectors.
///
/// This is the complement to [`RFromIter`]: while `RFromIter` creates collections
/// from vectors, `RToVec` extracts vectors from collections.
///
/// # Methods
///
/// - `r_to_vec()` - Collect all elements into a vector (cloning elements)
/// - `r_len()` - Get the number of elements
/// - `r_is_empty()` - Check if the collection is empty
///
/// # Design Note
///
/// Unlike Rust's `IntoIterator::into_iter()` which consumes the collection,
/// this trait borrows the collection and clones elements. This is necessary
/// because R's `ExternalPtr` pattern provides `&self`, not owned `self`.
///
/// For consuming iteration, use [`RIterator`] with interior mutability.
///
/// # Example
///
/// ```rust,ignore
/// use std::collections::HashSet;
///
/// #[derive(ExternalPtr)]
/// struct MySet(HashSet<i32>);
///
/// // RToVec is automatically available via blanket impl
/// #[miniextendr]
/// impl RToVec<i32> for MySet {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RToVec<i32> for MySet;
/// }
/// ```
///
/// In R:
/// ```r
/// set <- MySet$new(...)
/// vec <- set$r_to_vec()    # Get all elements as vector
/// set$r_len()              # Number of elements
/// set$r_is_empty()         # Check if empty
/// ```
pub trait RToVec<T> {
    /// Collect all elements into a vector.
    ///
    /// Elements are cloned from the collection.
    fn r_to_vec(&self) -> Vec<T>;

    /// Get the number of elements in the collection.
    fn r_len(&self) -> i64;

    /// Check if the collection is empty.
    fn r_is_empty(&self) -> bool {
        self.r_len() == 0
    }
}

// Blanket impl for any collection where:
// - &C can be iterated over (yielding &T references)
// - T: Clone (so we can clone elements into the Vec)
// - The iterator knows its exact size
//
// Note: Using HRTB (higher-ranked trait bounds) to express that &C
// can be iterated for any lifetime.
impl<C, T> RToVec<T> for C
where
    T: Clone,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    for<'a> <&'a C as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn r_to_vec(&self) -> Vec<T> {
        self.into_iter().cloned().collect()
    }

    fn r_len(&self) -> i64 {
        self.into_iter().len() as i64
    }
}

/// Adapter trait for creating iterator wrappers from collections.
///
/// This trait provides a way to create an [`RIterator`] wrapper from a collection.
/// Since `ExternalPtr` methods receive `&self`, this trait clones the underlying
/// data to create an independent iterator.
///
/// # Type Parameters
///
/// - `T`: The element type yielded by the iterator
/// - `I`: The iterator type returned (must implement [`RIterator`])
///
/// # Design Note
///
/// The returned iterator is independent from the source collection. Modifications
/// to the original collection after calling `r_make_iter()` won't affect the
/// iterator's output.
///
/// # Example
///
/// ```rust,ignore
/// use std::cell::RefCell;
///
/// #[derive(ExternalPtr)]
/// struct MyVec(Vec<i32>);
///
/// #[derive(ExternalPtr)]
/// struct MyVecIter(RefCell<std::vec::IntoIter<i32>>);
///
/// impl RIterator for MyVecIter {
///     type Item = i32;
///     fn r_next(&self) -> Option<i32> {
///         self.0.borrow_mut().next()
///     }
///     fn r_size_hint(&self) -> (i64, Option<i64>) {
///         let (lo, hi) = self.0.borrow().size_hint();
///         (lo as i64, hi.map(|h| h as i64))
///     }
/// }
///
/// impl RMakeIter<i32, MyVecIter> for MyVec {
///     fn r_make_iter(&self) -> MyVecIter {
///         MyVecIter(RefCell::new(self.0.clone().into_iter()))
///     }
/// }
///
/// #[miniextendr]
/// impl RMakeIter<i32, MyVecIter> for MyVec {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RMakeIter<i32, MyVecIter> for MyVec;
/// }
/// ```
///
/// In R:
/// ```r
/// v <- MyVec$new(c(1L, 2L, 3L))
/// it <- v$r_make_iter()   # Create iterator
/// it$r_next()             # 1L
/// it$r_next()             # 2L
/// v$r_to_vec()            # c(1L, 2L, 3L) - original unchanged
/// ```
pub trait RMakeIter<T, I>
where
    I: RIterator<Item = T>,
{
    /// Create a new iterator wrapper.
    ///
    /// The iterator is independent from this collection (typically by cloning
    /// the underlying data).
    fn r_make_iter(&self) -> I;
}

// Note: No blanket impl because:
// 1. The iterator type I must be a concrete type that implements RIterator
// 2. RIterator requires interior mutability (RefCell/Mutex)
// 3. Users must define their own iterator wrapper type

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

    // Tests for RIterator
    use std::cell::RefCell;

    /// Test iterator wrapper using RefCell for interior mutability.
    struct TestIter(RefCell<std::vec::IntoIter<i32>>);

    impl TestIter {
        fn new(data: Vec<i32>) -> Self {
            Self(RefCell::new(data.into_iter()))
        }
    }

    impl RIterator for TestIter {
        type Item = i32;

        fn r_next(&self) -> Option<Self::Item> {
            self.0.borrow_mut().next()
        }

        fn r_size_hint(&self) -> (i64, Option<i64>) {
            let (lo, hi) = self.0.borrow().size_hint();
            (lo as i64, hi.map(|h| h as i64))
        }
    }

    #[test]
    fn test_riterator_next() {
        let it = TestIter::new(vec![1, 2, 3]);
        assert_eq!(it.r_next(), Some(1));
        assert_eq!(it.r_next(), Some(2));
        assert_eq!(it.r_next(), Some(3));
        assert_eq!(it.r_next(), None);
        assert_eq!(it.r_next(), None); // Stays exhausted
    }

    #[test]
    fn test_riterator_size_hint() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(it.r_size_hint(), (5, Some(5)));
        it.r_next();
        assert_eq!(it.r_size_hint(), (4, Some(4)));
        it.r_next();
        it.r_next();
        assert_eq!(it.r_size_hint(), (2, Some(2)));
    }

    #[test]
    fn test_riterator_count() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(it.r_count(), 5);
        // Iterator is now exhausted
        assert_eq!(it.r_next(), None);
    }

    #[test]
    fn test_riterator_collect_n() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        let first_three = it.r_collect_n(3);
        assert_eq!(first_three, vec![1, 2, 3]);
        let remaining = it.r_collect_n(10); // Ask for more than available
        assert_eq!(remaining, vec![4, 5]);
    }

    #[test]
    fn test_riterator_skip() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        let skipped = it.r_skip(2);
        assert_eq!(skipped, 2);
        assert_eq!(it.r_next(), Some(3));

        // Skip more than remaining
        let skipped = it.r_skip(10);
        assert_eq!(skipped, 2); // Only 2 elements were left
    }

    #[test]
    fn test_riterator_nth() {
        let it = TestIter::new(vec![10, 20, 30, 40, 50]);
        // Get element at index 2 (third element)
        assert_eq!(it.r_nth(2), Some(30));
        // Iterator has consumed 0, 1, 2 - next is index 3
        assert_eq!(it.r_next(), Some(40));

        // Negative index returns None
        let it2 = TestIter::new(vec![1, 2, 3]);
        assert_eq!(it2.r_nth(-1), None);
    }

    #[test]
    fn test_riterator_empty() {
        let it = TestIter::new(vec![]);
        assert_eq!(it.r_next(), None);
        assert_eq!(it.r_size_hint(), (0, Some(0)));
        assert_eq!(it.r_count(), 0);
        assert_eq!(it.r_collect_n(5), Vec::<i32>::new());
        assert_eq!(it.r_skip(5), 0);
        assert_eq!(it.r_nth(0), None);
    }

    // Tests for RExtend
    struct TestExtendVec(RefCell<Vec<i32>>);

    impl TestExtendVec {
        fn new() -> Self {
            Self(RefCell::new(Vec::new()))
        }

        fn get(&self) -> Vec<i32> {
            self.0.borrow().clone()
        }
    }

    impl RExtend<i32> for TestExtendVec {
        fn r_extend_from_vec(&self, items: Vec<i32>) {
            self.0.borrow_mut().extend(items);
        }

        fn r_len(&self) -> i64 {
            self.0.borrow().len() as i64
        }
    }

    #[test]
    fn test_rextend_basic() {
        let v = TestExtendVec::new();
        assert_eq!(v.get(), Vec::<i32>::new());
        assert_eq!(v.r_len(), 0);

        v.r_extend_from_vec(vec![1, 2, 3]);
        assert_eq!(v.get(), vec![1, 2, 3]);
        assert_eq!(v.r_len(), 3);

        v.r_extend_from_vec(vec![4, 5]);
        assert_eq!(v.get(), vec![1, 2, 3, 4, 5]);
        assert_eq!(v.r_len(), 5);
    }

    #[test]
    fn test_rextend_empty() {
        let v = TestExtendVec::new();
        v.r_extend_from_vec(vec![]);
        assert_eq!(v.get(), Vec::<i32>::new());
        assert_eq!(v.r_len(), 0);
    }

    #[test]
    fn test_rextend_from_slice() {
        let v = TestExtendVec::new();
        let data = [1, 2, 3];
        v.r_extend_from_slice(&data);
        assert_eq!(v.get(), vec![1, 2, 3]);
    }

    // Tests for RFromIter
    #[test]
    fn test_rfromiter_vec() {
        let v: Vec<i32> = RFromIter::r_from_vec(vec![1, 2, 3]);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_rfromiter_hashset() {
        use std::collections::HashSet;
        let set: HashSet<i32> = RFromIter::r_from_vec(vec![1, 2, 2, 3, 3, 3]);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
    }

    #[test]
    fn test_rfromiter_string() {
        let s: String = RFromIter::r_from_vec(vec!['h', 'e', 'l', 'l', 'o']);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_rfromiter_empty() {
        let v: Vec<i32> = RFromIter::r_from_vec(vec![]);
        assert!(v.is_empty());
    }

    // Tests for RToVec
    #[test]
    fn test_rtovec_vec() {
        let v = vec![1, 2, 3];
        let collected: Vec<i32> = RToVec::r_to_vec(&v);
        assert_eq!(collected, vec![1, 2, 3]);
        assert_eq!(RToVec::<i32>::r_len(&v), 3);
        assert!(!RToVec::<i32>::r_is_empty(&v));
    }

    #[test]
    fn test_rtovec_empty() {
        let v: Vec<i32> = vec![];
        let collected: Vec<i32> = RToVec::r_to_vec(&v);
        assert!(collected.is_empty());
        assert_eq!(RToVec::<i32>::r_len(&v), 0);
        assert!(RToVec::<i32>::r_is_empty(&v));
    }

    #[test]
    fn test_rtovec_hashset() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let mut collected: Vec<i32> = RToVec::r_to_vec(&set);
        collected.sort();
        assert_eq!(collected, vec![1, 2, 3]);
        assert_eq!(RToVec::<i32>::r_len(&set), 3);
    }

    #[test]
    fn test_rtovec_slice() {
        let arr = [10, 20, 30];
        let collected: Vec<i32> = RToVec::r_to_vec(&arr);
        assert_eq!(collected, vec![10, 20, 30]);
        assert_eq!(RToVec::<i32>::r_len(&arr), 3);
    }

    // Tests for RMakeIter
    struct TestCollection(Vec<i32>);

    struct TestCollectionIter(RefCell<std::vec::IntoIter<i32>>);

    impl RIterator for TestCollectionIter {
        type Item = i32;

        fn r_next(&self) -> Option<i32> {
            self.0.borrow_mut().next()
        }

        fn r_size_hint(&self) -> (i64, Option<i64>) {
            let (lo, hi) = self.0.borrow().size_hint();
            (lo as i64, hi.map(|h| h as i64))
        }
    }

    impl RMakeIter<i32, TestCollectionIter> for TestCollection {
        fn r_make_iter(&self) -> TestCollectionIter {
            TestCollectionIter(RefCell::new(self.0.clone().into_iter()))
        }
    }

    #[test]
    fn test_rmakeiter_basic() {
        let coll = TestCollection(vec![1, 2, 3]);
        let iter = coll.r_make_iter();

        assert_eq!(iter.r_next(), Some(1));
        assert_eq!(iter.r_next(), Some(2));
        assert_eq!(iter.r_next(), Some(3));
        assert_eq!(iter.r_next(), None);
    }

    #[test]
    fn test_rmakeiter_independent() {
        let coll = TestCollection(vec![1, 2, 3]);

        // Create two independent iterators
        let iter1 = coll.r_make_iter();
        let iter2 = coll.r_make_iter();

        // Consuming one doesn't affect the other
        assert_eq!(iter1.r_next(), Some(1));
        assert_eq!(iter1.r_next(), Some(2));

        assert_eq!(iter2.r_next(), Some(1)); // iter2 starts fresh
        assert_eq!(iter2.r_size_hint(), (2, Some(2))); // 2 remaining in iter2
    }
}
