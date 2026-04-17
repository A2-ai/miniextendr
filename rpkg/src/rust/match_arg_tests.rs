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

// region: Test functions using #[miniextendr(match_arg)]

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

/// Auto-doc fixture: no user @param for mode — auto-injected from enum choices.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_auto_doc_mode(#[miniextendr(match_arg)] mode: Mode) -> String {
    format!("{:?}", mode)
}

/// Auto-doc fixture: no user @param for modes — auto-injected as several_ok.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_auto_doc_modes(
    #[miniextendr(match_arg, several_ok)] modes: Vec<Mode>,
) -> String {
    modes.iter().map(|m| format!("{:?}", m)).collect::<Vec<_>>().join(", ")
}
// endregion

// region: Test functions using #[miniextendr(choices(...))] for string parameters

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
// endregion

// region: Test functions using #[miniextendr(choices(...), several_ok)]

/// Select multiple colors.
///
/// @param colors One or more colors.
/// @export
#[miniextendr_api::miniextendr]
pub fn choices_multi_color(
    #[miniextendr(choices("red", "green", "blue"), several_ok)] colors: Vec<String>,
) -> String {
    colors.join(", ")
}

/// Select metrics with several.ok.
///
/// @param n Count.
/// @param metrics One or more metrics.
/// @export
#[miniextendr_api::miniextendr]
pub fn choices_multi_metrics(
    n: i32,
    #[miniextendr(choices("mean", "median", "sd", "var"), several_ok)] metrics: Vec<String>,
) -> String {
    format!("n={}, metrics={}", n, metrics.join("+"))
}
// endregion

// region: Test functions returning Vec<MatchArgEnum>

/// Return a Vec of Mode values as an R character vector.
///
/// Exercises `IntoR for Vec<Mode>` emitted by `#[derive(MatchArg)]`.
///
/// @param modes One or more Mode values (several.ok input).
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_return_modes(
    #[miniextendr(match_arg, several_ok)] modes: Vec<Mode>,
) -> Vec<Mode> {
    modes
}
// endregion

// region: Test functions using #[miniextendr(match_arg, several_ok)] on enum Vec

/// Select multiple modes (enum-based several_ok).
///
/// @param modes One or more Mode values.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_multi_mode(
    #[miniextendr(match_arg, several_ok)] modes: Vec<Mode>,
) -> String {
    modes.iter().map(|m| format!("{:?}", m)).collect::<Vec<_>>().join(", ")
}

/// Select multiple priorities with a regular param.
///
/// @param n Count.
/// @param priorities One or more Priority values.
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_multi_priority(
    n: i32,
    #[miniextendr(match_arg, several_ok)] priorities: Vec<Priority>,
) -> String {
    let ps: Vec<&str> = priorities.iter().map(|p| p.to_choice()).collect();
    format!("n={}, priorities={}", n, ps.join("+"))
}
// endregion
