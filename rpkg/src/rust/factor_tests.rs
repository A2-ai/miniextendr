//! Factor tests - demonstrates RFactor derive macro for enum ↔ R factor conversions.
//!
//! These tests use manual SEXP conversion since #[miniextendr] doesn't yet support
//! RFactor types directly as function parameters. The RFactor derive generates
//! TryFromSexp and IntoR implementations that can be used manually.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::{IntoR, RFactor, miniextendr_module};

/// Color enum demonstrating basic RFactor usage.
#[derive(Copy, Clone, Debug, PartialEq, RFactor)]
pub enum Color {
    Red,
    Green,
    Blue,
}

/// Status enum demonstrating rename_all attribute.
#[derive(Copy, Clone, Debug, PartialEq, RFactor)]
#[r_factor(rename_all = "snake_case")]
pub enum Status {
    InProgress,
    Completed,
    NotStarted,
}

/// Priority enum demonstrating individual rename attributes.
#[derive(Copy, Clone, Debug, PartialEq, RFactor)]
pub enum Priority {
    #[r_factor(rename = "low")]
    Low,
    #[r_factor(rename = "med")]
    Medium,
    #[r_factor(rename = "high")]
    High,
}

// ============================================================================
// Test functions using SEXP for factor input/output
// ============================================================================

/// Test: accepts a Color factor and returns a description.
#[miniextendr_api::miniextendr]
pub fn factor_describe_color(color: SEXP) -> &'static str {
    match Color::try_from_sexp(color) {
        Ok(Color::Red) => "The color is red!",
        Ok(Color::Green) => "The color is green!",
        Ok(Color::Blue) => "The color is blue!",
        Err(e) => {
            eprintln!("Error: {}", e);
            "unknown"
        }
    }
}

/// Test: returns a Color factor.
#[miniextendr_api::miniextendr]
pub fn factor_get_color(name: &str) -> SEXP {
    let color = match name {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        _ => Color::Red,
    };
    color.into_sexp()
}

/// Test: returns all colors as a factor vector.
#[miniextendr_api::miniextendr]
pub fn factor_get_all_colors() -> SEXP {
    // Use FactorVec newtype wrapper since orphan rules prevent impl IntoR for Vec<T: RFactor>
    miniextendr_api::FactorVec(vec![Color::Red, Color::Green, Color::Blue]).into_sexp()
}

/// Test: accepts a Status factor (with snake_case levels).
#[miniextendr_api::miniextendr]
pub fn factor_describe_status(status: SEXP) -> &'static str {
    match Status::try_from_sexp(status) {
        Ok(Status::InProgress) => "Work is in progress",
        Ok(Status::Completed) => "Work is completed",
        Ok(Status::NotStarted) => "Work has not started",
        Err(_) => "unknown status",
    }
}

/// Test: accepts a Priority factor (with renamed levels).
#[miniextendr_api::miniextendr]
pub fn factor_describe_priority(priority: SEXP) -> &'static str {
    match Priority::try_from_sexp(priority) {
        Ok(Priority::Low) => "Low priority",
        Ok(Priority::Medium) => "Medium priority",
        Ok(Priority::High) => "High priority",
        Err(_) => "unknown priority",
    }
}

/// Test: returns the level names for Color.
#[miniextendr_api::miniextendr]
pub fn factor_color_levels() -> Vec<&'static str> {
    Color::LEVELS.to_vec()
}

/// Test: returns the level names for Status (snake_case).
#[miniextendr_api::miniextendr]
pub fn factor_status_levels() -> Vec<&'static str> {
    Status::LEVELS.to_vec()
}

/// Test: returns the level names for Priority (custom renamed).
#[miniextendr_api::miniextendr]
pub fn factor_priority_levels() -> Vec<&'static str> {
    Priority::LEVELS.to_vec()
}

/// Test: accepts a vector of Colors and counts each.
#[miniextendr_api::miniextendr]
pub fn factor_count_colors(colors: SEXP) -> Vec<i32> {
    // Use FactorVec wrapper for Vec<Color> deserialization
    let colors: miniextendr_api::FactorVec<Color> = match TryFromSexp::try_from_sexp(colors) {
        Ok(c) => c,
        Err(_) => return vec![0, 0, 0],
    };
    let mut counts = [0i32; 3];
    for c in colors.iter() {
        match c {
            Color::Red => counts[0] += 1,
            Color::Green => counts[1] += 1,
            Color::Blue => counts[2] += 1,
        }
    }
    counts.to_vec()
}

/// Test: accepts Colors with NA values and describes them.
#[miniextendr_api::miniextendr]
pub fn factor_colors_with_na(colors: SEXP) -> Vec<&'static str> {
    // Use FactorOptionVec wrapper for Vec<Option<Color>> deserialization
    let colors: miniextendr_api::FactorOptionVec<Color> = match TryFromSexp::try_from_sexp(colors) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    colors
        .iter()
        .map(|c| match c {
            Some(Color::Red) => "red",
            Some(Color::Green) => "green",
            Some(Color::Blue) => "blue",
            None => "NA",
        })
        .collect()
}

// Module export for miniextendr_module! in lib.rs
miniextendr_module! {
    mod factor_tests;

    fn factor_describe_color;
    fn factor_get_color;
    fn factor_get_all_colors;
    fn factor_describe_status;
    fn factor_describe_priority;
    fn factor_color_levels;
    fn factor_status_levels;
    fn factor_priority_levels;
    fn factor_count_colors;
    fn factor_colors_with_na;
}
