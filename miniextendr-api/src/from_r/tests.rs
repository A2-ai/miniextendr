use super::*;

#[test]
fn real_na_follows_r_is_na() {
    assert!(NA_REAL.is_na());
    // `NA_real_ * 1` in R: the same payload with the quiet bit set.
    assert!(f64::from_bits(0x7FF8_0000_0000_07A2).is_na());
    // `-NA_real_`: the sign bit flips, R still reads NA.
    assert!(f64::from_bits(0xFFF0_0000_0000_07A2).is_na());
    assert!(!f64::NAN.is_na());
    assert!(!1954.0f64.is_na());
    assert!(!f64::from_bits(0x0000_0000_0000_07A2).is_na());
}

#[test]
fn batched_value_error_lists_values_and_one_based_elements() {
    let mut errors = BatchedErrors::default();
    errors.push(1, || "n/a".to_string());
    errors.push(4, || "<0.1".to_string());
    assert_eq!(
        errors.value_message("non-numeric"),
        r#"non-numeric value(s): "n/a", "<0.1" (elements 2, 5)"#
    );

    let mut one = BatchedErrors::default();
    one.push(0, || "a\"b\u{a0}".to_string());
    assert_eq!(
        one.value_message("non-numeric"),
        r#"non-numeric value(s): "a\"b\u{a0}" (element 1)"#
    );
}

#[test]
fn batched_value_error_caps_the_listing() {
    let mut errors = BatchedErrors::default();
    for i in 0..12 {
        errors.push(i, || format!("x{i}"));
    }
    let msg = errors.value_message("non-numeric");
    assert!(
        msg.starts_with(r#"non-numeric value(s): "x0", "x1","#),
        "{msg}"
    );
    assert!(
        msg.ends_with(r#""x9" (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 2 more"#),
        "{msg}"
    );
    assert!(!msg.contains("x10"), "{msg}");
}

#[test]
fn batched_element_error_groups_reasons_with_one_based_positions() {
    let mut errors = BatchedErrors::default();
    errors.push(1, || "NA is not allowed".to_string());
    errors.push(2, || "value out of range".to_string());
    errors.push(4, || "NA is not allowed".to_string());
    assert_eq!(
        errors.element_message(),
        "NA is not allowed (elements 2, 5); value out of range (element 3)"
    );

    let mut one = BatchedErrors::default();
    one.push(0, || "precision loss".to_string());
    let SexpError::InvalidValue(msg) = one.into_element_error() else {
        unreachable!()
    };
    assert_eq!(msg, "precision loss (element 1)");
}

#[test]
fn batched_element_error_caps_the_listing() {
    let mut errors = BatchedErrors::default();
    for i in 0..13 {
        errors.push(i, || "NA is not allowed".to_string());
    }
    assert_eq!(
        errors.element_message(),
        "NA is not allowed (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 3 more"
    );
}

/// `try_from_sexp_via_str_parse!` spells no associated item as
/// `Self::<Name>`, which is ambiguous for an enum with a variant of that
/// name (#1730): `Error` (`TryFromSexp`), `LABEL` (`ParseRStr`) and
/// `Inner` (`TryFromSexpElement`). Compiling this module is the test; the
/// downstream form is `miniextendr-macros/tests/ui/pass/assoc_item_variant_names.rs`.
mod str_parse_enum_with_assoc_item_variants {
    #[derive(Debug)]
    #[allow(clippy::upper_case_acronyms)]
    pub(super) enum Level {
        Error,
        Err,
        LABEL,
        Inner,
        Value,
    }

    impl std::str::FromStr for Level {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            match s {
                "error" => Ok(Level::Error),
                "err" => Ok(Level::Err),
                "label" => Ok(Level::LABEL),
                "inner" => Ok(Level::Inner),
                "value" => Ok(Level::Value),
                other => Err(format!("unknown level {other:?}")),
            }
        }
    }

    crate::try_from_sexp_via_str_parse!(Level, "level", |s| s.parse::<Level>());

    /// The container shapes come from the `TryFromSexpElement` blankets,
    /// and every shape reads only character input.
    #[test]
    fn containers_convert() {
        use crate::from_r::TryFromSexp;
        const {
            assert!(<Level as TryFromSexp>::CHARACTER_ONLY);
            assert!(<Option<Level> as TryFromSexp>::CHARACTER_ONLY);
            assert!(<Vec<Level> as TryFromSexp>::CHARACTER_ONLY);
            assert!(<Vec<Option<Level>> as TryFromSexp>::CHARACTER_ONLY);
        }
    }
}
