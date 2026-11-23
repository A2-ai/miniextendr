//! Test: a slice element with no R vector form names the derives that give
//! it one, and the `Coerce` + family derive route.

use miniextendr_macros::Altrep;

#[derive(Clone, Copy)]
struct Pip(u16);

#[derive(Altrep)]
struct Pips<const N: usize>([Pip; N]);

fn main() {}
