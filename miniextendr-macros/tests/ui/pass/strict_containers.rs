//! Strict returns reach the checked path for every contiguous container shape.
#![allow(dead_code)]
use miniextendr_api::miniextendr;

#[miniextendr(strict)]
pub fn strict_slice() -> &'static [i64] { &[1, 2] }
#[miniextendr(strict)]
pub fn strict_box() -> Box<[u64]> { vec![1, 2].into_boxed_slice() }
#[miniextendr(strict)]
pub fn strict_array() -> [usize; 2] { [1, 2] }
#[miniextendr(strict)]
pub fn strict_slice_rows() -> Vec<&'static [isize]> { vec![&[1, 2]] }
#[miniextendr(strict)]
pub fn strict_optional_slice_rows() -> Vec<Option<&'static [i64]>> { vec![None, Some(&[1, 2])] }
#[miniextendr(strict)]
pub fn strict_box_rows() -> Vec<Box<[u64]>> { vec![vec![1, 2].into_boxed_slice()] }
#[miniextendr(strict)]
pub fn strict_array_rows() -> Vec<[usize; 2]> { vec![[1, 2]] }
#[miniextendr(strict)]
pub fn strict_optional_box() -> Option<Box<[i64]>> { None }

fn main() {}
