//! Factor tests - demonstrates RFactor derive macro for enum ↔ R factor conversions.
//!
//! RFactor types can be used directly as function parameters since they implement
//! TryFromSexp. The derive macro also generates IntoR for returning factors.

use miniextendr_api::{MatchArg, RFactor};

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

// region: Test functions demonstrating direct RFactor parameter/return usage

/// Test describing a Color factor variant as a human-readable string.
/// @param color A Color factor value.
#[miniextendr_api::miniextendr]
pub fn factor_describe_color(color: Color) -> &'static str {
    match color {
        Color::Red => "The color is red!",
        Color::Green => "The color is green!",
        Color::Blue => "The color is blue!",
    }
}

/// Test constructing a Color factor from a string name.
/// @param name Character name of the color ("red", "green", or "blue").
#[miniextendr_api::miniextendr]
pub fn factor_get_color(name: &str) -> Color {
    match name {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        _ => Color::Red,
    }
}

/// Test returning all Color variants as a FactorVec.
#[miniextendr_api::miniextendr]
pub fn factor_get_all_colors() -> miniextendr_api::FactorVec<Color> {
    miniextendr_api::FactorVec(vec![Color::Red, Color::Green, Color::Blue])
}

/// Test describing a Status factor with snake_case rename_all.
/// @param status A Status factor value.
#[miniextendr_api::miniextendr]
pub fn factor_describe_status(status: Status) -> &'static str {
    match status {
        Status::InProgress => "Work is in progress",
        Status::Completed => "Work is completed",
        Status::NotStarted => "Work has not started",
    }
}

/// Test describing a Priority factor with individually renamed variants.
/// @param priority A Priority factor value.
#[miniextendr_api::miniextendr]
pub fn factor_describe_priority(priority: Priority) -> &'static str {
    match priority {
        Priority::Low => "Low priority",
        Priority::Medium => "Medium priority",
        Priority::High => "High priority",
    }
}

/// Test retrieving the Color factor level names via CHOICES.
#[miniextendr_api::miniextendr]
pub fn factor_color_levels() -> Vec<&'static str> {
    Color::CHOICES.to_vec()
}

/// Test retrieving the Status factor level names via CHOICES.
#[miniextendr_api::miniextendr]
pub fn factor_status_levels() -> Vec<&'static str> {
    Status::CHOICES.to_vec()
}

/// Test retrieving the Priority factor level names via CHOICES.
#[miniextendr_api::miniextendr]
pub fn factor_priority_levels() -> Vec<&'static str> {
    Priority::CHOICES.to_vec()
}

/// Test counting occurrences of each Color in a FactorVec.
/// @param colors A factor vector of Color values.
#[miniextendr_api::miniextendr]
pub fn factor_count_colors(colors: miniextendr_api::FactorVec<Color>) -> Vec<i32> {
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

/// Test handling NA values in a FactorOptionVec of Colors.
/// @param colors A factor vector of Color values that may contain NA.
#[miniextendr_api::miniextendr]
pub fn factor_colors_with_na(colors: miniextendr_api::FactorOptionVec<Color>) -> Vec<&'static str> {
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

// Module export for
// endregion
