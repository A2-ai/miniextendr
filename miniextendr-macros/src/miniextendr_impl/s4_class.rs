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
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    // Check if class has @noRd - if so, skip method documentation and exports
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

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
        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            // Use class name as @name to avoid duplicate "new" alias across S4 classes
            let method_doc =
                MethodDocBuilder::new(&class_name, "new", type_ident, &ctx.method.doc_tags)
                    .with_r_params(&ctx.params)
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
        lines.push(format!("  .val <- {}", ctx.static_call()));
        lines.extend(crate::method_return_builder::error_in_r_check_lines("  "));
        lines.push(format!("  methods::new(\"{}\", ptr = .val)", class_name));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods as S4 methods
    // Note: S4 uses empty param_defaults for method signatures (different from other systems)
    for method in parsed_impl.instance_methods() {
        let c_ident = method.c_wrapper_ident(type_ident, parsed_impl.label());
        let method_name = if let Some(ref generic) = method.method_attrs.generic {
            generic.clone()
        } else {
            format!("s4_{}", method.ident)
        };
        // S4 methods use empty defaults for consistency with setMethod
        let params = crate::r_wrapper_builder::build_r_formals_from_sig(
            &method.sig,
            &std::collections::HashMap::new(),
        );
        let args = crate::r_wrapper_builder::build_r_call_args_from_sig(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), x@ptr)", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), x@ptr, {})", c_ident, args)
        };
        let full_params = if params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", params)
        };

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
        let return_expr = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(method.method_attrs.error_in_r)
            .build_s4_inline();

        // Inject r_entry, on.exit, missing param defaults, lifecycle prelude, and precondition checks if present
        let r_entry = &method.method_attrs.r_entry;
        let r_on_exit = &method.method_attrs.r_on_exit;
        let missing = crate::r_wrapper_builder::build_missing_prelude(
            &method.sig.inputs,
            &method.param_defaults,
        );
        let what = format!("{}.{}", method_name, class_name);
        let lifecycle = method.lifecycle_prelude(&what);
        let preconditions = crate::r_preconditions::build_precondition_checks(
            &method.sig.inputs,
            &std::collections::HashSet::new(),
        )
        .static_checks;
        let r_post_checks = &method.method_attrs.r_post_checks;
        if r_entry.is_some()
            || r_on_exit.is_some()
            || !missing.is_empty()
            || lifecycle.is_some()
            || !preconditions.is_empty()
            || r_post_checks.is_some()
        {
            lines.push(format!(
                "methods::setMethod(\"{}\", \"{}\", function({}) {{",
                method_name, class_name, full_params
            ));
            if let Some(entry) = r_entry {
                for line in entry.lines() {
                    lines.push(format!("  {}", line));
                }
            }
            if let Some(on_exit) = r_on_exit {
                lines.push(format!("  {}", on_exit.to_r_code()));
            }
            for line in &missing {
                lines.push(format!("  {}", line));
            }
            if let Some(prelude) = lifecycle {
                lines.push(format!("  {}", prelude));
            }
            for check in &preconditions {
                lines.push(format!("  {}", check));
            }
            if let Some(post) = r_post_checks {
                for line in post.lines() {
                    lines.push(format!("  {}", line));
                }
            }
            lines.push(format!("  {}", return_expr));
            lines.push("})".to_string());
        } else {
            lines.push(format!(
                "methods::setMethod(\"{}\", \"{}\", function({}) {})",
                method_name, class_name, full_params, return_expr
            ));
        }
        lines.push(String::new());
    }

    // Static methods as regular functions
    for ctx in parsed_impl.static_method_contexts() {
        let method_name = ctx.method.r_method_name();
        let fn_name = format!("{}_{}", class_name, method_name);

        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_r_params(&ctx.params)
                    .with_r_name(fn_name.clone());
            lines.extend(method_doc.build());
        }
        // Export static methods so users can call them (if class should be exported)
        if should_export {
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
        // Inject r_post_checks
        if let Some(ref post) = ctx.method.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("  {}", line));
            }
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_expr = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .build_s4_inline();
        lines.push(format!("  {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}
