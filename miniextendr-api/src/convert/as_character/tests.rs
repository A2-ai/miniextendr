use super::*;

/// `no_na` refuses what the markers read as `None`; the strings `"NA"`,
/// `""` and `"NaN"` are values here.
#[test]
fn no_na_refuses_none_only() {
    assert!(!AsCharacter(Some("NA".to_string())).__mx_has_na());
    assert!(!AsCharacter(Some(String::new())).__mx_has_na());
    assert!(AsCharacter(None).__mx_has_na());

    let values =
        |v: &[Option<&str>]| AsCharacterVec(v.iter().map(|s| s.map(str::to_string)).collect());
    assert!(!values(&[]).__mx_has_na());
    assert!(!values(&[Some("a"), Some("NaN"), Some("")]).__mx_has_na());
    assert!(values(&[Some("a"), None]).__mx_has_na());

    assert!(!None::<AsCharacter>.__mx_has_na());
    assert!(Some(AsCharacter(None)).__mx_has_na());
    assert!(!crate::Missing::<AsCharacterVec>::Absent.__mx_has_na());
    assert!(crate::Missing::Present(values(&[None])).__mx_has_na());
}
