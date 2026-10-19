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
//! No reader matches a prefix of a name (#1922). On an S3 bare pointer `$`
//! and `[[` take an exact field name, and `[[` also a whole-number position
//! in the field order (`subscriptOutOfBoundsError` past the last field,
//! through the preamble's `.miniextendr_field_at`); an unknown name raises
//! `miniextendr_no_field` there, as a write does (#1921).
//! `s3(r_data_accessors, lookup = "list")` keeps a list's other lenience
//! (#1891): an unknown name or `NA` reads `NULL`, and `[[` takes a logical
//! or a fractional position (truncated) as a list's does. In both, a `[[`
//! index that is neither one name nor one position raises the preamble's
//! `.miniextendr_field_index_error` (the type's `field_error` classes, then
//! `notSubsettableError`, on the user's call) instead of reaching
//! `[[.default` (#1920). The S3 class also gets `names()` (the field names),
//! `as.list()` (every field, in order, `NULL` fields kept; #1890) and
//! `.DollarNames()` (#1885), and S4 and env classes `.DollarNames()`. A name
//! that is not a field falls through on a list or an environment receiver,
//! by the exact name: `$` reads a list's element with `.subset2()` (R's `$`
//! on a list matches a prefix) and an environment's through `NextMethod()`,
//! `[[` and the writes go to R's own `[[` (with `exact = TRUE`) / `$<-` /
//! `[[<-`, as do `names()` and `as.list()` there (`as.list.environment()`
//! by name on an environment, whose class attribute would send
//! `NextMethod()` to the default method, which can't coerce one). The writes
//! raise `miniextendr_no_field`, naming the R class and its fields, on a bare
//! pointer (an S3 or env object), as `$` does on an S4 object, which have
//! nothing to fall through to. A `[[<-` index that is not a name falls
//! through. Under `restored(refuse = "every_read")` every S3 reader first
//! checks a pointer, list or environment receiver (#1901, #1919): a restored
//! pointer and a list or environment without one are refused before the
//! index is looked at. Every replacement method puts `value` last and
//! returns the object (#1853).
//!
//! A field fallback (`fallback = "Self::f"`, #2150; see
//! [`FieldFallback`](super::FieldFallback)) answers the names no field has
//! on the bare pointer, through a `.Call` entry the impl block emits
//! ([`fallback_c_entries`]): `$` and `[[` by name on an S3 object, `$` on an
//! S4 object, and `$` / `[[` on an env object after its methods. The
//! fallback's `None` reads what an unknown name reads without it. Fields win,
//! a position counts the fields only, `names()`, `as.list()` and the writes
//! don't see it, and a list or environment receiver keeps its own elements
//! for those names. With `fallback_names = "Self::g"`, `.DollarNames()`
//! offers the fallback's names after the fields (and an env class's
//! methods).
//!
//! `.DollarNames()` is registered with `@exportS3Method utils::.DollarNames`
//! (`S3method(utils::.DollarNames, Class)`): the `@method` + `@export` pair
//! writes an unqualified `S3method(.DollarNames, Class)`, which fails to load
//! with only base attached (`R CMD check` loads a namespace that way). S4
//! takes the same S3 method, which `utils` dispatches on the S4 class.

use super::{ClassSystem, FieldLookup, ParsedImpl};
use crate::naming::{RDataHelper, rdata_helper_name};

/// The names of the type's field helpers.
struct Helpers {
    fields: String,
    get: String,
    set: String,
    no_field: String,
    /// The classes in front of the methods' own errors (#1899), `NULL`
    /// without `field_error(...)`.
    error_class: String,
    /// `TRUE` under `restored(refuse = "every_read")` (#1901): it gates the
    /// calls of `live`.
    every_read: String,
    /// The liveness check of `restored(refuse = "every_read")` (#1901), a
    /// stub that is never called without it.
    live: String,
    /// The `.Call` entry of the impl's `fallback = "..."` (#2150), for a
    /// name no field has ([`fallback_c_entries`]).
    fallback: Option<String>,
    /// The `.Call` entry of its `fallback_names = "..."`, for
    /// `.DollarNames()`.
    fallback_names: Option<String>,
}

impl Helpers {
    fn new(parsed_impl: &ParsedImpl) -> Self {
        let type_name = parsed_impl.type_ident.to_string();
        Helpers {
            fields: rdata_helper_name(RDataHelper::Fields, &type_name),
            get: rdata_helper_name(RDataHelper::Get, &type_name),
            set: rdata_helper_name(RDataHelper::Set, &type_name),
            no_field: rdata_helper_name(RDataHelper::NoField, &type_name),
            error_class: rdata_helper_name(RDataHelper::ErrorClass, &type_name),
            every_read: rdata_helper_name(RDataHelper::EveryRead, &type_name),
            live: rdata_helper_name(RDataHelper::Live, &type_name),
            fallback: parsed_impl
                .field_fallback
                .as_ref()
                .map(|_| crate::naming::sidecar_fallback_c_name(&type_name)),
            fallback_names: parsed_impl
                .field_fallback
                .as_ref()
                .and_then(|fallback| fallback.names.as_ref())
                .map(|_| crate::naming::sidecar_fallback_names_c_name(&type_name)),
        }
    }

    /// The fallback's read of `name`, a name no field has, on the bare
    /// pointer `receiver` (#2150): the method returns the value the fallback
    /// has for it. The method passes its own frame (`environment()`) as the
    /// value of `None`, which the entry hands back untouched, so no value the
    /// fallback computes can be taken for it; the lines after these decide
    /// what `None` reads. Nothing without a fallback.
    fn fallback_read(&self, receiver: &str, name: &str, indent: &str) -> Vec<String> {
        let Some(entry) = &self.fallback else {
            return Vec::new();
        };
        let mut lines = vec![format!(
            "{indent}.val <- .Call({entry}, {receiver}, {name}, environment())"
        )];
        lines.extend(guard(indent));
        lines.push(format!(
            "{indent}if (!identical(.val, environment())) return(.val)"
        ));
        lines
    }

    /// The call of the impl's `fallback_names` entry (#2150) on the bare
    /// pointer `receiver`, for `.DollarNames()`; `None` without one. The
    /// entry gives `character(0)` for a pointer without its Rust value, so
    /// completion still never raises on a restored object.
    fn fallback_names_call(&self, receiver: &str) -> Option<String> {
        let entry = self.fallback_names.as_ref()?;
        Some(format!(".Call({entry}, {receiver})"))
    }

    /// The last lines of a `.DollarNames()` with a `fallback_names` (#2150):
    /// `value`, which reads the fallback's names (or a list's own), bound to
    /// `.val` and checked, then `pattern` over `known` and `.val`, each name
    /// once (a name the fallback shares with a field reads the field).
    fn dollar_names_with_fallback(value: &str, known: &str) -> Vec<String> {
        let mut lines = vec![format!("  .val <- {value}")];
        lines.extend(guard("  "));
        lines.push(format!(
            "  grep(pattern, unique(c({known}, .val)), value = TRUE)"
        ));
        lines
    }

    /// Under `restored(refuse = "every_read")` (#1901), the check a read
    /// that reaches no accessor makes first on the bare pointer `receiver`:
    /// a pointer without its Rust value raises the type's restored error. A
    /// type without the option skips it (its flag is `FALSE`).
    fn live_check(&self, receiver: &str, indent: &str) -> Vec<String> {
        let mut lines = vec![format!("{indent}if ({}) {{", self.every_read)];
        lines.push(format!("{indent}  .val <- {}({receiver}, TRUE)", self.live));
        lines.extend(guard(&format!("{indent}  ")));
        lines.push(format!("{indent}}}"));
        lines
    }

    /// The first check of every S3 reader under `restored(refuse =
    /// "every_read")` (#1901, #1919), on a bare pointer and on a list or
    /// environment receiver: a pointer without its Rust value, a `.ptr` that
    /// is one, and a list or environment that carries none (the shape an
    /// earlier release may have saved) raise the type's refusal before the
    /// index is looked at, so a position or an unknown name is refused too.
    /// An atomic receiver was never saved; its field reads refuse it.
    fn every_read_check(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "  if ({} && typeof(x) %in% c(\"externalptr\", \"list\", \"environment\")) {{",
            self.every_read
        )];
        lines.push(format!("    .val <- {}(x, TRUE)", self.live));
        lines.extend(guard("    "));
        lines.push("  }".to_string());
        lines
    }

    /// The `.DollarNames()` guard: under `restored(refuse = "every_read")` a
    /// receiver without a live Rust value completes to nothing (#1901): a
    /// restored pointer, a list or environment carrying one or none (#1919),
    /// whose `names()` would refuse. `.DollarNames()` serves completion, so
    /// it never raises.
    fn dollar_names_guard(&self, receiver: &str) -> String {
        format!(
            "  if ({every_read} && !{live}({receiver}, FALSE)) return(character(0))",
            every_read = self.every_read,
            live = self.live
        )
    }
}

/// The tagged-condition guard after an accessor call: the method's frame is
/// the raise fallback, resolved to the generic's call (#1851).
fn guard(indent: &str) -> Vec<String> {
    crate::method_return_builder::condition_check_lines_with_default(indent, "environment()")
}

/// The help topic of the field methods (and of the S3 class environment):
/// the impl block's own `@rdname`, else the class page (#1900).
pub(super) fn topic(parsed_impl: &ParsedImpl) -> String {
    crate::roxygen::rdname_value(&parsed_impl.doc_tags)
        .map_or_else(|| parsed_impl.class_name(), str::to_owned)
}

/// The page lines of a field method's block: `@rdname` its topic
/// ([`topic`]) with the `@param` lines of `params` the page needs, or, on a
/// topic the author names with the impl's `@rdname`, `@order NaN` and no
/// `@param` lines: the topic's own block documents the arguments, and a
/// later block's `@param` would replace its text (#1590).
pub(super) fn page_lines(parsed_impl: &ParsedImpl, params: &[(&str, &str)]) -> Vec<String> {
    let topic = topic(parsed_impl);
    let mut lines = vec![format!("#' @rdname {topic}")];
    if topic == parsed_impl.class_name() {
        lines.extend(param_lines(parsed_impl, params));
    } else {
        lines.push(format!("#' {}", crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS));
    }
    lines
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
/// names the constructor documents there: roxygen2 keeps the last `@param`
/// of a name on a page, and these blocks must not replace the constructor's
/// text. An env class's `Class$new()` lists its arguments in a `\describe{}`
/// list, not as `@param` lines, so it takes nothing away. (The type's
/// standalone accessors, which documented `x` / `value` on the page named
/// after the type, are internal under field syntax, #1900.)
fn param_lines(parsed_impl: &ParsedImpl, params: &[(&str, &str)]) -> Vec<String> {
    let documented: Vec<String> = parsed_impl
        .constructor()
        .filter(|_| parsed_impl.class_system != ClassSystem::Env)
        .map(|ctor| {
            crate::r_wrapper_builder::r_formal_names(&ctor.sig.inputs)
                .map(|(name, _)| name)
                .collect()
        })
        .unwrap_or_default();
    params
        .iter()
        .filter(|(name, _)| !documented.iter().any(|f| f == name))
        .map(|(name, text)| format!("#' @param {name} {text}"))
        .collect()
}

/// The roxygen block of an S3 or env field method `<generic>.<Class>`: on the
/// class's topic ([`page_lines`]) for an exported class with a page, else no
/// page. The method is registered (`S3method()`) when `register` is set.
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
        lines.extend(page_lines(parsed_impl, params));
    } else {
        lines.push("#' @noRd".to_string());
    }
    lines.push(format!("#' @method {generic} {class_name}"));
    if register {
        lines.push("#' @export".to_string());
    }
}

/// The roxygen block of a `.DollarNames.<Class>` method: on the class's
/// topic ([`page_lines`]) for an exported class with a page, else no page,
/// and registered with `@exportS3Method utils::.DollarNames` when `register`
/// is set (see the module docs for why not `@method` + `@export`).
fn dollar_names_doc(
    lines: &mut Vec<String>,
    parsed_impl: &ParsedImpl,
    on_page: bool,
    register: bool,
) {
    if on_page {
        lines.extend(page_lines(
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

    let class_lit = format!("\"{}\"", crate::naming::r_string_escape(&class_name));
    let x = ("x", "An object.");
    let dots = ("...", "Additional arguments.");
    // One text per name for the page: roxygen2 keeps the last `@param` of a
    // name on a page, so the reader's and the writer's blocks say the same.
    let fallback = h.fallback.is_some();
    let (name, i) = match parsed_impl.field_lookup {
        FieldLookup::Strict if fallback => (
            (
                "name",
                "A field name, exactly; on the pointer itself another name is read through \
                 the class's fallback, an error when it has no value for it.",
            ),
            (
                "i",
                "A field name, exactly; `[[` on the pointer itself also takes a whole-number \
                 position in the field order and reads another name through the class's \
                 fallback. On a list or an environment carrying the pointer, any other index \
                 goes to the object's own `[[` / `[[<-`, which matches names exactly.",
            ),
        ),
        FieldLookup::List if fallback => (
            (
                "name",
                "A field name, exactly; on the pointer itself another name is read through \
                 the class's fallback, `NULL` when it has no value for it.",
            ),
            (
                "i",
                "A field name, exactly; `[[` on the pointer itself also takes a position in \
                 the field order, as for a list, and reads another name through the class's \
                 fallback. On a list or an environment carrying the pointer, any other index \
                 goes to the object's own `[[` / `[[<-`, which matches names exactly.",
            ),
        ),
        FieldLookup::Strict => (
            (
                "name",
                "A field name, exactly; on the pointer itself any other name is an error.",
            ),
            (
                "i",
                "A field name, exactly; `[[` on the pointer itself also takes a whole-number \
                 position in the field order. On a list or an environment carrying the \
                 pointer, any other index goes to the object's own `[[` / `[[<-`, which \
                 matches names exactly.",
            ),
        ),
        FieldLookup::List => (
            (
                "name",
                "A field name, exactly; on the pointer itself any other name reads `NULL`.",
            ),
            (
                "i",
                "A field name, exactly; `[[` on the pointer itself also takes a position in \
                 the field order, as for a list. On a list or an environment carrying the \
                 pointer, any other index goes to the object's own `[[` / `[[<-`, which \
                 matches names exactly.",
            ),
        ),
    };
    let value = ("value", "The new value of the field.");

    // `$`: an exact field name reads the field. On a bare pointer any other
    // name raises `miniextendr_no_field`, or reads NULL under
    // `lookup = "list"`; a list receiver reads its own element by the exact
    // name (`.subset2()`: R's `$` on a list would match a prefix, #1922), an
    // environment receiver through R's `$`, which is exact there.
    s3_method_doc(&mut lines, parsed_impl, "$", on_page, register, &[x, name]);
    lines.push(format!("{} <- function(x, name) {{", def("$")));
    lines.extend(h.every_read_check());
    lines.push(format!("  if (is.na(match(name, {}))) {{", h.fields));
    // What an unknown name reads on the bare pointer, and what the
    // fallback's `None` reads (#2150).
    let unknown = |name: &str| match parsed_impl.field_lookup {
        FieldLookup::Strict => format!("{}({name}, environment(), {class_lit})", h.no_field),
        FieldLookup::List => "return(NULL)".to_string(),
    };
    if fallback {
        lines.push("    if (typeof(x) == \"externalptr\") {".to_string());
        lines.extend(h.fallback_read("x", "name", "      "));
        lines.push(format!("      {}", unknown("name")));
        lines.push("    }".to_string());
    } else {
        lines.push(format!(
            "    if (typeof(x) == \"externalptr\") {}",
            unknown("name")
        ));
    }
    lines.push("    if (is.list(x)) return(.subset2(x, name))".to_string());
    lines.push("    return(NextMethod())".to_string());
    lines.push("  }".to_string());
    lines.push(format!("  .val <- {}(x, name)", h.get));
    lines.extend(guard("  "));
    lines.push("  .val".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    // `[[`: on a bare pointer one exact name or one position. The default
    // takes a whole number only and refuses an unknown name with
    // `miniextendr_no_field`; `lookup = "list"` takes a number or a logical
    // as a list's `[[` does (`NA` reads NULL, since a character vector's
    // `[[NA]]` is `NA` where a list's is NULL; the rest through
    // `.miniextendr_field_at`) and reads NULL for an unknown name. Any other
    // index raises the bad-index error there (#1920). A list or environment
    // receiver reaches its own `[[` for everything but an exact field, with
    // `exact = TRUE` whatever the call passed (#1922).
    s3_method_doc(
        &mut lines,
        parsed_impl,
        "[[",
        on_page,
        register,
        &[x, i, dots],
    );
    lines.push(format!("{} <- function(x, i, ...) {{", def("[[")));
    lines.extend(h.every_read_check());
    lines.push("  if (typeof(x) == \"externalptr\") {".to_string());
    let field_at = format!(
        "      i <- .miniextendr_field_at({}, i, environment(), {})",
        h.fields, h.error_class
    );
    match parsed_impl.field_lookup {
        FieldLookup::List => {
            lines.push("    if (is.character(i) && length(i) == 1L) {".to_string());
            if fallback {
                // `NA` reads NULL, as without the fallback.
                lines.push(format!("      if (is.na(match(i, {}))) {{", h.fields));
                lines.push("        if (!is.na(i)) {".to_string());
                lines.extend(h.fallback_read("x", "i", "          "));
                lines.push("        }".to_string());
                lines.push("        return(NULL)".to_string());
                lines.push("      }".to_string());
            } else {
                lines.push(format!(
                    "      if (is.na(match(i, {}))) return(NULL)",
                    h.fields
                ));
            }
            lines.push(
                "    } else if (length(i) == 1L && (is.numeric(i) || is.logical(i))) {".to_string(),
            );
            lines.push("      if (is.na(i)) return(NULL)".to_string());
            lines.push(field_at);
            lines.push(format!(
                "    }} else .miniextendr_field_index_error({class_lit}, environment(), {})",
                h.error_class
            ));
        }
        FieldLookup::Strict => {
            lines.push("    if (is.character(i) && length(i) == 1L && !is.na(i)) {".to_string());
            if fallback {
                lines.push(format!("      if (is.na(match(i, {}))) {{", h.fields));
                lines.extend(h.fallback_read("x", "i", "        "));
                lines.push(format!("        {}", unknown("i")));
                lines.push("      }".to_string());
            } else {
                lines.push(format!(
                    "      if (is.na(match(i, {}))) {}",
                    h.fields,
                    unknown("i")
                ));
            }
            lines.push(
                "    } else if (is.numeric(i) && length(i) == 1L && !is.na(i) && i == trunc(i)) {"
                    .to_string(),
            );
            lines.push(field_at);
            lines.push(format!(
                "    }} else .miniextendr_field_index_error({class_lit}, environment(), {}, strict = TRUE)",
                h.error_class
            ));
        }
    }
    lines.push(format!(
        "  }} else if ({}) {{",
        not_a_field_name("i", &h.fields)
    ));
    lines.push("    return(NextMethod(exact = TRUE))".to_string());
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
    // and keeps a NULL field (`out[k] <- list(.val)`; `out[[k]] <- NULL`
    // would drop it).
    s3_method_doc(&mut lines, parsed_impl, "names", on_page, register, &[x]);
    lines.push(format!("{} <- function(x) {{", def("names")));
    lines.extend(h.every_read_check());
    lines.push("  if (typeof(x) != \"externalptr\") return(NextMethod())".to_string());
    lines.push(format!("  {}", h.fields));
    lines.push("}".to_string());
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
    lines.extend(h.every_read_check());
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
    // own names. It reads only names, so it never raises; under
    // `restored(refuse = "every_read")` a restored pointer completes to
    // nothing (#1901).
    dollar_names_doc(&mut lines, parsed_impl, on_page, register);
    lines.push(format!(
        "{} <- function(x, pattern = \"\") {{",
        def(".DollarNames")
    ));
    lines.push(h.dollar_names_guard("x"));
    match h.fallback_names_call("x") {
        // The fallback's names follow the fields on the bare pointer, where
        // `$` reads them (#2150).
        Some(call) => lines.extend(Helpers::dollar_names_with_fallback(
            &format!("if (typeof(x) == \"externalptr\") {call} else names(x)"),
            &h.fields,
        )),
        None => lines.push(format!(
            "  grep(pattern, c({}, if (typeof(x) != \"externalptr\") names(x)), value = TRUE)",
            h.fields
        )),
    }
    lines.push("}".to_string());
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
        "    if (typeof(x) == \"externalptr\") {}(name, environment(), {class_lit})",
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
        "    if (typeof(x) == \"externalptr\" && is.character(i) && length(i) == 1L) {}(i, environment(), {class_lit})",
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
    let class_lit = format!("\"{}\"", crate::naming::r_string_escape(&class_name));
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
            lines.extend(page_lines(parsed_impl, params));
        } else {
            lines.push("#' @noRd".to_string());
        }
        if export {
            lines.push(format!("#' @exportMethod {generic}"));
        }
    };
    let x = ("x", "An object.");
    let name = (
        "name",
        if h.fallback.is_some() {
            "A field name, or a name the class's fallback reads."
        } else {
            "A field name."
        },
    );
    let value = ("value", "The new value of the field.");

    doc(&mut lines, "$", &[x, name]);
    lines.push(format!(
        "methods::setMethod(\"$\", \"{class_name}\", function(x, name) {{"
    ));
    lines.push(format!("  if (is.na(match(name, {}))) {{", h.fields));
    lines.extend(h.live_check("x@ptr", "    "));
    lines.extend(h.fallback_read("x@ptr", "name", "    "));
    lines.push(format!(
        "    {}(name, environment(), {class_lit})",
        h.no_field
    ));
    lines.push("  }".to_string());
    lines.push(format!("  .val <- {}(x@ptr, name)", h.get));
    lines.extend(guard("  "));
    lines.push("  .val".to_string());
    lines.push("})".to_string());
    lines.push(String::new());

    dollar_names_doc(&mut lines, parsed_impl, on_page, !parsed_impl.noexport);
    lines.push(format!(
        "{} <- function(x, pattern = \"\") {{",
        crate::naming::r_def_name(&format!(".DollarNames.{class_name}"))
    ));
    lines.push(h.dollar_names_guard("x@ptr"));
    match h.fallback_names_call("x@ptr") {
        Some(call) => lines.extend(Helpers::dollar_names_with_fallback(&call, &h.fields)),
        None => lines.push(format!("  grep(pattern, {}, value = TRUE)", h.fields)),
    }
    lines.push("}".to_string());
    lines.push(String::new());

    if parsed_impl.r_data_accessors.setters() {
        doc(&mut lines, "$<-", &[x, name, value]);
        lines.push(format!(
            "methods::setMethod(\"$<-\", \"{class_name}\", function(x, name, value) {{"
        ));
        lines.push(format!(
            "  if (is.na(match(name, {}))) {}(name, environment(), {class_lit})",
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

/// The check in an env class's `$.Class` before it returns `NULL` for a name
/// that is neither a field nor a method: under `restored(refuse =
/// "every_read")` a pointer without its Rust value raises the type's
/// restored error there (#1901). With a `fallback` (#2150) the bare pointer
/// then asks it, after the fields and the methods, so a value of the
/// fallback never hides a method; its `None` reads `NULL`, as an unknown
/// name does.
pub(super) fn env_unknown_name_check(parsed_impl: &ParsedImpl) -> Vec<String> {
    let h = Helpers::new(parsed_impl);
    let mut lines = h.live_check("self", "    ");
    if h.fallback.is_some() {
        lines.push("    if (typeof(self) == \"externalptr\") {".to_string());
        lines.extend(h.fallback_read("self", "name", "      "));
        lines.push("    }".to_string());
    }
    lines
}

/// The `@param name` text of an env class's `$.Class`.
pub(super) fn env_name_param(parsed_impl: &ParsedImpl) -> &'static str {
    if parsed_impl.field_fallback.is_some() {
        "A method or field name, or a name the class's fallback reads."
    } else {
        "A method or field name."
    }
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
        "{} <- function(x, pattern = \"\") {{",
        crate::naming::r_def_name(&format!(".DollarNames.{class_name}"))
    ));
    lines.push(h.dollar_names_guard("x"));
    match h.fallback_names_call("x") {
        // The fallback's names follow the methods, which `$` tries first
        // (#2150).
        Some(call) => lines.extend(Helpers::dollar_names_with_fallback(
            &format!("if (typeof(x) == \"externalptr\") {call}"),
            &format!("{}, ls({class_name})", h.fields),
        )),
        None => lines.push(format!(
            "  grep(pattern, c({}, ls({})), value = TRUE)",
            h.fields, class_name
        )),
    }
    lines.push("}".to_string());
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
    let class_lit = format!("\"{}\"", crate::naming::r_string_escape(&class_name));
    let x = ("x", "An object.");
    let value = ("value", "The new value of the field.");
    s3_method_doc(&mut lines, parsed_impl, "$<-", on_page, true, &[x, value]);
    lines.push(format!("`$<-.{class_name}` <- function(x, name, value) {{"));
    lines.push(format!("  if ({}) {{", not_a_field_name("name", &h.fields)));
    lines.push(format!(
        "    if (typeof(x) == \"externalptr\" && is.character(name) && length(name) == 1L) {}(name, environment(), {class_lit})",
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

/// The `.Call` entries of the impl's field fallback (#2150): `fallback =
/// "Self::f"` as `C_<crate>__mx_rdata_fallback_<Type>(x, name, none)`, and
/// `fallback_names = "Self::g"` as `C_<crate>__mx_rdata_names_<Type>(x)`.
/// Nothing without a fallback.
///
/// They are the sidecar getters' kind (`externalptr_derive`): no call slot
/// (#344, #348), the body under `with_r_unwind_protect`, so a panic or the
/// function's own `rust_error!(class = ...)` reaches R as a tagged condition
/// the method raises on its call, and the deferred conditions signalled at
/// the call. They borrow the value through `sidecar_r_struct`, as a struct
/// field's getter does: a restored pointer raises the type's restored error
/// before the function runs, a pointer of another type the receiver error,
/// and a running method holding the value mutably
/// `miniextendr_borrow_conflict`.
///
/// `none` is the value of `None`: the method passes its own frame, which the
/// entry returns untouched, so the entry holds no R value across an
/// allocation and the method tells `None` from any value with
/// `identical()`. `Some(v)` converts `v` with `IntoR`, so `Some(None)` of an
/// `Option<Option<i32>>` reads `NA`. The names entry gives `character(0)`
/// for a pointer without its Rust value, since `.DollarNames()` never
/// raises; a live one calls the function on the value.
///
/// The paths resolve a leading `Self` to the impl's type, since the entries
/// are free functions; a function of the wrong signature is reported at the
/// attribute's string.
pub(super) fn fallback_c_entries(parsed_impl: &ParsedImpl) -> proc_macro2::TokenStream {
    use quote::{format_ident, quote, quote_spanned};

    let Some(fallback) = &parsed_impl.field_fallback else {
        return proc_macro2::TokenStream::new();
    };
    let ty = &parsed_impl.type_ident;
    let type_name = crate::naming::ident_name(ty);
    let cfg_attrs = &parsed_impl.cfg_attrs;
    let path = |lit: &syn::LitStr| {
        let mut path: syn::Path = lit
            .parse()
            .expect("the impl attribute parser checked the path");
        crate::util::resolve_self(&mut path, ty);
        path
    };

    let get = path(&fallback.get);
    let span = fallback.get.span();
    // The `Option` and the conversion are spanned at the attribute's string,
    // so a function that returns no `Option`, or an `Option` of a type
    // without `IntoR`, is reported there.
    let value = quote_spanned! {span=>
        match #get(__mx_value, &__mx_name) {
            ::core::option::Option::Some(v) => ::miniextendr_api::into_r::IntoR::into_sexp(v),
            ::core::option::Option::None => none,
        }
    };
    let c_ident = format_ident!("{}", crate::naming::sidecar_fallback_c_name(&type_name));
    let def_ident = format_ident!("__MX_CALL_DEF_RDATA_FALLBACK_{type_name}");
    let registration = crate::call_method_def::call_method_def(&def_ident, &c_ident, 3);
    let doc = format!(
        "Generated fallback of the field methods of Rust type `{type_name}` (`fallback = {:?}`).",
        fallback.get.value()
    );
    let mut entries = quote! {
        #(#cfg_attrs)*
        #[doc = #doc]
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C-unwind" fn #c_ident(
            x: ::miniextendr_api::SEXP,
            name: ::miniextendr_api::SEXP,
            none: ::miniextendr_api::SEXP,
        ) -> ::miniextendr_api::SEXP {
            let __miniextendr_deferred_mark = ::miniextendr_api::deferred_condition::mark();
            let __miniextendr_value = ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                || {
                    // The readers pass one name, a string.
                    let __mx_name: ::std::string::String =
                        match ::miniextendr_api::TryFromSexp::try_from_sexp(name) {
                            ::core::result::Result::Ok(name) => name,
                            ::core::result::Result::Err(err) => {
                                ::core::panic!("the field fallback takes one name: {err}")
                            }
                        };
                    let __mx_data =
                        unsafe { ::miniextendr_api::externalptr::sidecar_r_struct::<#ty>(x) };
                    let __mx_value: &#ty = &__mx_data;
                    #value
                },
                ::core::option::Option::None,
            );
            unsafe {
                ::miniextendr_api::deferred_condition::finish(
                    __miniextendr_deferred_mark,
                    __miniextendr_value,
                    ::core::option::Option::None,
                )
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        #registration
    };

    if let Some(names) = &fallback.names {
        let names_path = path(names);
        let span = names.span();
        let value = quote_spanned! {span=>
            let __mx_names: ::std::vec::Vec<::std::string::String> = #names_path(__mx_value);
        };
        let c_ident = format_ident!(
            "{}",
            crate::naming::sidecar_fallback_names_c_name(&type_name)
        );
        let def_ident = format_ident!("__MX_CALL_DEF_RDATA_NAMES_{type_name}");
        let registration = crate::call_method_def::call_method_def(&def_ident, &c_ident, 1);
        let doc = format!(
            "Generated completions of the field fallback of Rust type `{type_name}` \
             (`fallback_names = {:?}`).",
            names.value()
        );
        entries.extend(quote! {
            #(#cfg_attrs)*
            #[doc = #doc]
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            #[allow(non_snake_case)]
            pub unsafe extern "C-unwind" fn #c_ident(
                x: ::miniextendr_api::SEXP,
            ) -> ::miniextendr_api::SEXP {
                let __miniextendr_deferred_mark = ::miniextendr_api::deferred_condition::mark();
                let __miniextendr_value = ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                    || {
                        use ::miniextendr_api::into_r::IntoR;
                        if !unsafe { ::miniextendr_api::externalptr::sidecar_r_live::<#ty>(x, false) } {
                            return ::std::vec::Vec::<::std::string::String>::new().into_sexp();
                        }
                        let __mx_data =
                            unsafe { ::miniextendr_api::externalptr::sidecar_r_struct::<#ty>(x) };
                        let __mx_value: &#ty = &__mx_data;
                        #value
                        __mx_names.into_sexp()
                    },
                    ::core::option::Option::None,
                );
                unsafe {
                    ::miniextendr_api::deferred_condition::finish(
                        __miniextendr_deferred_mark,
                        __miniextendr_value,
                        ::core::option::Option::None,
                    )
                }
            }

            #(#cfg_attrs)*
            #[doc(hidden)]
            #registration
        });
    }
    entries
}
