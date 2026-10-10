//! Fixtures for the unforced dots, `LazyDots` (#1892).
//!
//! A `LazyDots` parameter is R's `...`, which the generated wrapper passes as
//! its own frame, `environment()`, where a `&Dots` parameter's passes
//! `list(...)`. The body counts, names and reads the dots without forcing
//! them, sees which are empty (`f(a = )`, `x[1, , , ]`), and forces each
//! element in its own environment.
//!
//! - Standalone readers (`lazy_dots_len`, `_names`, `_missing`, `_exprs`) and
//!   forcers (`lazy_dots_force*`, `lazy_dots_try_force`).
//! - S3 methods: `subset.mx_lazy` refuses extra arguments before evaluating
//!   them, `update.mx_lazy` accepts `select = `, and `[.mx_lazy_grid` reports
//!   the subscript forms of `x[1, , , ]` with `Missing` formals and `NArgs`.
//! - One impl block per class system (env, R6, S3, S4, S7, vctrs).
//!
//! Each forcing fixture holds a [`DropSentinel`], so the tests can tell that
//! an R exit ran the Rust destructors: `lazy_dots_sentinel_drops()` counts
//! them. Tests: `rpkg/tests/testthat/test-lazy-dots.R`.

use std::sync::atomic::{AtomicI32, Ordering};

use miniextendr_api::condition::RConditionError;
use miniextendr_api::expression::RCall;
use miniextendr_api::prelude::{List, OwnedProtect, SEXP, SexpExt};
use miniextendr_api::{
    IntoR, LazyDots, Missing, NArgs, Quoted, SEXPTYPE, TryFromSexp, miniextendr,
};

// region: drop sentinel

static SENTINEL_DROPS: AtomicI32 = AtomicI32::new(0);

/// Counts its drops in [`SENTINEL_DROPS`].
struct DropSentinel;

impl Drop for DropSentinel {
    fn drop(&mut self) {
        SENTINEL_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

/// How many fixture sentinels have been dropped so far in this session.
#[miniextendr(noexport)]
pub fn lazy_dots_sentinel_drops() -> i32 {
    SENTINEL_DROPS.load(Ordering::SeqCst)
}

// endregion

// region: shared helpers

/// An R count as an `i32`.
fn count(n: usize) -> i32 {
    i32::try_from(n).expect("fewer than 2^31 dots")
}

/// `"<name>:empty"` or `"<name>:given"` per element, forcing nothing (an
/// unnamed element has an empty name).
fn describe(rest: &LazyDots) -> Vec<String> {
    rest.names()
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let state = if rest.is_missing_arg(i) {
                "empty"
            } else {
                "given"
            };
            format!("{}:{state}", name.unwrap_or(""))
        })
        .collect()
}

/// The sum of the elements that are not empty, each forced and read as a
/// number.
fn sum_given(rest: &LazyDots) -> f64 {
    (0..rest.len())
        .filter(|&i| !rest.is_missing_arg(i))
        .map(|i| f64::try_from_sexp(rest.force(i)).expect("each element is a number"))
        .sum()
}

/// How an element is named in a message: its name, else `..k`.
fn element_label(name: Option<&str>, i: usize) -> String {
    name.map_or_else(|| format!("..{}", i + 1), str::to_owned)
}

// endregion

// region: reading the dots without forcing them

/// `...length()`: the number of elements, empty ones included.
/// @param ... Anything; nothing is evaluated.
#[miniextendr(noexport)]
pub fn lazy_dots_len(rest: LazyDots) -> i32 {
    count(rest.len())
}

/// Whether `...` holds no elements.
/// @param ... Anything; nothing is evaluated.
#[miniextendr(noexport)]
pub fn lazy_dots_is_empty(rest: LazyDots) -> bool {
    rest.is_empty()
}

/// The name of each element, `NA` for an unnamed one.
/// @param ... Anything; nothing is evaluated.
#[miniextendr(noexport)]
pub fn lazy_dots_names(rest: LazyDots) -> Vec<Option<String>> {
    rest.names()
        .into_iter()
        .map(|name| name.map(str::to_owned))
        .collect()
}

/// Whether each element is empty (`f(a = )`).
/// @param ... Anything; nothing is evaluated.
#[miniextendr(noexport)]
pub fn lazy_dots_missing(rest: LazyDots) -> Vec<bool> {
    (0..rest.len()).map(|i| rest.is_missing_arg(i)).collect()
}

/// The expression of each element as written, in a list; the empty symbol
/// for an empty element.
/// @param ... Anything; nothing is evaluated.
#[miniextendr(noexport)]
pub fn lazy_dots_exprs(rest: LazyDots) -> List {
    // The expressions are rooted by `rest` until the list holds them.
    List::from_raw_values((0..rest.len()).map(|i| rest.expr(i)).collect())
}

/// `"<name>:empty"` / `"<name>:given"` per element, forcing nothing.
/// @param ... Anything; nothing is evaluated.
#[miniextendr(noexport)]
pub fn lazy_dots_describe(rest: LazyDots) -> Vec<String> {
    describe(&rest)
}

// endregion

// region: forcing

/// Force every element in order, empty ones included: an empty element
/// raises R's missing-argument error.
/// @param ... Values to force.
#[miniextendr(noexport)]
pub fn lazy_dots_force(rest: LazyDots) -> List {
    let _sentinel = DropSentinel;
    // Each value is rooted by the frame (its promise, or the `...` binding).
    List::from_raw_values((0..rest.len()).map(|i| rest.force(i)).collect())
}

/// Force every element that is not empty; `NULL` for an empty one.
/// @param ... Values to force; empty ones are skipped.
#[miniextendr(noexport)]
pub fn lazy_dots_force_present(rest: LazyDots) -> List {
    let _sentinel = DropSentinel;
    List::from_raw_values(
        (0..rest.len())
            .map(|i| {
                if rest.is_missing_arg(i) {
                    SEXP::nil()
                } else {
                    rest.force(i)
                }
            })
            .collect(),
    )
}

/// Force element `i` (0-based) and return its value.
/// @param i The 0-based index of the element.
/// @param ... Values; only element `i` is forced.
#[miniextendr(noexport)]
pub fn lazy_dots_force_at(i: i32, rest: LazyDots) -> SEXP {
    let _sentinel = DropSentinel;
    rest.force(usize::try_from(i).expect("a non-negative index"))
}

/// Force the first element twice and return both values.
/// @param ... Values; the first is forced twice.
#[miniextendr(noexport)]
pub fn lazy_dots_force_twice(rest: LazyDots) -> List {
    let first = rest.force(0);
    let second = rest.force(0);
    List::from_raw_values(vec![first, second])
}

/// `try_force` every element: the value, or the R error condition.
/// @param ... Values to force.
#[miniextendr(noexport)]
pub fn lazy_dots_try_force(rest: LazyDots) -> SEXP {
    let _sentinel = DropSentinel;
    let n = rest.len();
    // SAFETY: R's main thread; the list is rooted while it fills, and each
    // condition is stored while its `REvalError` still roots it.
    unsafe {
        let out = OwnedProtect::new(SEXP::alloc_list(isize::try_from(n).expect("dots fit")));
        for i in 0..n {
            let value = match rest.try_force(i) {
                Ok(value) => value,
                Err(err) => err.condition(),
            };
            out.get()
                .set_vector_elt(isize::try_from(i).expect("dots fit"), value);
        }
        out.get()
    }
}

// endregion

// region: S3 methods

/// Extra arguments refused by an `mx_lazy` method, before they are
/// evaluated.
#[derive(Debug, RConditionError)]
#[condition(
    class = "mx_lazy_unsupported_args",
    message = "{generic}() on an mx_lazy takes no further arguments, got: {args}"
)]
pub struct LazyUnsupportedArgs {
    /// The generic the method belongs to.
    generic: String,
    /// The refused arguments: their names, `..k` for an unnamed one.
    args: String,
}

/// The error refusing every element of `rest` for `generic`.
fn unsupported(generic: &str, rest: &LazyDots) -> LazyUnsupportedArgs {
    let args: Vec<String> = rest
        .names()
        .into_iter()
        .enumerate()
        .map(|(i, name)| element_label(name, i))
        .collect();
    LazyUnsupportedArgs {
        generic: generic.to_owned(),
        args: args.join(", "),
    }
}

/// Keep the rows of an `mx_lazy` (a data frame with that class in front)
/// where `subset` is `TRUE`, evaluated with the columns in scope. Every other
/// argument is refused with an `mx_lazy_unsupported_args` error before it is
/// evaluated, so `subset(x, TRUE, select = ID)` names `select` instead of
/// failing on `ID`.
///
/// @param x An `mx_lazy`.
/// @param subset A logical expression over the columns, unevaluated; every
///   row when omitted.
/// @param ... Refused, unevaluated.
/// @export
#[miniextendr(s3(generic = "subset", class = "mx_lazy"))]
pub fn mx_lazy_subset(
    x: SEXP,
    subset: Missing<Quoted>,
    rest: LazyDots,
) -> Result<SEXP, LazyUnsupportedArgs> {
    if !rest.is_empty() {
        return Err(unsupported("subset", &rest));
    }
    let Missing::Present(subset) = subset else {
        return Ok(x);
    };
    // SAFETY: R's main thread; every intermediate value is rooted before the
    // next allocation (`RCall` roots its arguments).
    unsafe {
        let keep = OwnedProtect::new(subset.eval_in(x));
        let rows = OwnedProtect::new(true_rows(keep.get()).into_sexp());
        // x[rows, , drop = FALSE]
        Ok(RCall::new("[")
            .quoted_arg(x)
            .arg(rows.get())
            .arg(SEXP::missing_arg())
            .named_arg("drop", SEXP::scalar_logical(false))
            .eval_with_handlers(miniextendr_api::sys::R_BaseEnv))
    }
}

/// The 1-based positions of the `TRUE` elements of a logical vector.
fn true_rows(keep: SEXP) -> Vec<i32> {
    assert!(
        keep.type_of() == SEXPTYPE::LGLSXP,
        "the condition must evaluate to a logical vector, not {}",
        keep.type_of().type_name()
    );
    (0..keep.len())
        .filter(|&i| keep.logical_elt(isize::try_from(i).expect("an R index")) == 1)
        .map(|i| count(i + 1))
        .collect()
}

/// The steps `update()` would run on an `mx_lazy`: one per named argument.
/// An empty argument (`update(x, select = )`) runs its step without a value.
/// Unnamed arguments are refused with an `mx_lazy_unsupported_args` error.
///
/// @param object An `mx_lazy`.
/// @param ... Named steps; a value is forced, an empty one is not.
/// @return A list: `steps` (the names), `empty` (whether each was empty) and
///   `values` (each forced value, `NULL` for an empty step).
/// @export
#[miniextendr(s3(generic = "update", class = "mx_lazy"))]
pub fn mx_lazy_update(object: SEXP, rest: LazyDots) -> Result<SEXP, LazyUnsupportedArgs> {
    let _ = object;
    let names = rest.names();
    if names.iter().any(Option::is_none) {
        return Err(unsupported("update", &rest));
    }
    let steps: Vec<String> = names.iter().flatten().map(|&n| n.to_owned()).collect();
    let empty: Vec<bool> = (0..rest.len()).map(|i| rest.is_missing_arg(i)).collect();
    // Forced values are rooted by the frame.
    let values: Vec<SEXP> = (0..rest.len())
        .map(|i| if empty[i] { SEXP::nil() } else { rest.force(i) })
        .collect();
    // SAFETY: R's main thread; each part is rooted until the outer list holds
    // it.
    unsafe {
        let steps = OwnedProtect::new(steps.into_sexp());
        let empty = OwnedProtect::new(empty.into_sexp());
        let values = OwnedProtect::new(List::from_raw_values(values).into_sexp());
        Ok(List::from_raw_pairs(vec![
            ("steps", steps.get()),
            ("empty", empty.get()),
            ("values", values.get()),
        ])
        .into_sexp())
    }
}

/// Whether a subscript was given.
fn given<T>(arg: &Missing<T>) -> &'static str {
    if arg.is_present() { "given" } else { "empty" }
}

/// Report the subscripts of `x[...]` on an `mx_lazy_grid` (a classed list,
/// subscripted like a 4-d array) as written: `nargs()`, whether `i`, `j` and
/// `drop` were given, and each element of `...` (its value, or `<empty>`).
/// The formals are base `[`'s `x[i, j, ..., drop]`, so `x[1, , , ]` reaches
/// the method with two empty elements in `...`.
///
/// @param x An `mx_lazy_grid`.
/// @param i,j Subscripts; may be empty.
/// @param ... Further subscripts; empty ones are reported, the others forced.
/// @param drop Matched by name only; may be omitted.
/// @export
#[miniextendr(s3(generic = "[", class = "mx_lazy_grid"))]
pub fn mx_lazy_grid_subset(
    x: SEXP,
    i: Missing<SEXP>,
    j: Missing<SEXP>,
    rest: LazyDots,
    drop: Missing<SEXP>,
    nargs: NArgs,
) -> String {
    let _ = x;
    let dots: Vec<String> = rest
        .names()
        .into_iter()
        .enumerate()
        .map(|(k, name)| {
            let name = name.map(|n| format!("{n}=")).unwrap_or_default();
            if rest.is_missing_arg(k) {
                format!("{name}<empty>")
            } else {
                let value = f64::try_from_sexp(rest.force(k))
                    .map_or_else(|_| "?".to_owned(), |v| v.to_string());
                format!("{name}{value}")
            }
        })
        .collect();
    format!(
        "nargs={} i={} j={} drop={} dots=[{}]",
        nargs.get(),
        given(&i),
        given(&j),
        given(&drop),
        dots.join(", ")
    )
}

// endregion

// region: methods, one class per class system

/// Env class whose methods take `LazyDots`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyDotsEnv {
    base: f64,
}

#[miniextendr(env, noexport)]
impl LazyDotsEnv {
    /// Create the fixture.
    /// @param base A number added by `sum`.
    pub fn new(base: f64) -> Self {
        Self { base }
    }

    /// Describe the dots, forcing nothing.
    /// @param ... Anything; nothing is evaluated.
    pub fn report(&self, rest: LazyDots) -> Vec<String> {
        describe(&rest)
    }

    /// `base` plus the elements that are not empty.
    /// @param ... Numbers; empty ones are skipped.
    pub fn sum(&self, rest: LazyDots) -> f64 {
        self.base + sum_given(&rest)
    }
}

/// R6 class whose methods take `LazyDots`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyDotsR6 {
    base: f64,
}

#[miniextendr(r6, noexport)]
impl LazyDotsR6 {
    /// Create the fixture.
    /// @param base A number added by `sum`.
    pub fn new(base: f64) -> Self {
        Self { base }
    }

    /// Describe the dots, forcing nothing.
    /// @param ... Anything; nothing is evaluated.
    pub fn report(&self, rest: LazyDots) -> Vec<String> {
        describe(&rest)
    }

    /// `base` plus the elements that are not empty.
    /// @param ... Numbers; empty ones are skipped.
    pub fn sum(&self, rest: LazyDots) -> f64 {
        self.base + sum_given(&rest)
    }
}

/// S3 class whose methods take `LazyDots`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyDotsS3 {
    base: f64,
}

/// S3 class whose methods take `LazyDots`. Exported: roxygen2 wants every S3
/// method exported or registered.
#[miniextendr(s3)]
impl LazyDotsS3 {
    /// Create the fixture.
    /// @param base A number added by `lazy_s3_sum`.
    pub fn new(base: f64) -> Self {
        Self { base }
    }

    /// Describe the dots, forcing nothing.
    /// @param ... Anything; nothing is evaluated.
    pub fn lazy_s3_report(&self, rest: LazyDots) -> Vec<String> {
        describe(&rest)
    }

    /// `base` plus the elements that are not empty.
    /// @param ... Numbers; empty ones are skipped.
    pub fn lazy_s3_sum(&self, rest: LazyDots) -> f64 {
        self.base + sum_given(&rest)
    }
}

/// S4 class whose methods take `LazyDots`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyDotsS4 {
    base: f64,
}

#[miniextendr(s4, noexport)]
impl LazyDotsS4 {
    /// Create the fixture.
    /// @param base A number added by `s4_lazy_sum`.
    pub fn new(base: f64) -> Self {
        Self { base }
    }

    /// Describe the dots, forcing nothing. A formal besides the generic's
    /// (`n`), so the method goes through S4's `.local` rewrite.
    /// @param n A number, reported first.
    /// @param ... Anything; nothing is evaluated.
    pub fn lazy_report(&self, n: i32, rest: LazyDots) -> Vec<String> {
        let mut out = vec![format!("n={n}")];
        out.extend(describe(&rest));
        out
    }

    /// `base` plus the elements that are not empty.
    /// @param ... Numbers; empty ones are skipped.
    pub fn lazy_sum(&self, rest: LazyDots) -> f64 {
        self.base + sum_given(&rest)
    }
}

/// S7 class whose methods take `LazyDots`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyDotsS7 {
    base: f64,
}

#[miniextendr(s7, noexport)]
impl LazyDotsS7 {
    /// Create the fixture.
    /// @param base A number added by `lazy_s7_sum`.
    pub fn new(base: f64) -> Self {
        Self { base }
    }

    /// Describe the dots, forcing nothing.
    /// @param ... Anything; nothing is evaluated.
    pub fn lazy_s7_report(&self, rest: LazyDots) -> Vec<String> {
        describe(&rest)
    }

    /// `base` plus the elements that are not empty.
    /// @param ... Numbers; empty ones are skipped.
    pub fn lazy_s7_sum(&self, rest: LazyDots) -> f64 {
        self.base + sum_given(&rest)
    }
}

/// vctrs class: vctrs impls take no instance methods (MXL120), so `LazyDots`
/// sits on a static helper and on the `format()` protocol method.
pub struct LazyDotsVctrs;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "ldv"), noexport)]
impl LazyDotsVctrs {
    /// Create the fixture.
    /// @param values Numeric payload.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(values: Vec<f64>) -> Vec<f64> {
        values
    }

    /// Describe the dots, forcing nothing.
    /// @param ... Anything; nothing is evaluated.
    pub fn report(rest: LazyDots) -> Vec<String> {
        describe(&rest)
    }

    /// `format()` that tags each value with the dots it got, forcing none of
    /// them: `<value>|<descriptions>`.
    /// @param x The vctrs payload.
    /// @param ... Anything; nothing is evaluated.
    #[miniextendr(vctrs(format))]
    pub fn format_lazy_dots(x: Vec<f64>, rest: LazyDots) -> Vec<String> {
        let tag = describe(&rest).join(",");
        x.iter().map(|v| format!("{v}|{tag}")).collect()
    }
}

// endregion
