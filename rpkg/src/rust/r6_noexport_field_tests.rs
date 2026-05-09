//! Fixture for testing `#' @field name NULL` opt-out for noexported R6 active bindings.
//!
//! roxygen2 8.0.0 introduced `@field name NULL` as the documented way to suppress
//! documentation for R6 fields and active bindings. When a method is tagged with
//! `#[miniextendr(noexport)]` or `#[miniextendr(internal)]`, the generated R wrapper
//! emits `#' @field name NULL` instead of a real description so roxygen2 drops the
//! binding from the rendered `.Rd`.

use miniextendr_api::miniextendr;

/// An R6 class demonstrating `@field name NULL` opt-out for active bindings.
///
/// `R6SensorReading` has two active bindings:
///
/// - `value`: exported, documents in the generated `.Rd`.
/// - `raw_bytes`: tagged `noexport`; gets `@field raw_bytes NULL` so roxygen2 8.0.0
///   drops it from docs while keeping the binding available at runtime.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6SensorReading {
    value: f64,
    raw: i32,
}

/// R6 sensor reading with one documented and one undocumented active binding.
/// @param value Numeric sensor value.
/// @param raw Integer raw ADC reading.
#[miniextendr(r6)]
impl R6SensorReading {
    /// Creates a new sensor reading.
    pub fn new(value: f64, raw: i32) -> Self {
        R6SensorReading { value, raw }
    }

    /// The calibrated sensor value (exported active binding).
    #[miniextendr(r6(active))]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Internal raw ADC reading — not part of the public API.
    #[miniextendr(r6(active), noexport)]
    pub fn raw_bytes(&self) -> i32 {
        self.raw
    }
}
