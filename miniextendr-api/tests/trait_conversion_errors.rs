//! Trait conversion failures unwind Rust resources and return tagged conditions.
mod r_test_utils;

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use miniextendr_api::condition::RCondition;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::trait_abi::{check_arity, from_sexp};
use miniextendr_api::unwind_protect::with_r_unwind_protect_shim;
use miniextendr_api::{IntoR, SEXP, TryFromSexp};

static ERROR_DROPPED: AtomicBool = AtomicBool::new(false);

pub struct Refusal;

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("conversion refused")
    }
}

impl Drop for Refusal {
    fn drop(&mut self) {
        ERROR_DROPPED.store(true, Ordering::SeqCst);
    }
}

pub struct RefusingValue;

impl TryFromSexp for RefusingValue {
    type Error = Refusal;

    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
        Err(Refusal)
    }
}

struct DropFlag<'a>(&'a Cell<bool>);

impl Drop for DropFlag<'_> {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

fn assert_panic(result: SEXP, expected: &str) {
    let (_, parts) = unsafe { RCondition::from_tagged_sexp(result) }
        .expect("Rust conversion failure must return a tagged condition")
        .into_parts();
    assert!(parts.message.contains(expected), "{}", parts.message);
}

#[test]
fn conversion_failure_drops_the_error_and_previously_converted_arguments() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let dropped = Cell::new(false);
        ERROR_DROPPED.store(false, Ordering::SeqCst);
        let args = [SEXP::nil()];
        let result = scope.protect(with_r_unwind_protect_shim(|| {
            let _previous_arg = DropFlag(&dropped);
            let _: RefusingValue = from_sexp(args[0]);
            SEXP::nil()
        }));
        assert_panic(result.get(), "conversion refused");
        assert!(dropped.get(), "earlier arguments must be dropped");
        assert!(
            ERROR_DROPPED.load(Ordering::SeqCst),
            "error must be dropped"
        );
    });
}

#[test]
fn an_arity_mismatch_panics_inside_the_boundary() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let dropped = Cell::new(false);
        let result = scope.protect(with_r_unwind_protect_shim(|| {
            let _previous_arg = DropFlag(&dropped);
            check_arity(2, 1, "method");
            SEXP::nil()
        }));
        assert_panic(result.get(), "method: expected 1 arguments, got 2");
        assert!(dropped.get());
        let value = scope.protect(42i32.into_sexp()).get();
        assert_eq!(from_sexp::<i32>(value), 42);
    });
}

// The generated paths must inspect concrete error types, not erase them to
// Display in a generic helper. The custom wording and field differ from Display.
impl miniextendr_api::condition::RConditionData for Refusal {
    fn message(&self) -> String {
        "classed refusal".into()
    }
    fn class(&self) -> Vec<String> {
        vec!["test_refusal".into()]
    }
    fn data(&self) -> Option<miniextendr_api::condition::ConditionData> {
        Some(vec![("detail".into(), "retained".into())])
    }
}

impl IntoR for RefusingValue {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(SEXP::nil())
    }
}

static ARG_DROPPED: AtomicBool = AtomicBool::new(false);
pub struct Tracked;
impl Drop for Tracked {
    fn drop(&mut self) {
        ARG_DROPPED.store(true, Ordering::SeqCst);
    }
}
impl TryFromSexp for Tracked {
    type Error = std::convert::Infallible;
    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
impl IntoR for Tracked {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(SEXP::nil())
    }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct Host;

#[miniextendr_api::miniextendr]
pub trait Convert {
    fn number(&self, number: i32);
    fn custom(&self, previous: Tracked, value: RefusingValue);
    fn rooted(&self, values: miniextendr_api::RCow<'_, i32>);
    fn other(&self, other: &Self);
}
impl Convert for Host {
    fn number(&self, _: i32) {
        panic!("conversion must refuse the input");
    }
    fn custom(&self, _: Tracked, _: RefusingValue) {
        panic!("conversion must refuse the input");
    }
    fn rooted(&self, _: miniextendr_api::RCow<'_, i32>) {
        panic!("conversion must refuse the input");
    }
    fn other(&self, _: &Self) {
        panic!("conversion must refuse the input");
    }
}

#[miniextendr_api::miniextendr]
pub trait ConvertGeneric<T> {
    fn number(&self, number: T);
    fn custom(&self, previous: Tracked, value: RefusingValue) {
        let _ = (previous, value);
        panic!("conversion must refuse the input");
    }
    fn rooted(&self, values: miniextendr_api::RCow<'_, i32>);
    fn other(&self, other: &Self);
}
#[miniextendr_api::miniextendr(env, noexport)]
impl ConvertGeneric<i32> for Host {
    fn number(&self, _renamed: i32) {
        panic!("conversion must refuse the input");
    }
    fn rooted(&self, _renamed: miniextendr_api::RCow<'_, i32>) {
        panic!("conversion must refuse the input");
    }
    fn other(&self, _renamed: &Self) {
        panic!("conversion must refuse the input");
    }
}

fn check_generated_shims(shims: [miniextendr_api::abi::mx_meth; 4]) {
    use miniextendr_api::RValue;
    unsafe {
        let scope = ProtectScope::new();
        let text = scope.protect("wrong type".into_sexp()).get();
        let host = std::cell::RefCell::new(Host);
        for (shim, param, rust_type, args) in [
            (shims[0], "number", "i32", vec![text]),
            (shims[1], "value", "RefusingValue", vec![SEXP::nil(), text]),
            (
                shims[2],
                "values",
                "miniextendr_api::RCow<'_, i32>",
                vec![text],
            ),
            (shims[3], "other", "&Self", vec![text]),
        ] {
            ERROR_DROPPED.store(false, Ordering::SeqCst);
            ARG_DROPPED.store(false, Ordering::SeqCst);
            let tagged = scope
                .protect(shim(
                    std::ptr::from_ref(&host).cast_mut().cast(),
                    i32::try_from(args.len()).unwrap(),
                    args.as_ptr(),
                ))
                .get();
            // Exercise the same decode/re-encode step a View and its outer
            // wrapper use, so kind and fields must survive the ABI round trip.
            let condition = RCondition::from_tagged_sexp(tagged).expect("tagged conversion error");
            let returned = scope
                .protect(miniextendr_api::error_value::rust_condition_value(
                    condition, None,
                ))
                .get();
            let (kind, parts) = RCondition::from_tagged_sexp(returned).unwrap().into_parts();
            assert_eq!(kind, "conversion");
            let fields = parts.data.unwrap();
            for (name, expected) in [("param", param), ("rust_type", rust_type)] {
                let value = fields.iter().find(|(key, _)| key == name).unwrap();
                assert!(
                    matches!(&value.1, RValue::Character(v) if v == &[Some(expected.into())]),
                    "{value:?}"
                );
            }
            assert!(parts.message.contains(param), "{}", parts.message);
            if param == "value" {
                assert_eq!(parts.class, ["test_refusal"]);
                assert!(
                    parts.message.contains("classed refusal"),
                    "{}",
                    parts.message
                );
                assert!(fields.iter().any(|(key, value)| key == "detail"
                    && matches!(value, RValue::Character(v) if v == &[Some("retained".into())])));
                assert!(ERROR_DROPPED.load(Ordering::SeqCst));
                assert!(ARG_DROPPED.load(Ordering::SeqCst));
            }
        }
    }
}

#[test]
fn generated_shims_preserve_conversion_context_and_custom_conditions() {
    r_test_utils::with_r_thread(|| {
        let plain = __Convert_build_vtable::<Host>();
        check_generated_shims([plain.number, plain.custom, plain.rooted, plain.other]);
        let entry = miniextendr_api::registry::MX_TRAIT_DISPATCH
            .iter()
            .find(|entry| entry.trait_tag == TAG_ConvertGeneric)
            .unwrap();
        let generic = unsafe { &*entry.vtable.cast::<ConvertGenericVTable>() };
        check_generated_shims([
            generic.number,
            generic.custom,
            generic.rooted,
            generic.other,
        ]);
    });
}

#[test]
fn inherited_c_wrapper_keeps_custom_conversion_condition() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let host = scope
            .protect(miniextendr_api::ExternalPtr::new(Host).into_sexp())
            .get();
        let text = scope.protect("wrong type".into_sexp()).get();
        let value = scope
            .protect(C_trait_conversion_errors_Host__ConvertGeneric__custom(
                SEXP::nil(),
                host,
                SEXP::nil(),
                text,
            ))
            .get();
        let (kind, parts) = RCondition::from_tagged_sexp(value).unwrap().into_parts();
        assert_eq!(kind, "conversion");
        assert_eq!(parts.class, ["test_refusal"]);
        assert!(
            parts.message.contains("classed refusal"),
            "{}",
            parts.message
        );
        assert!(parts.data.unwrap().iter().any(|(name, _)| name == "detail"));
    });
}
