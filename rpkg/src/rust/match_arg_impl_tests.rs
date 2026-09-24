//! `match_arg` / `choices` / `several_ok` on impl-block methods (issue #153).
//!
//! Unlike standalone functions, impl methods can't carry per-parameter
//! `#[miniextendr(match_arg)]` attributes — Rust's parser rejects attribute
//! macros on fn parameters inside impl items. The surface is instead
//! method-level: `#[miniextendr(match_arg(p), choices(q = "a, b"))]`.
//!
//! These fixtures exercise the commonly-used class-system generators
//! (r6, env, s7, s3, s4) to confirm:
//! 1. The generated R wrapper calls `base::match.arg()` before `.Call()`.
//! 2. The C wrapper wires `match_arg_several_ok_params` into the
//!    `match_arg_vec_from_sexp` path for Vec-typed several_ok params.
//! 3. The formal default is populated — either the
//!    `.__MX_MATCH_ARG_CHOICES_*__` placeholder (resolved at cdylib write
//!    time from the enum's `MatchArg::CHOICES`) or an explicit
//!    `c("a", "b", "c")` vector for `choices(...)` params.

use miniextendr_api::{MatchArg, Missing, miniextendr};

/// Enum shared by every fixture — keeps the R-side choice list identical across
/// class systems so the testthat file can assert against one canonical vector.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
pub enum ImplMode {
    Fast,
    Safe,
    Debug,
}

// region: R6 — scalar match_arg on constructor + instance method

#[derive(miniextendr_api::ExternalPtr)]
pub struct R6MatchArgCounter {
    mode: ImplMode,
    hits: i32,
}

#[miniextendr(r6)]
impl R6MatchArgCounter {
    #[miniextendr(match_arg(mode))]
    pub fn new(mode: ImplMode) -> Self {
        Self { mode, hits: 0 }
    }

    pub fn mode(&self) -> ImplMode {
        self.mode
    }

    #[miniextendr(match_arg(mode))]
    pub fn record(&mut self, mode: ImplMode) -> i32 {
        self.mode = mode;
        self.hits += 1;
        self.hits
    }

    /// Omittable optional mode (#1551): the formal keeps the choice vector,
    /// an omitted argument reports the current mode, `NULL` reports `"null"`.
    #[miniextendr(match_arg(mode))]
    pub fn peek(&self, mode: Missing<Option<ImplMode>>) -> String {
        match mode {
            Missing::Absent => format!("current:{:?}", self.mode),
            Missing::Present(None) => "null".to_string(),
            Missing::Present(Some(m)) => format!("{m:?}"),
        }
    }

    /// Static method with a choices() param — validates the `choices(...)` path
    /// independently of the derived-enum path used by match_arg.
    #[miniextendr(choices(level = "low, medium, high"))]
    pub fn describe_level(level: String) -> String {
        format!("level={level}")
    }
}

// endregion

// region: env — scalar match_arg + several_ok container

#[derive(miniextendr_api::ExternalPtr)]
pub struct EnvMatchArgCounter {
    modes: Vec<ImplMode>,
}

#[miniextendr(env)]
impl EnvMatchArgCounter {
    #[miniextendr(match_arg_several_ok(modes))]
    pub fn new(modes: Vec<ImplMode>) -> Self {
        Self { modes }
    }

    pub fn count(&self) -> i32 {
        i32::try_from(self.modes.len()).unwrap_or(i32::MAX)
    }

    #[miniextendr(match_arg_several_ok(modes))]
    pub fn reset(&mut self, modes: Vec<ImplMode>) -> i32 {
        self.modes = modes;
        self.count()
    }
}

// endregion

// region: S7 — scalar match_arg on S7 method

#[derive(miniextendr_api::ExternalPtr)]
pub struct S7MatchArgHolder {
    mode: ImplMode,
}

#[miniextendr(s7)]
impl S7MatchArgHolder {
    #[miniextendr(match_arg(mode))]
    pub fn new(mode: ImplMode) -> Self {
        Self { mode }
    }

    pub fn current(&self) -> ImplMode {
        self.mode
    }

    #[miniextendr(match_arg(mode))]
    pub fn set(&mut self, mode: ImplMode) -> ImplMode {
        self.mode = mode;
        self.mode
    }
}

// endregion

// region: S3 — choices() on static + match_arg on instance

#[derive(miniextendr_api::ExternalPtr)]
pub struct S3MatchArgPoint {
    label: String,
}

#[miniextendr(s3)]
impl S3MatchArgPoint {
    /// @param label One of "alpha", "beta", "gamma".
    #[miniextendr(choices(label = "alpha, beta, gamma"))]
    pub fn new(label: String) -> Self {
        Self { label }
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }

    #[miniextendr(match_arg(mode))]
    pub fn relabel(&mut self, mode: ImplMode) -> String {
        self.label = format!("{}-{:?}", self.label, mode);
        self.label.clone()
    }

    /// Optional relabel (#1473): the R formal is `mode = NULL`, and `NULL`
    /// leaves the label unchanged instead of picking the first choice.
    #[miniextendr(match_arg(mode))]
    pub fn maybe_relabel(&mut self, mode: Option<ImplMode>) -> String {
        if let Some(mode) = mode {
            self.label = format!("{}-{:?}", self.label, mode);
        }
        self.label.clone()
    }
}

// endregion

// region: S4 — scalar match_arg on constructor + instance method (#209)

#[derive(miniextendr_api::ExternalPtr)]
pub struct S4MatchArgHolder {
    mode: ImplMode,
}

// Methods are deliberately NOT prefixed with `s4_`: the S4 generator prepends
// the prefix itself (e.g. `mode_current` → generic `s4_mode_current`). Naming
// the method `s4_mode_current` would double-prefix to `s4_s4_mode_current`.
#[miniextendr(s4)]
impl S4MatchArgHolder {
    #[miniextendr(match_arg(mode))]
    pub fn new(mode: ImplMode) -> Self {
        Self { mode }
    }

    pub fn mode_current(&self) -> ImplMode {
        self.mode
    }

    #[miniextendr(match_arg(mode))]
    pub fn mode_set(&mut self, mode: ImplMode) -> ImplMode {
        self.mode = mode;
        self.mode
    }
}

// endregion

// region: vctrs — scalar match_arg on constructor returning vector data (#208)

/// Marker struct for the `impl` block. Vctrs classes don't need to carry
/// Rust state — the vector payload returned by `new` is wrapped directly by
/// `vctrs::new_vctr` at the R layer.
pub struct VctrsMatchArgScale;

/// vctrs fixture where the constructor takes a match_arg'd `ImplMode` scalar
/// and returns the numeric payload that `vctrs::new_vctr()` wraps. Exercises
/// the same `MethodContext::match_arg_prelude()` path as the other class
/// systems through the vctrs codegen branch that emits
/// `new_<name>(...)` + `vctrs::new_vctr(data, class = "…")`.
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "mode"))]
impl VctrsMatchArgScale {
    // Vctrs ctors return vector payload (not Self) — vctrs::new_vctr wraps it.
    /// @param mode One of "Fast", "Safe", "Debug".
    #[allow(clippy::new_ret_no_self)]
    #[miniextendr(match_arg(mode))]
    pub fn new(mode: ImplMode) -> Vec<f64> {
        match mode {
            ImplMode::Fast => vec![0.01, 0.1, 1.0],
            ImplMode::Safe => vec![1.0, 2.0, 3.0],
            ImplMode::Debug => vec![-1.0, 0.0, 1.0],
        }
    }
}

// endregion

// region: omittable choices on S3 / S4 / S7 / vctrs methods and on trait methods (#1551)
//
// `Missing<..>` choice parameters on the class systems that #1551's own
// fixtures (R6, env) do not reach. Each class has its own Rd page: a class page
// lists a parameter name once, so a `mode` here would replace the `mode` line
// of the classes above.

/// `"absent"` for an omitted argument, `"null"` for `NULL`, the matched mode
/// otherwise.
fn omitted_mode(mode: Missing<Option<ImplMode>>) -> String {
    match mode {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(mode)) => format!("{mode:?}"),
    }
}

/// `"absent"` for an omitted argument, the matched modes (comma-separated)
/// otherwise; `NULL` selects every choice.
fn omitted_modes(modes: Missing<Vec<ImplMode>>) -> String {
    match modes {
        Missing::Absent => "absent".to_string(),
        Missing::Present(modes) => modes
            .iter()
            .map(|mode| format!("{mode:?}"))
            .collect::<Vec<_>>()
            .join(","),
    }
}

/// `mode=<..>;modes=<..>;level=<..>`: what reached Rust for each of the three
/// omittable choice parameters (`level` is `"absent"` or the matched string).
fn omitted_report(
    mode: Missing<Option<ImplMode>>,
    modes: Missing<Vec<ImplMode>>,
    level: Missing<String>,
) -> String {
    format!(
        "mode={};modes={};level={}",
        omitted_mode(mode),
        omitted_modes(modes),
        level.into_option().unwrap_or_else(|| "absent".to_string())
    )
}

/// S3 class whose method takes omittable choices.
#[derive(miniextendr_api::ExternalPtr)]
pub struct OmitPickS3;

#[miniextendr(s3)]
impl OmitPickS3 {
    pub fn new() -> Self {
        OmitPickS3
    }

    /// Omittable choices on an S3 method: reports what reached Rust for each
    /// of `mode`, `modes` and `level`.
    #[miniextendr(
        match_arg(mode),
        match_arg_several_ok(modes),
        choices(level = "low, mid, high")
    )]
    pub fn omit_pick_s3(
        &self,
        mode: Missing<Option<ImplMode>>,
        modes: Missing<Vec<ImplMode>>,
        level: Missing<String>,
    ) -> String {
        omitted_report(mode, modes, level)
    }
}

/// S4 class whose method takes omittable choices.
#[derive(miniextendr_api::ExternalPtr)]
pub struct OmitPickS4;

#[miniextendr(s4)]
impl OmitPickS4 {
    pub fn new() -> Self {
        OmitPickS4
    }

    // Generic `s4_omit_pick(x, ...)`. The method's formals differ from it, so
    // R wraps the method in `.local()`; `missing()` still sees an omitted
    // argument there.
    /// Omittable choices on an S4 method: reports what reached Rust for each
    /// of `mode`, `modes` and `level`.
    #[miniextendr(
        match_arg(mode),
        match_arg_several_ok(modes),
        choices(level = "low, mid, high")
    )]
    pub fn omit_pick(
        &self,
        mode: Missing<Option<ImplMode>>,
        modes: Missing<Vec<ImplMode>>,
        level: Missing<String>,
    ) -> String {
        omitted_report(mode, modes, level)
    }
}

/// S7 class whose method (and its fast-path shortcut) takes omittable
/// choices.
#[derive(miniextendr_api::ExternalPtr)]
pub struct OmitPickS7;

#[miniextendr(s7)]
impl OmitPickS7 {
    pub fn new() -> Self {
        OmitPickS7
    }

    /// Omittable choices on an S7 method: reports what reached Rust for each
    /// of `mode`, `modes` and `level`.
    #[miniextendr(
        match_arg(mode),
        match_arg_several_ok(modes),
        choices(level = "low, mid, high")
    )]
    pub fn omit_pick_s7(
        &self,
        mode: Missing<Option<ImplMode>>,
        modes: Missing<Vec<ImplMode>>,
        level: Missing<String>,
    ) -> String {
        omitted_report(mode, modes, level)
    }
}

/// vctrs fixture whose constructor, static method and `format` protocol
/// method take omittable choices. The payload tells an omitted constructor
/// argument (`0`) from `NULL` (`-1`) and from each mode.
pub struct VctrsOmitScale;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "omit"))]
impl VctrsOmitScale {
    /// @param mode One of "Fast", "Safe", "Debug", or NULL; omitting the
    ///   argument means no choice.
    #[allow(clippy::new_ret_no_self)]
    #[miniextendr(match_arg(mode))]
    pub fn new(mode: Missing<Option<ImplMode>>) -> Vec<f64> {
        match mode {
            Missing::Absent => vec![0.0],
            Missing::Present(None) => vec![-1.0],
            Missing::Present(Some(ImplMode::Fast)) => vec![1.0],
            Missing::Present(Some(ImplMode::Safe)) => vec![2.0],
            Missing::Present(Some(ImplMode::Debug)) => vec![3.0],
        }
    }

    /// Omittable choices on a vctrs static method: reports what reached Rust
    /// for each of `mode`, `modes` and `level`.
    #[miniextendr(
        match_arg(mode),
        match_arg_several_ok(modes),
        choices(level = "low, mid, high")
    )]
    pub fn omit_pick(
        mode: Missing<Option<ImplMode>>,
        modes: Missing<Vec<ImplMode>>,
        level: Missing<String>,
    ) -> String {
        omitted_report(mode, modes, level)
    }

    /// `format()` protocol method with an omittable inline choice: `style`
    /// omitted formats each value as `<value>`, otherwise as `<style>:<value>`.
    ///
    /// @param x The vctrs payload.
    #[miniextendr(vctrs(format), choices(style = "short, long"))]
    #[allow(clippy::needless_pass_by_value)]
    pub fn format_omit_scale(x: Vec<f64>, style: Missing<String>) -> Vec<String> {
        let style = style.into_option();
        x.iter()
            .map(|value| match &style {
                None => format!("{value}"),
                Some(style) => format!("{style}:{value}"),
            })
            .collect()
    }
}

/// Omittable inline choices through the trait-method codegen path, which
/// takes `choices(p = "...")` / `choices_several_ok(p = "...")` on string
/// types only.
#[miniextendr]
pub trait OmitGrade {
    /// `"absent"`, `"null"` or the matched grade.
    fn omit_grade(&self, grade: Missing<Option<String>>) -> String;
    /// `"absent"` or the matched grades, comma-separated.
    fn omit_grades(&self, grades: Missing<Vec<String>>) -> String;
}

/// `"absent"` / `"null"` / the matched grade.
fn omitted_grade(grade: Missing<Option<String>>) -> String {
    match grade {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(grade)) => grade,
    }
}

/// `"absent"` / the matched grades.
fn omitted_grades(grades: Missing<Vec<String>>) -> String {
    grades
        .into_option()
        .map_or_else(|| "absent".to_string(), |grades| grades.join(","))
}

#[miniextendr(s3)]
impl OmitGrade for OmitPickS3 {
    #[miniextendr(choices(grade = "low, mid, high"))]
    fn omit_grade(&self, grade: Missing<Option<String>>) -> String {
        omitted_grade(grade)
    }

    #[miniextendr(choices_several_ok(grades = "low, mid, high"))]
    fn omit_grades(&self, grades: Missing<Vec<String>>) -> String {
        omitted_grades(grades)
    }
}

/// Calls the `OmitGrade` methods through the trait's View, the path another
/// package takes: an omitted, a `NULL` and a supplied grade, then an omitted
/// and a supplied list of grades.
#[miniextendr(no_worker)]
pub fn omit_grade_through_view() -> Vec<String> {
    unsafe {
        let erased = __mx_wrap_omitpicks3(OmitPickS3);
        let sexp = miniextendr_api::gc_protect::OwnedProtect::new(
            miniextendr_api::trait_abi::ccall::mx_wrap(erased),
        );
        let view = OmitGradeView::from_sexp(sexp.get());
        vec![
            view.omit_grade(Missing::Absent),
            view.omit_grade(Missing::Present(None)),
            view.omit_grade(Missing::Present(Some("mid".to_string()))),
            view.omit_grades(Missing::Absent),
            view.omit_grades(Missing::Present(vec![
                "low".to_string(),
                "high".to_string(),
            ])),
        ]
    }
}

#[miniextendr(s7)]
impl OmitGrade for OmitPickS7 {
    #[miniextendr(choices(grade = "low, mid, high"))]
    fn omit_grade(&self, grade: Missing<Option<String>>) -> String {
        omitted_grade(grade)
    }

    #[miniextendr(choices_several_ok(grades = "low, mid, high"))]
    fn omit_grades(&self, grades: Missing<Vec<String>>) -> String {
        omitted_grades(grades)
    }
}

// endregion
