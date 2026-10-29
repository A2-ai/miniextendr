use super::*;

#[test]
fn blank_strings_match_r_is_blank_string() {
    for s in [
        "",
        " ",
        "   ",
        "\t\n",
        "\u{0B}\u{0C}\r",
        "\u{3000}",
        "\u{2028}",
        "\u{2003}",
    ] {
        assert!(is_blank(s), "{s:?} should be blank");
    }
    // Neither glibc nor macOS `iswspace` accepts NEL or the no-break spaces.
    for s in [
        "\u{0085}", "\u{00A0}", "\u{2007}", "\u{202F}", "x", " 1 ", "\0",
    ] {
        assert!(!is_blank(s), "{s:?} should not be blank");
    }
}

#[test]
fn na_token_allows_surrounding_blanks_only() {
    for s in ["NA", " NA", "NA ", "\tNA\n"] {
        assert!(is_na_token(s), "{s:?} should be the NA token");
    }
    for s in ["na", "N A", "NAN", "NA1", "<NA>", "\u{00A0}NA"] {
        assert!(!is_na_token(s), "{s:?} should not be the NA token");
    }
}

/// `no_na` refuses what the markers read as missing, and `NaN`.
#[test]
fn no_na_refuses_none_and_nan() {
    assert!(!AsNumeric(Some(2.5)).__mx_has_na());
    assert!(!AsNumeric(Some(f64::INFINITY)).__mx_has_na());
    assert!(AsNumeric(None).__mx_has_na());
    assert!(AsNumeric(Some(f64::NAN)).__mx_has_na());

    assert!(!AsNumericVec(vec![]).__mx_has_na());
    assert!(!AsNumericVec(vec![Some(1.0), Some(-0.0)]).__mx_has_na());
    assert!(AsNumericVec(vec![Some(1.0), None]).__mx_has_na());
    assert!(AsNumericVec(vec![Some(f64::NAN), Some(1.0)]).__mx_has_na());
}

/// The layers a marker comes in forward the check: `NULL` (`None`) and an
/// omitted argument (`Absent`) pass, a present marker is asked. A type
/// that is not a marker keeps the default `false`.
#[test]
fn no_na_is_forwarded_through_the_layers() {
    use crate::Missing;
    use std::collections::{BTreeMap, HashMap};

    assert!(!None::<AsNumeric>.__mx_has_na());
    assert!(!Some(AsNumeric(Some(1.0))).__mx_has_na());
    assert!(Some(AsNumeric(None)).__mx_has_na());
    assert!(Some(AsNumericVec(vec![None])).__mx_has_na());

    assert!(!Missing::<AsNumeric>::Absent.__mx_has_na());
    assert!(!Missing::Present(AsNumeric(Some(1.0))).__mx_has_na());
    assert!(Missing::Present(AsNumeric(None)).__mx_has_na());
    assert!(!Missing::<Option<AsNumeric>>::Present(None).__mx_has_na());
    assert!(Missing::Present(Some(AsNumeric(None))).__mx_has_na());

    assert!(!Err::<AsNumeric, ()>(()).__mx_has_na());
    assert!(!Ok::<AsNumeric, ()>(AsNumeric(Some(1.0))).__mx_has_na());
    assert!(Ok::<AsNumeric, ()>(AsNumeric(None)).__mx_has_na());

    let mut map = HashMap::new();
    map.insert("a".to_string(), AsNumeric(Some(1.0)));
    assert!(!map.__mx_has_na());
    map.insert("b".to_string(), AsNumeric(None));
    assert!(map.__mx_has_na());
    assert!(Some(map).__mx_has_na());
    assert!(!None::<HashMap<String, AsNumeric>>.__mx_has_na());

    let tree: BTreeMap<String, AsNumericVec> =
        [("a".to_string(), AsNumericVec(vec![Some(1.0), None]))].into();
    assert!(tree.__mx_has_na());

    assert!(!Some(f64::NAN).__mx_has_na());
    assert!(!None::<f64>.__mx_has_na());
    assert!(!Missing::<f64>::Absent.__mx_has_na());
    assert!(!vec![None::<f64>].__mx_has_na());
}

#[test]
fn unread_rest_must_be_blank_utf8() {
    assert!(rest_is_blank(b""));
    assert!(rest_is_blank(b"  \t"));
    assert!(rest_is_blank("\u{3000}".as_bytes()));
    assert!(!rest_is_blank(b" x"));
    assert!(!rest_is_blank(b"\xa0"), "split multi-byte character");
}
