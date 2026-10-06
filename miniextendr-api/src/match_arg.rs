//! `match.arg`-style enum conversion for R string arguments.
//!
//! This module provides the [`MatchArg`] trait for converting between Rust
//! fieldless enums and R character strings with `match.arg` semantics
//! (exact match or unique partial matching).
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::MatchArg;
//!
//! #[derive(Copy, Clone, MatchArg)]
//! #[match_arg(rename_all = "snake_case")]
//! enum Mode {
//!     Fast,
//!     Safe,
//!     Debug,
//! }
//!
//! #[miniextendr]
//! fn run(#[miniextendr(match_arg)] mode: Mode) -> String {
//!     format!("{mode:?}")
//! }
//! ```
//!
//! The generated R wrapper uses `base::match.arg()` for validation before
//! the main `.Call()`, giving users familiar R error messages and partial
//! matching.
//!
//! An `Option<T>` parameter (`#[miniextendr(match_arg)] mode: Option<Mode>`)
//! is the optional form: the R formal defaults to `NULL`, `NULL` converts to
//! `None`, and any other value is matched as usual. The generated C wrapper
//! decodes it with [`match_arg_option_from_sexp`] (there is no
//! `TryFromSexp for Option<T>` blanket: that slot belongs to the newtype
//! forwarding impls in [`crate::newtype`], and a downstream crate cannot add
//! one for its own enum under the orphan rule), so a hand-written `MatchArg`
//! impl needs nothing extra.
//!
//! A `Missing<..>` around the parameter type (`Missing<T>`,
//! `Missing<Option<T>>`, `Missing<Vec<T>>` for `several_ok`) keeps the choice
//! vector as the R formal and reports an omitted argument as
//! `Missing::Absent` (#1551); the C wrapper decodes it with
//! [`match_arg_missing_or`] around the decoder of the inner type.
//!
//! With the `either` feature, `Either<T, R>` takes a choice or a value of
//! another kind: character or factor input is matched and becomes `Left(T)`,
//! anything else converts to `R` (`match_arg_either_or`). A `several_ok` list
//! takes the same split as `Either<Vec<T>, R>` or `Either<Box<[T]>, R>`
//! (#1612): character or factor input is matched element by element and
//! becomes `Left(Vec<T>)`, anything else converts to `R`.
//!
//! `several_ok` parameters are validated strictly on the R side: every element
//! has to match a choice, and `NULL` selects every choice (the same fallback
//! [`match_arg_vec_from_sexp`] applies). Under `Either`, `NULL` is not a
//! choice: it goes to `R` (or is `None` under `Option<Either<..>>`).

use crate::condition::ArgError;
use crate::from_r::{SexpError, TryFromSexp, charsxp_to_str};
use crate::gc_protect::ProtectScope;
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

/// Trait for enum types that support `match.arg`-style string conversion.
///
/// Implementors provide a fixed set of choice strings and bidirectional
/// conversion between enum variants and their string representations.
///
/// Use `#[derive(MatchArg)]` to auto-generate this implementation.
pub trait MatchArg: Sized + Copy + 'static {
    /// The canonical choice strings, in variant declaration order.
    ///
    /// The first choice is the default when the R argument is `NULL`.
    const CHOICES: &'static [&'static str];

    /// Convert a choice string to the corresponding enum variant.
    ///
    /// Returns `None` if the string doesn't match any choice exactly.
    fn from_choice(choice: &str) -> Option<Self>;

    /// Convert the enum variant to its canonical choice string.
    fn to_choice(self) -> &'static str;
}

/// The words after `'<param>' ` in a `match_arg` argument error, shared by
/// the wrappers preamble's `.miniextendr_match_arg` /
/// `.miniextendr_match_arg_several` ([`crate::registry::ARG_CHECK_HELPERS`],
/// built with `concat!`) and [`MatchArgError::arg_error`], so R and Rust word
/// these errors from one source (#1741). The generated conversion of a
/// `match_arg` type words its error with `arg_error` too (#1767). It expands
/// to string literals.
macro_rules! match_arg_wording {
    (not_character) => {
        "must be NULL or a character vector"
    };
    (not_scalar) => {
        "must be of length 1"
    };
    (not_a_choice) => {
        "should be one of"
    };
}
pub(crate) use match_arg_wording;

/// Error type for `MatchArg` conversion failures.
///
/// Every variant carries the type's choices, so an error can say what the
/// value must be (`'mode' should be one of "fast", "slow"`) whichever way it
/// was wrong.
#[derive(Debug, Clone)]
pub enum MatchArgError {
    /// The SEXP was not a character or factor type.
    InvalidType {
        /// The SEXPTYPE that was given.
        actual: SEXPTYPE,
        /// The valid choices.
        choices: &'static [&'static str],
    },
    /// The input had length != 1.
    InvalidLength {
        /// The length that was given.
        actual: usize,
        /// The valid choices.
        choices: &'static [&'static str],
    },
    /// The input was NA.
    IsNa {
        /// The valid choices.
        choices: &'static [&'static str],
    },
    /// No choice matched the input.
    NoMatch {
        /// The input string that didn't match.
        input: String,
        /// The valid choices.
        choices: &'static [&'static str],
    },
}

impl MatchArgError {
    /// The choices the value should have been one of.
    pub fn choices(&self) -> &'static [&'static str] {
        match self {
            MatchArgError::InvalidType { choices, .. }
            | MatchArgError::InvalidLength { choices, .. }
            | MatchArgError::IsNa { choices }
            | MatchArgError::NoMatch { choices, .. } => choices,
        }
    }

    /// What the value must be, in R terms: `one of "fast", "slow"`. An
    /// `Either` whose arms both refused a value names a `match_arg` arm with
    /// it (`'x' must be one of "oral", "bolus", or a single double: got
    /// logical`, #1594).
    pub fn expectation(&self) -> String {
        one_of(self.choices())
    }

    /// The argument error the generated wrapper's `match_arg` check
    /// (`.miniextendr_match_arg`) raises for this value of parameter `param`
    /// (its R name), word for word (#1741):
    ///
    /// - [`InvalidType`](Self::InvalidType): `'mode' must be NULL or a character vector`
    /// - [`InvalidLength`](Self::InvalidLength): `'mode' must be of length 1`
    /// - [`IsNa`](Self::IsNa), [`NoMatch`](Self::NoMatch): `'mode' should be
    ///   one of "fast", "safe"`
    ///
    /// The choices are quoted as R's `dQuote(choices, FALSE)` quotes them,
    /// without escapes. [`match_arg_param`] returns it; [`ArgError::raise`]
    /// raises it as that condition. The generated conversion of a `match_arg`
    /// type, with or without `#[miniextendr(match_arg)]`, raises its message
    /// too (#1767).
    pub fn arg_error(&self, param: &str) -> ArgError {
        ArgError::new(param, self.arg_message(param, self.choices()))
    }

    /// The message of [`arg_error`](Self::arg_error), with the choices listed
    /// in the order of `formal`, the choices of the parameter's R formal:
    /// `T::CHOICES`, or those of a `default = "..."` parameter, which lists
    /// its default first ([`match_arg_param_with_default`]).
    pub(crate) fn arg_message(&self, param: &str, formal: &[&str]) -> String {
        let what = match self {
            MatchArgError::InvalidType { .. } => match_arg_wording!(not_character).to_string(),
            MatchArgError::InvalidLength { .. } => match_arg_wording!(not_scalar).to_string(),
            MatchArgError::IsNa { .. } | MatchArgError::NoMatch { .. } => {
                let quoted: Vec<String> = formal.iter().map(|c| format!("\"{c}\"")).collect();
                format!("{} {}", match_arg_wording!(not_a_choice), quoted.join(", "))
            }
        };
        format!("'{param}' {what}")
    }

    /// The reason of this error in R terms, after `<expected>: ` when an
    /// `Either` names this arm in an argument error (#1591): `got "zzz"`,
    /// `got numeric`, `got length 2`, `NA is not allowed`. With
    /// `expected_known = false` (nothing before the reason names the choices)
    /// it is the `Display` text, which says what was expected.
    pub(crate) fn r_reason(&self, expected_known: bool) -> String {
        if !expected_known {
            return self.to_string();
        }
        match self {
            MatchArgError::InvalidType { actual, .. } => {
                format!("got {}", crate::typed_list::sexptype_name(*actual))
            }
            MatchArgError::InvalidLength { actual, .. } => format!("got length {actual}"),
            MatchArgError::IsNa { .. } => "NA is not allowed".to_string(),
            MatchArgError::NoMatch { input, .. } => format!("got {input:?}"),
        }
    }
}

/// `one of "fast", "slow"`: what a `match_arg` / `choices` value must be, in
/// the words of a conversion's argument error that names it beside other
/// accepted values (`'x' must be one of "oral", "bolus", or a single double:
/// got logical`, #1594).
pub fn one_of(choices: &[&str]) -> String {
    choice_expectation(choices, false, "")
}

/// `one of "fast", "slow"<suffix>`, or `one or more of "fast", "slow"<suffix>`
/// for a `several_ok` parameter (`several`), quoted as [`one_of`] quotes.
/// The generated argument error of a `match_arg` / `choices` parameter on
/// `Either<T, R>` passes the other accepted values as `suffix`
/// (`, or a data frame`), in the words of its `@param` line; an `Either`
/// whose arms both refused a value joins them with it too.
#[doc(hidden)]
pub fn choice_expectation(choices: &[&str], several: bool, suffix: &str) -> String {
    let quoted: Vec<String> = choices.iter().map(|c| format!("{c:?}")).collect();
    let lead = if several { "one or more of" } else { "one of" };
    format!("{lead} {}{suffix}", quoted.join(", "))
}

/// The text is complete on its own, for a conversion outside an argument:
/// `expected one of "fast", "slow", got "zzz"`, `expected a string or factor,
/// got numeric`.
impl std::fmt::Display for MatchArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let expected = self.expectation();
        match self {
            MatchArgError::InvalidType { actual, .. } => write!(
                f,
                "expected {expected} (a string or factor), got {}",
                crate::typed_list::sexptype_name(*actual)
            ),
            MatchArgError::InvalidLength { actual, .. } => {
                write!(f, "expected {expected}, got length {actual}")
            }
            MatchArgError::IsNa { .. } => write!(f, "expected {expected}, got NA"),
            MatchArgError::NoMatch { input, .. } => {
                write!(f, "expected {expected}, got {input:?}")
            }
        }
    }
}

impl std::error::Error for MatchArgError {}

/// Kept whole, so a `TryFromSexp` whose error is `SexpError` (the one
/// `#[derive(MatchArg)]` emits) still reports the choices.
impl From<MatchArgError> for crate::from_r::SexpError {
    fn from(e: MatchArgError) -> Self {
        crate::from_r::SexpError::MatchArg(e)
    }
}

/// Escape a Rust `&str` for embedding inside an R double-quoted string literal.
///
/// Handles `\`, `"`, newline, carriage return, and tab — the characters R
/// recognises as escape sequences inside `"..."`. Used when formatting
/// `MatchArg::CHOICES` into the default of a generated R wrapper formal, so
/// that a choice like `say "hi"` or `c:\path` cannot produce syntactically
/// invalid R code.
pub fn escape_r_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str(r"\\"),
            '"' => out.push_str(r#"\""#),
            '\n' => out.push_str(r"\n"),
            '\r' => out.push_str(r"\r"),
            '\t' => out.push_str(r"\t"),
            c => out.push(c),
        }
    }
    out
}

/// Build an R character vector (STRSXP) from the choices of a `MatchArg` type.
///
/// This is called by generated choices-helper C wrappers to provide the
/// choice list to `base::match.arg()` in the R wrapper.
pub fn choices_sexp<T: MatchArg>() -> SEXP {
    let choices = <T as MatchArg>::CHOICES;
    unsafe {
        let scope = ProtectScope::new();
        // Preserve the R_BlankString short-circuit: skips hash-lookup on the
        // interned empty CHARSXP for any empty choice strings.
        let vec = scope.alloc_strsxp(choices.len()).into_raw();
        for (i, s) in choices.iter().enumerate() {
            let charsxp = if s.is_empty() {
                SEXP::blank_string()
            } else {
                SEXP::charsxp(s)
            };
            vec.set_string_elt(i as R_xlen_t, charsxp);
        }
        vec
    }
}

/// Map a `SexpError` from a string `TryFromSexp` conversion into a `MatchArgError`.
///
/// Only `Type` and `Length` are produced by the `&str`/`Option<&str>`/
/// `Vec<Option<&str>>` conversions we delegate to — other variants are
/// unreachable in this context.
fn sexp_err_to_match_arg_err<T: MatchArg>(e: SexpError) -> MatchArgError {
    let choices = <T as MatchArg>::CHOICES;
    match e {
        SexpError::Type(t) => MatchArgError::InvalidType {
            actual: t.actual,
            choices,
        },
        SexpError::Length(l) => MatchArgError::InvalidLength {
            actual: l.actual,
            choices,
        },
        other => unreachable!("unexpected SexpError from string conversion: {other}"),
    }
}

/// The choices of a `match_arg` parameter's R formal: `choices` with the
/// default moved to the front and the others after it in their order
/// (`"Safe"` in `"Fast", "Safe", "Debug"` gives `"Safe", "Fast", "Debug"`),
/// or `None` when `is_default` holds for no choice.
///
/// `.miniextendr_match_arg()` takes the first element of the formal for
/// `NULL` and for the formal itself, so this order is what makes a
/// `default = "..."` the default, and the order its message lists. The
/// wrappers writer builds such a formal with it
/// ([`crate::registry::match_arg_formal`]) and
/// [`match_arg_param_with_default`] matches against it, so the two cannot
/// disagree (#1767).
pub(crate) fn default_first<'a>(
    choices: &[&'a str],
    is_default: impl Fn(&str) -> bool,
) -> Option<Vec<&'a str>> {
    let at = choices.iter().position(|choice| is_default(choice))?;
    let mut formal = Vec::with_capacity(choices.len());
    formal.push(choices[at]);
    formal.extend(
        choices
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != at)
            .map(|(_, choice)| *choice),
    );
    Some(formal)
}

/// The first choice of the formal: what `NULL` and the formal itself select.
fn first_choice<T: MatchArg>(formal: &[&str]) -> Result<T, MatchArgError> {
    formal
        .first()
        .and_then(|choice| T::from_choice(choice))
        .ok_or(MatchArgError::NoMatch {
            input: String::new(),
            choices: <T as MatchArg>::CHOICES,
        })
}

/// `NA` as a choice: `pmatch()`, and so the wrapper's check, matches it as
/// the string `"NA"`, so a choice that is `"NA"` or the only one starting
/// with it is selected. Anything else is [`MatchArgError::IsNa`].
fn match_na<T: MatchArg>() -> Result<T, MatchArgError> {
    match_choice::<T>("NA").map_err(|_| MatchArgError::IsNa {
        choices: <T as MatchArg>::CHOICES,
    })
}

/// Whether the `len` strings `label(i)` (`None` for `NA`) are `formal`, in
/// order: the formal default of a choice parameter, which an omitted argument
/// evaluates to.
fn is_formal<'a>(formal: &[&str], len: usize, label: impl Fn(isize) -> Option<&'a str>) -> bool {
    len == formal.len()
        && formal
            .iter()
            .zip(0..)
            .all(|(choice, i)| label(i) == Some(*choice))
}

/// The label of element `i` of a factor (`None` for a missing code or one
/// outside the levels).
fn factor_label(codes: SEXP, levels: SEXP, i: isize) -> Option<&'static str> {
    let code = codes.integer_elt(i);
    // R factor codes are 1-based; NA_integer_ is i32::MIN.
    let level = isize::try_from(code).ok()?.checked_sub(1)?;
    if level < 0 || level >= isize::try_from(levels.len()).ok()? {
        return None;
    }
    let charsxp = levels.string_elt(level);
    if charsxp.is_na_string() {
        return None;
    }
    // UTF-8 locale asserted at package init — charsxp_to_str is safe.
    Some(unsafe { charsxp_to_str(charsxp) })
}

/// A factor as a choice: read as its labels, like `as.character()`.
fn factor_to_choice<T: MatchArg>(sexp: SEXP, formal: &[&str]) -> Result<T, MatchArgError> {
    let levels = sexp.get_levels();
    let len = sexp.len();
    if is_formal(formal, len, |i| factor_label(sexp, levels, i)) {
        return first_choice::<T>(formal);
    }
    if len != 1 {
        return Err(MatchArgError::InvalidLength {
            actual: len,
            choices: <T as MatchArg>::CHOICES,
        });
    }
    match factor_label(sexp, levels, 0) {
        Some(label) => match_choice::<T>(label),
        None => match_na::<T>(),
    }
}

/// Match one choice argument with the semantics of the generated wrapper's
/// `match_arg` check (`.miniextendr_match_arg`), which are those of
/// `base::match.arg(arg, choices)`:
///
/// - `NULL` is the first choice;
/// - a factor is read as its labels;
/// - any other value that is not a character vector is
///   [`MatchArgError::InvalidType`];
/// - the full choices vector, `T::CHOICES` in order and without attributes
///   (what an omitted argument evaluates to when the choices are its formal
///   default), is the first choice;
/// - otherwise the value must have length 1 ([`MatchArgError::InvalidLength`])
///   and match a choice exactly or as a unique prefix
///   ([`MatchArgError::NoMatch`]). The empty string matches nothing, and `NA`
///   is matched as the string `"NA"`, as `pmatch()` does
///   ([`MatchArgError::IsNa`] when that matches nothing).
///
/// Used by the generated `TryFromSexp for T` implementation, and by the
/// generated C wrapper of a `#[miniextendr(match_arg)]` parameter, which only
/// sees the single choice the R wrapper already matched. A body matching a raw
/// `SEXP` argument wants [`match_arg_param`], which words its errors as the
/// wrapper does.
pub fn match_arg_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<T, MatchArgError> {
    match_formal::<T>(sexp, <T as MatchArg>::CHOICES)
}

/// [`match_arg_from_sexp`] for a parameter whose R formal is `formal`, a
/// permutation of `T::CHOICES`: its first element is what `NULL` and the
/// formal itself select. Matching a string does not depend on the order.
fn match_formal<T: MatchArg>(sexp: SEXP, formal: &[&str]) -> Result<T, MatchArgError> {
    let choices = <T as MatchArg>::CHOICES;
    let sexptype = sexp.type_of();
    if sexptype == SEXPTYPE::NILSXP {
        return first_choice::<T>(formal);
    }
    if sexp.is_factor() {
        return factor_to_choice::<T>(sexp, formal);
    }
    if sexptype != SEXPTYPE::STRSXP {
        return Err(MatchArgError::InvalidType {
            actual: sexptype,
            choices,
        });
    }
    let len = sexp.len();
    // `identical(arg, choices)` compares attributes too: a named vector of
    // the choices is not the formal default.
    let label = |i: isize| {
        let charsxp = sexp.string_elt(i);
        // UTF-8 locale asserted at package init — charsxp_to_str is safe.
        (!charsxp.is_na_string()).then(|| unsafe { charsxp_to_str(charsxp) })
    };
    if is_formal(formal, len, label) && !sexp.has_attributes() {
        return first_choice::<T>(formal);
    }
    if len != 1 {
        return Err(MatchArgError::InvalidLength {
            actual: len,
            choices,
        });
    }
    match label(0) {
        Some(input) => match_choice::<T>(input),
        None => match_na::<T>(),
    }
}

/// Match a choice argument a body received as a raw `SEXP` exactly as the
/// generated wrapper matches a `#[miniextendr(match_arg)]` parameter, and
/// with the same error.
///
/// The result is [`match_arg_from_sexp`]'s (`NULL` and the full choices
/// vector select the first choice, a factor is read as its labels, one string
/// matches exactly or as a unique prefix). The error is the argument error
/// the wrapper raises for parameter `param` (its R name), word for word:
///
/// - `'mode' must be NULL or a character vector`
/// - `'mode' must be of length 1`
/// - `'mode' should be one of "fast", "safe"`
///
/// [`ArgError::raise`] raises it as that condition (`kind = "conversion"`,
/// the crate's `conversion_error_class`, `e$param`; see
/// [`crate::arg_error!`]):
///
/// ```ignore
/// use miniextendr_api::{SEXP, match_arg_param, miniextendr};
///
/// #[miniextendr]
/// pub fn run(data: SEXP, mode: SEXP) -> String {
///     if is_data_frame(data) {
///         // Checked on this path only.
///         let mode: Mode = match_arg_param(mode, "mode").unwrap_or_else(|e| e.raise());
///         return format!("{mode:?}");
///     }
///     String::new()
/// }
/// ```
///
/// The choices are `T::CHOICES` in declaration order, the formal of a
/// `#[miniextendr(match_arg)]` parameter. A value forwarded from a parameter
/// with `default = "..."` wants [`match_arg_param_with_default`].
pub fn match_arg_param<T: MatchArg>(sexp: SEXP, param: &str) -> Result<T, ArgError> {
    match_arg_from_sexp::<T>(sexp).map_err(|e| e.arg_error(param))
}

/// [`match_arg_param`] for a choice argument whose R formal lists `default`
/// first: the formal of a `#[miniextendr(match_arg, default = "...")]`
/// parameter, which moves its default to the front and keeps the other
/// choices in their order (#1767).
///
/// The formal's order decides three things, and here they follow the formal
/// rather than `T::CHOICES`: the choice `NULL` selects, the vector that counts
/// as the formal itself (what an omitted argument evaluates to), and the
/// order of the choices in the message. For
/// `#[miniextendr(match_arg, default = "\"Safe\"")] mode: Mode` with
/// `Mode::CHOICES` `"Fast", "Safe", "Debug"`, the formal is
/// `c("Safe", "Fast", "Debug")`:
///
/// | Input | `match_arg_param` | `match_arg_param_with_default(.., Mode::Safe)` |
/// |---|---|---|
/// | `NULL` | `Fast` | `Safe` |
/// | `c("Safe", "Fast", "Debug")` | `'mode' must be of length 1` | `Safe` |
/// | `c("Fast", "Safe", "Debug")` | `Fast` | `'mode' must be of length 1` |
/// | `"zzz"` | `'mode' should be one of "Fast", "Safe", "Debug"` | `'mode' should be one of "Safe", "Fast", "Debug"` |
///
/// Matching a string is the same either way. The wrappers writer orders the
/// formal with the same code, so the two agree on every input.
///
/// `default` is a `T`, not its string or its position: it is always one of
/// the choices, a misspelled default does not compile, and a renamed or
/// reordered choice moves with it.
///
/// ```ignore
/// #[miniextendr]
/// pub fn run(mode: SEXP) -> String {
///     let mode: Mode = match_arg_param_with_default(mode, "mode", Mode::Safe)
///         .unwrap_or_else(|e| e.raise());
///     format!("{mode:?}")
/// }
/// ```
pub fn match_arg_param_with_default<T: MatchArg>(
    sexp: SEXP,
    param: &str,
    default: T,
) -> Result<T, ArgError> {
    let default = default.to_choice();
    let formal = default_first(<T as MatchArg>::CHOICES, |choice| choice == default)
        .expect("MatchArg::to_choice returns one of MatchArg::CHOICES");
    match_formal::<T>(sexp, &formal)
        .map_err(|e| ArgError::new(param, e.arg_message(param, &formal)))
}

/// Optional form of [`match_arg_from_sexp`] for `Option<T>` parameters (#1473).
///
/// `NULL` is `None` (the R formal of an `Option<T>` choice parameter defaults
/// to `NULL`, meaning no choice was made); anything else goes through the same
/// exact-or-partial matching as the plain type and becomes `Some`.
///
/// Used by the generated C wrapper for `#[miniextendr(match_arg)] x: Option<T>`
/// parameters instead of `TryFromSexp`, because `Option<T>` cannot carry a
/// `TryFromSexp` impl for a downstream enum (orphan rule) and a `T: MatchArg`
/// blanket would collide with the newtype blanket in [`crate::newtype`].
pub fn match_arg_option_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<Option<T>, MatchArgError> {
    match_arg_null_or(sexp, match_arg_from_sexp)
}

/// The `Option<_>` layer of a choice parameter: `NULL` is `None`, anything
/// else is decoded by `inner` and becomes `Some`.
///
/// Generated C wrappers compose this with the other layer helpers
/// ([`match_arg_missing_or`]) around the decoder of the choice itself;
/// [`match_arg_option_from_sexp`] is the scalar case.
pub fn match_arg_null_or<U, E>(
    sexp: SEXP,
    inner: impl FnOnce(SEXP) -> Result<U, E>,
) -> Result<Option<U>, E> {
    if sexp.type_of() == SEXPTYPE::NILSXP {
        Ok(None)
    } else {
        inner(sexp).map(Some)
    }
}

/// The `Missing<_>` layer of a choice parameter (#1551): the missing-argument
/// sentinel is [`Missing::Absent`](crate::Missing::Absent), anything else
/// (`NULL` included) is decoded by `inner` and becomes `Missing::Present`.
///
/// The generated R wrapper keeps the choice vector as the formal default,
/// skips the `match.arg()` check for an omitted argument and forwards R's
/// missing-argument sentinel (`if (missing(x)) quote(expr=) else x`), so
/// `#[miniextendr(match_arg)] mode: Missing<Option<Mode>>` reaches Rust as
/// `Absent` when omitted, `Present(None)` for `NULL`, and
/// `Present(Some(mode))` for a matched choice. The C wrapper decodes it with
/// `match_arg_missing_or(sexp, match_arg_option_from_sexp::<Mode>)`.
pub fn match_arg_missing_or<U, E>(
    sexp: SEXP,
    inner: impl FnOnce(SEXP) -> Result<U, E>,
) -> Result<crate::Missing<U>, E> {
    if crate::missing::is_missing_arg(sexp) {
        Ok(crate::Missing::Absent)
    } else {
        inner(sexp).map(crate::Missing::Present)
    }
}

/// The `Either<_, R>` layer of a choice parameter: a choice or a value of
/// another kind. Character and factor input (the forms `match.arg()` reads)
/// is decoded by `left` and becomes `Left`; anything else, `NULL` included, is
/// converted to `R` and becomes `Right`.
///
/// The generated R wrapper applies the same split: its prelude matches the
/// argument against the choices only when it is character or factor, so a
/// misspelled choice fails there and never reaches `R`, and a data frame is
/// never tried as a choice. For `#[miniextendr(match_arg)] route:
/// Either<Route, DataFrame>` the C wrapper decodes with
/// `match_arg_either_or::<_, DataFrame, _>(sexp, match_arg_from_sexp::<Route>)`;
/// for a `several_ok` list, `Either<Vec<Route>, DataFrame>` (#1612), `left` is
/// [`match_arg_vec_from_sexp::<Route>`](match_arg_vec_from_sexp); a
/// `choices(...)` parameter on `Either<String, R>` (or `Either<Vec<String>, R>`
/// with `several_ok`) passes the string type's `TryFromSexp` as `left`.
///
/// A value that `R` refuses is reported against the whole parameter:
/// `'route' must be one of "oral", "bolus", "infusion", or a data frame: got
/// integer`.
///
/// So `R` must read something other than character or factor input: an `R`
/// whose [`TryFromSexp::CHARACTER_ONLY`] is `true` could receive at most
/// `NULL`, and `#[miniextendr]` rejects the parameter at compile time.
///
/// This differs from `TryFromSexp for Either<L, R>`, which tries `L` first on
/// every input and would decode `NULL` as the first choice.
#[cfg(feature = "either")]
pub fn match_arg_either_or<L, R, E>(
    sexp: SEXP,
    left: impl FnOnce(SEXP) -> Result<L, E>,
) -> Result<either::Either<L, R>, SexpError>
where
    E: Into<SexpError>,
    R: TryFromSexp,
    R::Error: Into<SexpError>,
{
    if sexp.type_of() == SEXPTYPE::STRSXP || sexp.is_factor() {
        left(sexp).map(either::Either::Left).map_err(Into::into)
    } else {
        R::try_from_sexp(sexp)
            .map(either::Either::Right)
            .map_err(Into::into)
    }
}

/// Reads [`TryFromSexp::CHARACTER_ONLY`] of the `R` arm of an `Either<T, R>`
/// choice parameter without requiring `R: TryFromSexp`.
///
/// `#[miniextendr]` emits, for each such arm (inside the generated C
/// wrapper),
///
/// ```ignore
/// {
///     use ::miniextendr_api::match_arg::EitherArmProbeFallback as _;
///     const _: () = ::core::assert!(
///         !::miniextendr_api::match_arg::EitherArmProbe::<R>::CHARACTER_ONLY,
///         "...",
///     );
/// }
/// ```
///
/// The path resolves to the inherent const below when `R: TryFromSexp`
/// holds. When it does not, path resolution rejects the inherent candidate
/// (its impl bound fails) and falls back to the const of
/// [`EitherArmProbeFallback`], which is `false`. So the check adds no error
/// of its own for an `R` without a conversion: the decode's single "`R:
/// TryFromSexp` is not satisfied" is the whole diagnostic. Naming
/// `<R as TryFromSexp>::CHARACTER_ONLY` directly would report that missing
/// impl a second time.
#[doc(hidden)]
pub struct EitherArmProbe<R>(core::marker::PhantomData<fn() -> R>);

impl<R: TryFromSexp> EitherArmProbe<R> {
    /// `R`'s [`TryFromSexp::CHARACTER_ONLY`].
    pub const CHARACTER_ONLY: bool = R::CHARACTER_ONLY;
}

/// The fallback of [`EitherArmProbe`] for an `R` without `TryFromSexp`: the
/// check passes, and the decode reports the missing impl.
#[doc(hidden)]
pub trait EitherArmProbeFallback {
    /// Always `false`.
    const CHARACTER_ONLY: bool = false;
}

impl<R> EitherArmProbeFallback for EitherArmProbe<R> {}

/// Match a string against the choices of a `MatchArg` type (exact or partial).
fn match_choice<T: MatchArg>(input: &str) -> Result<T, MatchArgError> {
    let no_match = || MatchArgError::NoMatch {
        input: input.to_string(),
        choices: <T as MatchArg>::CHOICES,
    };
    // `pmatch()` matches the empty string to nothing, not even to an empty
    // choice; without this every choice would be a partial match.
    if input.is_empty() {
        return Err(no_match());
    }

    // Exact match
    if let Some(val) = T::from_choice(input) {
        return Ok(val);
    }

    // Unique partial match (like R's match.arg)
    let mut matches = <T as MatchArg>::CHOICES
        .iter()
        .filter(|choice| choice.starts_with(input));
    match (matches.next(), matches.next()) {
        (Some(choice), None) => T::from_choice(choice).ok_or_else(no_match),
        _ => Err(no_match()),
    }
}

/// Convert a `Vec<T: MatchArg>` to an R character vector (STRSXP).
///
/// Each element is written as its canonical choice string via [`MatchArg::to_choice`].
/// Empty choice strings are stored as `R_BlankString` (parity with [`choices_sexp`]).
///
/// Called by the [`MatchArg`]→[`IntoRVecElement`](crate::newtype::IntoRVecElement)
/// bridge below, which backs `IntoR for Vec<MyEnum>`.
pub fn match_arg_vec_into_sexp<T: MatchArg>(values: Vec<T>) -> SEXP {
    unsafe {
        let scope = ProtectScope::new();
        // Preserve the R_BlankString short-circuit: skips hash-lookup on the
        // interned empty CHARSXP for any empty choice strings.
        let vec = scope.alloc_strsxp(values.len()).into_raw();
        for (i, v) in values.into_iter().enumerate() {
            let s = v.to_choice();
            let charsxp = if s.is_empty() {
                SEXP::blank_string()
            } else {
                SEXP::charsxp(s)
            };
            vec.set_string_elt(i as R_xlen_t, charsxp);
        }
        vec
    }
}

/// Bridge: every [`MatchArg`] type is an [`IntoRVecElement`](crate::newtype::IntoRVecElement),
/// so `Vec<MyEnum>` converts to an R character vector (STRSXP) via
/// [`match_arg_vec_into_sexp`].
///
/// `IntoR for Vec<T>` has a single blanket slot (see [`crate::newtype`]). Routing
/// `MatchArg` through `IntoRVecElement` lets `#[derive(IntoR)]` newtypes share
/// that slot — a newtype implements `IntoRVecElement` concretely in its own crate,
/// which coexists with this bridge because a local newtype is provably not
/// `MatchArg` (deriving both `MatchArg` and `IntoR` on one type is an E0119
/// coherence error, by design). Stable Rust has no negative trait bounds, so a
/// *second* `impl<T: …> IntoR for Vec<T>` blanket would conflict directly; this
/// indirection is what avoids that.
impl<T: MatchArg> crate::newtype::IntoRVecElement for T {
    fn elements_into_sexp(values: Vec<Self>) -> SEXP {
        match_arg_vec_into_sexp(values)
    }
}

/// Extract multiple strings from an R SEXP (STRSXP) and match each against
/// the choices of a `MatchArg` type.
///
/// Used by the generated C wrapper for `match_arg + several_ok` parameters
/// (`match.arg` with `several.ok = TRUE`).
///
/// NULL input returns all variants. The R prelude's strict `several_ok`
/// helper maps `NULL` the same way and rejects any element that matches no
/// choice before the value reaches this function, so the per-element check
/// here is the second line of defence, not the only one (#1472). Under an
/// `Either<Vec<T>, R>` parameter (#1612), `match_arg_either_or` calls it
/// only for character or factor input, so `NULL` never reaches it there.
///
/// Note: factors (INTSXP) are not handled here — the R wrapper coerces factors
/// to character before the `.Call()` boundary, on the `Either` path too.
pub fn match_arg_vec_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<Vec<T>, MatchArgError> {
    // NIL → all choices (match.arg default with several.ok = TRUE).
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return <T as MatchArg>::CHOICES
            .iter()
            .map(|c| {
                T::from_choice(c).ok_or(MatchArgError::NoMatch {
                    input: (*c).to_string(),
                    choices: <T as MatchArg>::CHOICES,
                })
            })
            .collect();
    }

    // STRSXP path — delegate type check + per-element NA handling
    // to Vec<Option<&str>>. `None` means NA_character_ (IsNa error).
    <Vec<Option<&'static str>> as TryFromSexp>::try_from_sexp(sexp)
        .map_err(sexp_err_to_match_arg_err::<T>)?
        .into_iter()
        .map(|opt| {
            opt.ok_or(MatchArgError::IsNa {
                choices: <T as MatchArg>::CHOICES,
            })
            .and_then(match_choice::<T>)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::escape_r_string;

    #[test]
    fn escapes_backslash_and_quote() {
        assert_eq!(escape_r_string(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_r_string(r"c:\path"), r"c:\\path");
    }

    #[test]
    fn escapes_control_characters() {
        assert_eq!(escape_r_string("line1\nline2"), r"line1\nline2");
        assert_eq!(escape_r_string("tab\there"), r"tab\there");
        assert_eq!(escape_r_string("cr\rlf"), r"cr\rlf");
    }

    #[test]
    fn passes_through_plain_strings() {
        assert_eq!(escape_r_string("Fast"), "Fast");
        assert_eq!(escape_r_string("it's"), "it's");
        assert_eq!(escape_r_string(""), "");
    }

    /// The expectation of a choice parameter: one or several choices, the
    /// other accepted values after them, quoted as `one_of` quotes.
    #[test]
    fn choice_expectation_names_the_choices_and_the_rest() {
        use super::{choice_expectation, one_of};
        assert_eq!(choice_expectation(&["fast"], false, ""), r#"one of "fast""#);
        assert_eq!(
            choice_expectation(&["fast", "slow"], false, ", or a data frame"),
            r#"one of "fast", "slow", or a data frame"#
        );
        assert_eq!(
            choice_expectation(&["fast", "slow"], true, ", or a number"),
            r#"one or more of "fast", "slow", or a number"#
        );
        assert_eq!(
            choice_expectation(&[r#"say "hi""#, "bye"], false, ""),
            r#"one of "say \"hi\"", "bye""#
        );
        assert_eq!(one_of(&["fast", "slow"]), r#"one of "fast", "slow""#);
    }

    /// A `default = "..."` formal: the default first, the rest in their
    /// order (not a rotation of the list), and `None` for a default that is
    /// no choice (#1767).
    #[test]
    fn default_first_moves_the_default_to_the_front() {
        use super::default_first;
        let choices = ["Fast", "Safe", "Debug"];
        let is = |default: &'static str| move |c: &str| c == default;
        assert_eq!(
            default_first(&choices, is("Safe")).unwrap(),
            ["Safe", "Fast", "Debug"]
        );
        assert_eq!(
            default_first(&choices, is("Debug")).unwrap(),
            ["Debug", "Fast", "Safe"]
        );
        assert_eq!(default_first(&choices, is("Fast")).unwrap(), choices);
        assert_eq!(default_first(&choices, is("Slow")), None);
        assert_eq!(default_first(&["only"], is("only")).unwrap(), ["only"]);
    }
}
