//! R wrapper generation for trait methods across all class systems.
//!
//! Each class system (Env, S3, S4, S7, R6, Vctrs) has its own generator that
//! produces R code strings for instance methods, static methods, and associated
//! constants. The top-level [`generate_trait_r_wrapper`] dispatches to the
//! appropriate generator and applies post-processing for export/documentation control.

use super::method_context::{TraitMethodContext, trait_namespace_env_var, trait_namespace_target};
use super::{TraitConst, TraitMethod};
use crate::miniextendr_impl::ClassSystem;
use crate::r_class_formatter::emit_s3_generic_guard;

/// Options controlling export visibility and documentation for trait R wrapper generation.
pub(super) struct TraitWrapperOpts {
    /// Which R class system to generate wrappers for (env, r6, s3, s4, s7, vctrs).
    pub(super) class_system: ClassSystem,
    /// Whether the impl block has `@noRd`, suppressing roxygen documentation output.
    /// For S3/vctrs, method registration tags are preserved even when this is true.
    pub(super) class_has_no_rd: bool,
    /// Whether `#[miniextendr(internal)]` is set, adding `@keywords internal` and
    /// suppressing `@export`/`@exportMethod`.
    pub(super) internal: bool,
    /// Whether `#[miniextendr(noexport)]` is set, suppressing `@export`/`@exportMethod`
    /// without adding `@keywords internal`.
    pub(super) noexport: bool,
}

/// Generate R wrapper code for trait methods and consts, dispatching by class system.
///
/// Calls the appropriate class-system-specific generator (env, s3, s4, s7, r6),
/// then applies post-processing for `@noRd`, `internal`, and `noexport` options:
///
/// - `class_has_no_rd`: Strips roxygen blocks (for S3/vctrs, keeps `@method`/`@export` tags)
/// - `internal`: Replaces `@export`/`@exportMethod` with `@keywords internal`
/// - `noexport`: Removes `@export`/`@exportMethod` entirely
///
/// Returns the complete R wrapper code as a string ready for embedding in a `const`.
pub(super) fn generate_trait_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
    opts: TraitWrapperOpts,
) -> syn::Result<String> {
    let TraitWrapperOpts {
        class_system,
        class_has_no_rd,
        internal,
        noexport,
    } = opts;
    reject_unsupported_describe_in(methods, class_system)?;
    let result = match class_system {
        ClassSystem::Env => generate_trait_env_r_wrapper(type_ident, trait_name, methods, consts)?,
        ClassSystem::S3 => generate_trait_s3_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::S4 => generate_trait_s4_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::S7 => generate_trait_s7_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::R6 => generate_trait_r6_r_wrapper(type_ident, trait_name, methods, consts),
        // vctrs uses S3 under the hood, so use the S3 trait wrapper
        ClassSystem::Vctrs => generate_trait_s3_r_wrapper(type_ident, trait_name, methods, consts),
    };

    // When the impl block has @noRd, suppress documentation generation. A plain
    // `noexport` (without `internal`) is folded into the same gate — it must
    // produce no Rd contribution at all (no alias, no usage entry, nothing on a
    // shared page), same as `@noRd`. `internal` wins if both flags are set on
    // the same impl block (mirrors the standalone-fn precedent, where `internal`
    // + `noexport` together is a compile error). See #431 for the inherent-impl
    // S3 generator's analogous `should_register_s3method = !noexport` rule.
    let suppress_all_rd = class_has_no_rd || (noexport && !internal);
    if suppress_all_rd {
        if matches!(class_system, ClassSystem::S3 | ClassSystem::Vctrs) {
            // A user-written `@noRd` still preserves S3 dispatch registration
            // (`@method`/`@export`) so `S3method()` lands in NAMESPACE — the
            // class stays undocumented but dispatchable. A `noexport`-driven
            // suppression (no explicit `@noRd`) additionally drops `@export`:
            // `noexport` means zero observable trace, not "documented nowhere
            // but still dispatchable".
            let keep_export = class_has_no_rd;
            let mut filtered = Vec::new();
            let mut roxygen_block: Vec<&str> = Vec::new();

            let flush_block = |block: &mut Vec<&str>, out: &mut Vec<String>| {
                if block.iter().any(|line| line.contains("@method ")) {
                    out.push("#' @noRd".to_string());
                    for &line in block.iter() {
                        if line.contains("@method ")
                            || line.contains("@param ")
                            || (keep_export && line.contains("@export"))
                        {
                            out.push(line.to_string());
                        }
                    }
                }
                block.clear();
            };

            for line in result.lines() {
                if line.starts_with("#'") {
                    roxygen_block.push(line);
                    continue;
                }

                if !roxygen_block.is_empty() {
                    flush_block(&mut roxygen_block, &mut filtered);
                }
                filtered.push(line.to_string());
            }

            if !roxygen_block.is_empty() {
                flush_block(&mut roxygen_block, &mut filtered);
            }

            Ok(filtered.join("\n"))
        } else {
            Ok(result
                .lines()
                .filter(|line| !line.starts_with("#'"))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    } else if internal {
        // internal → documented, but @export/@exportMethod becomes @keywords internal
        let has_export = result.lines().any(|line| line.contains("@export"));
        let mut processed: Vec<String> = result
            .lines()
            .flat_map(|line| {
                if line.contains("@export") {
                    vec!["#' @keywords internal".to_string()]
                } else {
                    vec![line.to_string()]
                }
            })
            .collect();
        // For class systems without @export (e.g., Env), insert @keywords internal
        // before the first roxygen tag if no @export line was found to replace.
        if !has_export && let Some(pos) = processed.iter().position(|l| l.starts_with("#'")) {
            processed.insert(pos, "#' @keywords internal".to_string());
        }
        Ok(processed.join("\n"))
    } else {
        Ok(result)
    }
}

// region: author page tags (#1438, #1590)

/// The author tags of a trait-impl method that its own wrapper block
/// forwards besides the `@param` lines: the tags that decide the method's
/// page and where its arguments are documented. `@describeIn` / `@rdname`
/// route the block, `@name` names its topic, `@order` sorts it, and
/// `@inheritParams` / `@inherit` / `@inheritDotParams` fill its arguments.
const PAGE_TAGS: &[&str] = &[
    "describeIn",
    "rdname",
    "name",
    "order",
    "inheritParams",
    "inherit",
    "inheritDotParams",
];

/// The `@param` lines of the method's doc comment.
fn param_tags(method: &TraitMethod) -> impl Iterator<Item = &String> {
    method
        .doc_tags
        .iter()
        .filter(|tag| crate::roxygen::roxygen_tag_name(tag) == Some("param"))
}

/// The page a trait method's own block lands on, and with it the companion
/// block that must share it (the S3 generic block, named after the method):
/// the `@describeIn` destination, the method-level `@rdname`, else the type's
/// shared page. Generics of S4 / S7 and consts always use the type page.
fn method_page<'a>(method: &'a TraitMethod, type_str: &'a str) -> &'a str {
    crate::roxygen::method_page(&method.doc_tags, type_str)
}

/// Whether the method's page is an author topic other than the type page
/// (`@describeIn`, or an `@rdname` naming another page). The topic's own
/// block documents that page, so the method's blocks sort after it
/// (`@order NaN`) and leave the structural `@param` lines (`x`, `...`,
/// `self`) and generated prose to it.
fn joins_author_topic(method: &TraitMethod, type_str: &str) -> bool {
    crate::roxygen::joins_author_topic(&method.doc_tags, Some(type_str))
}

/// Whether the method's arguments are documented by another block: it joins
/// an author topic, or inherits them (`@inheritParams`, `@inherit`). The
/// generated `(undocumented)` fillers are left out then.
fn params_documented_elsewhere(method: &TraitMethod, type_str: &str) -> bool {
    crate::roxygen::params_documented_elsewhere(&method.doc_tags, Some(type_str))
}

/// Whether the method-level `@describeIn` lists the method's own block in
/// its destination's "Functions" section.
fn describes_in(method: &TraitMethod) -> bool {
    crate::roxygen::describe_in_topic(&method.doc_tags).is_some()
}

/// The page lines of a trait method's own wrapper block, as `MethodDocBuilder`
/// writes an inherent method's (#1590):
///
/// - the author's [`PAGE_TAGS`], verbatim;
/// - without an author `@describeIn`: `@name default_name` unless the author
///   named the topic (blocks roxygen2 cannot name from their object need
///   one), `@rdname <Type>` unless the author picked the page, and, on a page
///   the author's `@rdname` splits off, `@title split_title` (roxygen2 skips
///   a page without a title). With `@describeIn` roxygen2 takes the page from
///   the destination and rejects `@name` / `@rdname` next to it, so neither
///   is generated;
/// - `@order NaN` on an author topic the author did not order
///   (`roxygen::orders_after_topic_blocks`).
fn own_block_page_lines(
    method: &TraitMethod,
    type_str: &str,
    default_name: Option<&str>,
    split_title: Option<&str>,
) -> Vec<String> {
    let tags = &method.doc_tags;
    let mut lines = Vec::new();
    if !describes_in(method) {
        let has = |tag: &str| crate::roxygen::has_roxygen_tag(tags, tag);
        if has("rdname")
            && let Some(title) = split_title
        {
            lines.push(format!("#' @title {title}"));
        }
        if !has("name")
            && let Some(name) = default_name
        {
            lines.push(format!("#' @name {name}"));
        }
        if !has("rdname") {
            lines.push(format!("#' @rdname {type_str}"));
        }
    }
    let page_tags: Vec<String> = tags
        .iter()
        .filter(|tag| {
            crate::roxygen::roxygen_tag_name(tag).is_some_and(|name| PAGE_TAGS.contains(&name))
        })
        .cloned()
        .collect();
    crate::roxygen::push_roxygen_tags(&mut lines, &page_tags);
    crate::roxygen::push_order_after_topic_blocks(&mut lines, tags, type_str);
    lines
}

/// Push the method's `@param` lines, each continuation line with its own
/// `#' ` lead.
fn push_param_tags(lines: &mut Vec<String>, method: &TraitMethod) {
    let params: Vec<String> = param_tags(method).cloned().collect();
    crate::roxygen::push_roxygen_tags(lines, &params);
}

/// Reject a method-level `@describeIn` that roxygen2 cannot honour on the
/// trait wrapper (#1590), the rule `ParsedImpl::reject_unsupported_describe_in`
/// applies to inherent methods.
///
/// roxygen2 lists a `@describeIn` block in its destination's "Functions"
/// section through the R object the block documents. The trait wrappers give
/// the method's own block such an object for S3 and vctrs instance methods
/// (the S3 method `generic.Type`), S4 instance methods (the block sits on
/// `methods::setMethod()`, an S4 method object), S4 static methods (the
/// plain function `Type_Trait_method`) and S7 instance methods (the
/// fast-path shortcut `Type_method`). Every other method is a
/// `Type$Trait$method` (or `.Type__Trait$method`) namespace member, whose
/// block roxygen2 drops ("Block must have a @name"), and an S7 instance
/// method with `s7(no_shortcut)` has no block of its own. There the tag is a
/// compile error pointing at `@rdname`, reported for every such method at
/// once.
///
/// The set is wider than the inherent one on S4 instance methods: an
/// inherent S4 method's block sits on the `if (!exists(...))` generic guard,
/// a trait method's on its own `setMethod()` call.
fn reject_unsupported_describe_in(
    methods: &[TraitMethod],
    class_system: ClassSystem,
) -> syn::Result<()> {
    let system = match class_system {
        ClassSystem::S3 => "S3",
        ClassSystem::Vctrs => "vctrs",
        ClassSystem::S4 => "S4",
        ClassSystem::S7 => "S7",
        ClassSystem::Env => "Env",
        ClassSystem::R6 => "R6",
    };
    let mut errors: Option<syn::Error> = None;
    for method in methods.iter().filter(|m| describes_in(m)) {
        let supported = match class_system {
            ClassSystem::S3 | ClassSystem::Vctrs => method.has_self,
            ClassSystem::S4 => true,
            ClassSystem::S7 => method.has_self && !method.no_shortcut,
            ClassSystem::Env | ClassSystem::R6 => false,
        };
        if supported {
            continue;
        }
        let (kind, qualifier) = if !method.has_self {
            ("static method", "")
        } else if matches!(class_system, ClassSystem::S7) {
            ("instance method", " without a fast-path shortcut")
        } else {
            ("instance method", "")
        };
        let error = syn::Error::new_spanned(
            &method.ident,
            format!(
                "`@describeIn` is not supported on the {system} trait {kind} `{}`{qualifier}: \
                 its R wrapper is not an R function roxygen2 can list in the destination's \
                 \"Functions\" section, so roxygen2 would drop the block. Use `@rdname <topic>` \
                 to document it on that page instead. On trait impls `@describeIn` works on \
                 S3, vctrs and S4 instance methods, on S4 static methods, and on S7 instance \
                 methods with a fast-path shortcut.",
                method.ident,
            ),
        );
        match errors.as_mut() {
            Some(all) => all.combine(error),
            None => errors = Some(error),
        }
    }
    errors.map_or(Ok(()), Err)
}

// endregion

/// Generate Env-style R wrapper code for trait methods.
///
/// Env-class trait methods use a namespace hierarchy: `Type$Trait$method(x, ...)`.
/// Instance methods take `x` as the first parameter (the self object) and are
/// stamped with `.__mx_instance__` attribute for `$` dispatch detection.
/// Void instance methods return the receiver `x` for pipe-friendly chaining
/// (invisibly only when marked `Invisible<..>`, #1213).
///
/// Static methods and constants also live under `Type$Trait$name`.
///
/// Returns an error if an instance method has a parameter named `x` (collides
/// with the self parameter in env-class dispatch).
fn generate_trait_env_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> syn::Result<String> {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# Trait methods and consts for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Create trait namespace environment
    lines.push(format!(
        "{}${} <- new.env(parent = emptyenv())",
        type_ident, trait_name
    ));
    lines.push(String::new());

    for method in methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // Trait-namespace assignment target (`Type$Trait$method`), owned by
        // `trait_namespace_target` — see #1141.
        let target = ctx.namespace_target(ClassSystem::Env);

        // Build roxygen tags
        lines.extend(own_block_page_lines(
            method,
            &type_str,
            Some(&target),
            Some(&target),
        ));

        // Check for 'x' parameter collision in instance methods
        if method.has_self {
            for input in &method.sig.inputs {
                if let syn::FnArg::Typed(pt) = input
                    && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
                    && pat_ident.ident == "x"
                {
                    return Err(syn::Error::new_spanned(
                        &pat_ident.ident,
                        "trait instance method parameter cannot be named `x` \
                         (collides with self parameter in env-class dispatch)",
                    ));
                }
            }
        }

        // Build .Call() invocation — C name uses Rust ident, R name uses r_name
        let (full_params, call) = if method.has_self {
            let fp = if ctx.params.is_empty() {
                "x".to_string()
            } else {
                format!("x, {}", ctx.params)
            };
            (fp, ctx.instance_call("x"))
        } else {
            (ctx.params.clone(), ctx.static_call())
        };

        // Generate method wrapper (R-facing name)
        lines.push(format!("{target} <- function({full_params}) {{"));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::Env));
        if method.has_self {
            finish_instance_body(&mut lines, method, "x");
        }
        lines.push("}".to_string());

        // Stamp instance methods with attribute for $ dispatch detection
        if method.has_self {
            lines.push(format!("attr({target}, \".__mx_instance__\") <- TRUE"));
        }

        lines.push(String::new());
    }

    // Generate const wrappers
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let target = trait_namespace_target(ClassSystem::Env, type_ident, trait_name, &const_str);

        // Build roxygen tags
        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        // Build .Call() invocation
        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        // Generate const wrapper
        lines.push(format!("{target} <- function() {{"));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    Ok(lines.join("\n"))
}

/// Generate S3-style R wrapper code (generic + method.Type).
///
/// For `impl Counter for SimpleCounter`, generates:
/// - S3 generic `value(x, ...)` (if not already defined)
/// - S3 method `value.SimpleCounter <- function(x, ...) { .Call(...) }`
/// - S7 method registration if the generic is an S7 generic
///
/// Static methods and constants use `Type$Trait$name` namespace (env-style).
/// Void instance methods return the receiver `x` for pipe-friendly chaining
/// (invisibly only when marked `Invisible<..>`, #1213).
///
/// Also used for `ClassSystem::Vctrs` since vctrs uses S3 under the hood.
fn generate_trait_s3_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# S3 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s3)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Separate instance methods (S3 dispatch) from static methods (namespace access)
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S3 generics + methods for instance methods
    for method in &instance_methods {
        let generic_name = method.r_method_name();
        let s3_method_name = format!("{}.{}", generic_name, type_str);
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // S3 generic roxygen (only create if doesn't exist). The type-qualified
        // @name avoids duplicate aliases across types, but it is also the S3
        // method's own name, so the generic block must follow a method-level
        // `@rdname` / `@describeIn` onto the method's page or R CMD check
        // reports the alias as duplicated across two pages. The `@title` is
        // inert on the type page (the type block's title comes first) and
        // would beat the method's structural title on a split page, so it is
        // only emitted when the block stays on the type page. The prose is a
        // `@description`: a bare line here continues the tag before it
        // (`@rdname` is single-line and roxygen2 skips the page; #1552
        // surfaced this once `@source` stopped absorbing it).
        let tags = &method.doc_tags;
        let generic_roxygen = RoxygenBuilder::new();
        let generic_roxygen =
            if crate::roxygen::has_roxygen_tag(tags, "rdname") || describes_in(method) {
                generic_roxygen
            } else {
                generic_roxygen.title(format!("S3 generic for `{}`", generic_name))
            };
        let generic_roxygen = generic_roxygen
            .name(format!("{}.{}", generic_name, type_str))
            .rdname(method_page(method, &type_str));
        // On an author topic its own block describes the method and documents
        // `x` and `...` (#1590); the structural lines would only compete.
        let generic_roxygen = if joins_author_topic(method, &type_str) {
            generic_roxygen
        } else {
            generic_roxygen
                .description(format!("S3 generic for `{}`", generic_name))
                .custom("@param x An object")
                .custom("@param ... Additional arguments passed to methods")
        };
        let generic_roxygen = if crate::roxygen::orders_after_topic_blocks(tags, &type_str) {
            generic_roxygen.custom(crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS)
        } else {
            generic_roxygen
        };
        let generic_roxygen = generic_roxygen
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .export()
            .build();
        lines.extend(generic_roxygen);

        // S3 generic definition
        lines.push(emit_s3_generic_guard(generic_name.as_str()));
        lines.push(String::new());

        // S3 method roxygen (include @param tags from method doc comments).
        // roxygen2 names the block from the S3 method object, so it needs no
        // generated `@name`.
        lines.extend(own_block_page_lines(
            method,
            &type_str,
            None,
            Some(&s3_method_name),
        ));
        let mut method_roxygen = RoxygenBuilder::new()
            .export()
            .method(&generic_name, &type_str);
        for tag in param_tags(method) {
            method_roxygen = method_roxygen.custom(tag.clone());
        }
        lines.extend(method_roxygen.build());

        // S3 method: generic.class
        let full_params = if ctx.params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", ctx.params)
        };

        // Build .Call() invocation
        let call = ctx.instance_call("x");

        // Always define the S3 method (roxygen expects it for NAMESPACE export)
        lines.push(format!(
            "{} <- function({}) {{",
            crate::naming::r_def_name(&s3_method_name),
            full_params
        ));
        ctx.emit_method_prelude(&mut lines, "  ", &generic_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S3));
        finish_instance_body(&mut lines, method, "x");
        lines.push("}".to_string());

        // Additionally register as S7 method if the generic is S7
        // This ensures S7 dispatch works when the generic was defined by an S7 class
        lines.push(format!(
            "if (inherits(get0(\"{generic_name}\", mode = \"function\"), \"S7_generic\")) {{"
        ));
        lines.push(format!(
            "  S7::method({generic_name}, S7::new_S3_class(\"{type_str}\")) <- {s3_method_name}"
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Create trait namespace for static methods and consts BEFORE assigning to it
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "{}${} <- new.env(parent = emptyenv())",
            type_ident, trait_name
        ));
        lines.push(String::new());
    }

    // Generate static methods in Type$Trait$ namespace
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let target = ctx.namespace_target(ClassSystem::S3);

        // Static method roxygen
        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        lines.extend(own_block_page_lines(method, &type_str, Some(&target), None));

        let call = ctx.static_call();

        lines.push(format!("{target} <- function({}) {{", ctx.params));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S3));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in Type$Trait$ namespace
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let target = trait_namespace_target(ClassSystem::S3, type_ident, trait_name, &const_str);

        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{target} <- function() {{"));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate S4-style R wrapper code.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - `setOldClass("SimpleCounter")` to register the S3 class for S4 dispatch
/// - S4 generic `s4_trait_Counter_value(x, ...)` via `setGeneric()`
/// - S4 method via `setMethod("s4_trait_Counter_value", "SimpleCounter", ...)`
///
/// Generic names are prefixed with `s4_trait_{Trait}_` to avoid collisions
/// with user-defined S4 generics. Static methods and constants are generated
/// as standalone exported functions: `{Type}_{Trait}_{method}()`.
fn generate_trait_s4_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# S4 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s4)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // NOTE: We do NOT call setOldClass here. The inherent impl's class registration
    // (setClass for S4, or setOldClass for S3/env) takes care of that. Calling
    // setOldClass here would clobber a proper S4 setClass with slots.
    lines.push("#' @importFrom methods setGeneric setMethod".to_string());
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S4 generics + methods for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let generic_name = format!("s4_trait_{}_{}", trait_name, method.r_method_name());
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // Build full parameter list (x first, then others, then ...)
        let full_params = if ctx.params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", ctx.params)
        };

        // S4 generic roxygen (include @param tags from method doc comments)
        // S4 generic names are already type-qualified (s4_trait_TypeName_method)
        // so @name won't create duplicate aliases across types. The prose is a
        // `@description`, not a bare line (see the S3 generic above).
        let mut generic_roxygen = RoxygenBuilder::new()
            .description(format!(
                "S4 generic for trait method `{}::{}`",
                trait_name, method_name
            ))
            .name(&generic_name)
            .rdname(&type_str)
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .custom(format!("@param x A `{}` object", type_str))
            .custom("@param ... Additional arguments passed to methods");
        for tag in param_tags(method) {
            generic_roxygen = generic_roxygen.custom(tag.clone());
        }
        lines.extend(generic_roxygen.export().build());

        // Define generic only if it doesn't already exist in THIS namespace
        // (avoid clearing methods). Scoped to topenv(environment()) so an
        // attached installed copy of the package can't satisfy the check and
        // starve the setMethod below during load_all() (#1158).
        lines.push(format!(
            "if (!exists(\"{generic_name}\", where = topenv(environment()), inherits = FALSE)) methods::setGeneric(\"{generic_name}\", function(x, ...) standardGeneric(\"{generic_name}\"))"
        ));
        lines.push(String::new());

        // S4 method roxygen + definition (include @param tags from method doc
        // comments). The block sits on `methods::setMethod()`: roxygen2 names
        // it from the S4 method object, and a split page (#1438) still needs
        // a structural title or roxygen2 skips it.
        lines.extend(own_block_page_lines(
            method,
            &type_str,
            None,
            Some(&generic_name),
        ));
        push_param_tags(&mut lines, method);
        lines.push(format!("#' @exportMethod {}", generic_name));

        lines.push(format!(
            "methods::setMethod(\"{}\", \"{}\", function({}) {{",
            generic_name, type_str, full_params
        ));
        // S4 objects store the ExternalPtr in x@ptr — extract it for .Call()
        lines.push("  .ptr <- x@ptr".to_string());
        let s4_call = ctx.instance_call(".ptr");
        ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
        lines.extend(ctx.method_body_lines(&s4_call, ClassSystem::S4));
        finish_instance_body(&mut lines, method, "x");
        lines.push("})".to_string());
        lines.push(String::new());
    }

    // Generate static methods as standalone functions. S4 objects intercept
    // `$<-`, so these use the flat, class-qualified `Type_Trait_method` name
    // (owned by `trait_namespace_target`) rather than `Type$Trait$method`.
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let fn_name = ctx.namespace_target(ClassSystem::S4);

        // Static method roxygen. A `@describeIn` block is listed in its
        // destination's "Functions" section, whose own block titles the page,
        // so it carries no intro line that could title it instead.
        if !describes_in(method) {
            lines.push(format!(
                "#' Static trait method {}::{}() for {}",
                trait_name, r_name, type_str
            ));
        }
        lines.extend(own_block_page_lines(
            method,
            &type_str,
            Some(&fn_name),
            None,
        ));
        lines.push("#' @export".to_string());

        let call = ctx.static_call();

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S4));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers as standalone functions (flat name, as above)
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let fn_name = trait_namespace_target(ClassSystem::S4, type_ident, trait_name, &const_str);

        let roxygen = RoxygenBuilder::new()
            .name(&fn_name)
            .rdname(&type_str)
            .export()
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{} <- function() {{", fn_name));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate S7-style R wrapper code.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - S7 S3-class wrapper: `.s7_class_SimpleCounter <- S7::new_S3_class("SimpleCounter")`
/// - S7 generic: `s7_trait_Counter_value <- S7::new_generic(...)` (if not exists)
/// - S7 method registration: `S7::method(s7_trait_Counter_value, .s7_class_SimpleCounter) <- ...`
///
/// Generic names are prefixed with `s7_trait_{Trait}_` to avoid collisions.
/// Static methods and constants use `Type$Trait$name` namespace (env-style).
fn generate_trait_s7_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();
    let trait_str = trait_name.to_string();
    let s7_class_var = format!(".s7_class_{}", type_str);

    // Header comment
    lines.push(format!(
        "# S7 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s7)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Use the S7 class object directly for method dispatch.
    // new_S3_class("Foo") creates a descriptor for "Foo" but S7 new_class
    // creates instances with the namespaced class "pkg::Foo", so new_S3_class
    // wouldn't match. Using the class object directly works correctly.
    lines.push("#' @importFrom S7 new_generic method S7_dispatch".to_string());
    lines.push(format!("{} <- {}", s7_class_var, type_str));
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S7 generics + methods for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let generic_name = format!("s7_trait_{}_{}", trait_name, method.r_method_name());
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // Build full parameter list (x first, then others, then ...)
        let full_params = if ctx.params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", ctx.params)
        };

        // S7 generic roxygen
        // Note: Don't include method-specific @param tags here since S7 methods
        // are assignments and won't appear in \usage, which would cause warnings
        // S7 generic names are already type-qualified so @name won't duplicate.
        // The prose is a `@description`, not a bare line (see the S3 generic).
        let generic_roxygen = RoxygenBuilder::new()
            .description(format!(
                "S7 generic for trait method `{}::{}`",
                trait_name, method_name
            ))
            .name(&generic_name)
            .rdname(&type_str)
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .export()
            .build();
        lines.extend(generic_roxygen);

        // S7 generic definition
        lines.push(format!(
            "if (!exists(\"{generic_name}\", mode = \"function\")) {{"
        ));
        lines.push(format!(
            "  {generic_name} <- S7::new_generic(\"{generic_name}\", \"x\", function(x, ...) S7::S7_dispatch())"
        ));
        lines.push("}".to_string());
        lines.push(String::new());

        // S7 method definition
        lines.push(format!(
            "S7::method({}, {}) <- function({}) {{",
            generic_name, s7_class_var, full_params
        ));
        // S7 objects store the ExternalPtr in x@.ptr — extract it for .Call()
        lines.push("  .ptr <- x@.ptr".to_string());
        let s7_call = ctx.instance_call(".ptr");
        ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
        lines.extend(ctx.method_body_lines(&s7_call, ClassSystem::S7));
        finish_instance_body(&mut lines, method, "x");
        lines.push("}".to_string());
        lines.push(String::new());

        // Per-class fast-path dispatch shortcut (#987).
        //
        // Mirror the inherent-impl S7 shortcut (#982, see
        // `miniextendr_impl::s7_class`): alongside the trait generic, emit a
        // plain `<ClassName>_<method>(self, ...)` function that calls `.Call`
        // directly, bypassing `S7::S7_dispatch()`. The receiver is named `self`
        // here (the generic names it `x`) and wired through `self@.ptr`.
        // `s7(no_shortcut)` opts a method out.
        if !method.no_shortcut {
            let shortcut_name = format!("{}_{}", type_str, method.r_method_name());
            let shortcut_formals = if ctx.params.is_empty() {
                "self, ...".to_string()
            } else {
                format!("self, {}, ...", ctx.params)
            };
            let shortcut_call = ctx.instance_call("self@.ptr");

            // Roxygen: shared advisory prose + scaffolding. The shared @rdname
            // page (type_str) already carries the method's prose via the generic
            // block above, so only document `self` + each formal here to keep the
            // shortcut's \usage fully covered (no "undocumented argument" warning).
            //
            // A method-level `@rdname` naming another page or a `@describeIn`
            // sends the shortcut to a topic whose own block documents its
            // arguments (#1590): only the type page needs the structural `self` /
            // `...` lines, and the advisory's description would be a paragraph of
            // the topic's description that names no function. Its first line
            // stays as the block's title, which a page of the shortcut's own
            // (`@rdname` naming a new page) needs.
            let joined = joins_author_topic(method, &type_str);
            let advisory = crate::miniextendr_impl::s7_class::shortcut_advisory_lines(
                &method.r_method_name(),
                &type_str,
            );
            let advisory_len = if joined { 1 } else { advisory.len() };
            lines.extend(advisory.into_iter().take(advisory_len));
            if !joined {
                lines.push(format!("#' @param self A `{}` object.", type_str));
            }
            push_param_tags(&mut lines, method);
            // Auto-document any formal lacking an explicit @param tag. `...` is
            // included so roxygen2 covers it (otherwise R CMD check warns about
            // an undocumented argument). Split on top-level commas only — a
            // naive `split(", ")` breaks a `mode = c("fast", "slow")` default
            // into a bogus `"slow")` formal (undocumented-argument warning). A
            // method that inherits its arguments (`@inheritParams`) gets no
            // `(undocumented)` filler, which would block the inheritance.
            let fillers = !params_documented_elsewhere(method, &type_str);
            for formal in crate::roxygen::split_r_formals(&shortcut_formals) {
                let pname = crate::roxygen::formal_name(formal);
                if joined || pname == "self" {
                    continue;
                }
                let documented = crate::roxygen::param_documented(&method.doc_tags, pname);
                if documented {
                    continue;
                }
                if pname == "..." {
                    lines.push(
                        "#' @param ... Additional arguments; ignored by the fast-path shortcut."
                            .to_string(),
                    );
                } else if fillers {
                    lines.push(format!("#' @param {} (undocumented)", pname));
                }
            }
            lines.extend(own_block_page_lines(
                method,
                &type_str,
                Some(&shortcut_name),
                None,
            ));
            lines.extend(crate::roxygen::source_tag(format!(
                "Generated by miniextendr from `impl {} for {}` (`{}` shortcut)",
                trait_name, type_ident, method_name
            )));
            lines.push("#' @export".to_string());

            lines.push(format!(
                "{} <- function({}) {{",
                shortcut_name, shortcut_formals
            ));
            ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
            lines.extend(ctx.method_body_lines(&shortcut_call, ClassSystem::S7));
            finish_instance_body(&mut lines, method, "self");
            lines.push("}".to_string());
            lines.push(String::new());
        }
    }

    // Create trait namespace for static methods and consts.
    // For S7 classes, use a local variable + attr() to avoid S7's $<- interception.
    let trait_env_var = trait_namespace_env_var(type_ident, trait_name);
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!("{} <- new.env(parent = emptyenv())", trait_env_var));
        lines.push(String::new());
    }

    // Generate static methods in trait namespace. The wrapper is *assigned*
    // into the local env (`trait_namespace_target(S7, ..)` = `.Type__Trait$m`),
    // but its documented `@name` is the call-site form `Type$Trait$m` — S7's
    // `$` on the class object falls through to the attached attribute, so users
    // still spell it `Type$Trait$m`.
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        let name = format!("{}${}${}", type_str, trait_str, r_name);
        lines.extend(own_block_page_lines(method, &type_str, Some(&name), None));

        let call = ctx.static_call();

        lines.push(format!(
            "{} <- function({}) {{",
            ctx.namespace_target(ClassSystem::S7),
            ctx.params
        ));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S7));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in trait namespace (assigned into `.Type__Trait`,
    // documented as `Type$Trait$const` — see the static-method note above).
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();

        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, const_str))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!(
            "{} <- function() {{",
            trait_namespace_target(ClassSystem::S7, type_ident, trait_name, &const_str)
        ));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Attach the trait env to the S7 class via attr() to bypass S7's $<- interception.
    // R's $ accessor on S7 objects falls through to attributes, so Type$Trait$method still works.
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "attr({}, \"{}\") <- {}",
            type_ident, trait_name, trait_env_var
        ));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate R6-style R wrapper code.
///
/// R6 classes are defined monolithically (all methods in `R6Class()`), so trait
/// methods cannot be injected into the class definition. Instead, both instance
/// and static trait methods live in the class-scoped `Type$Trait$name`
/// namespace (env-style) — the R6 generator object is an environment, so
/// `Type$Trait <- new.env()` attaches cleanly and `Type$Trait$method(x)`
/// resolves at the call site.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - `SimpleCounter$Counter$value(x)`      -- instance method (takes the object)
/// - `SimpleCounter$Counter$increment(x)`  -- instance method
///
/// This class-qualified shape is collision-free by construction: two R6 impls
/// of one trait on different types no longer share an unqualified
/// `r6_trait_<Trait>_<method>` name (#1115). It also unifies R6 with the Env/S3
/// namespace shape (#1141) — R6 instance and static methods previously
/// disagreed on shape for no functional reason.
fn generate_trait_r6_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# R6 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(r6)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push("# Note: R6 trait methods live in the Type$Trait$method namespace".to_string());
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Create the trait namespace env up front — instance methods now live in it
    // too (not just static methods / consts).
    if !methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "{}${} <- new.env(parent = emptyenv())",
            type_ident, trait_name
        ));
        lines.push(String::new());
    }

    // Generate instance methods in the Type$Trait$ namespace
    for method in &instance_methods {
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let target = ctx.namespace_target(ClassSystem::R6);

        // Build parameter list (x first, then others)
        let full_params = if ctx.params.is_empty() {
            "x".to_string()
        } else {
            format!("x, {}", ctx.params)
        };

        // Namespace-member roxygen — a `$<-` assignment target, so roxygen emits
        // no `\usage` and needs no per-formal `@param` docs (matches Env; #1141).
        lines.extend(own_block_page_lines(
            method,
            &type_str,
            Some(&target),
            Some(&target),
        ));

        let call = ctx.instance_call(".ptr");

        lines.push(format!("{target} <- function({full_params}) {{"));
        // R6 objects store the ExternalPtr in private$.ptr — extract it for .Call()
        lines.push("  .ptr <- x$.__enclos_env__$private$.ptr".to_string());
        ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
        lines.extend(ctx.method_body_lines(&call, ClassSystem::R6));
        finish_instance_body(&mut lines, method, "x");
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate static methods in Type$Trait$ namespace
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let target = ctx.namespace_target(ClassSystem::R6);

        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        lines.extend(own_block_page_lines(method, &type_str, Some(&target), None));

        let call = ctx.static_call();

        lines.push(format!("{target} <- function({}) {{", ctx.params));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::R6));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in Type$Trait$ namespace
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let target = trait_namespace_target(ClassSystem::R6, type_ident, trait_name, &const_str);

        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{target} <- function() {{"));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Close an instance-method body (#1213).
///
/// A void method hands back its receiver (`x` / `self`) so calls chain under
/// the pipe; the receiver is returned visibly, like every other return. A
/// method whose declared return is `Invisible<..>` wraps its final expression
/// (the receiver, or the converted `.val` / classed value) in `invisible()`.
fn finish_instance_body(lines: &mut Vec<String>, method: &super::TraitMethod, receiver: &str) {
    if method.returns_unit() {
        lines.push(format!("  {receiver}"));
    }
    if method.is_invisible()
        && let Some(last) = lines.last_mut()
    {
        let expr = last.trim_start();
        let indent = &last[..last.len() - expr.len()];
        *last = format!("{indent}invisible({expr})");
    }
}
