//! R6-class R wrapper generator.

use super::ParsedImpl;

/// Return either a `.__MX_CLASS_REF_<name>__` placeholder (for bare identifiers)
/// or `name` verbatim (for namespaced / non-identifier strings).
///
/// Mirrors the function in `s7_class.rs`. Inline here to avoid a cross-module dep.
fn class_ref_or_verbatim(name: &str) -> String {
    let is_bare = {
        let mut chars = name.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            }
            _ => false,
        }
    };
    if is_bare {
        format!(".__MX_CLASS_REF_{name}__")
    } else {
        name.to_string()
    }
}

/// Generates the complete R wrapper string for an R6-style class.
///
/// Produces an `R6::R6Class(...)` definition that includes:
/// - `initialize` method: calls the Rust `new` constructor, or accepts a pre-made `.ptr`
///   when static methods return `Self` (factory pattern)
/// - Public methods: one R function per `&self`/`&mut self` instance method
/// - Private methods: methods marked with `#[miniextendr(private)]`
/// - Active bindings: getter/setter properties via `#[miniextendr(r6(prop = "..."))]`
/// - Private `.ptr` field: holds the `ExternalPtr` to the Rust struct
/// - Finalizer: optional destructor called when the R6 object is garbage-collected
/// - Deep clone: optional custom clone logic via `#[miniextendr(r6(deep_clone))]`
/// - Static methods: emitted as `ClassName$method_name <- function(...)` outside the class
/// - Class options: `lock_objects`, `lock_class`, `cloneable`, `portable`, `inherit`
///
/// Also generates roxygen2 documentation blocks for the class, its methods,
/// and active bindings.
pub fn generate_r6_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;

    // Check if .ptr parameter will be added to initialize (for static methods returning Self)
    let has_self_returning_methods = parsed_impl
        .methods
        .iter()
        .filter(|m| m.should_include())
        .any(|m| m.returns_self());

    let mut lines = Vec::new();

    // Start R6Class definition with documentation
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "R6")
            .with_imports("@importFrom R6 R6Class")
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
        // Insert before @export (which is last)
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(insert_pos, format!("#' {}", lc_import));
    }

    // Document .ptr param if initialize will have it (for static methods returning Self)
    if has_self_returning_methods && !crate::roxygen::has_roxygen_tag(class_doc_tags, "param .ptr")
    {
        // Insert before @export (which is last)
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(
            insert_pos,
            "#' @param .ptr Internal pointer (used by static methods, not for direct use)."
                .to_string(),
        );
    }
    // R6Class definition — optionally include inherit.
    // Use a placeholder so the resolver can look up the actual R class name
    // at cdylib write time (handles `class = "Override"` on the parent).
    if let Some(ref parent) = parsed_impl.r6_inherit {
        let parent_ref = class_ref_or_verbatim(parent);
        lines.push(format!(
            "{} <- R6::R6Class(\"{}\", inherit = {},",
            class_name, class_name, parent_ref
        ));
    } else {
        lines.push(format!("{} <- R6::R6Class(\"{}\",", class_name, class_name));
    }

    // Portable flag (only emit if explicitly set to FALSE, since TRUE is default)
    if parsed_impl.r6_portable == Some(false) {
        lines.push("  portable = FALSE,".to_string());
    }

    // Public list
    lines.push("  public = list(".to_string());

    // Public instance methods (collect first to know if we need trailing comma on initialize)
    let public_method_contexts: Vec<_> = parsed_impl.public_instance_method_contexts().collect();
    let has_public_methods = !public_method_contexts.is_empty();

    // Constructor (initialize) - accepts either normal params or a pre-made .ptr.
    // If there's no explicit `new()` but there are factory methods returning Self,
    // generate a minimal initialize(.ptr) so factories can call $new(.ptr = val).
    if let Some(ctx) = parsed_impl.constructor_context() {
        lines.push(format!("    {}", ctx.source_comment(type_ident)));
        // Add inline roxygen documentation for initialize method
        // Note: @title is replaced with @description for R6 inline docs (roxygen requirement)
        let has_description = ctx
            .method
            .doc_tags
            .iter()
            .any(|t| t.starts_with("@description ") || t.starts_with("@title "));
        if !has_description {
            lines.push(format!(
                "    #' @description Create a new `{}`.",
                class_name
            ));
        }
        for tag in &ctx.method.doc_tags {
            for line in tag.lines() {
                let line = if line.starts_with("@title ") {
                    line.replacen("@title ", "@description ", 1)
                } else {
                    line.to_string()
                };
                lines.push(format!("    #' {}", line));
            }
        }
        // Document constructor params that aren't already documented
        let ctor_mx_doc = ctx.match_arg_doc_placeholders();
        for param in ctx.params.split(", ").filter(|p| !p.is_empty()) {
            let param_name = param.split('=').next().unwrap_or(param).trim();
            if param_name == ".ptr" {
                continue;
            }
            let already_documented = ctx
                .method
                .doc_tags
                .iter()
                .any(|t| t.starts_with(&format!("@param {}", param_name)));
            if !already_documented {
                // match_arg'd constructor params get the write-time placeholder
                // so the cdylib pass renders `One of "A", "B".` (#210).
                let body = ctor_mx_doc
                    .get(param_name)
                    .map(String::as_str)
                    .unwrap_or("(no documentation available)");
                lines.push(format!("    #' @param {} {}", param_name, body));
            }
        }

        // Only add trailing comma if there are public methods after initialize
        let comma = if has_public_methods { "," } else { "" };

        // Precondition checks for constructor parameters
        let ctor_preconditions = ctx.precondition_checks();

        // Missing param prelude for constructor
        let ctor_missing = ctx.missing_prelude();

        let ctor_match_arg = ctx.match_arg_prelude();

        if has_self_returning_methods {
            let full_params = if ctx.params.is_empty() {
                ".ptr = NULL".to_string()
            } else {
                format!("{}, .ptr = NULL", ctx.params)
            };
            lines.push(format!("    initialize = function({}) {{", full_params));
            // Missing defaults + preconditions + match.arg only when not using .ptr shortcut
            if !ctor_missing.is_empty()
                || !ctor_preconditions.is_empty()
                || !ctor_match_arg.is_empty()
            {
                lines.push("      if (is.null(.ptr)) {".to_string());
                for line in &ctor_missing {
                    lines.push(format!("        {}", line));
                }
                for check in &ctor_preconditions {
                    lines.push(format!("        {}", check));
                }
                for line in &ctor_match_arg {
                    lines.push(format!("        {}", line));
                }
                lines.push("      }".to_string());
            }
            lines.push("      if (!is.null(.ptr)) {".to_string());
            lines.push("        private$.ptr <- .ptr".to_string());
            lines.push("      } else {".to_string());
            lines.push(format!("        .val <- {}", ctx.static_call()));
            lines.push(
                "        if (inherits(.val, \"rust_error_value\") && isTRUE(attr(.val, \"__rust_error__\"))) {"
                    .to_string(),
            );
            lines.push("          stop(structure(".to_string());
            lines.push(
                "            class = c(\"rust_error\", \"simpleError\", \"error\", \"condition\"),"
                    .to_string(),
            );
            lines.push(
                "            list(message = .val$error, call = .val$call %||% sys.call(), kind = .val$kind)"
                    .to_string(),
            );
            lines.push("          ))".to_string());
            lines.push("        }".to_string());
            lines.push("        private$.ptr <- .val".to_string());
            lines.push("      }".to_string());
            lines.push(format!("    }}{}", comma));
        } else {
            lines.push(format!("    initialize = function({}) {{", ctx.params));
            for line in &ctor_missing {
                lines.push(format!("      {}", line));
            }
            for check in &ctor_preconditions {
                lines.push(format!("      {}", check));
            }
            for line in &ctor_match_arg {
                lines.push(format!("      {}", line));
            }
            lines.push(format!("      .val <- {}", ctx.static_call()));
            lines.extend(crate::method_return_builder::error_in_r_check_lines(
                "      ",
            ));
            lines.push("      private$.ptr <- .val".to_string());
            lines.push(format!("    }}{}", comma));
        }
    } else if has_self_returning_methods {
        // No explicit new() constructor, but factory methods need $new(.ptr = val).
        // Generate a minimal initialize that only accepts .ptr.
        let comma = if has_public_methods { "," } else { "" };
        lines.push(format!(
            "    #' @description Create a new `{}`.",
            class_name
        ));
        lines.push("    initialize = function(.ptr = NULL) {".to_string());
        lines.push("      if (!is.null(.ptr)) {".to_string());
        lines.push("        private$.ptr <- .ptr".to_string());
        lines.push("      }".to_string());
        lines.push(format!("    }}{}", comma));
    }

    // Public instance methods
    for (i, ctx) in public_method_contexts.iter().enumerate() {
        let comma = if i < public_method_contexts.len() - 1 {
            ","
        } else {
            ""
        };

        lines.push(format!("    {}", ctx.source_comment(type_ident)));
        // Add inline roxygen documentation for this method
        // Note: @title is replaced with @description for R6 inline docs (roxygen requirement)
        let r_name = ctx.method.r_method_name();
        let has_description = ctx
            .method
            .doc_tags
            .iter()
            .any(|t| t.starts_with("@description ") || t.starts_with("@title "));
        if !has_description {
            lines.push(format!("    #' @description Method `{}`.", r_name));
        }
        for tag in &ctx.method.doc_tags {
            for line in tag.lines() {
                let line = if line.starts_with("@title ") {
                    line.replacen("@title ", "@description ", 1)
                } else {
                    line.to_string()
                };
                lines.push(format!("    #' {}", line));
            }
        }
        // Document method params that aren't already documented
        let method_mx_doc = ctx.match_arg_doc_placeholders();
        for param in ctx.params.split(", ").filter(|p| !p.is_empty()) {
            let param_name = param.split('=').next().unwrap_or(param).trim();
            let already_documented = ctx
                .method
                .doc_tags
                .iter()
                .any(|t| t.starts_with(&format!("@param {}", param_name)));
            if !already_documented {
                let body = method_mx_doc
                    .get(param_name)
                    .map(String::as_str)
                    .unwrap_or("(no documentation available)");
                lines.push(format!("    #' @param {} {}", param_name, body));
            }
        }
        lines.push(format!("    {} = function({}) {{", r_name, ctx.params));

        // Inject r_entry (user code before all checks)
        if let Some(ref entry) = ctx.method.method_attrs.r_entry {
            for line in entry.lines() {
                lines.push(format!("      {}", line));
            }
        }
        // Inject on.exit cleanup
        if let Some(ref on_exit) = ctx.method.method_attrs.r_on_exit {
            lines.push(format!("      {}", on_exit.to_r_code()));
        }
        // Inject missing param defaults
        for line in ctx.missing_prelude() {
            lines.push(format!("      {}", line));
        }
        // Inject lifecycle prelude if present
        let what = format!("{}${}", class_name, r_name);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("      {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("      {}", check));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("      {}", line));
        }
        // Inject r_post_checks (user code after all checks, before .Call)
        if let Some(ref post) = ctx.method.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("      {}", line));
            }
        }

        let call = ctx.instance_call("private$.ptr");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .with_indent(6); // R6 methods have 6-space indent
        lines.extend(return_builder.build_r6_body());

        lines.push(format!("    }}{}", comma));
    }

    lines.push("  ),".to_string());

    // Private list - includes .ptr and any private methods
    lines.push("  private = list(".to_string());

    // Private instance methods
    for ctx in parsed_impl.private_instance_method_contexts() {
        lines.push(format!("    {}", ctx.source_comment(type_ident)));
        lines.push(format!(
            "    {} = function({}) {{",
            ctx.method.r_method_name(),
            ctx.params
        ));

        // Inject r_entry
        if let Some(ref entry) = ctx.method.method_attrs.r_entry {
            for line in entry.lines() {
                lines.push(format!("      {}", line));
            }
        }
        // Inject on.exit cleanup
        if let Some(ref on_exit) = ctx.method.method_attrs.r_on_exit {
            lines.push(format!("      {}", on_exit.to_r_code()));
        }
        // Inject missing param defaults
        for line in ctx.missing_prelude() {
            lines.push(format!("      {}", line));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("      {}", line));
        }
        // Inject r_post_checks
        if let Some(ref post) = ctx.method.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("      {}", line));
            }
        }

        let call = ctx.instance_call("private$.ptr");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .with_indent(6);
        lines.extend(return_builder.build_r6_body());

        lines.push("    },".to_string());
    }

    // Finalizer (if any)
    if let Some(finalizer) = parsed_impl.finalizer() {
        let c_ident = finalizer.c_wrapper_ident(type_ident, parsed_impl.label());
        lines.push(format!(
            "    finalize = function() .Call({}, .call = match.call(), private$.ptr),",
            c_ident
        ));
    }

    // deep_clone (if any method marked with #[miniextendr(r6(deep_clone))])
    if let Some(dc_method) = parsed_impl
        .methods
        .iter()
        .find(|m| m.method_attrs.deep_clone && m.should_include())
    {
        let c_ident = dc_method.c_wrapper_ident(type_ident, parsed_impl.label());
        lines.push(format!(
            "    deep_clone = function(name, value) .Call({}, .call = match.call(), private$.ptr, name, value),",
            c_ident
        ));
    }

    // .ptr field (always last, no trailing comma)
    lines.push("    .ptr = NULL".to_string());
    lines.push("  ),".to_string());

    // Active bindings list (for property-like access)
    let active_method_contexts: Vec<_> = parsed_impl.active_instance_method_contexts().collect();
    if !active_method_contexts.is_empty() {
        lines.push("  active = list(".to_string());

        for (i, ctx) in active_method_contexts.iter().enumerate() {
            let comma = if i < active_method_contexts.len() - 1 {
                ","
            } else {
                ""
            };

            // Add inline @field documentation for active bindings
            // roxygen2 requires @field tags (not @description) for active bindings
            let method_name = ctx.method.r_method_name();
            if ctx.method.doc_tags.is_empty() {
                lines.push(format!("    #' @field {} Active binding.", method_name));
            }
            for tag in &ctx.method.doc_tags {
                for (line_idx, line) in tag.lines().enumerate() {
                    // Convert @description/@title to @field on first line only
                    let line = if line_idx == 0 {
                        if let Some(desc) = line.strip_prefix("@description ") {
                            format!("@field {} {}", method_name, desc)
                        } else if let Some(desc) = line.strip_prefix("@title ") {
                            format!("@field {} {}", method_name, desc)
                        } else if !line.starts_with('@') {
                            // Plain doc comment - treat as field description
                            format!("@field {} {}", method_name, line)
                        } else {
                            line.to_string()
                        }
                    } else {
                        // Continuation lines stay as-is
                        line.to_string()
                    };
                    lines.push(format!("    #' {}", line));
                }
            }

            // Determine the property name (from r6_prop or method name)
            let prop_name = ctx
                .method
                .method_attrs
                .r6_prop
                .clone()
                .unwrap_or_else(|| ctx.method.r_method_name());

            // Check if there's a matching setter for this property
            let setter = parsed_impl.find_setter_for_prop(&prop_name);

            if let Some(setter_method) = setter {
                // Combined getter/setter active binding
                // Format: name = function(value) { if (missing(value)) getter else setter }
                lines.push(format!("    {} = function(value) {{", prop_name));
                lines.push("      if (missing(value)) {".to_string());

                // Getter call
                let getter_call = ctx.instance_call("private$.ptr");
                lines.push(format!("        {}", getter_call));

                lines.push("      } else {".to_string());

                // Setter call - construct directly
                let setter_c_ident =
                    setter_method.c_wrapper_ident(type_ident, parsed_impl.label.as_deref());
                let setter_call = format!(
                    ".Call({}, .call = match.call(), private$.ptr, value)",
                    setter_c_ident
                );
                lines.push(format!("        {}", setter_call));
                lines.push("        invisible(self)".to_string());

                lines.push("      }".to_string());
                lines.push(format!("    }}{}", comma));
            } else {
                // Getter-only active binding (no parameters besides self)
                // Format: name = function() { ... }
                lines.push(format!("    {} = function() {{", prop_name));

                let call = ctx.instance_call("private$.ptr");
                let strategy = crate::ReturnStrategy::for_method(ctx.method);
                let return_builder = crate::MethodReturnBuilder::new(call)
                    .with_strategy(strategy)
                    .with_class_name(class_name.clone())
                    .with_error_in_r(ctx.method.method_attrs.error_in_r)
                    .with_indent(6); // R6 active bindings have 6-space indent
                lines.extend(return_builder.build_r6_body());

                lines.push(format!("    }}{}", comma));
            }
        }

        lines.push("  ),".to_string());
    }

    // Class options
    let lock_objects = parsed_impl.r6_lock_objects.unwrap_or(true);
    let lock_class = parsed_impl.r6_lock_class.unwrap_or(false);
    let cloneable = parsed_impl.r6_cloneable.unwrap_or(false);
    lines.push(format!(
        "  lock_objects = {},",
        if lock_objects { "TRUE" } else { "FALSE" }
    ));
    lines.push(format!(
        "  lock_class = {},",
        if lock_class { "TRUE" } else { "FALSE" }
    ));
    lines.push(format!(
        "  cloneable = {}",
        if cloneable { "TRUE" } else { "FALSE" }
    ));
    lines.push(")".to_string());

    // If r_data_accessors is set, apply sidecar active bindings from #[derive(ExternalPtr)]
    if parsed_impl.r_data_accessors {
        let type_name = type_ident.to_string();
        lines.push(format!(
            ".rdata_active_bindings_{}({})",
            type_name, class_name
        ));
    }

    // Check if class has @noRd
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");

    // Static methods as separate functions on the class object
    for ctx in parsed_impl.static_method_contexts() {
        let method_name = ctx.method.r_method_name();
        let static_method_name = format!("{}${}", class_name, method_name);
        lines.push(String::new());

        lines.push(ctx.source_comment(type_ident));
        let method_doc =
            MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                .with_name_prefix("$")
                .with_class_no_rd(class_has_no_rd);
        lines.extend(method_doc.build());

        lines.push(format!(
            "{} <- function({}) {{",
            static_method_name, ctx.params
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
        lines.extend(return_builder.build_r6_body());

        lines.push("}".to_string());
    }

    lines.join("\n")
}
