//! Test fixtures for streaming ALTREP data types.
//!
//! Uses the direct registration pattern: data struct with #[derive(Altrep)].

use miniextendr_api::altrep_data::{
    AltIntegerData, AltRealData, AltrepLen, StreamingIntData, StreamingRealData,
};
use miniextendr_api::prelude::*;

// region: StreamingIntData

type IntReader = Box<dyn Fn(usize, &mut [i32]) -> usize>;

#[derive(miniextendr_api::Altrep)]
#[altrep(class = "StreamingIntRange")]
pub struct StreamingIntRangeData {
    inner: StreamingIntData<IntReader>,
}

impl AltrepLen for StreamingIntRangeData {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltIntegerData for StreamingIntRangeData {
    fn elt(&self, i: usize) -> i32 {
        self.inner.elt(i)
    }
}

miniextendr_api::impl_altinteger_from_data!(StreamingIntRangeData);

/// Create a streaming integer ALTREP `1..=n`.
#[miniextendr]
pub fn streaming_int_range(n: i32) -> StreamingIntRangeData {
    let len = n as usize;
    StreamingIntRangeData {
        inner: StreamingIntData::new(
            len,
            64,
            Box::new(move |start, buf| {
                let count = buf.len().min(len.saturating_sub(start));
                for (i, slot) in buf[..count].iter_mut().enumerate() {
                    *slot = (start + i + 1) as i32;
                }
                count
            }),
        ),
    }
}

// endregion

// region: StreamingRealData

type RealReader = Box<dyn Fn(usize, &mut [f64]) -> usize>;

#[derive(miniextendr_api::Altrep)]
#[altrep(class = "StreamingRealSquares")]
pub struct StreamingRealSquaresData {
    inner: StreamingRealData<RealReader>,
}

impl AltrepLen for StreamingRealSquaresData {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltRealData for StreamingRealSquaresData {
    fn elt(&self, i: usize) -> f64 {
        self.inner.elt(i)
    }
}

miniextendr_api::impl_altreal_from_data!(StreamingRealSquaresData);

/// Create a streaming real ALTREP `1^2, 2^2, ..., n^2`.
#[miniextendr]
pub fn streaming_real_squares(n: i32) -> StreamingRealSquaresData {
    let len = n as usize;
    StreamingRealSquaresData {
        inner: StreamingRealData::new(
            len,
            32,
            Box::new(move |start, buf| {
                let count = buf.len().min(len.saturating_sub(start));
                for (i, slot) in buf[..count].iter_mut().enumerate() {
                    let v = (start + i + 1) as f64;
                    *slot = v * v;
                }
                count
            }),
        ),
    }
}

// endregion
