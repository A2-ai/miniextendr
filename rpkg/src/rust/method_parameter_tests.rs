//! Parameter attributes across inherent, trait-impl and declaration metadata (#2130).
use miniextendr_api::{ExternalPtr, MatchArg, miniextendr};

#[derive(Clone, Copy, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum ParameterMode {
    Fast,
    #[match_arg(alias = "careful")]
    Safe,
}

/// Parameter attribute regression fixture.
#[derive(ExternalPtr)]
pub struct ParameterOptions;

#[miniextendr(env)]
impl ParameterOptions {
    pub fn new() -> Self {
        Self
    }

    pub fn mode(
        &self,
        #[miniextendr(match_arg, default = "safe")] mode: ParameterMode,
    ) -> ParameterMode {
        mode
    }

    pub fn modes(
        &self,
        #[miniextendr(match_arg, several_ok)] modes: Vec<ParameterMode>,
    ) -> Vec<ParameterMode> {
        modes
    }

    pub fn labels(
        &self,
        #[miniextendr(choices("fast", "safe"), several_ok, no_default)] labels: Vec<String>,
    ) -> Vec<String> {
        labels
    }

    pub fn checked(&self, #[miniextendr(coerce, no_na)] value: i32) -> i32 {
        value
    }

    pub fn defaulted(&self, #[miniextendr(default = "7L")] value: i32) -> i32 {
        value
    }

    pub fn classes(
        &self,
        #[miniextendr(inherits = "allowed", not_inherits = "blocked", no_na)] value: Vec<f64>,
    ) -> Vec<f64> {
        value
    }

    pub fn unchecked(&self, #[miniextendr(no_preconditions)] value: i32) -> i32 {
        value
    }

    #[miniextendr(no_preconditions)]
    pub fn rechecked(&self, #[miniextendr(preconditions)] value: i32) -> i32 {
        value
    }

    pub fn converted(&self, #[miniextendr(convert)] value: &ParameterNumber) -> f64 {
        *value
    }
}

type ParameterNumber = f64;

#[miniextendr]
pub trait ParameterTrait {
    fn declared(&self, #[miniextendr(choices("fast", "safe"))] mode: String) -> String;
    fn implemented(&self, mode: ParameterMode) -> ParameterMode;
    fn named(&self, mode: ParameterMode) -> ParameterMode;
    fn inferred(&self, #[miniextendr(coerce, default = "3L")] n: i32) -> i32;
}

#[miniextendr(env)]
impl ParameterTrait for ParameterOptions {
    fn declared(&self, renamed: String) -> String {
        renamed
    }
    fn implemented(
        &self,
        #[miniextendr(match_arg, no_default)] mode: ParameterMode,
    ) -> ParameterMode {
        mode
    }
    #[miniextendr(match_arg(mode), defaults(mode = "safe"))]
    fn named(&self, mode: ParameterMode) -> ParameterMode {
        mode
    }
    fn inferred(&self, n: i32) -> i32 {
        n
    }
}

// The private alias must resolve at the declaration site, including the choice
// registration of an empty impl. The trait remains usable as a trait object.
mod declaration {
    use super::*;
    type PrivateMode = ParameterMode;
    #[miniextendr]
    pub trait ParameterDefault {
        fn inherited(
            &self,
            #[miniextendr(match_arg, default = "safe")] mode: PrivateMode,
        ) -> String {
            mode.to_choice().to_owned()
        }
        fn checked(&self, #[miniextendr(coerce, no_na)] n: i32) -> i32 {
            n
        }
    }
}
impl declaration::ParameterDefault for ParameterOptions {}
#[miniextendr(env)]
impl declaration::ParameterDefault for ParameterOptions {}

#[miniextendr]
pub trait ParameterGeneric<T> {
    fn generic_mode(&self, #[miniextendr(match_arg)] mode: ParameterMode) -> ParameterMode {
        mode
    }
    fn value(&self) -> T;
    fn generic_default(&self, #[miniextendr(choices("fast", "safe"))] mode: String) -> String {
        mode
    }
}
#[miniextendr(env)]
impl ParameterGeneric<i32> for ParameterOptions {
    fn value(&self) -> i32 {
        1
    }
}
