//! Shared generic help for inherent and trait methods (S3 #2050, S4 #2039).

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

/// An inherent method on an explicitly owned S4 generic.
#[derive(ExternalPtr)]
pub struct GenericDocS4Inherent;

#[miniextendr(s4)]
impl GenericDocS4Inherent {
    pub fn new() -> Self {
        Self
    }

    /// Return the inherent S4 implementation's value.
    #[miniextendr(s4(generic = "generic_doc_s4_shared", own, receiver = "object"))]
    pub fn value(&self) -> i32 {
        31
    }
}

/// A trait method on the same explicitly owned S4 generic.
#[derive(ExternalPtr)]
pub struct GenericDocS4Trait;

#[miniextendr(s4)]
impl GenericDocS4Trait {
    pub fn new() -> Self {
        Self
    }
}

/// Trait half of the owned S4 generic fixture.
#[miniextendr]
pub trait GenericDocS4Value {
    fn value(&self) -> i32;
}

#[miniextendr(s4)]
impl GenericDocS4Value for GenericDocS4Trait {
    #[miniextendr(s4(generic = "generic_doc_s4_shared", own, receiver = "object"))]
    fn value(&self) -> i32 {
        32
    }
}
