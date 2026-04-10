//! Env-class R wrapper generator.

use super::ParsedImpl;

/// Generates the complete R wrapper string for an environment-based class.
///
/// Produces an R environment object (`new.env(parent = emptyenv())`) that serves as a
/// class namespace, with methods attached as `ClassName$method_name`. This pattern
/// supports both inherent methods and trait namespace dispatch via `$`/`[[`.
///
/// The generated code includes:
/// - Class environment: `ClassName <- new.env(parent = emptyenv())`
/// - Constructor: `ClassName$new(...)` that calls the Rust `new` function, sets
///   `class(self) <- "ClassName"`, and returns the ExternalPtr as `self`
/// - Instance methods: `ClassName$method(x = self, ...)` using default-arg binding
///   so that `$` dispatch re-parents the environment to make `self` visible
/// - Static methods: `ClassName$method(...)` that call Rust directly
/// - `$.ClassName` S3 method: dispatches `obj$method(...)` by looking up the method
///   in the class environment, binding `self` for instance methods, and supporting
///   trait namespace environments (nested envs with `.__mx_instance__` attributes)
/// - `[[.ClassName` alias: delegates to `$.ClassName`
///
/// Roxygen2 documentation is generated for the class, each method, and the
/// dispatch methods, with appropriate `@export`/`@keywords internal`/`@noRd` tags.
pub fn generate_env_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    // Check if class has @noRd - if so, skip method documentation
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "noRd");

    let mut lines = Vec::new();

    // Class environment documentation and definition
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, &parsed_impl.doc_tags, "")
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
    lines.push(format!("{} <- new.env(parent = emptyenv())", class_name));
    lines.push(String::new());

    // Constructor
    if let Some(ctx) = parsed_impl.constructor_context() {
        lines.push(ctx.source_comment(type_ident));
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, "new", type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }
        lines.push(format!("{}$new <- function({}) {{", class_name, ctx.params));
        for line in ctx.missing_prelude() {
            lines.push(format!("  {}", line));
        }
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        lines.push(format!("  .val <- {}", ctx.static_call()));
        lines.extend(crate::method_return_builder::error_in_r_check_lines("  "));
        lines.push("  self <- .val".to_string());
        lines.push(format!("  class(self) <- \"{}\"", class_name));
        lines.push("  self".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods
    for ctx in parsed_impl.instance_method_contexts() {
        let method_name = ctx.method.r_method_name();
        lines.push(ctx.source_comment(type_ident));
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }

        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method_name, ctx.params
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
        let what = format!("{}${}", class_name, method_name);
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

        let call = ctx.instance_call("self");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods
    for ctx in parsed_impl.static_method_contexts() {
        let method_name = ctx.method.r_method_name();
        lines.push(ctx.source_comment(type_ident));
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }

        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method_name, ctx.params
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
        let what = format!("{}${}", class_name, method_name);
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

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // $ dispatch - export as S3 methods
    // Handles both functions (inherent methods) and environments (trait namespaces)
    let has_internal = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !has_internal && !parsed_impl.noexport;

    // Generate roxygen tags for dispatch methods
    if class_has_no_rd {
        // For internal classes, add @noRd to suppress roxygen2 S3 method detection
        lines.push("#' @noRd".to_string());
    } else {
        lines.push(format!("#' @rdname {}", class_name));
        lines.push("#' @param self The object instance.".to_string());
        lines.push("#' @param name Method name for dispatch.".to_string());
        if should_export {
            lines.push("#' @export".to_string());
        }
    }
    lines.push(format!("`$.{}` <- function(self, name) {{", class_name));
    lines.push(format!("  obj <- {}[[name]]", class_name));
    lines.push("  if (is.environment(obj)) {".to_string());
    lines.push("    # Trait namespace - wrap instance methods to prepend self".to_string());
    lines.push("    bound <- new.env(parent = emptyenv())".to_string());
    lines.push("    for (method_name in names(obj)) {".to_string());
    lines.push("      method <- obj[[method_name]]".to_string());
    lines.push("      if (is.function(method)) {".to_string());
    lines.push("        if (isTRUE(attr(method, \".__mx_instance__\"))) {".to_string());
    lines.push("          local({".to_string());
    lines.push("            m <- method".to_string());
    lines.push("            bound[[method_name]] <<- function(...) m(self, ...)".to_string());
    lines.push("          })".to_string());
    lines.push("        } else {".to_string());
    lines.push("          bound[[method_name]] <- method".to_string());
    lines.push("        }".to_string());
    lines.push("      }".to_string());
    lines.push("    }".to_string());
    lines.push("    bound".to_string());
    lines.push("  } else if (is.null(obj)) {".to_string());
    lines.push("    # Not found at top level -- search trait namespace environments".to_string());
    lines.push(format!("    for (ns_name in names({})) {{", class_name));
    lines.push(format!("      ns <- {}[[ns_name]]", class_name));
    lines.push(
        "      if (is.environment(ns) && exists(name, envir = ns, inherits = FALSE)) {".to_string(),
    );
    lines.push("        method <- ns[[name]]".to_string());
    lines.push(
        "        if (is.function(method) && isTRUE(attr(method, \".__mx_instance__\"))) {"
            .to_string(),
    );
    lines.push("          # Instance method -- bind self as first arg".to_string());
    lines.push("          m <- method".to_string());
    lines.push("          s <- self".to_string());
    lines.push("          return(function(...) m(s, ...))".to_string());
    lines.push("        } else if (is.function(method)) {".to_string());
    lines.push("          return(method)".to_string());
    lines.push("        }".to_string());
    lines.push("      }".to_string());
    lines.push("    }".to_string());
    lines.push("    NULL".to_string());
    lines.push("  } else {".to_string());
    lines.push("    environment(obj) <- environment()".to_string());
    lines.push("    obj".to_string());
    lines.push("  }".to_string());
    lines.push("}".to_string());
    if class_has_no_rd {
        lines.push("#' @noRd".to_string());
    } else {
        lines.push(format!("#' @rdname {}", class_name));
        if should_export {
            lines.push("#' @export".to_string());
        }
    }
    lines.push(format!("`[[.{}` <- `$.{}`", class_name, class_name));

    lines.join("\n")
}
