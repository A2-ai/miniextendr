use super::*;

#[test]
fn bigint_from_str() {
    let n = BigInt::from_str("12345678901234567890").unwrap();
    assert_eq!(ToString::to_string(&n), "12345678901234567890");
}

#[test]
fn rbigintops_sign() {
    let pos = BigInt::from_str("123").unwrap();
    let neg = BigInt::from_str("-456").unwrap();
    let zero = BigInt::from_str("0").unwrap();

    assert!(RBigIntOps::is_positive(&pos));
    assert!(!RBigIntOps::is_negative(&pos));
    assert_eq!(RBigIntOps::sign(&pos), 1);

    assert!(!RBigIntOps::is_positive(&neg));
    assert!(RBigIntOps::is_negative(&neg));
    assert_eq!(RBigIntOps::sign(&neg), -1);

    assert!(RBigIntOps::is_zero(&zero));
    assert_eq!(RBigIntOps::sign(&zero), 0);
}

#[test]
fn rbigintops_arithmetic() {
    let a = BigInt::from_str("1000000000000").unwrap();

    let sum = RBigIntOps::add_str(&a, "1").unwrap();
    assert_eq!(ToString::to_string(&sum), "1000000000001");

    let diff = RBigIntOps::sub_str(&a, "1").unwrap();
    assert_eq!(ToString::to_string(&diff), "999999999999");

    let prod = RBigIntOps::mul_str(&a, "2").unwrap();
    assert_eq!(ToString::to_string(&prod), "2000000000000");

    let quot = RBigIntOps::div_str(&a, "1000").unwrap();
    assert_eq!(ToString::to_string(&quot), "1000000000");

    let rem = RBigIntOps::rem_str(&a, "7").unwrap();
    assert_eq!(ToString::to_string(&rem), "1"); // 1000000000000 % 7 = 1
}

#[test]
fn rbigintops_pow_gcd() {
    let n = BigInt::from_str("2").unwrap();
    let pow10 = RBigIntOps::pow(&n, 10);
    assert_eq!(ToString::to_string(&pow10), "1024");

    let a = BigInt::from_str("48").unwrap();
    let gcd = RBigIntOps::gcd_str(&a, "18").unwrap();
    assert_eq!(ToString::to_string(&gcd), "6");
}

#[test]
fn rbigintops_abs_neg() {
    let neg = BigInt::from_str("-123").unwrap();
    assert_eq!(ToString::to_string(&RBigIntOps::abs(&neg)), "123");
    assert_eq!(ToString::to_string(&RBigIntOps::neg(&neg)), "123");

    let pos = BigInt::from_str("456").unwrap();
    assert_eq!(ToString::to_string(&RBigIntOps::abs(&pos)), "456");
    assert_eq!(ToString::to_string(&RBigIntOps::neg(&pos)), "-456");
}

#[test]
fn rbigintops_bits() {
    let n = BigInt::from_str("255").unwrap(); // 8 bits
    assert_eq!(RBigIntOps::bit_length(&n), 8);

    let n = BigInt::from_str("256").unwrap(); // 9 bits
    assert_eq!(RBigIntOps::bit_length(&n), 9);
}

#[test]
fn rbiguintops_basic() {
    let n = BigUint::from_str("12345").unwrap();
    assert!(!RBigUintOps::is_zero(&n));
    assert!(!RBigUintOps::is_one(&n));

    let zero = BigUint::from_str("0").unwrap();
    assert!(RBigUintOps::is_zero(&zero));

    let one = BigUint::from_str("1").unwrap();
    assert!(RBigUintOps::is_one(&one));
}

#[test]
fn rbiguintops_arithmetic() {
    let a = BigUint::from_str("1000").unwrap();

    let sum = RBigUintOps::add_str(&a, "234").unwrap();
    assert_eq!(ToString::to_string(&sum), "1234");

    let diff = RBigUintOps::sub_str(&a, "100").unwrap();
    assert_eq!(ToString::to_string(&diff), "900");

    // Subtraction that would be negative should error
    let result = RBigUintOps::sub_str(&a, "2000");
    assert!(result.is_err());
}

#[test]
fn rbigintops_division_by_zero() {
    let n = BigInt::from_str("100").unwrap();
    assert!(RBigIntOps::div_str(&n, "0").is_err());
    assert!(RBigIntOps::rem_str(&n, "0").is_err());
}

#[test]
fn rbigintbitops_and_or_xor() {
    let a = BigInt::from_str("255").unwrap(); // 0xFF
    let b = "15"; // 0x0F

    let and = RBigIntBitOps::bit_and_str(&a, b).unwrap();
    assert_eq!(ToString::to_string(&and), "15"); // 0xFF & 0x0F = 0x0F

    let or = RBigIntBitOps::bit_or_str(&a, "256").unwrap();
    assert_eq!(ToString::to_string(&or), "511"); // 0xFF | 0x100 = 0x1FF

    let xor = RBigIntBitOps::bit_xor_str(&a, "255").unwrap();
    assert_eq!(ToString::to_string(&xor), "0"); // 0xFF ^ 0xFF = 0
}

#[test]
fn rbigintbitops_shift() {
    let n = BigInt::from_str("1").unwrap();

    let shl = RBigIntBitOps::shl(&n, 8);
    assert_eq!(ToString::to_string(&shl), "256"); // 1 << 8

    let n256 = BigInt::from_str("256").unwrap();
    let shr = RBigIntBitOps::shr(&n256, 4);
    assert_eq!(ToString::to_string(&shr), "16"); // 256 >> 4
}

#[test]
fn rbigintbitops_count() {
    let n = BigInt::from_str("255").unwrap(); // 0xFF = 8 ones
    assert_eq!(RBigIntBitOps::count_ones(&n), 8);

    let n8 = BigInt::from_str("8").unwrap(); // 0b1000 = trailing zeros = 3
    assert_eq!(RBigIntBitOps::trailing_zeros(&n8), Some(3));

    let zero = BigInt::from_str("0").unwrap();
    assert_eq!(RBigIntBitOps::trailing_zeros(&zero), None);
}

#[test]
fn rbigintbitops_bit_access() {
    let n = BigInt::from_str("5").unwrap(); // 0b101

    assert!(RBigIntBitOps::bit(&n, 0)); // bit 0 is set
    assert!(!RBigIntBitOps::bit(&n, 1)); // bit 1 is not set
    assert!(RBigIntBitOps::bit(&n, 2)); // bit 2 is set
    assert!(!RBigIntBitOps::bit(&n, 3)); // bit 3 is not set

    let with_bit1 = RBigIntBitOps::set_bit(&n, 1);
    assert_eq!(ToString::to_string(&with_bit1), "7"); // 0b111

    let without_bit2 = RBigIntBitOps::clear_bit(&n, 2);
    assert_eq!(ToString::to_string(&without_bit2), "1"); // 0b001
}

#[test]
fn rbiguintbitops_basic() {
    let a = BigUint::from_str("255").unwrap();

    let and = RBigUintBitOps::bit_and_str(&a, "15").unwrap();
    assert_eq!(ToString::to_string(&and), "15");

    let shl = RBigUintBitOps::shl(&a, 4);
    assert_eq!(ToString::to_string(&shl), "4080"); // 255 << 4

    assert_eq!(RBigUintBitOps::count_ones(&a), 8);
}

#[test]
fn rbigintbitops_not() {
    let n = BigInt::from_str("0").unwrap();
    let not_n = RBigIntBitOps::bit_not(&n);
    assert_eq!(ToString::to_string(&not_n), "-1"); // ~0 = -1 in two's complement

    let one = BigInt::from_str("1").unwrap();
    let not_one = RBigIntBitOps::bit_not(&one);
    assert_eq!(ToString::to_string(&not_one), "-2"); // ~1 = -2
}
