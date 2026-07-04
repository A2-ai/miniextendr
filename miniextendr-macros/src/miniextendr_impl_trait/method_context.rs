//! Shared per-method context for trait-impl R wrapper generation.
//!
//! Parallels `r_class_formatter::MethodContext` (used by the 6 inherent-impl
//! class generators) so the 5 trait generators (env/s3/s4/s7/r6) build
//! `.Call()` invocations and method preludes the same way instead of each
//! hand-rolling its own version. See
//! `audit/2026-07-03-dogfooding-macros-codegen.md` finding #1: the trait path
//! previously shared nothing above `DotCallBuilder`, which caused a
//! substring-corruption bug in the receiver-ptr extraction (S4/S7/R6) and a
//! silent 3-of-6-step prelude omission (missing `precondition_checks` /
//! `match_arg_prelude`) relative to inherent methods.

use super::TraitMethod;

/// Pre-computed context for a trait method, mirroring `MethodContext`
/// (`r_class_formatter.rs`) for inherent-impl methods. Holds the C wrapper
/// name, R formals (with defaults), and `.Call()` argument string so all 5
/// trait generators build calls identically.
pub(super) struct TraitMethodContext<'a> {
    /// Reference to the parsed trait method metadata.
    pub(super) method: &'a TraitMethod,
    /// The C wrapper identifier string (e.g., `"C_Foo__Bar__value"`), used in `.Call()`.
    pub(super) c_ident: String,
    /// R formals string with defaults, used in `function(...)` signatures.
    pub(super) params: String,
    /// R call arguments string (without defaults), used inside `.Call()`
    /// expressions. `Missing<T>` parameters are forwarded as
    /// `if (missing(p)) quote(expr=) else p` — see
    /// `r_wrapper_builder::RArgumentBuilder::build_call_args_vec`. Before this
    /// context existed, trait methods built call args via
    /// `collect_param_idents` instead, which skipped that forwarding entirely.
    pub(super) args: String,
}

impl<'a> TraitMethodContext<'a> {
    /// Build a context for `method`, which implements `trait_name` for `type_ident`.
    pub(super) fn new(
        method: &'a TraitMethod,
        type_ident: &syn::Ident,
        trait_name: &syn::Ident,
    ) -> Self {
        let c_ident = method.c_wrapper_ident_string(type_ident, trait_name);
        // match_arg/choices formal defaults are load-bearing for match.arg()
        // (see `effective_r_defaults` docs) — not just cosmetic.
        let effective_defaults = crate::r_class_formatter::effective_r_defaults(
            &method.param_defaults,
            &method.per_param,
            &c_ident,
        );
        let params =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &effective_defaults);
        let args = crate::r_wrapper_builder::build_r_call_args_from_sig(&method.sig);
        Self {
            method,
            c_ident,
            params,
            args,
        }
    }

    /// Build the `.Call()` expression for a static (non-receiver) trait method.
    pub(super) fn static_call(&self) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .with_args_str(&self.args)
            .build()
    }

    /// Build the `.Call()` expression for an instance trait method, with
    /// `self_expr` passed directly as the receiver argument (e.g. `".ptr"`,
    /// `"x"`, `"self@.ptr"`).
    ///
    /// This is the structured equivalent of `MethodContext::instance_call` —
    /// no string surgery. It fixes the substring-corruption bug where S4/S7/R6
    /// built the call with `self = "x"` and then ran
    /// `call.replace(", x", ", .ptr")`: `str::replace` rewrites *every* match
    /// of the substring `", x"`, so a parameter whose R name started with `x`
    /// (e.g. `x_factor`) was corrupted into `.ptr_factor`, producing a runtime
    /// "object '.ptr_factor' not found" error. Passing the receiver expression
    /// directly to `with_self` never touches the other arguments.
    pub(super) fn instance_call(&self, self_expr: &str) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .with_self(self_expr)
            .with_args_str(&self.args)
            .build()
    }

    /// Build R prelude lines validating `match_arg`/`choices` params. See
    /// `r_class_formatter::build_match_arg_prelude`.
    pub(super) fn match_arg_prelude(&self) -> Vec<String> {
        crate::r_class_formatter::build_match_arg_prelude(&self.method.per_param)
    }

    /// R-side `stopifnot()` precondition checks for this method's parameters.
    /// See `MethodContext::precondition_checks` for the inherent-impl twin.
    pub(super) fn precondition_checks(&self) -> Vec<String> {
        crate::r_class_formatter::build_method_precondition_checks(
            &self.method.sig.inputs,
            &self.method.per_param,
            self.method.coerce,
        )
    }

    /// Emit the shared method prelude into `lines`, each line prefixed with
    /// `indent` — the trait-impl twin of `MethodContext::emit_method_prelude`.
    /// In order: `r_entry`, `r_on_exit`, `lifecycle_prelude`,
    /// `precondition_checks`, `match_arg_prelude`, `r_post_checks`.
    ///
    /// `Missing<T>` forwarding is not a prelude step here either — it's inline
    /// in `self.args` (built in `new`), matching the inherent path.
    ///
    /// `what` is the human-readable label passed to the lifecycle prelude.
    /// This mirrors the pre-refactor `trait_method_preamble_lines`, which
    /// always used the R-facing method name as `what` regardless of class
    /// system (inherent methods instead use a class-qualified label like
    /// `"Type.method"` — trait methods keep the simpler unqualified label to
    /// avoid changing existing lifecycle-warning wording).
    pub(super) fn emit_method_prelude(&self, lines: &mut Vec<String>, indent: &str, what: &str) {
        let m = self.method;
        if let Some(ref entry) = m.r_entry {
            for line in entry.lines() {
                lines.push(format!("{indent}{line}"));
            }
        }
        if let Some(ref on_exit) = m.r_on_exit {
            lines.push(format!("{indent}{}", on_exit.to_r_code()));
        }
        if let Some(ref spec) = m.lifecycle
            && let Some(prelude) = spec.r_prelude(what)
        {
            for line in prelude.lines() {
                lines.push(format!("{indent}{line}"));
            }
        }
        for check in self.precondition_checks() {
            lines.push(format!("{indent}{check}"));
        }
        for line in self.match_arg_prelude() {
            lines.push(format!("{indent}{line}"));
        }
        if let Some(ref post) = m.r_post_checks {
            for line in post.lines() {
                lines.push(format!("{indent}{line}"));
            }
        }
    }
}
