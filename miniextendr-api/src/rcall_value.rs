//! `RCallValue`: an R call as an owned, `Send` value.
//!
//! [`RValue`] holds R data; `RCallValue` holds a call, such as
//! `x$table <- value`: its function and its arguments, each a symbol, another
//! call or an [`RValue`] constant. Like `RValue`, it holds no `SEXP`, so it can
//! travel in a condition built anywhere (a `panic_any` payload, a `defer_*`
//! sent from another thread) and becomes an R call on R's main thread.
//!
//! Its use is the call a condition reports ([`ConditionCall::Given`]): a
//! method names the call the user wrote, not the one the transport captured.
//!
//! ```
//! use miniextendr_api::{RCallPart, RCallValue};
//!
//! // `x$table <- value`, which R parses as `` `<-`(x$table, value) ``.
//! let call = RCallValue::new("<-")
//!     .arg(
//!         RCallValue::new("$")
//!             .arg(RCallPart::symbol("x"))
//!             .arg(RCallPart::symbol("table")),
//!     )
//!     .arg(RCallPart::symbol("value"));
//! assert_eq!(call.args().len(), 2);
//! ```
//!
//! [`RCallValue`] is the `Send` counterpart of [`RCall`](crate::expression::RCall),
//! the builder of a call to evaluate, which holds `SEXP`s.
//!
//! [`ConditionCall::Given`]: crate::condition::ConditionCall::Given

use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;
use crate::sexp_ext::PairListExt;
use crate::sexp_types::SEXPTYPE;
use crate::{RValue, SEXP, SexpExt};

/// R refuses a symbol or argument name longer than this many bytes
/// (`MAXIDSIZE` in R's `Defn.h`).
const MAX_NAME_BYTES: usize = 10_000;

/// An R call as an owned, `Send` value: a function and its arguments. See the
/// [module docs](self).
///
/// It always becomes an R call (`is.call()` is `TRUE`), never a symbol or a
/// constant on its own.
#[derive(Debug, Clone)]
pub struct RCallValue {
    function: RCallPart,
    args: Vec<(Option<String>, RCallPart)>,
}

/// The function or one argument of an [`RCallValue`].
///
/// A value that converts into an [`RValue`] converts into a constant, so a
/// string is the string `"x"`, as in R's `call("f", "x")`. A name is a
/// [`symbol`](Self::symbol).
#[derive(Debug, Clone)]
pub enum RCallPart {
    /// A symbol (`quote(x)`). The empty name is R's empty argument, the
    /// second slot of `x[1, ]`.
    Symbol(String),
    /// A call.
    Call(Box<RCallValue>),
    /// A constant.
    Value(RValue),
}

impl RCallPart {
    /// The symbol `name`. The empty name is R's empty argument.
    pub fn symbol(name: impl Into<String>) -> Self {
        RCallPart::Symbol(name.into())
    }
}

impl From<RCallValue> for RCallPart {
    fn from(call: RCallValue) -> Self {
        RCallPart::Call(Box::new(call))
    }
}

impl<T: Into<RValue>> From<T> for RCallPart {
    fn from(value: T) -> Self {
        RCallPart::Value(value.into())
    }
}

impl RCallValue {
    /// A call of the function named `function`, with no arguments yet.
    pub fn new(function: impl Into<String>) -> Self {
        RCallValue::with_function(RCallPart::Symbol(function.into()))
    }

    /// A call of `function`, which can itself be a call (`pkg::f` is
    /// `` `::`(pkg, f) ``), with no arguments yet.
    pub fn with_function(function: RCallPart) -> Self {
        RCallValue {
            function,
            args: Vec::new(),
        }
    }

    /// Add an unnamed argument.
    pub fn arg(mut self, value: impl Into<RCallPart>) -> Self {
        self.args.push((None, value.into()));
        self
    }

    /// Add the argument `name = value`. An empty name adds an unnamed one.
    pub fn named_arg(mut self, name: impl Into<String>, value: impl Into<RCallPart>) -> Self {
        let name: String = name.into();
        self.args
            .push(((!name.is_empty()).then_some(name), value.into()));
        self
    }

    /// The function.
    pub fn function(&self) -> &RCallPart {
        &self.function
    }

    /// The arguments, each with its name or `None`.
    pub fn args(&self) -> &[(Option<String>, RCallPart)] {
        &self.args
    }

    /// Why R can't build this call, or `None` when it can: a symbol or
    /// argument name longer than R's 10000 bytes, or one holding a NUL.
    /// Checked before anything is allocated, since R would raise an error
    /// while building the call.
    pub fn unbuildable(&self) -> Option<String> {
        let name_problem = |name: &str| {
            if name.len() > MAX_NAME_BYTES {
                Some(format!(
                    "a name of {} bytes is longer than R's {MAX_NAME_BYTES}",
                    name.len()
                ))
            } else if name.contains('\0') {
                Some(format!("the name {name:?} holds a NUL"))
            } else {
                None
            }
        };
        let part_problem = |part: &RCallPart| match part {
            RCallPart::Symbol(name) => name_problem(name),
            RCallPart::Call(call) => call.unbuildable(),
            RCallPart::Value(_) => None,
        };
        part_problem(&self.function).or_else(|| {
            self.args.iter().find_map(|(name, value)| {
                name.as_deref()
                    .and_then(name_problem)
                    .or_else(|| part_problem(value))
            })
        })
    }

    /// The call, unprotected.
    ///
    /// # Safety
    ///
    /// R's main thread, and [`unbuildable`](Self::unbuildable) is `None`.
    pub(crate) unsafe fn build(self) -> SEXP {
        unsafe {
            // Each part is allocated here, not rooted anywhere yet: protect it
            // before the next allocation, and the tail as it grows.
            let scope = crate::ProtectScope::new();
            let function = scope.protect_raw(self.function.build());
            let mut tail = SEXP::nil();
            for (name, value) in self.args.into_iter().rev() {
                let value = scope.protect_raw(value.build());
                tail = scope.protect_raw(value.cons(tail));
                if let Some(name) = name {
                    tail.set_tag(install(&name));
                }
            }
            function.lcons(tail)
        }
    }
}

impl RCallPart {
    /// This part, unprotected.
    ///
    /// # Safety
    ///
    /// As [`RCallValue::build`].
    unsafe fn build(self) -> SEXP {
        match self {
            RCallPart::Symbol(name) if name.is_empty() => SEXP::missing_arg(),
            RCallPart::Symbol(name) => unsafe { install(&name) },
            RCallPart::Call(call) => unsafe { call.build() },
            RCallPart::Value(value) => value.into_sexp(),
        }
    }
}

/// The symbol `name`: not empty, at most [`MAX_NAME_BYTES`] and without a NUL,
/// as [`RCallValue::unbuildable`] checked.
///
/// # Safety
///
/// R's main thread.
unsafe fn install(name: &str) -> SEXP {
    let name = std::ffi::CString::new(name).expect("unbuildable() refuses a NUL");
    unsafe { crate::sys::Rf_install(name.as_ptr()) }
}

// region: IntoR

/// The R call. A name R refuses ([`RCallValue::unbuildable`]) is an error
/// with that reason.
impl IntoR for RCallValue {
    type Error = crate::condition::RError;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        match self.unbuildable() {
            Some(problem) => Err(crate::condition::RError::new(format!(
                "cannot build the call: {problem}"
            ))),
            // SAFETY: `IntoR` runs on R's main thread; the names are checked.
            None => Ok(unsafe { self.build() }),
        }
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
}

// endregion

// region: TryFromSexp

/// Reads a call (`LANGSXP`). Its function and arguments are symbols, calls or
/// values [`RValue`] reads; anything else (a function object, as `do.call()`
/// puts in a call, or an environment) is an error.
impl TryFromSexp for RCallValue {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() != SEXPTYPE::LANGSXP {
            return Err(SexpError::Type(SexpTypeError {
                expected: SEXPTYPE::LANGSXP,
                actual: sexp.type_of(),
            }));
        }
        let function = RCallPart::try_from_sexp(sexp.car())?;
        let mut args = Vec::new();
        let mut node = sexp.cdr();
        while node.type_of() == SEXPTYPE::LISTSXP {
            let tag = node.tag();
            let name = if tag.is_symbol() {
                // SAFETY: a symbol's print name is a CHARSXP.
                let name = unsafe { crate::from_r::try_charsxp_to_str(tag.printname()) }?;
                (!name.is_empty()).then(|| name.to_owned())
            } else {
                None
            };
            args.push((name, RCallPart::try_from_sexp(node.car())?));
            node = node.cdr();
        }
        Ok(RCallValue { function, args })
    }
}

/// Reads a symbol, a call or a value [`RValue`] reads.
impl TryFromSexp for RCallPart {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp.type_of() {
            // R's empty argument is the symbol with the empty name.
            SEXPTYPE::SYMSXP => {
                // SAFETY: a symbol's print name is a CHARSXP.
                let name = unsafe { crate::from_r::try_charsxp_to_str(sexp.printname()) }?;
                Ok(RCallPart::Symbol(name.to_owned()))
            }
            SEXPTYPE::LANGSXP => Ok(RCallPart::Call(Box::new(RCallValue::try_from_sexp(sexp)?))),
            _ => Ok(RCallPart::Value(RValue::try_from_sexp(sexp)?)),
        }
    }
}

// endregion

#[cfg(test)]
mod tests;
