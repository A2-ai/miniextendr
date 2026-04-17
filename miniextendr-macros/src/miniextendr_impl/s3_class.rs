//! S3-class R wrapper generator.

use super::ParsedImpl;

/// Generates the complete R wrapper string for an S3-style class.
///
/// Produces the following R code:
/// - Constructor: `new_<class>(...)` function that calls the Rust `new` constructor
///   and wraps the result with `structure(.val, class = "<class>")`
/// - S3 generics: for each instance method, a `UseMethod()` generic is created
///   (unless overriding an existing generic via `#[miniextendr(generic = "...")]`)
/// - S3 methods: `<generic>.<class>` functions dispatching to the Rust `.Call()` wrapper,
///   with the ExternalPtr extracted from `x`
/// - Static methods: regular functions named `<class>_<method>(...)`
/// - Class environment: `ClassName <- new.env(parent = emptyenv())` for `Class$new()`
///   syntax and trait namespace compatibility
///
/// Custom double-dispatch patterns (e.g., `vec_ptype2.a.b`) are supported via
/// `#[miniextendr(generic = "...", class = "...")]` attributes.
pub fn generate_s3_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    // S3 convention: lowercase constructor name
    let ctor_name = format!("new_{}", class_name.to_lowercase());
    let class_doc_tags = &parsed_impl.doc_tags;
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    let mut lines = Vec::new();

    // Constructor with combined class and constructor documentation
    if let Some(ctx) = parsed_impl.constructor_context() {
        lines.push(ctx.source_comment(type_ident));
        let mut ctor_doc_tags = Vec::new();
        ctor_doc_tags.extend(class_doc_tags.iter().cloned());
        ctor_doc_tags.extend(ctx.method.doc_tags.iter().cloned());

        lines.extend(
            ClassDocBuilder::new(&class_name, type_ident, &ctor_doc_tags, "S3")
                .with_export_control(parsed_impl.internal, parsed_impl.noexport)
                .build(),
        );
        // Inject lifecycle imports from methods into class-level roxygen block
        if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
            parsed_impl
                .methods
                .iter()
                .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
        ) {
            let insert_pos = lines.len().saturating_sub(1);
            lines.insert(insert_pos, format!("#' {}", lc_import));
        }
        lines.push(format!("{} <- function({}) {{", ctor_name, ctx.params));
        for line in ctx.missing_prelude() {
            lines.push(format!("  {}", line));
        }
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("  {}", line));
        }
        lines.push(format!("  .val <- {}", ctx.static_call()));
        lines.extend(crate::method_return_builder::error_in_r_check_lines("  "));
        lines.push(format!("  structure(.val, class = \"{}\")", class_name));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods as S3 generics + methods
    for ctx in parsed_impl.instance_method_contexts() {
        lines.push(ctx.source_comment(type_ident));
        let generic_name = ctx.generic_name();
        // Use custom class suffix if provided (for double-dispatch patterns like vec_ptype2.a.b)
        let method_class_suffix = ctx
            .class_suffix()
            .map(|s| s.to_string())
            .unwrap_or_else(|| class_name.clone());
        let s3_method_name = format!("{}.{}", generic_name, method_class_suffix);
        let full_params = ctx.instance_formals(true); // adds x, ..., params

        // Only create the S3 generic if no generic/class override was provided
        // (custom class suffix implies using an existing generic)
        if !ctx.has_generic_override() && !ctx.has_class_override() {
            // Create the S3 generic (only for custom generics, not base R overrides)
            if class_has_no_rd {
                lines.push("#' @noRd".to_string());
            } else {
                lines.push(format!("#' @title S3 generic for `{}`", generic_name));
                lines.push(format!("#' S3 generic for `{}`", generic_name));
                // Use class-qualified name to avoid duplicate alias when multiple
                // classes define the same S3 generic (e.g., get_value).
                lines.push(format!("#' @name {}.{}", generic_name, class_name));
                lines.push(format!("#' @rdname {}", class_name));
                lines.push("#' @param x An object".to_string());
                lines.push("#' @param ... Additional arguments passed to methods".to_string());
                lines.push(format!(
                    "#' @source Generated by miniextendr from `{}::{}`",
                    type_ident, ctx.method.ident
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
            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {{
  {generic_name} <- function(x, ...) UseMethod(\"{generic_name}\")
}}"
            ));
            lines.push(String::new());
        }

        // Then create the S3 method
        if class_has_no_rd {
            // @noRd class: minimal roxygen — just @method + @export for NAMESPACE.
            // @export on S3 methods produces S3method() in NAMESPACE (not export()).
            lines.push(format!("#' @method {} {}", generic_name, class_name));
            lines.push("#' @export".to_string());
        } else {
            let qualified_name = format!("{}.{}", generic_name, class_name);
            let method_doc =
                MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags)
                    .with_r_params(&ctx.params)
                    .with_r_name(qualified_name);
            lines.extend(method_doc.build());
            lines.push(format!("#' @method {} {}", generic_name, class_name));
            // roxygen2 can't parse generic blocks wrapped in if (!exists(...)),
            // so @param x/@param ... must also appear on the method block
            lines.push("#' @param x An object.".to_string());
            lines.push("#' @param ... Additional arguments.".to_string());
            if should_export {
                lines.push("#' @export".to_string());
            }
        }
        lines.push(format!(
            "{} <- function({}) {{",
            s3_method_name, full_params
        ));

        // Inject r_entry
        if let Some(ref entry) = ctx.method.method_attrs.r_entry {
            for line in entry.lines() {
                lines.push(format!("  {}", line));
            }
        }
        // Inject on.exit cleanup
        if let Some(ref on_exit) = ctx.method.method_attrs.r_on_exit {
            lines.push(format!("  {}", on_exit.to_r_code()));
        }
        // Inject missing param defaults
        for line in ctx.missing_prelude() {
            lines.push(format!("  {}", line));
        }
        // Inject lifecycle prelude if present
        let what = format!("{}.{}", generic_name, class_name);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("  {}", line));
        }
        // Inject r_post_checks
        if let Some(ref post) = ctx.method.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("  {}", line));
            }
        }

        let call = ctx.instance_call("x");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_chain_var("x".to_string())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
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

        let method_doc =
            MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                .with_r_params(&ctx.params)
                .with_r_name(fn_name.clone())
                .with_class_no_rd(class_has_no_rd);
        lines.extend(method_doc.build());
        // Export static methods so users can call them
        if !class_has_no_rd {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));

        // Inject r_entry
        if let Some(ref entry) = ctx.method.method_attrs.r_entry {
            for line in entry.lines() {
                lines.push(format!("  {}", line));
            }
        }
        // Inject on.exit cleanup
        if let Some(ref on_exit) = ctx.method.method_attrs.r_on_exit {
            lines.push(format!("  {}", on_exit.to_r_code()));
        }
        // Inject missing param defaults
        for line in ctx.missing_prelude() {
            lines.push(format!("  {}", line));
        }
        // Inject lifecycle prelude if present
        if let Some(prelude) = ctx.method.lifecycle_prelude(&fn_name) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("  {}", line));
        }
        // Inject r_post_checks
        if let Some(ref post) = ctx.method.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("  {}", line));
            }
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Create class environment for static methods and trait namespace compatibility
    // Check if class should be exported
    let has_no_rd = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "noRd");
    let has_internal = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "keywords internal")
        || parsed_impl.internal;
    let export_line = if !has_no_rd && !has_internal && !parsed_impl.noexport {
        "#' @export\n"
    } else {
        ""
    };
    if has_no_rd {
        lines.push(format!(
            "#' @noRd
{} <- new.env(parent = emptyenv())",
            class_name
        ));
    } else {
        lines.push(format!(
            "#' @rdname {}
{}{} <- new.env(parent = emptyenv())",
            class_name, export_line, class_name
        ));
    }
    lines.push(String::new());

    // Add $new binding to class environment (for Class$new() syntax)
    if parsed_impl.constructor_context().is_some() {
        lines.push(format!("{}$new <- {}", class_name, ctor_name));
        lines.push(String::new());
    }

    lines.join("\n")
}
