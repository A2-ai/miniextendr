//! The dots left unforced: [`LazyDots`].
//!
//! A `&Dots` parameter receives `list(...)`, which the generated wrapper
//! evaluates before the Rust body runs: every element is forced, and an empty
//! argument (`f(a = )`, `x[1, , , ]`) stops the call with R's "argument is
//! missing, with no default". A parameter typed `LazyDots` is R's `...` too,
//! but the wrapper passes its own frame, `environment()`, in place of
//! `list(...)`, and nothing is evaluated on the way in. The body counts and
//! names the dots, sees which are empty, reads what was written, and forces
//! the elements it wants, each in its own environment.
//!
//! ```ignore
//! use miniextendr_api::{LazyDots, SEXP, miniextendr};
//!
//! /// `subset()` for whole rows only: anything in `...` is refused before it
//! /// is evaluated, so `subset(x, TRUE, select = ID)` reports `select`, not
//! /// "object 'ID' not found".
//! #[miniextendr(s3(generic = "subset", class = "my_tbl"))]
//! pub fn my_tbl_subset(x: SEXP, subset: SEXP, rest: LazyDots) -> Result<SEXP, String> {
//!     if !rest.is_empty() {
//!         let names: Vec<_> = rest.names().into_iter().map(|n| n.unwrap_or("")).collect();
//!         return Err(format!("unsupported arguments: {}", names.join(", ")));
//!     }
//!     // ...
//! #   Ok(x)
//! }
//! ```
//!
//! | method | R equivalent | forces |
//! |---|---|---|
//! | [`len`](LazyDots::len) / [`is_empty`](LazyDots::is_empty) | `...length()` | nothing |
//! | [`names`](LazyDots::names) | `...names()` | nothing |
//! | [`is_missing_arg(i)`](LazyDots::is_missing_arg) | element `i` of `substitute(list(...))` is the empty argument | nothing |
//! | [`expr(i)`](LazyDots::expr) | element `i` of `substitute(list(...))` | nothing |
//! | [`force(i)`](LazyDots::force) | `..k` (`k = i + 1`) | element `i` |
//! | [`try_force(i)`](LazyDots::try_force) | `..k`, with an R error returned as `Err` | element `i` |
//!
//! Indices are 0-based, as in [`List::get_index`](crate::List::get_index):
//! element `i` is R's `..k` with `k = i + 1`. An index past the end panics
//! before anything is read or forced for it, as slice indexing does.
//!
//! # Forwarded dots
//!
//! Each element keeps the environment it was written in, also when another
//! function passed its own dots on (`g <- function(...) f(x, ...)`):
//! `force(i)` evaluates `..k` in the wrapper's frame, which forces the
//! element's promise in that environment, and `expr(i)` gives what the caller
//! of `g` wrote.
//!
//! # Empty and missing elements
//!
//! [`is_missing_arg(i)`](LazyDots::is_missing_arg) is `true` only for a
//! literally empty slot, the element R stores as the missing argument:
//! `f(a = )`, the fourth position of `x[1, , , ]`. That is what R 4.6's
//! `R_GetDotType()` reports as missing.
//!
//! An element forwarded from a missing argument is not empty: in
//! `g <- function(a) f(1, a)` called as `g()`, element 1 is the promise of
//! `a`, so `is_missing_arg(1)` is `false`. Forcing it raises R's own error,
//! `argument "a" is missing, with no default`, as `list(...)` would.
//!
//! Forcing an empty element raises R's error for it,
//! `argument "..k" is missing, with no default` (class `missingArgError`
//! from R 4.6, `simpleError` before), whose call is the generated function's
//! own call. A body that accepts empty arguments checks `is_missing_arg(i)`
//! before forcing.
//!
//! # Conditions
//!
//! [`force`](LazyDots::force) evaluates through
//! [`eval_with_handlers`](crate::expression::eval_with_handlers): the caller's
//! `withCallingHandlers()` / `suppressWarnings()` see what forcing signals,
//! and an error, or the exit of a `tryCatch()` handler, unwinds through the
//! Rust frames (running their destructors) and carries on in R as the
//! original condition. [`try_force`](LazyDots::try_force) evaluates through
//! [`try_eval_with_handlers`](crate::expression::try_eval_with_handlers):
//! warnings and messages still reach the caller's handlers, and an R error
//! comes back as `Err(`[`REvalError`]`)` for the body to handle. The calls
//! behind `len`, `names`, `is_missing_arg` and `expr` evaluate through
//! `eval_with_handlers` too, never a bare `Rf_eval`.
//!
//! # Caching
//!
//! - **Forced values are not cached in Rust.** R stores a promise's value in
//!   the promise, so forcing an element twice evaluates it once and the
//!   second `force(i)` returns the stored value.
//! - **The count and the expressions are read once per value.** The first
//!   call that needs them evaluates `...length()` or `substitute(list(...))`,
//!   and the value keeps the result, so a loop over the elements copies the
//!   expressions once.
//!
//! # Restrictions
//!
//! - Main thread only: the type is `!Send` and borrows the `.Call()`
//!   argument for the call.
//! - It takes no per-parameter option (`default`, `coerce`, `match_arg`,
//!   `no_na`, ...) and no `dots = typed_list!(..)`, which validates a forced
//!   list. `&LazyDots`, `Option<LazyDots>` and `Missing<LazyDots>` are compile
//!   errors: the parameter is always present, taken by value.
//! - A function takes at most one `...`: `&Dots` or `LazyDots`, not both.
//! - Standalone functions, `s3(...)` functions, and the impl-block methods of
//!   every class system (env, R6, S3, S4, S7, vctrs) accept it. Trait methods
//!   and `extern "C-unwind"` functions do not.
//!
//! # GC
//!
//! The frame is a `.Call()` argument, which R protects for the call, and the
//! dots are its `...` binding: the promises and the values they cache stay
//! reachable until the call returns. The list `substitute(list(...))` gives
//! is rooted by the `LazyDots` value (`R_PreserveObject`) and released when it
//! drops.

use std::cell::{Cell, OnceCell};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use crate::expression::{REvalError, eval_with_handlers, try_eval_with_handlers};
use crate::from_r::charsxp_to_str;
use crate::gc_protect::OwnedProtect;
use crate::sexp_ext::PairListExt;
use crate::{SEXP, SEXPTYPE, SexpExt};

/// The expressions `substitute(list(...))` gives in the frame, read once.
struct Exprs {
    /// The `list(...)` call, rooted with `R_PreserveObject` until the
    /// [`LazyDots`] drops.
    root: SEXP,
    /// Its arguments, one per element; rooted by `root`.
    elts: Vec<SEXP>,
}

/// R's `...`, passed unforced: the generated wrapper's own frame, whose `...`
/// binding holds the dots as R matched them.
///
/// Take it as a `#[miniextendr]` parameter, at the position of `...`:
/// `fn f(x: SEXP, rest: LazyDots)` becomes `f <- function(x, ...)`, and the
/// wrapper passes `environment()` where a `&Dots` parameter's passes
/// `list(...)`. See the [module docs](self).
///
/// ```ignore
/// use miniextendr_api::{LazyDots, SEXP, miniextendr};
///
/// /// `update()` with a step per named argument; an empty `select = ` runs the
/// /// step without its argument.
/// #[miniextendr(s3(generic = "update", class = "my_obj"))]
/// pub fn my_obj_update(object: SEXP, rest: LazyDots) -> SEXP {
///     for (i, name) in rest.names().into_iter().enumerate() {
///         let arg = if rest.is_missing_arg(i) { None } else { Some(rest.force(i)) };
///         // ... run step `name` with `arg`
/// #       let _ = (name, arg);
///     }
///     object
/// }
/// ```
pub struct LazyDots<'a> {
    /// The generated wrapper's frame (`environment()`).
    env: SEXP,
    /// `...length()`, read once.
    len: Cell<Option<usize>>,
    /// `substitute(list(...))`, read once.
    exprs: OnceCell<Exprs>,
    /// Borrows the `.Call()` argument that roots the frame.
    _arg: PhantomData<&'a SEXP>,
    /// R objects stay on R's main thread.
    _not_send: PhantomData<*const ()>,
}

impl<'a> LazyDots<'a> {
    /// Read the argument a generated wrapper passed for a `LazyDots`
    /// parameter: its own frame. Generated code calls this; user code has no
    /// reason to.
    ///
    /// # Panics
    ///
    /// When `arg` is not an environment.
    ///
    /// # Safety
    ///
    /// R's main thread; `arg` is the frame of a function with a `...` formal
    /// and stays rooted for `'a`.
    #[doc(hidden)]
    pub unsafe fn from_wrapper_arg(arg: &'a SEXP) -> Self {
        let env = *arg;
        assert!(
            env.is_environment(),
            "a `LazyDots` argument is passed as `environment()`, got {}",
            env.type_of().type_name(),
        );
        LazyDots {
            env,
            len: Cell::new(None),
            exprs: OnceCell::new(),
            _arg: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// The number of elements in `...` (`...length()`). Empty elements count.
    /// Forces nothing.
    pub fn len(&self) -> usize {
        if let Some(n) = self.len.get() {
            return n;
        }
        // SAFETY: a `LazyDots` exists only inside the `#[miniextendr]` call
        // that received it, on R's main thread; the frame is rooted by it.
        let n = unsafe {
            let n = OwnedProtect::new(self.call_in_frame(c"...length", None));
            n.get()
                .as_integer()
                .and_then(|n| usize::try_from(n).ok())
                .expect("`...length()` is a non-negative integer")
        };
        self.len.set(Some(n));
        n
    }

    /// Whether `...` holds no elements (`...length() == 0`).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The name of each element, as written (`...names()`): `None` for an
    /// unnamed one, every entry `None` when nothing is named. A repeated name
    /// is kept (`f(a = 1, a = 2)` gives `a`, `a`). Forces nothing.
    ///
    /// The names are the print names of the argument tags, which R keeps for
    /// the session.
    pub fn names(&self) -> Vec<Option<&'a str>> {
        let n = self.len();
        // SAFETY: as in `len`. The strings are symbol print names (or the
        // empty string), which R never collects.
        unsafe {
            let names = OwnedProtect::new(self.call_in_frame(c"...names", None));
            let names = names.get();
            if names.type_of() != SEXPTYPE::STRSXP {
                return vec![None; n];
            }
            (0..n)
                .map(|i| {
                    let elt = names.string_elt(r_index(i));
                    if elt.is_na_string() {
                        return None;
                    }
                    let name = charsxp_to_str(elt);
                    (!name.is_empty()).then_some(name)
                })
                .collect()
        }
    }

    /// Whether element `i` is empty: the slot of `f(a = )` or of the fourth
    /// position of `x[1, , , ]`, which R stores as the missing argument.
    /// Forces nothing.
    ///
    /// Only a literally empty slot counts. An element forwarded from a
    /// missing argument (`g <- function(a) f(1, a)` called as `g()`) is the
    /// promise of `a`, so this is `false`; forcing it raises
    /// `argument "a" is missing, with no default`.
    ///
    /// The first call reads `substitute(list(...))` in the frame; the value
    /// keeps it, so a loop over the elements reads it once.
    ///
    /// # Panics
    ///
    /// When `i >= self.len()`.
    pub fn is_missing_arg(&self, i: usize) -> bool {
        self.expr(i) == SEXP::missing_arg()
    }

    /// The expression element `i` was written as, as `substitute(list(...))`
    /// gives it: a promise's expression (for a forwarded element, what the
    /// caller of the forwarding function wrote), a constant as it is, and the
    /// empty symbol (`quote(expr = )`) for an empty element. Forces nothing.
    ///
    /// The expression is rooted by this value: it stays valid until the
    /// `LazyDots` drops. The first call reads `substitute(list(...))` in the
    /// frame; later calls reuse it.
    ///
    /// # Panics
    ///
    /// When `i >= self.len()`.
    pub fn expr(&self, i: usize) -> SEXP {
        self.check_index(i);
        self.exprs().elts[i]
    }

    /// Force element `i` and return its value, as `..k` (`k = i + 1`) gives it
    /// in the frame: a promise is forced in its own environment, and a
    /// constant comes back as it is.
    ///
    /// Evaluated with
    /// [`eval_with_handlers`](crate::expression::eval_with_handlers): the
    /// caller's handlers see what forcing signals, and an R error unwinds as
    /// the original condition. An empty element raises
    /// `argument "..k" is missing, with no default`; check
    /// [`Self::is_missing_arg`] first to accept one.
    ///
    /// R caches a promise's value, so forcing an element again returns the
    /// stored value without evaluating anything. The value is reachable from
    /// the frame (the promise, or the `...` binding itself) until the call
    /// returns.
    ///
    /// # Panics
    ///
    /// When `i >= self.len()`, before anything is forced.
    pub fn force(&self, i: usize) -> SEXP {
        self.check_index(i);
        // SAFETY: as in `len`; the symbol needs no root.
        unsafe { eval_with_handlers(dot_symbol(i), self.env) }
    }

    /// [`Self::force`], but an R error raised while forcing comes back as
    /// `Err(`[`REvalError`]`)` instead of unwinding.
    ///
    /// Evaluated with
    /// [`try_eval_with_handlers`](crate::expression::try_eval_with_handlers):
    /// warnings and messages still reach the caller's handlers, and any other
    /// exit (a `tryCatch()` handler's, a restart) unwinds as under
    /// [`Self::force`]. An empty element is an `Err` with R's
    /// `argument "..k" is missing, with no default`.
    ///
    /// # Panics
    ///
    /// When `i >= self.len()`, before anything is forced.
    pub fn try_force(&self, i: usize) -> Result<SEXP, REvalError> {
        self.check_index(i);
        // SAFETY: as in `force`.
        unsafe { try_eval_with_handlers(dot_symbol(i), self.env) }
    }

    /// Panic on an index past the end, as slice indexing does.
    fn check_index(&self, i: usize) {
        let len = self.len();
        assert!(
            i < len,
            "index out of bounds: `...` has {len} element{} but the index is {i}",
            if len == 1 { "" } else { "s" },
        );
    }

    /// `substitute(list(...))` in the frame, read on first use and rooted
    /// until the value drops.
    fn exprs(&self) -> &Exprs {
        self.exprs.get_or_init(|| {
            // SAFETY: as in `len`. The list is rooted before `R_PreserveObject`
            // allocates, then by the preserve until `Drop`.
            unsafe {
                let list_call = OwnedProtect::new(crate::sys::Rf_lang2(
                    crate::sys::Rf_install(c"list".as_ptr()),
                    crate::sys::Rf_install(c"...".as_ptr()),
                ));
                let list =
                    OwnedProtect::new(self.call_in_frame(c"substitute", Some(list_call.get())));
                crate::sys::R_PreserveObject(list.get());
                let mut elts = Vec::with_capacity(self.len());
                let mut node = list.get().cdr();
                while node.type_of() != SEXPTYPE::NILSXP {
                    elts.push(node.car());
                    node = node.cdr();
                }
                debug_assert_eq!(elts.len(), self.len());
                Exprs {
                    root: list.get(),
                    elts,
                }
            }
        })
    }

    /// Evaluate `fun()` or `fun(arg)` in the frame, `fun` the base function of
    /// that name. The call's head is the function itself, looked up in the
    /// base environment, so no formal or variable named like it can shadow
    /// it; `arg` is passed as written (the functions used here either take no
    /// argument or don't evaluate it).
    ///
    /// # Safety
    ///
    /// R's main thread, inside the `#[miniextendr]` call; `arg` is rooted.
    /// The result is unprotected.
    unsafe fn call_in_frame(&self, fun: &CStr, arg: Option<SEXP>) -> SEXP {
        unsafe {
            // A base binding: base keeps the function for the session.
            let fun =
                eval_with_handlers(crate::sys::Rf_install(fun.as_ptr()), crate::sys::R_BaseEnv);
            let call = OwnedProtect::new(match arg {
                None => crate::sys::Rf_lang1(fun),
                Some(arg) => crate::sys::Rf_lang2(fun, arg),
            });
            eval_with_handlers(call.get(), self.env)
        }
    }
}

impl Drop for LazyDots<'_> {
    fn drop(&mut self) {
        if let Some(exprs) = self.exprs.get() {
            // SAFETY: the value never left R's main thread (`!Send`).
            unsafe { crate::sys::R_ReleaseObject(exprs.root) };
        }
    }
}

impl std::fmt::Debug for LazyDots<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyDots")
            .field("len", &self.len.get())
            .finish_non_exhaustive()
    }
}

/// The symbol `..k` for 0-based element `i` (`k = i + 1`).
///
/// # Safety
///
/// R's main thread. Symbols are never collected.
unsafe fn dot_symbol(i: usize) -> SEXP {
    let name = CString::new(format!("..{}", i + 1)).expect("no NUL in `..k`");
    unsafe { crate::sys::Rf_install(name.as_ptr()) }
}

/// An R vector index.
fn r_index(i: usize) -> isize {
    isize::try_from(i).expect("an R vector index fits in isize")
}
