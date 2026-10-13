//! Compile-pass test: the constructor of a type split across impl blocks is
//! decided when the wrapper writer merges the blocks, so each case below
//! compiles. Each block that infers a constructor also registers how it reads
//! when another block declares one; the install-time choice and its errors are
//! unit-tested in miniextendr-api's `registry.rs`.

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

/// `new` in the first block: the second block's `from_half` is a static method.
#[derive(ExternalPtr)]
pub struct Declared {
    value: i32,
}

#[miniextendr(r6)]
impl Declared {
    pub fn new(value: i32) -> Self {
        Declared { value }
    }
}

#[miniextendr(r6)]
impl Declared {
    pub fn from_half(half: i32) -> Self {
        Declared { value: half * 2 }
    }
}

/// The one candidate is in the second block: it is the constructor.
#[derive(ExternalPtr)]
pub struct Inferred {
    value: i32,
}

#[miniextendr(s4)]
impl Inferred {
    pub fn value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(s4)]
impl Inferred {
    pub fn from_value(value: i32) -> Self {
        Inferred { value }
    }
}

/// One candidate in each block: the compiler sees one per block, and the
/// wrapper writer refuses the type when the package installs.
#[derive(ExternalPtr)]
pub struct TwoBlocks {
    value: i32,
}

#[miniextendr(env)]
impl TwoBlocks {
    pub fn zero() -> Self {
        TwoBlocks { value: 0 }
    }
}

#[miniextendr(env)]
impl TwoBlocks {
    pub fn one() -> Self {
        TwoBlocks { value: 1 }
    }
}

/// A constructor marked in each block: refused by the wrapper writer as two
/// blocks that define the class.
#[derive(ExternalPtr)]
pub struct TwoMarked {
    value: i32,
}

#[miniextendr(s7)]
impl TwoMarked {
    #[miniextendr(s7(constructor))]
    pub fn zero() -> Self {
        TwoMarked { value: 0 }
    }
}

#[miniextendr(s7)]
impl TwoMarked {
    #[miniextendr(s7(constructor))]
    pub fn one() -> Self {
        TwoMarked { value: 1 }
    }
}

/// Several candidates, one marked: the marked one is the constructor, the
/// others are static methods.
#[derive(ExternalPtr)]
pub struct Factories {
    value: i32,
}

#[miniextendr(s3)]
impl Factories {
    #[miniextendr(s3(constructor))]
    pub fn zero() -> Self {
        Factories { value: 0 }
    }

    pub fn one() -> Self {
        Factories { value: 1 }
    }

    pub fn factories_value(&self) -> i32 {
        self.value
    }
}

/// No candidate: a fallible factory is no constructor, so the class has none
/// and its objects come from the factories.
#[derive(ExternalPtr)]
pub struct Parsed {
    value: i32,
}

#[miniextendr(s7)]
impl Parsed {
    pub fn parse(text: &str) -> Result<Self, String> {
        text.parse()
            .map(|value| Parsed { value })
            .map_err(|_| text.to_string())
    }

    pub fn find(value: i32) -> Option<Self> {
        (value >= 0).then_some(Parsed { value })
    }

    pub fn parsed_value(&self) -> i32 {
        self.value
    }
}

fn main() {}
