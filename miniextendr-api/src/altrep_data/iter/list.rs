//! A lazy list iterator with an owned, GC-visible cache.

use std::cell::{Cell, OnceCell, RefCell};

use crate::altrep_data::{AltListData, AltrepLen, AltrepSerialize};
use crate::externalptr::PoolRoot;
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

/// Iterator-backed list data with a rooted cache of the yielded R objects.
///
/// Construction does not advance the iterator or allocate R storage. The first
/// in-bounds read allocates one list of the declared length, and each yielded
/// object goes into it before the next call to `Iterator::next`. So the
/// iterator can return fresh, unprotected objects, as in
/// `(0..n).map(|i| vec![i].into_sexp())`. It remains responsible for rooting
/// any R objects it keeps between calls.
///
/// The cache list is rooted in the same pool as owning `ExternalPtr` handles.
/// The adapter roots it itself, so it is safe before it is attached to an
/// ALTREP object, and releases it on drop. That pool releases in O(1) in any
/// order: R's finalizers drop ALTREP owners newest first, whatever order their
/// caches were created in.
///
/// Reads generate only the requested prefix. Elements past the end of a short
/// source read as R `NULL`; the source is dropped on its first `None` and never
/// resumed. An infinite source is consumed only up to the declared length.
///
/// `Clone` copies the cache list, sharing its elements as R's
/// `shallow_duplicate()` does, and clones the remaining iterator without
/// advancing it. Serialization realizes the declared prefix, `NULL`s included,
/// and restoration needs no `I`. Delegate these data traits from a concrete
/// `#[derive(AltrepList)]` class with
/// `#[altrep(manual, serialize, duplicate = "clone")]`.
///
/// The adapter is `!Send + !Sync`: build, read, clone and drop it on R's main
/// thread.
///
/// ```ignore
/// use miniextendr_api::{IntoR, altrep_data::IterListData};
/// let data = IterListData::from_exact_iter((0..5).map(|x| vec![x, x + 1].into_sexp()));
/// ```
pub struct IterListData<I: Iterator<Item = SEXP>> {
    len: usize,
    iter: RefCell<Option<I>>,
    cached: Cell<usize>,
    cache: OnceCell<PoolRoot>,
}

/// An in-bounds list index as R's index type.
fn r_index(i: usize) -> R_xlen_t {
    R_xlen_t::try_from(i).expect("ALTREP list length exceeds R_xlen_t")
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
            .get_or_init(|| unsafe { PoolRoot::new(SEXP::alloc_list(r_index(self.len))) })
            .get()
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
        // Hold the borrow during cache allocation and next(): a reentrant read
        // fails here instead of advancing the same source twice.
        let mut iter = self.iter.borrow_mut();
        let cache = self.cache();
        while self.cached.get() <= i {
            let Some(value) = iter.as_mut().and_then(Iterator::next) else {
                *iter = None;
                break;
            };
            // Nothing allocates between next() returning and this store.
            cache.set_vector_elt(r_index(self.cached.get()), value);
            self.cached.set(self.cached.get() + 1);
        }
        if self.cached.get() == self.len {
            *iter = None;
        }
        cache.vector_elt(r_index(i))
    }
}

impl<I: Iterator<Item = SEXP> + Clone> Clone for IterListData<I> {
    fn clone(&self) -> Self {
        let cache = OnceCell::new();
        if let Some(values) = self.cache.get() {
            // The copy fills its own slots as its iterator advances. R copies
            // an element both lists hold before changing it.
            let _ = cache.set(unsafe { PoolRoot::new(values.get().shallow_duplicate()) });
        }
        Self {
            len: self.len,
            iter: RefCell::new(self.iter.borrow().clone()),
            cached: Cell::new(self.cached.get()),
            cache,
        }
    }
}

impl<I: Iterator<Item = SEXP>> AltrepSerialize for IterListData<I> {
    fn serialized_state(&self) -> SEXP {
        if self.len != 0 {
            self.elt(self.len - 1);
        }
        // A list of its own, so the caller may change it.
        self.cache().shallow_duplicate()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        if state.type_of() != SEXPTYPE::VECSXP {
            return None;
        }
        // R hands over a list it just read, so it becomes the cache as is.
        let len = state.len();
        Some(Self {
            len,
            iter: RefCell::new(None),
            cached: Cell::new(len),
            cache: OnceCell::from(unsafe { PoolRoot::new(state) }),
        })
    }
}
