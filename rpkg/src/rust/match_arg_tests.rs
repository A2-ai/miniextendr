//! MatchArg tests - demonstrates `#[derive(MatchArg)]` for enum ↔ R string
//! conversion with `match.arg` semantics (partial matching, NULL default).

use miniextendr_api::MatchArg;

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

/// Set mode using match_arg.
///
/// @param mode A Mode enum value.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_set_mode(#[miniextendr(match_arg)] mode: Mode) -> String {
    format!("{:?}", mode)
}

/// Set status using match_arg.
///
/// @param status A BuildStatus enum value.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_set_status(#[miniextendr(match_arg)] status: BuildStatus) -> String {
    format!("{:?}", status)
}

/// Set priority using match_arg.
///
/// @param priority A Priority enum value.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_set_priority(#[miniextendr(match_arg)] priority: Priority) -> String {
    format!("{:?}", priority)
}

/// Function with match_arg + regular param to test mixing.
///
/// @param x An integer input.
/// @param mode A Mode enum value.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_mixed(x: i32, #[miniextendr(match_arg)] mode: Mode) -> String {
    format!("x={}, mode={:?}", x, mode)
}

/// Function with explicit default override for match_arg param.
///
/// @param mode A Mode enum value.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_with_default(
    #[miniextendr(match_arg, default = "\"Safe\"")] mode: Mode,
) -> String {
    format!("{:?}", mode)
}

/// Return the choices for Mode (for testing from R).
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_mode_choices() -> Vec<&'static str> {
    Mode::CHOICES.to_vec()
}

/// Return the choices for BuildStatus.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_status_choices() -> Vec<&'static str> {
    BuildStatus::CHOICES.to_vec()
}

/// Return the choices for Priority.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_priority_choices() -> Vec<&'static str> {
    Priority::CHOICES.to_vec()
}

/// Test returning a MatchArg enum (uses IntoR -> character scalar).
///
/// @param mode A Mode enum value.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_return_mode(#[miniextendr(match_arg)] mode: Mode) -> Mode {
    mode
}

// ============================================================================
// Test functions using #[miniextendr(choices(...))] for string parameters
// ============================================================================

/// @param x First value.
/// @param y Second value.
#[miniextendr_api::miniextendr]
pub fn choices_correlation(
    x: f64,
    y: f64,
    #[miniextendr(choices("pearson", "kendall", "spearman"))] method: &str,
) -> String {
    format!("method={}, x={}, y={}", method, x, y)
}

#[miniextendr_api::miniextendr]
pub fn choices_color(#[miniextendr(choices("red", "green", "blue"))] color: String) -> String {
    format!("color={}", color)
}

/// @param n Count.
/// @param verbose Whether to print details.
#[miniextendr_api::miniextendr]
pub fn choices_mixed(
    n: i32,
    #[miniextendr(choices("fast", "safe"))] mode: &str,
    verbose: bool,
) -> String {
    format!("n={}, mode={}, verbose={}", n, mode, verbose)
}
