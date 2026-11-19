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

// region: Variants named like associated items (#1730)

/// Outcome factor whose variant names match associated items of the traits the
/// derive implements (`TryFromSexp::Error`, `IntoR::Error`), so the generated
/// impls must not write `Self::Error` (#1730).
#[derive(Copy, Clone, Debug, PartialEq, RFactor)]
#[r_factor(rename_all = "snake_case")]
pub enum Outcome {
    Error,
    Value,
    Output,
}

/// Interaction factor with an outer variant named `Error`.
#[derive(Copy, Clone, Debug, PartialEq, RFactor)]
#[r_factor(interaction = ["error", "value", "output"])]
pub enum OutcomeStage {
    Error(Outcome),
    Value(Outcome),
}

/// Test round-tripping an Outcome factor through R.
/// @param outcome An Outcome factor value.
#[miniextendr_api::miniextendr]
pub fn factor_outcome_round_trip(outcome: Outcome) -> Outcome {
    outcome
}

/// Test retrieving the Outcome factor level names via CHOICES.
#[miniextendr_api::miniextendr]
pub fn factor_outcome_levels() -> Vec<&'static str> {
    Outcome::CHOICES.to_vec()
}

/// Test round-tripping an OutcomeStage interaction factor through R.
/// @param stage An OutcomeStage factor value.
#[miniextendr_api::miniextendr]
pub fn factor_outcome_stage_round_trip(stage: OutcomeStage) -> OutcomeStage {
    stage
}

/// Test retrieving the OutcomeStage interaction levels via CHOICES.
#[miniextendr_api::miniextendr]
pub fn factor_outcome_stage_levels() -> Vec<&'static str> {
    OutcomeStage::CHOICES.to_vec()
}
// endregion

// region: Containers of an RFactor enum are factors (#2365)

/// Test returning a `Vec<Color>`: a factor with Color's levels, like a scalar Color.
#[miniextendr_api::miniextendr]
pub fn factor_colors_vec() -> Vec<Color> {
    vec![Color::Blue, Color::Red, Color::Blue]
}

/// Test returning a `Box<[Color]>`: a factor with Color's levels.
#[miniextendr_api::miniextendr]
pub fn factor_colors_boxed() -> Box<[Color]> {
    vec![Color::Green, Color::Red].into_boxed_slice()
}

/// Test returning an `Option<Vec<Color>>`: a factor, or `NULL` for `None`.
/// @param present Whether to return the colors.
#[miniextendr_api::miniextendr]
pub fn factor_colors_maybe(present: bool) -> Option<Vec<Color>> {
    present.then(|| vec![Color::Green])
}

/// Test returning a `Vec<Option<Color>>`: a factor, `NA` for `None`.
#[miniextendr_api::miniextendr]
pub fn factor_colors_with_missing() -> Vec<Option<Color>> {
    vec![Some(Color::Green), None, Some(Color::Blue)]
}

/// Test returning a `Vec` of an interaction factor enum.
#[miniextendr_api::miniextendr]
pub fn factor_outcome_stages() -> Vec<OutcomeStage> {
    vec![
        OutcomeStage::Value(Outcome::Output),
        OutcomeStage::Error(Outcome::Error),
    ]
}

/// Test reading a factor as `Vec<Color>` and returning it.
/// @param colors A factor with Color's levels.
#[miniextendr_api::miniextendr]
pub fn factor_colors_round_trip(colors: Vec<Color>) -> Vec<Color> {
    colors
}

/// Test reading a factor as `Box<[Color]>` and returning it reversed.
/// @param colors A factor with Color's levels.
#[miniextendr_api::miniextendr]
pub fn factor_colors_boxed_reversed(colors: Box<[Color]>) -> Box<[Color]> {
    let mut colors = colors;
    colors.reverse();
    colors
}

/// Test reading a factor with `NA` as `Vec<Option<Color>>` and returning it.
/// @param colors A factor with Color's levels, `NA` allowed.
#[miniextendr_api::miniextendr]
pub fn factor_colors_option_round_trip(colors: Vec<Option<Color>>) -> Vec<Option<Color>> {
    colors
}

/// Test reading an `Option<Color>`: `NULL`, `NA` and a factor `NA` are `None`.
/// @param color A length-one factor with Color's levels, `NA` or `NULL`.
#[miniextendr_api::miniextendr]
pub fn factor_describe_optional_color(color: Option<Color>) -> &'static str {
    match color {
        Some(Color::Red) => "The color is red!",
        Some(Color::Green) => "The color is green!",
        Some(Color::Blue) => "The color is blue!",
        None => "No color",
    }
}

/// Test reading a `Vec` of an interaction factor enum and returning it.
/// @param stages A factor with OutcomeStage's levels.
#[miniextendr_api::miniextendr]
pub fn factor_outcome_stages_round_trip(stages: Vec<OutcomeStage>) -> Vec<OutcomeStage> {
    stages
}
// endregion
