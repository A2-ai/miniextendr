//! Write-time class resolution for fieldwise comparisons.

/// One field compared by R's own `all.equal` method.
pub struct AllEqualField {
    /// Field name, used by the default Component diagnostic.
    pub name: &'static str,
    /// An explicit diagnostic prefix.
    pub label: Option<&'static str>,
    /// Fixed field overrides: tolerance, scale, check.attributes, check.names.
    pub settings: [Option<&'static str>; 4],
    /// The conversion's Rust value type; aliases resolve to their real type.
    pub type_path: fn() -> &'static str,
}

/// A fieldwise comparison registered by `#[derive(AllEqual)]`.
pub struct AllEqualEntry {
    /// Canonical identity, matching the inherent class's type_path.
    pub type_path: fn() -> &'static str,
    /// The derive's type spelling, for diagnostics.
    pub rust_type: &'static str,
    /// Native callback returning the converted fields as a protected list.
    pub c_name: &'static str,
    /// Formal defaults: tolerance, scale, check.attributes, check.names.
    pub defaults: [Option<&'static str>; 4],
    /// Unskipped fields in source order.
    pub fields: &'static [AllEqualField],
    /// Derive source file.
    pub source_file: &'static str,
    /// Derive source line.
    pub source_line: u32,
}

/// Infer a getter's return type without invoking it. Instantiating its input
/// lifetime at 'static also accommodates getters returning borrowed values.
#[doc(hidden)]
pub fn all_equal_getter_type<T: 'static, R>(_: fn(&'static T) -> R) -> &'static str {
    std::any::type_name::<R>()
}

#[cfg(not(target_arch = "wasm32"))]
use super::{MergedClass, RWrapperPriority, WrapperFragment};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::{HashMap, HashSet};

#[cfg(not(target_arch = "wasm32"))]
const SETTINGS: [&str; 4] = ["tolerance", "scale", "check.attributes", "check.names"];

#[cfg(not(target_arch = "wasm32"))]
fn literal(text: &str) -> String {
    format!("\"{}\"", crate::match_arg::escape_r_string(text))
}

/// Wrap a converted nested pointer in its registered R class. Matching uses
/// canonical Rust identities, so aliases and cross-module fields work too.
/// ExternalPtr's generic spelling is obtained from the same compiler, not
/// from a hard-coded crate/module path.
#[cfg(not(target_arch = "wasm32"))]
fn wrap_field(ty: &str, expr: &str, classes: &HashMap<&'static str, MergedClass>) -> String {
    let external = std::any::type_name::<crate::ExternalPtr<()>>();
    let prefix = external
        .strip_suffix("()>")
        .expect("ExternalPtr<()> type spelling");
    let ty = ty
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(ty);
    let Some(class) = classes.get(ty) else {
        return expr.to_string();
    };
    super::wrap_class_value(class.class_system, class.r_class_name, expr)
}

#[cfg(not(target_arch = "wasm32"))]
fn guard(lines: &mut Vec<String>) {
    lines.push("  if (base::inherits(.val, \"rust_condition_value\") && base::isTRUE(base::attr(.val, \"__rust_condition__\"))) return(.miniextendr_raise_condition(.val, environment()))".into());
}

#[cfg(not(target_arch = "wasm32"))]
fn render(
    entry: &AllEqualEntry,
    class: &MergedClass,
    classes: &HashMap<&'static str, MergedClass>,
) -> String {
    let name = class.r_class_name;
    let mut lines = vec![format!(
        "# Generated from Rust derive AllEqual on `{}` ({}:{})",
        entry.rust_type, entry.source_file, entry.source_line
    )];
    if class.documented {
        if class.comparison_needs_title {
            lines.push(format!("#' @title Compare {name} objects"));
        }
        if class.comparison_internal {
            lines.push("#' @keywords internal".into());
        }
        lines.extend([
            format!("#' @rdname {}", class.topic),
            "#' @param target,current Objects to compare.".into(),
            "#' @param ... Additional arguments passed to each field's all.equal method.".into(),
            "#' @param tolerance,scale,check.attributes,check.names Comparison settings. Field overrides take precedence over supplied arguments and struct defaults; unset settings preserve each field method's default.".into(),
            "#' @return TRUE when every compared field is equal; otherwise all differences, prefixed by field name or label.".into(),
        ]);
    } else {
        lines.push("#' @noRd".into());
    }
    let mut formals = vec![
        "target".to_string(),
        "current".to_string(),
        "...".to_string(),
    ];
    for (setting, default) in SETTINGS.iter().zip(entry.defaults) {
        formals.push(default.map_or_else(
            || setting.to_string(),
            |value| format!("{setting} = {value}"),
        ));
    }
    let formals = formals.join(", ");
    match class.class_system {
        "s4" => {
            lines.push("#' @exportMethod all.equal".into());
            lines.push(format!(
                "methods::setMethod(\"all.equal\", {}, function({formals}) {{",
                literal(name)
            ));
        }
        "s7" => {
            if class.documented {
                lines.push(format!("#' @name {name}-all.equal"));
                lines.push(format!("#' @usage all.equal({formals})"));
            }
            // Replacement assignment to base::all.equal is invalid R, and a
            // bare assignment would replace a package's S4 generic binding.
            // The public replacement function registers the same S7 method.
            lines.push(format!(
                "S7::`method<-`(base::all.equal, {name}, value = function({formals}) {{"
            ));
        }
        _ => {
            lines.push(format!("#' @method all.equal {name}"));
            lines.push("#' @export".into());
            lines.push(format!("`all.equal.{name}` <- function({formals}) {{"));
        }
    }
    lines.extend([
        "  if (!base::identical(base::class(target), base::class(current)))".into(),
        "    return(base::sprintf(\"target is %s, current is %s\", base::data.class(target), base::data.class(current)))".into(),
        format!("  .val <- .Call({}, target)", entry.c_name),
    ]);
    guard(&mut lines);
    lines.push("  .target <- .val".into());
    lines.push(format!("  .val <- .Call({}, current)", entry.c_name));
    guard(&mut lines);
    lines.push("  .current <- .val".into());
    // R's promoted S4 generic delegates ordinary values to base::all.equal;
    // calling the base closure directly would skip a nested S4 method.
    lines.push("  .compare <- methods::getGeneric(\"all.equal\")".into());
    lines.push("  if (base::is.null(.compare)) .compare <- base::all.equal".into());
    lines.push("  .args <- base::list(...)".into());
    for (setting, default) in SETTINGS.iter().zip(entry.defaults) {
        let assignment = format!(".args[{}] <- base::list({setting})", literal(setting));
        lines.push(if default.is_some() {
            format!("  {assignment}")
        } else {
            format!("  if (!missing({setting})) {assignment}")
        });
    }
    lines.push("  .out <- base::character()".into());
    for (i, field) in entry.fields.iter().enumerate() {
        lines.push("  .field_args <- .args".into());
        for (setting, value) in SETTINGS.iter().zip(field.settings) {
            if let Some(value) = value {
                lines.push(format!(
                    "  .field_args[{}] <- base::list({value})",
                    literal(setting)
                ));
            }
        }
        let target = wrap_field(
            (field.type_path)(),
            &format!(".target[[{}L]]", i + 1),
            classes,
        );
        let current = wrap_field(
            (field.type_path)(),
            &format!(".current[[{}L]]", i + 1),
            classes,
        );
        lines.push(format!("  .diff <- base::do.call(.compare, base::c(base::list(target = {target}, current = {current}), .field_args), quote = TRUE)"));
        let prefix = field.label.map_or_else(
            || {
                if field.name.bytes().all(|byte| byte.is_ascii_digit()) {
                    literal(&format!("Component {}: ", field.name))
                } else {
                    format!(
                        "base::paste0(\"Component \", base::dQuote({}), \": \")",
                        literal(field.name)
                    )
                }
            },
            |label| literal(&format!("{label}: ")),
        );
        lines.push(format!(
            "  if (!base::isTRUE(.diff)) .out <- base::c(.out, base::paste0({prefix}, .diff))"
        ));
    }
    lines.push("  if (base::length(.out)) .out else TRUE".into());
    lines.push(
        if matches!(class.class_system, "s4" | "s7") {
            "})"
        } else {
            "}"
        }
        .into(),
    );
    lines.push(String::new());
    lines.join("\n")
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn fragments(
    entries: &[AllEqualEntry],
    classes: &HashMap<&'static str, MergedClass>,
) -> Result<Vec<WrapperFragment>, String> {
    let mut entries: Vec<_> = entries.iter().collect();
    entries.sort_by_key(|entry| (entry.source_file, entry.source_line, entry.rust_type));
    let mut seen = HashSet::new();
    let mut errors = Vec::new();
    let mut fragments = Vec::new();
    for entry in entries {
        let ty = (entry.type_path)();
        if !seen.insert(ty) {
            errors.push(format!(
                "miniextendr: more than one all.equal provider for `{ty}`"
            ));
            continue;
        }
        let Some(class) = classes.get(ty) else {
            errors.push(format!("miniextendr: #[derive(AllEqual)] on `{}` ({}:{}) requires an inherent #[miniextendr] class impl", entry.rust_type, entry.source_file, entry.source_line));
            continue;
        };
        fragments.push(WrapperFragment {
            priority: RWrapperPriority::TraitImpl,
            source_file: entry.source_file,
            source_line: entry.source_line,
            content: render(entry, class, classes).into(),
        });
    }
    if errors.is_empty() {
        Ok(fragments)
    } else {
        Err(errors.join("\n"))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    fn entry() -> AllEqualEntry {
        AllEqualEntry {
            type_path: || "Fit",
            rust_type: "Fit",
            c_name: "C_Fit",
            defaults: [Some("1e-6"), None, None, None],
            fields: &[AllEqualField {
                name: "estimate",
                label: Some("Estimate"),
                settings: [Some("0"), None, None, None],
                type_path: || "f64",
            }],
            source_file: "fit.rs",
            source_line: 1,
        }
    }
    fn class(system: &'static str) -> MergedClass {
        MergedClass {
            r_class_name: "renamed_fit",
            documented: true,
            comparison_needs_title: false,
            comparison_internal: false,
            topic: "fit_topic".into(),
            class_system: system,
            origin: "fit.rs:2".into(),
        }
    }
    #[test]
    fn resolves_dispatch_and_settings_for_every_class_system() {
        for system in ["env", "r6", "s3", "s4", "s7", "vctrs"] {
            let classes = HashMap::from([("Fit", class(system))]);
            let out = fragments(&[entry()], &classes)
                .unwrap()
                .pop()
                .unwrap()
                .content;
            assert!(
                out.contains("tolerance = 1e-6, scale, check.attributes, check.names"),
                "{out}"
            );
            assert!(out.contains("if (!missing(scale)) .args[\"scale\"] <- base::list(scale)"));
            assert!(out.contains(".field_args[\"tolerance\"] <- base::list(0)"));
            assert!(out.contains("#' @rdname fit_topic"));
            assert!(
                out.contains(match system {
                    "s4" => "methods::setMethod(\"all.equal\", \"renamed_fit\"",
                    "s7" => "S7::`method<-`(base::all.equal, renamed_fit,",
                    _ => "`all.equal.renamed_fit`",
                }),
                "{out}"
            );
        }
    }
    #[test]
    fn missing_classes_and_duplicate_providers_are_errors() {
        assert!(
            fragments(&[entry()], &HashMap::new())
                .err()
                .unwrap()
                .contains("requires an inherent")
        );
        let classes = HashMap::from([("Fit", class("s3"))]);
        assert!(
            fragments(&[entry(), entry()], &classes)
                .err()
                .unwrap()
                .contains("more than one")
        );
    }
    #[test]
    fn comparison_docs_respect_class_pages() {
        let entry = entry();
        let mut class = class("s3");
        class.comparison_needs_title = true;
        class.comparison_internal = true;
        let rendered = render(&entry, &class, &HashMap::new());
        assert!(rendered.contains("#' @title Compare renamed_fit objects"));
        assert!(rendered.contains("#' @keywords internal"));
        class.documented = false;
        let rendered = render(&entry, &class, &HashMap::new());
        assert!(rendered.contains("#' @noRd"));
        assert!(!rendered.contains("#' @title"));
        assert!(!rendered.contains("#' @rdname"));
        assert!(rendered.contains("#' @export"));
    }

    #[test]
    fn pointer_type_aliases_resolve_for_nested_dispatch() {
        type Handle = crate::ExternalPtr<()>;
        let classes = HashMap::from([(std::any::type_name::<()>(), class("s4"))]);
        assert_eq!(
            wrap_field(std::any::type_name::<Handle>(), ".x", &classes),
            "methods::new(\"renamed_fit\", ptr = .x)"
        );
    }
}
