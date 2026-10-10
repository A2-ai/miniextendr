//! Built-in ALTREP container fixtures.

use miniextendr_api::{Altrep, Rcomplex, miniextendr};
use std::collections::VecDeque;

fn shifted<T>(values: Vec<T>, shift: usize) -> VecDeque<T> {
    let mut values = VecDeque::from(values);
    if !values.is_empty() {
        values.rotate_left(shift % values.len());
    }
    values
}

/// Return an integer ring buffer as ALTREP.
///
/// Retains the Rust deque. Element and region reads do not flatten its two
/// segments; a request for a contiguous pointer rearranges the existing buffer.
/// @param values Input values.
/// @param shift Number of positions to rotate left, modulo the input length.
/// @return An ALTREP vector in the deque's logical order.
#[miniextendr]
pub fn altrep_deque_integers(values: Vec<i32>, shift: usize) -> Altrep<VecDeque<i32>> {
    Altrep(shifted(values, shift))
}

/// Return a real ring buffer as ALTREP.
/// @rdname altrep_deque_integers
#[miniextendr]
pub fn altrep_deque_reals(values: Vec<f64>, shift: usize) -> Altrep<VecDeque<f64>> {
    Altrep(shifted(values, shift))
}

/// Return a raw ring buffer as ALTREP.
/// @rdname altrep_deque_integers
#[miniextendr]
pub fn altrep_deque_raw(values: Vec<u8>, shift: usize) -> Altrep<VecDeque<u8>> {
    Altrep(shifted(values, shift))
}

/// Return a complex ring buffer as ALTREP.
/// @rdname altrep_deque_integers
#[miniextendr]
pub fn altrep_deque_complex(values: Vec<Rcomplex>, shift: usize) -> Altrep<VecDeque<Rcomplex>> {
    Altrep(shifted(values, shift))
}
