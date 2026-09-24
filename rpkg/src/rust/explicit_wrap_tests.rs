//! Explicit cross-class return wrapping and trait factory regressions (#1521).

use crate::s3_tests::S3Counter;
use crate::s4_tests::S4Counter;
use crate::s7_tests::S7Counter;
use crate::trait_r6_collision::DoublerEnv;
use miniextendr_api::{
    ExternalPtr, Invisible, WrapAsEnv, WrapAsR6, WrapAsS3, WrapAsS4, WrapAsS7, miniextendr,
};

/// A board returned by a different class's instance method.
#[derive(ExternalPtr)]
pub struct WrapBoard {
    w: i32,
    h: i32,
}

#[miniextendr(r6)]
impl WrapBoard {
    /// Construct a board.
    /// @param w,h Board width and height.
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }
    /// Read the dimensions to demonstrate that the returned object is usable.
    pub fn dimensions(&self) -> Vec<i32> {
        vec![self.w, self.h]
    }
}

/// An Env class that explicitly returns instances of other class systems.
#[derive(ExternalPtr)]
pub struct WrapFactory;

#[miniextendr(env)]
impl WrapFactory {
    /// Construct the factory.
    /// @examples
    /// factory <- WrapFactory$new()
    /// board <- factory$build(3L, 4L)
    /// board$dimensions()
    /// factory$build_attr(3L, 4L)$dimensions()
    pub fn new() -> Self {
        Self
    }
    /// Return a usable R6 board from an Env instance method.
    /// @param w,h Board dimensions.
    pub fn build(&self, w: i32, h: i32) -> WrapAsR6<WrapBoard> {
        WrapAsR6(WrapBoard::new(w, h))
    }
    /// The equivalent attribute spelling keeps the Rust return type unchanged.
    /// @param w,h Board dimensions.
    #[miniextendr(wrap = "r6")]
    pub fn build_attr(&self, w: i32, h: i32) -> WrapBoard {
        WrapBoard::new(w, h)
    }
    /// Wrap an S7 counter.
    pub fn s7(&self) -> WrapAsS7<S7Counter> {
        WrapAsS7(S7Counter::new(7))
    }
    /// Attribute equivalent for S7.
    #[miniextendr(wrap = "s7")]
    pub fn s7_attr(&self) -> S7Counter {
        S7Counter::new(7)
    }
    /// Wrap an S4 counter.
    pub fn s4(&self) -> WrapAsS4<S4Counter> {
        WrapAsS4(S4Counter::new(4))
    }
    /// Attribute equivalent for S4.
    #[miniextendr(wrap = "s4")]
    pub fn s4_attr(&self) -> S4Counter {
        S4Counter::new(4)
    }
    /// Wrap an S3 counter.
    pub fn s3(&self) -> WrapAsS3<S3Counter> {
        WrapAsS3(S3Counter::new(3))
    }
    /// Attribute equivalent for S3.
    #[miniextendr(wrap = "s3")]
    pub fn s3_attr(&self) -> S3Counter {
        S3Counter::new(3)
    }
    /// Wrap another Env class.
    pub fn env(&self) -> WrapAsEnv<DoublerEnv> {
        WrapAsEnv(DoublerEnv::new(8))
    }
    /// Nested class-system attribute spelling.
    #[miniextendr(env(wrap = "env"))]
    pub fn env_attr(&self) -> DoublerEnv {
        DoublerEnv::new(8)
    }
    /// Preserve error transport before constructing the R6 object.
    /// @param fail Whether to return an error.
    pub fn fallible(&self, fail: bool) -> Result<WrapAsR6<WrapBoard>, String> {
        if fail {
            Err("board construction failed".into())
        } else {
            Ok(self.build(2, 5))
        }
    }
    /// Wrap each returned board independently.
    pub fn boards(&self) -> Vec<WrapAsR6<WrapBoard>> {
        vec![self.build(1, 2), self.build(3, 4)]
    }
    /// Optional method returns retain the boundary error on None.
    /// @param present Whether to return a vector.
    pub fn optional_boards(&self, present: bool) -> Option<Vec<WrapAsR6<WrapBoard>>> {
        present.then(|| self.boards())
    }
    /// Ordinary optional class returns retain the boundary's None error.
    /// @param present Whether to return a board.
    pub fn optional_board(&self, present: bool) -> Option<WrapAsR6<WrapBoard>> {
        present.then(|| self.build(6, 7))
    }
    /// Visibility applies to the completed R6 object.
    pub fn hidden(&self) -> Invisible<WrapAsR6<WrapBoard>> {
        Invisible(self.build(8, 9))
    }
}

/// Explicit marker and attribute trait factories returning Self.
#[miniextendr]
pub trait WrapBoardFactory {
    fn copy_board(&self) -> WrapAsR6<Self>
    where
        Self: Sized;
    fn create_board(w: i32, h: i32) -> Self;
}

#[miniextendr(r6)]
impl WrapBoardFactory for WrapBoard {
    fn copy_board(&self) -> WrapAsR6<Self> {
        WrapAsR6(Self::new(self.w, self.h))
    }
    #[miniextendr(wrap = "r6")]
    fn create_board(w: i32, h: i32) -> Self {
        Self::new(w, h)
    }
}

/// Explicit wrapping also applies to free functions.
/// @param w,h Board dimensions.
#[miniextendr]
pub fn wrapped_board(w: i32, h: i32) -> WrapAsR6<WrapBoard> {
    WrapAsR6(WrapBoard::new(w, h))
}

/// Attribute equivalent for a free function.
/// @param w,h Board dimensions.
#[miniextendr(wrap = "r6")]
pub fn wrapped_board_attr(w: i32, h: i32) -> WrapBoard {
    WrapBoard::new(w, h)
}

/// Worker results are wrapped after reaching the R thread.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker)]
pub fn wrapped_board_worker() -> WrapAsR6<WrapBoard> {
    WrapAsR6(WrapBoard::new(9, 10))
}

#[cfg(feature = "vctrs")]
mod vctrs {
    use miniextendr_api::vctrs::VctrsClass;
    use miniextendr_api::{PreferVctrs, Vctrs, WrapAsVctrs, miniextendr};
    /// A record whose vctrs class hierarchy must survive explicit wrapping.
    #[derive(Vctrs, PreferVctrs)]
    #[vctrs(class = "WrappedRecord", base = "record")]
    pub struct WrappedRecord {
        #[vctrs(data)]
        x: Vec<i32>,
        y: Vec<i32>,
    }

    /// Return a record with its complete vctrs hierarchy.
    #[miniextendr]
    pub fn wrapped_record() -> WrapAsVctrs<WrappedRecord> {
        WrapAsVctrs(WrappedRecord {
            x: vec![1, 2],
            y: vec![3, 4],
        })
    }
    /// Attribute equivalent for a vctrs record.
    #[miniextendr(wrap = "vctrs")]
    pub fn wrapped_record_attr() -> WrappedRecord {
        wrapped_record().into_inner()
    }
}

/// Explicit optional class returns raise on None, including free functions.
/// @param present Whether to return a vector of boards.
#[miniextendr]
pub fn wrapped_optional_boards(present: bool) -> Option<Vec<WrapAsR6<WrapBoard>>> {
    present.then(|| vec![WrapAsR6(WrapBoard::new(1, 2))])
}

/// The equivalent attribute form of an optional free-function vector.
/// @param present Whether to return a vector of boards.
#[miniextendr(wrap = "r6")]
pub fn wrapped_optional_boards_attr(present: bool) -> Option<Vec<WrapBoard>> {
    present.then(|| vec![WrapBoard::new(1, 2)])
}

/// Result vectors preserve error transport and wrap each successful element.
/// @param fail Whether to return an error.
#[miniextendr(wrap = "r6")]
pub fn wrapped_result_boards(fail: bool) -> Result<Vec<WrapBoard>, String> {
    if fail {
        Err("board list failed".into())
    } else {
        Ok(vec![WrapBoard::new(1, 2)])
    }
}
