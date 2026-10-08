//! The call an S3 method's conditions report (#1851), and `call = none` on a
//! function.
//!
//! Under `UseMethod()` dispatch R records the dispatched call in the method's
//! frame, `summary.mx_rec(x)` for `summary(x)`, and a replacement method's
//! call carries the new value, `` `$<-.mx_rec`(`*tmp*`, f, value = ...) ``.
//! An S3 method generated here reports the generic's call instead, `summary(x)`
//! and `x$f <- value`; a direct call of the method keeps its name.
//!
//! `mx_rec` is a classed list built in R
//! (`structure(list(a = 1, f = list(col = c(1, 2, 3)), locked = list(col = c(1, 2, 3))), class = "mx_rec")`);
//! only its methods live here. Its `locked` field can be read but not
//! replaced, so every replacement method has an error to report, and `a`
//! drives `summary()` and `format()`: negative is an error, zero a warning.
//! `MxTally` is the impl-block counterpart. Tests live in
//! `rpkg/tests/testthat/test-s3-call.R`.

use miniextendr_api::dots::Dots;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{ProtectScope, SexpExt};
use miniextendr_api::{List, SEXP, defer_warning, miniextendr, rust_error};

use crate::match_arg_tests::Mode;

// region: field lookup

/// The 0-based position of the field called `name`.
fn position_of(x: List, name: &str) -> Result<isize, String> {
    let names = x.names();
    (0..x.len())
        .find(|&k| names.is_some_and(|names| names.string_elt_str(k) == Some(name)))
        .ok_or_else(|| format!("an mx_rec has no field `{name}`"))
}

/// The name of the field at 0-based position `k`.
fn field_name(x: List, k: isize) -> String {
    x.names()
        .and_then(|names| names.string_elt_str(k).map(str::to_string))
        .unwrap_or_default()
}

/// The 0-based position of the one field `i` selects, by name or by 1-based
/// position (a double from `x[[2]]`, an integer from `lapply()`).
fn single_position(x: List, i: SEXP) -> Result<isize, String> {
    if i.xlength() != 1 {
        return Err(format!(
            "`[[` selects one mx_rec field, not {}",
            i.xlength()
        ));
    }
    if i.is_character() {
        let name = i
            .string_elt_str(0)
            .ok_or_else(|| "a field name can't be NA".to_string())?;
        return position_of(x, name);
    }
    let position = if i.is_integer() {
        f64::from(i.integer_elt(0))
    } else if i.is_real() {
        i.real_elt(0)
    } else {
        return Err("select mx_rec fields by name or by position".to_string());
    };
    position_at(x, position)
}

/// The 0-based position of the field at 1-based `position`.
fn position_at(x: List, position: f64) -> Result<isize, String> {
    (0..x.len())
        .find(|&k| {
            i32::try_from(k + 1)
                .ok()
                .is_some_and(|p| f64::from(p) == position)
        })
        .ok_or_else(|| format!("an mx_rec has no field at position {position}"))
}

/// The `locked` field can't be replaced.
fn refuse_locked(x: List, k: isize) -> Result<(), String> {
    if field_name(x, k) == "locked" {
        return Err("the `locked` field of an mx_rec can't be replaced".to_string());
    }
    Ok(())
}

/// The `a` field as a number.
fn a_of(x: List) -> Result<f64, String> {
    let a = x.as_sexp().vector_elt(position_of(x, "a")?);
    if a.xlength() != 1 {
        return Err("`a` must be one number".to_string());
    }
    if a.is_real() {
        Ok(a.real_elt(0))
    } else if a.is_integer() {
        Ok(f64::from(a.integer_elt(0)))
    } else {
        Err("`a` must be a number".to_string())
    }
}

/// A copy of `x` changed by `edit`, rooted before whatever `edit` allocates.
fn edited(x: List, edit: impl FnOnce(SEXP)) -> SEXP {
    let scope = unsafe { ProtectScope::new() };
    let copy = unsafe { scope.protect_raw(x.as_sexp().shallow_duplicate()) };
    edit(copy);
    copy
}

// endregion

// region: standalone methods on ordinary generics

/// `summary(rec)`: `a` summed with the number of extra arguments. An error
/// from the body, and a warning deferred from it, report `summary(rec)`, the
/// generic's call, under dispatch; `summary.mx_rec(rec)` called directly
/// reports that.
///
/// @param object An `mx_rec`.
/// @param ... Counted.
/// @return A number.
/// @export
#[miniextendr(s3(generic = "summary", class = "mx_rec"))]
pub fn summary_mx_rec(object: List, dots: &Dots) -> Result<f64, String> {
    let a = a_of(object)?;
    if a < 0.0 {
        return Err(format!("`a` must be non-negative, got {a}"));
    }
    if a == 0.0 {
        defer_warning!("`a` is zero");
    }
    let extras = u32::try_from(dots.len()).map_err(|e| e.to_string())?;
    Ok(a + f64::from(extras))
}

/// `format(rec)`, with `call = none`: its conditions carry no call at all,
/// however the method was reached. Same rules for `a` as `summary()`.
///
/// @param x An `mx_rec`.
/// @param ... Ignored.
/// @return A string.
/// @export
#[miniextendr(s3(generic = "format", class = "mx_rec"), call = none)]
pub fn format_mx_rec(x: List, _dots: &Dots) -> Result<String, String> {
    let a = a_of(x)?;
    if a < 0.0 {
        return Err(format!("`a` must be non-negative, got {a}"));
    }
    if a == 0.0 {
        defer_warning!("`a` is zero");
    }
    Ok(format!("mx_rec(a = {a})"))
}

/// `rec + 1`, `rec == rec`: the `Ops` group generic. No operator is defined,
/// so every one is an error, reported as the operator call, `rec + 1`.
///
/// @param e1 An `mx_rec`, or the other operand.
/// @param e2 An `mx_rec`, or the other operand.
/// @export
#[miniextendr(s3(generic = "Ops", class = "mx_rec"))]
pub fn ops_mx_rec(_e1: SEXP, _e2: SEXP) -> Result<SEXP, String> {
    Err("no operator is defined on an mx_rec".to_string())
}

// endregion

// region: standalone methods on extraction generics

/// The names of the fields `i` selects, through `[`: `rec[2]`. A position
/// the record does not have is an error, reported as `rec[5]`.
///
/// @param x An `mx_rec`.
/// @param i 1-based positions.
/// @param ... Ignored.
/// @export
#[miniextendr(s3(generic = "[", class = "mx_rec"))]
pub fn subset_mx_rec(x: List, i: Vec<f64>, _dots: &Dots) -> Result<Vec<String>, String> {
    i.iter()
        .map(|&p| position_at(x, p).map(|k| field_name(x, k)))
        .collect()
}

/// A field by name, through `$`: `rec$f`.
///
/// @param x An `mx_rec`.
/// @param name The field name. R passes it as a character string.
/// @export
#[miniextendr(s3(generic = "$", class = "mx_rec"))]
pub fn dollar_mx_rec(x: List, name: &str) -> Result<SEXP, String> {
    Ok(x.as_sexp().vector_elt(position_of(x, name)?))
}

// endregion

// region: standalone methods on replacement generics

/// Replace a field through `$<-`: `rec$f <- value`, and the nested
/// `rec$f$col[i] <- v`, which hands this method the whole new `f`. Refusing
/// `locked` reports `rec$locked <- value`, never the value.
///
/// @param x An `mx_rec`.
/// @param name The field name. R passes it as a character string.
/// @param value The new field value.
/// @return The changed record.
/// @export
#[miniextendr(s3(generic = "$<-", class = "mx_rec"))]
pub fn dollar_assign_mx_rec(x: List, name: &str, value: SEXP) -> Result<SEXP, String> {
    let k = position_of(x, name)?;
    refuse_locked(x, k)?;
    Ok(edited(x, |copy| copy.set_vector_elt(k, value)))
}

/// Replace a field by name or position through `[[<-`: `rec[["f"]] <- v`,
/// `rec[[2]] <- v`.
///
/// @param x An `mx_rec`.
/// @param i A field name, or a 1-based position.
/// @param value The new field value.
/// @return The changed record.
/// @export
#[miniextendr(s3(generic = "[[<-", class = "mx_rec"))]
pub fn element_assign_mx_rec(x: List, i: SEXP, value: SEXP) -> Result<SEXP, String> {
    let k = single_position(x, i)?;
    refuse_locked(x, k)?;
    Ok(edited(x, |copy| copy.set_vector_elt(k, value)))
}

/// Replace one field through `[<-`: `rec["f"] <- list(v)`.
///
/// @param x An `mx_rec`.
/// @param i A field name, or a 1-based position.
/// @param ... Further indices, as in `rec[i, j] <- value`. A record takes
///   one index, so any further index is an error.
/// @param value A list with the one new value.
/// @return The changed record.
/// @export
#[miniextendr(s3(generic = "[<-", class = "mx_rec"))]
pub fn subset_assign_mx_rec(x: List, i: SEXP, rest: &Dots, value: List) -> Result<SEXP, String> {
    if !rest.is_empty() {
        return Err(format!(
            "an mx_rec takes one index in `[<-`, not {}",
            rest.len() + 1
        ));
    }
    let k = single_position(x, i)?;
    refuse_locked(x, k)?;
    if value.len() != 1 {
        return Err(format!(
            "`[<-` takes one value for one field, not {}",
            value.len()
        ));
    }
    Ok(edited(x, |copy| {
        copy.set_vector_elt(k, value.as_sexp().vector_elt(0));
    }))
}

/// Rename the fields through `names<-`: `names(rec) <- value`. The `locked`
/// field keeps its name, so any other set of names is an error, reported as
/// `names(rec) <- value`.
///
/// @param x An `mx_rec`.
/// @param value The new field names, one per field.
/// @return The renamed record.
/// @export
#[miniextendr(s3(generic = "names<-", class = "mx_rec"))]
pub fn names_assign_mx_rec(x: List, value: Vec<String>) -> Result<SEXP, String> {
    let locked = position_of(x, "locked")?;
    if usize::try_from(locked)
        .ok()
        .and_then(|k| value.get(k))
        .is_none_or(|name| name != "locked")
    {
        return Err("the `locked` field of an mx_rec keeps its name".to_string());
    }
    if isize::try_from(value.len()).ok() != Some(x.len()) {
        return Err(format!(
            "an mx_rec has {} fields, so it takes {} names, not {}",
            x.len(),
            x.len(),
            value.len()
        ));
    }
    Ok(edited(x, |copy| {
        copy.set_names(value.into_sexp());
    }))
}

// endregion

// region: an impl-block S3 class

/// A tally of values, whose methods are generated from an impl block.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxTally {
    values: Vec<f64>,
}

#[miniextendr(s3)]
impl MxTally {
    /// @param values The values.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    /// The value at 1-based position `i`, through `peek()`: an error for a
    /// position the tally does not have, reported as `peek(t, i)` under
    /// dispatch, and a warning for position zero.
    ///
    /// @param i 1-based position; zero warns and returns `NA`.
    pub fn peek(&self, i: i32) -> Result<Option<f64>, String> {
        if i == 0 {
            defer_warning!("peeking at position zero");
            return Ok(None);
        }
        usize::try_from(i)
            .ok()
            .and_then(|k| k.checked_sub(1))
            .and_then(|k| self.values.get(k).copied())
            .map(Some)
            .ok_or_else(|| format!("index {i} out of range for {} values", self.values.len()))
    }

    /// The value at 1-based position `i`, through `[[`: `t[[i]]`.
    ///
    /// @param i 1-based position.
    #[miniextendr(s3(generic = "[["))]
    pub fn at(&self, i: i32) -> Result<f64, String> {
        usize::try_from(i)
            .ok()
            .and_then(|k| k.checked_sub(1))
            .and_then(|k| self.values.get(k).copied())
            .ok_or_else(|| format!("index {i} out of range for {} values", self.values.len()))
    }
}

// endregion

// region: call = none on a function

/// A function whose conditions carry no call (`call = none`): an error from
/// the body (negative `x`), a deferred warning (zero), a panic (one), a classed
/// error (two), the R-side checks of `x` (an integer scalar) and `mode` (a
/// choice), and a failed conversion (`NA`). Every one has a `NULL`
/// `conditionCall()`. [with_call_verb()] is the same function with the default
/// attribution, for comparison.
///
/// @param x A count.
/// @param mode One of the modes.
/// @return `x`.
/// @export
#[miniextendr(call = none)]
pub fn no_call_verb(x: i32, #[miniextendr(match_arg)] mode: Mode) -> Result<i32, String> {
    call_verb(x, mode)
}

/// [no_call_verb()] with the default attribution: every condition names
/// `with_call_verb(...)`.
///
/// @param x A count.
/// @param mode One of the modes.
/// @return `x`.
/// @export
#[miniextendr]
pub fn with_call_verb(x: i32, #[miniextendr(match_arg)] mode: Mode) -> Result<i32, String> {
    call_verb(x, mode)
}

fn call_verb(x: i32, mode: Mode) -> Result<i32, String> {
    let _ = mode;
    match x {
        x if x < 0 => Err(format!("x must be non-negative, got {x}")),
        0 => {
            defer_warning!("x is zero");
            Ok(x)
        }
        1 => panic!("x is one"),
        2 => rust_error!(class = "pkg_two", "x is two"),
        x => Ok(x),
    }
}

// endregion
