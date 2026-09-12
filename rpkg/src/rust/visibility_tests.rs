//! Tests for R return value visibility (visible/invisible returns).

use miniextendr_api::miniextendr;

#[miniextendr]
/// @title Visibility and Interrupt Tests
/// @name rpkg_visibility_interrupts
/// @description Visibility and interrupt checks
/// @examples
/// invisibly_return_no_arrow()
/// force_invisible_i32()
/// with_interrupt_check(2L)
/// try(invisibly_option_return_none())
/// \dontrun{
/// unsafe_C_check_interupt_after()
/// }
/// @aliases invisibly_return_no_arrow invisibly_return_arrow invisibly_option_return_none
///   invisibly_option_return_some invisibly_result_return_ok force_invisible_i32 force_visible_unit
///   with_interrupt_check unsafe_C_check_interupt_after unsafe_C_check_interupt_unwind
pub fn invisibly_return_no_arrow() {}

/// Test that a function with explicit -> () return is invisible in R.
#[miniextendr]
#[allow(clippy::unused_unit)]
pub fn invisibly_return_arrow() -> () {}

/// Test that returning None from Option<()> produces an R error.
#[miniextendr]
pub fn invisibly_option_return_none() -> Option<()> {
    None // expectation: error!
}

/// Test that returning Some(()) from Option<()> is invisible in R.
#[miniextendr]
pub fn invisibly_option_return_some() -> Option<()> {
    Some(())
}

/// Test that returning Ok(()) from Result<(), ()> is invisible in R.
#[miniextendr]
#[allow(clippy::result_unit_err)]
pub fn invisibly_result_return_ok() -> Result<(), ()> {
    Ok(())
}

/// Test that Result Err(()) returns NULL in R, Ok returns doubled value.
/// @param x Integer input (negative triggers Err).
#[miniextendr]
#[allow(clippy::result_unit_err)]
pub fn result_null_on_err(x: i32) -> Result<i32, ()> {
    if x >= 0 {
        Ok(x * 2)
    } else {
        Err(()) // Should return NULL
    }
}

// Test explicit invisible attribute (force i32 return to be invisible)
/// Test that #[miniextendr(invisible)] forces an i32 return to be invisible in R.
#[miniextendr(invisible)]
pub fn force_invisible_i32() -> i32 {
    42
}

// Test explicit visible attribute (force () return to be visible)
/// Test that #[miniextendr(visible)] forces a unit return to be visible in R.
#[miniextendr(visible)]
pub fn force_visible_unit() {}

// Test check_interrupt attribute - checks for Ctrl+C before executing
/// Test that `#[miniextendr(check_interrupt)]` checks for user interrupts before execution.
/// @param x Integer scalar input.
#[miniextendr(check_interrupt)]
pub fn with_interrupt_check(x: i32) -> i32 {
    x * 2
}

/// Test `unwrap_in_r`: the `Result` is converted to `list(error = ...)` on the
/// R side instead of raising a condition.
/// @param x Integer input (negative triggers Err with message).
#[miniextendr(unwrap_in_r)]
pub fn result_unwrap_in_r(x: i32) -> Result<i32, String> {
    if x >= 0 {
        Ok(x * 2)
    } else {
        Err(format!("negative input: {}", x))
    }
}

// region: Visibility markers (#1213)
//
// Nothing is invisible unless it says so, except a bare fn whose R value is
// `NULL`. `Invisible<T>` / `Visible<T>` and the `invisible` / `visible`
// attribute are two spellings of one decision; the receiver a method hands
// back for chaining is visible by default.

use miniextendr_api::{Invisible, Visible};

/// `Invisible<i32>`: the value is returned invisibly.
#[miniextendr]
pub fn marker_invisible_i32() -> Invisible<i32> {
    Invisible(7)
}

/// `Invisible<()>`: the explicit spelling of the unit default.
#[miniextendr]
pub fn marker_invisible_unit() -> Invisible<()> {
    Invisible(())
}

/// `Visible<()>`: a visible `NULL`.
#[miniextendr]
pub fn marker_visible_unit() -> Visible<()> {
    Visible(())
}

/// The marker is transparent to `Option` handling: a bare function's
/// `Option<i32>` `None` is `NA`, as without the marker.
/// @param present Whether a value is returned.
#[miniextendr]
pub fn marker_invisible_option(present: bool) -> Invisible<Option<i32>> {
    Invisible(present.then_some(3))
}

/// A marker and an agreeing attribute.
#[miniextendr(invisible)]
pub fn marker_and_attr_agree() -> Invisible<i32> {
    Invisible(1)
}

/// R6 handle: receiver-returning tails are visible unless marked.
#[derive(miniextendr_api::ExternalPtr)]
pub struct VisibilityCounter {
    n: i32,
}

#[miniextendr(r6)]
impl VisibilityCounter {
    /// A counter starting at zero.
    pub fn new() -> Self {
        VisibilityCounter { n: 0 }
    }

    /// Unmarked `&mut self -> ()`: chainable, returns `self` visibly.
    pub fn tick(&mut self) {
        self.n += 1;
    }

    /// `Invisible<()>`: chainable and silent.
    pub fn tick_quietly(&mut self) -> Invisible<()> {
        self.n += 1;
        Invisible(())
    }

    /// The `r6(invisible)` option, same as `Invisible<()>`.
    #[miniextendr(r6(invisible))]
    pub fn tick_attr(&mut self) {
        self.n += 1;
    }

    /// Unmarked self-ref builder: visible `self`.
    /// @param k Amount to add.
    pub fn add(&mut self, k: i32) -> &mut Self {
        self.n += k;
        self
    }

    /// Marked self-ref builder: `invisible(self)`.
    /// @param k Amount to add.
    pub fn add_quietly(&mut self, k: i32) -> Invisible<&mut Self> {
        self.n += k;
        Invisible(self)
    }

    /// `Invisible<i32>` on a value method.
    pub fn peek(&self) -> Invisible<i32> {
        Invisible(self.n)
    }

    /// `Invisible<Self>`: the classed copy is returned invisibly.
    pub fn snapshot(&self) -> Invisible<Self> {
        Invisible(VisibilityCounter { n: self.n })
    }

    /// The current count.
    pub fn value(&self) -> i32 {
        self.n
    }
}

/// S3 handle: void methods hand back `x`, visibly unless marked.
#[derive(miniextendr_api::ExternalPtr)]
pub struct VisibilityGauge {
    level: i32,
}

#[miniextendr(s3)]
impl VisibilityGauge {
    /// A gauge at zero.
    pub fn new() -> Self {
        VisibilityGauge { level: 0 }
    }

    /// Unmarked void method: returns `x` visibly.
    pub fn nudge_gauge(&mut self) {
        self.level += 1;
    }

    /// Marked void method: `invisible(x)`.
    pub fn quiet_nudge_gauge(&mut self) -> Invisible<()> {
        self.level += 1;
        Invisible(())
    }

    /// Marked value method: the level, invisibly.
    pub fn gauge_level(&self) -> Invisible<i32> {
        Invisible(self.level)
    }
}

/// Trait with one marked void method: the marker lives in the declaration
/// and the implementing type's R wrapper honours it.
#[miniextendr]
pub trait QuietBell {
    /// Ring: an unmarked void trait method returns the receiver visibly.
    fn ring(&mut self);
    /// Ring silently: `Invisible<()>`.
    fn ring_quietly(&mut self) -> Invisible<()>;
    /// Rings so far, returned invisibly.
    fn rings_quietly(&self) -> Invisible<i32>;
}

/// Env-class bell implementing [`QuietBell`].
#[derive(miniextendr_api::ExternalPtr)]
pub struct Bell {
    rings: i32,
}

#[miniextendr(env)]
impl Bell {
    /// A bell that has not rung yet.
    pub fn new() -> Self {
        Bell { rings: 0 }
    }

    /// Rings so far.
    pub fn rings(&self) -> i32 {
        self.rings
    }
}

#[miniextendr(env)]
impl QuietBell for Bell {
    fn ring(&mut self) {
        self.rings += 1;
    }

    fn ring_quietly(&mut self) -> Invisible<()> {
        self.rings += 1;
        Invisible(())
    }

    fn rings_quietly(&self) -> Invisible<i32> {
        Invisible(self.rings)
    }
}

// endregion
