//! S4-class R wrapper generator.
//!
//! Generates `methods::setClass(...)` with an `externalptr` slot, plus
//! `methods::setGeneric` / `methods::setMethod` for each instance method.
//! Supports **formal slot validation, multi-dispatch on method signatures,
//! and contains-based inheritance** — the only system here with native
//! multi-dispatch. Cost: slowest dispatch path, all helpers live in the
//! `methods::` namespace (`methods` must be imported, not `base`), and the
//! ecosystem is increasingly legacy. Pick S4 for Bioconductor interop;
//! use S7 for new packages wanting similar formal semantics.

use super::{ParsedImpl, ParsedMethod};

/// The S4 generic an instance method dispatches through: an explicit
/// `s4(generic = "...")` verbatim, else `s4_<R method name>`, so `r_name`,
/// `postfix` and raw identifiers follow the usual naming rules. Shared by the
/// generator and `check_s4_generic_collisions`, so both agree on the names.
pub(crate) fn s4_generic_name(method: &ParsedMethod) -> String {
    match &method.method_attrs.generic {
        Some(generic) => generic.clone(),
        None => format!("s4_{}", method.r_method_name()),
    }
}

/// The line that defines an S4 instance method's generic before its
/// `setMethod()`, shared by the inherent and trait generators.
///
/// A generic of the package's own (`explicit` false) is defined unless this
/// namespace already has it. Unconditional `setGeneric()` would replace the
/// generic object and clear the methods other types registered on it, and the
/// check must be a namespace-local `exists()`: a bare `isGeneric()` searches
/// S4 metadata globally, so with an installed copy of the package attached,
/// `load_all()` would skip `setGeneric()` while `setMethod()` still can't see
/// the generic from the namespace being loaded, and `isGeneric(where =)`
/// resolves the namespace to a package name, which fails mid-install (#1158).
///
/// An explicit `generic = "..."` may name a generic that lives elsewhere:
/// `methods::show` (an S4 generic `exists()` misses when `methods` is not
/// attached), the primitive `length`, base's S3 `summary`. It is defined only
/// when no function of that name is visible and `methods::isGeneric()` finds
/// no generic of another package, with `receiver` as its dispatch formal;
/// otherwise `setMethod()` attaches to the existing one instead of masking it
/// with a `function(x, ...)` generic of this package (#1677). A generic whose
/// `@package` is this namespace's name is an installed copy's, the #1158 case,
/// so it is defined anyway. `environmentName()` is a plain lookup, unlike
/// `isGeneric(where =)`. A name nothing defines is created, but gets no
/// generated page: whether it is the package's own is only known at load time
/// (#2039).
pub(crate) fn s4_generic_guard(generic: &str, explicit: bool, receiver: &str) -> String {
    let condition = if explicit {
        format!(
            "!exists(\"{generic}\", mode = \"function\") && !(methods::isGeneric(\"{generic}\") && \
             methods::getGeneric(\"{generic}\")@package != environmentName(topenv(environment())))"
        )
    } else {
        format!("!exists(\"{generic}\", where = topenv(environment()), inherits = FALSE)")
    };
    format!(
        "if ({condition}) methods::setGeneric(\"{generic}\", function({receiver}, ...) standardGeneric(\"{generic}\"))"
    )
}

/// Generates the R wrapper of an S4-style class, in the parts the wrapper
/// writer merges per type ([`super::ClassWrapper`], #1776).
///
/// Produces the following R code:
/// - Class definition: `methods::setClass("<class>", slots = c(ptr = "externalptr"))`
///   with a single `ptr` slot holding the `ExternalPtr` to the Rust struct
/// - Constructor function: `ClassName(...)` that calls the Rust constructor
///   and wraps the result with `methods::new("<class>", ptr = .val)`. A class
///   without a constructor has no `ClassName()`: its factories call
///   `methods::new()`, and a package may define its own (#2069).
/// - S4 generics: `methods::setGeneric(...)` for each instance method, guarded by
///   [`s4_generic_guard`]
/// - S4 methods: `methods::setMethod("<generic>", "<class>", function(x, ...) ...)`
///   dispatching to the Rust `.Call()` wrapper, extracting the ptr via `x@ptr`
/// - Static methods: regular functions named `<class>_<method>(...)`
///
/// Roxygen2 `@exportMethod`, `@importFrom methods`, and `@slot` tags are generated
/// as appropriate.
///
/// The class, its constructor and the field methods are the head; the S4
/// methods and the static methods are the members.
pub fn generate_s4_class(parsed_impl: &ParsedImpl) -> super::ClassWrapper {
    use crate::r_class_formatter::{
        ClassDocBuilder, MethodContext, MethodDocBuilder, ParsedImplExt, should_export_from_tags,
    };

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    // Check if class has @noRd - if so, skip method documentation and exports. A
    // plain `noexport` (without `internal`) is folded in too — it must suppress
    // Rd contribution entirely, matching `ClassDocBuilder::build`'s `suppress_rd`
    // gate. `should_export` (below) already independently gates @export/@exportMethod.
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd")
        || (parsed_impl.noexport && !parsed_impl.internal);
    let should_export =
        should_export_from_tags(class_doc_tags, parsed_impl.noexport || parsed_impl.internal);

    let mut w = super::ClassWrapper::default();
    let mut lines = Vec::new();

    // Class definition with documentation (S4 uses setClass, no @export on class definition)
    let has_export = crate::roxygen::has_roxygen_tag(class_doc_tags, "export");
    let class_docs = ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "S4")
        .with_imports("@importFrom methods setClass setGeneric setMethod new")
        .with_export_control(parsed_impl.internal, parsed_impl.noexport);
    lines.extend(class_docs.build());
    w.class_docs = class_docs.docs();
    w.mark_class_doc_lines(&mut lines, parsed_impl);
    // Remove the @export that ClassDocBuilder adds (S4 doesn't export the class
    // definition). Only pop when the last line actually IS the auto-added
    // @export — with internal/noexport the builder emits no @export, and a
    // blind pop would instead drop `@keywords internal` / `@noRd` / a user tag.
    if !has_export
        && lines
            .last()
            .is_some_and(|l| *l == super::class_default("#' @export"))
    {
        lines.pop();
    }
    if !class_has_no_rd {
        lines.push(format!(
            "#' @slot ptr External pointer to Rust `{}` struct",
            type_ident
        ));
    }
    lines.push(format!(
        "methods::setClass(\"{}\", slots = c(ptr = \"externalptr\"))",
        class_name
    ));
    lines.push(String::new());

    // Constructor function
    if let Some(ctx) = parsed_impl.constructor_context() {
        lines.push(ctx.source_comment(type_ident));
        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            // Use class name as @name to avoid duplicate "new" alias across S4 classes
            let mx_doc = ctx.choice_param_docs();
            let method_doc =
                MethodDocBuilder::new(&class_name, "new", type_ident, &ctx.method.doc_tags)
                    .with_source_method(&ctx.method.ident)
                    .with_r_params(&ctx.params)
                    .with_choice_param_docs(&mx_doc)
                    .with_r_name(class_name.clone());
            lines.extend(method_doc.build());
        }
        // Export the constructor function so users can create instances (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", class_name, ctx.params));
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("  {}", line));
        }
        lines.push(format!("  .val <- {}", ctx.static_call()));
        lines.extend(crate::method_return_builder::condition_check_lines("  "));
        lines.push(format!("  methods::new(\"{}\", ptr = .val)", class_name));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // `$` / `$<-` field methods and `.DollarNames` of `s4(r_data_accessors)`
    // (#1848, #1885), documented on the class page wherever the class has one
    // (R CMD check wants an `\alias{$,Class-method}`).
    if parsed_impl.r_data_accessors.enabled() {
        lines.extend(super::field_syntax::s4_field_methods(
            parsed_impl,
            !class_has_no_rd,
            should_export,
        ));
    }
    w.head = std::mem::take(&mut lines);

    // Instance methods as S4 methods
    // Note: S4 uses empty param_defaults for method signatures (different from other systems)
    for method in parsed_impl.instance_methods() {
        let start = method.ident.span().start();
        let method_name = s4_generic_name(method);
        w.member_names.push(format!(
            "the S4 method of `{method_name}` for `{class_name}`"
        ));

        // Emit the generic-doc marker BEFORE the source comment so the write-time
        // pass can place the standalone Rd page before the method block.  This
        // ensures the synthesised doc block (ending with NULL) is separated from
        // the method's own roxygen comment by the `# source` line.
        if !class_has_no_rd && method.method_attrs.generic.is_none() {
            lines.push(format!(
                ".__MX_GENERIC_DOC__(kind=\"S4\", generic=\"{method_name}\", class=\"{class_name}\", export={should_export})"
            ));
        }

        lines.push(format!(
            "# {}::{} ({}:{})",
            type_ident,
            method.ident,
            start.line,
            start.column + 1,
        ));
        // Build a MethodContext so S4 methods participate in the shared
        // match_arg prelude + formal-default machinery (#209). The ctx's
        // `params`/`instance_formals` carry the `c("a", "b")` default for
        // match_arg'd params, and `match_arg_prelude()` emits the
        // `base::match.arg()` validation block injected below.
        let ctx = MethodContext::new(method, type_ident)
            .with_impl_preconditions(parsed_impl.preconditions);
        let receiver = method.method_attrs.receiver.as_deref().unwrap_or("x");
        let call = ctx.instance_call(&format!("{receiver}@ptr"));
        let full_params =
            ctx.instance_formals_with_receiver(receiver, !method.method_attrs.no_dispatch_dots);

        // Documentation for the generic - skip if class has @noRd
        // Use class-qualified @name to avoid duplicate \alias{generic} warnings
        // when multiple S4 classes share the same generic (e.g., s4_get_value on
        // both S4TraitCounter and CounterTraitS4). The @exportMethod directive
        // (added separately) correctly exports the bare generic name.
        if !class_has_no_rd {
            let qualified_name = format!("{}-{}", class_name, method_name);
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &method.doc_tags)
                    .with_source_method(&method.ident)
                    .with_suppress_params()
                    .with_r_name(qualified_name);
            let mut doc_lines = method_doc.build();
            // Add S4 method-specific alias so R CMD check finds the documented method
            doc_lines.push(format!("#' @aliases {},{}-method", method_name, class_name));
            lines.extend(doc_lines);
        }

        // Define the generic unless it exists. Several types can share one
        // (s4_get_value on both S4TraitCounter and CounterTraitS4), and an
        // explicit `generic` may be another package's (`show`).
        lines.push(s4_generic_guard(
            &method_name,
            method.method_attrs.generic.is_some(),
            receiver,
        ));

        // Define method with @exportMethod for proper S4 dispatch (if class should be exported)
        if should_export {
            lines.push(format!("#' @exportMethod {}", method_name));
        }

        let strategy = crate::ReturnStrategy::for_method(method);
        let body_lines = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_chain_var(receiver.to_string())
            .with_invisible(method.is_invisible())
            .with_class_name(class_name.clone())
            .with_return_class_from_method(method, &type_ident.to_string())
            .build_s4_body();

        let what = format!("{}.{}", method_name, class_name);
        let signature = if method.method_attrs.receiver.is_some() {
            format!("c({receiver} = \"{class_name}\")")
        } else {
            format!("\"{class_name}\"")
        };
        lines.push(format!(
            "methods::setMethod(\"{method_name}\", {signature}, function({full_params}) {{"
        ));
        ctx.emit_method_prelude(&mut lines, "  ", &what);
        lines.extend(body_lines);
        lines.push("})".to_string());
        lines.push(String::new());
    }

    // Static methods as regular functions
    for ctx in parsed_impl.static_method_contexts() {
        lines.push(ctx.source_comment(type_ident));
        let method_name = ctx.method.r_method_name();
        let fn_name = format!("{}_{}", class_name, method_name);

        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            let mx_doc = ctx.choice_param_docs();
            let doc_tags = parsed_impl.static_doc_tags(
                ctx.method,
                &format!("{fn_name}()"),
                &format!("{class_name}()"),
                true,
            );
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &doc_tags)
                    .with_source_method(&ctx.method.ident)
                    .with_r_params(&ctx.params)
                    .with_choice_param_docs(&mx_doc)
                    .with_r_name(fn_name.clone());
            lines.extend(method_doc.build());
        }
        // Export static methods so users can call them (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!(
            "{} <- function({}) {{",
            crate::naming::r_def_name(&fn_name),
            ctx.params
        ));

        ctx.emit_method_prelude(&mut lines, "  ", &fn_name);

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_expr = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_invisible(ctx.method.is_invisible())
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method, &type_ident.to_string())
            .build_s4_inline();
        lines.push(format!("  {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    w.members = lines;
    w
}

/// The S4 class as one string: [`generate_s4_class`], rendered.
#[cfg(test)]
pub fn generate_s4_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    generate_s4_class(parsed_impl).render()
}
