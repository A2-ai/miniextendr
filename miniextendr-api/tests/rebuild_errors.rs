//! Rebuild-hook failures as values (#2217): a hook's `Err(RCondition)` keeps
//! the condition's classes, message and fields, as an entry point's `Err`
//! does, and `ExternalPtr::wrap_sexp_with_error` returns a failed rebuild as
//! `Err` while the `TryFromSexp` conversion still raises it.

mod r_test_utils;

use miniextendr_api::condition::{
    ConditionCall, RCondition, RConditionData, RError, catch_condition,
};
use miniextendr_api::expression::RCall;
use miniextendr_api::externalptr::{ExternalPtr, Saved, Sidecar, TypeMismatchError};
use miniextendr_api::from_r::SexpError;
use miniextendr_api::{OwnedProtect, RValue, SEXP, TryFromSexp};

// Defines this crate's marker type, `__MiniextendrInitMarker`, which the
// derive's hook glue names under the `serde` feature.
miniextendr_api::miniextendr_init!(rebuild_errors);

/// The hook's failure: classes, fields and no call.
fn quota_condition() -> RCondition {
    RCondition::Error {
        message: "quota exceeded by 3".to_owned(),
        class: vec!["pkg_quota_exceeded".to_owned(), "pkg_quota".to_owned()],
        data: Some(vec![
            ("n".to_owned(), RValue::from(3i32)),
            ("what".to_owned(), RValue::from("rows")),
        ]),
        call: ConditionCall::None,
    }
}

/// Rebuilt from its saved values; a negative one fails the hook with
/// [`quota_condition`].
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(rebuild = Self::from_saved)]
struct Hooked {
    #[r_data]
    values: Sidecar<Vec<f64>>,
    total: f64,
}

impl Hooked {
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RCondition> {
        let values = Self::values(saved);
        if values.iter().any(|v| *v < 0.0) {
            return Err(quota_condition());
        }
        Ok(Hooked {
            total: values.iter().sum(),
            values: Sidecar::default(),
        })
    }
}

/// `x` saved and restored: a pointer without its Rust value, its slots kept.
fn reload(x: SEXP) -> OwnedProtect {
    // SAFETY: on R's main thread; each value is rooted before the next call
    // allocates.
    unsafe {
        let bytes = OwnedProtect::new(
            RCall::new("serialize")
                .arg(x)
                .arg(SEXP::nil())
                .eval_base()
                .expect("serialize"),
        );
        OwnedProtect::new(
            RCall::new("unserialize")
                .arg(bytes.get())
                .eval_base()
                .expect("unserialize"),
        )
    }
}

/// A `Hooked` pointer with `values`, saved and restored.
fn restored(values: Vec<f64>) -> OwnedProtect {
    let total = values.iter().sum();
    let ptr = ExternalPtr::new(Hooked {
        values: Sidecar::new(values),
        total,
    });
    reload(ptr.as_sexp())
}

/// Holds a `Hooked` pointer. Its hook converts it inside `catch_condition`
/// and returns what that caught: the inner refusal, which carries its own
/// cause.
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(rebuild = Self::from_saved)]
struct Pair {
    #[r_data]
    inner: Sidecar<SEXP>,
    total: f64,
}

impl Pair {
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RCondition> {
        let inner = Self::inner(saved);
        let inner = catch_condition(|| {
            ExternalPtr::<Hooked>::try_from_sexp(inner).expect("a `Hooked` pointer")
        })?;
        let total = inner.borrow().total;
        Ok(Pair {
            inner: Sidecar::new(SEXP::nil()),
            total,
        })
    }
}

/// The `(name, value)` pairs of a named `RValue::List`.
fn entries(value: &RValue) -> &[(Option<String>, RValue)] {
    match value {
        RValue::List(entries) => entries,
        other => panic!("not a list: {other:?}"),
    }
}

fn entry<'a>(entries: &'a [(Option<String>, RValue)], name: &str) -> &'a RValue {
    entries
        .iter()
        .find(|(n, _)| n.as_deref() == Some(name))
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("no `{name}` in {entries:?}"))
}

/// The refusal of a failed `Hooked` rebuild, and its cause: the hook's
/// condition, unchanged.
fn assert_refusal(refusal: &RCondition) {
    assert!(matches!(refusal, RCondition::Error { .. }), "{refusal:?}");
    assert_eq!(
        refusal.class(),
        ["miniextendr_restored_no_value", "miniextendr_restored"]
    );
    assert!(
        refusal
            .message()
            .ends_with("rebuilding it failed: quota exceeded by 3"),
        "{}",
        refusal.message()
    );
    let data = refusal.data().expect("the cause");
    let [(name, parent)] = data.as_slice() else {
        panic!("one field, the cause: {data:?}");
    };
    assert_eq!(name, ".mx_parent");
    let parent = entries(parent);
    assert!(
        matches!(entry(parent, "error"), RValue::Character(v) if v == &[Some("quota exceeded by 3".to_owned())])
    );
    assert!(
        matches!(entry(parent, "kind"), RValue::Character(v) if v == &[Some("error".to_owned())])
    );
    assert!(matches!(
        entry(parent, "class"),
        RValue::Character(v) if v == &[Some("pkg_quota_exceeded".to_owned()), Some("pkg_quota".to_owned())]
    ));
    // `ConditionCall::None` travels as `FALSE`.
    assert!(matches!(entry(parent, "call"), RValue::Logical(v) if v == &[Some(false)]));
    let fields = entries(entry(parent, "data"));
    assert!(matches!(entry(fields, "n"), RValue::Integer(v) if v == &[Some(3)]));
    assert!(
        matches!(entry(fields, "what"), RValue::Character(v) if v == &[Some("rows".to_owned())])
    );
}

#[test]
fn an_rcondition_err_keeps_its_parts_through_the_probe() {
    let parts = miniextendr_api::__mx_result_err_parts!(
        quota_condition(),
        "pkg_error",
        crate::__MiniextendrInitMarker
    );
    assert_eq!(parts.message, "quota exceeded by 3");
    assert_eq!(parts.class, ["pkg_quota_exceeded", "pkg_quota"]);
    let data = parts.data.expect("data");
    assert_eq!(data[0].0, "n");
    assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(3)]));
    assert_eq!(data[1].0, "what");
    assert_eq!(parts.call, ConditionCall::None);
}

/// The `.mx_parent` field of a condition's fields: its cause, as a tagged
/// value.
fn cause_of(data: &[(String, RValue)]) -> &[(Option<String>, RValue)] {
    let (_, parent) = data
        .iter()
        .find(|(name, _)| name == ".mx_parent")
        .unwrap_or_else(|| panic!("no cause in {data:?}"));
    entries(parent)
}

/// A condition with a cause, as a caught refusal is.
fn parented_load() -> RCondition {
    RCondition::Error {
        message: "could not load".to_owned(),
        class: vec!["pkg_load".to_owned()],
        data: None,
        call: ConditionCall::Inherit,
    }
    .with_parent(quota_condition())
}

#[test]
fn a_caught_condition_keeps_its_cause_and_marker_through_the_probe() {
    // A condition with a cause, as a caught refusal is: the cause rides in
    // the reserved `.mx_parent` field, which the probe passes on.
    let parts = miniextendr_api::__mx_result_err_parts!(
        parented_load(),
        "pkg_error",
        crate::__MiniextendrInitMarker
    );
    assert_eq!(parts.class, ["pkg_load"]);
    let data = parts.data.expect("the cause");
    let cause = cause_of(&data);
    assert!(matches!(
        entry(cause, "class"),
        RValue::Character(v) if v == &[Some("pkg_quota_exceeded".to_owned()), Some("pkg_quota".to_owned())]
    ));

    // Converted into a `SexpError`, as a `?` in a conversion does, it keeps
    // the cause on the argument error.
    let refused = SexpError::from(parented_load());
    let parts = miniextendr_api::__mx_conversion_err_parts!(refused);
    assert!(
        parts.class.iter().any(|c| c == "pkg_load"),
        "{:?}",
        parts.class
    );
    let data = parts.data.expect("the cause");
    assert!(matches!(
        entry(cause_of(&data), "class"),
        RValue::Character(v) if v == &[Some("pkg_quota_exceeded".to_owned()), Some("pkg_quota".to_owned())]
    ));

    // A field the author named `.mx_parent` is still refused.
    let named = RError::new("bad").data(".mx_parent", 1i32);
    let refused = std::panic::catch_unwind(|| {
        miniextendr_api::__mx_result_err_parts!(named, "pkg_error", crate::__MiniextendrInitMarker)
    })
    .expect_err("a reserved name");
    let refused = refused.downcast_ref::<String>().expect("a panic message");
    assert!(refused.contains("is reserved"), "{refused}");
    let named = SexpError::from(RError::new("bad").data(".mx_parent", 1i32));
    assert!(
        std::panic::catch_unwind(|| miniextendr_api::__mx_conversion_err_parts!(named)).is_err()
    );

    // An argument error keeps `e$param` and the class marker the generated R
    // replaces with the crate's conversion classes.
    let arg = RCondition::arg_error("x", "bad x".to_owned(), ConditionCall::Inherit);
    let parts =
        miniextendr_api::__mx_result_err_parts!(arg, "pkg_error", crate::__MiniextendrInitMarker);
    assert_eq!(
        parts.class,
        [miniextendr_api::condition::CONVERSION_ERROR_CLASS_MARKER]
    );
    let data = parts.data.expect("param");
    assert_eq!(data[0].0, "param");
    assert!(matches!(&data[0].1, RValue::Character(v) if v == &[Some("x".to_owned())]));
}

#[test]
fn wrap_sexp_with_error_returns_a_failed_rebuild() {
    r_test_utils::with_r_thread(|| {
        let back = restored(vec![1.0, -2.0]);
        match unsafe { ExternalPtr::<Hooked>::wrap_sexp_with_error(back.get()) } {
            Err(TypeMismatchError::RebuildFailed { condition }) => assert_refusal(&condition),
            Err(other) => panic!("another error: {other:?}"),
            Ok(_) => panic!("rebuilt a refused save"),
        }
        // Nothing was installed: the pointer is still restored.
        assert!(unsafe { ExternalPtr::<Hooked>::wrap_sexp(back.get()) }.is_none());
    });
}

#[test]
fn the_conversion_still_raises_the_refusal() {
    r_test_utils::with_r_thread(|| {
        let back = restored(vec![-1.0]);
        let raised = catch_condition(|| ExternalPtr::<Hooked>::try_from_sexp(back.get()).is_ok())
            .expect_err("the conversion raises");
        assert_refusal(&raised);
        let returned = match unsafe { ExternalPtr::<Hooked>::wrap_sexp_with_error(back.get()) } {
            Err(TypeMismatchError::RebuildFailed { condition }) => condition,
            other => panic!("not a failed rebuild: {:?}", other.map(|_| ())),
        };
        assert_eq!(raised.message(), returned.message());
        assert_eq!(raised.class(), returned.class());
    });
}

#[test]
fn a_rebuild_that_succeeds_installs_the_value() {
    r_test_utils::with_r_thread(|| {
        let back = restored(vec![1.0, 2.0]);
        let ptr =
            unsafe { ExternalPtr::<Hooked>::wrap_sexp_with_error(back.get()) }.expect("rebuilt");
        assert_eq!(ptr.borrow().total, 3.0);
        assert_eq!(Hooked::values(&ptr), [1.0, 2.0]);
    });
}

#[test]
fn a_hook_returning_a_caught_refusal_keeps_the_whole_chain() {
    r_test_utils::with_r_thread(|| {
        let inner = ExternalPtr::new(Hooked {
            values: Sidecar::new(vec![-1.0]),
            total: -1.0,
        });
        let pair = ExternalPtr::new(Pair {
            inner: Sidecar::new(inner.as_sexp()),
            total: -1.0,
        });
        drop(inner);
        let back = reload(pair.as_sexp());
        let refusal = match unsafe { ExternalPtr::<Pair>::wrap_sexp_with_error(back.get()) } {
            Err(TypeMismatchError::RebuildFailed { condition }) => condition,
            other => panic!("not a failed rebuild: {:?}", other.map(|_| ())),
        };
        // The pair's refusal, caused by the inner refusal, caused by the
        // inner hook's condition.
        assert!(
            refusal.message().contains("`Pair`"),
            "{}",
            refusal.message()
        );
        let data = refusal.data().expect("the cause");
        let inner_refusal = cause_of(&data);
        assert!(matches!(
            entry(inner_refusal, "class"),
            RValue::Character(v) if v == &[Some("miniextendr_restored_no_value".to_owned()), Some("miniextendr_restored".to_owned())]
        ));
        let inner_fields: Vec<(String, RValue)> = entries(entry(inner_refusal, "data"))
            .iter()
            .map(|(name, value)| (name.clone().unwrap_or_default(), value.clone()))
            .collect();
        let inner_cause = cause_of(&inner_fields);
        assert!(matches!(
            entry(inner_cause, "class"),
            RValue::Character(v) if v == &[Some("pkg_quota_exceeded".to_owned()), Some("pkg_quota".to_owned())]
        ));

        // A good inner save rebuilds both.
        let inner = ExternalPtr::new(Hooked {
            values: Sidecar::new(vec![1.0, 2.0]),
            total: 3.0,
        });
        let pair = ExternalPtr::new(Pair {
            inner: Sidecar::new(inner.as_sexp()),
            total: 3.0,
        });
        drop(inner);
        let back = reload(pair.as_sexp());
        let rebuilt =
            unsafe { ExternalPtr::<Pair>::wrap_sexp_with_error(back.get()) }.expect("rebuilt");
        assert_eq!(rebuilt.borrow().total, 3.0);
    });
}
