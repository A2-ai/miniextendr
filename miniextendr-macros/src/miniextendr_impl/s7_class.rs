//! S7-class R wrapper generator.

use super::{ParsedImpl, ParsedMethod};

/// Map a Rust return type to an S7 class name.
///
/// Returns `None` if the type doesn't map to a specific S7 class (uses class_any).
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

/// Generate R wrapper string for S7-style class.
///
/// Creates:
/// - S7::new_class with constructor and .ptr property
/// - S7::new_property for computed/dynamic properties (from #[s7(getter)]/setter)
/// - S7::new_generic + S7::method for each instance method
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
    struct S7Property {
        name: String,
        getter_method_ident: Option<String>,
        setter_method_ident: Option<String>,
        validator_method_ident: Option<String>,
        /// S7 class type inferred from getter return type (e.g., "S7::class_double")
        class_type: Option<String>,
        /// Default value (R expression)
        default_value: Option<String>,
        /// Property is required (error if not provided)
        required: bool,
        /// Property is frozen (can only be set once)
        frozen: bool,
        /// Deprecation message
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

        if attrs.s7_getter || attrs.s7_setter || attrs.s7_validate {
            let method_ident = method.ident.to_string();
            let prop_name = attrs
                .s7_prop
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

            if attrs.s7_getter {
                entry.getter_method_ident = Some(method_ident.clone());
                // Extract S7 class type from getter's return type
                if let syn::ReturnType::Type(_, ret_type) = &method.sig.output {
                    entry.class_type = rust_type_to_s7_class(ret_type);
                }
                // Capture property attributes from getter
                if let Some(ref default) = attrs.s7_default {
                    entry.default_value = Some(default.clone());
                }
                if attrs.s7_required {
                    entry.required = true;
                }
                if attrs.s7_frozen {
                    entry.frozen = true;
                }
                if let Some(ref msg) = attrs.s7_deprecated {
                    entry.deprecated = Some(msg.clone());
                }
            }
            if attrs.s7_setter {
                entry.setter_method_ident = Some(method_ident.clone());
            }
            if attrs.s7_validate {
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
            && (m.method_attrs.s7_convert_from.is_some() || m.method_attrs.s7_convert_to.is_some())
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

    // Document .ptr param - S7::new_class always creates a constructor that accepts
    // all properties as parameters, so .ptr is always a valid parameter
    // Skip if class has @noRd
    if !class_has_no_rd && !crate::roxygen::has_roxygen_tag(class_doc_tags, "param .ptr") {
        lines.push(
            "#' @param .ptr Internal pointer (used by static methods, not for direct use)."
                .to_string(),
        );
    }

    // S7::new_class — optionally include parent and abstract
    if let Some(ref parent) = parsed_impl.s7_parent {
        lines.push(format!(
            "{} <- S7::new_class(\"{}\", parent = {},",
            class_name, class_name, parent
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
            // Validator is called with just the value, not self
            // Generate: validator = function(value) .Call(C_Type__validate_prop, value)
            prop_parts.push(format!(
                "validator = function(value) .Call({}, .call = match.call(), value)",
                ctx.c_ident
            ));
        }

        // Generate getter (with optional deprecation warning)
        if let Some(ref getter_ident) = prop.getter_method_ident
            && let Some(getter_method) = find_method(getter_ident)
        {
            let ctx = MethodContext::new(getter_method, type_ident, parsed_impl.label());
            let getter_call = ctx.instance_call("self@.ptr");
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
            let setter_call = ctx.instance_call("self@.ptr");

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
        if has_self_returning_methods {
            let params_with_ptr = if ctx.params.is_empty() {
                ".ptr = NULL".to_string()
            } else {
                format!("{}, .ptr = NULL", ctx.params)
            };
            lines.push(format!("  constructor = function({}) {{", params_with_ptr));
            // Only check preconditions when not using .ptr shortcut
            if !ctor_preconditions.is_empty() {
                lines.push("    if (is.null(.ptr)) {".to_string());
                for check in &ctor_preconditions {
                    lines.push(format!("      {}", check));
                }
                lines.push("    }".to_string());
            }
            lines.push("    if (!is.null(.ptr)) {".to_string());
            lines.push("      S7::new_object(S7::S7_object(), .ptr = .ptr)".to_string());
            lines.push("    } else {".to_string());
            lines.push(format!(
                "      S7::new_object(S7::S7_object(), .ptr = {})",
                ctx.static_call()
            ));
            lines.push("    }".to_string());
            lines.push("  }".to_string());
        } else {
            lines.push(format!("  constructor = function({}) {{", ctx.params));
            for check in &ctor_preconditions {
                lines.push(format!("    {}", check));
            }
            lines.push(format!(
                "    S7::new_object(S7::S7_object(), .ptr = {})",
                ctx.static_call()
            ));
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

        let generic_name = ctx.generic_name();
        let full_params = ctx.instance_formals(true); // adds x, ..., params
        let method_attrs = &ctx.method.method_attrs;

        // For fallback methods, use tryCatch to avoid slot-access errors on non-S7 objects
        let self_expr = if method_attrs.s7_fallback {
            "tryCatch(x@.ptr, error = function(e) x)"
        } else {
            "x@.ptr"
        };
        let call = ctx.instance_call(self_expr);

        // Determine dispatch class (fallback -> class_any, normal -> class_name)
        let method_class = if method_attrs.s7_fallback {
            "S7::class_any".to_string()
        } else {
            class_name.clone()
        };

        // Documentation - skip if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags);
            lines.extend(method_doc.build());
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

            // Inject lifecycle prelude and precondition checks if present
            let what = format!("{}.{}", generic_name, class_name);
            let lifecycle = ctx.method.lifecycle_prelude(&what);
            let preconditions = ctx.precondition_checks();
            if lifecycle.is_some() || !preconditions.is_empty() {
                lines.push(format!(
                    "S7::method({gen_name}, {method_class}) <- function({full_params}) {{"
                ));
                if let Some(prelude) = lifecycle {
                    lines.push(format!("  {prelude}"));
                }
                for check in &preconditions {
                    lines.push(format!("  {check}"));
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
            // Add @export so roxygen generates export() in NAMESPACE (if class should be exported)
            if should_export {
                lines.push("#' @export".to_string());
            }

            // Determine dispatch arguments (default: "x", or custom via dispatch = "x,y")
            let dispatch_args = if let Some(ref dispatch) = method_attrs.s7_dispatch {
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
            let generic_sig = if method_attrs.s7_no_dots {
                // no_dots: strict generic without ...
                if let Some(ref dispatch) = method_attrs.s7_dispatch {
                    let args: Vec<&str> = dispatch.split(',').map(|s| s.trim()).collect();
                    format!("function({}) S7::S7_dispatch()", args.join(", "))
                } else {
                    "function(x) S7::S7_dispatch()".to_string()
                }
            } else {
                // Default: include ... for extra args
                if let Some(ref dispatch) = method_attrs.s7_dispatch {
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
            let method_formals = ctx.instance_formals_with_dots(true, !method_attrs.s7_no_dots);

            // Inject lifecycle prelude and precondition checks if present
            let what = format!("{}.{}", generic_name, class_name);
            let lifecycle = ctx.method.lifecycle_prelude(&what);
            let preconditions = ctx.precondition_checks();
            if lifecycle.is_some() || !preconditions.is_empty() {
                lines.push(format!(
                    "S7::method({generic_name}, {method_class}) <- function({method_formals}) {{"
                ));
                if let Some(prelude) = lifecycle {
                    lines.push(format!("  {prelude}"));
                }
                for check in &preconditions {
                    lines.push(format!("  {check}"));
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
        let fn_name = format!("{}_{}", class_name, ctx.method.ident);
        let method_name = ctx.method.ident.to_string();

        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_r_name(fn_name.clone());
            lines.extend(method_doc.build());
        }
        // Export static methods so users can call them (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));

        // Inject lifecycle prelude if present
        if let Some(prelude) = ctx.method.lifecycle_prelude(&fn_name) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
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
        if let Some(ref from_type) = attrs.s7_convert_from {
            let ctx = MethodContext::new(method, type_ident, parsed_impl.label());

            // Documentation for convert method (skip if class has @noRd)
            if !class_has_no_rd {
                lines.push(format!("#' @name convert-{}-to-{}", from_type, class_name));
                lines.push(format!("#' @rdname {}", class_name));
                lines.push(format!(
                    "#' @source Generated by miniextendr from `{}::{}`",
                    type_ident, method.ident
                ));
            }

            // Generate: S7::method(S7::convert, list(FromType, ThisClass)) <- function(from, to) ...
            // The convert_from method takes the source object as its sole parameter
            // We pass from@.ptr to extract the ExternalPtr from the S7 object
            let call_with_from = format!(".Call({}, .call = match.call(), from@.ptr)", ctx.c_ident);

            let strategy = crate::ReturnStrategy::for_method(method);
            let return_expr = crate::MethodReturnBuilder::new(call_with_from)
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_error_in_r(method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use imported `convert` - requires `@importFrom S7 convert` in package
            lines.push(format!(
                "S7::method(convert, list({}, {})) <- function(from, to) {}",
                from_type, class_name, return_expr
            ));
            lines.push(String::new());
        }

        // Handle convert_to (instance method pattern)
        // S7 convert signature is function(from, to) - self becomes from
        if let Some(ref to_type) = attrs.s7_convert_to {
            let ctx = MethodContext::new(method, type_ident, parsed_impl.label());

            // Documentation for convert method (skip if class has @noRd)
            if !class_has_no_rd {
                lines.push(format!("#' @name convert-{}-to-{}", class_name, to_type));
                lines.push(format!("#' @rdname {}", class_name));
                lines.push(format!(
                    "#' @source Generated by miniextendr from `{}::{}`",
                    type_ident, method.ident
                ));
            }

            // Generate: S7::method(convert, list(ThisClass, ToType)) <- function(from, to) ...
            // The convert_to method is an instance method where self is mapped to from@.ptr
            let call = format!(".Call({}, .call = match.call(), from@.ptr)", ctx.c_ident);

            // Force ReturnSelf strategy for convert methods since they return S7 class types
            // that need to be wrapped: ToType(.ptr = <result>)
            let return_expr = crate::MethodReturnBuilder::new(call)
                .with_strategy(crate::ReturnStrategy::ReturnSelf)
                .with_class_name(to_type.clone())
                .with_error_in_r(method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use imported `convert` - requires `@importFrom S7 convert` in package
            lines.push(format!(
                "S7::method(convert, list({}, {})) <- function(from, to) {}",
                class_name, to_type, return_expr
            ));
            lines.push(String::new());
        }
    }

    lines.join("\n")
}
