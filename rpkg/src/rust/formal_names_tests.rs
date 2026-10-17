//! Explicit R parameter names across the wrapper generators (#1842).
use miniextendr_api::{ExternalPtr, MatchArg, Missing, Quoted, SEXP, miniextendr};

#[derive(Clone, Copy, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum FormalMode {
    Fast,
    Safe,
}

/// Dotted names, defaults, omission, choice matching and diagnostics.
/// @param conf.int Include an interval.
/// @param conf.level Confidence level.
/// @param optional.value Optional integer.
/// @param pick.mode Processing mode.
/// @param literal.mode Literal processing mode.
#[miniextendr(internal)]
pub fn formal_arguments(
    #[miniextendr(formal = "conf.int", default = "FALSE")] conf_int: bool,
    #[miniextendr(formal = "conf.level", default = "0.95", no_na)] conf_level: f64,
    #[miniextendr(formal = "optional.value")] optional: Missing<i32>,
    #[miniextendr(formal = "pick.mode", match_arg, default = "\"safe\"")] mode: FormalMode,
    #[miniextendr(formal = "literal.mode", choices("fast", "safe"))] literal: String,
) -> Vec<String> {
    vec![
        conf_int.to_string(),
        conf_level.to_string(),
        match optional {
            Missing::Absent => "missing".into(),
            Missing::Present(x) => x.to_string(),
        },
        match mode {
            FormalMode::Fast => "fast",
            FormalMode::Safe => "safe",
        }
        .into(),
        literal,
    ]
}

/// Bypass R checks to exercise native conversion diagnostics.
/// @param exact.value An integer.
#[miniextendr(noexport, no_preconditions)]
pub fn formal_native(#[miniextendr(formal = "exact.value")] value: i32) -> i32 {
    value
}

/// Coercion options still follow the Rust binding.
/// @param numeric.value A coercible number.
#[miniextendr(noexport)]
pub fn formal_coerce(#[miniextendr(formal = "numeric.value", coerce)] value: f64) -> f64 {
    value
}

/// Dotted callees cannot be shadowed by argument values.
/// @param is.character Ignored function.
/// @param parent.frame Ignored function.
/// @param .Call Ignored function.
/// @param quoted.expr An expression to capture.
#[miniextendr(noexport)]
pub fn formal_shadow(
    #[miniextendr(formal = "is.character")] character: SEXP,
    #[miniextendr(formal = "parent.frame")] frame: SEXP,
    #[miniextendr(formal = ".Call")] call: SEXP,
    #[miniextendr(formal = "quoted.expr")] expr: Quoted,
) -> SEXP {
    let _ = (character, frame, call);
    expr.expr()
}

/// Choice names differing only by punctuation need distinct registration symbols.
/// @param conf.int First choice.
/// @param conf_int Second choice.
#[miniextendr(noexport)]
pub fn formal_choice_symbols(
    #[miniextendr(formal = "conf.int", match_arg)] dotted: FormalMode,
    #[miniextendr(match_arg)] conf_int: FormalMode,
) -> bool {
    matches!(dotted, FormalMode::Fast) && matches!(conf_int, FormalMode::Safe)
}

/// Trait declaration formals survive impl binding changes and inherited defaults.
#[miniextendr]
pub trait FormalTrait {
    /// Add one to a named argument.
    /// @param trait.value Integer input.
    fn named(&self, #[miniextendr(formal = "trait.value", default = "5L")] value: i32) -> i32 {
        value + 1
    }
}
impl<T> FormalTrait for T {}

/// Generic traits use concrete shims with the same formal names.
#[miniextendr]
pub trait FormalGeneric<T> {
    /// Return a named argument.
    /// @param generic.value Input value.
    fn generic_named(&self, #[miniextendr(formal = "generic.value")] value: T) -> T {
        value
    }
}
impl<X, T> FormalGeneric<T> for X {}

macro_rules! formal_class {
    ($type:ident, $system:ident, $method:ident, $capture:ident) => {
        #[derive(ExternalPtr)]
        pub struct $type {
            start: i32,
        }

        /// Explicit formal regression fixture.
        #[miniextendr($system, internal)]
        impl $type {
            /// Construct a fixture.
            /// @param start.at Initial value.
            pub fn new(#[miniextendr(formal = "start.at", default = "1L")] start: i32) -> Self {
                Self { start }
            }

            /// Add to the initial value.
            /// @param add.by Increment.
            pub fn $method(
                &self,
                #[miniextendr(formal = "add.by", default = "2L")] increment: i32,
            ) -> i32 {
                self.start + increment
            }

            /// Capture an optional expression without evaluating its default.
            /// @param quoted.expr Optional expression.
            pub fn $capture(
                &self,
                #[miniextendr(formal = "quoted.expr", default = "stop('never')")] expr: Missing<
                    Quoted,
                >,
            ) -> SEXP {
                match expr {
                    Missing::Absent => SEXP::nil(),
                    Missing::Present(expr) => expr.expr(),
                }
            }

            /// Static methods share the argument naming policy.
            /// @param static.value Integer input.
            pub fn named_static(#[miniextendr(formal = "static.value")] value: i32) -> i32 {
                value
            }
        }
        #[miniextendr($system, internal)]
        impl FormalTrait for $type {}
        #[miniextendr($system, internal, blanket)]
        impl FormalGeneric<i32> for $type {}
    };
}
formal_class!(FormalEnv, env, fe_add, fe_capture);
formal_class!(FormalR6, r6, fr_add, fr_capture);
formal_class!(FormalS3, s3, fs3_add, fs3_capture);
formal_class!(FormalS4, s4, fs4_add, fs4_capture);
formal_class!(FormalS7, s7, fs7_add, fs7_capture);

/// vctrs constructor and static argument names.
pub struct FormalVctrs;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "fm"), internal)]
impl FormalVctrs {
    /// Construct the fixture.
    /// @param .data Numeric payload.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(#[miniextendr(formal = ".data")] values: Vec<f64>) -> Vec<f64> {
        values
    }

    /// Return the static argument.
    /// @param na.rm Whether to omit missing values.
    pub fn named_static(#[miniextendr(formal = "na.rm", default = "FALSE")] flag: bool) -> bool {
        flag
    }
}

/// A dotted formal may share a name with its own type predicate.
/// @param is.character Character input.
#[miniextendr(noexport)]
pub fn formal_shadow_predicate(#[miniextendr(formal = "is.character")] value: String) -> String {
    value
}

/// A standalone S3 method keeps explicit formals.
#[miniextendr(s3(generic = "format", class = "formal_result"))]
pub fn formal_format(
    x: SEXP,
    #[miniextendr(formal = "conf.int", default = "FALSE")] conf_int: bool,
    _dots: &miniextendr_api::dots::Dots,
) -> String {
    let _ = x;
    conf_int.to_string()
}

#[derive(Default, ExternalPtr)]
pub struct FormalProperties {
    value: i32,
}

/// R6 active setters receive the assigned value by position.
#[miniextendr(r6, internal)]
impl FormalProperties {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    #[miniextendr(r6(active))]
    pub fn reading(&self) -> i32 {
        self.value
    }

    #[miniextendr(r6(setter, prop = "reading"))]
    pub fn set_reading(&mut self, #[miniextendr(formal = "new.value")] input: i32) {
        self.value = input;
    }
}

#[derive(Default, ExternalPtr)]
pub struct FormalS7Properties {
    value: i32,
}

/// S7's fixed property adapters forward the assigned value by position.
#[miniextendr(s7, internal)]
impl FormalS7Properties {
    /// Dispatch on the resolved R parameter name.
    /// @param other.value The other integer.
    #[miniextendr(s7(dispatch = "x, other.value"))]
    pub fn formal_dispatch(&self, #[miniextendr(formal = "other.value")] other: i32) -> i32 {
        self.value + other
    }

    pub fn new() -> Self {
        Self { value: 0 }
    }

    #[miniextendr(s7(getter))]
    pub fn reading(&self) -> i32 {
        self.value
    }

    #[miniextendr(s7(setter, prop = "reading"))]
    pub fn set_reading(&mut self, #[miniextendr(formal = "new.value")] input: i32) {
        self.value = input;
    }
}
