//! A serde type's value comes back after `unserialize()` (#2186): the derive's
//! probe finds serde, the pointer carries the value into the save, and the
//! first access attaches what the carrier decoded. A type that is not serde
//! keeps the restored refusal. A save is found by its crate and type name, so
//! one made before the type moved to another module comes back too.
#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::condition::{RCondition, RConditionData, catch_condition};
use miniextendr_api::expression::RCall;
use miniextendr_api::externalptr::{
    ExternalPtr, Sidecar, TypeMismatchError, TypedExternal, rewrite_stored_type_for_tests,
};
use miniextendr_api::{OwnedProtect, SEXP, SexpExt, TryFromSexp};

/// A serde type, with a sidecar field the save keeps itself.
#[derive(miniextendr_api::ExternalPtr, serde::Serialize, serde::Deserialize)]
struct Counter {
    n: i32,
    label: String,
    #[serde(skip)]
    #[r_data]
    tags: Sidecar<Vec<String>>,
}

/// Not serde: refused after a reload, as before.
#[derive(miniextendr_api::ExternalPtr)]
struct Plain {
    _n: i32,
}

/// Serde, but its decoding refuses a negative value.
#[derive(miniextendr_api::ExternalPtr, serde::Serialize, serde::Deserialize)]
struct Positive {
    #[serde(deserialize_with = "non_negative")]
    n: i32,
}

fn non_negative<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
    let n = <i32 as serde::Deserialize>::deserialize(d)?;
    if n < 0 {
        return Err(serde::de::Error::custom(format!("{n} is negative")));
    }
    Ok(n)
}

/// Two serde types of this crate named `Twin`, in two modules: they would
/// share a carrier class.
mod first {
    #[derive(miniextendr_api::ExternalPtr, serde::Serialize, serde::Deserialize)]
    pub struct Twin {
        pub n: i32,
    }
}

mod second {
    #[derive(miniextendr_api::ExternalPtr, serde::Serialize, serde::Deserialize)]
    pub struct Twin {
        pub n: i32,
    }
}

/// Values the general serde-to-R mapping can't read back (#2256): the save
/// uses the exact encoding.
#[derive(miniextendr_api::ExternalPtr, serde::Serialize, serde::Deserialize)]
struct Shapes {
    values: Values,
    notice: Notice,
    na: f64,
    nested: Option<Option<f64>>,
    min: i32,
    big: u64,
    by_id: std::collections::HashMap<u32, f64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Values {
    Many(Vec<String>),
    Flag(bool),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(tag = "kind")]
enum Notice {
    Dropped { profiles: Vec<String> },
}

/// Registers the carrier classes, as `R_init` does through the derive's
/// `MX_ALTREP_REGISTRATIONS` entries.
fn register() {
    miniextendr_api::externalptr::__mx_register_snapshot::<Counter>();
    miniextendr_api::externalptr::__mx_register_snapshot::<Plain>();
    miniextendr_api::externalptr::__mx_register_snapshot::<Positive>();
    miniextendr_api::externalptr::__mx_register_snapshot::<Shapes>();
}

/// `x` saved and read back.
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
        let back = RCall::new("unserialize")
            .arg(bytes.get())
            .eval_base()
            .expect("unserialize");
        // The protect stack is LIFO: release `bytes` before protecting the
        // result (unprotecting allocates nothing).
        drop(bytes);
        OwnedProtect::new(back)
    }
}

#[test]
fn the_probe_finds_serde() {
    assert!(Counter::__mx_snapshot().is_some());
    assert!(Positive::__mx_snapshot().is_some());
    assert!(Plain::__mx_snapshot().is_none());
}

#[test]
fn a_serde_value_comes_back() {
    r_test_utils::with_r_thread(|| {
        register();
        let mut ptr = ExternalPtr::new(Counter {
            n: 7,
            label: "seven".to_owned(),
            tags: Sidecar::new(vec!["a".to_owned(), "b".to_owned()]),
        });
        Counter::set_tags(&mut ptr, vec!["kept".to_owned()]);
        let back = reload(ptr.as_sexp());
        // Read back, then saved again before any access: the second save
        // writes the state the first read.
        let back = reload(back.get());
        // The non-raising wrap attaches the value too.
        let again = reload(back.get());
        let wrapped = unsafe { ExternalPtr::<Counter>::wrap_sexp(again.get()) }.expect("attached");
        assert_eq!(wrapped.borrow().n, 7);
        drop(wrapped);
        drop(again);
        let mut restored = ExternalPtr::<Counter>::try_from_sexp(back.get()).expect("attached");
        assert_eq!(restored.borrow().n, 7);
        assert_eq!(restored.borrow().label, "seven");
        assert_eq!(Counter::tags(&restored), ["kept"]);
        // Attached, it saves its current value.
        restored.borrow_mut().n = 8;
        let again = reload(back.get());
        let again = ExternalPtr::<Counter>::try_from_sexp(again.get()).expect("attached");
        assert_eq!(again.borrow().n, 8);
    });
}

#[test]
fn a_value_the_general_mapping_changes_comes_back_exactly() {
    r_test_utils::with_r_thread(|| {
        register();
        let na = f64::from_bits(0x7ff0_0000_0000_07a2);
        let ptr = ExternalPtr::new(Shapes {
            values: Values::Many(vec!["1".to_owned()]),
            notice: Notice::Dropped {
                profiles: vec!["ID=1".to_owned()],
            },
            na,
            nested: Some(None),
            min: i32::MIN,
            big: 9_007_199_254_740_993,
            by_id: [(u32::MAX, -0.0)].into(),
        });
        let back = reload(ptr.as_sexp());
        let back = ExternalPtr::<Shapes>::try_from_sexp(back.get()).expect("attached");
        let back = back.borrow();
        assert_eq!(back.values, Values::Many(vec!["1".to_owned()]));
        assert_eq!(
            back.notice,
            Notice::Dropped {
                profiles: vec!["ID=1".to_owned()]
            }
        );
        assert_eq!(back.na.to_bits(), na.to_bits());
        assert_eq!(back.nested, Some(None));
        assert_eq!(back.min, i32::MIN);
        assert_eq!(back.big, 9_007_199_254_740_993);
        assert_eq!(back.by_id[&u32::MAX].to_bits(), (-0.0f64).to_bits());
    });
}

#[test]
fn a_save_while_a_call_holds_the_value_mutably_is_a_borrow_conflict() {
    r_test_utils::with_r_thread(|| {
        register();
        let mut ptr = ExternalPtr::new(Counter {
            n: 1,
            label: "held".to_owned(),
            tags: Sidecar::new(Vec::new()),
        });
        let sexp = ptr.as_sexp();
        {
            // A `&mut self` method running R code that saves the object.
            let mut held = ptr.borrow_mut();
            held.n = 2;
            // SAFETY: on R's main thread; `ptr` roots `sexp`.
            let err = unsafe {
                RCall::new("serialize")
                    .arg(sexp)
                    .arg(SEXP::nil())
                    .eval_base()
            }
            .expect_err("the save reads the value through a borrow");
            assert!(
                err.inherits("miniextendr_borrow_conflict"),
                "{:?}",
                err.classes()
            );
            assert!(
                err.message()
                    .contains("is in use by a call that is still running"),
                "{}",
                err.message()
            );
        }
        // Released, the save reads the value.
        let back = reload(sexp);
        let back = ExternalPtr::<Counter>::try_from_sexp(back.get()).expect("attached");
        assert_eq!(back.borrow().n, 2);
    });
}

#[test]
fn a_type_that_is_not_serde_is_refused() {
    r_test_utils::with_r_thread(|| {
        register();
        let ptr = ExternalPtr::new(Plain { _n: 1 });
        let back = reload(ptr.as_sexp());
        match catch_condition(|| ExternalPtr::<Plain>::try_from_sexp(back.get()).is_ok()) {
            Ok(converted) => assert!(!converted, "a plain type has no saved value"),
            Err(refusal) => assert!(
                refusal
                    .class()
                    .iter()
                    .any(|c| c == "miniextendr_restored_no_value"),
                "{refusal:?}"
            ),
        }
        assert!(unsafe { ExternalPtr::<Plain>::wrap_sexp(back.get()) }.is_none());
    });
}

#[test]
fn a_save_that_does_not_decode_is_refused_with_its_cause() {
    r_test_utils::with_r_thread(|| {
        register();
        let ptr = ExternalPtr::new(Positive { n: -3 });
        let back = reload(ptr.as_sexp());
        let refusal: RCondition =
            catch_condition(|| ExternalPtr::<Positive>::try_from_sexp(back.get()).is_ok())
                .expect_err("the conversion raises the refusal");
        assert_eq!(
            refusal.class(),
            ["miniextendr_restored_no_value", "miniextendr_restored"]
        );
        assert!(
            refusal
                .message()
                .contains("could not be read: -3 is negative"),
            "{}",
            refusal.message()
        );
        let RCondition::Error { data, .. } = &refusal else {
            panic!("{refusal:?}")
        };
        let data = data.as_ref().expect("the parent rides in the data");
        assert!(
            data.iter().any(|(name, _)| name == ".mx_parent"),
            "{data:?}"
        );
        // The pointer stays without a value, and refuses again.
        assert!(unsafe { ExternalPtr::<Positive>::wrap_sexp(back.get()) }.is_none());
        assert!(
            catch_condition(|| ExternalPtr::<Positive>::try_from_sexp(back.get()).is_ok()).is_err()
        );
    });
}

#[test]
fn the_carrier_copies_as_an_empty_raw_vector() {
    r_test_utils::with_r_thread(|| {
        register();
        let ptr = ExternalPtr::new(Positive { n: 1 });
        // SAFETY: on R's main thread; the handle roots its pointer and its
        // protection list, whose last element is the carrier.
        unsafe {
            let prot = ptr.prot_raw();
            let carrier = prot.vector_elt(prot.xlength() - 1);
            assert!(carrier.is_altrep());
            // R's default `Duplicate` copies the data through `DATAPTR()`.
            let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(carrier));
            assert_eq!(copy.get().type_of(), miniextendr_api::SEXPTYPE::RAWSXP);
            assert_eq!(copy.get().xlength(), 0);
        }
    });
}

/// A live `Counter` whose type ID says it was built by `crate_name` for the
/// type at `path`, saved and read back.
fn reload_as(crate_name: &str, path: &str) -> OwnedProtect {
    let mut ptr = ExternalPtr::new(Counter {
        n: 5,
        label: "moved".to_owned(),
        tags: Sidecar::new(Vec::new()),
    });
    Counter::set_tags(&mut ptr, vec!["t".to_owned()]);
    // SAFETY: on R's main thread; `ptr` roots its pointer.
    unsafe { rewrite_stored_type_for_tests(ptr.as_sexp(), crate_name, path) };
    reload(ptr.as_sexp())
}

#[test]
fn a_save_made_before_the_type_moved_comes_back() {
    r_test_utils::with_r_thread(|| {
        register();
        let back = reload_as(env!("CARGO_PKG_NAME"), "snapshot::models::Counter");
        let restored = ExternalPtr::<Counter>::try_from_sexp(back.get()).expect("attached");
        assert_eq!(restored.borrow().n, 5);
        assert_eq!(restored.borrow().label, "moved");
        assert_eq!(Counter::tags(&restored), ["t"]);
        // Attached, the pointer carries the type's own ID again.
        drop(restored);
        let again = reload(back.get());
        let again = ExternalPtr::<Counter>::try_from_sexp(again.get()).expect("attached");
        assert_eq!(again.borrow().n, 5);
    });
}

#[test]
fn a_save_of_another_type_name_or_crate_is_not_attached() {
    r_test_utils::with_r_thread(|| {
        register();
        for (crate_name, path, found) in [
            (
                env!("CARGO_PKG_NAME"),
                "snapshot::Tally",
                "::snapshot::Tally",
            ),
            ("otherpkg", "snapshot::Counter", "otherpkg@"),
        ] {
            let back = reload_as(crate_name, path);
            // SAFETY: on R's main thread; `back` roots the pointer.
            match unsafe { ExternalPtr::<Counter>::wrap_sexp_with_error(back.get()) } {
                Err(TypeMismatchError::Mismatch { found: id, .. }) => {
                    assert!(id.contains(found), "{id}")
                }
                Err(other) => panic!("{path}: {other:?}"),
                Ok(_) => panic!("{path}: a save of another type was attached"),
            }
            assert!(unsafe { ExternalPtr::<Counter>::wrap_sexp(back.get()) }.is_none());
        }
    });
}

#[test]
fn two_serde_types_of_one_crate_with_the_same_name_fail_to_register() {
    r_test_utils::with_r_thread(|| {
        miniextendr_api::externalptr::__mx_register_snapshot::<first::Twin>();
        // Registering a type again is a no-op.
        miniextendr_api::externalptr::__mx_register_snapshot::<first::Twin>();
        let payload = std::panic::catch_unwind(|| {
            miniextendr_api::externalptr::__mx_register_snapshot::<second::Twin>()
        })
        .expect_err("the second `Twin` would share the first one's class");
        let message = payload
            .downcast_ref::<String>()
            .expect("a formatted panic message");
        assert!(
            message.contains("`snapshot::first::Twin` and `snapshot::second::Twin`"),
            "{message}"
        );
        // The first one still works.
        let ptr = ExternalPtr::new(first::Twin { n: 3 });
        let back = reload(ptr.as_sexp());
        let back = ExternalPtr::<first::Twin>::try_from_sexp(back.get()).expect("attached");
        assert_eq!(back.borrow().n, 3);
    });
}
