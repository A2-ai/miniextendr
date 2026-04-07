//! Vctrs class R wrapper generator.

use super::{ParsedImpl, VctrsKind};

/// Generates the complete R wrapper string for a vctrs-compatible S3 class.
///
/// This is used when an `impl` block is annotated with `#[miniextendr(vctrs)]`.
/// Unlike the `#[derive(Vctrs)]` macro (which generates standalone S3 methods from
/// struct attributes), this generator produces class wrappers from `impl` block methods.
///
/// Produces the following R code:
/// - Constructor: `new_<class>(...)` that calls the Rust `new` constructor, then wraps
///   the result with `vctrs::new_vctr()`, `vctrs::new_rcrd()`, or `vctrs::new_list_of()`
///   depending on the `VctrsKind`
/// - `vec_ptype_abbr.<class>`: compact abbreviation for printing (if `abbr` is specified)
/// - `vec_ptype2.<class>.<class>`: self-coercion prototype (returns empty typed vector)
/// - `vec_cast.<class>.<class>`: identity cast (returns `x` unchanged)
/// - Instance methods: S3 generics + `<generic>.<class>` methods, with support for
///   vctrs protocol overrides via `#[miniextendr(vctrs_protocol = "...")]` and
///   double-dispatch class suffixes via `#[miniextendr(class = "...")]`
/// - Static methods: regular functions named `<class>_<method>(...)`
///
/// Roxygen2 documentation and `@importFrom vctrs ...` tags are generated automatically.
pub fn generate_vctrs_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    let vctrs_attrs = &parsed_impl.vctrs_attrs;
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    // Constructor name follows vctrs convention: new_<class>
    let ctor_name = format!("new_{}", class_name.to_lowercase());

    let mut lines = Vec::new();

    // Constructor with combined class and constructor documentation
    if let Some(ctx) = parsed_impl.constructor_context() {
        let mut ctor_doc_tags = Vec::new();
        ctor_doc_tags.extend(class_doc_tags.iter().cloned());
        ctor_doc_tags.extend(ctx.method.doc_tags.iter().cloned());

        lines.extend(
            ClassDocBuilder::new(&class_name, type_ident, &ctor_doc_tags, "vctrs S3")
                .with_imports("@importFrom vctrs new_vctr new_rcrd new_list_of vec_ptype2 vec_cast vec_ptype_abbr")
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

        // Generate constructor body based on vctrs kind
        lines.push(format!("{} <- function({}) {{", ctor_name, ctx.params));
        for line in ctx.missing_prelude() {
            lines.push(format!("  {}", line));
        }
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        lines.push(format!("  .val <- {}", ctx.static_call()));
        lines.extend(crate::method_return_builder::error_in_r_check_lines("  "));
        lines.push("  data <- .val".to_string());

        match vctrs_attrs.kind {
            VctrsKind::Vctr => {
                // Build new_vctr call with optional inherit_base_type
                let inherit_arg = match vctrs_attrs.inherit_base_type {
                    Some(true) => ", inherit_base_type = TRUE",
                    Some(false) => ", inherit_base_type = FALSE",
                    None => "",
                };
                lines.push(format!(
                    "  vctrs::new_vctr(data, class = \"{}\"{})",
                    class_name, inherit_arg
                ));
            }
            VctrsKind::Rcrd => {
                // Record type - data should be a list
                lines.push(format!(
                    "  vctrs::new_rcrd(data, class = \"{}\")",
                    class_name
                ));
            }
            VctrsKind::ListOf => {
                // list_of - needs ptype
                let ptype_arg = vctrs_attrs
                    .ptype
                    .as_ref()
                    .map(|p| format!(", ptype = {}", p))
                    .unwrap_or_default();
                lines.push(format!(
                    "  vctrs::new_list_of(data, class = \"{}\"{})",
                    class_name, ptype_arg
                ));
            }
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // vec_ptype_abbr for compact printing (if abbr is specified)
    if let Some(abbr) = &vctrs_attrs.abbr {
        lines.push(format!("#' @rdname {}", class_name));
        lines.push(format!("#' @method vec_ptype_abbr {}", class_name));
        if should_export {
            lines.push("#' @export".to_string());
        }
        lines.push(format!(
            "vec_ptype_abbr.{} <- function(x, ...) \"{}\"",
            class_name, abbr
        ));
        lines.push(String::new());
    }

    // Self-coercion methods (required for vctrs to work properly)
    // vec_ptype2.<class>.<class> - returns prototype for combining same types
    lines.push(format!("#' @rdname {}", class_name));
    lines.push(format!(
        "#' @method vec_ptype2 {}.{}",
        class_name, class_name
    ));
    lines.push(format!("#' @param x A {} vector.", class_name));
    lines.push(format!("#' @param y A {} vector.", class_name));
    lines.push("#' @param ... Additional arguments (unused).".to_string());
    if should_export {
        lines.push("#' @export".to_string());
    }
    match vctrs_attrs.kind {
        VctrsKind::Vctr => {
            let base_type = vctrs_attrs
                .base
                .as_ref()
                .map(|b| format!("{}()", b))
                .unwrap_or_else(|| "double()".to_string());
            let inherit_arg = match vctrs_attrs.inherit_base_type {
                Some(true) => ", inherit_base_type = TRUE",
                Some(false) => ", inherit_base_type = FALSE",
                None => "",
            };
            lines.push(format!(
                "vec_ptype2.{c}.{c} <- function(x, y, ...) vctrs::new_vctr({base}, class = \"{c}\"{inherit})",
                c = class_name,
                base = base_type,
                inherit = inherit_arg
            ));
        }
        VctrsKind::Rcrd => {
            // For records, return empty record with same field structure
            lines.push(format!(
                "vec_ptype2.{c}.{c} <- function(x, y, ...) x[0]",
                c = class_name
            ));
        }
        VctrsKind::ListOf => {
            let ptype_arg = vctrs_attrs
                .ptype
                .as_ref()
                .map(|p| format!(", ptype = {}", p))
                .unwrap_or_default();
            lines.push(format!(
                "vec_ptype2.{c}.{c} <- function(x, y, ...) vctrs::new_list_of(list(), class = \"{c}\"{ptype})",
                c = class_name,
                ptype = ptype_arg
            ));
        }
    }
    lines.push(String::new());

    // vec_cast.<class>.<class> - identity cast (no-op for same type)
    lines.push(format!("#' @rdname {}", class_name));
    lines.push(format!("#' @method vec_cast {}.{}", class_name, class_name));
    lines.push(format!("#' @param x A {} vector to cast.", class_name));
    lines.push(format!("#' @param to A {} prototype.", class_name));
    lines.push("#' @param ... Additional arguments (unused).".to_string());
    if should_export {
        lines.push("#' @export".to_string());
    }
    lines.push(format!(
        "vec_cast.{c}.{c} <- function(x, to, ...) x",
        c = class_name
    ));
    lines.push(String::new());

    // Instance methods as S3 generics + methods
    for ctx in parsed_impl.instance_method_contexts() {
        // vctrs protocol override: use the protocol name as the S3 generic
        let is_protocol = ctx.method.method_attrs.vctrs_protocol.is_some();
        let generic_name = if let Some(ref proto) = ctx.method.method_attrs.vctrs_protocol {
            proto.clone()
        } else {
            ctx.generic_name()
        };
        // Use custom class suffix if provided (for double-dispatch patterns like vec_ptype2.a.b)
        let method_class_suffix = ctx
            .class_suffix()
            .map(|s| s.to_string())
            .unwrap_or_else(|| class_name.clone());
        let s3_method_name = format!("{}.{}", generic_name, method_class_suffix);
        let full_params = ctx.instance_formals(true); // adds x, ..., params

        // Only create the S3 generic if no generic/class override was provided
        // vctrs protocol methods use existing generics from the vctrs package
        if !is_protocol && !ctx.has_generic_override() && !ctx.has_class_override() {
            lines.push(format!("#' @title S3 generic for `{}`", generic_name));
            lines.push(format!("#' S3 generic for `{}`", generic_name));
            lines.push(format!("#' @rdname {}", class_name));
            lines.push(format!("#' @name {}", generic_name));
            lines.push("#' @param x An object".to_string());
            lines.push("#' @param ... Additional arguments passed to methods".to_string());
            lines.push(format!(
                "#' @source Generated by miniextendr from `{}::{}`",
                type_ident, ctx.method.ident
            ));
            if should_export {
                lines.push("#' @export".to_string());
            }
            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {{
  {generic_name} <- function(x, ...) UseMethod(\"{generic_name}\")
}}"
            ));
            lines.push(String::new());
        }

        // Then create the S3 method
        let method_doc =
            MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags)
                .with_r_params(&ctx.params);
        lines.extend(method_doc.build());
        lines.push(format!(
            "#' @method {} {}",
            generic_name, method_class_suffix
        ));
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
        let method_name = ctx.method.r_method_name();
        let fn_name = format!("{}_{}", class_name.to_lowercase(), method_name);

        let method_doc =
            MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                .with_r_params(&ctx.params)
                .with_r_name(fn_name.clone());
        lines.extend(method_doc.build());

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

    lines.join("\n")
}
