//! `serde_error_shape!` (#2207): a shape declared once for an error type, which
//! the generated `Err` arms look up under the crate's marker type. The arms
//! are driven here through the probe macros they expand to; no R runtime is
//! needed to build the condition parts.

#![cfg(feature = "serde")]

use miniextendr_api::RValue;
use miniextendr_api::condition::{
    ConditionData, ErrParts, RCondition, RConditionData, SerdeErrorShape, serde_err_parts,
};

// Defines this crate's marker type, `__MiniextendrInitMarker`, at the root.
miniextendr_api::miniextendr_init!(serde_error_shape_declared);

/// The marker of another crate, which declares nothing.
enum OtherCrate {}

/// An error that classes itself through `RConditionData` too.
struct Quota {
    n: u32,
}

impl serde::Serialize for Quota {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("Quota", 2)?;
        st.serialize_field("kind", "exceeded")?;
        st.serialize_field("n", &self.n)?;
        st.end()
    }
}

impl std::fmt::Display for Quota {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "quota exceeded by {}", self.n)
    }
}

impl RConditionData for Quota {
    fn message(&self) -> String {
        self.to_string()
    }

    fn class(&self) -> Vec<String> {
        vec!["by_impl".to_string()]
    }

    fn data(&self) -> Option<ConditionData> {
        None
    }
}

const QUOTA: SerdeErrorShape = SerdeErrorShape::new().prefix("quota").as_double(&["n"]);

miniextendr_api::serde_error_shape!(Quota = QUOTA);

// `String` is std's: a declaration may name a type from any crate.
miniextendr_api::serde_error_shape!(String = SerdeErrorShape::new().prefix("text"));

fn field<'a>(parts: &'a ErrParts, name: &str) -> &'a RValue {
    parts
        .data
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("no field `{name}` in {:?}", parts.data))
}

#[test]
fn the_probe_applies_the_declared_shape_before_rconditiondata() {
    let quota = Quota { n: 3 };
    let parts =
        miniextendr_api::__mx_result_err_parts!(quota, "pkg_error", crate::__MiniextendrInitMarker);
    assert_eq!(parts.class, ["quota_exceeded", "quota"]);
    assert_eq!(parts.message, "quota exceeded by 3");
    assert!(matches!(field(&parts, "n"), RValue::Double(v) if v == &[3.0]));
}

#[test]
fn another_crates_marker_does_not_see_the_declaration() {
    let quota = Quota { n: 3 };
    let parts = miniextendr_api::__mx_result_err_parts!(quota, "pkg_error", OtherCrate);
    assert_eq!(parts.class, ["by_impl"]);

    let text = String::from("boom");
    let parts = miniextendr_api::__mx_result_err_parts!(text, "pkg_error", OtherCrate);
    assert_eq!(parts.class, ["pkg_error"]);
}

#[test]
fn a_type_from_another_crate_takes_its_declared_shape() {
    let text = String::from("boom");
    let parts =
        miniextendr_api::__mx_result_err_parts!(text, "pkg_error", crate::__MiniextendrInitMarker);
    assert_eq!(parts.class, ["text"]);
    assert_eq!(parts.message, "boom");
}

#[test]
fn serde_error_options_apply_after_the_declared_shape() {
    const OWN: SerdeErrorShape = SerdeErrorShape::new().as_integer(&["n"]);
    let e = Quota { n: 3 };
    let declared = miniextendr_api::__mx_declared_serde_shape!(e, crate::__MiniextendrInitMarker);
    assert_eq!(declared, &QUOTA);
    let parts = serde_err_parts(&e, "pkg_error", &[declared, &OWN]);
    assert_eq!(parts.class, ["quota_exceeded", "quota"]);
    assert!(matches!(field(&parts, "n"), RValue::Integer(v) if v == &[Some(3)]));

    // No declaration under the marker: a shape that sets nothing.
    let declared = miniextendr_api::__mx_declared_serde_shape!(e, OtherCrate);
    assert_eq!(declared, &SerdeErrorShape::new());
    let parts = serde_err_parts(&e, "pkg_error", &[declared, &OWN]);
    assert_eq!(parts.class, ["pkg_error_exceeded", "pkg_error"]);
}

/// A `Serialize + Display` error without a declared shape or an
/// `RConditionData` impl.
struct Overdrawn {
    by: u32,
}

impl serde::Serialize for Overdrawn {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("Overdrawn", 2)?;
        st.serialize_field("kind", "overdrawn")?;
        st.serialize_field("by", &self.by)?;
        st.end()
    }
}

impl std::fmt::Display for Overdrawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "overdrawn by {}", self.by)
    }
}

/// The parts an entry point's `Err(e)` arm builds in this crate.
macro_rules! entry_point_parts {
    ($e:expr) => {
        miniextendr_api::__mx_result_err_parts!(
            $e,
            concat!(env!("CARGO_CRATE_NAME"), "_error"),
            crate::__MiniextendrInitMarker
        )
    };
}

/// `serde_condition!` (#2217) builds what an entry point returning `Err(e)`
/// raises: same message, classes, fields and call choice, as an error.
fn assert_entry_point_condition(condition: &RCondition, parts: &ErrParts) {
    assert!(
        matches!(condition, RCondition::Error { .. }),
        "{condition:?}"
    );
    assert_eq!(RConditionData::message(condition), parts.message);
    assert_eq!(condition.class(), parts.class);
    assert_eq!(
        format!("{:?}", condition.data()),
        format!("{:?}", parts.data)
    );
    assert_eq!(condition.call(), parts.call);
}

#[test]
fn serde_condition_applies_the_declared_shape() {
    let quota = Quota { n: 3 };
    let condition = miniextendr_api::serde_condition!(&quota);
    assert_eq!(condition.class(), ["quota_exceeded", "quota"]);
    assert_eq!(condition.message(), "quota exceeded by 3");
    let data = condition.data().expect("data");
    assert_eq!(data[0].0, "n");
    assert!(matches!(&data[0].1, RValue::Double(v) if v == &[3.0]));
    assert_entry_point_condition(&condition, &entry_point_parts!(quota));

    // A type from another crate, declared here.
    let text = String::from("boom");
    let condition = miniextendr_api::serde_condition!(&text);
    assert_eq!(condition.class(), ["text"]);
    assert_entry_point_condition(&condition, &entry_point_parts!(text));
}

#[test]
fn serde_condition_without_a_declaration_takes_the_crate_family_class() {
    let e = Overdrawn { by: 4 };
    let condition = miniextendr_api::serde_condition!(&e);
    assert_eq!(
        condition.class(),
        [
            "serde_error_shape_declared_error_overdrawn",
            "serde_error_shape_declared_error"
        ]
    );
    assert_eq!(condition.message(), "overdrawn by 4");
    assert_entry_point_condition(&condition, &entry_point_parts!(e));
}

#[test]
fn serde_condition_is_in_the_prelude() {
    use miniextendr_api::prelude::serde_condition;
    let condition = serde_condition!(&Overdrawn { by: 1 });
    assert_eq!(condition.message(), "overdrawn by 1");
}

#[test]
fn serde_condition_takes_a_temporary() {
    let by = std::hint::black_box(2);
    let condition = miniextendr_api::serde_condition!(&Overdrawn { by });
    assert_eq!(condition.message(), "overdrawn by 2");
}
