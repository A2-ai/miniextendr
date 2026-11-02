use super::*;
use crate::condition::{ERRORS_FIELD, into_r_err_parts};

#[test]
fn display_string_too_long() {
    let e = IntoRError::StringTooLong { len: 3_000_000_000 };
    assert!(e.to_string().contains("3000000000"));
    assert!(e.to_string().contains("i32 limit"));
}

#[test]
fn display_length_overflow() {
    let e = IntoRError::LengthOverflow { len: 42 };
    assert!(e.to_string().contains("42"));
}

#[test]
fn a_length_error_has_no_class_and_no_fields() {
    let e = IntoRError::StringTooLong { len: 3_000_000_000 };
    assert_eq!(e.message(), e.to_string());
    assert!(e.class().is_empty());
    assert!(e.data().is_none());
}

// region: helpers

/// An element's refusal with the given message, classes and one field.
struct Refusal {
    message: &'static str,
    class: &'static [&'static str],
}

impl RConditionData for Refusal {
    fn message(&self) -> String {
        self.message.to_owned()
    }

    fn class(&self) -> Vec<String> {
        self.class.iter().map(|class| (*class).to_owned()).collect()
    }

    fn data(&self) -> Option<ConditionData> {
        Some(vec![("value".to_owned(), RValue::from(7))])
    }
}

const SERDE: &[&str] = &["miniextendr_serde_error"];
const CAPPED: &[&str] = &["over_cap_error", "cap_error"];

fn serde_refusal() -> Refusal {
    Refusal {
        message: "map keys must be strings for R named lists",
        class: SERDE,
    }
}

fn elements(error: &IntoRError) -> &FailedElements {
    match error {
        IntoRError::Elements(failed) => failed,
        other => panic!("expected failed elements, got {other:?}"),
    }
}

fn field<'a>(list: &'a RValue, name: &str) -> &'a RValue {
    match list {
        RValue::List(pairs) => pairs
            .iter()
            .find(|(key, _)| key.as_deref() == Some(name))
            .map(|(_, value)| value)
            .unwrap_or_else(|| panic!("no field `{name}` in {list:?}")),
        other => panic!("expected a list, got {other:?}"),
    }
}

fn text(value: &RValue) -> Vec<String> {
    match value {
        RValue::Character(values) => values.iter().map(|v| v.clone().unwrap()).collect(),
        other => panic!("expected character, got {other:?}"),
    }
}

/// The `.mx_errors` entries of an error's condition data.
fn entries(error: &IntoRError) -> Vec<RValue> {
    let data = error.data().expect("failed elements carry `.mx_errors`");
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].0, ERRORS_FIELD);
    match &data[0].1 {
        RValue::List(entries) => entries
            .iter()
            .map(|(name, entry)| {
                assert!(name.is_none(), "entries are unnamed");
                entry.clone()
            })
            .collect(),
        other => panic!("expected a list of entries, got {other:?}"),
    }
}

fn entry_index(entry: &RValue) -> Vec<Option<i32>> {
    match field(entry, "index") {
        RValue::Integer(positions) => positions.clone(),
        other => panic!("expected an integer index, got {other:?}"),
    }
}

// endregion

// region: batching

#[test]
fn failing_fields_that_share_a_class_keep_it_and_every_condition() {
    let mut errors = ElementFailures::default();
    errors.push(1, &serde_refusal());
    errors.push(3, &serde_refusal());
    assert!(!errors.is_empty());
    let error = errors.into_error();

    assert_eq!(
        error.message(),
        "map keys must be strings for R named lists (elements 2, 4)"
    );
    assert_eq!(error.class(), vec!["miniextendr_serde_error".to_owned()]);
    let failed = elements(&error);
    assert_eq!(failed.elements().len(), 2);
    assert_eq!(failed.elements()[0].index(), &[2]);
    assert_eq!(failed.elements()[1].index(), &[4]);
    assert_eq!(
        failed.elements()[0].message(),
        "map keys must be strings for R named lists"
    );

    let entries = entries(&error);
    assert_eq!(entries.len(), 2);
    assert_eq!(entry_index(&entries[0]), vec![Some(2)]);
    assert_eq!(entry_index(&entries[1]), vec![Some(4)]);
    // Each entry's condition is the element's own tagged value: its message,
    // `kind = "result_err"`, its classes and its fields.
    let condition = field(&entries[0], "condition");
    assert_eq!(
        text(field(condition, "error")),
        vec!["map keys must be strings for R named lists"]
    );
    assert_eq!(text(field(condition, "kind")), vec!["result_err"]);
    assert_eq!(
        text(field(condition, "class")),
        vec!["miniextendr_serde_error"]
    );
    assert!(matches!(field(condition, "call"), RValue::Null));
    assert!(matches!(
        field(field(condition, "data"), "value"),
        RValue::Integer(v) if v == &[Some(7)]
    ));
}

#[test]
fn failing_fields_of_different_classes_have_no_shared_class() {
    let mut errors = ElementFailures::default();
    errors.push(0, &serde_refusal());
    errors.push(
        1,
        &Refusal {
            message: "20 exceeds the cap 10",
            class: CAPPED,
        },
    );
    let error = errors.into_error();

    assert_eq!(
        error.message(),
        "map keys must be strings for R named lists (element 1); \
         20 exceeds the cap 10 (element 2)"
    );
    assert!(error.class().is_empty());
    let entries = entries(&error);
    assert_eq!(entries.len(), 2);
    assert_eq!(
        text(field(field(&entries[0], "condition"), "class")),
        vec!["miniextendr_serde_error"]
    );
    assert_eq!(
        text(field(field(&entries[1], "condition"), "class")),
        vec!["over_cap_error", "cap_error"]
    );
}

#[test]
fn an_unclassed_failure_among_classed_ones_drops_the_shared_class() {
    let mut errors = ElementFailures::default();
    errors.push(0, &serde_refusal());
    errors.push(1, &IntoRError::StringTooLong { len: 3_000_000_000 });
    errors.push(2, &serde_refusal());
    let error = errors.into_error();
    assert!(error.class().is_empty());
    // The unclassed element's condition has no class slot.
    let entries = entries(&error);
    assert!(matches!(
        field(field(&entries[1], "condition"), "class"),
        RValue::Null
    ));
}

#[test]
fn rows_of_tuples_flatten_with_the_row_before_the_field() {
    // Row 3 (0-based 2): fields 2 and 3 fail; row 5: field 1 fails.
    let row = |fields: &[usize]| {
        let mut errors = ElementFailures::default();
        for &field in fields {
            errors.push(field, &serde_refusal());
        }
        errors.into_error()
    };
    let mut rows = ElementFailures::default();
    rows.push_row(2, row(&[1, 2]));
    rows.push_row(4, row(&[0]));
    let error = rows.into_error();

    // The message is the rows' batched messages, as before.
    assert_eq!(
        error.message(),
        "map keys must be strings for R named lists (elements 2, 3) (element 3); \
         map keys must be strings for R named lists (element 1) (element 5)"
    );
    assert_eq!(error.class(), vec!["miniextendr_serde_error".to_owned()]);
    let entries = entries(&error);
    let indices: Vec<_> = entries.iter().map(entry_index).collect();
    assert_eq!(
        indices,
        vec![
            vec![Some(3), Some(2)],
            vec![Some(3), Some(3)],
            vec![Some(5), Some(1)]
        ]
    );
}

#[test]
fn a_row_whose_fields_differ_drops_the_shared_class_of_the_rows() {
    let mut mixed = ElementFailures::default();
    mixed.push(0, &serde_refusal());
    mixed.push(
        1,
        &Refusal {
            message: "20 exceeds the cap 10",
            class: CAPPED,
        },
    );
    let mut same = ElementFailures::default();
    same.push(0, &serde_refusal());

    let mut rows = ElementFailures::default();
    rows.push_row(0, same.into_error());
    rows.push_row(1, mixed.into_error());
    let error = rows.into_error();
    assert!(error.class().is_empty());
    assert_eq!(entries(&error).len(), 3);
}

#[test]
fn entries_list_every_failure_past_the_message_cap() {
    let mut errors = ElementFailures::default();
    for index in 0..11 {
        errors.push(index, &serde_refusal());
    }
    // The twelfth failure is past the message's cap: the message counts it,
    // `e$errors` still has its entry, and its class still decides.
    errors.push(
        11,
        &Refusal {
            message: "20 exceeds the cap 10",
            class: CAPPED,
        },
    );
    let error = errors.into_error();
    assert!(
        error.message().ends_with("; and 2 more"),
        "{}",
        error.message()
    );
    assert_eq!(elements(&error).elements().len(), 12);
    assert_eq!(entries(&error).len(), 12);
    assert!(error.class().is_empty());
}

// endregion

// region: the condition parts R gets

#[test]
fn failed_elements_pass_the_reserved_field_check() {
    let mut errors = ElementFailures::default();
    errors.push(0, &serde_refusal());
    let error = errors.into_error();
    // The `.mx_errors` field is the framework's own: the parts of the error
    // keep it instead of refusing a reserved name.
    let parts = into_r_err_parts(&error);
    assert_eq!(parts.class, vec!["miniextendr_serde_error".to_owned()]);
    let data = parts.data.expect("the failed elements");
    assert_eq!(data[0].0, ERRORS_FIELD);
}

#[test]
fn a_users_errors_field_is_refused() {
    struct Spoof;
    impl RConditionData for Spoof {
        fn message(&self) -> String {
            "spoof".to_owned()
        }
        fn data(&self) -> Option<ConditionData> {
            Some(vec![(ERRORS_FIELD.to_owned(), RValue::Null)])
        }
    }
    let panic = std::panic::catch_unwind(|| into_r_err_parts(&Spoof)).unwrap_err();
    let message = panic
        .downcast_ref::<String>()
        .expect("a formatted panic message");
    assert!(message.contains("`.mx_errors`"), "{message}");
}

#[test]
fn a_nested_tuple_field_keeps_its_own_failed_elements() {
    // A tuple inside a tuple field: the inner error is a generic field
    // error, so its entries nest under its own condition.
    let mut inner = ElementFailures::default();
    inner.push(1, &serde_refusal());
    let mut outer = ElementFailures::default();
    outer.push(0, &inner.into_error());
    let error = outer.into_error();
    assert_eq!(error.class(), vec!["miniextendr_serde_error".to_owned()]);
    let entries = entries(&error);
    assert_eq!(entry_index(&entries[0]), vec![Some(1)]);
    let condition = field(&entries[0], "condition");
    let nested = field(field(condition, "data"), ERRORS_FIELD);
    assert!(matches!(nested, RValue::List(items) if items.len() == 1));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn a_position_past_rs_integers_is_a_double() {
    let big = usize::try_from(i32::MAX).unwrap() + 1;
    match index_rvalue(&[big, 2]) {
        RValue::Double(positions) => assert_eq!(positions, vec![2_147_483_648.0, 2.0]),
        other => panic!("expected a double index, got {other:?}"),
    }
    assert!(matches!(index_rvalue(&[2]), RValue::Integer(v) if v == vec![Some(2)]));
}

// endregion
