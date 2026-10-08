//! Standalone S3 methods on R's extraction and replacement generics (#1853):
//! `$`, `[[`, `$<-`, `[[<-`, `[<-` and `names<-`.
//!
//! `mx_thing` is a classed list built in R
//! (`structure(list(f = list(col = c(1, 2, 3)), g = "a"), class = "mx_thing")`);
//! only its methods live here. A thing has a fixed set of fields: every method
//! refuses a field the thing does not have, where a plain list would return
//! `NULL` or add the field, so a test can tell the Rust method ran.
//!
//! Each replacement method edits a copy of `x` (R may still share `x`'s
//! elements with other bindings while it runs the assignment), records the
//! generic it implements in the copy's `mx_via` attribute, and returns the
//! copy: R assigns whatever a replacement method returns to the variable.

use miniextendr_api::dots::Dots;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{ProtectScope, SexpExt};
use miniextendr_api::{List, SEXP, miniextendr};

// region: field lookup

/// The 0-based position of the field called `name`.
fn position_of(x: List, name: &str) -> Result<isize, String> {
    let names = x.names();
    (0..x.len())
        .find(|&k| names.is_some_and(|names| names.string_elt_str(k) == Some(name)))
        .ok_or_else(|| format!("an mx_thing has no field `{name}`"))
}

/// The 0-based position of the field at 1-based `position`.
fn position_at(x: List, position: f64) -> Result<isize, String> {
    let len = i32::try_from(x.len()).map_err(|_| "an mx_thing has too many fields".to_string())?;
    (1..=len)
        .position(|p| f64::from(p) == position)
        .and_then(|k| isize::try_from(k).ok())
        .ok_or_else(|| format!("an mx_thing has no field at position {position}"))
}

/// The 0-based positions of the fields `i` selects: field names, or 1-based
/// positions. R passes `x[[2]]` as a double and `lapply()` as an integer.
fn field_positions(x: List, i: SEXP) -> Result<Vec<isize>, String> {
    let n = i.xlength();
    if i.is_character() {
        (0..n)
            .map(|k| {
                let name = i
                    .string_elt_str(k)
                    .ok_or_else(|| "a field name can't be NA".to_string())?;
                position_of(x, name)
            })
            .collect()
    } else if i.is_integer() {
        (0..n)
            .map(|k| match i.integer_elt(k) {
                i32::MIN => Err("a field position can't be NA".to_string()),
                p => position_at(x, f64::from(p)),
            })
            .collect()
    } else if i.is_real() {
        (0..n).map(|k| position_at(x, i.real_elt(k))).collect()
    } else {
        Err("select mx_thing fields by name or by position".to_string())
    }
}

/// The one field `i` selects, for `[[` and `[[<-`.
fn single_position(x: List, i: SEXP) -> Result<isize, String> {
    match field_positions(x, i)?.as_slice() {
        [k] => Ok(*k),
        positions => Err(format!(
            "`[[` selects one mx_thing field, not {}",
            positions.len()
        )),
    }
}

// endregion

// region: the edited copy

/// A copy of `x` changed by `edit`, its `mx_via` attribute naming `generic`.
///
/// `edit` runs after every check, so it can't fail. The copy is rooted before
/// the allocations that follow it (the attribute value, and whatever `edit`
/// allocates).
fn edited(x: List, generic: &str, edit: impl FnOnce(SEXP)) -> SEXP {
    let scope = unsafe { ProtectScope::new() };
    let copy = unsafe { scope.protect_raw(x.as_sexp().shallow_duplicate()) };
    edit(copy);
    copy.set_attr(
        SEXP::symbol("mx_via"),
        SEXP::scalar_string_from_str(generic),
    );
    copy
}

// endregion

// region: extraction generics

/// A field of a thing by name, through `$`: `thing$f`.
///
/// @param x An `mx_thing`.
/// @param name The field name. R passes it as a character string.
/// @export
#[miniextendr(s3(generic = "$", class = "mx_thing"))]
pub fn mx_thing_dollar(x: List, name: &str) -> Result<SEXP, String> {
    Ok(x.as_sexp().vector_elt(position_of(x, name)?))
}

/// A field of a thing by name or position, through `[[`: `thing[["f"]]`,
/// `thing[[2]]`. `lapply()` and `str()` call it too.
///
/// @param x An `mx_thing`.
/// @param i A field name, or a 1-based position.
/// @export
#[miniextendr(s3(generic = "[[", class = "mx_thing"))]
pub fn mx_thing_element(x: List, i: SEXP) -> Result<SEXP, String> {
    Ok(x.as_sexp().vector_elt(single_position(x, i)?))
}

// endregion

// region: replacement generics

/// Replace a field through `$<-`: `thing$f <- value`, and the nested
/// `thing$f$col[i] <- v`, which hands this method the whole new `f`.
///
/// @param x An `mx_thing`.
/// @param name The field name. R passes it as a character string.
/// @param value The new field value.
/// @return The changed thing.
/// @export
#[miniextendr(s3(generic = "$<-", class = "mx_thing"))]
pub fn mx_thing_dollar_assign(x: List, name: &str, value: SEXP) -> Result<SEXP, String> {
    let k = position_of(x, name)?;
    Ok(edited(x, "$<-", |copy| copy.set_vector_elt(k, value)))
}

/// Replace a field by name or position through `[[<-`: `thing[["f"]] <- v`,
/// `thing[[2]] <- v`, `thing[["f"]]$col <- v`, and `modifyList()`.
///
/// @param x An `mx_thing`.
/// @param i A field name, or a 1-based position.
/// @param value The new field value.
/// @return The changed thing.
/// @export
#[miniextendr(s3(generic = "[[<-", class = "mx_thing"))]
pub fn mx_thing_element_assign(x: List, i: SEXP, value: SEXP) -> Result<SEXP, String> {
    let k = single_position(x, i)?;
    Ok(edited(x, "[[<-", |copy| copy.set_vector_elt(k, value)))
}

/// Replace several fields through `[<-`: `thing[c("f", "g")] <- list(v, w)`.
///
/// @param x An `mx_thing`.
/// @param i Field names, or 1-based positions.
/// @param ... Further indices, as in `thing[i, j] <- value`. A thing takes
///   one index, so any further index is an error.
/// @param value A list with one new value per selected field.
/// @return The changed thing.
/// @export
#[miniextendr(s3(generic = "[<-", class = "mx_thing"))]
pub fn mx_thing_subset_assign(x: List, i: SEXP, rest: &Dots, value: List) -> Result<SEXP, String> {
    if !rest.is_empty() {
        return Err(format!(
            "an mx_thing takes one index in `[<-`, not {}",
            rest.len() + 1
        ));
    }
    let positions = field_positions(x, i)?;
    if usize::try_from(value.len()).ok() != Some(positions.len()) {
        return Err(format!(
            "`[<-` needs one value per selected field: {} fields, {} values",
            positions.len(),
            value.len()
        ));
    }
    Ok(edited(x, "[<-", |copy| {
        for (k, &position) in (0..value.len()).zip(&positions) {
            copy.set_vector_elt(position, value.as_sexp().vector_elt(k));
        }
    }))
}

/// Rename the fields through `names<-`: `names(thing) <- value`, and
/// `names(thing)[2] <- "h"`, which hands this method the whole new names
/// vector.
///
/// @param x An `mx_thing`.
/// @param value The new field names, one per field, all different.
/// @return The renamed thing.
/// @export
#[miniextendr(s3(generic = "names<-", class = "mx_thing"))]
pub fn mx_thing_names_assign(x: List, value: Vec<String>) -> Result<SEXP, String> {
    if isize::try_from(value.len()).ok() != Some(x.len()) {
        return Err(format!(
            "an mx_thing has {} fields, so it takes {} names, not {}",
            x.len(),
            x.len(),
            value.len()
        ));
    }
    if let Some((k, name)) = value
        .iter()
        .enumerate()
        .find(|&(k, name)| value[..k].contains(name))
    {
        return Err(format!(
            "mx_thing field names must differ: `{name}` (field {}) repeats an earlier one",
            k + 1
        ));
    }
    Ok(edited(x, "names<-", |copy| {
        copy.set_names(value.into_sexp())
    }))
}

// endregion

// region: gctorture fixture

/// Every method above on a thing built here, for the no-arg gctorture sweep
/// (#430): each replacement method holds its copy of `x` across the
/// allocations that follow it.
#[miniextendr(noexport)]
pub fn gc_stress_s3_replacement() -> Result<(), String> {
    let scope = unsafe { ProtectScope::new() };
    let col = unsafe { scope.protect_raw(vec![1.0, 2.0, 3.0].into_sexp()) };
    let f = unsafe { scope.protect_raw(List::from_raw_pairs(vec![("col", col)]).as_sexp()) };
    let g = unsafe { scope.protect_raw("a".into_sexp()) };
    let thing = List::from_raw_pairs(vec![("f", f), ("g", g)]);
    unsafe { scope.protect_raw(thing.as_sexp()) };
    let thing = thing.set_class_str(&["mx_thing"]);

    let thing =
        unsafe { List::from_raw(scope.protect_raw(mx_thing_dollar_assign(thing, "g", g)?)) };
    let position = unsafe { scope.protect_raw(2.0_f64.into_sexp()) };
    let thing = unsafe {
        List::from_raw(scope.protect_raw(mx_thing_element_assign(thing, position, col)?))
    };
    let name = unsafe { scope.protect_raw("f".into_sexp()) };
    let values =
        unsafe { List::from_raw(scope.protect_raw(List::from_raw_values(vec![g]).as_sexp())) };
    let thing = unsafe {
        List::from_raw(scope.protect_raw(mx_thing_subset_assign(
            thing,
            name,
            &Dots::empty(),
            values,
        )?))
    };
    let thing = unsafe {
        List::from_raw(scope.protect_raw(mx_thing_names_assign(
            thing,
            vec!["a".to_string(), "b".to_string()],
        )?))
    };
    mx_thing_dollar(thing, "b")?;
    mx_thing_element(thing, position)?;
    let via = thing.as_sexp().get_attr(SEXP::symbol("mx_via"));
    if !(via.is_character() && via.string_elt_str(0) == Some("names<-")) {
        return Err("the last edit was not recorded".to_string());
    }
    Ok(())
}

// endregion
