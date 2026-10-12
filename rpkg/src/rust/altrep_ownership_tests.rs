//! ALTREP ownership-policy fixtures for `test-altrep-ownership.R`.

use miniextendr_api::altrep_data::{AltIntegerData, AltStringData, AltrepDataptr, AltrepLen};
use miniextendr_api::altrep_registration::RegisterAltrep;
use miniextendr_api::{Altrep, AltrepBuffer, RLogical, SEXP, SexpExt, miniextendr};

// region: duplicate = "clone"

/// An integer vector whose `Duplicate` clones its Rust owner.
#[derive(Clone, miniextendr_api::AltrepInteger)]
#[altrep(manual, dataptr, duplicate = "clone", class = "OwnershipClonedInts")]
pub struct ClonedInts(Vec<i32>);

impl AltrepLen for ClonedInts {
    fn len(&self) -> usize {
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
    fn len(&self) -> usize {
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
