//! Bitvec adapter tests
use miniextendr_api::bitvec_impl::{bitvec_count_ones, bitvec_count_zeros, bitvec_from_bools, bitvec_to_bools, RBitVec};
use miniextendr_api::{miniextendr, miniextendr_module};

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

miniextendr_module! {
    mod bitvec_adapter_tests;
    fn bitvec_roundtrip;
    fn bitvec_ones;
    fn bitvec_zeros;
    fn bitvec_from_vec;
    fn bitvec_to_vec;
    fn bitvec_len;
}
