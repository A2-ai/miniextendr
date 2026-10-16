//! Vctrs class R wrapper generator.
//!
//! Generates an S3 class that **integrates with the tidyverse `vctrs`
//! package**: emits a `new_<class>` wrapper around `vctrs::new_vctr` /
//! `new_rcrd` / `new_list_of` (selected via `VctrsKind`), plus the standard
//! `vec_ptype2` / `vec_cast` self-coercion methods so the type composes
//! cleanly inside `tibble`, `dplyr`, and `tidyr`. Pick vctrs when your Rust
//! type is "a vector of X" and you want first-class tibble columns; use
//! plain S3/S7 for scalar-like objects.

use super::{ParsedImpl, VctrsKind};

/// The roxygen imports of a vctrs class's roxygen block.
const VCTRS_IMPORTS: &str =
    "@importFrom vctrs new_vctr new_rcrd new_list_of vec_ptype2 vec_cast vec_ptype_abbr";

/// Generates the R wrapper of a vctrs-compatible S3 class, in the parts the
/// wrapper writer merges per type ([`super::ClassWrapper`], #1776).
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
/// - Static methods: regular functions named `<class>_<method>(...)`, or
///   `<protocol>.<class>` S3 methods with `#[miniextendr(vctrs(<protocol>))]`
///   (instance methods are refused, MXL120)
///
/// Roxygen2 documentation and `@importFrom vctrs ...` tags are generated automatically.
/// A class-level `@noRd` (or a plain `noexport` without `internal`) suppresses the Rd
/// contribution of every block — self-coercion methods and static method docs
/// all collapse to `@noRd` (#1180), like the other five
/// class-system generators. S3 `S3method()` dispatch registration (`@method` +
/// `@export`) is kept unconditionally: vctrs generics dispatch from the vctrs
/// namespace, so an unregistered method would leave even a gated class non-functional
/// (and roxygen2 warns on any recognized-but-unregistered S3 method).
///
/// The constructor, `vec_ptype_abbr` and the self-coercion methods are the
/// head; the static methods and protocol methods are the members.
pub fn generate_vctrs_class(parsed_impl: &ParsedImpl) -> super::ClassWrapper {
    use crate::r_class_formatter::{
        ClassDocBuilder, MethodDocBuilder, ParsedImplExt, should_export_from_tags,
    };

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    let vctrs_attrs = &parsed_impl.vctrs_attrs;
    let should_export =
        should_export_from_tags(class_doc_tags, parsed_impl.noexport || parsed_impl.internal);
    // Check if the class has @noRd — if so, skip method documentation (#1180). A
    // plain `noexport` (without `internal`) is folded into the gate too: it must
    // suppress Rd contribution entirely (audit A10 semantics), matching the other
    // five class generators. `internal` keeps full docs (the constructor's
    // ClassDocBuilder adds `@keywords internal`); `should_export` (above) gates
    // the export()-shaped surface (static helpers) and is always false when
    // this gate is true.
    //
    // S3-method-shaped emissions (`vec_ptype_abbr`/`vec_ptype2`/`vec_cast`,
    // protocol overrides) keep their `@method` + `@export`
    // pair unconditionally: with `@method`, `@export` produces an `S3method()`
    // dispatch registration in NAMESPACE — not an `export()` — and both
    // roxygen2 (`warn_missing_s3_exports`) and vctrs itself (generics dispatch
    // from the vctrs namespace) require registered methods. A gated class stays
    // hidden (no Rd page, no `export()` entries) but remains a functioning vctr.
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd")
        || (parsed_impl.noexport && !parsed_impl.internal);

    // Constructor name follows vctrs convention: new_<class>
    let ctor_name = format!("new_{}", class_name.to_lowercase());

    let mut w = super::ClassWrapper::default();
    let mut lines = Vec::new();
    // The class block is the constructor's; a block without one still
    // documents the class with its impl docs.
    crate::roxygen::push_roxygen_tags(&mut w.class_docs, class_doc_tags);

    // Constructor with combined class and constructor documentation
    if let Some(ctx) = parsed_impl.constructor_context() {
        lines.push(ctx.source_comment(type_ident));
        let mut ctor_doc_tags = Vec::new();
        ctor_doc_tags.extend(class_doc_tags.iter().cloned());
        ctor_doc_tags.extend(ctx.method.doc_tags.iter().cloned());
        let ctor_choice_docs = ctx.choice_param_docs();

        let class_docs = ClassDocBuilder::new(&class_name, type_ident, &ctor_doc_tags, "vctrs S3")
            .with_ctor_params(&ctx.params, &ctor_choice_docs)
            .with_imports(VCTRS_IMPORTS)
            .with_export_control(parsed_impl.internal, parsed_impl.noexport);
        lines.extend(class_docs.build());
        w.class_docs = class_docs.docs();
        w.mark_class_doc_lines(&mut lines, parsed_impl);

        // Generate constructor body based on vctrs kind
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
    } else {
        super::s3_class::no_constructor_head(
            parsed_impl,
            &mut w,
            &mut lines,
            "vctrs S3",
            Some(VCTRS_IMPORTS),
        );
    }

    // vec_ptype_abbr for compact printing (if abbr is specified)
    if let Some(abbr) = &vctrs_attrs.abbr {
        if class_has_no_rd {
            // Gated class (@noRd / plain noexport): no Rd contribution — keep
            // only the @method + @export S3method() registration pair.
            lines.push("#' @noRd".to_string());
        } else {
            lines.push(super::member_default(format!("#' @rdname {}", class_name)));
        }
        lines.push(format!("#' @method vec_ptype_abbr {}", class_name));
        lines.push("#' @export".to_string());
        lines.push(format!(
            "vec_ptype_abbr.{} <- function(x, ...) \"{}\"",
            class_name, abbr
        ));
        lines.push(String::new());
    }

    // Self-coercion methods (required for vctrs to work properly)
    // vec_ptype2.<class>.<class> - returns prototype for combining same types
    if class_has_no_rd {
        lines.push("#' @noRd".to_string());
        lines.push(format!(
            "#' @method vec_ptype2 {}.{}",
            class_name, class_name
        ));
    } else {
        lines.push(super::member_default(format!("#' @rdname {}", class_name)));
        lines.push(format!(
            "#' @method vec_ptype2 {}.{}",
            class_name, class_name
        ));
        lines.push(super::member_default(format!(
            "#' @param x A {} vector.",
            class_name
        )));
        lines.push(super::member_default(format!(
            "#' @param y A {} vector.",
            class_name
        )));
        lines.push(super::member_default(
            "#' @param ... Additional arguments (unused).".to_string(),
        ));
    }
    lines.push("#' @export".to_string());
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
    if class_has_no_rd {
        lines.push("#' @noRd".to_string());
        lines.push(format!("#' @method vec_cast {}.{}", class_name, class_name));
    } else {
        lines.push(super::member_default(format!("#' @rdname {}", class_name)));
        lines.push(format!("#' @method vec_cast {}.{}", class_name, class_name));
        lines.push(super::member_default(format!(
            "#' @param x A {} vector to cast.",
            class_name
        )));
        lines.push(super::member_default(format!(
            "#' @param to A {} prototype.",
            class_name
        )));
        lines.push(super::member_default(
            "#' @param ... Additional arguments (unused).".to_string(),
        ));
    }
    lines.push("#' @export".to_string());
    lines.push(format!(
        "vec_cast.{c}.{c} <- function(x, to, ...) x",
        c = class_name
    ));
    lines.push(String::new());
    w.head = std::mem::take(&mut lines);

    // No instance methods: `ParsedImpl::parse` refuses every receiver on a
    // vctrs impl (MXL120), since the R object is the vector, not a pointer.

    // Static methods as regular functions, or as vctrs protocol S3 methods when
    // `#[miniextendr(vctrs(protocol))]` is set (e.g. `vctrs(format)` → `format.<Class>`).
    for ctx in parsed_impl.static_method_contexts() {
        lines.push(ctx.source_comment(type_ident));

        let is_protocol = ctx.method.method_attrs.vctrs_protocol.is_some();
        let fn_name = if let Some(ref proto) = ctx.method.method_attrs.vctrs_protocol {
            // vctrs protocol override: emit as `<protocol>.<Class>` S3 method
            format!("{}.{}", proto, class_name)
        } else {
            let method_name = ctx.method.r_method_name();
            format!("{}_{}", class_name.to_lowercase(), method_name)
        };
        let r_name = fn_name.clone();

        let mx_doc = ctx.choice_param_docs();
        let method_name = ctx.method.r_method_name();
        if is_protocol {
            let proto = ctx.method.method_attrs.vctrs_protocol.as_ref().unwrap();
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_source_method(&ctx.method.ident)
                    .with_r_params(&ctx.params)
                    .with_choice_param_docs(&mx_doc)
                    .with_r_name(r_name.clone())
                    .with_class_no_rd(class_has_no_rd);
            lines.extend(method_doc.build());
            lines.push(format!("#' @method {} {}", proto, class_name));
            lines.push("#' @export".to_string());
        } else {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_source_method(&ctx.method.ident)
                    .with_r_params(&ctx.params)
                    .with_choice_param_docs(&mx_doc)
                    .with_r_name(r_name.clone())
                    .with_class_no_rd(class_has_no_rd);
            lines.extend(method_doc.build());
            // Export the helper so users can call it, as the S3 / S4 / S7
            // generators do for their static methods; without it the
            // function was documented on the class page but unreachable.
            if should_export {
                lines.push("#' @export".to_string());
            }
        }

        // Protocol methods accept `...` so `format(x, nsmall = 2)` and similar
        // S3 dispatch calls with extra arguments don't error with "unused argument".
        // The `...` is silently dropped; the underlying Rust function has a fixed signature.
        let formals = if is_protocol && !ctx.method.has_dots {
            if ctx.params.is_empty() {
                "...".to_string()
            } else {
                format!("{}, ...", ctx.params)
            }
        } else {
            ctx.params.to_string()
        };
        lines.push(format!(
            "{} <- function({}) {{",
            crate::naming::r_def_name(&fn_name),
            formals
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

    w.members = lines;
    w
}

/// The vctrs class as one string: [`generate_vctrs_class`], rendered.
#[cfg(test)]
pub fn generate_vctrs_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    generate_vctrs_class(parsed_impl).render()
}
