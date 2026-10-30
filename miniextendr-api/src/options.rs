//! Reading R options (`getOption()`) from Rust.
//!
//! [`get_option`] reads one option and converts it with [`TryFromSexp`].
//! A value that doesn't convert is an [`RError`] whose message names the
//! option, so `?` in a `#[miniextendr]` function returning `Result<_, RError>`
//! raises it with its classes:
//!
//! ```ignore
//! use miniextendr_api::condition::RError;
//! use miniextendr_api::{get_option, miniextendr};
//!
//! #[miniextendr]
//! pub fn report_width() -> Result<i32, RError> {
//!     Ok(get_option::<i32>("mypkg.width")?.unwrap_or(80))
//! }
//! ```
//!
//! ```r
//! options(mypkg.width = "wide")
//! report_width()
//! #> Error in report_width() :
//! #>   options("mypkg.width") is invalid: expected integer, got character
//! ```

use crate::condition::RError;
use crate::from_r::{SexpError, TryFromSexp};
use crate::gc_protect::OwnedProtect;
use crate::sexp::SEXP;

/// The class every [`get_option`] conversion error carries.
pub const OPTION_ERROR_CLASS: &str = "miniextendr_option_error";

/// Read the R option `name`, as `getOption(name)` does, and convert it to `T`.
///
/// `Ok(None)` when the option is unset or set to `NULL` (R does not tell the
/// two apart). A value that doesn't convert is an error: its message names
/// `options("name")` and gives the conversion's reason in R terms, its class
/// is [`OPTION_ERROR_CLASS`] (after the classes of a refusal that carries its
/// own, such as a `#[try_from_sexp(validate = ...)]` check), and `e$option`
/// is `name`.
///
/// Read into an owned `T` (`String`, `Vec<_>`, ...). A borrowed `T` such as
/// `&str` points into the option's value, which nothing roots once this
/// returns: `options(name = NULL)` or a new value lets R free it.
///
/// # Panics
///
/// Off R's main thread, before reading anything.
pub fn get_option<T>(name: &str) -> Result<Option<T>, RError>
where
    T: TryFromSexp,
    SexpError: From<T::Error>,
{
    assert!(
        crate::is_r_main_thread(),
        "`get_option(\"{name}\")` reads R's options on R's main thread: call it there, \
         not from a thread the package spawned",
    );

    // Symbols are never collected; `Rf_GetOption1` doesn't allocate.
    let value = unsafe { crate::sys::Rf_GetOption1(SEXP::symbol(name)) };
    if value.is_nil() {
        return Ok(None);
    }

    // The conversion may run R code that resets the option, after which only
    // this protection keeps the value alive.
    // SAFETY: R's main thread (checked above); released when the guard drops.
    let guard = unsafe { OwnedProtect::new(value) };
    T::try_from_sexp(guard.get())
        .map(Some)
        .map_err(|e| option_error(name, SexpError::from(e)))
}

/// The error of an option `name` whose value didn't convert: the refusal's
/// reason in R terms, its classes and fields kept when it carries its own.
fn option_error(name: &str, err: SexpError) -> RError {
    let reason = err.r_reason(false);
    let mut out = RError::new(format!("options(\"{name}\") is invalid: {reason}"));
    if let SexpError::Condition(inner) = &err {
        out = out.class(inner.classes().to_vec());
        for (field, value) in inner.fields() {
            out = out.data(field.clone(), value.clone());
        }
    }
    out.class(OPTION_ERROR_CLASS).data("option", name)
}
