// Producer package: Creates objects with various class systems for cross-package testing
//
// This demonstrates cross-package interoperability:
// - Env-style (default): SharedData, EnvPoint
// - R6-style: R6Point
// - S3-style: S3Point
// - S4-style: S4Point
// - S7-style: S7Point
// - Trait dispatch: SimpleCounter implements shared Counter trait

use miniextendr_api::{ExternalPtr, SEXP, miniextendr, trait_abi::ccall};
// Condition macros — use fully-qualified paths to avoid module/macro name collision
// (pub mod error and pub mod condition at crate root shadow the macros if imported).

miniextendr_api::miniextendr_init!();

// Import the shared Counter trait and its generated ABI types
pub use shared_traits::{__counter_build_vtable, Counter, CounterVTable, CounterView, TAG_COUNTER};

// Import the shared Resettable trait and its generated ABI types
pub use shared_traits::{
    __resettable_build_vtable, Resettable, ResettableVTable, ResettableView, TAG_RESETTABLE,
};

// region: Env-style types (default class system)

/// A simple data container for testing cross-package ExternalPtr dispatch.
/// Uses the default Env-style class system.
#[derive(ExternalPtr)]
pub struct SharedData {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

/// @title SharedData (Env-style)
/// @name SharedData
/// @description A point with label using Env-style class system.
#[miniextendr]
impl SharedData {
    /// Create a new SharedData
    fn create(x: f64, y: f64, label: String) -> Self {
        Self { x, y, label }
    }

    /// Get the x coordinate
    fn get_x(&self) -> f64 {
        self.x
    }

    /// Get the y coordinate
    fn get_y(&self) -> f64 {
        self.y
    }

    /// Get the label
    fn get_label(&self) -> String {
        self.label.clone()
    }
}

/// A 2D point using Env-style class system (default).
#[derive(ExternalPtr)]
pub struct EnvPoint {
    x: f64,
    y: f64,
}

/// @title EnvPoint (Env-style)
/// @name EnvPoint
/// @description A 2D point using the default Env-style class system.
#[miniextendr]
impl EnvPoint {
    /// Create a new point
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Get x coordinate
    fn x(&self) -> f64 {
        self.x
    }

    /// Get y coordinate
    fn y(&self) -> f64 {
        self.y
    }

    /// Calculate distance from origin
    fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Add another point's coordinates
    fn add(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}
// endregion

// region: R6-style type

/// A 2D point using R6-style class system.
#[derive(ExternalPtr)]
pub struct R6Point {
    x: f64,
    y: f64,
}

/// @title R6Point (R6-style)
/// @name R6Point
/// @description A 2D point using R6-style class system.
#[miniextendr(r6)]
impl R6Point {
    /// Create a new point
    /// @param x X coordinate.
    /// @param y Y coordinate.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Get x coordinate
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Get y coordinate
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Calculate distance from origin
    pub fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Add to coordinates
    /// @param dx Amount to add to x.
    /// @param dy Amount to add to y.
    pub fn add(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}
// endregion

// region: S3-style type

/// A 2D point using S3-style class system.
#[derive(ExternalPtr)]
pub struct S3Point {
    x: f64,
    y: f64,
}

/// @title S3Point (S3-style)
/// @name S3Point
/// @description A 2D point using S3-style class system.
/// @aliases new_s3point s3point_x s3point_y s3point_distance
#[miniextendr(s3)]
impl S3Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Get x coordinate
    pub fn s3point_x(&self) -> f64 {
        self.x
    }

    /// Get y coordinate
    pub fn s3point_y(&self) -> f64 {
        self.y
    }

    /// Calculate distance from origin
    pub fn s3point_distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Add to coordinates
    pub fn s3point_add(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}
// endregion

// region: S4-style type

/// A 2D point using S4-style class system.
#[derive(ExternalPtr)]
pub struct S4Point {
    x: f64,
    y: f64,
}

/// @title S4Point (S4-style)
/// @name S4Point
/// @description A 2D point using S4-style class system.
/// @aliases S4Point s4point_x s4point_y s4point_distance
#[miniextendr(s4)]
impl S4Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Get x coordinate
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Get y coordinate
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Calculate distance from origin
    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Add to coordinates
    pub fn add(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}
// endregion

// region: S7-style type

/// A 2D point using S7-style class system.
#[derive(ExternalPtr)]
pub struct S7Point {
    x: f64,
    y: f64,
}

/// @title S7Point (S7-style)
/// @name S7Point
/// @description A 2D point using S7-style class system.
/// @aliases S7Point s7point_x s7point_y s7point_distance
#[miniextendr(s7)]
impl S7Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Get x coordinate
    pub fn s7point_x(&self) -> f64 {
        self.x
    }

    /// Get y coordinate
    pub fn s7point_y(&self) -> f64 {
        self.y
    }

    /// Calculate distance from origin
    pub fn s7point_distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Add to coordinates
    pub fn s7point_add(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}
// endregion

// region: SimpleCounter with trait dispatch (for cross-package trait testing)

#[derive(ExternalPtr)]
pub struct SimpleCounter {
    value: i32,
}

impl SimpleCounter {
    fn create(initial: i32) -> Self {
        Self { value: initial }
    }
}

#[miniextendr]
impl SimpleCounter {
    /// Get current value (inherent method)
    fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr]
impl Counter for SimpleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }

    /// Panic with a plain message so the consumer can test rust_error class layering.
    fn panic_plain(&self) {
        panic!(
            "SimpleCounter::panic_plain triggered (value={})",
            self.value
        );
    }

    /// Use error!() with a custom class so the consumer can test user-class layering.
    fn error_with_class(&self, class_name: String) {
        miniextendr_api::error!(
            class = class_name,
            "SimpleCounter::error_with_class triggered (value={})",
            self.value
        );
    }

    /// Bare error!() — no user class, so the consumer verifies plain rust_error.
    fn raise_error(&self, msg: String) {
        miniextendr_api::error!("{}", msg);
    }

    /// warning!() — consumer verifies rust_warning + e$kind round-trip.
    fn raise_warning(&self, msg: String) {
        miniextendr_api::warning!("{}", msg);
    }

    /// message!() — consumer verifies rust_message + e$kind round-trip.
    fn raise_message(&self, msg: String) {
        miniextendr_api::message!("{}", msg);
    }

    /// condition!() with a custom class — consumer verifies catchability + kind.
    fn raise_condition_classed(&self, class_name: String, msg: String) {
        miniextendr_api::condition!(class = class_name, "{}", msg);
    }

    /// Raise an error with structured data fields — tests ConditionData
    /// round-trip through from_tagged_sexp slot \[4\] (issue #996 path-1).
    fn raise_error_with_data(&self) {
        miniextendr_api::error!(
            class = "data_bearing_error",
            data = [
                ("field_a", self.value),
                ("field_b", "hello from producer"),
                ("flag", true),
                ("score", 1.23_f64)
            ],
            "error with structured data (value={})",
            self.value
        );
    }
}

#[miniextendr]
impl Resettable for SimpleCounter {
    fn reset(&mut self) {
        self.value = 0;
    }

    fn is_default(&self) -> bool {
        self.value == 0
    }
}
// endregion

// region: StatefulCounter: tracks history and implements both Counter and Resettable

#[derive(ExternalPtr)]
pub struct StatefulCounter {
    value: i32,
    history: Vec<i32>,
}

impl StatefulCounter {
    fn create(initial: i32) -> Self {
        Self {
            value: initial,
            history: vec![initial],
        }
    }
}

#[miniextendr]
impl StatefulCounter {
    /// Get current value (inherent method)
    fn get_value(&self) -> i32 {
        self.value
    }

    /// Get how many history entries exist
    fn history_len(&self) -> i32 {
        self.history.len() as i32
    }
}

#[miniextendr]
impl Counter for StatefulCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
        self.history.push(self.value);
    }

    fn add(&mut self, n: i32) {
        self.value += n;
        self.history.push(self.value);
    }

    fn panic_plain(&self) {
        panic!(
            "StatefulCounter::panic_plain triggered (value={})",
            self.value
        );
    }

    fn error_with_class(&self, class_name: String) {
        miniextendr_api::error!(
            class = class_name,
            "StatefulCounter::error_with_class triggered (value={})",
            self.value
        );
    }

    fn raise_error(&self, msg: String) {
        miniextendr_api::error!("{}", msg);
    }

    fn raise_warning(&self, msg: String) {
        miniextendr_api::warning!("{}", msg);
    }

    fn raise_message(&self, msg: String) {
        miniextendr_api::message!("{}", msg);
    }

    fn raise_condition_classed(&self, class_name: String, msg: String) {
        miniextendr_api::condition!(class = class_name, "{}", msg);
    }

    fn raise_error_with_data(&self) {
        miniextendr_api::error!(
            class = "data_bearing_error",
            data = [
                ("field_a", self.value),
                ("field_b", "hello from stateful producer"),
                ("flag", true),
                ("score", 1.23_f64)
            ],
            "stateful error with structured data (value={})",
            self.value
        );
    }
}

#[miniextendr]
impl Resettable for StatefulCounter {
    fn reset(&mut self) {
        self.value = 0;
        self.history.clear();
        self.history.push(0);
    }

    fn is_default(&self) -> bool {
        self.value == 0 && self.history.len() == 1
    }
}

/// Create a new StatefulCounter with cross-package trait dispatch support
/// @param initial Initial counter value
/// @return An external pointer to the wrapped stateful counter
/// @export
#[miniextendr]
pub fn new_stateful_counter(initial: i32) -> SEXP {
    let counter = StatefulCounter::create(initial);
    let erased = __mx_wrap_statefulcounter(counter);
    unsafe { ccall::mx_wrap(erased) }
}

/// Create a new counter with cross-package trait dispatch support
/// @param initial Initial counter value
/// @return An external pointer to the wrapped counter
/// @export
#[miniextendr]
pub fn new_counter(initial: i32) -> SEXP {
    let counter = SimpleCounter::create(initial);
    let erased = __mx_wrap_simplecounter(counter);
    unsafe { ccall::mx_wrap(erased) }
}

/// Get the value from a trait-wrapped counter
/// @param counter_sexp An external pointer from new_counter()
/// @return The counter's current value
/// @export
#[miniextendr]
pub fn counter_get_value(counter_sexp: SEXP) -> i32 {
    let view = unsafe { CounterView::try_from_sexp(counter_sexp) }
        .expect("Object does not implement Counter trait");
    view.value()
}
// endregion

// region: Debug/utility functions

/// Debug: Get TAG_COUNTER as hex string
/// @export
#[miniextendr]
pub fn debug_tag_counter() -> String {
    format!("{:016x}{:016x}", TAG_COUNTER.hi, TAG_COUNTER.lo)
}

/// Debug: Get TAG_RESETTABLE as hex string
/// @export
#[miniextendr]
pub fn debug_tag_resettable() -> String {
    format!("{:016x}{:016x}", TAG_RESETTABLE.hi, TAG_RESETTABLE.lo)
}

/// Debug: Get the ExternalPtr type name for SharedData
/// @export
#[miniextendr]
pub fn debug_shared_data_type_name() -> String {
    use miniextendr_api::externalptr::TypedExternal;
    SharedData::TYPE_NAME.to_string()
}

/// Get class of any R object (for cross-package testing)
/// @param x Any R object
/// @return Character vector of class names
/// @export
#[miniextendr]
pub fn get_r_class(x: SEXP) -> SEXP {
    use miniextendr_api::prelude::SexpExt;
    x.get_class()
}
// endregion

// region: Cross-package vctrs type export (producer_temp)

// VctrsClass must be in scope for the `#[derive(Vctrs)]`-generated
// `attrs()` / `CLASS_NAME` references to resolve.
#[cfg(feature = "vctrs")]
use miniextendr_api::vctrs::VctrsClass;

// A vctrs vector type the consumer package constructs and dispatches on.
//
// vctrs types are pure-R-attribute objects (class vector + data), and their
// behaviour (format, coercion, proxy) is implemented by S3 methods registered
// in NAMESPACE via `S3method()`. Once producer.pkg is loaded, those S3 methods
// live in R's global S3 method table, so ANY package — including consumer.pkg —
// dispatches on `producer_temp` correctly. No `R_GetCCallable` / vtable shim is
// involved: unlike trait-ABI ExternalPtr objects, vctrs export rides entirely
// on R's S3 method registry plus an exported constructor the consumer imports.
//
// `coerce = "double"` additionally emits `vec_ptype2.producer_temp.double` /
// `vec_cast.producer_temp.double` (and the reverse), so the consumer can cast a
// producer_temp to/from a bare double across the boundary.
#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs)]
#[vctrs(
    class = "producer_temp",
    base = "double",
    abbr = "degC",
    coerce = "double"
)]
pub struct ProducerTemp {
    #[vctrs(data)]
    values: Vec<f64>,
}

/// Create a `producer_temp` vctrs vector (Celsius temperatures).
///
/// This is the exported constructor consumer.pkg imports via
/// `importFrom(producer.pkg, new_temperature)` to build a producer-owned vctrs
/// type. Dispatch (format/coercion/`vec_c`) then resolves through R's S3 method
/// registry once producer.pkg is loaded.
///
/// @param x Numeric temperatures in Celsius.
/// @return A `producer_temp` vctrs vector.
/// @export
#[cfg(feature = "vctrs")]
#[miniextendr]
pub fn new_temperature(x: Vec<f64>) -> miniextendr_api::AsVctrs<ProducerTemp> {
    miniextendr_api::AsVctrs(ProducerTemp { values: x })
}
// endregion

// region: Cross-package vctrs inheritance (producer_oven_temp extends producer_temp)

// A vctrs type that inherits from `producer_temp` via `extends`. The derive:
//   1. Prepends the parent into the class vector:
//      c("producer_oven_temp", "producer_temp", "vctrs_vctr", "double").
//      Because `producer_oven_temp` defines no `format`/`vec_ptype_abbr` method,
//      S3 dispatch falls through the class vector to the parent's
//      `format.producer_temp` / `vec_ptype_abbr.producer_temp` automatically.
//   2. Emits bidirectional vec_ptype2/vec_cast stubs between child and parent.
//      Following vctrs' supertype rule, the common type of child and parent is
//      the PARENT, so `vec_c(child, parent)` resolves to `producer_temp`.
// The parent shares this type's `base = "double"`.
#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs)]
#[vctrs(
    class = "producer_oven_temp",
    base = "double",
    extends = "producer_temp"
)]
pub struct ProducerOvenTemp {
    #[vctrs(data)]
    values: Vec<f64>,
}

/// Create a `producer_oven_temp` vctrs vector that inherits from `producer_temp`.
///
/// Exercises `extends` (#1039): the child inherits the parent's format/abbr S3
/// methods through the class vector, and coercion between the two resolves to
/// the parent (the supertype). consumer.pkg imports this constructor to verify
/// inheritance dispatches across the package boundary.
///
/// @param x Numeric oven temperatures in Celsius.
/// @return A `producer_oven_temp` vctrs vector inheriting from `producer_temp`.
/// @export
#[cfg(feature = "vctrs")]
#[miniextendr]
pub fn new_oven_temperature(x: Vec<f64>) -> miniextendr_api::AsVctrs<ProducerOvenTemp> {
    miniextendr_api::AsVctrs(ProducerOvenTemp { values: x })
}
// endregion
