//! Owned construction of heterogeneous named lists and data frames.

use crate::dataframe::{BuiltDataFrame, DataFrameError, RootedSentinel};
use crate::{IntoR, SEXP, SexpExt, sys};

/// A safe, owned builder for a named R list of mixed [`IntoR`] values.
///
/// One growing `VECSXP` holds the values and is rooted with `R_PreserveObject`.
/// Moving or dropping builders in any order is safe: no protect-stack entries
/// are held by the builder, regardless of the number of values pushed.
/// Construction, use and drop must take place on R's main thread (`!Send + !Sync`).
///
/// Return the builder from a `#[miniextendr]` function to return a named list.
/// To keep working with the finished value while R allocates (set a class,
/// pass it to R code, read it back), consume it with
/// [`into_list`](Self::into_list) for a rooted [`BuiltList`], or with
/// [`into_data_frame`](Self::into_data_frame) for a rooted data frame. Empty
/// results have `names = character(0)`. There is no bare [`List`](super::List)
/// finish.
///
/// ```ignore
/// let mut columns = NamedListBuilder::with_capacity(2);
/// columns.push("id", vec![1, 2, 3]);
/// columns.push("value", vec![0.5, 1.5, 2.5]);
/// columns.rename(1, "estimate");
/// let frame = columns.into_data_frame()?;
/// ```
pub struct NamedListBuilder {
    values: SEXP,
    names: Vec<String>,
    _not_send: std::marker::PhantomData<*mut ()>,
}

impl NamedListBuilder {
    /// Create an empty builder on R's main thread.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Reserve space for `n` entries on R's main thread.
    pub fn with_capacity(n: usize) -> Self {
        let capacity = isize::try_from(n).expect("list capacity exceeds isize::MAX");
        // SAFETY: constructor runs on R main thread.
        let values = unsafe { SEXP::alloc_list(capacity) };
        // SAFETY: R main thread. Preserve immediately; R_PreserveObject
        // protects its argument across its own cons-cell allocation.
        unsafe { sys::R_PreserveObject(values) };
        Self {
            values,
            names: Vec::new(),
            _not_send: std::marker::PhantomData,
        }
    }

    /// Convert and append a value, rooting it before any later allocation.
    pub fn push(&mut self, name: impl Into<String>, value: impl IntoR) {
        self.push_sexp(name, value.into_sexp());
    }

    /// Append an existing R value, retaining its attributes and identity.
    ///
    /// The value must be live when passed in. A freshly allocated, unrooted
    /// value can be passed directly: even when growth is needed it is rooted
    /// before the allocation. The builder then keeps it alive independently
    /// of the caller's roots.
    pub fn push_sexp(&mut self, name: impl Into<String>, sexp: SEXP) {
        let index = isize::try_from(self.len()).expect("list length exceeds isize::MAX");
        if index == self.values.xlength() {
            // Growth allocates before the value can enter the container.
            // This temporary precious-list root uses no protect-stack slot.
            let pending = RootedSentinel::new(sexp);
            let capacity = self.len().saturating_mul(2).max(4);
            self.resize(capacity);
            self.values.set_vector_elt(index, pending.as_sexp());
        } else {
            // SET_VECTOR_ELT on this ordinary VECSXP does not allocate.
            self.values.set_vector_elt(index, sexp);
        }
        // Even a user-defined name conversion may allocate in R. Store the
        // value first so it stays rooted throughout that conversion too.
        self.names.push(name.into());
    }

    /// Rename the entry at the zero-based `index`.
    ///
    /// Panics if the index is outside `0..self.len()`.
    pub fn rename(&mut self, index: usize, name: impl Into<String>) {
        self.names[index] = name.into();
    }

    /// Entry names in insertion order.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Edit names without changing the number or order of values.
    pub fn names_mut(&mut self) -> &mut [String] {
        &mut self.names
    }

    /// Number of entries pushed so far.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether no entries have been pushed.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Finish as an owned, GC-rooted named list.
    ///
    /// The [`BuiltList`] keeps the list reachable while it lives, so its
    /// attributes can be set and it can be handed to R code before it is
    /// returned.
    pub fn into_list(mut self) -> BuiltList {
        self.finish();
        // SAFETY: R main thread (`!Send`); `self.values` is a VECSXP that
        // `self` keeps rooted until BuiltList has added its own root.
        unsafe { BuiltList::adopt_sexp(self.values) }
    }

    /// Finish as an owned, GC-rooted data frame with compact `row.names`.
    ///
    /// Returns [`DataFrameError::UnequalLengths`] naming every column whose
    /// length differs from the first. No recycling is performed. Zero columns
    /// produce a zero-row frame with `names = character(0)`.
    pub fn into_data_frame(mut self) -> Result<BuiltDataFrame, DataFrameError> {
        self.finish();
        // SAFETY: self keeps the list rooted throughout validation and all
        // attribute allocations, until BuiltDataFrame has its own root.
        let list = unsafe { super::List::from_raw(self.values) };
        let named = super::NamedList::new(list).expect("builder always sets names");
        let nrow = crate::dataframe::validate_equal_lengths(&named)?;
        let nrow = i32::try_from(nrow)
            .map_err(|_| DataFrameError::BadRowNames("row count exceeds i32::MAX".into()))?;
        let row_names = if nrow == 0 {
            Vec::<i32>::new()
        } else {
            vec![i32::MIN, -nrow]
        };
        let row_names = RootedSentinel::new(row_names);
        self.values.set_row_names(row_names.as_sexp());
        let class = RootedSentinel::new("data.frame");
        self.values.set_class(class.as_sexp());
        // SAFETY: equal lengths, names and valid row.names/class set above.
        Ok(unsafe { BuiltDataFrame::adopt_sexp(self.values) })
    }

    /// Reallocate, keeping both containers rooted until copying is complete.
    fn resize(&mut self, capacity: usize) {
        let mut replacement = Self::with_capacity(capacity);
        for i in 0..self.len() {
            let i = isize::try_from(i).expect("list index exceeds isize::MAX");
            replacement
                .values
                .set_vector_elt(i, self.values.vector_elt(i));
        }
        std::mem::swap(&mut self.values, &mut replacement.values);
        // replacement drops the old container's root, after the new one
        // owns every child. Its names Vec is empty.
    }

    fn finish(&mut self) {
        if self.values.len() != self.len() {
            self.resize(self.len());
        }
        let names = RootedSentinel::new(self.names.as_slice());
        self.values.set_names(names.as_sexp());
    }
}

impl Default for NamedListBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NamedListBuilder {
    fn drop(&mut self) {
        // SAFETY: !Send keeps drop on the constructing R thread. Release is
        // allocation-free and removes precisely this builder's root.
        unsafe { sys::R_ReleaseObject(self.values) };
    }
}

impl IntoR for NamedListBuilder {
    type Error = std::convert::Infallible;

    fn try_into_sexp(mut self) -> Result<SEXP, Self::Error> {
        self.finish();
        // Drop releases the root without allocating. As with BuiltDataFrame,
        // the IntoR caller takes responsibility for the returned SEXP.
        Ok(self.values)
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
}

/// An owned, GC-rooted R list: the plain-list finish of [`NamedListBuilder`].
///
/// The list analogue of [`BuiltDataFrame`]: it holds an `R_PreserveObject`
/// root for as long as it lives (`!Send + !Sync`), dereferences to the
/// [`List`](super::List) view for reads, and returns to R through [`IntoR`].
/// [`set_class`](Self::set_class) and [`set_attr`](Self::set_attr) allocate
/// while the list stays rooted, and return the handle, so a chain is rooted at
/// every link:
///
/// ```ignore
/// let out = columns.into_list().set_class(&["my_result", "list"]);
/// call_r_helper(out.as_sexp()); // `out` keeps the list alive across the call
/// out
/// ```
///
/// A `List` copied out of the handle (`*out`) is an unrooted view: it stays
/// valid only while the handle lives.
pub struct BuiltList {
    view: super::List,
    _not_send: std::marker::PhantomData<*mut ()>,
}

impl BuiltList {
    /// Root an existing list and take ownership of that root.
    ///
    /// # Safety
    ///
    /// R main thread; `sexp` is a live VECSXP.
    unsafe fn adopt_sexp(sexp: SEXP) -> Self {
        // SAFETY: caller contract. R_PreserveObject keeps `sexp` reachable
        // across its own allocation; precious-list entries stack, so a
        // builder releasing its own root afterwards leaves this one.
        unsafe { sys::R_PreserveObject(sexp) };
        Self {
            // SAFETY: caller guarantees a VECSXP.
            view: unsafe { super::List::from_raw(sexp) },
            _not_send: std::marker::PhantomData,
        }
    }

    /// Set the `class` attribute.
    pub fn set_class(self, classes: &[&str]) -> Self {
        let class = RootedSentinel::new(classes);
        self.view.as_sexp().set_class(class.as_sexp());
        self
    }

    /// Set the attribute `name` to `value`.
    pub fn set_attr(self, name: &str, value: impl IntoR) -> Self {
        let value = RootedSentinel::new(value);
        // Symbols are never collected.
        let symbol = SEXP::symbol(name);
        self.view.as_sexp().set_attr(symbol, value.as_sexp());
        self
    }

    fn release_into_sexp(self) -> SEXP {
        let sexp = self.view.as_sexp();
        // SAFETY: `!Send` keeps us on the R main thread; release the root
        // added in `adopt_sexp`, and skip Drop so it isn't released twice.
        unsafe { sys::R_ReleaseObject(sexp) };
        std::mem::forget(self);
        sexp
    }
}

impl std::ops::Deref for BuiltList {
    type Target = super::List;
    #[inline]
    fn deref(&self) -> &super::List {
        &self.view
    }
}

impl Drop for BuiltList {
    fn drop(&mut self) {
        // SAFETY: `!Send` keeps drop on the R main thread. Release is
        // allocation-free and removes exactly the root added in `adopt_sexp`.
        unsafe { sys::R_ReleaseObject(self.view.as_sexp()) };
    }
}

impl IntoR for BuiltList {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        // As with BuiltDataFrame, the IntoR caller takes responsibility for
        // the returned SEXP.
        Ok(self.release_into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.release_into_sexp())
    }

    #[inline]
    fn into_sexp(self) -> SEXP {
        self.release_into_sexp()
    }
}

impl std::fmt::Debug for BuiltList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltList")
            .field("len", &self.view.as_sexp().xlength())
            .finish()
    }
}
