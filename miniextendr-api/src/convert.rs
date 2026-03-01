//! Wrapper helpers to force specific `IntoR` representations.
//!
//! This module provides two approaches for controlling how Rust types are converted to R:
//!
//! ## 1. `As*` Wrappers (Call-site Control)
//!
//! Use these wrappers when you want to override the conversion for a single return value:
//!
//! - [`AsList<T>`]: Convert `T` to an R list via [`IntoList`]
//! - [`AsExternalPtr<T>`]: Convert `T` to an R external pointer
//! - [`AsRNative<T>`]: Convert scalar `T` to a length-1 R vector
//!
//! ```ignore
//! #[miniextendr]
//! fn get_data() -> AsList<MyStruct> {
//!     AsList(MyStruct { x: 1, y: 2 })
//! }
//! ```
//!
//! ## 2. `Prefer*` Derive Macros (Type-level Control)
//!
//! Use these derives when a type should *always* use a specific conversion:
//!
//! - `#[derive(IntoList, PreferList)]`: Type always converts to R list
//! - `#[derive(ExternalPtr, PreferExternalPtr)]`: Type always converts to external pointer
//! - `#[derive(RNativeType, PreferRNativeType)]`: Newtype always converts to native R scalar
//!
//! ```ignore
//! #[derive(IntoList, PreferList)]
//! struct Point { x: f64, y: f64 }
//!
//! #[miniextendr]
//! fn make_point() -> Point {  // Automatically becomes R list
//!     Point { x: 1.0, y: 2.0 }
//! }
//! ```
//!
//! ## 3. `#[miniextendr(return = "...")]` Attribute
//!
//! Use this when you want to control conversion for a specific `#[miniextendr]` function
//! without modifying the type itself:
//!
//! - `return = "list"`: Wrap result in `AsList`
//! - `return = "externalptr"`: Wrap result in `AsExternalPtr`
//! - `return = "native"`: Wrap result in `AsRNative`
//!
//! ```ignore
//! #[miniextendr(return = "list")]
//! fn get_as_list() -> MyStruct {
//!     MyStruct { x: 1 }
//! }
//! ```
//!
//! ## Choosing the Right Approach
//!
//! | Situation | Recommended Approach |
//! |-----------|---------------------|
//! | Type should *always* convert one way | `Prefer*` derive |
//! | Override conversion for one function | `As*` wrapper or `return` attribute |
//! | Type has multiple valid representations | Don't use `Prefer*`; use `As*` or `return` |

use crate::externalptr::{ExternalPtr, IntoExternalPtr};
use crate::ffi::RNativeType;
use crate::into_r::IntoR;
use crate::list::{IntoList, List};
use crate::named_vector::AtomicElement;

/// Wrap a value and convert it to an R list via [`IntoList`] when returned from Rust.
///
/// Use this wrapper when you want to convert a single value to an R list without
/// making that the default behavior for the type.
///
/// # Example
///
/// ```ignore
/// #[derive(IntoList)]
/// struct Point { x: f64, y: f64 }
///
/// #[miniextendr]
/// fn make_point() -> AsList<Point> {
///     AsList(Point { x: 1.0, y: 2.0 })
/// }
/// // In R: make_point() returns list(x = 1.0, y = 2.0)
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AsList<T: IntoList>(pub T);

impl<T: IntoList> From<T> for AsList<T> {
    fn from(value: T) -> Self {
        AsList(value)
    }
}

impl<T: IntoList> IntoR for AsList<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.0.into_list().into_sexp()
    }
}

/// Wrap a value and convert it to an R data.frame via [`IntoDataFrame`] when returned from Rust.
///
/// Use this wrapper when you want to convert a single value to an R data.frame without
/// making that the default behavior for the type.
///
/// # Example
///
/// ```ignore
/// struct TimeSeries {
///     timestamps: Vec<f64>,
///     values: Vec<f64>,
/// }
///
/// impl IntoDataFrame for TimeSeries {
///     fn into_data_frame(self) -> List {
///         List::from_pairs(vec![
///             ("timestamp", self.timestamps),
///             ("value", self.values),
///         ])
///         .set_class_str(&["data.frame"])
///         .set_row_names_int(self.timestamps.len())
///     }
/// }
///
/// #[miniextendr]
/// fn make_time_series() -> ToDataFrame<TimeSeries> {
///     ToDataFrame(TimeSeries {
///         timestamps: vec![1.0, 2.0, 3.0],
///         values: vec![10.0, 20.0, 30.0],
///     })
/// }
/// // In R: make_time_series() returns a data.frame
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ToDataFrame<T: IntoDataFrame>(pub T);

impl<T: IntoDataFrame> From<T> for ToDataFrame<T> {
    fn from(value: T) -> Self {
        ToDataFrame(value)
    }
}

impl<T: IntoDataFrame> IntoR for ToDataFrame<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.0.into_data_frame().into_sexp()
    }
}

/// IntoR implementation for DataFrame.
///
/// This allows DataFrame to be returned directly from `#[miniextendr]` functions.
impl<T: IntoList> IntoR for DataFrame<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.into_data_frame().into_sexp()
    }
}

/// Trait for types that can be converted into R data frames.
///
/// This trait allows Rust types to define how they convert to R data frames.
/// Use with [`ToDataFrame`] wrapper or `#[derive(PreferDataFrame)]` to enable
/// automatic conversion.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::IntoDataFrame;
/// use miniextendr_api::List;
///
/// struct TimeSeries {
///     timestamps: Vec<f64>,
///     values: Vec<f64>,
/// }
///
/// impl IntoDataFrame for TimeSeries {
///     fn into_data_frame(self) -> List {
///         List::from_pairs(vec![
///             ("timestamp", self.timestamps),
///             ("value", self.values),
///         ])
///         .set_class_str(&["data.frame"])
///         .set_row_names_int(self.timestamps.len())
///     }
/// }
/// ```
///
/// # Comparison with `AsDataFrame` coercion trait
///
/// - [`AsDataFrame`](crate::as_coerce::AsDataFrame): Used with `#[miniextendr(as = "data.frame")]`
///   to generate S3 methods for `as.data.frame()` on external pointer types
/// - `IntoDataFrame`: Used for direct conversion when returning from functions
///
/// Both return a `List` with appropriate data.frame attributes, but serve different purposes:
/// - S3 `AsDataFrame` is for coercion methods on existing objects (`&self`)
/// - `IntoDataFrame` is for consuming conversion (`self`) when returning from functions
pub trait IntoDataFrame {
    /// Convert this value into an R data.frame.
    ///
    /// The returned List should have:
    /// - Named columns of equal length
    /// - Class attribute set to "data.frame"
    /// - row.names attribute set appropriately
    ///
    /// # Example
    ///
    /// ```ignore
    /// impl IntoDataFrame for MyStruct {
    ///     fn into_data_frame(self) -> List {
    ///         List::from_pairs(vec![
    ///             ("col1", self.field1),
    ///             ("col2", self.field2),
    ///         ])
    ///         .set_class_str(&["data.frame"])
    ///         .set_row_names_int(self.field1.len())
    ///     }
    /// }
    /// ```
    fn into_data_frame(self) -> List;
}

// =============================================================================
// Serde Row Wrapper
// =============================================================================

/// Wrap a serde-serializable value for use as a data frame row.
///
/// This wrapper implements [`IntoList`] via serde serialization, allowing
/// types that implement `serde::Serialize` to be used with [`DataFrame`]
/// without manually implementing [`IntoList`].
///
/// # Feature Flag
///
/// Requires the `serde` feature to be enabled.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, convert::{AsSerializeRow, DataFrame}};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Measurement {
///     time: f64,
///     value: f64,
/// }
///
/// #[miniextendr]
/// fn get_data() -> DataFrame<AsSerializeRow<Measurement>> {
///     DataFrame::from_rows(vec![
///         AsSerializeRow(Measurement { time: 1.0, value: 10.0 }),
///         AsSerializeRow(Measurement { time: 2.0, value: 20.0 }),
///     ])
/// }
/// ```
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Copy)]
pub struct AsSerializeRow<T: serde::Serialize>(pub T);

#[cfg(feature = "serde")]
impl<T: serde::Serialize> From<T> for AsSerializeRow<T> {
    fn from(value: T) -> Self {
        AsSerializeRow(value)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> IntoList for AsSerializeRow<T> {
    fn into_list(self) -> List {
        use crate::ffi::{SEXPTYPE, TYPEOF};
        use crate::serde::RSerializer;
        match RSerializer::to_sexp(&self.0) {
            Ok(sexp) => {
                if unsafe { TYPEOF(sexp) } as SEXPTYPE == SEXPTYPE::VECSXP {
                    unsafe { List::from_raw(sexp) }
                } else {
                    // Non-list SEXP (e.g., scalar) — wrap in a single-element list
                    List::from_raw_values(vec![sexp])
                }
            }
            Err(e) => {
                crate::r_error!("AsSerializeRow: serde serialization failed: {e}");
            }
        }
    }
}

/// Type alias for a [`DataFrame`] of serde-serializable rows.
///
/// This is equivalent to `DataFrame<AsSerializeRow<T>>` but more concise.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, SerializeDataFrame};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: i32,
/// }
///
/// #[miniextendr]
/// fn get_people() -> SerializeDataFrame<Person> {
///     let people = vec![
///         Person { name: "Alice".into(), age: 30 },
///         Person { name: "Bob".into(), age: 25 },
///     ];
///     SerializeDataFrame::from_serialize(people)
/// }
/// ```
#[cfg(feature = "serde")]
pub type SerializeDataFrame<T> = DataFrame<AsSerializeRow<T>>;

// =============================================================================
// Data Frame Row Conversion
// =============================================================================

/// Convert row-oriented data into a column-oriented R data.frame.
///
/// This type collects a sequence of row elements (structs implementing [`IntoList`])
/// and transposes them into column vectors suitable for creating an R data.frame.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, convert::DataFrame};
///
/// #[derive(IntoList)]
/// struct Person {
///     name: String,
///     age: i32,
///     height: f64,
/// }
///
/// #[miniextendr]
/// fn make_people() -> DataFrame<Person> {
///     DataFrame::from_rows(vec![
///         Person { name: "Alice".into(), age: 30, height: 165.0 },
///         Person { name: "Bob".into(), age: 25, height: 180.0 },
///         Person { name: "Carol".into(), age: 35, height: 170.0 },
///     ])
/// }
/// // In R: make_people() returns a data.frame with 3 rows and columns: name, age, height
/// ```
///
/// # Row-oriented to Column-oriented
///
/// R data frames are column-oriented (each column is a vector), but data is often
/// produced row-by-row in Rust. `DataFrame` handles the transposition:
///
/// ```text
/// Input (row-oriented):           Output (column-oriented):
/// Row 1: {name: "A", age: 30}     name column:  ["A", "B", "C"]
/// Row 2: {name: "B", age: 25}  →  age column:   [30, 25, 35]
/// Row 3: {name: "C", age: 35}
/// ```
#[derive(Debug, Clone)]
pub struct DataFrame<T: IntoList> {
    rows: Vec<T>,
}

impl<T: IntoList> DataFrame<T> {
    /// Create a new `DataFrame` from a vector of row elements.
    pub fn from_rows(rows: Vec<T>) -> Self {
        Self { rows }
    }

    /// Create an empty `DataFrame`.
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Add a row to the data frame.
    pub fn push(&mut self, row: T) {
        self.rows.push(row);
    }

    /// Get the number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> DataFrame<AsSerializeRow<T>> {
    /// Create a DataFrame from serde-serializable rows.
    ///
    /// This is a convenience method that wraps each row in [`AsSerializeRow`]
    /// automatically, avoiding the need to manually map over the input.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use miniextendr_api::DataFrame;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Person { name: String, age: i32 }
    ///
    /// let people = vec![
    ///     Person { name: "Alice".into(), age: 30 },
    ///     Person { name: "Bob".into(), age: 25 },
    /// ];
    ///
    /// // Instead of:
    /// // DataFrame::from_iter(people.into_iter().map(AsSerializeRow))
    ///
    /// // Just write:
    /// let df = DataFrame::from_serialize(people);
    /// ```
    pub fn from_serialize(rows: impl IntoIterator<Item = T>) -> Self {
        Self::from_iter(rows.into_iter().map(AsSerializeRow))
    }
}

impl<T: IntoList> Default for DataFrame<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IntoList> FromIterator<T> for DataFrame<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            rows: iter.into_iter().collect(),
        }
    }
}

impl<T: IntoList> IntoDataFrame for DataFrame<T> {
    fn into_data_frame(self) -> List {
        if self.rows.is_empty() {
            // Empty data frame
            return List::from_raw_pairs(Vec::<(&str, crate::ffi::SEXP)>::new())
                .set_class_str(&["data.frame"])
                .set_row_names_int(0);
        }

        let mut n_protect: i32 = 0;

        // Convert all rows to lists, protecting each from GC.
        let lists: Vec<List> = self
            .rows
            .into_iter()
            .map(|row| {
                let list = row.into_list();
                unsafe { crate::ffi::Rf_protect(list.as_sexp()) };
                n_protect += 1;
                list
            })
            .collect();
        let n_rows = lists.len() as isize;

        // Get column names from the first row
        let first_names_sexp = lists[0].names();
        if first_names_sexp.is_none() {
            unsafe { crate::ffi::Rf_unprotect(n_protect) };
            crate::r_error!("cannot create data frame from unnamed list elements");
        }

        // Extract column names as Vec<String>
        let names_sexp = first_names_sexp.unwrap();
        let n_cols = unsafe { crate::ffi::Rf_xlength(names_sexp) };
        let mut col_names = Vec::with_capacity(n_cols as usize);
        for i in 0..n_cols {
            unsafe {
                let name_sexp = crate::ffi::STRING_ELT(names_sexp, i);
                let name_ptr = crate::ffi::R_CHAR(name_sexp);
                let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                if let Ok(s) = name_cstr.to_str() {
                    col_names.push(s.to_string());
                }
            }
        }

        // Transpose: collect values by column.
        // Element SEXPs from get_named are children of protected row lists,
        // so they don't need individual protection.
        use std::collections::HashMap;
        let mut columns: HashMap<String, Vec<crate::ffi::SEXP>> =
            HashMap::with_capacity(col_names.len());
        for name in &col_names {
            columns.insert(name.clone(), Vec::with_capacity(n_rows as usize));
        }

        for list in &lists {
            for name in &col_names {
                let value = list
                    .get_named::<crate::ffi::SEXP>(name)
                    .unwrap_or(unsafe { crate::ffi::R_NilValue });
                columns.get_mut(name).unwrap().push(value);
            }
        }

        // Build column vectors, protecting each from GC.
        let mut df_pairs: Vec<(String, crate::ffi::SEXP)> = Vec::with_capacity(col_names.len());
        for name in col_names {
            let col_values = columns.remove(&name).unwrap();
            let col_sexp = List::from_raw_values(col_values).as_sexp();
            unsafe { crate::ffi::Rf_protect(col_sexp) };
            n_protect += 1;
            df_pairs.push((name, col_sexp));
        }

        let result = List::from_raw_pairs(df_pairs)
            .set_class_str(&["data.frame"])
            .set_row_names_int(n_rows as usize);
        unsafe { crate::ffi::Rf_unprotect(n_protect) };
        result
    }
}

/// Wrap a value and convert it to an R external pointer when returned from Rust.
///
/// Use this wrapper when you want to return a Rust value as an opaque pointer
/// that R code can pass back to Rust functions later.
///
/// # Example
///
/// ```ignore
/// struct Connection { handle: u64 }
///
/// impl IntoExternalPtr for Connection { /* ... */ }
///
/// #[miniextendr]
/// fn open_connection(path: &str) -> AsExternalPtr<Connection> {
///     AsExternalPtr(Connection { handle: 42 })
/// }
/// // In R: open_connection("foo") returns an external pointer
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AsExternalPtr<T: IntoExternalPtr>(pub T);

impl<T: IntoExternalPtr> From<T> for AsExternalPtr<T> {
    fn from(value: T) -> Self {
        AsExternalPtr(value)
    }
}

impl<T: IntoExternalPtr> IntoR for AsExternalPtr<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        ExternalPtr::new(self.0).into_sexp()
    }
}

/// Wrap a scalar [`RNativeType`] and force native R vector conversion.
///
/// This creates a length-1 R vector containing the scalar value. Use this when
/// you want to ensure a value is converted to its native R representation (e.g.,
/// `i32` → integer vector, `f64` → numeric vector) rather than another path
/// like `IntoExternalPtr`.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, Copy, RNativeType)]
/// struct Meters(f64);
///
/// #[miniextendr]
/// fn distance() -> AsRNative<Meters> {
///     AsRNative(Meters(42.5))
/// }
/// // In R: distance() returns 42.5 (numeric vector of length 1)
/// ```
///
/// # Performance
///
/// This wrapper directly allocates an R vector and writes the value,
/// avoiding intermediate Rust allocations.
#[derive(Debug, Clone, Copy)]
pub struct AsRNative<T: RNativeType>(pub T);

impl<T: RNativeType> From<T> for AsRNative<T> {
    fn from(value: T) -> Self {
        AsRNative(value)
    }
}

impl<T: RNativeType> IntoR for AsRNative<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }

    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // Directly allocate a length-1 R vector and write the scalar value.
        // This avoids the intermediate Rust Vec allocation.
        unsafe {
            let sexp = crate::ffi::Rf_allocVector(T::SEXP_TYPE, 1);
            let ptr = T::dataptr_mut(sexp);
            std::ptr::write(ptr, self.0);
            sexp
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let sexp = crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, 1);
            let ptr = T::dataptr_mut(sexp);
            std::ptr::write(ptr, self.0);
            sexp
        }
    }
}

// =============================================================================
// Named pair wrappers
// =============================================================================

/// Wrap a tuple pair collection and convert it to a **named R list** (VECSXP).
///
/// Preserves insertion order and allows duplicate names (sequence semantics).
///
/// # Supported input types
///
/// | Input | Bounds |
/// |-------|--------|
/// | `Vec<(K, V)>` | `K: AsRef<str>`, `V: IntoR` |
/// | `[(K, V); N]` | `K: AsRef<str>`, `V: IntoR` |
/// | `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + IntoR` |
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn make_config() -> AsNamedList<Vec<(String, i32)>> {
///     AsNamedList(vec![
///         ("width".into(), 100),
///         ("height".into(), 200),
///     ])
/// }
/// // In R: make_config() returns list(width = 100L, height = 200L)
/// ```
#[derive(Debug, Clone)]
pub struct AsNamedList<T>(pub T);

impl<T> From<T> for AsNamedList<T> {
    fn from(value: T) -> Self {
        AsNamedList(value)
    }
}

impl<K: AsRef<str>, V: IntoR> IntoR for AsNamedList<Vec<(K, V)>> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::ffi::SEXP {
        let pairs: Vec<(K, crate::ffi::SEXP)> = self
            .0
            .into_iter()
            .map(|(k, v)| (k, v.into_sexp()))
            .collect();
        List::from_raw_pairs(pairs).into_sexp()
    }
}

impl<K: AsRef<str>, V: IntoR, const N: usize> IntoR for AsNamedList<[(K, V); N]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::ffi::SEXP {
        let pairs: Vec<(K, crate::ffi::SEXP)> = self
            .0
            .into_iter()
            .map(|(k, v)| (k, v.into_sexp()))
            .collect();
        List::from_raw_pairs(pairs).into_sexp()
    }
}

impl<K: AsRef<str>, V: Clone + IntoR> IntoR for AsNamedList<&[(K, V)]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::ffi::SEXP {
        let pairs: Vec<(&K, crate::ffi::SEXP)> = self
            .0
            .iter()
            .map(|(k, v)| (k, v.clone().into_sexp()))
            .collect();
        List::from_raw_pairs(pairs).into_sexp()
    }
}

/// Wrap a tuple pair collection and convert it to a **named atomic R vector**
/// (INTSXP, REALSXP, LGLSXP, RAWSXP, or STRSXP).
///
/// Preserves insertion order and allows duplicate names (sequence semantics).
/// Values must be homogeneous and implement [`AtomicElement`].
///
/// # Supported input types
///
/// | Input | Bounds |
/// |-------|--------|
/// | `Vec<(K, V)>` | `K: AsRef<str>`, `V: AtomicElement` |
/// | `[(K, V); N]` | `K: AsRef<str>`, `V: AtomicElement` |
/// | `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + AtomicElement` |
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn make_scores() -> AsNamedVector<Vec<(&str, f64)>> {
///     AsNamedVector(vec![("alice", 95.0), ("bob", 87.5)])
/// }
/// // In R: make_scores() returns c(alice = 95.0, bob = 87.5)
/// ```
#[derive(Debug, Clone)]
pub struct AsNamedVector<T>(pub T);

impl<T> From<T> for AsNamedVector<T> {
    fn from(value: T) -> Self {
        AsNamedVector(value)
    }
}

impl<K: AsRef<str>, V: AtomicElement> IntoR for AsNamedVector<Vec<(K, V)>> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::ffi::SEXP {
        named_vector_from_pairs(self.0)
    }
}

impl<K: AsRef<str>, V: AtomicElement, const N: usize> IntoR for AsNamedVector<[(K, V); N]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::ffi::SEXP {
        named_vector_from_pairs(self.0)
    }
}

impl<K: AsRef<str>, V: Clone + AtomicElement> IntoR for AsNamedVector<&[(K, V)]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::ffi::SEXP {
        let (keys, values): (Vec<&K>, Vec<V>) = self.0.iter().map(|(k, v)| (k, v.clone())).unzip();
        let sexp = V::vec_to_sexp(values);
        unsafe {
            crate::ffi::Rf_protect(sexp);
            crate::named_vector::set_names_on_sexp(sexp, &keys);
            crate::ffi::Rf_unprotect(1);
        }
        sexp
    }
}

/// Shared helper: build a named atomic vector from an owning iterator of (key, value) pairs.
fn named_vector_from_pairs<K, V>(pairs: impl IntoIterator<Item = (K, V)>) -> crate::ffi::SEXP
where
    K: AsRef<str>,
    V: AtomicElement,
{
    let (keys, values): (Vec<K>, Vec<V>) = pairs.into_iter().unzip();
    let sexp = V::vec_to_sexp(values);
    unsafe {
        crate::ffi::Rf_protect(sexp);
        crate::named_vector::set_names_on_sexp(sexp, &keys);
        crate::ffi::Rf_unprotect(1);
    }
    sexp
}

// =============================================================================
// Extension traits for ergonomic wrapping
// =============================================================================
//
// These extension traits provide method-style wrapping that works even when
// the destination type isn't constrained (i.e., `value.as_list()` instead of
// `value.into()` which requires type inference).
//
// ```ignore
// // These all work without type annotations:
// let wrapped = my_struct.as_list();
// let ptr = my_value.as_external_ptr();
// let native = my_num.as_r_native();
// ```

/// Extension trait for wrapping values as [`AsList`].
///
/// This trait is automatically implemented for all types that implement [`IntoList`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::AsListExt;
///
/// #[derive(IntoList)]
/// struct Point { x: f64, y: f64 }
///
/// let point = Point { x: 1.0, y: 2.0 };
/// let wrapped: AsList<Point> = point.as_list();
/// ```
pub trait AsListExt: IntoList + Sized {
    /// Wrap `self` in [`AsList`] for R list conversion.
    ///
    /// Note: This method consumes `self` despite the `as_` prefix because
    /// it wraps the value in an `AsList` wrapper (matching the type name).
    #[allow(clippy::wrong_self_convention)]
    fn as_list(self) -> AsList<Self> {
        AsList(self)
    }
}

impl<T: IntoList> AsListExt for T {}

/// Extension trait for wrapping values as [`ToDataFrame`].
///
/// This trait is automatically implemented for all types that implement [`IntoDataFrame`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::ToDataFrameExt;
///
/// struct TimeSeries {
///     timestamps: Vec<f64>,
///     values: Vec<f64>,
/// }
///
/// impl IntoDataFrame for TimeSeries {
///     fn into_data_frame(self) -> List {
///         List::from_pairs(vec![
///             ("timestamp", self.timestamps),
///             ("value", self.values),
///         ])
///         .set_class_str(&["data.frame"])
///         .set_row_names_int(self.timestamps.len())
///     }
/// }
///
/// let ts = TimeSeries { timestamps: vec![1.0, 2.0], values: vec![10.0, 20.0] };
/// let wrapped: ToDataFrame<TimeSeries> = ts.to_data_frame();
/// ```
pub trait ToDataFrameExt: IntoDataFrame + Sized {
    /// Wrap `self` in [`ToDataFrame`] for R data.frame conversion.
    fn to_data_frame(self) -> ToDataFrame<Self> {
        ToDataFrame(self)
    }
}

impl<T: IntoDataFrame> ToDataFrameExt for T {}

/// Extension trait for wrapping values as [`AsExternalPtr`].
///
/// This trait is automatically implemented for all types that implement [`IntoExternalPtr`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::AsExternalPtrExt;
///
/// #[derive(ExternalPtr)]
/// struct Connection { handle: u64 }
///
/// let conn = Connection { handle: 42 };
/// let wrapped: AsExternalPtr<Connection> = conn.as_external_ptr();
/// ```
pub trait AsExternalPtrExt: IntoExternalPtr + Sized {
    /// Wrap `self` in [`AsExternalPtr`] for R external pointer conversion.
    ///
    /// Note: This method consumes `self` despite the `as_` prefix because
    /// it wraps the value in an `AsExternalPtr` wrapper (matching the type name).
    #[allow(clippy::wrong_self_convention)]
    fn as_external_ptr(self) -> AsExternalPtr<Self> {
        AsExternalPtr(self)
    }
}

impl<T: IntoExternalPtr> AsExternalPtrExt for T {}

/// Extension trait for wrapping values as [`AsRNative`].
///
/// This trait is automatically implemented for all types that implement [`RNativeType`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::AsRNativeExt;
///
/// let x: f64 = 42.5;
/// let wrapped: AsRNative<f64> = x.as_r_native();
/// ```
pub trait AsRNativeExt: RNativeType + Sized {
    /// Wrap `self` in [`AsRNative`] for native R scalar conversion.
    fn as_r_native(self) -> AsRNative<Self> {
        AsRNative(self)
    }
}

impl<T: RNativeType> AsRNativeExt for T {}

/// Extension trait for wrapping tuple pair collections as [`AsNamedList`].
///
/// # Example
///
/// ```ignore
/// let pairs = vec![("x".to_string(), 1i32), ("y".to_string(), 2i32)];
/// let wrapped = pairs.as_named_list();
/// ```
pub trait AsNamedListExt: Sized {
    /// Wrap `self` in [`AsNamedList`] for named R list conversion.
    #[allow(clippy::wrong_self_convention)]
    fn as_named_list(self) -> AsNamedList<Self> {
        AsNamedList(self)
    }
}

impl<K: AsRef<str>, V: IntoR> AsNamedListExt for Vec<(K, V)> {}
impl<K: AsRef<str>, V: IntoR, const N: usize> AsNamedListExt for [(K, V); N] {}
impl<K: AsRef<str>, V: Clone + IntoR> AsNamedListExt for &[(K, V)] {}

/// Extension trait for wrapping tuple pair collections as [`AsNamedVector`].
///
/// # Example
///
/// ```ignore
/// let pairs = vec![("alice".to_string(), 95.0f64), ("bob".to_string(), 87.5)];
/// let wrapped = pairs.as_named_vector();
/// ```
pub trait AsNamedVectorExt: Sized {
    /// Wrap `self` in [`AsNamedVector`] for named atomic R vector conversion.
    #[allow(clippy::wrong_self_convention)]
    fn as_named_vector(self) -> AsNamedVector<Self> {
        AsNamedVector(self)
    }
}

impl<K: AsRef<str>, V: AtomicElement> AsNamedVectorExt for Vec<(K, V)> {}
impl<K: AsRef<str>, V: AtomicElement, const N: usize> AsNamedVectorExt for [(K, V); N] {}
impl<K: AsRef<str>, V: Clone + AtomicElement> AsNamedVectorExt for &[(K, V)] {}
