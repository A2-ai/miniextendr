//! Every place a gate can sit inside a `#[miniextendr]` item is resolved before
//! expansion (#2107). A disabled member names `MissingType`, so any generated
//! code that still referred to it would fail to compile.
#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

mod fn_gates {
    use super::*;

    #[cfg(any())]
    #[miniextendr]
    pub fn before(_x: MissingType) -> MissingType { unreachable!() }

    #[miniextendr]
    #[cfg(any())]
    pub fn after(_x: MissingType) -> MissingType { unreachable!() }

    #[miniextendr]
    #[cfg_attr(all(), cfg(any()))]
    pub fn after_cfg_attr(_x: MissingType) -> MissingType { unreachable!() }

    #[miniextendr]
    #[cfg(all())]
    pub fn alternative() -> i32 { 1 }

    #[miniextendr]
    #[cfg(any())]
    pub fn alternative() -> MissingType { unreachable!() }

    #[miniextendr]
    pub fn params(
        #[cfg(any())] _hidden: MissingType,
        #[cfg(all())] kept: i32,
        #[cfg_attr(all(), cfg(any()))] _nested: MissingType,
    ) -> i32 {
        kept
    }
}

mod impl_gates {
    use super::*;

    #[derive(ExternalPtr)]
    pub struct Host;

    #[miniextendr(env)]
    impl Host {
        pub fn new() -> Self { Host }

        #[cfg(any())]
        pub fn hidden(&self, _x: MissingType) -> MissingType { unreachable!() }

        #[cfg(any())]
        pub const HIDDEN: MissingType = unreachable!();
    }

    #[miniextendr(env)]
    #[cfg(any())]
    impl Host {
        pub fn whole_block(&self) -> MissingType { unreachable!() }
    }
}

mod trait_gates {
    use super::*;

    #[miniextendr]
    pub trait Shape {
        fn area(&self) -> f64;

        #[cfg(any())]
        fn hidden(&self) -> MissingType;
    }

    #[miniextendr]
    #[cfg(any())]
    pub trait Hidden {
        fn hidden(&self) -> MissingType;
    }

    #[derive(ExternalPtr)]
    pub struct Square;

    #[miniextendr(env)]
    impl Square {
        pub fn new() -> Self { Square }
    }

    #[miniextendr(env)]
    impl Shape for Square {
        #[cfg(all())]
        fn area(&self) -> f64 { 1.0 }

        #[cfg(any())]
        fn area(&self) -> f64 { 2.0 }
    }

    #[miniextendr(env)]
    #[cfg(any())]
    impl Hidden for Square {
        fn hidden(&self) -> MissingType { unreachable!() }
    }
}

mod type_gates {
    use super::*;

    #[miniextendr]
    pub struct Pointer {
        value: i32,
        #[cfg(any())]
        hidden: MissingType,
    }

    #[miniextendr(list)]
    pub struct Record {
        pub value: i32,
        #[cfg(any())]
        pub hidden: MissingType,
    }

    #[miniextendr(dataframe)]
    pub struct Row {
        pub value: i32,
        #[cfg(any())]
        pub hidden: MissingType,
    }

    #[miniextendr]
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Level {
        Low,
        #[cfg(any())]
        Hidden,
        High,
    }

    #[miniextendr(match_arg)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Mode {
        Fast,
        #[cfg_attr(all(), cfg(any()))]
        Hidden,
        Slow,
    }
}

fn main() {}
