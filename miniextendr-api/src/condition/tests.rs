use super::{ConditionData, RCondition};
use crate::RValue;

/// The marker type of a crate that declares no `serde_error_shape!`, for
/// the `Err` probe's marker argument (resolved only under `serde`).
#[cfg(feature = "serde")]
enum NoShapes {}

/// Catch the `panic_any(RCondition)` raised by a macro invocation and
/// return the payload. No R runtime needed — the macros panic before any
/// R API call.
fn catch(f: impl FnOnce() + std::panic::UnwindSafe) -> RCondition {
    let payload = std::panic::catch_unwind(f).expect_err("macro must panic");
    *payload
        .downcast::<RCondition>()
        .expect("payload must be RCondition")
}

fn assert_data(data: &Option<ConditionData>, expected: &[(&str, RValue)]) {
    let data = data.as_ref().expect("data must be Some");
    assert_eq!(data.len(), expected.len());
    for ((name, value), (exp_name, exp_value)) in data.iter().zip(expected) {
        assert_eq!(name, exp_name);
        // RValue has no PartialEq (f64); compare via Debug.
        assert_eq!(format!("{value:?}"), format!("{exp_value:?}"));
    }
}

/// `catch_condition` returns every condition kind as `Err` with its
/// classes and data, and the value on success.
#[test]
fn catch_condition_returns_each_kind() {
    use super::catch_condition;
    use crate::error_value::kind;
    assert_eq!(catch_condition(|| 3).unwrap(), 3);
    let error = catch_condition::<()>(|| {
        crate::error!(
            class = ["member", "family"],
            data = ("n", 2),
            "failed {}",
            1
        )
    })
    .unwrap_err();
    let (error_kind, parts) = error.into_parts();
    assert_eq!(error_kind, kind::ERROR);
    assert_eq!(parts.message, "failed 1");
    assert_eq!(parts.class, ["member", "family"]);
    assert_data(&parts.data, &[("n", RValue::from(2))]);
    let kinds = [
        catch_condition::<()>(|| crate::warning!("w")),
        catch_condition::<()>(|| crate::message!("m")),
        catch_condition::<()>(|| crate::condition!("c")),
        catch_condition::<()>(|| {
            RCondition::arg_error("x", "bad".to_string(), super::ConditionCall::Inherit).raise()
        }),
    ]
    .map(|caught| caught.unwrap_err().into_parts().0);
    assert_eq!(
        kinds,
        [
            kind::WARNING,
            kind::MESSAGE,
            kind::CONDITION,
            kind::CONVERSION
        ]
    );
}

/// Any other payload, a plain `panic!()` included, resumes unchanged.
#[test]
fn catch_condition_resumes_other_panics() {
    use super::catch_condition;
    struct Foreign(u8);
    let payload = std::panic::catch_unwind(|| catch_condition::<()>(|| panic!("plain")))
        .expect_err("a plain panic must resume");
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"plain"));
    let payload =
        std::panic::catch_unwind(|| catch_condition::<()>(|| std::panic::panic_any(Foreign(9))))
            .expect_err("a foreign payload must resume");
    assert_eq!(payload.downcast_ref::<Foreign>().map(|f| f.0), Some(9));
}

/// `arg_error!` (#1740): the conversion kind, the crate-class marker as
/// the only class, `param` as the only field, and the call choice.
#[test]
fn arg_error_payload() {
    use super::{CONVERSION_ERROR_CLASS_MARKER, ConditionCall};
    let choices = r#""fast", "slow""#;
    let check = |cond: RCondition, call: ConditionCall| {
        let RCondition::Conversion {
            message,
            class,
            data,
            call: got_call,
        } = cond
        else {
            panic!("expected RCondition::Conversion, got {cond:?}");
        };
        assert_eq!(message, r#"'mode' should be one of "fast", "slow""#);
        assert_eq!(class, [CONVERSION_ERROR_CLASS_MARKER]);
        assert_data(&data, &[("param", RValue::from("mode"))]);
        assert_eq!(got_call, call);
    };
    check(
        catch(|| crate::arg_error!(param = "mode", "'mode' should be one of {choices}")),
        ConditionCall::Inherit,
    );
    let param = String::from("mode");
    check(
        catch(move || {
            crate::arg_error!(
                call = none,
                param = param,
                "'mode' should be one of {}",
                choices
            )
        }),
        ConditionCall::None,
    );
    check(
        catch(|| {
            super::ArgError::new("mode", format!("'mode' should be one of {choices}")).raise()
        }),
        ConditionCall::Inherit,
    );

    let (kind, parts) = catch(|| crate::arg_error!(param = "x", "bad")).into_parts();
    assert_eq!(kind, crate::error_value::kind::CONVERSION);
    assert_eq!(parts.message, "bad");
}

/// A raising guard resolves the marker in Rust: the crate's classes take
/// its place, every other class keeps its position, and with no crate
/// classes the marker just goes.
#[test]
fn raising_guard_resolves_the_conversion_class_marker() {
    use super::CONVERSION_ERROR_CLASS_MARKER as MARKER;
    use super::resolve_conversion_error_class_marker as resolve;
    let class = || vec!["a".to_string(), MARKER.to_string(), "b".to_string()];
    assert_eq!(
        resolve(class(), &["pkg_error_argument", "pkg_error"]),
        ["a", "pkg_error_argument", "pkg_error", "b"]
    );
    assert_eq!(resolve(class(), &[]), ["a", "b"]);
    assert_eq!(
        resolve(vec![MARKER.to_string()], &["pkg_error"]),
        ["pkg_error"]
    );
    assert_eq!(resolve(vec!["a".to_string()], &["pkg_error"]), ["a"]);
}

#[test]
fn error_class_vector_and_keyed_data() {
    let cond = catch(|| {
        crate::error!(
            class = ["member", "family"],
            data = { rule = 1, detail = "inner" },
            "layered"
        )
    });
    match cond {
        RCondition::Error {
            message,
            class,
            data,
            call: super::ConditionCall::Inherit,
        } => {
            assert_eq!(message, "layered");
            assert_eq!(class, vec!["member", "family"]);
            assert_data(
                &data,
                &[
                    ("rule", RValue::Integer(vec![Some(1)])),
                    ("detail", RValue::Character(vec![Some("inner".to_string())])),
                ],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn error_class_vector_from_vec_string() {
    let classes = vec!["a".to_string(), "b".to_string()];
    let cond = catch(move || crate::warning!(class = classes, "w"));
    match cond {
        RCondition::Warning { class, .. } => assert_eq!(class, vec!["a", "b"]),
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn reserved_field_name_runtime_check_panics() {
    let name = String::from("kind");
    let payload = std::panic::catch_unwind(move || crate::error!(data = (name, 1), "x"))
        .expect_err("must panic");
    let msg = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .expect("plain panic message");
    assert!(msg.contains("reserved"), "got: {msg}");
    assert!(msg.contains("rename"), "got: {msg}");
}

#[cfg(feature = "serde")]
mod serde_err_parts {
    use super::super::{SerdeErrorShape, serde_err_parts};
    use crate::RValue;
    use ::serde::Serialize;

    #[derive(Serialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum Tagged {
        MissingField { field: String },
        Io,
    }

    impl std::fmt::Display for Tagged {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Tagged::MissingField { field } => write!(f, "field `{field}` is missing"),
                Tagged::Io => f.write_str("io"),
            }
        }
    }

    #[derive(Serialize)]
    struct Plain {
        code: i32,
    }

    impl std::fmt::Display for Plain {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "plain {}", self.code)
        }
    }

    #[derive(Serialize)]
    enum Clash {
        Bad { kind: String },
    }

    impl std::fmt::Display for Clash {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("clash")
        }
    }

    #[test]
    fn internally_tagged_variant_becomes_member_class_and_fields_become_data() {
        let parts = serde_err_parts(&Tagged::MissingField { field: "id".into() }, "engine", &[]);
        assert_eq!(parts.message, "field `id` is missing");
        assert_eq!(parts.class, ["engine_missing_field", "engine"]);
        let data = parts.data.expect("fields become data");
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].0, "field");
        assert!(matches!(&data[0].1, RValue::Character(v) if v == &vec![Some("id".to_string())]));
    }

    #[test]
    fn unit_variant_has_classes_but_no_data() {
        let parts = serde_err_parts(&Tagged::Io, "engine", &[]);
        assert_eq!(parts.class, ["engine_io", "engine"]);
        assert!(parts.data.is_none());
    }

    #[test]
    fn value_without_variant_gets_only_the_family_class() {
        let parts = serde_err_parts(&Plain { code: 3 }, "engine", &[]);
        assert_eq!(parts.message, "plain 3");
        assert_eq!(parts.class, ["engine"]);
        let data = parts.data.expect("struct fields become data");
        assert_eq!(data[0].0, "code");
    }

    #[test]
    fn reserved_payload_field_is_rejected() {
        let err = std::panic::AssertUnwindSafe(Clash::Bad { kind: "x".into() });
        let payload = std::panic::catch_unwind(move || {
            serde_err_parts(&err.0, "p", &[&SerdeErrorShape::new().tag("type")])
        })
        .expect_err("must panic");
        let msg = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(msg.contains("reserved"), "{msg}");
        assert!(msg.contains("`kind`"), "{msg}");
    }

    /// The wrapped-parser-error shape (#1457): a `message` payload field.
    #[derive(Serialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum Wrapped {
        /// `Display` adds context, so `message` is not redundant.
        Parse {
            message: String,
            line: u32,
        },
        /// `Display` is the field verbatim.
        Echo {
            message: String,
        },
        Bad {
            call: String,
        },
    }

    impl std::fmt::Display for Wrapped {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Wrapped::Parse { message, line } => write!(f, "line {line}: {message}"),
                Wrapped::Echo { message } => f.write_str(message),
                Wrapped::Bad { .. } => f.write_str("bad"),
            }
        }
    }

    fn panic_message(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
        let payload = std::panic::catch_unwind(f).expect_err("must panic");
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    fn names(parts: &super::super::ErrParts) -> Vec<&str> {
        parts
            .data
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|(n, _)| n.as_str())
            .collect()
    }

    #[test]
    fn message_field_equal_to_display_is_dropped_without_options() {
        let parts = serde_err_parts(
            &Wrapped::Echo {
                message: "boom".into(),
            },
            "p",
            &[],
        );
        assert_eq!(parts.message, "boom");
        assert_eq!(parts.class, ["p_echo", "p"]);
        assert!(parts.data.is_none(), "{:?}", parts.data);
    }

    #[test]
    fn message_field_differing_from_display_is_still_rejected() {
        let err = std::panic::AssertUnwindSafe(Wrapped::Parse {
            message: "unexpected token".into(),
            line: 3,
        });
        let msg = panic_message(move || {
            serde_err_parts(&err.0, "p", &[]);
        });
        assert!(msg.contains("reserved"), "{msg}");
        assert!(msg.contains("`message`"), "{msg}");
    }

    #[test]
    fn skip_drops_the_named_field_and_is_a_no_op_elsewhere() {
        let parts = serde_err_parts(
            &Wrapped::Parse {
                message: "unexpected token".into(),
                line: 3,
            },
            "p",
            &[&SerdeErrorShape::new().skip(&["message", "absent"])],
        );
        assert_eq!(parts.message, "line 3: unexpected token");
        assert_eq!(parts.class, ["p_parse", "p"]);
        assert_eq!(names(&parts), ["line"]);

        // `skip` runs before the Display-equality rule; same outcome.
        let parts = serde_err_parts(
            &Wrapped::Echo {
                message: "boom".into(),
            },
            "p",
            &[&SerdeErrorShape::new().skip(&["message"])],
        );
        assert!(parts.data.is_none());

        // A variant without the skipped field but with another reserved
        // name still fails the reserved-name check.
        let err = std::panic::AssertUnwindSafe(Wrapped::Bad { call: "f()".into() });
        let msg = panic_message(move || {
            serde_err_parts(&err.0, "p", &[&SerdeErrorShape::new().skip(&["message"])]);
        });
        assert!(msg.contains("`call`"), "{msg}");
    }

    #[test]
    fn rename_splices_the_field_under_the_new_name() {
        let parts = serde_err_parts(
            &Wrapped::Parse {
                message: "unexpected token".into(),
                line: 3,
            },
            "p",
            &[&SerdeErrorShape::new().rename(&[("message", "detail"), ("absent", "other")])],
        );
        assert_eq!(names(&parts), ["detail", "line"]);
        let data = parts.data.expect("data");
        assert!(matches!(
            &data[0].1,
            RValue::Character(v) if v == &vec![Some("unexpected token".to_string())]
        ));

        // `rename` wins over the Display-equality drop: the caller asked
        // for the field explicitly.
        let parts = serde_err_parts(
            &Wrapped::Echo {
                message: "boom".into(),
            },
            "p",
            &[&SerdeErrorShape::new().rename(&[("message", "detail")])],
        );
        assert_eq!(names(&parts), ["detail"]);
    }

    /// The probe's second stage under the `serde` feature: an
    /// `E: Serialize + Display` is classed with the defaults and the
    /// macro-supplied prefix; a `String` error gets the family class and
    /// its text unquoted.
    #[test]
    fn err_parts_probe_classes_serialize_display_types() {
        let parts = crate::__mx_result_err_parts!(Tagged::Io, "p", super::NoShapes);
        assert_eq!(parts.message, "io");
        assert_eq!(parts.class, ["p_io", "p"]);
        assert!(parts.data.is_none());

        let parts = crate::__mx_result_err_parts!(
            Tagged::MissingField { field: "id".into() },
            "p",
            super::NoShapes
        );
        assert_eq!(parts.class, ["p_missing_field", "p"]);
        assert_eq!(names(&parts), ["field"]);

        let parts = crate::__mx_result_err_parts!(String::from("bad input"), "p", super::NoShapes);
        assert_eq!(parts.message, "bad input");
        assert_eq!(parts.class, ["p"]);
        assert!(parts.data.is_none());

        let parts = crate::__mx_result_err_parts!("static text", "p", super::NoShapes);
        assert_eq!(parts.message, "static text");
        assert_eq!(parts.class, ["p"]);
    }

    /// #1459: a `rename` target the variant already carries would give the
    /// condition two `line` entries; R reads only the first.
    #[test]
    fn rename_onto_a_name_the_variant_carries_panics() {
        let err = std::panic::AssertUnwindSafe(Wrapped::Parse {
            message: "unexpected token".into(),
            line: 3,
        });
        let msg = panic_message(move || {
            serde_err_parts(
                &err.0,
                "p",
                &[&SerdeErrorShape::new().rename(&[("message", "line")])],
            );
        });
        assert!(msg.contains("field `line` appears twice"), "{msg}");
        assert!(msg.contains("`rename(message = \"line\")`"), "{msg}");

        // The same attribute is fine on a variant without `line`.
        let parts = serde_err_parts(
            &Wrapped::Echo {
                message: "boom".into(),
            },
            "p",
            &[&SerdeErrorShape::new().rename(&[("message", "line")])],
        );
        assert_eq!(names(&parts), ["line"]);
    }

    /// A target that only *another* variant carries is no collision, and
    /// `skip` frees a name for `rename` because both run before the check.
    #[test]
    fn rename_target_is_checked_per_variant_after_skip() {
        let parts = serde_err_parts(
            &Wrapped::Bad { call: "f()".into() },
            "p",
            &[&SerdeErrorShape::new().rename(&[("call", "line")])],
        );
        assert_eq!(names(&parts), ["line"]);

        let parts = serde_err_parts(
            &Wrapped::Parse {
                message: "unexpected token".into(),
                line: 3,
            },
            "p",
            &[&SerdeErrorShape::new()
                .skip(&["line"])
                .rename(&[("message", "line")])],
        );
        assert_eq!(names(&parts), ["line"]);
        let data = parts.data.expect("data");
        assert!(matches!(
            &data[0].1,
            RValue::Character(v) if v == &vec![Some("unexpected token".to_string())]
        ));
    }
}

/// `SerdeErrorShape`'s const checks (#2207). Called at run time here, the
/// same panics a `const` turns into a compile error.
mod serde_error_shape_checks {
    use super::super::SerdeErrorShape;

    fn panic_message(f: impl FnOnce() -> SerdeErrorShape + std::panic::UnwindSafe) -> String {
        let payload = std::panic::catch_unwind(f).expect_err("must panic");
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    #[test]
    fn a_consistent_shape_builds_in_a_const() {
        const ENGINE: SerdeErrorShape = SerdeErrorShape::new()
            .tag("type")
            .prefix("engine")
            .skip(&["message"])
            .rename(&[("source", "cause")])
            .as_character(&["rule", "column", "source"])
            .as_integer(&["n"])
            .as_double(&["rate"])
            .as_logical(&["exact"]);
        assert_ne!(ENGINE, SerdeErrorShape::new());
        assert_eq!(SerdeErrorShape::default(), SerdeErrorShape::new());
    }

    #[test]
    fn a_field_in_two_type_lists_is_refused() {
        let msg = panic_message(|| {
            SerdeErrorShape::new()
                .as_integer(&["rule", "n"])
                .as_double(&["n"])
        });
        assert_eq!(
            msg,
            "SerdeErrorShape: field `n` is in both `as_integer` and `as_double`; a field has \
                 one R type"
        );
    }

    #[test]
    fn a_skipped_field_takes_no_type_or_name() {
        let msg = panic_message(|| SerdeErrorShape::new().skip(&["n"]).as_integer(&["n"]));
        assert!(
            msg.contains("field `n` is in both `skip` and `as_integer`"),
            "{msg}"
        );
        let msg = panic_message(|| SerdeErrorShape::new().as_logical(&["n"]).skip(&["n"]));
        assert!(
            msg.contains("field `n` is in both `as_logical` and `skip`"),
            "{msg}"
        );
        let msg = panic_message(|| {
            SerdeErrorShape::new()
                .skip(&["message"])
                .rename(&[("message", "detail")])
        });
        assert!(
            msg.contains("field `message` is in both `skip` and `rename`"),
            "{msg}"
        );
        let msg = panic_message(|| {
            SerdeErrorShape::new()
                .rename(&[("message", "detail")])
                .skip(&["message"])
        });
        assert!(
            msg.contains("field `message` is in both `rename` and `skip`"),
            "{msg}"
        );
    }

    #[test]
    fn rename_mirrors_the_attribute_grammar() {
        let msg = panic_message(|| SerdeErrorShape::new().rename(&[("detail", "kind")]));
        assert!(msg.contains("target `kind` is reserved"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().rename(&[("a", "b"), ("a", "c")]));
        assert!(msg.contains("field `a` is renamed twice"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().rename(&[("a", "b"), ("c", "b")]));
        assert!(msg.contains("two `rename` pairs target `b`"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().rename(&[("a", "")]));
        assert!(msg.contains("`rename` has an empty name"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().rename(&[]));
        assert!(msg.contains("`rename` needs at least one pair"), "{msg}");
    }

    #[test]
    fn lists_and_options_are_set_once_and_not_empty() {
        let msg = panic_message(|| SerdeErrorShape::new().skip(&["a"]).skip(&["b"]));
        assert!(msg.contains("`skip` is set twice"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().as_double(&["a"]).as_double(&["b"]));
        assert!(msg.contains("`as_double` is set twice"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().prefix("a").prefix("b"));
        assert!(msg.contains("`prefix` is set twice"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().tag(""));
        assert!(msg.contains("`tag` must not be empty"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().as_character(&[]));
        assert!(
            msg.contains("`as_character` needs at least one field"),
            "{msg}"
        );
        let msg = panic_message(|| SerdeErrorShape::new().skip(&[""]));
        assert!(msg.contains("`skip` has an empty field name"), "{msg}");
        let msg = panic_message(|| SerdeErrorShape::new().as_integer(&["n", "n"]));
        assert!(
            msg.contains("field `n` is listed twice in `as_integer`"),
            "{msg}"
        );
    }

    /// The message buffer is 256 bytes; a name it cuts inside a
    /// character falls back to a message without the name.
    #[test]
    fn a_long_field_name_still_panics_with_a_message() {
        // "SerdeErrorShape: field `" is 24 bytes; the buffer ends inside `é`.
        let name: &'static str = Box::leak(format!("{}é", "x".repeat(231)).into_boxed_str());
        let names: &'static [&'static str] = Box::leak(Box::new([name]));
        let msg = panic_message(move || SerdeErrorShape::new().as_integer(names).skip(names));
        assert_eq!(
            msg,
            "SerdeErrorShape: contradictory options; a field name is too long to quote"
        );
    }
}

/// The `as_*` options and shape precedence at the condition boundary
/// (#2207), on a hand-written `Serialize` that uses no field helper.
#[cfg(feature = "serde")]
mod serde_error_shape {
    use super::super::{SHAPE_TYPE_OPTIONS, SerdeErrorShape, serde_err_parts};
    use crate::RValue;
    use crate::serde::rvalue_ser::Atomic;

    /// The shape of an error type in a crate that can't depend on
    /// miniextendr: internally tagged by hand.
    struct Engine {
        rule: Vec<String>,
        n: u32,
        rate: i64,
        flags: Vec<Option<bool>>,
    }

    impl ::serde::Serialize for Engine {
        fn serialize<S: ::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            use ::serde::ser::SerializeStruct;
            let mut st = s.serialize_struct("Engine", 5)?;
            st.serialize_field("kind", "rule")?;
            st.serialize_field("rule", &self.rule)?;
            st.serialize_field("n", &self.n)?;
            st.serialize_field("rate", &self.rate)?;
            st.serialize_field("flags", &self.flags)?;
            st.end()
        }
    }

    impl std::fmt::Display for Engine {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("rule failed")
        }
    }

    fn engine() -> Engine {
        Engine {
            rule: Vec::new(),
            n: 3,
            rate: 5,
            flags: vec![Some(true), None],
        }
    }

    fn field<'a>(parts: &'a super::super::ErrParts, name: &str) -> &'a RValue {
        parts
            .data
            .as_deref()
            .unwrap_or_default()
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("no field `{name}` in {:?}", parts.data))
    }

    fn panic_message(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
        let payload = std::panic::catch_unwind(f).expect_err("must panic");
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    #[test]
    fn atomic_order_matches_the_shape_lists() {
        for (atomic, option) in Atomic::ALL.iter().zip(SHAPE_TYPE_OPTIONS) {
            assert_eq!(atomic.helper(), option);
        }
    }

    #[test]
    fn without_options_the_fields_keep_their_usual_serialization() {
        let parts = serde_err_parts(&engine(), "p", &[]);
        assert_eq!(parts.class, ["p_rule", "p"]);
        assert!(matches!(field(&parts, "rule"), RValue::List(v) if v.is_empty()));
        assert!(matches!(field(&parts, "n"), RValue::Integer(v) if v == &[Some(3)]));
        assert!(matches!(field(&parts, "flags"), RValue::List(v) if v.len() == 2));
    }

    #[test]
    fn each_type_option_gives_its_vector() {
        const SHAPE: SerdeErrorShape = SerdeErrorShape::new()
            .as_character(&["rule"])
            .as_double(&["n"])
            .as_integer(&["rate"])
            .as_logical(&["flags"]);
        let parts = serde_err_parts(&engine(), "p", &[&SHAPE]);
        assert!(matches!(field(&parts, "rule"), RValue::Character(v) if v.is_empty()));
        assert!(matches!(field(&parts, "n"), RValue::Double(v) if v == &[3.0]));
        assert!(matches!(field(&parts, "rate"), RValue::Integer(v) if v == &[Some(5)]));
        assert!(matches!(
            field(&parts, "flags"),
            RValue::Logical(v) if v == &[Some(true), None]
        ));
    }

    #[test]
    fn a_value_that_does_not_fit_names_the_field_and_the_option() {
        let shape = SerdeErrorShape::new().as_integer(&["rate"]);
        let err = std::panic::AssertUnwindSafe(Engine {
            rate: 1 << 40,
            ..engine()
        });
        let msg = panic_message(move || {
            serde_err_parts(&err.0, "p", &[&shape]);
        });
        assert_eq!(
            msg,
            "#[miniextendr(serde_error)]: `as_integer` on condition data field `rate` \
                 failed: the field is not an integer that fits an R integer or `None`; the \
                 original error was: rule failed"
        );

        let shape = SerdeErrorShape::new().as_logical(&["rule"]);
        let err = std::panic::AssertUnwindSafe(Engine {
            rule: vec!["a".into(), "b".into()],
            ..engine()
        });
        let msg = panic_message(move || {
            serde_err_parts(&err.0, "p", &[&shape]);
        });
        assert!(
            msg.contains(
                "`as_logical` on condition data field `rule` failed: the field is not a logical"
            ),
            "{msg}"
        );
    }

    /// The keys are the names as the error serializes them: an `as_*`
    /// option and a `rename` of one field compose.
    #[test]
    fn type_and_rename_compose_on_the_serialized_name() {
        let shape = SerdeErrorShape::new()
            .rename(&[("rule", "rules")])
            .as_character(&["rule"]);
        let parts = serde_err_parts(&engine(), "p", &[&shape]);
        assert!(matches!(field(&parts, "rules"), RValue::Character(v) if v.is_empty()));
    }

    /// The function's options (the last shape) win for each setting of a
    /// field; what only the named shape sets stays.
    #[test]
    fn later_shapes_win_per_field() {
        let named = SerdeErrorShape::new()
            .tag("type")
            .prefix("engine")
            .skip(&["n"])
            .as_integer(&["rate"])
            .rename(&[("rule", "rules")])
            .as_character(&["rule"]);
        let own = SerdeErrorShape::new().tag("kind").as_double(&["n", "rate"]);
        let parts = serde_err_parts(&engine(), "p", &[&named, &own]);
        assert_eq!(parts.class, ["engine_rule", "engine"]);
        assert!(matches!(field(&parts, "n"), RValue::Double(v) if v == &[3.0]));
        assert!(matches!(field(&parts, "rate"), RValue::Double(v) if v == &[5.0]));
        assert!(matches!(field(&parts, "rules"), RValue::Character(v) if v.is_empty()));

        // A later `skip` drops a field the earlier shape typed; a later
        // `rename` keeps the earlier type.
        let named = SerdeErrorShape::new()
            .as_integer(&["n"])
            .as_double(&["rate"]);
        let own = SerdeErrorShape::new()
            .prefix("own")
            .skip(&["rate"])
            .rename(&[("n", "count")]);
        let parts = serde_err_parts(&engine(), "p", &[&named, &own]);
        assert_eq!(parts.class, ["own_rule", "own"]);
        let names: Vec<&str> = parts
            .data
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();
        assert_eq!(names, ["rule", "count", "flags"]);
        assert!(matches!(field(&parts, "count"), RValue::Integer(v) if v == &[Some(3)]));

        // A later rename keeps a field an earlier shape skipped.
        let named = SerdeErrorShape::new().skip(&["n"]);
        let own = SerdeErrorShape::new().rename(&[("n", "count")]);
        let parts = serde_err_parts(&engine(), "p", &[&named, &own]);
        assert!(matches!(field(&parts, "count"), RValue::Integer(v) if v == &[Some(3)]));
    }

    /// A field the payload lacks is a no-op, as for `skip` / `rename`.
    #[test]
    fn a_field_the_payload_lacks_is_a_no_op() {
        let shape = SerdeErrorShape::new()
            .as_character(&["column"])
            .as_integer(&["absent"]);
        let parts = serde_err_parts(&engine(), "p", &[&shape]);
        assert_eq!(parts.data.as_deref().unwrap_or_default().len(), 4);
    }
}

#[test]
fn rerror_reserved_field_rejected_at_err_arm() {
    use super::RError;
    let err = std::panic::AssertUnwindSafe(RError::new("m").data("kind", 2));
    let payload =
        std::panic::catch_unwind(move || crate::__mx_result_err_parts!(err.0, "p", NoShapes))
            .expect_err("must panic");
    let msg = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .expect("plain panic message");
    assert!(msg.contains("reserved"), "got: {msg}");
    assert!(msg.contains("`kind`"), "got: {msg}");
}

/// The probe's first stage: an `RConditionData` impl wins outright.
#[test]
fn err_parts_probe_prefers_rconditionerror() {
    use super::RError;
    let e = RError::new("m")
        .class(["member", "family"])
        .data("input", 1);
    let parts = crate::__mx_result_err_parts!(e, "p", NoShapes);
    assert_eq!(parts.message, "m");
    assert_eq!(parts.class, ["member", "family"]);
    assert_eq!(parts.data.expect("data")[0].0, "input");
}

/// The probe's last resort: a type that is neither `RConditionData` nor
/// `Serialize` renders with `Debug`, no class, no data.
#[test]
fn err_parts_probe_falls_back_to_debug() {
    #[derive(Debug)]
    struct Opaque;
    let parts = crate::__mx_result_err_parts!(Opaque, "p", NoShapes);
    assert_eq!(parts.message, "Opaque");
    assert!(parts.class.is_empty());
    assert!(parts.data.is_none());
}

/// The conversion probe's built-in arm (#1591): the framework's own
/// conversion errors are not `RConditionData`, keep the plain class
/// vector and no data, and read in R terms instead of their `Display`
/// text. Other error types still render with `Display` (not `Debug`).
#[test]
fn conversion_probe_rewords_builtin_errors() {
    use crate::SEXPTYPE;
    use crate::from_r::{SexpClassError, SexpError, SexpLengthError, SexpNaError, SexpTypeError};

    // The probe takes the owned error, as in the generated `Err(e)` arm.
    let message =
        |e: &SexpError, known: bool| crate::__mx_conversion_err_parts!(e.clone(), known).message;

    let e = SexpError::InvalidValue("non-numeric value(s): \"BLQ\" (element 2)".into());
    let parts = crate::__mx_conversion_err_parts!(e, true);
    assert_eq!(parts.message, "non-numeric value(s): \"BLQ\" (element 2)");
    assert!(parts.class.is_empty());
    assert!(parts.data.is_none());
    assert_eq!(message(&e, false), message(&e, true));

    let e = SexpError::Type(SexpTypeError {
        expected: SEXPTYPE::INTSXP,
        actual: SEXPTYPE::STRSXP,
    });
    assert_eq!(message(&e, true), "got character");
    assert_eq!(message(&e, false), "expected integer, got character");
    // The macro states a type error's expectation; the error gives none.
    assert_eq!(e.r_expectation(), None);
    // A class error names the class, as the expectation and in the
    // per-element reason.
    let e = SexpError::Class(SexpClassError {
        expected: "a data frame",
        actual: SEXPTYPE::VECSXP,
    });
    assert_eq!(message(&e, true), "got list");
    assert_eq!(message(&e, false), "expected a data frame, got list");
    assert_eq!(e.r_expectation().as_deref(), Some("a data frame"));
    let e = SexpError::Length(SexpLengthError {
        expected: 1,
        actual: 2,
    });
    assert_eq!(message(&e, true), "got length 2");
    assert_eq!(message(&e, false), "expected length 1, got length 2");
    let e = SexpError::Na(SexpNaError {
        sexp_type: SEXPTYPE::REALSXP,
    });
    assert_eq!(message(&e, true), "NA is not allowed");
    assert_eq!(message(&e, false), "NA is not allowed");
    assert_eq!(
        message(&SexpError::MissingField("id".into()), true),
        "missing field 'id'"
    );
    assert_eq!(
        message(&SexpError::DuplicateName("id".into()), true),
        "duplicate name 'id'"
    );
    // The bare error structs some conversions return take the same arm.
    let e = SexpTypeError {
        expected: SEXPTYPE::REALSXP,
        actual: SEXPTYPE::VECSXP,
    };
    assert_eq!(
        crate::__mx_conversion_err_parts!(e, true).message,
        "got list"
    );
    // The one-argument form is the no-expectation form.
    assert_eq!(
        crate::__mx_conversion_err_parts!(e).message,
        "expected numeric, got list"
    );
    let e = SexpClassError {
        expected: "a data frame",
        actual: SEXPTYPE::INTSXP,
    };
    assert_eq!(
        crate::__mx_conversion_err_parts!(e, true).message,
        "got integer"
    );
    assert_eq!(
        crate::__mx_conversion_err_parts!(e).message,
        "expected a data frame, got integer"
    );

    // `MatchArgError` takes the built-in arm too: R-worded, and after the
    // `one of ...` expectation just the input.
    use crate::match_arg::MatchArgError;
    const CHOICES: &[&str] = &["fast", "slow"];
    let e = MatchArgError::IsNa {
        choices: CHOICES,
        aliases: &[],
    };
    let parts = crate::__mx_conversion_err_parts!(e.clone());
    assert_eq!(parts.message, r#"expected one of "fast", "slow", got NA"#);
    assert!(parts.class.is_empty());
    assert!(parts.data.is_none());
    assert_eq!(
        crate::__mx_conversion_err_parts!(e, true).message,
        "NA is not allowed"
    );
    let e = MatchArgError::NoMatch {
        input: "zzz".into(),
        choices: CHOICES,
        aliases: &[],
    };
    assert_eq!(
        crate::__mx_conversion_err_parts!(e.clone()).message,
        r#"expected one of "fast", "slow", got "zzz""#
    );
    assert_eq!(
        crate::__mx_conversion_err_parts!(e, true).message,
        r#"got "zzz""#
    );
    let e = MatchArgError::InvalidType {
        actual: SEXPTYPE::REALSXP,
        choices: CHOICES,
        aliases: &[],
    };
    assert_eq!(
        crate::__mx_conversion_err_parts!(e.clone()).message,
        r#"expected one of "fast", "slow" (a string or factor), got numeric"#
    );
    assert_eq!(
        crate::__mx_conversion_err_parts!(e, true).message,
        "got numeric"
    );
    let e = MatchArgError::InvalidLength {
        actual: 2,
        choices: CHOICES,
        aliases: &[],
    };
    assert_eq!(
        crate::__mx_conversion_err_parts!(e, true).message,
        "got length 2"
    );

    // A plain `&str` / `String` error (neither `Serialize`-classed nor
    // `Debug`-quoted, unlike the `Result` probe).
    let parts = crate::__mx_conversion_err_parts!(String::from("bad input"), true);
    assert_eq!(parts.message, "bad input");
    assert!(parts.class.is_empty());
}

/// A `match_arg` choice error knows what the value should have been, as a
/// `MatchArgError` or inside the `SexpError` `#[derive(MatchArg)]`'s
/// `TryFromSexp` returns; other errors do not (#1594). The prefix built
/// from it reads like a statically known one.
#[test]
fn conversion_expectation_probe_names_match_arg_choices() {
    use super::conversion_prefix;
    use crate::from_r::SexpError;
    use crate::match_arg::MatchArgError;
    let e = MatchArgError::NoMatch {
        input: "zzz".into(),
        choices: &["fast", "slow"],
        aliases: &[],
    };
    let expected = Some(r#"one of "fast", "slow""#.to_string());
    assert_eq!(crate::__mx_conversion_expectation!(e.clone()), expected);
    let e = SexpError::from(e);
    assert_eq!(crate::__mx_conversion_expectation!(e.clone()), expected);
    assert_eq!(
        crate::__mx_conversion_err_parts!(e, true).message,
        r#"got "zzz""#
    );
    assert_eq!(
        crate::__mx_conversion_expectation!(SexpError::InvalidValue("x".into())),
        None
    );
    assert_eq!(
        crate::__mx_conversion_expectation!(String::from("bad input")),
        None
    );

    assert_eq!(
        conversion_prefix("mode", false, expected.as_deref()),
        r#"'mode' must be one of "fast", "slow""#
    );
    assert_eq!(
        conversion_prefix("mode", true, expected.as_deref()),
        r#"'mode' must be NULL or one of "fast", "slow""#
    );
    assert_eq!(conversion_prefix("x", true, None), "invalid 'x' argument");
}

/// An `Either` whose arms both refused the kind of value (a type or class
/// error, a `match_arg` refusal of a non-string, or a nested `Either` of
/// those) names both arms, as the reason and as the expectation; an arm
/// that got further gives the reason instead.
#[cfg(feature = "either")]
#[test]
fn either_conversion_names_both_arms_that_refused_the_kind() {
    use crate::SEXPTYPE;
    use crate::from_r::{SexpClassError, SexpError, SexpLengthError, SexpTypeError};
    use crate::match_arg::MatchArgError;

    let either = |l: SexpError, r: SexpError| SexpError::EitherConversion {
        left_error: Box::new(l),
        right_error: Box::new(r),
    };
    let ty = |expected, actual| SexpError::Type(SexpTypeError { expected, actual });
    let frame = |actual| {
        SexpError::Class(SexpClassError {
            expected: "a data frame",
            actual,
        })
    };
    let refused = |actual, choices| {
        SexpError::MatchArg(MatchArgError::InvalidType {
            actual,
            choices,
            aliases: &[],
        })
    };
    const ROUTES: &[&str] = &["oral", "bolus"];
    const MODES: &[&str] = &["fast", "bolus"];
    // (error, reason with the expectation known, reason without it,
    // expectation)
    let check = |e: SexpError, known: &str, unknown: &str, expected: Option<&str>| {
        assert_eq!(e.r_reason(true), known, "{e}");
        assert_eq!(e.r_reason(false), unknown, "{e}");
        assert_eq!(e.r_expectation().as_deref(), expected, "{e}");
    };
    use SEXPTYPE::{INTSXP, LGLSXP, REALSXP, STRSXP, VECSXP};

    check(
        either(ty(INTSXP, REALSXP), ty(STRSXP, REALSXP)),
        "got numeric",
        "expected integer or character, got numeric",
        Some("integer or character"),
    );
    check(
        either(ty(REALSXP, VECSXP), frame(VECSXP)),
        "got list",
        "expected numeric or a data frame, got list",
        Some("numeric or a data frame"),
    );
    check(
        either(frame(INTSXP), ty(STRSXP, INTSXP)),
        "got integer",
        "expected a data frame or character, got integer",
        Some("a data frame or character"),
    );
    check(
        either(refused(LGLSXP, ROUTES), ty(REALSXP, LGLSXP)),
        "got logical",
        r#"expected one of "oral", "bolus", or numeric, got logical"#,
        Some(r#"one of "oral", "bolus", or numeric"#),
    );
    check(
        either(ty(REALSXP, LGLSXP), refused(LGLSXP, ROUTES)),
        "got logical",
        r#"expected numeric or one of "oral", "bolus", got logical"#,
        Some(r#"numeric or one of "oral", "bolus""#),
    );
    // Two choice lists read as one, each choice once.
    check(
        either(refused(REALSXP, ROUTES), refused(REALSXP, MODES)),
        "got numeric",
        r#"expected one of "oral", "bolus", "fast", got numeric"#,
        Some(r#"one of "oral", "bolus", "fast""#),
    );
    // The listed aliases follow the choices, before the other arm; an
    // unlisted one is left out (#2148). Two choice lists keep both
    // lists' aliases, in the order of the joined choices.
    use crate::match_arg::MatchArgAlias;
    const ROUTE_ALIASES: &[MatchArgAlias] = &[
        MatchArgAlias::new("po", "oral"),
        MatchArgAlias::unlisted("iv", "bolus"),
    ];
    const MODE_ALIASES: &[MatchArgAlias] = &[MatchArgAlias::new("quick", "fast")];
    let aliased = |actual, choices, aliases| {
        SexpError::MatchArg(MatchArgError::InvalidType {
            actual,
            choices,
            aliases,
        })
    };
    check(
        either(aliased(LGLSXP, ROUTES, ROUTE_ALIASES), ty(REALSXP, LGLSXP)),
        "got logical",
        r#"expected one of "oral", "bolus" (or "po" for "oral"), or numeric, got logical"#,
        Some(r#"one of "oral", "bolus" (or "po" for "oral"), or numeric"#),
    );
    check(
        either(ty(REALSXP, LGLSXP), aliased(LGLSXP, ROUTES, ROUTE_ALIASES)),
        "got logical",
        r#"expected numeric or one of "oral", "bolus" (or "po" for "oral"), got logical"#,
        Some(r#"numeric or one of "oral", "bolus" (or "po" for "oral")"#),
    );
    check(
        either(
            aliased(REALSXP, MODES, MODE_ALIASES),
            aliased(REALSXP, ROUTES, ROUTE_ALIASES),
        ),
        "got numeric",
        r#"expected one of "fast", "bolus", "oral" (or "quick" for "fast", "po" for "oral"), got numeric"#,
        Some(r#"one of "fast", "bolus", "oral" (or "quick" for "fast", "po" for "oral")"#),
    );
    // A nested `Either` whose arms both refused the kind is one arm.
    let inner = either(refused(VECSXP, ROUTES), ty(REALSXP, VECSXP));
    check(
        either(ty(LGLSXP, VECSXP), inner),
        "got list",
        r#"expected logical or one of "oral", "bolus", or numeric, got list"#,
        Some(r#"logical or one of "oral", "bolus", or numeric"#),
    );

    // A string that matched no choice got further than a type error: its
    // reason, and no joined expectation.
    let no_match = SexpError::MatchArg(MatchArgError::NoMatch {
        input: "zzz".into(),
        choices: ROUTES,
        aliases: &[],
    });
    check(
        either(no_match.clone(), ty(REALSXP, STRSXP)),
        r#"got "zzz""#,
        r#"expected one of "oral", "bolus", got "zzz""#,
        None,
    );
    let length = SexpError::Length(SexpLengthError {
        expected: 1,
        actual: 2,
    });
    check(
        either(refused(REALSXP, ROUTES), length.clone()),
        "got length 2",
        "expected length 1, got length 2",
        None,
    );
    let bad_value = SexpError::InvalidValue("data.frame has no column names".into());
    check(
        either(ty(REALSXP, VECSXP), bad_value.clone()),
        "data.frame has no column names",
        "data.frame has no column names",
        None,
    );
    check(
        either(bad_value, length),
        "data.frame has no column names; got length 2",
        "data.frame has no column names; expected length 1, got length 2",
        None,
    );
    // A nested `Either` that got further is not a kind mismatch.
    check(
        either(ty(LGLSXP, STRSXP), either(no_match, ty(REALSXP, STRSXP))),
        r#"got "zzz""#,
        r#"expected one of "oral", "bolus", got "zzz""#,
        None,
    );
}

/// An `Either` parameter is worded side by side from what the wrapper
/// knows (`from_r::either_expectation`): a known side by its text,
/// whatever its error, so the text is the same for every failure; an
/// opaque side by what its type declares, else by what its own error
/// expected (a kind, the choices of a `match_arg` enum), else by its noun.
/// `None` only when no side is named by more than its noun.
#[cfg(feature = "either")]
#[test]
fn either_expectation_words_each_side() {
    use crate::SEXPTYPE::{REALSXP, STRSXP, VECSXP};
    use crate::from_r::{
        ExpectedArm, SexpClassError, SexpError, SexpLengthError, SexpTypeError, either_expectation,
    };
    use crate::match_arg::MatchArgError;

    let either = |l: SexpError, r: SexpError| SexpError::EitherConversion {
        left_error: Box::new(l),
        right_error: Box::new(r),
    };
    let ty = |expected, actual| SexpError::Type(SexpTypeError { expected, actual });
    let frame = || {
        SexpError::Class(SexpClassError {
            expected: "a data frame",
            actual: REALSXP,
        })
    };
    let length = || {
        SexpError::Length(SexpLengthError {
            expected: 1,
            actual: 2,
        })
    };
    let bad_value = || SexpError::InvalidValue("invalid UUID".into());
    const ROUTES: &[&str] = &["oral", "bolus"];
    let no_match = || {
        SexpError::MatchArg(MatchArgError::NoMatch {
            input: "zzz".into(),
            choices: ROUTES,
            aliases: &[],
        })
    };

    let number = ExpectedArm::Known("a single number");
    let table = ExpectedArm::Known("a data frame");
    let dose = ExpectedArm::Opaque {
        declared: Some("a single number"),
        noun: "a `Dose`",
    };
    let route = ExpectedArm::Opaque {
        declared: None,
        noun: "a `Route`",
    };
    let doc_id = ExpectedArm::Opaque {
        declared: None,
        noun: "a `DocId`",
    };
    let pattern = ExpectedArm::Opaque {
        declared: None,
        noun: "a `Pattern`",
    };
    let words = |arms: &ExpectedArm<'_>, e: SexpError| either_expectation(&e, arms);

    // Known sides: the same text whichever side got further.
    let plain = ExpectedArm::Either(&number, &table);
    for e in [
        either(length(), frame()),
        either(ty(REALSXP, VECSXP), frame()),
    ] {
        assert_eq!(
            words(&plain, e).as_deref(),
            Some("a single number or a data frame")
        );
    }
    // A newtype side declares its inner type's text.
    let newtype = ExpectedArm::Either(&dose, &table);
    assert_eq!(
        words(&newtype, either(length(), frame())).as_deref(),
        Some("a single number or a data frame")
    );
    // An opaque side named by its error: a type error's kind, a
    // `match_arg` enum's choices whether or not the value was a string.
    let by_kind = ExpectedArm::Either(&doc_id, &number);
    assert_eq!(
        words(&by_kind, either(ty(STRSXP, VECSXP), length())).as_deref(),
        Some("character or a single number")
    );
    let choices = ExpectedArm::Either(&route, &number);
    for e in [either(no_match(), frame()), either(no_match(), length())] {
        assert_eq!(
            words(&choices, e).as_deref(),
            Some(r#"one of "oral", "bolus", or a single number"#)
        );
    }
    // An opaque side whose error names nothing: its noun, beside a side
    // that is known.
    assert_eq!(
        words(&by_kind, either(bad_value(), ty(REALSXP, STRSXP))).as_deref(),
        Some("a `DocId` or a single number")
    );
    // `NULL or` an optional side, and a nested `Either` side by side.
    let optional = ExpectedArm::Nullable(&table);
    let nullable = ExpectedArm::Either(&number, &optional);
    assert_eq!(
        words(&nullable, either(length(), frame())).as_deref(),
        Some("a single number or NULL or a data frame")
    );
    let inner = ExpectedArm::Either(&route, &table);
    let nested = ExpectedArm::Either(&number, &inner);
    assert_eq!(
        words(&nested, either(length(), either(no_match(), frame()))).as_deref(),
        Some(r#"a single number or one of "oral", "bolus", or a data frame"#)
    );
    // No side named by more than its noun: no expectation.
    let opaque = ExpectedArm::Either(&doc_id, &pattern);
    assert_eq!(words(&opaque, either(bad_value(), length())), None);
    // An error that is not an `Either` one does not fit the sides.
    assert_eq!(words(&plain, length()), None);
}

/// The conversion probe's preferred arm: an `RConditionData` error type
/// contributes its class vector, message and data.
#[test]
fn conversion_probe_prefers_rconditionerror() {
    use super::RError;
    let e = RError::new("must be positive")
        .class(["pkg_bad_arg", "pkg_error"])
        .data("value", -1);
    let parts = crate::__mx_conversion_err_parts!(e, true);
    assert_eq!(parts.message, "must be positive");
    assert_eq!(parts.class, ["pkg_bad_arg", "pkg_error"]);
    assert_eq!(parts.data.expect("data")[0].0, "value");
}

/// A reserved data name on a conversion error is rejected like on a
/// `Result` error, instead of overwriting `e$kind`.
#[test]
fn conversion_probe_rejects_reserved_fields() {
    use super::RError;
    let err = std::panic::AssertUnwindSafe(RError::new("m").data("kind", 2));
    let payload = std::panic::catch_unwind(move || crate::__mx_conversion_err_parts!(err.0))
        .expect_err("must panic");
    let msg = payload
        .downcast_ref::<String>()
        .cloned()
        .expect("plain panic message");
    assert!(msg.contains("reserved"), "got: {msg}");
}

fn field_names(parts: &super::ErrParts) -> Vec<&str> {
    parts
        .data
        .as_ref()
        .map(|d| d.iter().map(|(n, _)| n.as_str()).collect())
        .unwrap_or_default()
}

fn character_field<'a>(parts: &'a super::ErrParts, name: &str) -> Option<&'a str> {
    parts
        .data
        .as_ref()?
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| match v {
            crate::RValue::Character(v) => v.first()?.as_deref(),
            _ => None,
        })
}

/// `conversion_err_parts`: the prefix leads the message, the class is
/// kept, `param` comes first in the data and `rust_type` second.
#[test]
fn conversion_err_parts_adds_prefix_param_and_rust_type() {
    use super::{ErrParts, conversion_err_parts};
    use crate::RValue;

    let plain = conversion_err_parts(
        "'dv' must be numeric",
        "dv",
        Some("AsNumericVec"),
        &[],
        ErrParts {
            message: "non-numeric value(s): \"BLQ\" (element 2)".into(),
            class: Vec::new(),
            data: None,
            call: super::ConditionCall::Inherit,
        },
        None,
    );
    assert_eq!(
        plain.message,
        "'dv' must be numeric: non-numeric value(s): \"BLQ\" (element 2)"
    );
    assert!(plain.class.is_empty());
    assert_eq!(field_names(&plain), ["param", "rust_type"]);
    assert_eq!(character_field(&plain, "param"), Some("dv"));
    assert_eq!(character_field(&plain, "rust_type"), Some("AsNumericVec"));

    let classed = conversion_err_parts(
        "invalid 'x' argument",
        "x",
        Some("Count"),
        &[],
        ErrParts {
            message: "must be positive".into(),
            class: vec!["pkg_bad_arg".into(), "pkg_error".into()],
            data: Some(vec![("value".into(), RValue::from(-1))]),
            call: super::ConditionCall::Inherit,
        },
        None,
    );
    assert_eq!(classed.message, "invalid 'x' argument: must be positive");
    assert_eq!(classed.class, ["pkg_bad_arg", "pkg_error"]);
    assert_eq!(field_names(&classed), ["param", "rust_type", "value"]);

    // Without a Rust type only `param` is added.
    let untyped = conversion_err_parts(
        "'value' must be a number",
        "value",
        None,
        &[],
        ErrParts {
            message: "got \"abc\"".into(),
            class: Vec::new(),
            data: None,
            call: super::ConditionCall::Inherit,
        },
        None,
    );
    assert_eq!(untyped.message, "'value' must be a number: got \"abc\"");
    assert_eq!(field_names(&untyped), ["param"]);
}

/// An argument error keeps the call whatever the error type's `call()`
/// says: the R-side checks raise the same condition with the call.
#[test]
fn conversion_err_parts_keeps_the_call() {
    use super::{ConditionCall, ErrParts, conversion_err_parts};

    let parts = conversion_err_parts(
        "invalid 'x' argument",
        "x",
        None,
        &[],
        ErrParts {
            message: "bad".into(),
            class: Vec::new(),
            data: None,
            call: ConditionCall::None,
        },
        None,
    );
    assert_eq!(parts.call, ConditionCall::Inherit);
}

/// The crate's `conversion_error_class` follows the error's own classes,
/// skipping those it already names; a plain error gets only the crate's.
#[test]
fn conversion_err_parts_appends_the_crate_class() {
    use super::{ErrParts, conversion_err_parts};

    let crate_class = ["pkg_error_argument", "pkg_error"];
    let plain = conversion_err_parts(
        "'x' must be a single integer",
        "x",
        Some("i32"),
        &crate_class,
        ErrParts {
            message: "got character".into(),
            class: Vec::new(),
            data: None,
            call: super::ConditionCall::Inherit,
        },
        None,
    );
    assert_eq!(plain.class, ["pkg_error_argument", "pkg_error"]);

    let classed = conversion_err_parts(
        "invalid 'x' argument",
        "x",
        Some("Count"),
        &crate_class,
        ErrParts {
            message: "must be positive".into(),
            class: vec!["pkg_error_negative".into(), "pkg_error".into()],
            data: None,
            call: super::ConditionCall::Inherit,
        },
        None,
    );
    assert_eq!(
        classed.class,
        ["pkg_error_negative", "pkg_error", "pkg_error_argument"]
    );
}

/// `arg_check_parts`: the message as given, the crate class alone, and
/// `param` as the only field (no `rust_type`), as the R guard raises it.
#[test]
fn arg_check_parts_matches_the_r_guard() {
    use super::arg_check_parts;

    let parts = arg_check_parts("'dv' must not contain NA", "dv", &[]);
    assert_eq!(parts.message, "'dv' must not contain NA");
    assert!(parts.class.is_empty());
    assert_eq!(field_names(&parts), ["param"]);
    assert_eq!(character_field(&parts, "param"), Some("dv"));

    let custom = "`x` can't be NA: 100% \"sure\"\ncaf\u{e9}()";
    let parts = arg_check_parts(custom, "x", &["pkg_error_argument", "pkg_error"]);
    assert_eq!(parts.message, custom);
    assert_eq!(parts.class, ["pkg_error_argument", "pkg_error"]);
    assert_eq!(field_names(&parts), ["param"]);
}

/// A `param` or `rust_type` field of the error type's own wins: the
/// framework's field is not added, so `e$param` / `e$rust_type` read the
/// type's value.
#[test]
fn conversion_err_parts_keeps_the_types_own_fields() {
    use super::{ErrParts, conversion_err_parts};
    use crate::RValue;

    let parts = conversion_err_parts(
        "invalid 'hyper' argument",
        "hyper",
        Some("Hyperparams"),
        &[],
        ErrParts {
            message: "hyperparameter 'beta' must be non-negative".into(),
            class: Vec::new(),
            data: Some(vec![
                ("value".into(), RValue::from(-2.0)),
                ("param".into(), RValue::from("beta")),
            ]),
            call: super::ConditionCall::Inherit,
        },
        None,
    );
    assert_eq!(field_names(&parts), ["rust_type", "value", "param"]);
    assert_eq!(character_field(&parts, "param"), Some("beta"));

    let parts = conversion_err_parts(
        "invalid 'x' argument",
        "x",
        Some("Wrapped"),
        &[],
        ErrParts {
            message: "bad".into(),
            class: Vec::new(),
            data: Some(vec![("rust_type".into(), RValue::from("Inner"))]),
            call: super::ConditionCall::Inherit,
        },
        None,
    );
    assert_eq!(field_names(&parts), ["param", "rust_type"]);
    assert_eq!(character_field(&parts, "rust_type"), Some("Inner"));
}

/// An error that words the whole argument error gives the message in
/// place of `<prefix>: <reason>`; class and fields are as for any other.
#[test]
fn conversion_err_parts_takes_a_whole_message() {
    use super::{ErrParts, conversion_err_parts};

    let parts = conversion_err_parts(
        "'mode' must be one of \"fast\", \"slow\"",
        "mode",
        Some("Mode"),
        &["pkg_error_argument"],
        ErrParts {
            message: "got \"zzz\"".into(),
            class: Vec::new(),
            data: None,
            call: super::ConditionCall::Inherit,
        },
        Some("'mode' should be one of \"fast\", \"slow\"".into()),
    );
    assert_eq!(parts.message, "'mode' should be one of \"fast\", \"slow\"");
    assert_eq!(parts.class, ["pkg_error_argument"]);
    assert_eq!(field_names(&parts), ["param", "rust_type"]);
}

/// A `match_arg` choice error, bare or at the top of a `SexpError`, words
/// the whole argument error as `.miniextendr_match_arg` does (#1767);
/// other errors do not.
#[test]
fn conversion_arg_message_probe_words_match_arg_errors_as_the_check() {
    use crate::SEXPTYPE;
    use crate::from_r::SexpError;
    use crate::match_arg::MatchArgError;

    const CHOICES: &[&str] = &["fast", "slow"];
    let one_of = Some(r#"'mode' should be one of "fast", "slow""#.to_string());
    let no_match = MatchArgError::NoMatch {
        input: "zzz".into(),
        choices: CHOICES,
        aliases: &[],
    };
    assert_eq!(
        crate::__mx_conversion_arg_message!(no_match.clone(), "mode"),
        one_of
    );
    assert_eq!(
        crate::__mx_conversion_arg_message!(SexpError::from(no_match), "mode"),
        one_of
    );
    let na = MatchArgError::IsNa {
        choices: CHOICES,
        aliases: &[],
    };
    assert_eq!(
        crate::__mx_conversion_arg_message!(na, "mode").as_deref(),
        Some("'mode' must not be NA")
    );
    let long = MatchArgError::InvalidLength {
        actual: 2,
        choices: CHOICES,
        aliases: &[],
    };
    assert_eq!(
        crate::__mx_conversion_arg_message!(SexpError::from(long), "mode").as_deref(),
        Some("'mode' must be of length 1")
    );
    let number = MatchArgError::InvalidType {
        actual: SEXPTYPE::REALSXP,
        choices: CHOICES,
        aliases: &[],
    };
    assert_eq!(
        crate::__mx_conversion_arg_message!(number, "speed").as_deref(),
        Some("'speed' must be NULL or a character vector")
    );

    assert_eq!(
        crate::__mx_conversion_arg_message!(SexpError::InvalidValue("x".into()), "mode"),
        None
    );
    assert_eq!(
        crate::__mx_conversion_arg_message!(String::from("bad input"), "mode"),
        None
    );
}

/// A choice error inside an `Either` leaves the message to the `Either`,
/// which names both arms.
#[cfg(feature = "either")]
#[test]
fn conversion_arg_message_probe_leaves_an_either_alone() {
    use crate::from_r::SexpError;
    use crate::match_arg::MatchArgError;

    let either = SexpError::EitherConversion {
        left_error: Box::new(SexpError::from(MatchArgError::NoMatch {
            input: "zzz".into(),
            choices: &["fast", "slow"],
            aliases: &[],
        })),
        right_error: Box::new(SexpError::InvalidValue("x".into())),
    };
    assert_eq!(crate::__mx_conversion_arg_message!(either, "mode"), None);
}

/// A classed error with an `argument_message` words the whole argument
/// error, directly or carried in a `SexpError` from a type's
/// `check_sexp`; without one, or inside an `Either`, it does not (#1833).
#[test]
fn conversion_arg_message_probe_takes_a_classed_errors_argument_message() {
    use super::{ConditionCall, RConditionData, RError};
    use crate::from_r::SexpError;

    let advice = "use model_from_df() for a data frame";
    let refusal = || {
        RError::new("got a data frame")
            .class("mx_model_refused")
            .argument_message(advice)
    };
    assert_eq!(
        crate::__mx_conversion_arg_message!(refusal(), "fit").as_deref(),
        Some(advice)
    );
    let carried = SexpError::from(refusal());
    assert_eq!(
        crate::__mx_conversion_arg_message!(carried, "fit").as_deref(),
        Some(advice)
    );
    assert_eq!(
        crate::__mx_conversion_arg_message!(RError::new("got a data frame"), "fit"),
        None
    );
    assert_eq!(
        crate::__mx_conversion_arg_message!(SexpError::from(RError::new("no")), "fit"),
        None
    );

    // A hand-written impl answers through the trait method.
    struct ModelRefused;
    impl RConditionData for ModelRefused {
        fn message(&self) -> String {
            "got a data frame".into()
        }
        fn argument_message(&self) -> Option<String> {
            Some("use model_from_df() for a data frame".into())
        }
    }
    assert_eq!(
        crate::__mx_conversion_arg_message!(ModelRefused, "fit").as_deref(),
        Some(advice)
    );
    // The conversion into `SexpError` keeps it, and the call choice.
    let SexpError::Condition(kept) = SexpError::from(ModelRefused) else {
        panic!("a classed error converts into SexpError::Condition");
    };
    assert_eq!(kept.argument_message_str(), Some(advice));
    assert!(matches!(kept.call(), ConditionCall::Inherit));
}

/// A type's refusal inside an `Either` leaves the message to the
/// `Either`, which names both arms.
#[cfg(feature = "either")]
#[test]
fn conversion_arg_message_probe_leaves_a_classed_refusal_in_an_either_alone() {
    use super::RError;
    use crate::from_r::SexpError;

    let either = SexpError::EitherConversion {
        left_error: Box::new(SexpError::from(
            RError::new("got a data frame").argument_message("use model_from_df()"),
        )),
        right_error: Box::new(SexpError::InvalidValue("x".into())),
    };
    assert_eq!(crate::__mx_conversion_arg_message!(either, "fit"), None);
}

#[test]
fn rerror_builder_and_from_error() {
    use super::{RConditionData, RError};
    let e: RError = "x".parse::<i32>().unwrap_err().into();
    let e = e.class(["member", "family"]).data("input", "x");
    assert!(e.message_str().contains("invalid digit"));
    assert_eq!(RConditionData::class(&e), vec!["member", "family"]);
    let data = RConditionData::data(&e).expect("data");
    assert_eq!(data[0].0, "input");

    assert!(RConditionData::data(&RError::new("m")).is_none());
    assert!(super::is_reserved_condition_field("call"));
    assert!(!super::is_reserved_condition_field("calls"));
}

#[test]
fn error_message_only_backcompat() {
    let cond = catch(|| crate::error!("plain {}", 42));
    match cond {
        RCondition::Error {
            message,
            class,
            data,
            call: super::ConditionCall::Inherit,
        } => {
            assert_eq!(message, "plain 42");
            assert!(class.is_empty());
            assert!(data.is_none());
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn error_class_only_backcompat() {
    let cond = catch(|| crate::error!(class = "my_error", "missing field: {}", "x"));
    match cond {
        RCondition::Error {
            message,
            class,
            data,
            call: super::ConditionCall::Inherit,
        } => {
            assert_eq!(message, "missing field: x");
            assert_eq!(class, vec!["my_error"]);
            assert!(data.is_none());
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn error_single_data_pair() {
    let value = 41_i32;
    let cond = catch(move || crate::error!(data = ("value", value), "v = {value}"));
    match cond {
        RCondition::Error {
            message,
            class,
            data,
            call: super::ConditionCall::Inherit,
        } => {
            assert_eq!(message, "v = 41");
            assert!(class.is_empty());
            assert_data(&data, &[("value", RValue::Integer(vec![Some(41)]))]);
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn error_class_and_data_list_all_value_types() {
    let cond = catch(|| {
        crate::error!(
            class = "validation_error",
            data = [
                ("value", 1.5),
                ("code", 7),
                ("label", "lhs"),
                ("fatal", false),
                ("ints", vec![1, 2]),
                ("reals", vec![0.5_f64]),
                ("flags", vec![true]),
                ("labels", vec!["a".to_string()])
            ],
            "out of range"
        )
    });
    match cond {
        RCondition::Error {
            message,
            class,
            data,
            call: super::ConditionCall::Inherit,
        } => {
            assert_eq!(message, "out of range");
            assert_eq!(class, vec!["validation_error"]);
            assert_data(
                &data,
                &[
                    ("value", RValue::Double(vec![1.5])),
                    ("code", RValue::Integer(vec![Some(7)])),
                    ("label", RValue::Character(vec![Some("lhs".into())])),
                    ("fatal", RValue::Logical(vec![Some(false)])),
                    ("ints", RValue::Integer(vec![Some(1), Some(2)])),
                    ("reals", RValue::Double(vec![0.5])),
                    ("flags", RValue::Logical(vec![Some(true)])),
                    ("labels", RValue::Character(vec![Some("a".into())])),
                ],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn warning_with_class_and_data() {
    let cond = catch(|| crate::warning!(class = "trunc", data = ("dropped", 3), "dropped"));
    match cond {
        RCondition::Warning {
            message,
            class,
            data,
            call: super::ConditionCall::Inherit,
        } => {
            assert_eq!(message, "dropped");
            assert_eq!(class, vec!["trunc"]);
            assert_data(&data, &[("dropped", RValue::Integer(vec![Some(3)]))]);
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn message_with_data() {
    let cond = catch(|| crate::message!(data = ("step", 2), "step {}", 2));
    match cond {
        RCondition::Message { message, data, .. } => {
            assert_eq!(message, "step 2");
            assert_data(&data, &[("step", RValue::Integer(vec![Some(2)]))]);
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn condition_with_class_and_data() {
    let cond =
        catch(|| crate::condition!(class = "progress", data = [("n", 10)], "processed {}", 10));
    match cond {
        RCondition::Condition {
            message,
            class,
            data,
            call: super::ConditionCall::Inherit,
        } => {
            assert_eq!(message, "processed 10");
            assert_eq!(class, vec!["progress"]);
            assert_data(&data, &[("n", RValue::Integer(vec![Some(10)]))]);
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn call_none_on_every_condition_macro() {
    use super::ConditionCall;

    match catch(|| crate::error!(call = none, "plain {}", 1)) {
        RCondition::Error {
            message,
            class,
            data,
            call,
        } => {
            assert_eq!(message, "plain 1");
            assert!(class.is_empty());
            assert!(data.is_none());
            assert_eq!(call, ConditionCall::None);
        }
        other => panic!("wrong variant: {other:?}"),
    }
    match catch(|| crate::rust_error!(call = none, class = ["member", "family"], "e")) {
        RCondition::Error { class, call, .. } => {
            assert_eq!(class, vec!["member", "family"]);
            assert_eq!(call, ConditionCall::None);
        }
        other => panic!("wrong variant: {other:?}"),
    }
    match catch(|| {
        crate::warning!(
            call = none,
            class = "pkg_override",
            data = { profile = 3 },
            "overrides an earlier row"
        )
    }) {
        RCondition::Warning {
            message,
            class,
            data,
            call,
        } => {
            assert_eq!(message, "overrides an earlier row");
            assert_eq!(class, vec!["pkg_override"]);
            assert_data(&data, &[("profile", RValue::Integer(vec![Some(3)]))]);
            assert_eq!(call, ConditionCall::None);
        }
        other => panic!("wrong variant: {other:?}"),
    }
    match catch(|| crate::condition!(call = none, data = ("n", 1), "tick")) {
        RCondition::Condition { data, call, .. } => {
            assert_data(&data, &[("n", RValue::Integer(vec![Some(1)]))]);
            assert_eq!(call, ConditionCall::None);
        }
        other => panic!("wrong variant: {other:?}"),
    }
    match catch(|| crate::rust_condition!(call = none, "tick")) {
        RCondition::Condition { call, .. } => assert_eq!(call, ConditionCall::None),
        other => panic!("wrong variant: {other:?}"),
    }
    // Without `call = none` the call is inherited.
    match catch(|| crate::warning!("sibling")) {
        RCondition::Warning { call, .. } => assert_eq!(call, ConditionCall::Inherit),
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn rerror_without_call_reaches_the_err_parts() {
    use super::{ConditionCall, RConditionData, RError};

    let plain = RError::new("m").class("c");
    assert_eq!(RConditionData::call(&plain), ConditionCall::Inherit);
    let callless = plain.clone().without_call();
    assert_eq!(RConditionData::call(&callless), ConditionCall::None);
    let parts = crate::__mx_result_err_parts!(callless, "pkg_error", NoShapes);
    assert_eq!(parts.call, ConditionCall::None);
    assert_eq!(parts.class, vec!["c"]);
    let parts = crate::__mx_result_err_parts!(plain, "pkg_error", NoShapes);
    assert_eq!(parts.call, ConditionCall::Inherit);
}

#[test]
fn data_list_trailing_comma() {
    let cond = catch(|| crate::error!(data = [("a", 1), ("b", 2),], "msg"));
    match cond {
        RCondition::Error { data, .. } => {
            assert_data(
                &data,
                &[
                    ("a", RValue::Integer(vec![Some(1)])),
                    ("b", RValue::Integer(vec![Some(2)])),
                ],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

// region: keyed builder sugar (ported from #1044/#995)

#[test]
fn keyed_builder_arm_stringifies_idents() {
    let cond = catch(|| crate::error!(data = { value = 42, code = 7 }, "boom"));
    match cond {
        RCondition::Error { data, .. } => {
            assert_data(
                &data,
                &[
                    ("value", RValue::Integer(vec![Some(42)])),
                    ("code", RValue::Integer(vec![Some(7)])),
                ],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn keyed_builder_arm_trailing_comma_and_mixed_types() {
    let cond = catch(|| {
        crate::warning!(
            class = "trunc",
            data = { dropped = 3, ratio = 0.5_f64, tag = "rows", },
            "dropped some"
        )
    });
    match cond {
        RCondition::Warning { data, class, .. } => {
            assert_eq!(class, vec!["trunc"]);
            assert_data(
                &data,
                &[
                    ("dropped", RValue::Integer(vec![Some(3)])),
                    ("ratio", RValue::Double(vec![0.5])),
                    ("tag", RValue::Character(vec![Some("rows".into())])),
                ],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

// endregion

// region: NA-aware + wide-int + debug value types via the macro (#995)

#[test]
fn option_scalar_fields_carry_na() {
    let cond = catch(|| {
        crate::error!(
            data = [("present", Some(9_i32)), ("missing", None::<i32>)],
            "opts"
        )
    });
    match cond {
        RCondition::Error { data, .. } => {
            assert_data(
                &data,
                &[
                    ("present", RValue::Integer(vec![Some(9)])),
                    ("missing", RValue::Integer(vec![None])),
                ],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn vec_option_field_carries_embedded_na() {
    let cond = catch(|| crate::error!(data = ("codes", vec![Some(1_i32), None, Some(3)]), "vec"));
    match cond {
        RCondition::Error { data, .. } => {
            assert_data(
                &data,
                &[("codes", RValue::Integer(vec![Some(1), None, Some(3)]))],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn wide_integer_ladder_via_macro() {
    // Fits in i32 → integer; beyond → double.
    let cond = catch(|| {
        crate::error!(
            data = [("small", 42_i64), ("big", 5_000_000_000_i64)],
            "wide"
        )
    });
    match cond {
        RCondition::Error { data, .. } => {
            assert_data(
                &data,
                &[
                    ("small", RValue::Integer(vec![Some(42)])),
                    ("big", RValue::Double(vec![5_000_000_000.0])),
                ],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn debug_fallback_via_macro() {
    let cond = catch(|| crate::error!(data = ("range", RValue::debug(0..=100)), "dbg"));
    match cond {
        RCondition::Error { data, .. } => {
            assert_data(
                &data,
                &[("range", RValue::Character(vec![Some("0..=100".into())]))],
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

// endregion
