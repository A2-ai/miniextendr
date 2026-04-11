//! `ListAccumulator` — unknown-length list construction with bounded stack usage.
//!
//! Unlike [`ListBuilder`](super::ListBuilder) which requires knowing the size at construction,
//! `ListAccumulator` supports dynamic growth via `push`. It uses
//! `ReprotectSlot` internally to maintain O(1) protect stack usage.

use crate::ffi::SEXPTYPE::{STRSXP, VECSXP};
use crate::ffi::{self, SEXP, SexpExt};
use crate::from_r::SexpError;
use crate::gc_protect::{OwnedProtect, ProtectScope, ReprotectSlot, Root};
use crate::into_r::IntoR;

use super::ListMut;

/// Accumulator for building lists when the length is unknown upfront.
///
/// Unlike [`super::ListBuilder`] which requires knowing the size at construction,
/// `ListAccumulator` supports dynamic growth via [`push`](Self::push). It uses
/// [`ReprotectSlot`] internally to maintain **O(1) protect stack usage** regardless
/// of how many elements are pushed.
///
/// # When to Use
///
/// | Scenario | Recommended Type |
/// |----------|-----------------|
/// | Known size | [`super::ListBuilder`] - more efficient, no reallocation |
/// | Unknown size | `ListAccumulator` - bounded stack, dynamic growth |
/// | Streaming/iterators | `ListAccumulator` or [`collect_list`] |
///
/// # Growth Strategy
///
/// The internal list grows exponentially (2x) when capacity is exceeded,
/// achieving amortized O(1) push. Elements are copied during growth.
///
/// # Example
///
/// ```ignore
/// unsafe fn collect_filtered(items: &[i32]) -> SEXP {
///     let scope = ProtectScope::new();
///     let mut acc = ListAccumulator::new(&scope, 4); // initial capacity hint
///
///     for &item in items {
///         if item > 0 {
///             acc.push(item);  // auto-converts via IntoR
///         }
///     }
///
///     acc.into_root().get()
/// }
/// ```
pub struct ListAccumulator<'a> {
    /// The current list container (protected via ReprotectSlot).
    list: ReprotectSlot<'a>,
    /// Temporary slot for element conversion and list growth.
    temp: ReprotectSlot<'a>,
    /// Number of elements currently in the list.
    len: usize,
    /// Current capacity of the list.
    cap: usize,
    /// Reference to the scope for creating the final Root.
    scope: &'a ProtectScope,
    /// Per-element names (None = unnamed, Some = named).
    names: Vec<Option<String>>,
}

impl<'a> ListAccumulator<'a> {
    /// Create a new accumulator with the given initial capacity.
    ///
    /// A capacity of 0 is valid; the list will grow on first push.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn new(scope: &'a ProtectScope, initial_cap: usize) -> Self {
        let cap = initial_cap.max(1); // At least 1 to avoid edge cases
        let cap_isize: isize = cap.try_into().expect("capacity exceeds isize::MAX");
        let list_sexp = unsafe { ffi::Rf_allocVector(VECSXP, cap_isize) };
        let list = unsafe { scope.protect_with_index(list_sexp) };
        let temp = unsafe { scope.protect_with_index(SEXP::nil()) };

        Self {
            list,
            temp,
            len: 0,
            cap,
            scope,
            names: Vec::new(),
        }
    }

    /// Push a value onto the accumulator.
    ///
    /// The value is converted to a SEXP via [`IntoR`] and inserted.
    /// If the internal list is full, it grows automatically.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn push<T: IntoR>(&mut self, value: T) {
        // Grow if needed
        if self.len >= self.cap {
            unsafe { self.grow() };
        }

        // Convert value using temp slot for protection during conversion
        let sexp = unsafe { self.temp.set_with(|| value.into_sexp()) };

        // Insert into list (list and temp are both protected)
        let len_isize: isize = self.len.try_into().expect("list length exceeds isize::MAX");
        self.list.get().set_vector_elt(len_isize, sexp);

        self.names.push(None);
        self.len += 1;
    }

    /// Push a raw SEXP onto the accumulator.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `sexp` must be a valid SEXP (it will be temporarily protected)
    pub unsafe fn push_sexp(&mut self, sexp: SEXP) {
        // Grow if needed
        if self.len >= self.cap {
            unsafe { self.grow() };
        }

        // Protect the sexp during insertion using temp slot
        let len_isize: isize = self.len.try_into().expect("list length exceeds isize::MAX");
        unsafe {
            self.temp.set(sexp);
            self.list.get().set_vector_elt(len_isize, sexp);
        }

        self.names.push(None);
        self.len += 1;
    }

    /// Push a named value onto the accumulator.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn push_named<T: IntoR>(&mut self, name: &str, value: T) {
        // Grow if needed
        if self.len >= self.cap {
            unsafe { self.grow() };
        }

        let sexp = unsafe { self.temp.set_with(|| value.into_sexp()) };

        let len_isize: isize = self.len.try_into().expect("list length exceeds isize::MAX");
        self.list.get().set_vector_elt(len_isize, sexp);

        self.names.push(Some(name.to_string()));
        self.len += 1;
    }

    /// Push a value only if the condition is true.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn push_if<T: IntoR>(&mut self, condition: bool, value: T) {
        if condition {
            unsafe { self.push(value) };
        }
    }

    /// Push a lazily-evaluated value only if the condition is true.
    ///
    /// The closure is only called if `condition` is true.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn push_if_with<T: IntoR>(&mut self, condition: bool, f: impl FnOnce() -> T) {
        if condition {
            unsafe { self.push(f()) };
        }
    }

    /// Push all items from an iterator.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn extend_from<I, T>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
        T: IntoR,
    {
        for item in iter {
            unsafe { self.push(item) };
        }
    }

    /// Grow the internal list by 2x.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    unsafe fn grow(&mut self) {
        let new_cap = self.cap.saturating_mul(2).max(4);
        let new_cap_isize: isize = new_cap.try_into().expect("new capacity exceeds isize::MAX");

        // Allocate new list via temp slot (safe pattern)
        let old_list = self.list.get();
        unsafe {
            self.temp
                .set_with(|| ffi::Rf_allocVector(VECSXP, new_cap_isize));
        }
        let new_list = self.temp.get();

        // Copy existing elements
        for i in 0..self.len {
            let idx: isize = i.try_into().expect("index exceeds isize::MAX");
            let elem = old_list.vector_elt(idx);
            new_list.set_vector_elt(idx, elem);
        }

        // Replace list slot with new list
        unsafe { self.list.set(new_list) };
        self.cap = new_cap;
    }

    /// Get the current number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the accumulator is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the current capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Finalize the accumulator and return a `Root` pointing to the list.
    ///
    /// The returned list is truncated to the actual length (if smaller than capacity).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn into_root(self) -> Root<'a> {
        let has_names = self.names.iter().any(|n| n.is_some());

        // If len < cap, we need to shrink the list
        let len_isize: isize = self.len.try_into().expect("list length exceeds isize::MAX");
        let root = if self.len < self.cap {
            unsafe {
                let shrunk = self.list.get().resize(len_isize);
                // The shrunk list might be the same or a new allocation
                // Either way, we protect it via the scope
                self.scope.protect(shrunk)
            }
        } else {
            // List is already the right size, create a Root without extra protection
            unsafe { self.scope.rooted(self.list.get()) }
        };

        if has_names {
            unsafe {
                // OwnedProtect handles Rf_protect/Rf_unprotect automatically.
                // Rf_mkCharLenCE can allocate, so names_sexp must be protected.
                let names_sexp = OwnedProtect::new(ffi::Rf_allocVector(STRSXP, len_isize));
                for (i, name) in self.names.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    if let Some(n) = name {
                        let _n_len: i32 = n.len().try_into().expect("name exceeds i32::MAX bytes");
                        let charsxp = ffi::SEXP::charsxp(n);
                        names_sexp.get().set_string_elt(idx, charsxp);
                    } else {
                        names_sexp.get().set_string_elt(idx, SEXP::blank_string());
                    }
                }
                root.get().set_names(names_sexp.get());
            }
        }

        root
    }

    /// Finalize and return the raw SEXP.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn into_sexp(self) -> SEXP {
        unsafe { self.into_root().get() }
    }
}

/// Collect an iterator into an R list with bounded protect stack usage.
///
/// This is a convenience wrapper around [`ListAccumulator`] for iterator-based
/// collection. Each element is converted via [`IntoR`].
///
/// # Safety
///
/// Must be called from the R main thread.
///
/// # Example
///
/// ```ignore
/// unsafe fn squares(n: usize) -> SEXP {
///     let scope = ProtectScope::new();
///     collect_list(&scope, (0..n).map(|i| (i * i) as i32)).get()
/// }
/// ```
pub unsafe fn collect_list<'a, I, T>(scope: &'a ProtectScope, iter: I) -> Root<'a>
where
    I: IntoIterator<Item = T>,
    T: IntoR,
{
    let iter = iter.into_iter();
    let (lower, upper) = iter.size_hint();
    let initial_cap = upper.unwrap_or(lower).max(4);

    let mut acc = unsafe { ListAccumulator::new(scope, initial_cap) };

    for item in iter {
        unsafe { acc.push(item) };
    }

    unsafe { acc.into_root() }
}

impl ListMut {
    /// Wrap an existing `VECSXP` without additional checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure `sexp` is a valid `VECSXP` and remains managed by R.
    #[inline]
    pub const unsafe fn from_raw(sexp: SEXP) -> Self {
        ListMut(sexp)
    }

    /// Get the underlying `SEXP`.
    #[inline]
    pub const fn as_sexp(&self) -> SEXP {
        self.0
    }

    /// Length of the list (number of elements).
    #[inline]
    pub fn len(&self) -> isize {
        unsafe { ffi::Rf_xlength(self.0) }
    }

    /// Returns true if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get raw SEXP element at 0-based index. Returns `None` if out of bounds.
    #[inline]
    pub fn get(&self, idx: isize) -> Option<SEXP> {
        if idx < 0 || idx >= self.len() {
            return None;
        }
        Some(self.0.vector_elt(idx))
    }

    /// Set raw SEXP element at 0-based index.
    #[inline]
    pub fn set(&mut self, idx: isize, value: SEXP) -> Result<(), SexpError> {
        if idx < 0 || idx >= self.len() {
            return Err(SexpError::InvalidValue("index out of bounds".into()));
        }
        self.0.set_vector_elt(idx, value);
        Ok(())
    }
}
// endregion
