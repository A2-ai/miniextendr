//! S7-class R wrapper generator.

use super::{ParsedImpl, ParsedMethod};
use crate::r_class_formatter::{class_ref_or_verbatim, is_bare_identifier};

/// S7-property variant of [`class_ref_or_verbatim`] that asks the resolver to
/// fall back silently to `S7::class_any` on miss (unregistered type, or a
/// registered-but-non-S7 class). Prevents the load-time `object not found`
/// noise called out in #203.
fn class_ref_or_any_or_verbatim(name: &str) -> String {
    if is_bare_identifier(name) {
        format!(".__MX_CLASS_REF_OR_ANY_{name}__")
    } else {
        name.to_string()
    }
}

/// Map a Rust return type to an S7 class name.
///
/// Returns `None` if the type doesn't map to any S7 class — the caller
/// then omits the `class = …` constraint (so S7 uses `class_any`).
///
/// # S7 Class Mapping
///
/// | Rust Type | S7 Class |
/// |-----------|----------|
/// | `i32`, `i16`, `i8` | `class_integer` |
/// | `f64`, `f32` | `class_double` |
/// | `bool` | `class_logical` |
/// | `u8` | `class_raw` |
/// | `String`, `&str` | `class_character` |
/// | `Vec<i32>` | `class_integer` |
/// | `Vec<f64>` | `class_double` |
/// | `Vec<bool>` | `class_logical` |
/// | `Vec<String>` | `class_character` |
/// | `Option<T>` | `NULL | class_T` (union) |
/// | `SomeUserType` (bare ident) | `.__MX_CLASS_REF_SomeUserType__` |
///
/// For bare user-defined path types the macro emits the same
/// `.__MX_CLASS_REF_<Type>__` placeholder used for parent-class and
/// `convert_from`/`convert_to` references (see [`#154`]). The resolver in
/// [`miniextendr_api::registry::write_r_wrappers_to_file`] swaps the
/// placeholder for the R-visible class name recorded in `MX_CLASS_NAMES`
/// — so `class = "Override"` on an S7 impl block is honored, and child
/// class properties tighten from `class_any` to the real class.
///
/// Unresolved types fall through the existing CLASS_REF mechanism:
/// the bare Rust name is emitted with a compile-time warning. If the
/// user returns a type that isn't a registered class at all, they'll
/// see that warning + a `object '…' not found` at R load time.
pub(super) fn rust_type_to_s7_class(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => {
            let seg = type_path.path.segments.last()?;
            let ident = seg.ident.to_string();

            match ident.as_str() {
                // Scalar types
                "i32" | "i16" | "i8" | "isize" => Some("S7::class_integer".to_string()),
                "f64" | "f32" => Some("S7::class_double".to_string()),
                "bool" => Some("S7::class_logical".to_string()),
                "u8" => Some("S7::class_raw".to_string()),
                "String" => Some("S7::class_character".to_string()),

                // Vec types - check inner type
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        // Recursively get the inner type's class
                        return rust_type_to_s7_class(inner);
                    }
                    None
                }

                // Option types - create union with NULL
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                        && let Some(inner_class) = rust_type_to_s7_class(inner)
                    {
                        return Some(format!("NULL | {}", inner_class));
                    }
                    None
                }

                // Result types - use the Ok type
                "Result" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        return rust_type_to_s7_class(inner);
                    }
                    None
                }

                // Bare, un-generic single-segment identifier — reuse the
                // CLASS_REF write-time placeholder so the resolver swaps
                // in the registered R-visible class name (honoring any
                // `class = "Override"` on the referenced impl block).
                // Paths with `::`, generics, or a lowercase leading char
                // are rejected to avoid matching crate-local aliases,
                // primitives, or type parameters — those fall through
                // to `None` and the caller omits the `class =` entirely.
                _ if type_path.path.segments.len() == 1
                    && matches!(seg.arguments, syn::PathArguments::None)
                    && is_bare_identifier(&ident)
                    && ident.chars().next().is_some_and(|c| c.is_ascii_uppercase()) =>
                {
                    // S7 property class constraint: use the OR_ANY variant so
                    // an unregistered type (or a registered-but-non-S7 class)
                    // falls back silently to `class_any`, restoring pre-#154
                    // behavior for those edge cases (#203).
                    Some(class_ref_or_any_or_verbatim(&ident))
                }

                _ => None,
            }
        }
        syn::Type::Reference(type_ref) => {
            // Handle &str
            if let syn::Type::Path(type_path) = type_ref.elem.as_ref()
                && let Some(seg) = type_path.path.segments.last()
                && seg.ident == "str"
            {
                return Some("S7::class_character".to_string());
            }
            // Recurse for other reference types
            rust_type_to_s7_class(&type_ref.elem)
        }
        _ => None,
    }
}

/// Generates the complete R wrapper string for an S7-style class.
///
/// Produces the following R code:
/// - Class definition: `ClassName <- S7::new_class("ClassName", ...)` with a `.ptr` property
///   of `class_any` holding the `ExternalPtr`, plus optional computed properties
/// - Constructor: inline in `new_class(constructor = function(...) ...)`, supports
///   `.ptr` shortcut parameter for factory methods returning `Self`
/// - Properties: `S7::new_property(...)` for each getter/setter/validator annotated
///   with `#[miniextendr(s7(getter))]` etc., with support for class constraints,
///   defaults, required, frozen, and deprecated modifiers
/// - Instance methods: `S7::new_generic(...)` + `S7::method(generic, class)` pairs
///   dispatching to Rust `.Call()` wrappers via `x@.ptr`
/// - External generics: `S7::new_external_generic("pkg", "name")` for overriding
///   generics from other packages
/// - Multiple dispatch: via `#[miniextendr(s7(dispatch = "x,y"))]`
/// - Fallback methods: `S7::method(generic, S7::class_any)` with `tryCatch` for
///   safe slot access on non-S7 objects
/// - Static methods: regular functions named `ClassName_method(...)`
/// - Convert methods: `S7::method(convert, list(From, To))` for `convert_from`
///   and `convert_to` annotations
/// - S7 parent/abstract: optional `parent` and `abstract = TRUE` in class definition
///
/// Roxygen2 documentation and `@importFrom S7 ...` tags are generated automatically.
pub fn generate_s7_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{
        ClassDocBuilder, MethodContext, MethodDocBuilder, ParsedImplExt,
    };

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    // Check if class has @noRd - if so, skip method documentation and exports
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    let mut lines = Vec::new();

    // Collect S7 property getters, setters, and validators
    // Property name is: s7_prop if specified, else method name
    // We store method idents so we can look them up later
    /// Accumulated metadata for a single S7 property, built up from getter,
    /// setter, and validator method annotations during the first pass over methods.
    struct S7Property {
        /// Property name (from `#[miniextendr(s7(prop = "..."))]` or the method ident).
        name: String,
        /// Ident of the method annotated with `#[miniextendr(s7(getter))]`.
        getter_method_ident: Option<String>,
        /// Ident of the method annotated with `#[miniextendr(s7(setter))]`.
        setter_method_ident: Option<String>,
        /// Ident of the method annotated with `#[miniextendr(s7(validate))]`.
        validator_method_ident: Option<String>,
        /// S7 class type inferred from the getter's return type (e.g., `"S7::class_double"`).
        class_type: Option<String>,
        /// Default value as an R expression string (from `#[miniextendr(s7(default = "..."))]`).
        default_value: Option<String>,
        /// When `true`, the property errors if not provided during construction.
        required: bool,
        /// When `true`, the property can only be set once (subsequent sets error).
        frozen: bool,
        /// If set, a deprecation warning is emitted when the property is accessed or set.
        deprecated: Option<String>,
    }

    let mut properties: std::collections::BTreeMap<String, S7Property> =
        std::collections::BTreeMap::new();
    let mut property_method_idents: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // First pass: collect all property methods (getters, setters, validators)
    for method in &parsed_impl.methods {
        if !method.should_include() {
            continue;
        }
        let attrs = &method.method_attrs;

        if attrs.s7.getter || attrs.s7.setter || attrs.s7.validate {
            let method_ident = method.ident.to_string();
            let prop_name = attrs
                .s7
                .prop
                .clone()
                .unwrap_or_else(|| method_ident.clone());

            property_method_idents.insert(method_ident.clone());

            let entry = properties.entry(prop_name.clone()).or_insert(S7Property {
                name: prop_name,
                getter_method_ident: None,
                setter_method_ident: None,
                validator_method_ident: None,
                class_type: None,
                default_value: None,
                required: false,
                frozen: false,
                deprecated: None,
            });

            if attrs.s7.getter {
                entry.getter_method_ident = Some(method_ident.clone());
                // Extract S7 class type from getter's return type
                if let syn::ReturnType::Type(_, ret_type) = &method.sig.output {
                    entry.class_type = rust_type_to_s7_class(ret_type);
                }
                // Capture property attributes from getter
                if let Some(ref default) = attrs.s7.default {
                    entry.default_value = Some(default.clone());
                }
                if attrs.s7.required {
                    entry.required = true;
                }
                if attrs.s7.frozen {
                    entry.frozen = true;
                }
                if let Some(ref msg) = attrs.s7.deprecated {
                    entry.deprecated = Some(msg.clone());
                }
            }
            if attrs.s7.setter {
                entry.setter_method_ident = Some(method_ident.clone());
            }
            if attrs.s7.validate {
                entry.validator_method_ident = Some(method_ident);
            }
        }
    }

    // Helper to find method by ident
    let find_method = |ident: &str| -> Option<&ParsedMethod> {
        parsed_impl.methods.iter().find(|m| m.ident == ident)
    };

    // Constructor - check if .ptr param will be added (for static methods returning Self)
    let has_self_returning_methods = parsed_impl
        .methods
        .iter()
        .filter(|m| m.should_include())
        .any(|m| m.returns_self());

    // Determine imports based on whether we have properties and what class types are used
    let base_imports = "new_class class_any new_object S7_object new_generic method";
    let mut import_parts: Vec<&str> = vec![base_imports];

    if !properties.is_empty() {
        import_parts.push("new_property");
    }

    // Check if any methods use S7 convert (convert_from or convert_to)
    let has_convert_methods = parsed_impl.methods.iter().any(|m| {
        m.should_include()
            && (m.method_attrs.s7.convert_from.is_some() || m.method_attrs.s7.convert_to.is_some())
    });
    if has_convert_methods {
        import_parts.push("convert");
    }

    // Collect unique S7 class types used in properties
    let mut class_imports: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for prop in properties.values() {
        if let Some(ref class_type) = prop.class_type {
            // Extract class name from "S7::class_xxx" or "NULL | S7::class_xxx"
            for part in class_type.split('|') {
                let part = part.trim();
                if let Some(class_name) = part.strip_prefix("S7::") {
                    class_imports.insert(class_name);
                }
            }
        }
    }
    // Sort for deterministic output
    let mut sorted_imports: Vec<&str> = class_imports.into_iter().collect();
    sorted_imports.sort();
    for class_name in sorted_imports {
        import_parts.push(class_name);
    }

    let imports = format!("@importFrom S7 {}", import_parts.join(" "));

    // Class definition with documentation
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "S7")
            .with_imports(&imports)
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

    // Document constructor params — include constructor @param tags and auto-generate
    // for undocumented ones. Class-level @param tags are already emitted by ClassDocBuilder;
    // constructor-level @param tags must be explicitly pushed here since S7 inlines the
    // constructor inside new_class().
    // Skip if class has @noRd
    if !class_has_no_rd {
        if let Some(ctx) = parsed_impl.constructor_context() {
            let mx_doc = ctx.match_arg_doc_placeholders();
            for param in ctx.params.split(", ").filter(|p| !p.is_empty()) {
                let param_name = param.split('=').next().unwrap_or(param).trim();
                if param_name == ".ptr" || param_name == "..." {
                    continue;
                }
                // Check if already documented at the class level (impl block doc_tags)
                let in_class_docs = class_doc_tags
                    .iter()
                    .any(|t| t.starts_with(&format!("@param {}", param_name)));
                if in_class_docs {
                    continue; // Already emitted by ClassDocBuilder
                }
                // Check if documented in the constructor method's doc_tags
                let ctor_tag = ctx
                    .method
                    .doc_tags
                    .iter()
                    .find(|t| t.starts_with(&format!("@param {}", param_name)));
                if let Some(tag) = ctor_tag {
                    lines.push(format!("#' {}", tag));
                } else if let Some(placeholder) = mx_doc.get(param_name) {
                    // match_arg'd constructor param — placeholder rewritten at
                    // cdylib write time to rendered choice description (#210).
                    lines.push(format!("#' @param {} {}", param_name, placeholder));
                } else {
                    lines.push(format!("#' @param {} (undocumented)", param_name));
                }
            }
        }
        // .ptr is always a constructor param
        if !crate::roxygen::has_roxygen_tag(class_doc_tags, "param .ptr") {
            lines.push(
                "#' @param .ptr Internal pointer (used by static methods, not for direct use)."
                    .to_string(),
            );
        }
    }

    // S7::new_class — optionally include parent and abstract
    if let Some(ref parent) = parsed_impl.s7_parent {
        // Use a placeholder so the resolver can look up the actual R class name
        // at cdylib write time (handles `class = "Override"` on the parent).
        let parent_ref = class_ref_or_verbatim(parent);
        lines.push(format!(
            "{} <- S7::new_class(\"{}\", parent = {},",
            class_name, class_name, parent_ref
        ));
    } else {
        lines.push(format!(
            "{} <- S7::new_class(\"{}\",",
            class_name, class_name
        ));
    }

    if parsed_impl.s7_abstract {
        lines.push("  abstract = TRUE,".to_string());
    }

    // Properties - .ptr holds the ExternalPtr, plus computed/dynamic properties
    // When r_data_accessors is set, merge with sidecar properties from #[derive(ExternalPtr)]
    // Collect property items into a vec, then join with ",\n" to avoid bare commas on standalone lines
    let mut prop_items: Vec<String> = Vec::new();
    prop_items.push("    .ptr = S7::class_any".to_string());

    // Generate computed/dynamic properties
    for prop in properties.values() {
        // Generate property definition
        let mut prop_parts = Vec::new();

        // Add class constraint if known (inferred from getter return type)
        if let Some(ref class_type) = prop.class_type {
            prop_parts.push(format!("class = {}", class_type));
        }

        // Handle default value or required pattern
        if prop.required {
            // Required pattern: error if not provided
            prop_parts.push(format!(
                "default = quote(stop(\"@{} is required\"))",
                prop.name
            ));
        } else if let Some(ref default) = prop.default_value {
            // Explicit default value (R expression)
            prop_parts.push(format!("default = {}", default));
        }

        // Add validator if present
        if let Some(ref validator_ident) = prop.validator_method_ident
            && let Some(validator_method) = find_method(validator_ident)
        {
            let ctx = MethodContext::new(validator_method, type_ident, parsed_impl.label());
            // Validator is called with just the value, not self.
            // Use null_call_attribution: this runs inside S7's dispatch lambda, so
            // match.call() would capture S7 internals, not the user's call site.
            let validator_call = crate::r_wrapper_builder::DotCallBuilder::new(&ctx.c_ident)
                .null_call_attribution()
                .with_args(&["value"])
                .build();
            prop_parts.push(format!("validator = function(value) {validator_call}"));
        }

        // Generate getter (with optional deprecation warning)
        if let Some(ref getter_ident) = prop.getter_method_ident
            && let Some(getter_method) = find_method(getter_ident)
        {
            let ctx = MethodContext::new(getter_method, type_ident, parsed_impl.label());
            // Use null_call_attribution: runs inside S7's property dispatch lambda.
            let getter_call = ctx.instance_call_null_attr("self@.ptr");
            if let Some(ref msg) = prop.deprecated {
                // Deprecated getter: emit warning then return value
                prop_parts.push(format!(
                    "getter = function(self) {{ warning(\"Property @{} is deprecated: {}\"); {} }}",
                    prop.name, msg, getter_call
                ));
            } else {
                prop_parts.push(format!("getter = function(self) {}", getter_call));
            }
        }

        // Generate setter (with optional frozen/deprecation handling)
        if let Some(ref setter_ident) = prop.setter_method_ident
            && let Some(setter_method) = find_method(setter_ident)
        {
            let ctx = MethodContext::new(setter_method, type_ident, parsed_impl.label());
            // Use null_call_attribution: runs inside S7's property dispatch lambda.
            let setter_call = ctx.instance_call_null_attr("self@.ptr");

            if prop.frozen {
                // Frozen pattern: error if property was already set (non-NULL)
                // Note: This is a simplified check; true frozen behavior would need
                // a separate flag in the object to track if ever set
                if let Some(ref msg) = prop.deprecated {
                    prop_parts.push(format!(
                        "setter = function(self, value) {{ warning(\"Property @{} is deprecated: {}\"); if (!is.null(self@{})) stop(\"Property @{} is frozen and cannot be modified\"); {}; self }}",
                        prop.name, msg, prop.name, prop.name, setter_call
                    ));
                } else {
                    prop_parts.push(format!(
                        "setter = function(self, value) {{ if (!is.null(self@{})) stop(\"Property @{} is frozen and cannot be modified\"); {}; self }}",
                        prop.name, prop.name, setter_call
                    ));
                }
            } else if let Some(ref msg) = prop.deprecated {
                // Deprecated setter: emit warning then set value
                prop_parts.push(format!(
                    "setter = function(self, value) {{ warning(\"Property @{} is deprecated: {}\"); {}; self }}",
                    prop.name, msg, setter_call
                ));
            } else {
                // Normal setter
                prop_parts.push(format!(
                    "setter = function(self, value) {{ {}; self }}",
                    setter_call
                ));
            }
        }

        if prop_parts.is_empty() {
            // This shouldn't happen, but handle gracefully
            prop_items.push(format!("    {} = S7::new_property()", prop.name));
        } else {
            prop_items.push(format!(
                "    {} = S7::new_property({})",
                prop.name,
                prop_parts.join(", ")
            ));
        }
    }

    if parsed_impl.r_data_accessors {
        lines.push("  properties = c(list(".to_string());
    } else {
        lines.push("  properties = list(".to_string());
    }
    lines.push(prop_items.join(",\n"));

    // Close the properties list (or merge with sidecar properties)
    if parsed_impl.r_data_accessors {
        let type_name = type_ident.to_string();
        lines.push(format!("  ), .rdata_properties_{}),", type_name));
    } else {
        lines.push("  ),".to_string());
    }

    if let Some(ctx) = parsed_impl.constructor_context() {
        let ctor_preconditions = ctx.precondition_checks();
        let ctor_missing = ctx.missing_prelude();
        let ctor_match_arg = ctx.match_arg_prelude();
        if has_self_returning_methods {
            let params_with_ptr = if ctx.params.is_empty() {
                ".ptr = NULL".to_string()
            } else {
                format!("{}, .ptr = NULL", ctx.params)
            };
            lines.push(format!("  constructor = function({}) {{", params_with_ptr));
            // Missing defaults + preconditions + match.arg only when not using .ptr shortcut
            if !ctor_missing.is_empty()
                || !ctor_preconditions.is_empty()
                || !ctor_match_arg.is_empty()
            {
                lines.push("    if (is.null(.ptr)) {".to_string());
                for line in &ctor_missing {
                    lines.push(format!("      {}", line));
                }
                for check in &ctor_preconditions {
                    lines.push(format!("      {}", check));
                }
                for line in &ctor_match_arg {
                    lines.push(format!("      {}", line));
                }
                lines.push("    }".to_string());
            }
            lines.push("    if (!is.null(.ptr)) {".to_string());
            lines.push("      S7::new_object(S7::S7_object(), .ptr = .ptr)".to_string());
            lines.push("    } else {".to_string());
            lines.push(format!("      .val <- {}", ctx.static_call()));
            lines.extend(crate::method_return_builder::error_in_r_check_lines(
                "      ",
            ));
            lines.push("      S7::new_object(S7::S7_object(), .ptr = .val)".to_string());
            lines.push("    }".to_string());
            lines.push("  }".to_string());
        } else {
            lines.push(format!("  constructor = function({}) {{", ctx.params));
            for line in &ctor_missing {
                lines.push(format!("    {}", line));
            }
            for check in &ctor_preconditions {
                lines.push(format!("    {}", check));
            }
            for line in &ctor_match_arg {
                lines.push(format!("    {}", line));
            }
            lines.push(format!("    .val <- {}", ctx.static_call()));
            lines.extend(crate::method_return_builder::error_in_r_check_lines("    "));
            lines.push("    S7::new_object(S7::S7_object(), .ptr = .val)".to_string());
            lines.push("  }".to_string());
        }
    }

    lines.push(")".to_string());
    lines.push(String::new());

    // Instance methods as S7 generics + methods
    // Skip methods that are property getters/setters (they're handled as S7 properties)
    for ctx in parsed_impl.instance_method_contexts() {
        let method_ident = ctx.method.ident.to_string();
        if property_method_idents.contains(&method_ident) {
            continue;
        }
        lines.push(ctx.source_comment(type_ident));

        let generic_name = ctx.generic_name();
        let full_params = ctx.instance_formals(true); // adds x, ..., params
        let method_attrs = &ctx.method.method_attrs;

        // For fallback methods (class_any), check class before using @ to extract
        // the pointer. Non-S7 objects can't have @.ptr — error in R rather than
        // passing a wrong type to Rust (which would segfault).
        let self_expr = if method_attrs.s7.fallback {
            "if (inherits(x, \"S7_object\")) x@.ptr else stop(paste0(\"expected an S7 object, got \", class(x)[[1]]))"
        } else {
            "x@.ptr"
        };
        let call = ctx.instance_call(self_expr);

        // Determine dispatch class (fallback -> class_any, normal -> class_name)
        let method_class = if method_attrs.s7.fallback {
            "S7::class_any".to_string()
        } else {
            class_name.clone()
        };

        // Documentation - skip if class has @noRd.
        // Use class-qualified @name to avoid duplicate \alias{generic} warnings
        // when multiple S7 classes share the same generic (e.g., get_value on both
        // S7TraitCounter and CounterTraitS7). The @export is replaced with
        // @rawNamespace to explicitly export the bare generic name.
        if !class_has_no_rd {
            let qualified_name = format!("{}-{}", class_name, generic_name);
            let method_doc =
                MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags)
                    .with_suppress_params()
                    .with_r_name(qualified_name);
            let mut doc_lines = method_doc.build();
            doc_lines.push(format!("#' @aliases {}${}", class_name, generic_name));
            lines.extend(doc_lines);
        }

        if ctx.has_generic_override() {
            // Parse "pkg::name" format for external generics
            let (pkg, gen_name) = if generic_name.contains("::") {
                let parts: Vec<&str> = generic_name.split("::").collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("base".to_string(), generic_name.clone())
            };

            // Use S7::new_external_generic for existing generics from other packages
            lines.push(format!(
                "if (!exists(\"{gen_name}\", mode = \"function\")) {{"
            ));
            lines.push(format!(
                "  {gen_name} <- S7::new_external_generic(\"{pkg}\", \"{gen_name}\")"
            ));
            lines.push("}".to_string());

            // Define method using the resolved generic name
            let strategy = crate::ReturnStrategy::for_method(ctx.method);
            let return_expr = crate::MethodReturnBuilder::new(call.clone())
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_error_in_r(ctx.method.method_attrs.error_in_r)
                .build_s7_inline();

            // Inject r_entry, on.exit, missing param defaults, lifecycle prelude, precondition checks, match_arg, r_post_checks
            let what = format!("{}.{}", generic_name, class_name);
            let r_entry = &ctx.method.method_attrs.r_entry;
            let r_on_exit = &ctx.method.method_attrs.r_on_exit;
            let missing = ctx.missing_prelude();
            let lifecycle = ctx.method.lifecycle_prelude(&what);
            let preconditions = ctx.precondition_checks();
            let match_arg_lines = ctx.match_arg_prelude();
            let r_post_checks = &ctx.method.method_attrs.r_post_checks;
            if r_entry.is_some()
                || r_on_exit.is_some()
                || !missing.is_empty()
                || lifecycle.is_some()
                || !preconditions.is_empty()
                || !match_arg_lines.is_empty()
                || r_post_checks.is_some()
            {
                lines.push(format!(
                    "S7::method({gen_name}, {method_class}) <- function({full_params}) {{"
                ));
                if let Some(entry) = r_entry {
                    for line in entry.lines() {
                        lines.push(format!("  {line}"));
                    }
                }
                if let Some(on_exit) = r_on_exit {
                    lines.push(format!("  {}", on_exit.to_r_code()));
                }
                for line in &missing {
                    lines.push(format!("  {line}"));
                }
                if let Some(prelude) = lifecycle {
                    lines.push(format!("  {prelude}"));
                }
                for check in &preconditions {
                    lines.push(format!("  {check}"));
                }
                for line in &match_arg_lines {
                    lines.push(format!("  {line}"));
                }
                if let Some(post) = r_post_checks {
                    for line in post.lines() {
                        lines.push(format!("  {line}"));
                    }
                }
                lines.push(format!("  {return_expr}"));
                lines.push("}".to_string());
            } else {
                lines.push(format!(
                    "S7::method({gen_name}, {method_class}) <- function({full_params}) {return_expr}"
                ));
            }
        } else {
            // Create new S7 generic if it doesn't exist
            // Use @rawNamespace to explicitly export the bare generic name.
            // Plain @export would export the qualified @name (e.g., "ClassName-method")
            // instead of the bare generic.
            if should_export {
                lines.push(format!("#' @rawNamespace export({})", generic_name));
            }

            // Determine dispatch arguments (default: "x", or custom via dispatch = "x,y")
            let dispatch_args = if let Some(ref dispatch) = method_attrs.s7.dispatch {
                // Multiple dispatch: "x,y" -> c("x", "y")
                let args: Vec<&str> = dispatch.split(',').map(|s| s.trim()).collect();
                if args.len() == 1 {
                    format!("\"{}\"", args[0])
                } else {
                    format!(
                        "c({})",
                        args.iter()
                            .map(|a| format!("\"{}\"", a))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            } else {
                "\"x\"".to_string()
            };

            // Determine function signature (with or without ...)
            let generic_sig = if method_attrs.s7.no_dots {
                // no_dots: strict generic without ...
                if let Some(ref dispatch) = method_attrs.s7.dispatch {
                    let args: Vec<&str> = dispatch.split(',').map(|s| s.trim()).collect();
                    format!("function({}) S7::S7_dispatch()", args.join(", "))
                } else {
                    "function(x) S7::S7_dispatch()".to_string()
                }
            } else {
                // Default: include ... for extra args
                if let Some(ref dispatch) = method_attrs.s7.dispatch {
                    let args: Vec<&str> = dispatch.split(',').map(|s| s.trim()).collect();
                    format!("function({}, ...) S7::S7_dispatch()", args.join(", "))
                } else {
                    "function(x, ...) S7::S7_dispatch()".to_string()
                }
            };

            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {{"
            ));
            lines.push(format!(
                "  {generic_name} <- S7::new_generic(\"{generic_name}\", {dispatch_args}, {generic_sig})"
            ));
            lines.push("}".to_string());

            // Define method
            let strategy = crate::ReturnStrategy::for_method(ctx.method);
            let return_expr = crate::MethodReturnBuilder::new(call)
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_error_in_r(ctx.method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use matching formals for method (with or without ...)
            let method_formals = ctx.instance_formals_with_dots(true, !method_attrs.s7.no_dots);

            // Inject r_entry, on.exit, missing param defaults, lifecycle prelude, precondition checks, match_arg, r_post_checks
            let what = format!("{}.{}", generic_name, class_name);
            let r_entry = &ctx.method.method_attrs.r_entry;
            let r_on_exit = &ctx.method.method_attrs.r_on_exit;
            let missing = ctx.missing_prelude();
            let lifecycle = ctx.method.lifecycle_prelude(&what);
            let preconditions = ctx.precondition_checks();
            let match_arg_lines = ctx.match_arg_prelude();
            let r_post_checks = &ctx.method.method_attrs.r_post_checks;
            if r_entry.is_some()
                || r_on_exit.is_some()
                || !missing.is_empty()
                || lifecycle.is_some()
                || !preconditions.is_empty()
                || !match_arg_lines.is_empty()
                || r_post_checks.is_some()
            {
                lines.push(format!(
                    "S7::method({generic_name}, {method_class}) <- function({method_formals}) {{"
                ));
                if let Some(entry) = r_entry {
                    for line in entry.lines() {
                        lines.push(format!("  {line}"));
                    }
                }
                if let Some(on_exit) = r_on_exit {
                    lines.push(format!("  {}", on_exit.to_r_code()));
                }
                for line in &missing {
                    lines.push(format!("  {line}"));
                }
                if let Some(prelude) = lifecycle {
                    lines.push(format!("  {prelude}"));
                }
                for check in &preconditions {
                    lines.push(format!("  {check}"));
                }
                for line in &match_arg_lines {
                    lines.push(format!("  {line}"));
                }
                if let Some(post) = r_post_checks {
                    for line in post.lines() {
                        lines.push(format!("  {line}"));
                    }
                }
                lines.push(format!("  {return_expr}"));
                lines.push("}".to_string());
            } else {
                lines.push(format!(
                    "S7::method({generic_name}, {method_class}) <- function({method_formals}) {return_expr}"
                ));
            }
        }
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
        let return_expr = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .build_s7_inline();
        lines.push(format!("  {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Phase 4: S7 convert() methods from Rust From/TryFrom patterns
    // Convert methods enable type coercion between S7 classes using S7::convert()
    //
    // Two patterns:
    // 1. convert_from = "OtherType" on static method: converts FROM OtherType TO this class
    //    Rust: fn from_other(other: OtherType) -> Self
    //    R: S7::method(S7::convert, list(OtherType, ThisClass)) <- function(from, to) ...
    //
    // 2. convert_to = "OtherType" on instance method: converts FROM this class TO OtherType
    //    Rust: fn to_other(&self) -> OtherType
    //    R: S7::method(S7::convert, list(ThisClass, OtherType)) <- function(from, to) ...

    for method in &parsed_impl.methods {
        if !method.should_include() {
            continue;
        }
        let attrs = &method.method_attrs;

        // Handle convert_from (static method pattern)
        // S7 convert signature is function(from, to) - one parameter for the source object
        if let Some(ref from_type) = attrs.s7.convert_from {
            let ctx = MethodContext::new(method, type_ident, parsed_impl.label());

            // Documentation for convert method (skip if class has @noRd)
            if !class_has_no_rd {
                lines.push(format!("#' @name convert-{}-to-{}", from_type, class_name));
                lines.push(format!("#' @rdname {}", class_name));
                lines.push(format!(
                    "#' @source Generated by miniextendr from `{}::{}`",
                    type_ident, method.ident
                ));
                // Add @aliases convert so roxygen2 emits \alias{convert} in the
                // merged .Rd file. Without this, R CMD check warns:
                //   "Objects in \usage without \alias in Rd file '...Rd': 'convert'"
                lines.push("#' @aliases convert".to_string());
                // S7's `convert` generic is `function(from, to, ...)`. Document
                // `...` so the rendered \usage{} matches and codoc passes.
                lines.push(
                    "#' @param ... Additional arguments passed to the S7 convert generic."
                        .to_string(),
                );
            }

            // Generate: S7::method(S7::convert, list(FromType, ThisClass)) <- function(from, to, ...) ...
            // The convert_from method takes the source object as its sole parameter
            // We pass from@.ptr to extract the ExternalPtr from the S7 object
            let call_with_from = crate::r_wrapper_builder::DotCallBuilder::new(&ctx.c_ident)
                .with_self("from@.ptr")
                .build();

            let strategy = crate::ReturnStrategy::for_method(method);
            let return_expr = crate::MethodReturnBuilder::new(call_with_from)
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_error_in_r(method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use imported `convert` - requires `@importFrom S7 convert` in package.
            // from_type is a cross-reference → placeholder so the resolver can look it up.
            // Method signature includes `...` to match the S7 `convert` generic
            // (function(from, to, ...)) and silence R CMD check codoc warnings.
            let from_type_ref = class_ref_or_verbatim(from_type);
            lines.push(format!(
                "S7::method(convert, list({}, {})) <- function(from, to, ...) {}",
                from_type_ref, class_name, return_expr
            ));
            lines.push(String::new());
        }

        // Handle convert_to (instance method pattern)
        // S7 convert signature is function(from, to) - self becomes from
        if let Some(ref to_type) = attrs.s7.convert_to {
            let ctx = MethodContext::new(method, type_ident, parsed_impl.label());

            // Documentation for convert method (skip if class has @noRd)
            if !class_has_no_rd {
                lines.push(format!("#' @name convert-{}-to-{}", class_name, to_type));
                lines.push(format!("#' @rdname {}", class_name));
                lines.push(format!(
                    "#' @source Generated by miniextendr from `{}::{}`",
                    type_ident, method.ident
                ));
                // Add @aliases convert so roxygen2 emits \alias{convert} in the
                // merged .Rd file. Without this, R CMD check warns:
                //   "Objects in \usage without \alias in Rd file '...Rd': 'convert'"
                lines.push("#' @aliases convert".to_string());
                // S7's `convert` generic is `function(from, to, ...)`. Document
                // `...` so the rendered \usage{} matches and codoc passes.
                lines.push(
                    "#' @param ... Additional arguments passed to the S7 convert generic."
                        .to_string(),
                );
            }

            // Generate: S7::method(convert, list(ThisClass, ToType)) <- function(from, to, ...) ...
            // The convert_to method is an instance method where self is mapped to from@.ptr
            let call = crate::r_wrapper_builder::DotCallBuilder::new(&ctx.c_ident)
                .with_self("from@.ptr")
                .build();

            // to_type is a cross-reference → placeholder for resolver.
            // We also pass the placeholder to MethodReturnBuilder so the
            // emitted `ToType(.ptr = <result>)` uses the resolved name.
            let to_type_ref = class_ref_or_verbatim(to_type);

            // Force ReturnSelf strategy for convert methods since they return S7 class types
            // that need to be wrapped: ToType(.ptr = <result>)
            let return_expr = crate::MethodReturnBuilder::new(call)
                .with_strategy(crate::ReturnStrategy::ReturnSelf)
                .with_class_name(to_type_ref.clone())
                .with_error_in_r(method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use imported `convert` - requires `@importFrom S7 convert` in package.
            // Method signature includes `...` to match the S7 `convert` generic
            // (function(from, to, ...)) and silence R CMD check codoc warnings.
            lines.push(format!(
                "S7::method(convert, list({}, {})) <- function(from, to, ...) {}",
                class_name, to_type_ref, return_expr
            ));
            lines.push(String::new());
        }
    }

    lines.join("\n")
}
