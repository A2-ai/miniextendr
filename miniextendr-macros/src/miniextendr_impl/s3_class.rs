//! S3-class R wrapper generator.
//!
//! Generates **lightweight, single-dispatch** S3 generics with
//! `<generic>.<class>` methods. Method dispatch is driven by the `class()`
//! attribute vector — first match wins — so inheritance is "string-prefix"
//! based and cheap. No formal slot validation, no multi-dispatch (vctrs-style
//! double-dispatch is supported via `#[miniextendr(s3(generic = "...", class = "..."))]` for
//! `vec_ptype2.a.b` patterns). Pick S3 for tidyverse interop and for
//! extending existing base generics (`print`, `format`, `summary`); use S4/S7
//! when you need validation or formal hierarchies.

use super::ParsedImpl;

/// Generates the R wrapper of an S3-style class, in the parts the wrapper
/// writer merges per type ([`super::ClassWrapper`], #1776).
///
/// Produces the following R code:
/// - Constructor: `new_<class>(...)` function that calls the Rust constructor
///   and wraps the result with `structure(.val, class = "<class>")`; none for
///   a class without a constructor (see [`no_constructor_head`])
/// - S3 generics: for each instance method, a `UseMethod()` generic is created
///   (unless overriding an existing generic via `#[miniextendr(generic = "...")]`)
/// - S3 methods: `<generic>.<class>` functions dispatching to the Rust `.Call()` wrapper,
///   with the ExternalPtr extracted from `x`
/// - Static methods: regular functions named `<class>_<method>(...)`
/// - Class environment: `ClassName <- new.env(parent = emptyenv())` for `Class$new()`
///   syntax and trait namespaces, when the impl has a constructor or the class
///   is named as its Rust type (#1918)
///
/// Custom double-dispatch patterns (e.g., `vec_ptype2.a.b`) are supported via
/// `#[miniextendr(generic = "...", class = "...")]` attributes.
///
/// The constructor and the field methods are the head; the S3 methods and the
/// static methods are the members; the class environment is the tail.
pub fn generate_s3_class(parsed_impl: &ParsedImpl) -> super::ClassWrapper {
    use crate::r_class_formatter::{
        ClassDocBuilder, MethodDocBuilder, ParsedImplExt, emit_s3_generic_guard,
        should_export_from_tags,
    };

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    // S3 convention: lowercase constructor name
    let ctor_name = format!("new_{}", class_name.to_lowercase());
    let class_doc_tags = &parsed_impl.doc_tags;
    // A plain `noexport` (without `internal`) is folded into the @noRd gate too —
    // it must suppress Rd contribution entirely, matching `ClassDocBuilder::build`'s
    // `suppress_rd` gate. `@method`/`@export` tags stay preserved for S3 dispatch
    // registration by the existing @noRd-handling branches below.
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd")
        || (parsed_impl.noexport && !parsed_impl.internal);
    // Generic export (NAMESPACE `export(generic_name)`): suppressed by
    // @noRd / internal / noexport. An internal class doesn't pollute the
    // package's user-facing surface with a bare generic.
    let should_export =
        should_export_from_tags(class_doc_tags, parsed_impl.noexport || parsed_impl.internal);
    // Method @export (NAMESPACE `S3method(generic, Class)`): only suppressed
    // by `noexport`. `internal` should keep S3method registration so dispatch
    // still works for instances of the class — without it, callers (including
    // the package's own tests) couldn't dispatch on the type at all. See #431.
    let should_register_s3method = !parsed_impl.noexport;

    let mut w = super::ClassWrapper::default();
    let mut lines = Vec::new();
    // The class block is the constructor's; a block without one still
    // documents the class with its impl docs.
    crate::roxygen::push_roxygen_tags(&mut w.class_docs, class_doc_tags);

    // Constructor with combined class and constructor documentation
    if let Some(ctx) = parsed_impl.constructor_context() {
        w.member_names.push(format!("`{class_name}$new`"));
        lines.push(ctx.source_comment(type_ident));
        let mut ctor_doc_tags = Vec::new();
        ctor_doc_tags.extend(class_doc_tags.iter().cloned());
        ctor_doc_tags.extend(ctx.method.doc_tags.iter().cloned());
        let ctor_choice_docs = ctx.choice_param_docs();

        let class_docs = ClassDocBuilder::new(&class_name, type_ident, &ctor_doc_tags, "S3")
            .with_ctor_params(&ctx.params, &ctor_choice_docs)
            .with_export_control(parsed_impl.internal, parsed_impl.noexport);
        lines.extend(class_docs.build());
        w.class_docs = class_docs.docs();
        w.mark_class_doc_lines(&mut lines, parsed_impl);
        lines.push(format!("{} <- function({}) {{", ctor_name, ctx.params));
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("  {}", line));
        }
        lines.push(format!("  .val <- {}", ctx.static_call()));
        lines.extend(crate::method_return_builder::condition_check_lines("  "));
        lines.push(format!("  structure(.val, class = \"{}\")", class_name));
        lines.push("}".to_string());
        lines.push(String::new());
    } else {
        no_constructor_head(parsed_impl, &mut w, &mut lines, "S3", None);
    }

    // The field methods of `s3(r_data_accessors)` (`$` / `[[` / `names` /
    // `as.list` / `.DollarNames`, and the setters; #1848, #1890, #1885),
    // ahead of the instance methods: on a shared page roxygen2 keeps the last
    // `@param` of a name, so a method's own text for `name` or `value` wins.
    // They are documented whenever the class is: `internal` keeps the class
    // out of the exports, and its page (`\keyword{internal}`) still documents
    // the registered methods, as for S4 (#1918).
    if parsed_impl.r_data_accessors.enabled() {
        lines.extend(super::field_syntax::s3_field_methods(
            parsed_impl,
            !class_has_no_rd,
            should_register_s3method,
        ));
    }
    w.head = std::mem::take(&mut lines);

    // Instance methods as S3 generics + methods. Their conditions report the
    // generic's call, resolved from the method frame on a raise (#1851).
    for ctx in parsed_impl.instance_method_contexts() {
        let ctx = ctx.with_call_attribution(crate::r_wrapper_builder::CallAttribution::Generic);
        lines.push(ctx.source_comment(type_ident));
        let generic_name = ctx.generic_name();
        // Use custom class suffix if provided (for double-dispatch patterns like vec_ptype2.a.b)
        let method_class_suffix = ctx
            .class_suffix()
            .map(|s| s.to_string())
            .unwrap_or_else(|| class_name.clone());
        let s3_method_name = format!("{}.{}", generic_name, method_class_suffix);
        let receiver = ctx.method.method_attrs.receiver.as_deref().unwrap_or("x");
        let full_params = ctx.s3_method_formals(); // x, params, ... (`value` last for `*<-`)

        // Only create the S3 generic if no generic/class override was provided
        // (custom class suffix implies using an existing generic)
        if !ctx.has_generic_override() && !ctx.has_class_override() {
            // Create the S3 generic (only for custom generics, not base R overrides)
            if class_has_no_rd {
                lines.push("#' @noRd".to_string());
            } else {
                // The generic block is named after the S3 method (`generic.Class`,
                // class-qualified so classes sharing a generic don't collide on
                // `\alias{generic}`). That is the same alias the method block
                // below produces, so both must land on the same page: a
                // method-level `@rdname` or `@describeIn` moves the generic block
                // along with the method, otherwise R CMD check reports the alias
                // as duplicated across the class page and the method's page.
                let method_tags = &ctx.method.doc_tags;
                let joined = crate::roxygen::joins_author_topic(method_tags, Some(&class_name));
                if !joined {
                    // roxygen2 keeps the first `@title` on a page. On the class
                    // page that is the class block's, so this one is inert; on a
                    // split page it would beat the method's structural title.
                    // The author's topic describes the method itself, so the
                    // structural description would only add a stray paragraph.
                    lines.push(format!("#' @title S3 generic for `{}`", generic_name));
                    lines.push(format!("#' @description S3 generic for `{}`", generic_name));
                }
                lines.push(format!("#' @name {}.{}", generic_name, class_name));
                crate::roxygen::push_rdname_or_default(&mut lines, method_tags, &class_name);
                if !joined {
                    // On the author's topic its own block documents `x` and
                    // `...`; a generated line would replace that text (#1590).
                    lines.push("#' @param x An object".to_string());
                    lines.push("#' @param ... Additional arguments passed to methods".to_string());
                }
                crate::roxygen::push_order_after_topic_blocks(&mut lines, method_tags, &class_name);
                lines.extend(crate::roxygen::method_source_tag(
                    type_ident,
                    &ctx.method.ident,
                ));
                if should_export {
                    // Explicit name on @export: the generic is wrapped in
                    // `if (!exists(...))`, which roxygen2 can't introspect, and the
                    // @name tag above is the class-qualified form (to dedupe aliases
                    // when several classes share a generic). Without an explicit
                    // target on @export, roxygen2 attaches the export to the next
                    // parseable function (the S3 method), producing a bogus
                    // `export(generic.Class)` instead of `export(generic)`.
                    lines.push(format!("#' @export {}", generic_name));
                }
            }
            lines.push(emit_s3_generic_guard(&generic_name));
            lines.push(String::new());
        }

        // Then create the S3 method
        if class_has_no_rd {
            // @noRd class: minimal roxygen — just @method (+ @export for NAMESPACE
            // dispatch registration, gated by `should_register_s3method` so a
            // `noexport`-driven suppression — folded into `class_has_no_rd` above —
            // also drops S3 dispatch, not just Rd docs; a plain user `@noRd` with
            // no `noexport` attr still registers dispatch as before, since
            // `should_register_s3method` is `true` whenever `noexport` isn't set).
            // @export on S3 methods produces S3method() in NAMESPACE (not export()).
            lines.push(format!("#' @method {} {}", generic_name, class_name));
            if should_register_s3method {
                lines.push("#' @export".to_string());
            }
        } else {
            let qualified_name = format!("{}.{}", generic_name, class_name);
            let mx_doc = ctx.choice_param_docs();
            let method_doc =
                MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags)
                    .with_source_method(&ctx.method.ident)
                    .with_r_params(&ctx.params)
                    .with_choice_param_docs(&mx_doc)
                    .with_r_name(qualified_name);
            lines.extend(method_doc.build());
            lines.push(format!("#' @method {} {}", generic_name, class_name));
            // roxygen2 can't parse generic blocks wrapped in if (!exists(...)),
            // so @param x/@param ... must also appear on the method block. On
            // an author topic (method-level `@rdname` / `@describeIn`) the
            // topic's own block documents them instead: roxygen2 keeps the
            // later `@param` of a name, and this block sorts last (#1590).
            if !crate::roxygen::joins_author_topic(&ctx.method.doc_tags, Some(&class_name)) {
                lines.push(format!("#' @param {receiver} An object."));
                if !ctx.method.method_attrs.no_dispatch_dots || ctx.method.has_dots {
                    lines.push("#' @param ... Additional arguments.".to_string());
                }
            }
            if should_register_s3method {
                lines.push("#' @export".to_string());
            }
        }
        lines.push(format!(
            "{} <- function({}) {{",
            crate::naming::r_def_name(&s3_method_name),
            full_params
        ));

        let what = format!("{}.{}", generic_name, class_name);
        ctx.emit_method_prelude(&mut lines, "  ", &what);

        let call = ctx.instance_call(receiver);
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_raise_default(ctx.call_attribution.raise_default())
            .with_strategy(strategy)
            .with_invisible(ctx.method.is_invisible())
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method, &type_ident.to_string())
            .with_chain_var(receiver.to_string());
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods as regular functions
    for ctx in parsed_impl.static_method_contexts() {
        lines.push(ctx.source_comment(type_ident));
        // Static methods get a prefix to avoid naming conflicts
        let method_name = ctx.method.r_method_name();
        let fn_name = format!("{}_{}", class_name.to_lowercase(), method_name);

        let mx_doc = ctx.choice_param_docs();
        let doc_tags = parsed_impl.static_doc_tags(
            ctx.method,
            &format!("{fn_name}()"),
            &format!("{ctor_name}()"),
            true,
        );
        let method_doc = MethodDocBuilder::new(&class_name, &method_name, type_ident, &doc_tags)
            .with_source_method(&ctx.method.ident)
            .with_r_params(&ctx.params)
            .with_choice_param_docs(&mx_doc)
            .with_r_name(fn_name.clone())
            .with_class_no_rd(class_has_no_rd);
        lines.extend(method_doc.build());
        // Export static methods so users can call them
        if !class_has_no_rd {
            lines.push("#' @export".to_string());
        }

        lines.push(format!(
            "{} <- function({}) {{",
            crate::naming::r_def_name(&fn_name),
            ctx.params
        ));

        ctx.emit_method_prelude(&mut lines, "  ", &fn_name);

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_invisible(ctx.method.is_invisible())
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method, &type_ident.to_string());
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }
    w.members = std::mem::take(&mut lines);

    // The class environment holds `Class$new()` and the trait namespaces
    // (`Type$Trait`, named by the Rust type, for a trait's static methods and
    // consts; static methods of the impl are plain functions). Emit it only
    // when one of them can land in it: an impl with a constructor, or a class
    // named as its Rust type. An impl that only adds the field readers to a
    // class the package names and builds itself gets none, so no second
    // binding of a same-named constructor, export or `\alias` (#1918).
    let has_class_env = parsed_impl.constructor_context().is_some() || *type_ident == class_name;
    let export_line = if should_export { "#' @export\n" } else { "" };
    if has_class_env {
        if class_has_no_rd {
            lines.push(format!(
                "#' @noRd
{} <- new.env(parent = emptyenv())",
                class_name
            ));
        } else {
            // On the impl's `@rdname` topic when it names one, after the topic's
            // own block (#1900).
            let topic = super::field_syntax::topic(parsed_impl);
            let order = if topic == class_name {
                String::new()
            } else {
                format!("#' {}\n", crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS)
            };
            lines.push(format!(
                "#' @rdname {topic}\n{order}{export_line}{class_name} <- new.env(parent = emptyenv())"
            ));
        }
        lines.push(String::new());
    }

    // Add $new binding to class environment (for Class$new() syntax)
    if parsed_impl.constructor_context().is_some() {
        lines.push(format!("{}$new <- {}", class_name, ctor_name));
        lines.push(String::new());
    }

    w.tail = lines;
    w
}

/// The S3 class as one string: [`generate_s3_class`], rendered.
#[cfg(test)]
pub fn generate_s3_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    generate_s3_class(parsed_impl).render()
}

/// The head of an S3 or vctrs class without a constructor: `kind` is the
/// class system in its title, `imports` its roxygen imports. Neither gets a
/// `new_<class>()`: a package that builds its objects in R may write its own
/// (#2069).
///
/// - A class named as its Rust type has its class block on `NULL`, without
///   an `@export` of the class name: S3's class environment exports it, and
///   vctrs has no object of that name.
/// - A class the package names and builds itself (`class = "..."`) has no
///   class environment either (#1918). Its roxygen block only carries the
///   impl docs ([`super::docs_only_class_block`]).
pub(super) fn no_constructor_head(
    parsed_impl: &ParsedImpl,
    w: &mut super::ClassWrapper,
    lines: &mut Vec<String>,
    kind: &'static str,
    imports: Option<&str>,
) {
    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    if *type_ident != class_name {
        if let Some(imports) = imports {
            lines.push(format!("#' {imports}"));
            lines.push("NULL".to_string());
            lines.push(String::new());
        }
        lines.extend(super::docs_only_class_block(parsed_impl, kind));
        w.mark_class_doc_lines(lines, parsed_impl);
        lines.push(String::new());
        return;
    }
    let mut class_docs = crate::r_class_formatter::ClassDocBuilder::new(
        &class_name,
        type_ident,
        &parsed_impl.doc_tags,
        kind,
    )
    .with_export_control(parsed_impl.internal, parsed_impl.noexport);
    if let Some(imports) = imports {
        class_docs = class_docs.with_imports(imports);
    }
    let export = super::class_default("#' @export");
    lines.extend(
        class_docs
            .build()
            .into_iter()
            .filter(|line| *line != export),
    );
    w.class_docs = class_docs.docs();
    lines.push("NULL".to_string());
    w.mark_class_doc_lines(lines, parsed_impl);
    lines.push(String::new());
}
