use super::*;

#[test]
fn logical_roundtrip() {
    assert_eq!(Logical::False.to_r_int(), 0);
    assert_eq!(Logical::True.to_r_int(), 1);
    assert_eq!(Logical::Na.to_r_int(), i32::MIN);

    assert_eq!(Logical::from_r_int(0), Logical::False);
    assert_eq!(Logical::from_r_int(1), Logical::True);
    assert_eq!(Logical::from_r_int(i32::MIN), Logical::Na);
}

#[test]
fn sortedness_roundtrip() {
    let cases = [
        Sortedness::Unknown,
        Sortedness::KnownUnsorted,
        Sortedness::Increasing,
        Sortedness::Decreasing,
        Sortedness::IncreasingNaFirst,
        Sortedness::DecreasingNaFirst,
    ];
    for s in cases {
        let code = s.to_r_int();
        assert_eq!(Sortedness::from_r_int(code), s);
    }
}
