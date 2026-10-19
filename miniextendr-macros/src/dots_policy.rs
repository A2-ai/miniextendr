//! `#[miniextendr(dots = ...)]`: `typed_list!(...)`, or what the function's
//! `...` is for (#2157).
//!
//! An S3 method takes `...` because its generic does, and R hands it every
//! argument the method has no formal for. A method that reads none of them
//! loses a misspelt one without a word: `confint(fit, levle = 0.9)` returns
//! the 95% interval. The role says which arguments `...` takes, and the
//! generated R wrapper refuses the others before it evaluates anything:
//!
//! | `dots =` | `...` takes | the Rust function |
//! |---|---|---|
//! | `empty` | nothing | reads no dots |
//! | `empty(allow = ["optional"])` | the listed names | ignores them |
//! | `forward(allow = ["digits"])` | the listed names | passes them on to another R function |
//! | `data` | anything, unchecked | reads them |
//!
//! An argument the function reads is a parameter, not a name in `...`: R then
//! matches it by name or by position, as for any function.
//!
//! An unnamed argument in `...` is one of the generic's arguments given by
//! position (#2185). Under `forward` it passes: the function the dots go on to
//! matches it by position itself. Under `empty` nothing reads it, so only the
//! count matters: up to the number of allowed names not given by name are
//! taken, and the rest refused.
//!
//! `class = "..."` (or `class = ["...", ...]`) puts the package's classes in
//! front of the refusal's own.
//!
//! The check is one statement, `if (...length()) .miniextendr_check_dots(...)`
//! ([`DotsPolicy::r_check`]), after the lifecycle notice and before the
//! argument checks, so a misspelt argument is reported before a missing one
//! (`predict(fit, nwedata = nd)` names `nwedata`, not the missing `newdata`).
//! It reads `...length()` and `...names()` only, so no argument is forced. The
//! helper is in the wrappers preamble (`miniextendr_api::registry`): one
//! classed argument error for every refused argument, with the formal or
//! listed name each one most likely meant.

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

/// What the function's `...` takes (see the [module docs](self)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DotsRole {
    /// `empty` / `empty(allow = [..])`: nothing, or names the function ignores.
    Empty,
    /// `forward(allow = [..])`: the names the function passes on.
    Forward,
    /// `data`: anything; no check.
    Data,
}

impl DotsRole {
    /// The spelling in the attribute, for diagnostics.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Forward => "forward",
            Self::Data => "data",
        }
    }
}

/// A parsed `dots = empty | empty(..) | forward(..) | data`.
#[derive(Clone, Debug)]
pub(crate) struct DotsPolicy {
    pub(crate) role: DotsRole,
    /// The names `...` takes, in order.
    pub(crate) allow: Vec<String>,
    /// The package's classes for the refusal, in front of its own.
    pub(crate) class: Vec<String>,
    /// The attribute value, for diagnostics about the signature.
    pub(crate) span: Span,
}

/// The value of `dots = ...`.
pub(crate) enum DotsAttr {
    /// `typed_list!(...)`, as written.
    TypedList(TokenStream),
    /// A role for `...`.
    Policy(DotsPolicy),
}

const EXPECTED: &str = "dots expects `typed_list!(...)`, `empty`, `empty(allow = [\"name\", ...])`, \
                        `forward(allow = [\"name\", ...])` or `data`";

/// Parse the value of `dots = ...`.
pub(crate) fn parse_dots_attr(value: &syn::Expr) -> syn::Result<DotsAttr> {
    match value {
        syn::Expr::Macro(expr_macro) if expr_macro.mac.path.is_ident("typed_list") => {
            Ok(DotsAttr::TypedList(quote::quote!(#expr_macro)))
        }
        syn::Expr::Path(path) => match role_of(&path.path) {
            Some(DotsRole::Forward) => Err(syn::Error::new_spanned(
                path,
                "`forward` passes the names it lists: `forward(allow = [\"name\", ...])`; \
                 `data` passes every argument",
            )),
            Some(role) => Ok(DotsAttr::Policy(DotsPolicy {
                role,
                allow: Vec::new(),
                class: Vec::new(),
                span: value.span(),
            })),
            None => Err(syn::Error::new_spanned(path, EXPECTED)),
        },
        syn::Expr::Call(call) => {
            let syn::Expr::Path(func) = call.func.as_ref() else {
                return Err(syn::Error::new_spanned(&call.func, EXPECTED));
            };
            let Some(role) = role_of(&func.path) else {
                return Err(syn::Error::new_spanned(func, EXPECTED));
            };
            if role == DotsRole::Data {
                return Err(syn::Error::new_spanned(
                    call,
                    "`data` takes no options: it passes every argument unchecked",
                ));
            }
            let (allow, class) = parse_options(call)?;
            if role == DotsRole::Forward && allow.is_empty() {
                return Err(syn::Error::new_spanned(
                    call,
                    "`forward` passes the names it lists: `forward(allow = [\"name\", ...])`; \
                     `data` passes every argument",
                ));
            }
            Ok(DotsAttr::Policy(DotsPolicy {
                role,
                allow,
                class,
                span: value.span(),
            }))
        }
        _ => Err(syn::Error::new_spanned(value, EXPECTED)),
    }
}

fn role_of(path: &syn::Path) -> Option<DotsRole> {
    let ident = path.get_ident()?;
    match ident.to_string().as_str() {
        "empty" => Some(DotsRole::Empty),
        "forward" => Some(DotsRole::Forward),
        "data" => Some(DotsRole::Data),
        _ => None,
    }
}

/// The `allow = ..` and `class = ..` options of `empty(..)` / `forward(..)`,
/// each at most once. Every problem is reported at once.
fn parse_options(call: &syn::ExprCall) -> syn::Result<(Vec<String>, Vec<String>)> {
    let mut errors: Option<syn::Error> = None;
    let mut push = |err: syn::Error| match &mut errors {
        Some(errors) => errors.combine(err),
        None => errors = Some(err),
    };
    let mut allow: Option<Vec<String>> = None;
    let mut class: Option<Vec<String>> = None;
    for arg in &call.args {
        let syn::Expr::Assign(assign) = arg else {
            push(syn::Error::new_spanned(
                arg,
                "expected `allow = [\"name\", ...]` or `class = \"...\"`",
            ));
            continue;
        };
        let key = match assign.left.as_ref() {
            syn::Expr::Path(p) => p.path.get_ident().map(ToString::to_string),
            _ => None,
        };
        let slot = match key.as_deref() {
            Some("allow") => &mut allow,
            Some("class") => &mut class,
            _ => {
                push(syn::Error::new_spanned(
                    &assign.left,
                    "unknown option; expected `allow = [\"name\", ...]` or `class = \"...\"`",
                ));
                continue;
            }
        };
        if slot.is_some() {
            push(syn::Error::new_spanned(
                &assign.left,
                format!("`{}` is given twice", key.unwrap_or_default()),
            ));
            continue;
        }
        match string_list(&assign.right, key.as_deref() == Some("allow")) {
            Ok(values) => *slot = Some(values),
            Err(err) => push(err),
        }
    }
    match errors {
        Some(errors) => Err(errors),
        None => Ok((allow.unwrap_or_default(), class.unwrap_or_default())),
    }
}

/// A string, or a non-empty array of distinct strings, none empty or holding
/// a NUL. Names (`allow`) can't be `...` either.
fn string_list(expr: &syn::Expr, names: bool) -> syn::Result<Vec<String>> {
    let items: Vec<&syn::Expr> = match expr {
        syn::Expr::Array(array) if !array.elems.is_empty() => array.elems.iter().collect(),
        syn::Expr::Array(array) => {
            return Err(syn::Error::new_spanned(
                array,
                "the list is empty; give at least one string",
            ));
        }
        other => vec![other],
    };
    let mut errors: Option<syn::Error> = None;
    let mut push = |err: syn::Error| match &mut errors {
        Some(errors) => errors.combine(err),
        None => errors = Some(err),
    };
    let mut values: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) = item
        else {
            push(syn::Error::new_spanned(item, "expected a string literal"));
            continue;
        };
        let value = lit.value();
        if value.is_empty() {
            push(syn::Error::new_spanned(lit, "the string is empty"));
        } else if value.contains('\0') {
            push(syn::Error::new_spanned(
                lit,
                "an R string cannot hold a NUL",
            ));
        } else if names && value == "..." {
            push(syn::Error::new_spanned(
                lit,
                "`...` is not an argument name; list the names `...` takes",
            ));
        } else if values.contains(&value) {
            push(syn::Error::new_spanned(
                lit,
                format!("\"{value}\" is listed twice"),
            ));
        } else {
            values.push(value);
        }
    }
    match errors {
        Some(errors) => Err(errors),
        None => Ok(values),
    }
}

impl DotsPolicy {
    /// Refuse a role the signature can't have, spanned on the attribute:
    /// `forward` and `data` hand the arguments to the Rust function, so it
    /// takes `...` (`_: ...`, `&Dots` or `LazyDots`); `wrapper_dots` says
    /// whether the R wrapper has a `...` formal for `empty` to check.
    pub(crate) fn check_signature(&self, rust_dots: bool, wrapper_dots: bool) -> syn::Result<()> {
        match self.role {
            DotsRole::Forward | DotsRole::Data if !rust_dots => Err(syn::Error::new(
                self.span,
                format!(
                    "`dots = {}` passes arguments to the function, which takes none: add a \
                     `...` parameter (`_: ...`, `&Dots` or `LazyDots`)",
                    self.role.name()
                ),
            )),
            DotsRole::Empty if !wrapper_dots => Err(syn::Error::new(
                self.span,
                "`dots = empty` checks the R wrapper's `...`, and this one has none: R already \
                 refuses an argument the function has no formal for",
            )),
            _ => Ok(()),
        }
    }

    /// The wrapper statement that refuses what `...` doesn't take, for a
    /// wrapper with `formals` (an R formals list); `call` is the wrapper's
    /// [`r_check_call`](crate::r_wrapper_builder::CallAttribution::r_check_call).
    /// `None` for `data`, and for a wrapper without a `...` formal, which R
    /// keeps from receiving any extra argument.
    ///
    /// The formals other than `...` and the generated `.call`, `.mx_self` and
    /// `.ptr` are what the helper suggests from, after the allowed names.
    pub(crate) fn r_check(&self, formals: &str, call: Option<&str>) -> Option<String> {
        if self.role == DotsRole::Data {
            return None;
        }
        let names: Vec<&str> = crate::roxygen::split_r_formals(formals)
            .into_iter()
            .map(crate::roxygen::formal_name)
            .filter(|name| !name.is_empty())
            .collect();
        if !names.contains(&"...") {
            return None;
        }
        let formals: Vec<String> = names
            .into_iter()
            .filter(|name| !matches!(*name, "..." | ".call" | ".mx_self" | ".ptr"))
            .map(str::to_string)
            .collect();
        let mut args = vec![
            "...names()".to_string(),
            "...length()".to_string(),
            r_chr(&formals),
        ];
        if !self.allow.is_empty() {
            args.push(format!("allow = {}", r_chr(&self.allow)));
        }
        if self.role == DotsRole::Forward {
            args.push("forward = TRUE".to_string());
        }
        if !self.class.is_empty() {
            args.push(format!("class = {}", r_chr(&self.class)));
        }
        if let Some(call) = call {
            args.push(format!("call = {call}"));
        }
        Some(format!(
            "if (...length()) .miniextendr_check_dots({})",
            args.join(", ")
        ))
    }
}

/// An R character vector literal: `character()`, `"a"` or `c("a", "b")`.
fn r_chr(values: &[String]) -> String {
    let quoted: Vec<String> = values
        .iter()
        .map(|v| format!("\"{}\"", crate::naming::r_string_escape(v)))
        .collect();
    match quoted.as_slice() {
        [] => "character()".to_string(),
        [single] => single.clone(),
        _ => format!("c({})", quoted.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(value: syn::Expr) -> DotsPolicy {
        match parse_dots_attr(&value) {
            Ok(DotsAttr::Policy(policy)) => policy,
            Ok(DotsAttr::TypedList(_)) => panic!("parsed as typed_list!"),
            Err(err) => panic!("{err}"),
        }
    }

    fn error(value: syn::Expr) -> String {
        match parse_dots_attr(&value) {
            Err(err) => err
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(" | "),
            Ok(_) => panic!("parsed"),
        }
    }

    #[test]
    fn spellings() {
        let p = policy(syn::parse_quote!(empty));
        assert_eq!(
            (p.role, p.allow.len(), p.class.len()),
            (DotsRole::Empty, 0, 0)
        );
        let p = policy(syn::parse_quote!(empty()));
        assert_eq!((p.role, p.allow.len()), (DotsRole::Empty, 0));
        let p = policy(syn::parse_quote!(empty(allow = ["row.names", "optional"])));
        assert_eq!(p.allow, ["row.names", "optional"]);
        let p = policy(syn::parse_quote!(forward(
            allow = "digits",
            class = "pkg_unused"
        )));
        assert_eq!(
            (p.role, p.allow, p.class),
            (
                DotsRole::Forward,
                vec!["digits".into()],
                vec!["pkg_unused".into()]
            )
        );
        let p = policy(syn::parse_quote!(data));
        assert_eq!(p.role, DotsRole::Data);
        assert!(matches!(
            parse_dots_attr(&syn::parse_quote!(typed_list!(x => numeric()))),
            Ok(DotsAttr::TypedList(_))
        ));
    }

    #[test]
    fn refusals_are_batched() {
        let msg = error(syn::parse_quote!(empty(
            allow = ["a", "", "a", 1, "..."],
            class = []
        )));
        assert!(msg.contains("the string is empty"), "{msg}");
        assert!(msg.contains("\"a\" is listed twice"), "{msg}");
        assert!(msg.contains("expected a string literal"), "{msg}");
        assert!(msg.contains("`...` is not an argument name"), "{msg}");
        assert!(msg.contains("the list is empty"), "{msg}");
        assert!(error(syn::parse_quote!(forward)).contains("`forward` passes the names it lists"));
        assert!(
            error(syn::parse_quote!(forward())).contains("`forward` passes the names it lists")
        );
        assert!(error(syn::parse_quote!(data(allow = "x"))).contains("`data` takes no options"));
        assert!(error(syn::parse_quote!(empty(allow = "a", allow = "b"))).contains("given twice"));
        assert!(error(syn::parse_quote!(empty(keep = "a"))).contains("unknown option"));
        assert!(error(syn::parse_quote!(nothing)).contains("dots expects"));
        assert!(error(syn::parse_quote!(other_list!(x))).contains("dots expects"));
    }

    #[test]
    fn r_check_reads_names_only() {
        let p = policy(syn::parse_quote!(empty));
        assert_eq!(
            p.r_check("object, parm, level = 0.95, ...", None).unwrap(),
            "if (...length()) .miniextendr_check_dots(...names(), ...length(), \
             c(\"object\", \"parm\", \"level\"))"
        );
        let p = policy(syn::parse_quote!(empty(
            allow = ["optional"],
            class = ["a\"b"]
        )));
        assert_eq!(
            p.r_check(".mx_self, x, ..., .call = NULL", Some("environment()"))
                .unwrap(),
            "if (...length()) .miniextendr_check_dots(...names(), ...length(), \"x\", \
             allow = \"optional\", class = \"a\\\"b\", call = environment())"
        );
        // No `...` formal: R refuses extra arguments itself.
        assert!(p.r_check("x, y", None).is_none());
        // `forward` lets unnamed arguments through.
        assert_eq!(
            policy(syn::parse_quote!(forward(allow = ["digits", "nsmall"])))
                .r_check("x, ...", None)
                .unwrap(),
            "if (...length()) .miniextendr_check_dots(...names(), ...length(), \"x\", \
             allow = c(\"digits\", \"nsmall\"), forward = TRUE)"
        );
        // `data` checks nothing.
        assert!(
            policy(syn::parse_quote!(data))
                .r_check("x, ...", None)
                .is_none()
        );
        // A default holding a comma doesn't split a formal.
        assert_eq!(
            policy(syn::parse_quote!(empty))
                .r_check("x, mode = c(\"a\", \"b\"), ...", None)
                .unwrap(),
            "if (...length()) .miniextendr_check_dots(...names(), ...length(), c(\"x\", \"mode\"))"
        );
    }
}
