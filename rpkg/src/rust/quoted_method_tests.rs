//! Unevaluated arguments on inherent and trait methods (#1839).
use miniextendr_api::{ExternalPtr, IntoR, Missing, Quosure, Quoted, SEXP, miniextendr};

fn quoted_part(expr: Quoted, mode: i32) -> SEXP {
    match mode {
        0 => expr.expr(),
        1 => expr.env(),
        2 => expr.eval(),
        _ => panic!("invalid fixture mode"),
    }
}

fn quosure_part(expr: Quosure, mode: i32) -> SEXP {
    match mode {
        0 => expr.expr(),
        1 => expr.env(),
        2 => expr.eval_tidy(SEXP::nil()),
        _ => panic!("invalid fixture mode"),
    }
}

/// Trait methods capture both marker kinds without forcing either.
#[miniextendr]
pub trait MethodCapture {
    /// Read or evaluate one capture; the other must stay unevaluated.
    /// @param expr A base R expression.
    /// @param tidy A tidy expression.
    /// @param mode Zero through five select expression, environment or value.
    fn capture(&self, expr: Quoted, tidy: Quosure, mode: i32) -> SEXP {
        if mode < 3 {
            quoted_part(expr, mode)
        } else {
            quosure_part(tidy, mode - 3)
        }
    }

    /// Preserve omitted arguments through trait transport.
    /// @param expr An optional base R expression.
    /// @param tidy An optional tidy expression.
    fn absent(&self, expr: Missing<Quoted>, tidy: Missing<Quosure>) -> Vec<bool> {
        vec![
            matches!(expr, Missing::Absent),
            matches!(tidy, Missing::Absent),
        ]
    }
}
impl<T> MethodCapture for T {}

macro_rules! capture_class {
    ($type:ident, $system:ident, $quoted:ident, $quosure:ident, $optional:ident) => {
        #[derive(ExternalPtr)]
        pub struct $type {
            value: i32,
        }

        /// Class-method capture regression fixture.
        #[miniextendr($system, internal)]
        impl $type {
            /// Evaluate an optional constructor expression in its caller.
            /// @param expr An optional integer expression.
            pub fn new(expr: Missing<Quoted>) -> Self {
                let value = match expr {
                    Missing::Absent => 0,
                    Missing::Present(expr) => {
                        <i32 as miniextendr_api::TryFromSexp>::try_from_sexp(expr.eval())
                            .expect("integer constructor expression")
                    }
                };
                Self { value }
            }

            /// Read a base R capture.
            /// @param expr An expression, never forced by the wrapper.
            /// @param mode Zero, one or two select expression, environment or value.
            pub fn $quoted(&self, expr: Quoted, mode: i32) -> SEXP {
                if mode == 3 {
                    self.value.into_sexp()
                } else {
                    quoted_part(expr, mode)
                }
            }

            /// Read a tidy capture.
            /// @param expr A tidy expression, never forced by the wrapper.
            /// @param mode Zero, one or two select expression, environment or value.
            pub fn $quosure(&self, expr: Quosure, mode: i32) -> SEXP {
                quosure_part(expr, mode)
            }

            /// Defaults describe formals and are never evaluated.
            /// @param expr An optional expression.
            /// @param tidy An optional tidy expression.
            #[miniextendr(defaults(
                expr = "stop('unused default')",
                tidy = "stop('unused tidy default')"
            ))]
            pub fn $optional(&self, expr: Missing<Quoted>, tidy: Missing<Quosure>) -> Vec<bool> {
                vec![
                    matches!(expr, Missing::Absent),
                    matches!(tidy, Missing::Absent),
                ]
            }

            /// Static methods keep the same capture contract.
            /// @param expr A base R expression.
            pub fn static_capture(expr: Quoted) -> SEXP {
                expr.expr()
            }
        }

        #[miniextendr($system, internal)]
        impl MethodCapture for $type {}
    };
}

capture_class!(QuotedEnv, env, qe_quoted, qe_quosure, qe_optional);
capture_class!(QuotedR6, r6, qr_quoted, qr_quosure, qr_optional);
capture_class!(QuotedS3, s3, qs3_quoted, qs3_quosure, qs3_optional);
capture_class!(QuotedS4, s4, qs4_quoted, qs4_quosure, qs4_optional);
capture_class!(QuotedS7, s7, qs7_quoted, qs7_quosure, qs7_optional);

/// vctrs exposes static methods rather than pointer-backed receivers.
pub struct QuotedVctrs;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "qv"), internal)]
impl QuotedVctrs {
    /// Construct the fixture.
    /// @param values Numeric payload.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(values: Vec<f64>) -> Vec<f64> {
        values
    }

    /// Read a static base R capture.
    /// @param expr A base R expression.
    /// @param mode Zero, one or two select expression, environment or value.
    pub fn quoted(expr: Quoted, mode: i32) -> SEXP {
        quoted_part(expr, mode)
    }

    /// Read a static tidy capture.
    /// @param expr A tidy expression.
    /// @param mode Zero, one or two select expression, environment or value.
    pub fn quosure(expr: Quosure, mode: i32) -> SEXP {
        quosure_part(expr, mode)
    }
}

/// Forward both captures through a trait View, retaining their roots.
/// @param obj A capture fixture.
/// @param expr A base R expression.
/// @param tidy A tidy expression.
/// @param mode Zero through five select expression, environment or value.
#[miniextendr(noexport)]
pub fn quoted_method_view(obj: SEXP, expr: Quoted, tidy: Quosure, mode: i32) -> SEXP {
    let view = unsafe { MethodCaptureView::from_sexp(obj) };
    view.capture(expr, tidy, mode)
}

/// Forward optional captures through the same trait View.
/// @param obj A capture fixture.
/// @param expr An optional base R expression.
/// @param tidy An optional tidy expression.
#[miniextendr(noexport)]
pub fn quoted_method_view_absent(
    obj: SEXP,
    expr: Missing<Quoted>,
    tidy: Missing<Quosure>,
) -> Vec<bool> {
    let view = unsafe { MethodCaptureView::from_sexp(obj) };
    view.absent(expr, tidy)
}

/// Construct a trait-ABI erased owner for View transport tests.
#[miniextendr(noexport)]
pub fn quoted_method_owner() -> SEXP {
    unsafe {
        miniextendr_api::trait_abi::ccall::mx_wrap(__mx_wrap_QuotedEnv(QuotedEnv::new(
            Missing::Absent,
        )))
    }
}
