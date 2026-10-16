//! One generated S3 generic shared by inherent and trait methods (#2050).

use miniextendr_api::{ExternalPtr, miniextendr};

/// An inherent implementation of the shared documentation-test generic.
#[derive(ExternalPtr)]
pub struct GenericDocInherent;

#[miniextendr(s3)]
impl GenericDocInherent {
    pub fn new() -> Self {
        Self
    }

    /// Return the inherent implementation's value.
    pub fn generic_doc_shared(&self) -> i32 {
        11
    }
}

/// A trait implementation of the shared documentation-test generic.
#[derive(ExternalPtr)]
pub struct GenericDocTrait;

#[miniextendr(s3)]
impl GenericDocTrait {
    pub fn new() -> Self {
        Self
    }
}

/// Trait half of the S3 generic documentation fixture.
#[miniextendr]
pub trait GenericDocValue {
    fn generic_doc_shared(&self) -> i32;
}

#[miniextendr(s3)]
impl GenericDocValue for GenericDocTrait {
    fn generic_doc_shared(&self) -> i32 {
        22
    }
}
