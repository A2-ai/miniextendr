//! `$` / `[[` field syntax for the `#[r_data]` fields of S3, S4 and env
//! classes: `s3(r_data_accessors)`, `s4(r_data_accessors)`,
//! `env(r_data_accessors)`, and their getters-only form
//! `r_data_accessors = "get"` (#1848).
//!
//! The methods call the helpers `#[derive(ExternalPtr)]` emits for the type
//! (`.rdata_fields_T`, `.rdata_get_T`, `.rdata_set_T`, `.rdata_no_field_T`,
//! see `externalptr_derive::generate_field_syntax_helpers`), which call the
//! per-field `.Call()` accessors. Those carry no call slot (#344, #348), so
//! each method raises a tagged condition value with its own frame as the
//! fallback (`environment()`), which `.miniextendr_frame_call` turns into the
//! generic's call: `x$keys`, `x$keys <- value` (#1851).
//!
//! On an S3 bare pointer the reads look the fields up as a list's `$` / `[[`
//! do (#1891): `$` takes an exact name, else a unique prefix (with the
//! `warnPartialMatchDollar` warning), else `NULL`; `[[` takes an exact name,
//! else `NULL`, or a position in the field order (truncated like a list's,
//! `subscriptOutOfBoundsError` past the last field, through the preamble's
//! `.miniextendr_field_at`). The S3 class also gets `names()` (the field
//! names), `as.list()` (every field, in order, `NULL` fields kept; #1890) and
//! `.DollarNames()` (#1885), and S4 and env classes `.DollarNames()`. A name
//! that is not a field falls through to R's own `$` / `[[` / `$<-` / `[[<-`
//! (`NextMethod()`) on a list or an environment receiver, as do `names()` and
//! `as.list()` there (`as.list.environment()` by name on an environment,
//! whose class attribute would send `NextMethod()` to the default method,
//! which can't coerce one), and the writes raise `miniextendr_no_field`, naming the
//! fields, on a bare pointer (an S3 or env object), as `$` does on an S4
//! object, which have nothing to fall through to. A `[[` / `[[<-` index that
//! is neither a name nor (for `[[` on a bare pointer) a position always falls
//! through. Every replacement method puts `value` last and returns the object
//! (#1853).
//!
//! `.DollarNames()` is registered with `@exportS3Method utils::.DollarNames`
//! (`S3method(utils::.DollarNames, Class)`): the `@method` + `@export` pair
//! writes an unqualified `S3method(.DollarNames, Class)`, which fails to load
//! with only base attached (`R CMD check` loads a namespace that way). S4
//! takes the same S3 method, which `utils` dispatches on the S4 class.

use super::{ClassSystem, ParsedImpl};
use crate::naming::{RDataHelper, rdata_helper_name};

/// The names of the type's field helpers.
struct Helpers {
    fields: String,
    get: String,
    set: String,
    no_field: String,
}

impl Helpers {
    fn new(parsed_impl: &ParsedImpl) -> Self {
        let type_name = parsed_impl.type_ident.to_string();
        Helpers {
            fields: rdata_helper_name(RDataHelper::Fields, &type_name),
            get: rdata_helper_name(RDataHelper::Get, &type_name),
            set: rdata_helper_name(RDataHelper::Set, &type_name),
            no_field: rdata_helper_name(RDataHelper::NoField, &type_name),
        }
    }
}

/// The tagged-condition guard after an accessor call: the method's frame is
/// the raise fallback, resolved to the generic's call (#1851).
fn guard(indent: &str) -> Vec<String> {
    crate::method_return_builder::condition_check_lines_with_default(indent, "environment()")
}

/// The test of a `[[` / `[[<-` index (or env's `$` / `[[` name): a field
/// name is a length-1 character vector naming a field.
fn not_a_field_name(var: &str, fields: &str) -> String {
    format!("!is.character({var}) || length({var}) != 1L || is.na(match({var}, {fields}))")
}

/// A top-level check, emitted next to the class's methods, that the type has
/// the field helpers: an impl that opts in for a type without an `RSidecar`
/// selector or without a `pub` `#[r_data]` field fails at install time
/// instead of at the first `$` (for env, at every method call).
fn load_check(parsed_impl: &ParsedImpl, system: &str) -> String {
    let helpers = Helpers::new(parsed_impl);
    let type_name = parsed_impl.type_ident.to_string();
    format!(
        "if (!is.character(get0(\"{fields}\", envir = topenv(environment()), inherits = FALSE))) \
         stop(\"`{system}(r_data_accessors)` on `{type_name}` needs the field helpers \
         `#[derive(ExternalPtr)]` emits for a type with an `#[r_data]` `RSidecar` selector and a \
         `pub` `#[r_data]` field\", call. = FALSE)",
        fields = helpers.fields,
    )
}

/// The `@param` lines of the field methods on the class page, without the
/// names another block documents there: roxygen2 keeps the last `@param` of
/// a name on a page, and these blocks must not replace the constructor's
/// text, nor the `x` / `value` of the standalone `Type_get_f(x)` /
/// `Type_set_f(x, value)`, which the derive documents on the page named after
/// the type (the class page unless `class = "..."` renames the class).
fn param_lines(parsed_impl: &ParsedImpl, params: &[(&str, &str)]) -> Vec<String> {
    let mut documented: Vec<String> = parsed_impl
        .constructor()
        .map(|ctor| {
            crate::r_wrapper_builder::r_formal_names(&ctor.sig.inputs)
                .map(|(name, _)| name)
                .collect()
        })
        .unwrap_or_default();
    if parsed_impl.type_ident == parsed_impl.class_name() {
        documented.extend(["x".to_string(), "value".to_string()]);
    }
    params
        .iter()
        .filter(|(name, _)| !documented.iter().any(|f| f == name))
        .map(|(name, text)| format!("#' @param {name} {text}"))
        .collect()
}

/// The roxygen block of an S3 or env field method `<generic>.<Class>`: on the
/// class page for an exported class with a page, else no page. The method is
/// registered (`S3method()`) when `register` is set.
fn s3_method_doc(
    lines: &mut Vec<String>,
    parsed_impl: &ParsedImpl,
    generic: &str,
    on_page: bool,
    register: bool,
    params: &[(&str, &str)],
) {
    let class_name = parsed_impl.class_name();
    if on_page {
        lines.push(format!("#' @rdname {class_name}"));
        lines.extend(param_lines(parsed_impl, params));
    } else {
        lines.push("#' @noRd".to_string());
    }
    lines.push(format!("#' @method {generic} {class_name}"));
    if register {
        lines.push("#' @export".to_string());
    }
}

/// The roxygen block of a `.DollarNames.<Class>` method: on the class page
/// for an exported class with a page, else no page, and registered with
/// `@exportS3Method utils::.DollarNames` when `register` is set (see the
/// module docs for why not `@method` + `@export`).
fn dollar_names_doc(
    lines: &mut Vec<String>,
    parsed_impl: &ParsedImpl,
    on_page: bool,
    register: bool,
) {
    if on_page {
        lines.push(format!("#' @rdname {}", parsed_impl.class_name()));
        lines.extend(param_lines(
            parsed_impl,
            &[
                ("x", "An object."),
                (
                    "pattern",
                    "A regular expression; the names matching it are the completions.",
                ),
            ],
        ));
    } else {
        lines.push("#' @noRd".to_string());
    }
    if register {
        lines.push("#' @exportS3Method utils::.DollarNames".to_string());
    }
}

/// `$.Class`, `[[.Class`, `names.Class`, `as.list.Class` and
/// `.DollarNames.Class`, and with setters `$<-.Class` / `[[<-.Class`, for an
/// S3 class with `s3(r_data_accessors)`.
///
/// The receiver is the classed pointer the constructor (or the package's own
/// R code) returns, on which the reads look the fields up as a list's do, or
/// a list or environment carrying the pointer in `.ptr` (#1469), whose own
/// elements a name that is not a field reaches through `NextMethod()`.
pub(super) fn s3_field_methods(
    parsed_impl: &ParsedImpl,
    on_page: bool,
    register: bool,
) -> Vec<String> {
    let class_name = parsed_impl.class_name();
    let h = Helpers::new(parsed_impl);
    let def = |generic: &str| crate::naming::r_def_name(&format!("{generic}.{class_name}"));
    let mut lines = vec![
        format!(
            "# Field syntax for the `#[r_data]` fields of {} (`s3(r_data_accessors)`).",
            parsed_impl.type_ident
        ),
        load_check(parsed_impl, "s3"),
        String::new(),
    ];

    let x = ("x", "An object.");
    let dots = ("...", "Additional arguments.");
    // One text per name for the page: roxygen2 keeps the last `@param` of a
    // name on a page, so the reader's and the writer's blocks say the same.
    let name = (
        "name",
        "A field name; `$` on the pointer itself also takes a unique prefix of one, as for a \
         list.",
    );
    let i = (
        "i",
        "A field name; `[[` on the pointer itself also takes a position in the field order. \
         Any other index goes to the object's own `[[` / `[[<-`.",
    );
    let value = ("value", "The new value of the field.");

    // `$`: on a bare pointer a list's lookup (`pmatch`: exact, then a unique
    // prefix, else NULL), with R's partial-match warning on the generic's
    // call; on a list or environment receiver an exact field, else its own.
    s3_method_doc(&mut lines, parsed_impl, "$", on_page, register, &[x, name]);
    lines.push(format!("{} <- function(x, name) {{", def("$")));
    lines.push("  if (typeof(x) == \"externalptr\") {".to_string());
    lines.push(format!("    i <- pmatch(name, {})", h.fields));
    lines.push("    if (is.na(i)) return(NULL)".to_string());
    lines.push(format!("    if ({}[[i]] != name) {{", h.fields));
    lines.push(format!(
        "      if (isTRUE(getOption(\"warnPartialMatchDollar\"))) warning(simpleWarning(sprintf(\"partial match of '%s' to '%s'\", name, {}[[i]]), .miniextendr_frame_call(environment())))",
        h.fields
    ));
    lines.push(format!("      name <- {}[[i]]", h.fields));
    lines.push("    }".to_string());
    lines.push(format!(
        "  }} else if (is.na(match(name, {}))) {{",
        h.fields
    ));
    lines.push("    return(NextMethod())".to_string());
    lines.push("  }".to_string());
    lines.push(format!("  .val <- {}(x, name)", h.get));
    lines.extend(guard("  "));
    lines.push("  .val".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    // `[[`: on a bare pointer a position (a number, or a logical as a list
    // takes one) picks a field as a list's `[[` picks an element, `NA` first
    // since a character vector's `[[NA]]` is `NA` where a list's is NULL
    // (`.miniextendr_field_at`), an exact name its field, and an unknown
    // name is NULL; a list or environment receiver reaches its own `[[` for
    // everything but an exact field.
    s3_method_doc(
        &mut lines,
        parsed_impl,
        "[[",
        on_page,
        register,
        &[x, i, dots],
    );
    lines.push(format!("{} <- function(x, i, ...) {{", def("[[")));
    lines.push(
        "  if (typeof(x) == \"externalptr\" && length(i) == 1L && (is.numeric(i) || is.logical(i))) {"
            .to_string(),
    );
    lines.push("    if (is.na(i)) return(NULL)".to_string());
    lines.push(format!(
        "    i <- .miniextendr_field_at({}, i, environment())",
        h.fields
    ));
    lines.push(format!(
        "  }} else if ({}) {{",
        not_a_field_name("i", &h.fields)
    ));
    lines.push(
        "    if (typeof(x) == \"externalptr\" && is.character(i) && length(i) == 1L) return(NULL)"
            .to_string(),
    );
    lines.push("    return(NextMethod())".to_string());
    lines.push("  }".to_string());
    lines.push(format!("  .val <- {}(x, i)", h.get));
    lines.extend(guard("  "));
    lines.push("  .val".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    // `names()` and `as.list()` on a bare pointer only (#1890): a list or
    // environment receiver keeps R's own, so `names(x)` agrees with
    // `length(x)` and `x[[i]]` there. An environment gets
    // `as.list.environment()` by name: its class attribute would send
    // `NextMethod()` to `as.list.default()`, which can't coerce one.
    // `as.list()` reads every field through the getters, in declared order,
    // and keeps a NULL field (`out[k] <- list(NULL)`; `out[[k]] <- NULL`
    // would drop it).
    s3_method_doc(&mut lines, parsed_impl, "names", on_page, register, &[x]);
    lines.push(format!(
        "{} <- function(x) if (typeof(x) == \"externalptr\") {} else NextMethod()",
        def("names"),
        h.fields
    ));
    lines.push(String::new());

    s3_method_doc(
        &mut lines,
        parsed_impl,
        "as.list",
        on_page,
        register,
        &[x, dots],
    );
    lines.push(format!("{} <- function(x, ...) {{", def("as.list")));
    lines.push("  if (is.environment(x)) return(as.list.environment(x, ...))".to_string());
    lines.push("  if (typeof(x) != \"externalptr\") return(NextMethod())".to_string());
    lines.push(format!("  .out <- vector(\"list\", length({}))", h.fields));
    lines.push(format!("  names(.out) <- {}", h.fields));
    lines.push(format!("  for (.k in seq_along({})) {{", h.fields));
    lines.push(format!("    .val <- {}(x, {}[[.k]])", h.get, h.fields));
    lines.extend(guard("    "));
    lines.push("    .out[.k] <- list(.val)".to_string());
    lines.push("  }".to_string());
    lines.push("  .out".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    // `.DollarNames()` (#1885): the fields, then a list's or environment's
    // own names. It reads only names, so it never raises.
    dollar_names_doc(&mut lines, parsed_impl, on_page, register);
    lines.push(format!(
        "{} <- function(x, pattern = \"\") grep(pattern, c({}, if (typeof(x) != \"externalptr\") names(x)), value = TRUE)",
        def(".DollarNames"),
        h.fields
    ));
    lines.push(String::new());

    if !parsed_impl.r_data_accessors.setters() {
        return lines;
    }

    s3_method_doc(
        &mut lines,
        parsed_impl,
        "$<-",
        on_page,
        register,
        &[x, name, value],
    );
    lines.push(format!("{} <- function(x, name, value) {{", def("$<-")));
    lines.push(format!("  if (is.na(match(name, {}))) {{", h.fields));
    lines.push(format!(
        "    if (typeof(x) == \"externalptr\") {}(name, environment())",
        h.no_field
    ));
    lines.push("    return(NextMethod())".to_string());
    lines.push("  }".to_string());
    lines.push(format!("  .val <- {}(x, name, value)", h.set));
    lines.extend(guard("  "));
    lines.push("  x".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    s3_method_doc(
        &mut lines,
        parsed_impl,
        "[[<-",
        on_page,
        register,
        &[x, i, dots, value],
    );
    lines.push(format!("{} <- function(x, i, ..., value) {{", def("[[<-")));
    lines.push(format!("  if ({}) {{", not_a_field_name("i", &h.fields)));
    lines.push(format!(
        "    if (typeof(x) == \"externalptr\" && is.character(i) && length(i) == 1L) {}(i, environment())",
        h.no_field
    ));
    lines.push("    return(NextMethod())".to_string());
    lines.push("  }".to_string());
    lines.push(format!("  .val <- {}(x, i, value)", h.set));
    lines.extend(guard("  "));
    lines.push("  x".to_string());
    lines.push("}".to_string());
    lines.push(String::new());
    lines
}

/// `setMethod("$")`, `.DollarNames.Class`, and with setters
/// `setMethod("$<-")`, for an S4 class with `s4(r_data_accessors)`. An S4
/// object of the class is no list (its only slot is `ptr`), so a name that
/// is not a field raises `miniextendr_no_field`. The methods pass `x@ptr` to
/// the accessors. `.DollarNames` is an S3 method, which `utils` dispatches on
/// the S4 class; it is registered unless the class is `noexport`, as an S3
/// class's methods are.
pub(super) fn s4_field_methods(
    parsed_impl: &ParsedImpl,
    on_page: bool,
    export: bool,
) -> Vec<String> {
    let class_name = parsed_impl.class_name();
    let h = Helpers::new(parsed_impl);
    let mut lines = vec![
        format!(
            "# Field syntax for the `#[r_data]` fields of {} (`s4(r_data_accessors)`).",
            parsed_impl.type_ident
        ),
        load_check(parsed_impl, "s4"),
        String::new(),
    ];
    let doc = |lines: &mut Vec<String>, generic: &str, params: &[(&str, &str)]| {
        if on_page {
            lines.push(format!("#' @rdname {class_name}"));
            lines.extend(param_lines(parsed_impl, params));
        } else {
            lines.push("#' @noRd".to_string());
        }
        if export {
            lines.push(format!("#' @exportMethod {generic}"));
        }
    };
    let x = ("x", "An object.");
    let name = ("name", "A field name.");
    let value = ("value", "The new value of the field.");

    doc(&mut lines, "$", &[x, name]);
    lines.push(format!(
        "methods::setMethod(\"$\", \"{class_name}\", function(x, name) {{"
    ));
    lines.push(format!(
        "  if (is.na(match(name, {}))) {}(name, environment())",
        h.fields, h.no_field
    ));
    lines.push(format!("  .val <- {}(x@ptr, name)", h.get));
    lines.extend(guard("  "));
    lines.push("  .val".to_string());
    lines.push("})".to_string());
    lines.push(String::new());

    dollar_names_doc(&mut lines, parsed_impl, on_page, !parsed_impl.noexport);
    lines.push(format!(
        "{} <- function(x, pattern = \"\") grep(pattern, {}, value = TRUE)",
        crate::naming::r_def_name(&format!(".DollarNames.{class_name}")),
        h.fields
    ));
    lines.push(String::new());

    if parsed_impl.r_data_accessors.setters() {
        doc(&mut lines, "$<-", &[x, name, value]);
        lines.push(format!(
            "methods::setMethod(\"$<-\", \"{class_name}\", function(x, name, value) {{"
        ));
        lines.push(format!(
            "  if (is.na(match(name, {}))) {}(name, environment())",
            h.fields, h.no_field
        ));
        lines.push(format!("  .val <- {}(x@ptr, name, value)", h.set));
        lines.extend(guard("  "));
        lines.push("  x".to_string());
        lines.push("})".to_string());
        lines.push(String::new());
    }
    lines
}

/// The field branch at the top of an env class's `$.Class` (and its
/// `[[.Class` alias), ahead of the method lookup: a field wins over a method
/// of the same name, as R6's active bindings do, and a field read doesn't
/// pay for the lookup.
pub(super) fn env_field_branch(parsed_impl: &ParsedImpl) -> Vec<String> {
    let h = Helpers::new(parsed_impl);
    let mut lines = vec![format!(
        "  if (!({})) {{",
        not_a_field_name("name", &h.fields)
    )];
    lines.push(format!("    .val <- {}(self, name)", h.get));
    lines.extend(guard("    "));
    lines.push("    return(.val)".to_string());
    lines.push("  }".to_string());
    lines
}

/// The load check of an env class with `env(r_data_accessors)`.
pub(super) fn env_load_check(parsed_impl: &ParsedImpl) -> String {
    load_check(parsed_impl, "env")
}

/// `.DollarNames.Class` for an env class with `env(r_data_accessors)`: the
/// fields, then the class's methods (`ls(<Class>)`), since `$` reaches both.
/// `on_page` follows the `$.Class` block's gating; the method is always
/// registered, as `$.Class` is.
pub(super) fn env_dollar_names(parsed_impl: &ParsedImpl, on_page: bool) -> Vec<String> {
    let class_name = parsed_impl.class_name();
    let h = Helpers::new(parsed_impl);
    let mut lines = Vec::new();
    dollar_names_doc(&mut lines, parsed_impl, on_page, true);
    lines.push(format!(
        "{} <- function(x, pattern = \"\") grep(pattern, c({}, ls({})), value = TRUE)",
        crate::naming::r_def_name(&format!(".DollarNames.{class_name}")),
        h.fields,
        class_name
    ));
    lines
}

/// `$<-.Class` and its `[[<-.Class` alias for an env class with
/// `env(r_data_accessors)`. An env object is a classed pointer, so a name
/// that is not a field raises `miniextendr_no_field`; a list or environment
/// carrying the pointer in `.ptr` writes its own element (`NextMethod()`).
/// `on_page` follows the `$.Class` block's gating; the methods are always
/// registered, as `$.Class` is.
pub(super) fn env_replacement_methods(parsed_impl: &ParsedImpl, on_page: bool) -> Vec<String> {
    let class_name = parsed_impl.class_name();
    let h = Helpers::new(parsed_impl);
    let mut lines = Vec::new();
    let x = ("x", "An object.");
    let value = ("value", "The new value of the field.");
    s3_method_doc(&mut lines, parsed_impl, "$<-", on_page, true, &[x, value]);
    lines.push(format!("`$<-.{class_name}` <- function(x, name, value) {{"));
    lines.push(format!("  if ({}) {{", not_a_field_name("name", &h.fields)));
    lines.push(format!(
        "    if (typeof(x) == \"externalptr\" && is.character(name) && length(name) == 1L) {}(name, environment())",
        h.no_field
    ));
    lines.push("    return(NextMethod())".to_string());
    lines.push("  }".to_string());
    lines.push(format!("  .val <- {}(x, name, value)", h.set));
    lines.extend(guard("  "));
    lines.push("  x".to_string());
    lines.push("}".to_string());
    s3_method_doc(&mut lines, parsed_impl, "[[<-", on_page, true, &[]);
    lines.push(format!("`[[<-.{class_name}` <- `$<-.{class_name}`"));
    lines
}

/// Refuses an impl method on a generic the field methods define: two
/// definitions of `$.Class` (or `setMethod("$", "Class")`) would follow, and
/// the later one would win. The getters-only form generates no `$<-` /
/// `[[<-`, so a hand-written one is the point of it; the readers (`$`, `[[`,
/// `names`, `as.list` and `.DollarNames` on S3, `$` and `.DollarNames` on
/// S4) are generated in both forms. Env classes have no check: their
/// methods are not S3 methods.
pub(super) fn check_collisions(parsed_impl: &ParsedImpl) -> syn::Result<()> {
    let setters = parsed_impl.r_data_accessors.setters();
    let generated: &[&str] = match (parsed_impl.class_system, setters) {
        _ if !parsed_impl.r_data_accessors.enabled() => return Ok(()),
        (ClassSystem::S3, true) => &["$", "[[", "names", "as.list", ".DollarNames", "$<-", "[[<-"],
        (ClassSystem::S3, false) => &["$", "[[", "names", "as.list", ".DollarNames"],
        (ClassSystem::S4, true) => &["$", ".DollarNames", "$<-"],
        (ClassSystem::S4, false) => &["$", ".DollarNames"],
        _ => return Ok(()),
    };
    let class_name = parsed_impl.class_name();
    for method in parsed_impl.instance_methods() {
        // S3 dispatches an instance method under its generic, else its R
        // name; S4 under its generic, else `s4_<name>`.
        let generic = match parsed_impl.class_system {
            ClassSystem::S3 => method
                .method_attrs
                .generic
                .clone()
                .unwrap_or_else(|| method.r_method_name()),
            _ => match &method.method_attrs.generic {
                Some(generic) => generic.clone(),
                None => continue,
            },
        };
        // A method on another class (`s3(generic = "$", class = "other")`)
        // defines another class's method.
        if method
            .method_attrs
            .class
            .as_deref()
            .is_some_and(|class| class != class_name)
        {
            continue;
        }
        if !generated.contains(&generic.as_str()) {
            continue;
        }
        let system = if parsed_impl.class_system == ClassSystem::S3 {
            "s3"
        } else {
            "s4"
        };
        let advice = if setters && generic.ends_with("<-") {
            format!(
                " To write the fields with your own replacement methods, use the getters-only \
                 form `{system}(r_data_accessors = \"get\")`, which generates no `$<-` / `[[<-`."
            )
        } else {
            String::new()
        };
        return Err(syn::Error::new_spanned(
            &method.ident,
            format!(
                "`{}` defines the `{generic}` method of `{class_name}`, which \
                 `{system}(r_data_accessors)` generates for the `#[r_data]` fields: the class \
                 would get two `{generic}` methods. Remove the method or the option.{advice}",
                method.ident
            ),
        ));
    }
    Ok(())
}
