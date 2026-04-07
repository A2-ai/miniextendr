//! Bitvec adapter tests
use miniextendr_api::bitvec_impl::{
    RBitVec, bitvec_count_ones, bitvec_count_zeros, bitvec_from_bools, bitvec_to_bools,
};
use miniextendr_api::miniextendr;

/// Test RBitVec roundtrip through R.
/// @param bits Bit vector to roundtrip.
#[miniextendr]
pub fn bitvec_roundtrip(bits: RBitVec) -> RBitVec {
    bits
}

/// Test counting the number of set bits (ones) in a bit vector.
/// @param bits Bit vector to count ones in.
#[miniextendr]
pub fn bitvec_ones(bits: RBitVec) -> i32 {
    bitvec_count_ones(&bits) as i32
}

/// Test counting the number of unset bits (zeros) in a bit vector.
/// @param bits Bit vector to count zeros in.
#[miniextendr]
pub fn bitvec_zeros(bits: RBitVec) -> i32 {
    bitvec_count_zeros(&bits) as i32
}

/// Test creating a bit vector from a logical vector.
/// @param bools Logical vector to convert.
#[miniextendr]
pub fn bitvec_from_vec(bools: Vec<bool>) -> RBitVec {
    bitvec_from_bools(&bools)
}

/// Test converting a bit vector back to a logical vector.
/// @param bits Bit vector to convert.
#[miniextendr]
pub fn bitvec_to_vec(bits: RBitVec) -> Vec<bool> {
    bitvec_to_bools(&bits)
}

/// Test getting the length of a bit vector.
/// @param bits Bit vector to measure.
#[miniextendr]
pub fn bitvec_len(bits: RBitVec) -> i32 {
    bits.len() as i32
}

/// Test creating and roundtripping an empty bit vector.
#[miniextendr]
pub fn bitvec_empty() -> RBitVec {
    RBitVec::new()
}

/// Test creating an all-ones bit vector of given length.
/// @param n Number of bits.
#[miniextendr]
pub fn bitvec_all_ones(n: i32) -> RBitVec {
    RBitVec::repeat(true, n as usize)
}

/// Test creating an all-zeros bit vector of given length.
/// @param n Number of bits.
#[miniextendr]
pub fn bitvec_all_zeros(n: i32) -> RBitVec {
    RBitVec::repeat(false, n as usize)
}

/// Test toggling (flipping) all bits in a bit vector.
/// @param bits Bit vector to flip.
#[miniextendr]
pub fn bitvec_toggle(bits: RBitVec) -> RBitVec {
    let mut result = bits;
    for mut bit in result.iter_mut() {
        *bit = !*bit;
    }
    result
}
