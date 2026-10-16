//! Bound-based condition dispatch for the exported wrapper macros.
//!
//! Indexing chooses the first applicable reference tier without consulting
//! consumer traits. Its output is a copyable strategy with by-value inherent
//! methods, so executing it is unambiguous too. Keep execution as a direct
//! call: a function-pointer strategy would lose `#[track_caller]` propagation.
//! Each operation has its own witness type: multiple Index implementations
//! differing only in the index type can stop autoderef at the wrong tier.
//! The error itself is only borrowed and never stored in a strategy.

use super::{ConditionCall, ConditionData, ErrParts, RConditionData, check_condition_data};
use std::marker::PhantomData;
use std::ops::Index;

/// Type witness inferred from the borrowed error.
pub struct Probe<E, Operation>(PhantomData<fn() -> (E, Operation)>);

impl<E, Operation> Probe<E, Operation> {
    pub fn new(_: &E) -> Self {
        Self(PhantomData)
    }
}

/// A selected strategy: 0 = fallback, 1 = trait-based, 2 = built-in.
pub struct Selected<E, const KIND: u8>(PhantomData<fn() -> E>);

impl<E, const KIND: u8> Copy for Selected<E, KIND> {}
impl<E, const KIND: u8> Clone for Selected<E, KIND> {
    fn clone(&self) -> Self {
        *self
    }
}

/// First result-error stage: prefer `RConditionData`.
pub struct ClassedParts;
/// Second result-error stage: prefer serde + Display, then Debug.
pub struct FallbackParts;
/// Conversion reason, classes, fields and call policy.
pub struct ConversionParts;
/// Runtime R-facing expectation, when a built-in error knows one.
pub struct Expectation;
/// Whole argument message, when the error supplies one.
pub struct ArgumentMessage;

macro_rules! select {
    ($receiver:ty, $kind:literal $(, $($bound:tt)+)?) => {
        impl<E> Index<()> for $receiver $(where E: $($bound)+)? {
            type Output = Selected<E, $kind>;
            fn index(&self, _: ()) -> &Self::Output {
                &Selected(PhantomData)
            }
        }
    };
}

select!(&Probe<E, ClassedParts>, 1, RConditionData);
select!(Probe<E, ClassedParts>, 0);
#[cfg(feature = "serde")]
select!(
    &Probe<E, FallbackParts>,
    1,
    ::serde::Serialize + std::fmt::Display
);
select!(Probe<E, FallbackParts>, 0);
select!(&&Probe<E, ConversionParts>, 2, BuiltinError);
select!(&Probe<E, ConversionParts>, 1, RConditionData);
select!(Probe<E, ConversionParts>, 0);
select!(&Probe<E, Expectation>, 2, BuiltinError);
select!(Probe<E, Expectation>, 0);
select!(&&Probe<E, ArgumentMessage>, 2, BuiltinError);
select!(&Probe<E, ArgumentMessage>, 1, RConditionData);
select!(Probe<E, ArgumentMessage>, 0);

impl<E> Selected<E, 0> {
    pub fn classed(self, _: &E) -> Option<ErrParts> {
        None
    }
    pub fn expectation(self, _: &E) -> Option<String> {
        None
    }
    pub fn argument_message(self, _: &E, _: &str) -> Option<String> {
        None
    }

    // Keep these methods inherent even for an unsupported E. A method bound
    // reports the missing formatting trait, rather than "cannot index", and
    // prevents a consumer trait from supplying a substitute method.
    pub fn fallback(self, e: &E, _: &str) -> ErrParts
    where
        E: std::fmt::Debug,
    {
        plain_parts(format!("{e:?}"))
    }

    pub fn conversion(self, e: &E, _: bool) -> ErrParts
    where
        E: std::fmt::Display,
    {
        plain_parts(e.to_string())
    }
}

impl<E: RConditionData> Selected<E, 1> {
    #[track_caller]
    pub fn classed(self, e: &E) -> Option<ErrParts> {
        Some(self.conversion(e, false))
    }

    #[track_caller]
    pub fn conversion(self, e: &E, _: bool) -> ErrParts {
        ErrParts {
            message: e.message(),
            class: e.class(),
            data: check_condition_data(e.data()),
            call: e.call(),
        }
    }

    pub fn argument_message(self, e: &E, _: &str) -> Option<String> {
        e.argument_message()
    }
}

#[cfg(feature = "serde")]
impl<E: ::serde::Serialize + std::fmt::Display> Selected<E, 1> {
    #[track_caller]
    pub fn fallback(self, e: &E, prefix: &str) -> ErrParts {
        super::serde_err_parts(e, "kind", prefix, &[], &[])
    }
}

impl<E: BuiltinError> Selected<E, 2> {
    pub fn conversion(self, e: &E, expected_known: bool) -> ErrParts {
        e.parts(expected_known)
    }
    pub fn expectation(self, e: &E) -> Option<String> {
        e.expectation()
    }
    pub fn argument_message(self, e: &E, name: &str) -> Option<String> {
        e.argument_message(name)
    }
}

/// Framework conversion errors, whose wording targets R users instead of
/// their Rust `Display` diagnostics. These types do not implement
/// `RConditionData`. Public only for the exported macros' bound selection.
pub trait BuiltinError {
    /// Omit an expectation already present in the wrapper's message prefix.
    fn parts(&self, expected_known: bool) -> ErrParts;
    fn expectation(&self) -> Option<String> {
        None
    }
    fn argument_message(&self, _name: &str) -> Option<String> {
        None
    }
}

fn plain_parts(message: String) -> ErrParts {
    ErrParts {
        message,
        class: Vec::new(),
        data: None,
        call: ConditionCall::Inherit,
    }
}

impl BuiltinError for crate::from_r::SexpTypeError {
    fn parts(&self, expected_known: bool) -> ErrParts {
        plain_parts(self.r_reason(expected_known))
    }
}

impl BuiltinError for crate::from_r::SexpClassError {
    fn parts(&self, expected_known: bool) -> ErrParts {
        plain_parts(self.r_reason(expected_known))
    }
}

impl BuiltinError for crate::from_r::SexpLengthError {
    fn parts(&self, expected_known: bool) -> ErrParts {
        plain_parts(self.r_reason(expected_known))
    }
}

impl BuiltinError for crate::from_r::SexpNaError {
    fn parts(&self, _: bool) -> ErrParts {
        plain_parts("NA is not allowed".to_string())
    }
}

impl BuiltinError for crate::from_r::SexpError {
    fn parts(&self, expected_known: bool) -> ErrParts {
        let mut parts = plain_parts(self.r_reason(expected_known));
        let mut data = ConditionData::new();
        for refusal in self.reported_conditions() {
            for class in refusal.classes() {
                if !parts.class.contains(class) {
                    parts.class.push(class.clone());
                }
            }
            for (name, value) in refusal.fields() {
                if !data.iter().any(|(seen, _)| seen == name) {
                    data.push((name.clone(), value.clone()));
                }
            }
        }
        parts.data = check_condition_data((!data.is_empty()).then_some(data));
        parts
    }

    fn expectation(&self) -> Option<String> {
        self.r_expectation()
    }

    fn argument_message(&self, name: &str) -> Option<String> {
        match self {
            Self::MatchArg(e) => Some(e.arg_error(name).message().to_string()),
            // A type's own refusal, including check_sexp under Option/Vec.
            // An Either reports both arms; it is not a whole-message error.
            Self::Condition(e) => e.argument_message_str().map(str::to_string),
            _ => None,
        }
    }
}

impl BuiltinError for crate::match_arg::MatchArgError {
    fn parts(&self, expected_known: bool) -> ErrParts {
        plain_parts(self.r_reason(expected_known))
    }
    fn expectation(&self) -> Option<String> {
        Some(self.expectation())
    }
    fn argument_message(&self, name: &str) -> Option<String> {
        Some(self.arg_error(name).message().to_string())
    }
}
