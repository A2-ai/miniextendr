//! ALTREP ownership-policy fixtures for `test-altrep-ownership.R`.

use miniextendr_api::altrep_data::{AltIntegerData, AltStringData, AltrepDataptr, AltrepLen};
use miniextendr_api::miniextendr;

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
