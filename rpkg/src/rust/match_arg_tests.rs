//! MatchArg tests - demonstrates `#[derive(MatchArg)]` for enum ↔ R string
//! conversion with `match.arg` semantics (partial matching, NULL default).

use miniextendr_api::{MatchArg, miniextendr_module};

/// Simple mode enum with default variant ordering.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
pub enum Mode {
    Fast,
    Safe,
    Debug,
}

/// Status enum demonstrating rename_all to snake_case.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
#[match_arg(rename_all = "snake_case")]
pub enum BuildStatus {
    InProgress,
    Completed,
    NotStarted,
}

/// Priority enum demonstrating individual rename.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
pub enum Priority {
    #[match_arg(rename = "lo")]
    Low,
    #[match_arg(rename = "med")]
    Medium,
    #[match_arg(rename = "hi")]
    High,
}

// ============================================================================
// Test functions using #[miniextendr(match_arg)]
// ============================================================================

#[miniextendr_api::miniextendr]
pub fn match_arg_set_mode(#[miniextendr(match_arg)] mode: Mode) -> String {
    format!("{:?}", mode)
}

#[miniextendr_api::miniextendr]
pub fn match_arg_set_status(#[miniextendr(match_arg)] status: BuildStatus) -> String {
    format!("{:?}", status)
}

#[miniextendr_api::miniextendr]
pub fn match_arg_set_priority(#[miniextendr(match_arg)] priority: Priority) -> String {
    format!("{:?}", priority)
}

/// Function with match_arg + regular param to test mixing.
#[miniextendr_api::miniextendr]
pub fn match_arg_mixed(x: i32, #[miniextendr(match_arg)] mode: Mode) -> String {
    format!("x={}, mode={:?}", x, mode)
}

/// Function with explicit default override for match_arg param.
#[miniextendr_api::miniextendr]
pub fn match_arg_with_default(
    #[miniextendr(match_arg, default = "\"Safe\"")] mode: Mode,
) -> String {
    format!("{:?}", mode)
}

/// Return the choices for Mode (for testing from R).
#[miniextendr_api::miniextendr]
pub fn match_arg_mode_choices() -> Vec<&'static str> {
    Mode::CHOICES.to_vec()
}

/// Return the choices for BuildStatus.
#[miniextendr_api::miniextendr]
pub fn match_arg_status_choices() -> Vec<&'static str> {
    BuildStatus::CHOICES.to_vec()
}

/// Return the choices for Priority.
#[miniextendr_api::miniextendr]
pub fn match_arg_priority_choices() -> Vec<&'static str> {
    Priority::CHOICES.to_vec()
}

/// Test returning a MatchArg enum (uses IntoR → character scalar).
#[miniextendr_api::miniextendr]
pub fn match_arg_return_mode(#[miniextendr(match_arg)] mode: Mode) -> Mode {
    mode
}

// ============================================================================
// Test functions using #[miniextendr(choices(...))] for string parameters
// ============================================================================

/// Idiomatic choices on a string parameter.
#[miniextendr_api::miniextendr]
pub fn choices_correlation(
    x: f64,
    y: f64,
    #[miniextendr(choices("pearson", "kendall", "spearman"))] method: &str,
) -> String {
    format!("method={}, x={}, y={}", method, x, y)
}

/// Choices on an owned String parameter.
#[miniextendr_api::miniextendr]
pub fn choices_color(
    #[miniextendr(choices("red", "green", "blue"))] color: String,
) -> String {
    format!("color={}", color)
}

/// Choices mixed with regular params.
#[miniextendr_api::miniextendr]
pub fn choices_mixed(
    n: i32,
    #[miniextendr(choices("fast", "safe"))] mode: &str,
    verbose: bool,
) -> String {
    format!("n={}, mode={}, verbose={}", n, mode, verbose)
}

miniextendr_module! {
    mod match_arg_tests;

    fn match_arg_set_mode;
    fn match_arg_set_status;
    fn match_arg_set_priority;
    fn match_arg_mixed;
    fn match_arg_with_default;
    fn match_arg_mode_choices;
    fn match_arg_status_choices;
    fn match_arg_priority_choices;
    fn match_arg_return_mode;
    fn choices_correlation;
    fn choices_color;
    fn choices_mixed;
}
