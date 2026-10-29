use super::*;

#[test]
fn recognizes_truthy_values_case_insensitively() {
    for v in [
        "yes", "YES", "true", "TRUE", "True", "1", "on", "ON", " true ",
    ] {
        assert_eq!(parse_bool(v), Some(true), "{v:?}");
    }
}

#[test]
fn recognizes_falsy_values_case_insensitively() {
    for v in ["no", "NO", "false", "FALSE", "0", "off", "OFF", "", "  "] {
        assert_eq!(parse_bool(v), Some(false), "{v:?}");
    }
}

#[test]
fn returns_none_for_unrecognized() {
    for v in ["garbage", "2", "tru", "yesno"] {
        assert_eq!(parse_bool(v), None, "{v:?}");
    }
}
