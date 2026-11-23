//! Classes renamed with `class = "..."`, reached from other expansions
//! (#2265, #2266): a free function's `WrapAs*` return and a trait impl name
//! the class by its R name, resolved at write time through `MX_CLASS_NAMES`.
//! So do the names a trait impl builds from the class (#2285): the S4 trait
//! statics (`RenamedShape_Area_unit()`), the S7 trait shortcuts
//! (`RenamedPoint_area()`) and the S7 trait namespace (`.RenamedPoint__Area`).

use crate::class_system_matrix::MatrixCounter;
use miniextendr_api::{WrapAsR6, WrapAsS3, WrapAsS7, miniextendr};

/// The area of a shape, implemented on the renamed S4 and S7 classes below
/// (#2285).
#[miniextendr]
pub trait Area {
    fn area(&self) -> f64;
    fn unit() -> String;
}

// region: S3

/// A tally whose S3 class is `renamed_tally`, not its Rust name.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RenamedTallyS3 {
    value: i32,
}

/// A tally whose S3 class is `renamed_tally`, not its Rust name.
#[miniextendr(s3, class = "renamed_tally")]
impl RenamedTallyS3 {
    /// @param value Initial tally.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

// The methods dispatch on `renamed_tally` and the static lives in
// `renamed_tally$MatrixCounter`. Their docs still land on a page named after
// the Rust type (#2144).
#[miniextendr(s3)]
impl MatrixCounter for RenamedTallyS3 {
    fn custom_get(&self) -> i32 {
        self.value
    }
    /// @param n Amount to add.
    fn custom_add(&mut self, n: i32) {
        self.value += n;
    }
    fn default_value() -> i32 {
        11
    }
}

/// A free function returning the renamed S3 class through its marker.
/// @param value Initial tally.
#[miniextendr]
pub fn renamed_tally_wrapped(value: i32) -> WrapAsS3<RenamedTallyS3> {
    WrapAsS3(RenamedTallyS3::new(value))
}

/// The attribute spelling of `renamed_tally_wrapped()`.
/// @param value Initial tally.
#[miniextendr(wrap = "s3")]
pub fn renamed_tally_wrapped_attr(value: i32) -> RenamedTallyS3 {
    RenamedTallyS3::new(value)
}
// endregion

// region: R6

/// A board whose R6 class is `RenamedBoard`, not its Rust name.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RenamedBoardR6 {
    width: i32,
}

/// A board whose R6 class is `RenamedBoard`, not its Rust name.
#[miniextendr(r6, class = "RenamedBoard")]
impl RenamedBoardR6 {
    /// @param width Board width.
    pub fn new(width: i32) -> Self {
        Self { width }
    }
    /// The board width.
    pub fn width(&self) -> i32 {
        self.width
    }
}

// The trait namespace is `RenamedBoard$MatrixCounter`.
#[miniextendr(r6)]
impl MatrixCounter for RenamedBoardR6 {
    fn custom_get(&self) -> i32 {
        self.width
    }
    fn custom_add(&mut self, n: i32) {
        self.width += n;
    }
    fn default_value() -> i32 {
        12
    }
}

/// A free function returning the renamed R6 class through its marker.
/// @param width Board width.
#[miniextendr]
pub fn renamed_board_wrapped(width: i32) -> WrapAsR6<RenamedBoardR6> {
    WrapAsR6(RenamedBoardR6::new(width))
}
// endregion

// region: S7

/// A point whose S7 class is `RenamedPoint`, not its Rust name.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RenamedPointS7 {
    x: f64,
}

/// A point whose S7 class is `RenamedPoint`, not its Rust name.
#[miniextendr(s7, class = "RenamedPoint")]
impl RenamedPointS7 {
    /// @param x The x coordinate.
    pub fn new(x: f64) -> Self {
        Self { x }
    }
    /// The x coordinate.
    pub fn renamed_point_x(&self) -> f64 {
        self.x
    }
}

/// A free function returning the renamed S7 class through its marker.
/// @param x The x coordinate.
#[miniextendr]
pub fn renamed_point_wrapped(x: f64) -> WrapAsS7<RenamedPointS7> {
    WrapAsS7(RenamedPointS7::new(x))
}

// The shortcut is `RenamedPoint_area()` and the static lives in
// `attr(RenamedPoint, "Area")`, gathered in `.RenamedPoint__Area`.
#[miniextendr(s7)]
impl Area for RenamedPointS7 {
    fn area(&self) -> f64 {
        self.x * self.x
    }
    fn unit() -> String {
        "point".to_string()
    }
}
// endregion

// region: S4

/// A shape whose S4 class is `RenamedShape`, not its Rust name.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RenamedShapeS4 {
    side: f64,
}

/// A shape whose S4 class is `RenamedShape`, not its Rust name.
#[miniextendr(s4, class = "RenamedShape")]
impl RenamedShapeS4 {
    /// @param side Side length.
    pub fn new(side: f64) -> Self {
        Self { side }
    }
}

// The method dispatches on `RenamedShape` and the static is
// `RenamedShape_Area_unit()`.
#[miniextendr(s4)]
impl Area for RenamedShapeS4 {
    fn area(&self) -> f64 {
        self.side * self.side
    }
    fn unit() -> String {
        "shape".to_string()
    }
}
// endregion
