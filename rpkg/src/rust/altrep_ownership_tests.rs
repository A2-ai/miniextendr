//! ALTREP ownership-policy fixtures for `test-altrep-ownership.R`.

use miniextendr_api::altrep_data::{
    AltIntegerData, AltListData, AltRealData, AltStringData, AltrepDataptr, AltrepLen,
    AltrepSerialize, IterIntData, IterListData, IterRealData,
};
use miniextendr_api::altrep_registration::RegisterAltrep;
use miniextendr_api::{Altrep, AltrepBuffer, IntoR, RLogical, SEXP, SexpExt, miniextendr};

// region: duplicate = "clone"

/// An integer vector whose `Duplicate` clones its Rust owner.
#[derive(Clone, miniextendr_api::AltrepInteger)]
#[altrep(manual, dataptr, duplicate = "clone", class = "OwnershipClonedInts")]
pub struct ClonedInts(Vec<i32>);

impl AltrepLen for ClonedInts {
    fn altrep_len(&self) -> usize {
        self.0.len()
    }
}

impl AltIntegerData for ClonedInts {
    fn elt(&self, i: usize) -> i32 {
        self.0[i]
    }
}

impl AltrepDataptr<i32> for ClonedInts {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        Some(self.0.as_mut_ptr())
    }
}

/// A string vector whose `Duplicate` clones its owner and its string cache.
#[derive(Clone, miniextendr_api::AltrepString)]
#[altrep(manual, duplicate = "clone", class = "OwnershipClonedStrings")]
pub struct ClonedStrings(Vec<Option<String>>);

impl AltrepLen for ClonedStrings {
    fn altrep_len(&self) -> usize {
        self.0.len()
    }
}

impl AltStringData for ClonedStrings {
    fn elt(&self, i: usize) -> Option<&str> {
        self.0[i].as_deref()
    }
}

/// An integer ALTREP that stays ALTREP when R duplicates it.
#[miniextendr(noexport)]
pub fn altrep_cloned_ints(values: Vec<i32>) -> ClonedInts {
    ClonedInts(values)
}

/// A string ALTREP that stays ALTREP when R duplicates it.
#[miniextendr(noexport)]
pub fn altrep_cloned_strings(values: Vec<Option<String>>) -> ClonedStrings {
    ClonedStrings(values)
}
// endregion

// region: AltrepBuffer

/// A native integer buffer ALTREP.
#[miniextendr(noexport)]
pub fn altrep_buffer_ints(values: Vec<i32>) -> Altrep<AltrepBuffer<i32>> {
    Altrep(AltrepBuffer::new(values))
}

/// A native real buffer ALTREP.
#[miniextendr(noexport)]
pub fn altrep_buffer_reals(values: Vec<f64>) -> Altrep<AltrepBuffer<f64>> {
    Altrep(AltrepBuffer::new(values))
}

/// A native logical buffer ALTREP.
#[miniextendr(noexport)]
pub fn altrep_buffer_logicals(values: Vec<RLogical>) -> Altrep<AltrepBuffer<RLogical>> {
    Altrep(AltrepBuffer::new(values))
}

/// Append to an integer buffer ALTREP in place and return it.
#[miniextendr(noexport)]
pub fn altrep_buffer_extend(x: SEXP, values: Vec<i32>) -> SEXP {
    let class = <AltrepBuffer<i32> as RegisterAltrep>::get_or_init_class();
    assert!(
        x.is_altrep() && unsafe { class.inherits(x) },
        "not an integer buffer ALTREP"
    );
    let buffer = unsafe { miniextendr_api::altrep_data1_mut::<AltrepBuffer<i32>>(x) }
        .expect("integer buffer owner");
    buffer.edit(|current| current.extend(values));
    x
}
// endregion

// region: native-read guards

/// Constant integers under the `rust_unwind` native-read guard.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(
    len = "len",
    elt = "value",
    rust_unwind,
    class = "OwnershipRustUnwindInts"
)]
pub struct RustUnwindInts {
    pub len: usize,
    pub value: i32,
}

/// Strings under the `unsafe` native-read guard, cloned on duplication.
/// Reading the value `"boom"` panics.
#[derive(Clone, miniextendr_api::AltrepString)]
#[altrep(
    manual,
    r#unsafe,
    duplicate = "clone",
    class = "OwnershipUnsafeStrings"
)]
pub struct UnsafeStrings(pub Vec<String>);

impl AltrepLen for UnsafeStrings {
    fn altrep_len(&self) -> usize {
        self.0.len()
    }
}

impl AltStringData for UnsafeStrings {
    fn elt(&self, i: usize) -> Option<&str> {
        assert_ne!(self.0[i], "boom", "UnsafeStrings refuses element {}", i + 1);
        Some(&self.0[i])
    }
}

/// Constant integers whose callbacks run under `rust_unwind`.
#[miniextendr(noexport)]
pub fn altrep_rust_unwind_ints(len: usize, value: i32) -> RustUnwindInts {
    RustUnwindInts { len, value }
}

/// Strings whose native reads run under the `unsafe` guard.
#[miniextendr(noexport)]
pub fn altrep_unsafe_strings(values: Vec<String>) -> UnsafeStrings {
    UnsafeStrings(values)
}
// endregion

// region: iterator caches

/// A lazily cached integer range that clones and serializes its cache.
#[derive(Clone, miniextendr_api::AltrepInteger)]
#[altrep(manual, serialize, duplicate = "clone", class = "OwnershipRangeCache")]
pub struct RangeCache(IterIntData<std::ops::Range<i32>>);

impl AltrepLen for RangeCache {
    fn altrep_len(&self) -> usize {
        self.0.len()
    }
}

impl AltIntegerData for RangeCache {
    fn elt(&self, i: usize) -> i32 {
        self.0.elt(i)
    }
}

impl AltrepSerialize for RangeCache {
    fn serialized_state(&self) -> SEXP {
        self.0.serialized_state()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        IterIntData::unserialize(state).map(Self)
    }
}

/// Reals from a source that ends before the declared length.
#[derive(Clone, miniextendr_api::AltrepReal)]
#[altrep(
    manual,
    serialize,
    duplicate = "clone",
    class = "OwnershipShortRealCache"
)]
pub struct ShortRealCache(IterRealData<std::vec::IntoIter<f64>>);

impl AltrepLen for ShortRealCache {
    fn altrep_len(&self) -> usize {
        self.0.len()
    }
}

impl AltRealData for ShortRealCache {
    fn elt(&self, i: usize) -> f64 {
        self.0.elt(i)
    }
}

impl AltrepSerialize for ShortRealCache {
    fn serialized_state(&self) -> SEXP {
        self.0.serialized_state()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        IterRealData::unserialize(state).map(Self)
    }
}

/// `from:(to - 1)`, read lazily from a Rust range.
#[miniextendr(noexport)]
pub fn altrep_range_cache(from: i32, to: i32) -> RangeCache {
    RangeCache(IterIntData::from_exact_iter(from..to))
}

/// `values` from a source declared `len` long.
#[miniextendr(noexport)]
pub fn altrep_short_real_cache(values: Vec<f64>, len: usize) -> ShortRealCache {
    ShortRealCache(IterRealData::from_iter(values.into_iter(), len))
}
// endregion

// region: list iterator caches

type ListSource = std::iter::Map<std::ops::Range<i32>, fn(i32) -> SEXP>;

fn list_element(i: i32) -> SEXP {
    vec![i, i + 10].into_sexp()
}

/// A lazily cached list that clones and serializes its cache.
#[derive(Clone, miniextendr_api::AltrepList)]
#[altrep(manual, serialize, duplicate = "clone", class = "OwnershipListCache")]
pub struct ListCache(IterListData<ListSource>);

impl AltrepLen for ListCache {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl AltListData for ListCache {
    fn elt(&self, i: usize) -> SEXP {
        self.0.elt(i)
    }
}

impl AltrepSerialize for ListCache {
    fn serialized_state(&self) -> SEXP {
        self.0.serialized_state()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        IterListData::unserialize(state).map(Self)
    }
}

/// `list(c(0L, 10L), c(1L, 11L), ...)` from a source of `values` elements,
/// declared `len` long and read lazily.
#[miniextendr(noexport)]
pub fn altrep_list_cache(values: i32, len: usize) -> ListCache {
    let source = (0..values).map(list_element as fn(i32) -> SEXP);
    ListCache(IterListData::from_iter(source, len))
}
// endregion
