use super::*;

#[test]
fn test_bitvec_from_bools() {
    let bits = bitvec_from_bools(&[true, false, true, true, false]);
    assert_eq!(bits.len(), 5);
    assert!(bits[0]);
    assert!(!bits[1]);
    assert!(bits[2]);
    assert!(bits[3]);
    assert!(!bits[4]);
}

#[test]
fn test_bitvec_to_bools() {
    let mut bits = RBitVec::new();
    bits.push(true);
    bits.push(false);
    bits.push(true);

    let bools = bitvec_to_bools(&bits);
    assert_eq!(bools, vec![true, false, true]);
}

#[test]
fn test_bitvec_count() {
    let bits = bitvec_from_bools(&[true, false, true, true, false]);
    assert_eq!(bitvec_count_ones(&bits), 3);
    assert_eq!(bitvec_count_zeros(&bits), 2);
}

#[test]
fn test_bitvec_empty() {
    let bits = RBitVec::new();
    assert_eq!(bits.len(), 0);
    assert_eq!(bitvec_count_ones(&bits), 0);
}

#[test]
fn test_bitvec_repeat() {
    let bits = RBitVec::repeat(true, 10);
    assert_eq!(bits.len(), 10);
    assert_eq!(bitvec_count_ones(&bits), 10);

    let bits = RBitVec::repeat(false, 10);
    assert_eq!(bitvec_count_zeros(&bits), 10);
}

#[test]
fn test_bitvec_msb0() {
    let mut bits = BitVec::<u8, Msb0>::new();
    bits.push(true);
    bits.push(false);
    bits.push(true);

    assert!(bits[0]);
    assert!(!bits[1]);
    assert!(bits[2]);
}
