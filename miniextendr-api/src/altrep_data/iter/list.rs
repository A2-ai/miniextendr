//! A lazy list iterator with an owned, GC-visible cache.

use std::cell::{Cell, OnceCell, RefCell};

use crate::altrep_data::{AltListData, AltrepLen, AltrepSerialize};
use crate::{OwnedProtect, Preserved, SEXP, SEXPTYPE, SexpExt};

/// Iterator-backed list data with a rooted cache of the yielded R objects.
///
/// Construction does not advance the iterator or allocate R storage. The first
/// in-bounds read allocates one preserved list of the declared length. Each
/// yielded object is stored there before the next call to `Iterator::next`,
/// so allocating iterators can return fresh, unprotected objects. The iterator
/// remains responsible for rooting any R objects it keeps between calls.
///
/// Reads generate only the requested prefix. Missing elements of a short
/// source read as R `NULL`; the source is dropped on its first `None` and never
/// resumed. An infinite source is consumed only up to the declared length.
///
/// `Clone` copies the cache and the remaining iterator without advancing it.
/// Serialization realizes the finite sequence, including missing `NULL`s;
/// restoration does not require an instance of `I`. Delegate these data traits
/// from a concrete `#[derive(AltrepList)]` class with
/// `#[altrep(manual, serialize, duplicate = "clone")]`.
///
/// Like [`Preserved`], this adapter is `!Send + !Sync`. Access, clone and drop
/// it on R's main thread. Its cache stays rooted even before it is attached to
/// an ALTREP owner, and is released when the adapter drops.
///
/// ```ignore
/// use miniextendr_api::{IntoR, altrep_data::IterListData};
/// let data = IterListData::from_exact_iter((0..5).map(|x| vec![x, x + 1].into_sexp()));
/// ```
pub struct IterListData<I: Iterator<Item = SEXP>> {
    len: usize,
    iter: RefCell<Option<I>>,
    cached: Cell<usize>,
    cache: OnceCell<Preserved>,
}

impl<I: Iterator<Item = SEXP>> IterListData<I> {
    /// Create a lazy adapter for exactly the declared prefix.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            len,
            iter: RefCell::new(Some(iter)),
            cached: Cell::new(0),
            cache: OnceCell::new(),
        }
    }

    fn cache(&self) -> SEXP {
        self.cache
            .get_or_init(|| {
                let len = crate::R_xlen_t::try_from(self.len)
                    .expect("ALTREP list length exceeds R_xlen_t");
                // The new list is preserved immediately, before another R allocation.
                Preserved::new(unsafe { SEXP::alloc_list(len) })
            })
            .as_sexp()
    }
}

impl<I: ExactSizeIterator<Item = SEXP>> IterListData<I> {
    /// Create a lazy adapter using the iterator's remaining length.
    pub fn from_exact_iter(iter: I) -> Self {
        let len = iter.len();
        Self::from_iter(iter, len)
    }
}

impl<I: Iterator<Item = SEXP>> AltrepLen for IterListData<I> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<I: Iterator<Item = SEXP>> AltListData for IterListData<I> {
    fn elt(&self, i: usize) -> SEXP {
        if i >= self.len {
            return SEXP::nil();
        }
        // Hold the borrow during cache allocation and next(): reentry must
        // fail before it can advance the same source or initialize a second cache.
        let mut iter = self.iter.borrow_mut();
        let cache = self.cache();
        while self.cached.get() <= i {
            match iter.as_mut().and_then(Iterator::next) {
                Some(value) => {
                    let value = unsafe { OwnedProtect::new(value) };
                    cache.set_vector_elt(
                        crate::R_xlen_t::try_from(self.cached.get()).unwrap(),
                        value.get(),
                    );
                    self.cached.set(self.cached.get() + 1);
                }
                None => {
                    iter.take();
                    break;
                }
            }
        }
        if self.cached.get() == self.len {
            iter.take();
        }
        cache.vector_elt(crate::R_xlen_t::try_from(i).unwrap())
    }
}

impl<I: Iterator<Item = SEXP> + Clone> Clone for IterListData<I> {
    fn clone(&self) -> Self {
        let source = self.iter.borrow();
        let iter = source.clone();
        let cache = OnceCell::new();
        if let Some(values) = self.cache.get() {
            // A clone must not overwrite slots in the original when their
            // iterators advance independently. Copy nested list values too.
            let copy = Preserved::new(unsafe { crate::sys::Rf_duplicate(values.as_sexp()) });
            let _ = cache.set(copy);
        }
        Self {
            len: self.len,
            iter: RefCell::new(iter),
            cached: Cell::new(self.cached.get()),
            cache,
        }
    }
}

impl<I: Iterator<Item = SEXP>> AltrepSerialize for IterListData<I> {
    fn serialized_state(&self) -> SEXP {
        if self.len != 0 {
            self.elt(self.len - 1);
        } else {
            self.iter.borrow_mut().take();
        }
        // Publish an independent list, retaining no writable alias of the cache.
        unsafe { crate::sys::Rf_duplicate(self.cache()) }
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        if state.type_of() != SEXPTYPE::VECSXP {
            return None;
        }
        let state = Preserved::new(state);
        let restored = Self {
            len: state.len(),
            iter: RefCell::new(None),
            cached: Cell::new(state.len()),
            cache: OnceCell::new(),
        };
        let cache = restored.cache();
        // Copy into a plain list even when the input is itself ALTREP. Root
        // each possibly transient Elt result before reading the next one.
        for i in 0..state.xlength() {
            let value = unsafe { OwnedProtect::new(state.vector_elt(i)) };
            cache.set_vector_elt(i, value.get());
        }
        Some(restored)
    }
}
