//! Bitvec adapter tests
use miniextendr_api::bitvec_impl::{
    RBitVec, bitvec_count_ones, bitvec_count_zeros, bitvec_from_bools, bitvec_to_bools,
};
use miniextendr_api::miniextendr;

/// @noRd
#[miniextendr]
pub fn bitvec_roundtrip(bits: RBitVec) -> RBitVec {
    bits
}

/// @noRd
#[miniextendr]
pub fn bitvec_ones(bits: RBitVec) -> i32 {
    bitvec_count_ones(&bits) as i32
}

/// @noRd
#[miniextendr]
pub fn bitvec_zeros(bits: RBitVec) -> i32 {
    bitvec_count_zeros(&bits) as i32
}

/// @noRd
#[miniextendr]
pub fn bitvec_from_vec(bools: Vec<bool>) -> RBitVec {
    bitvec_from_bools(&bools)
}

/// @noRd
#[miniextendr]
pub fn bitvec_to_vec(bits: RBitVec) -> Vec<bool> {
    bitvec_to_bools(&bits)
}

/// @noRd
#[miniextendr]
pub fn bitvec_len(bits: RBitVec) -> i32 {
    bits.len() as i32
}

/// Empty bitvec roundtrip
/// @noRd
#[miniextendr]
pub fn bitvec_empty() -> RBitVec {
    RBitVec::new()
}

/// All-ones bitvec
/// @noRd
#[miniextendr]
pub fn bitvec_all_ones(n: i32) -> RBitVec {
    RBitVec::repeat(true, n as usize)
}

/// All-zeros bitvec
/// @noRd
#[miniextendr]
pub fn bitvec_all_zeros(n: i32) -> RBitVec {
    RBitVec::repeat(false, n as usize)
}

/// Toggle bits: flip all bits in a bitvec
/// @noRd
#[miniextendr]
pub fn bitvec_toggle(bits: RBitVec) -> RBitVec {
    let mut result = bits;
    for mut bit in result.iter_mut() {
        *bit = !*bit;
    }
    result
}
