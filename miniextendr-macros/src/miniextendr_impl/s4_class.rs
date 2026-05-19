//! S4-class R wrapper generator.

use super::ParsedImpl;

/// Generates the complete R wrapper string for an S4-style class.
///
/// Produces the following R code:
/// - Class definition: `methods::setClass("<class>", slots = c(ptr = "externalptr"))`
///   with a single `ptr` slot holding the `ExternalPtr` to the Rust struct
/// - Constructor function: `ClassName(...)` that calls the Rust `new` constructor
///   and wraps the result with `methods::new("<class>", ptr = .val)`
/// - S4 generics: `methods::setGeneric(...)` for each instance method (idempotent,
///   always emitted rather than using conditional `isGeneric()` checks)
/// - S4 methods: `methods::setMethod("<generic>", "<class>", function(x, ...) ...)`
///   dispatching to the Rust `.Call()` wrapper, extracting the ptr via `x@ptr`
/// - Static methods: regular functions named `<class>_<method>(...)`
///
/// Roxygen2 `@exportMethod`, `@importFrom methods`, and `@slot` tags are generated
/// as appropriate.
pub fn generate_s4_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{
        ClassDocBuilder, MethodContext, MethodDocBuilder, ParsedImplExt, should_export_from_tags,
    };

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    // Check if class has @noRd - if so, skip method documentation and exports
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let should_export =
        should_export_from_tags(class_doc_tags, parsed_impl.noexport || parsed_impl.internal);

    let mut lines = Vec::new();

    // Class definition with documentation (S4 uses setClass, no @export on class definition)
    let has_export = crate::roxygen::has_roxygen_tag(class_doc_tags, "export");
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "S4")
            .with_imports("@importFrom methods setClass setGeneric setMethod new")
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
    // Remove the @export that ClassDocBuilder adds (S4 doesn't export the class definition)
    if !has_export {
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
            let mx_doc = ctx.match_arg_doc_placeholders();
            let method_doc =
                MethodDocBuilder::new(&class_name, "new", type_ident, &ctx.method.doc_tags)
                    .with_r_params(&ctx.params)
                    .with_match_arg_doc_placeholders(&mx_doc)
                    .with_r_name(class_name.clone());
            lines.extend(method_doc.build());
        }
        // Export the constructor function so users can create instances (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", class_name, ctx.params));
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
        lines.extend(crate::method_return_builder::condition_check_lines("  "));
        lines.push(format!("  methods::new(\"{}\", ptr = .val)", class_name));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods as S4 methods
    // Note: S4 uses empty param_defaults for method signatures (different from other systems)
    for method in parsed_impl.instance_methods() {
        let start = method.ident.span().start();
        lines.push(format!(
            "# {}::{} ({}:{})",
            type_ident,
            method.ident,
            start.line,
            start.column + 1,
        ));
        let method_name = if let Some(ref generic) = method.method_attrs.generic {
            generic.clone()
        } else {
            format!("s4_{}", method.ident)
        };
        // Build a MethodContext so S4 methods participate in the shared
        // match_arg prelude + formal-default machinery (#209). The ctx's
        // `params`/`instance_formals` carry the `c("a", "b")` default for
        // match_arg'd params, and `match_arg_prelude()` emits the
        // `base::match.arg()` validation block injected below.
        let ctx = MethodContext::new(method, type_ident, parsed_impl.label());
        let call = ctx.instance_call("x@ptr");
        let full_params = ctx.instance_formals(true);

        // Documentation for the generic - skip if class has @noRd
        // Use class-qualified @name to avoid duplicate \alias{generic} warnings
        // when multiple S4 classes share the same generic (e.g., s4_get_value on
        // both S4TraitCounter and CounterTraitS4). The @exportMethod directive
        // (added separately) correctly exports the bare generic name.
        if !class_has_no_rd {
            let qualified_name = format!("{}-{}", class_name, method_name);
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &method.doc_tags)
                    .with_suppress_params()
                    .with_r_name(qualified_name);
            let mut doc_lines = method_doc.build();
            // Add S4 method-specific alias so R CMD check finds the documented method
            doc_lines.push(format!("#' @aliases {},{}-method", method_name, class_name));
            lines.extend(doc_lines);
        }

        // Define generic only if it doesn't already exist. Unconditional setGeneric()
        // replaces the generic object, clearing previously registered methods. This
        // matters when multiple types share the same generic name (e.g., s4_get_value
        // used by both S4TraitCounter and CounterTraitS4).
        lines.push(format!(
            "if (!methods::isGeneric(\"{0}\")) methods::setGeneric(\"{0}\", function(x, ...) standardGeneric(\"{0}\"))",
            method_name
        ));

        // Define method with @exportMethod for proper S4 dispatch (if class should be exported)
        if should_export {
            lines.push(format!("#' @exportMethod {}", method_name));
        }

        let strategy = crate::ReturnStrategy::for_method(method);
        let body_lines = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .build_s4_body();

        let what = format!("{}.{}", method_name, class_name);
        lines.push(format!(
            "methods::setMethod(\"{}\", \"{}\", function({}) {{",
            method_name, class_name, full_params
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
            let mx_doc = ctx.match_arg_doc_placeholders();
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_r_params(&ctx.params)
                    .with_match_arg_doc_placeholders(&mx_doc)
                    .with_r_name(fn_name.clone());
            lines.extend(method_doc.build());
        }
        // Export static methods so users can call them (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));

        ctx.emit_method_prelude(&mut lines, "  ", &fn_name);

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_expr = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .build_s4_inline();
        lines.push(format!("  {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}
