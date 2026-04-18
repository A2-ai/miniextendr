//! `match_arg` / `choices` / `several_ok` on impl-block methods (issue #153).
//!
//! Unlike standalone functions, impl methods can't carry per-parameter
//! `#[miniextendr(match_arg)]` attributes — Rust's parser rejects attribute
//! macros on fn parameters inside impl items. The surface is instead
//! method-level: `#[miniextendr(match_arg(p), choices(q = "a, b"))]`.
//!
//! These fixtures exercise the five commonly-used class-system generators
//! (r6, env, s7, s3, vctrs) to confirm:
//! 1. The generated R wrapper calls `base::match.arg()` before `.Call()`.
//! 2. The C wrapper wires `match_arg_several_ok_params` into the
//!    `match_arg_vec_from_sexp` path for Vec-typed several_ok params.
//! 3. The formal default is populated — either the
//!    `.__MX_MATCH_ARG_CHOICES_*__` placeholder (resolved at cdylib write
//!    time from the enum's `MatchArg::CHOICES`) or an explicit
//!    `c("a", "b", "c")` vector for `choices(...)` params.

use miniextendr_api::{MatchArg, miniextendr};

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
}

// endregion

// vctrs intentionally omitted: vctrs constructors expect the `.data` argument
// to be vector-shaped, so a straightforward struct-returning ctor doesn't fit
// the fixture pattern used above. The match.arg prelude is shared code
// emitted by the same `MethodContext::match_arg_prelude()` path, so r6/env/s7/s3
// coverage exercises the same codegen surface.
