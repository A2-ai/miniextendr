//! `Computed` fields (#1883, #2269): a read-only R field whose generated
//! getter calls a plain Rust function of the live value on every read. The
//! getter's value converts with `IntoR`; a `Result<T, E>` getter raises its
//! `Err` with the error's classes, `Err(())` of a `Result<T, ()>` and a unit
//! getter read `NULL`; there is no setter; a pointer restored from a saved
//! session is refused.

mod r_test_utils;

use miniextendr_api::condition::{RCondition, RConditionData};
use miniextendr_api::expression::RCall;
use miniextendr_api::externalptr::{Computed, RSidecar};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, IntoR, SEXP, TryFromSexp};

// Defines this crate's marker type, `__MiniextendrInitMarker`, which the
// `Err` arm of a `Result` getter names under the `serde` feature.
miniextendr_api::miniextendr_init!(computed_fields);

/// The level above which [`Gauge::checked`] refuses.
const LIMIT: i32 = 10;

/// A level over [`LIMIT`], with its classes and the level as a field.
#[derive(Debug)]
pub struct OverLimit {
    level: i32,
}

impl RConditionData for OverLimit {
    fn message(&self) -> String {
        format!("level {} is over the limit {LIMIT}", self.level)
    }

    fn class(&self) -> Vec<String> {
        vec!["over_limit".to_string(), "gauge_error".to_string()]
    }

    fn data(&self) -> Option<miniextendr_api::condition::ConditionData> {
        Some(vec![("level".to_string(), self.level.into())])
    }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct Gauge {
    #[r_data]
    _r: RSidecar,
    /// Twice the level.
    #[r_data(get = "Self::doubled")]
    pub doubled: Computed,
    /// The level, or the classed [`OverLimit`] above [`LIMIT`].
    #[r_data(get = "Self::checked")]
    pub checked: Computed,
    /// The level, or `NULL` above [`LIMIT`].
    #[r_data(get = "Self::within_limit")]
    pub within_limit: Computed,
    /// Always `NULL`.
    #[r_data(get = "Self::nothing")]
    pub nothing: Computed,
    /// The level, a struct field with a setter.
    #[r_data]
    pub level: i32,
}

impl Gauge {
    fn doubled(&self) -> i32 {
        self.level * 2
    }

    fn checked(&self) -> Result<i32, OverLimit> {
        if self.level > LIMIT {
            Err(OverLimit { level: self.level })
        } else {
            Ok(self.level)
        }
    }

    fn within_limit(&self) -> Result<i32, ()> {
        self.checked().map_err(|_| ())
    }

    fn nothing(&self) {}
}

fn gauge(level: i32) -> Gauge {
    Gauge {
        _r: RSidecar,
        doubled: Computed,
        checked: Computed,
        within_limit: Computed,
        nothing: Computed,
        level,
    }
}

/// The registered `.Call` entry whose name ends with `suffix`, if any.
fn entry(suffix: &str) -> Option<&'static miniextendr_api::sys::R_CallMethodDef> {
    miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(suffix.as_bytes())
        })
}

/// The field `name` of `x`, read through its generated R getter: the value,
/// or the tagged condition the getter returned.
fn read(x: SEXP, name: &str) -> Result<SEXP, RCondition> {
    let entry = entry(&format!("__mx_rdata_get_Gauge_{name}"))
        .unwrap_or_else(|| panic!("the getter of `{name}` must be registered"));
    assert_eq!(entry.numArgs, 2);
    let getter = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
        >(entry.fun.unwrap())
    };
    let value = unsafe { getter(x, false.into_sexp()) };
    match unsafe { RCondition::from_tagged_sexp(value) } {
        Some(condition) => Err(condition),
        None => Ok(value),
    }
}

fn int(x: SEXP) -> i32 {
    i32::try_from_sexp(x).unwrap()
}

#[test]
fn a_computed_field_is_a_zero_sized_marker() {
    assert_eq!(size_of::<Computed>(), 0);
    assert_eq!(Computed, Computed::default());
}

#[test]
fn computed_getters_read_the_live_value_on_every_read() {
    r_test_utils::with_r_thread(|| {
        let mut owner = ExternalPtr::new(gauge(3));
        let x = owner.as_sexp();
        assert_eq!(int(read(x, "doubled").unwrap()), 6);
        assert_eq!(int(read(x, "checked").unwrap()), 3);
        assert_eq!(int(read(x, "within_limit").unwrap()), 3);
        assert!(read(x, "nothing").unwrap().is_nil());

        // Nothing is kept: a change to the value shows on the next read.
        owner.borrow_mut().level = 5;
        assert_eq!(int(read(x, "doubled").unwrap()), 10);
        assert_eq!(int(read(x, "checked").unwrap()), 5);
    });
}

#[test]
fn result_getters_raise_or_read_null() {
    r_test_utils::with_r_thread(|| {
        let owner = ExternalPtr::new(gauge(LIMIT + 1));
        let x = owner.as_sexp();
        match read(x, "checked") {
            Err(RCondition::Error {
                message,
                class,
                data,
                ..
            }) => {
                assert_eq!(message, "level 11 is over the limit 10");
                assert_eq!(&class[..2], ["over_limit", "gauge_error"]);
                let data = data.expect("the error's fields");
                assert_eq!(data.len(), 1);
                assert_eq!(data[0].0, "level");
            }
            other => panic!("expected the classed error, got {other:?}"),
        }
        // `Err(())` reads `NULL`, not an error.
        assert!(read(x, "within_limit").unwrap().is_nil());
        // An infallible getter of the same value still reads.
        assert_eq!(int(read(x, "doubled").unwrap()), 22);
    });
}

#[test]
fn a_computed_field_has_no_setter() {
    // The struct field next to it has one.
    assert!(entry("__mx_rdata_set_Gauge_level").is_some());
    for field in ["doubled", "checked", "within_limit", "nothing"] {
        assert!(entry(&format!("__mx_rdata_get_Gauge_{field}")).is_some());
        assert!(
            entry(&format!("__mx_rdata_set_Gauge_{field}")).is_none(),
            "`{field}` is computed, so read-only"
        );
    }
}

#[test]
fn a_restored_pointer_is_refused() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let owner = ExternalPtr::new(gauge(2));
        // A save and a reload keep the type ID but not the Rust value.
        let saved = scope
            .protect(
                RCall::new("serialize")
                    .arg(owner.as_sexp())
                    .arg(SEXP::nil())
                    .eval_base()
                    .unwrap(),
            )
            .get();
        let back = scope
            .protect(RCall::new("unserialize").arg(saved).eval_base().unwrap())
            .get();
        match read(back, "doubled") {
            Err(RCondition::Error { class, .. }) => {
                assert!(
                    class.iter().any(|c| c == "miniextendr_restored_no_value"),
                    "{class:?}"
                );
            }
            other => panic!("expected the restored refusal, got {other:?}"),
        }
        // The live pointer still reads.
        assert_eq!(int(read(owner.as_sexp(), "doubled").unwrap()), 4);
    });
}
