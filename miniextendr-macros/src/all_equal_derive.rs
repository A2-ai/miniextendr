//! # `#[derive(AllEqual)]` - `all.equal()` for a pointer-backed class (#2149)
//!
//! `all.equal()` on two objects of a class backed by an external pointer
//! returns `TRUE` whatever they hold: `all.equal.default()` compares only
//! `data.class()` for an external pointer, and an env-based (R6) or S4 object
//! leads base R to the pointer, which it again can't look into. The derive
//! gives the type's class an `all.equal()` method that compares the struct
//! field by field, each with base `all.equal()` on the field's R value, so
//! each field dispatches on its own class: numbers keep R's tolerance, lists
//! and data frames use base's methods, and a field holding another pointer
//! class (`T`, `ExternalPtr<T>`, `Option<ExternalPtr<T>>`, or a `Vec` of `T`
//! or `ExternalPtr<T>`) uses `T`'s method.
//!
//! ## What the derive emits
//!
//! - One `.Call` entry, `C_<crate>__mx_all_equal_<Type>(x, i)`, of the
//!   sidecar accessors' kind (no call slot): it returns compared field `i`
//!   (1-based) of the Rust value `x` carries, `x` being any receiver
//!   `sidecar_r_struct` reads (a pointer, or an R6 / S4 / S7 object, a list
//!   or an environment holding one in `.ptr`). Reading the value refuses a
//!   restored object with the type's `miniextendr_restored_*` error. A
//!   field's R value is
//!   - by default a clone converted with `IntoR` (`Clone + IntoR`);
//!   - `#[all_equal(with = "Self::f")]`: the getter's value, a
//!     `fn(&Self) -> impl IntoR`;
//!   - a `Computed` field (`#[r_data(get = "Self::f")]`): its getter's value;
//!   - a `Sidecar<T>` field: its slot in the pointer's protection list;
//!   - an `ExternalPtr<T>` or `Option<ExternalPtr<T>>` field: the pointer
//!     itself (no copy).
//!
//!   The `RSidecar` selector holds nothing and is never compared. Each call
//!   returns one value, so no R value is held across an allocation.
//! - The R value of a field whose type names a pointer class is a bare
//!   external pointer (or a list of them), which base `all.equal()` would
//!   find equal to any other. The method gives it the class's R class, as a
//!   method returning that type does (`.__MX_WRAP_RETURN_T__` and
//!   `.__MX_WRAP_LIST_RETURN_T__`, resolved when the wrappers are written),
//!   so the comparison dispatches to the class's own `all.equal()`. The
//!   field types are read as the method return types are
//!   (`ParsedMethod::inner_class_ident`), so `T`, `ExternalPtr<T>`,
//!   `Option<ExternalPtr<T>>` and a `Vec` of `T` or `ExternalPtr<T>` are
//!   wrapped (`Option<T>` of a class `T` is not `IntoR`, so it does not
//!   compile); a `T` that is not a class is left as it is. A getter's type (`with`, `Computed`) is
//!   not known to the derive, so a getter returning a pointer gives a bare
//!   pointer (#2191).
//! - A `registry::AllEqualEntry` with the method's R source and its page.
//!   The class system and R class name are on the type's impl block, a
//!   separate expansion, so the wrapper writer names and registers the
//!   method (`registry::all_equal_fragments`).
//!
//! ## The method
//!
//! `all.equal.<Class>(target, current, tolerance, scale, ..., check.attributes,
//! check.names)`, in the argument order of `all.equal.numeric()`, so a third
//! positional argument is the tolerance as it is for numbers:
//!
//! - A `current` whose `data.class()` differs from `target`'s gives base's
//!   one string, `"target is <class>, current is <class>"`, before any field
//!   is read.
//! - Each compared field is compared with
//!   `all.equal(target_field, current_field, <settings>, ...)`. A setting is
//!   passed only when something sets it, so a field's own method keeps its
//!   default otherwise: the field's `#[all_equal(...)]` fixes it, else the
//!   call gives it, else the struct's `#[all_equal(...)]` is the formal's
//!   default. The settings are formals, so a call's `check.attributes` can't
//!   also reach a field through `...`.
//! - The result is `TRUE`, or every field's messages in field order, each
//!   with `all.equal.list()`'s prefix (`Component “estimates”: `, or
//!   `Component 1: ` for a tuple field) or the field's
//!   `#[all_equal(label = "...")]` (`Estimates: `).

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, LitStr};

use crate::externalptr_derive::{SlotKind, is_rsidecar_type, parse_sidecar_info};
use crate::miniextendr_impl::{ClassSystem, ParsedMethod};

// region: settings

/// An argument of base `all.equal()` the struct or a field can set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Setting {
    Tolerance,
    Scale,
    CheckAttributes,
    CheckNames,
}

impl Setting {
    /// Every setting, in the method's formal order.
    const ALL: [Self; 4] = [
        Self::Tolerance,
        Self::Scale,
        Self::CheckAttributes,
        Self::CheckNames,
    ];

    /// The name in `#[all_equal(...)]`.
    fn rust_name(self) -> &'static str {
        match self {
            Self::Tolerance => "tolerance",
            Self::Scale => "scale",
            Self::CheckAttributes => "check_attributes",
            Self::CheckNames => "check_names",
        }
    }

    /// The method's formal, and the argument of `all.equal()` it fills.
    fn r_name(self) -> &'static str {
        match self {
            Self::Tolerance => "tolerance",
            Self::Scale => "scale",
            Self::CheckAttributes => "check.attributes",
            Self::CheckNames => "check.names",
        }
    }

    /// Whether the formal comes before `...`, as in `all.equal.numeric()`.
    fn before_dots(self) -> bool {
        matches!(self, Self::Tolerance | Self::Scale)
    }

    fn from_path(path: &syn::Path) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|setting| path.is_ident(setting.rust_name()))
    }

    /// The value of `name = value` as R source: a number for `tolerance`
    /// (not negative) and `scale` (positive), `TRUE` / `FALSE` for the checks.
    fn parse_value(self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
        let expr: syn::Expr = meta.value()?.parse()?;
        let lit = match &expr {
            syn::Expr::Lit(lit) => Some(&lit.lit),
            _ => None,
        };
        match self {
            Self::Tolerance | Self::Scale => {
                let number = match lit {
                    Some(syn::Lit::Float(lit)) => lit.base10_parse::<f64>().ok(),
                    Some(syn::Lit::Int(lit)) => lit.base10_parse::<f64>().ok(),
                    _ => None,
                };
                let (kind, example) = if self == Self::Scale {
                    ("a positive", "scale = 1")
                } else {
                    ("a non-negative", "tolerance = 1e-6")
                };
                match number {
                    Some(n) if n.is_finite() && (n > 0.0 || (n == 0.0 && self != Self::Scale)) => {
                        Ok(r_number(n))
                    }
                    _ => Err(syn::Error::new_spanned(
                        &expr,
                        format!(
                            "`{}` takes {kind} number, e.g. `{example}`",
                            self.rust_name()
                        ),
                    )),
                }
            }
            Self::CheckAttributes | Self::CheckNames => match lit {
                Some(syn::Lit::Bool(lit)) => {
                    Ok(if lit.value { "TRUE" } else { "FALSE" }.to_owned())
                }
                _ => Err(syn::Error::new_spanned(
                    &expr,
                    format!("`{}` takes `true` or `false`", self.rust_name()),
                )),
            },
        }
    }
}

/// A finite number as R source: a whole number without a fraction
/// (`0`, `2`), any other in Rust's shortest form (`1e-6`, `0.5`).
fn r_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{n:.0}")
    } else {
        format!("{n:?}")
    }
}

/// The settings given, with their values as R source, in source order.
type Settings = Vec<(Setting, String)>;

/// Adds the setting `meta` names to `settings`, refusing a second one.
fn push_setting(
    settings: &mut Settings,
    setting: Setting,
    meta: &syn::meta::ParseNestedMeta,
) -> syn::Result<()> {
    if settings.iter().any(|(given, _)| *given == setting) {
        return Err(meta.error(format!("`{}` is given twice", setting.rust_name())));
    }
    let value = setting.parse_value(meta)?;
    settings.push((setting, value));
    Ok(())
}

/// The value `settings` gives `setting`.
fn setting_value(settings: &Settings, setting: Setting) -> Option<&str> {
    settings
        .iter()
        .find(|(given, _)| *given == setting)
        .map(|(_, value)| value.as_str())
}

/// The struct's `#[all_equal(...)]`: the defaults of the method's formals.
fn parse_struct_settings(input: &DeriveInput) -> syn::Result<Settings> {
    let mut settings = Settings::new();
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("all_equal"))
    {
        attr.parse_nested_meta(|meta| {
            if let Some(setting) = Setting::from_path(&meta.path) {
                return push_setting(&mut settings, setting, &meta);
            }
            let key = path_name(&meta.path);
            if ["skip", "label", "with"].contains(&key.as_str()) {
                Err(meta.error(format!(
                    "`{key}` applies to a field; the struct's `#[all_equal(...)]` takes \
                     `tolerance`, `scale`, `check_attributes` and `check_names`"
                )))
            } else {
                Err(meta.error(format!(
                    "unknown `all_equal` option `{key}`; the struct's `#[all_equal(...)]` takes \
                     `tolerance`, `scale`, `check_attributes` and `check_names`"
                )))
            }
        })?;
    }
    Ok(settings)
}

fn path_name(path: &syn::Path) -> String {
    quote!(#path).to_string().replace(' ', "")
}

// endregion

// region: fields

/// A field's `#[all_equal(...)]`.
#[derive(Default)]
struct FieldOptions {
    /// `skip`: the field is not compared.
    skip: Option<Span>,
    /// `label = "..."`: the prefix of the field's messages, before `": "`.
    label: Option<LitStr>,
    /// `with = "Self::f"`: the getter whose value is compared.
    with: Option<LitStr>,
    /// The settings that field fixes.
    settings: Settings,
}

fn parse_field_options(field: &syn::Field) -> syn::Result<FieldOptions> {
    let mut options = FieldOptions::default();
    // Where the first option other than `skip` is given.
    let mut other: Option<Span> = None;
    for attr in field
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("all_equal"))
    {
        attr.parse_nested_meta(|meta| {
            let key = path_name(&meta.path);
            let twice = || meta.error(format!("`{key}` is given twice"));
            if key == "skip" {
                if options.skip.is_some() {
                    return Err(twice());
                }
                options.skip = Some(meta.path.span());
                return Ok(());
            }
            other.get_or_insert(meta.path.span());
            match key.as_str() {
                "label" => {
                    if options.label.is_some() {
                        return Err(twice());
                    }
                    options.label = Some(meta.value()?.parse()?);
                }
                "with" => {
                    if options.with.is_some() {
                        return Err(twice());
                    }
                    options.with = Some(meta.value()?.parse()?);
                }
                _ => match Setting::from_path(&meta.path) {
                    Some(setting) => push_setting(&mut options.settings, setting, &meta)?,
                    None => {
                        return Err(meta.error(format!(
                            "unknown `all_equal` option `{key}`; a field takes `skip`, \
                             `label = \"...\"`, `with = \"Self::f\"`, `tolerance`, `scale`, \
                             `check_attributes` and `check_names`"
                        )));
                    }
                },
            }
            Ok(())
        })?;
    }
    if let (Some(skip), Some(_)) = (options.skip, other) {
        return Err(syn::Error::new(
            skip,
            "`skip` leaves the field out of the comparison, so it takes no other option",
        ));
    }
    Ok(options)
}

/// Where a compared field's R value comes from (see the module docs).
enum FieldValue {
    /// A clone of the field, converted with `IntoR`.
    Converted,
    /// A getter's value: `with = "..."`, or a `Computed` field's `get`. The
    /// path has its leading `Self` replaced; the span is the attribute's.
    Getter(syn::Path, Span),
    /// A `Sidecar<T>` field: this slot of the pointer's protection list.
    Sidecar(usize),
    /// An `ExternalPtr<T>` field (`optional`: `Option<ExternalPtr<T>>`): the
    /// pointer itself.
    Pointer { optional: bool },
}

/// The class the method gives a field's R value: the R class of the Rust
/// type `class`, on the value (`list`: on each element of the list), unless
/// the value is `NULL` (`optional`: an `Option` field's `None`).
struct ClassWrap {
    class: String,
    list: bool,
    optional: bool,
}

impl ClassWrap {
    /// The wrap marker for `.val`, which the wrappers writer resolves.
    fn r_expression(&self) -> String {
        let class = &self.class;
        let wrapped = if self.list {
            format!(".__MX_WRAP_LIST_RETURN_{class}__(.val)")
        } else {
            format!(".__MX_WRAP_RETURN_{class}__(.val)")
        };
        if self.optional {
            format!("if (is.null(.val)) NULL else {wrapped}")
        } else {
            wrapped
        }
    }
}

/// A field the method compares.
struct ComparedField {
    member: syn::Member,
    ty: syn::Type,
    /// The field's R name: `#[r_data(name = "...")]`, else the identifier,
    /// or the 1-based position of a tuple field.
    name: String,
    label: Option<String>,
    settings: Settings,
    value: FieldValue,
    /// The class of a field whose type names a pointer class.
    wrap: Option<ClassWrap>,
}

impl ComparedField {
    /// The R expression of the prefix of the field's messages.
    fn prefix(&self) -> String {
        match (&self.label, &self.member) {
            (Some(label), _) => format!("\"{}: \"", crate::naming::r_string_escape(label)),
            (None, syn::Member::Named(_)) => format!(
                "paste0(\"Component \", dQuote(\"{}\"), \": \")",
                crate::naming::r_string_escape(&self.name)
            ),
            (None, syn::Member::Unnamed(_)) => format!("\"Component {}: \"", self.name),
        }
    }
}

/// The class of a field of type `ty` when it names a pointer class, read as a
/// method's return type is: `T` or `ExternalPtr<T>` (`T` a capitalized path
/// that is not a builtin), in `Option<..>`, `Vec<..>` or `Option<Vec<..>>`.
/// Whether `T` is a class is known when the wrappers are written; a `T` that
/// is not keeps the value as it is.
fn class_wrap(ty: &syn::Type) -> Option<ClassWrap> {
    let (inner, optional) = match option_inner(ty) {
        Some(inner) => (inner, true),
        None => (ty, false),
    };
    let (class, list) = match ParsedMethod::vec_inner_class_ident(inner) {
        Some(class) => (class, true),
        None => (ParsedMethod::inner_class_ident(inner)?, false),
    };
    Some(ClassWrap {
        class: class.to_string(),
        list,
        optional,
    })
}

/// `T` of `Option<T>`.
fn option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    crate::first_type_argument(segment)
}

/// Whether `ty` is `ExternalPtr<T>`.
fn is_external_ptr(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path)
        if path.path.segments.last().is_some_and(|segment| segment.ident == "ExternalPtr"))
}

/// A getter path from `with = "..."`, its leading `Self` replaced by the
/// struct's name: the `.Call` entry is a free function, outside any impl.
fn getter_path(lit: &LitStr, struct_name: &syn::Ident) -> syn::Result<syn::Path> {
    let mut path: syn::Path = lit.parse().map_err(|err| {
        syn::Error::new_spanned(
            lit,
            format!("`with` must be a path to a getter function, e.g. `Self::f`: {err}"),
        )
    })?;
    if let Some(first) = path.segments.first_mut()
        && first.ident == "Self"
    {
        first.ident = syn::Ident::new(&struct_name.to_string(), first.ident.span());
    }
    Ok(path)
}

/// The fields the method compares, in declaration order.
fn compared_fields(input: &DeriveInput) -> syn::Result<Vec<ComparedField>> {
    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`AllEqual` derives on a struct: it compares the struct's fields",
        ));
    };
    // The `#[r_data]` fields; `#[derive(ExternalPtr)]` reports what is wrong
    // with them, so a struct it refuses compares no sidecar field here.
    let slots = parse_sidecar_info(input, ClassSystem::Env)
        .map(|info| info.slots)
        .unwrap_or_default();
    let mut fields = Vec::new();
    for (i, field) in data.fields.iter().enumerate() {
        let options = parse_field_options(field)?;
        if options.skip.is_some() || is_rsidecar_type(field) {
            continue;
        }
        let member = match &field.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index::from(i)),
        };
        let slot = field
            .ident
            .as_ref()
            .and_then(|ident| slots.iter().find(|slot| slot.name == *ident));
        let name = match (slot, &field.ident) {
            (Some(slot), _) => slot.r_name.clone(),
            (None, Some(ident)) => crate::naming::ident_name(ident),
            (None, None) => (i + 1).to_string(),
        };
        let (value, wrap) = if let Some(with) = &options.with {
            let getter = getter_path(with, &input.ident)?;
            (FieldValue::Getter(getter, with.span()), None)
        } else if let Some(slot) = slot
            && let SlotKind::Sidecar(index) = slot.kind
        {
            (FieldValue::Sidecar(index), None)
        } else if let Some(slot) = slot
            && let SlotKind::Computed(path) = &slot.kind
        {
            (FieldValue::Getter(path.clone(), field.ty.span()), None)
        } else {
            let wrap = class_wrap(&field.ty);
            let optional = option_inner(&field.ty);
            let value = match &wrap {
                // The pointer itself, not a clone of its value.
                Some(ClassWrap { list: false, .. })
                    if is_external_ptr(optional.unwrap_or(&field.ty)) =>
                {
                    FieldValue::Pointer {
                        optional: optional.is_some(),
                    }
                }
                _ => FieldValue::Converted,
            };
            (value, wrap)
        };
        fields.push(ComparedField {
            member,
            ty: field.ty.clone(),
            name,
            label: options.label.as_ref().map(LitStr::value),
            settings: options.settings,
            value,
            wrap,
        });
    }
    if fields.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`#[derive(AllEqual)]` has no field to compare: every field is skipped \
             (`#[all_equal(skip)]`, or the `RSidecar` selector)",
        ));
    }
    Ok(fields)
}

// endregion

// region: the R method

/// The method's R source, `function(target, current, ...) { ... }`, reading
/// the fields through the `.Call` entry `c_name`.
fn r_method(c_name: &str, struct_settings: &Settings, fields: &[ComparedField]) -> String {
    let formal = |setting: Setting| match setting_value(struct_settings, setting) {
        Some(value) => format!("{} = {value}", setting.r_name()),
        None => setting.r_name().to_owned(),
    };
    let mut formals = vec!["target".to_owned(), "current".to_owned()];
    formals.extend(
        Setting::ALL
            .into_iter()
            .filter(|s| s.before_dots())
            .map(formal),
    );
    formals.push("...".to_owned());
    formals.extend(
        Setting::ALL
            .into_iter()
            .filter(|s| !s.before_dots())
            .map(formal),
    );

    let guard =
        crate::method_return_builder::condition_check_lines_with_default("  ", "environment()");
    let mut lines = vec![
        format!("function({}) {{", formals.join(", ")),
        "  if (data.class(target) != data.class(current)) {".to_owned(),
        "    return(gettextf(\"target is %s, current is %s\", data.class(target), \
         data.class(current), domain = \"R-base\"))"
            .to_owned(),
        "  }".to_owned(),
        "  .settings <- list()".to_owned(),
    ];
    for setting in Setting::ALL {
        let r = setting.r_name();
        let set = format!(".settings[\"{r}\"] <- list({r})");
        lines.push(if setting_value(struct_settings, setting).is_some() {
            format!("  {set}")
        } else {
            format!("  if (!missing({r})) {set}")
        });
    }
    lines.push("  .dots <- list(...)".to_owned());
    lines.push("  .msg <- character(0)".to_owned());
    for (i, field) in fields.iter().enumerate() {
        let index = i + 1;
        lines.push(format!("  # `{}`", field.name));
        for (receiver, var) in [("target", ".target"), ("current", ".current")] {
            lines.push(format!("  .val <- .Call({c_name}, {receiver}, {index}L)"));
            lines.extend(guard.iter().cloned());
            let value = field.wrap.as_ref().map(ClassWrap::r_expression);
            lines.push(format!("  {var} <- {}", value.as_deref().unwrap_or(".val")));
        }
        let args = if field.settings.is_empty() {
            ".settings"
        } else {
            lines.push("  .args <- .settings".to_owned());
            for setting in Setting::ALL {
                if let Some(value) = setting_value(&field.settings, setting) {
                    lines.push(format!(
                        "  .args[\"{}\"] <- list({value})",
                        setting.r_name()
                    ));
                }
            }
            ".args"
        };
        lines.push(format!(
            "  .val <- do.call(all.equal, c(list(.target, .current), {args}, .dots))"
        ));
        lines.push(format!(
            "  if (is.character(.val)) .msg <- c(.msg, paste0({}, .val))",
            field.prefix()
        ));
    }
    lines.push("  if (length(.msg)) .msg else TRUE".to_owned());
    lines.push("}".to_owned());
    lines.join("\n")
}

/// The roxygen lines of the method's page after its title.
fn r_docs(type_name: &str, struct_settings: &Settings, fields: &[ComparedField]) -> String {
    let listed: Vec<String> = fields
        .iter()
        .map(|field| {
            let fixed: Vec<String> = Setting::ALL
                .into_iter()
                .filter_map(|setting| {
                    setting_value(&field.settings, setting)
                        .map(|value| format!("`{} = {value}`", setting.r_name()))
                })
                .collect();
            if fixed.is_empty() {
                format!("`{}`", field.name)
            } else {
                format!("`{}` ({})", field.name, fixed.join(", "))
            }
        })
        .collect();
    let listed = match &listed[..] {
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
        [] => unreachable!("a derive compares at least one field"),
    };
    let defaults: Vec<String> = Setting::ALL
        .into_iter()
        .filter_map(|setting| {
            setting_value(struct_settings, setting)
                .map(|value| format!("`{} = {value}`", setting.r_name()))
        })
        .collect();
    let defaults = if defaults.is_empty() {
        String::new()
    } else {
        format!(
            "\n#'   The type's defaults are {}.",
            match &defaults[..] {
                [one] => one.clone(),
                [init @ .., last] => format!("{} and {last}", init.join(", ")),
                [] => unreachable!(),
            }
        )
    };
    format!(
        "#' The `all.equal()` method of the Rust type `{type_name}`, from\n\
         #' `#[derive(AllEqual)]`. It compares `target` and `current` field by field,\n\
         #' each field with `all.equal()`, which dispatches on the field's own class.\n\
         #' The fields compared: {listed}.\n\
         #'\n\
         #' @param target,current The objects to compare.\n\
         #' @param tolerance,scale,check.attributes,check.names Passed on to the\n\
         #'   comparison of each field when the call or the type gives them, unless\n\
         #'   the field fixes its own; see [base::all.equal()].{defaults}\n\
         #' @param ... Passed on to the comparison of each field.\n\
         #' @return `TRUE` when every field is equal, otherwise a character vector of\n\
         #'   the differences, each prefixed with its field."
    )
}

// endregion

/// `#[derive(AllEqual)]`: see the module docs.
pub(crate) fn derive_all_equal(input: DeriveInput) -> syn::Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "`AllEqual` does not support generic structs: its `.Call` entry cannot be generic",
        ));
    }
    let struct_settings = parse_struct_settings(&input)?;
    let fields = compared_fields(&input)?;

    let name = &input.ident;
    let type_name = crate::naming::ident_name(name);
    let c_name = crate::naming::all_equal_c_name(&type_name);
    let c_ident = format_ident!("{c_name}");

    let arms = fields.iter().enumerate().map(|(i, field)| {
        let index = i32::try_from(i + 1).expect("a struct has fewer than i32::MAX fields");
        let member = &field.member;
        let borrow = quote! {
            let __mx_data = unsafe { ::miniextendr_api::externalptr::sidecar_r_struct::<#name>(x) };
            let __mx_value: &#name = &__mx_data;
        };
        let body = match &field.value {
            // The conversion is spanned at the field's type, so a type that
            // is not `Clone + IntoR` is reported there.
            FieldValue::Converted => {
                let convert = quote_spanned! {field.ty.span()=>
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::core::clone::Clone::clone(&__mx_value.#member),
                    )
                };
                quote! {{ #borrow #convert }}
            }
            FieldValue::Getter(path, span) => {
                let convert = quote_spanned! {*span=>
                    ::miniextendr_api::into_r::IntoR::into_sexp(#path(__mx_value))
                };
                quote! {{ #borrow #convert }}
            }
            FieldValue::Sidecar(index) => quote! {
                unsafe { ::miniextendr_api::externalptr::sidecar_r_get::<#name>(x, #index, true) }
            },
            FieldValue::Pointer { optional: false } => quote! {{
                #borrow
                ::miniextendr_api::into_r::IntoR::into_sexp(
                    ::miniextendr_api::externalptr::ExternalPtr::same_object(&__mx_value.#member),
                )
            }},
            FieldValue::Pointer { optional: true } => quote! {{
                #borrow
                ::miniextendr_api::into_r::IntoR::into_sexp(::core::option::Option::map(
                    ::core::option::Option::as_ref(&__mx_value.#member),
                    ::miniextendr_api::externalptr::ExternalPtr::same_object,
                ))
            }},
        };
        quote! { #index => #body, }
    });

    let def_ident = format_ident!("__MX_CALL_DEF_ALL_EQUAL_{type_name}");
    let registration = crate::call_method_def::call_method_def(&def_ident, &c_ident, 2);
    let entry_ident = format_ident!("__MX_ALL_EQUAL_{type_name}");
    let c_doc =
        format!("Generated field reader of the `all.equal()` method of Rust type `{type_name}`.");
    let source_location_doc = crate::source_location_doc(name.span());
    let start = name.span().start();
    let source_line = syn::LitInt::new(&start.line.to_string(), name.span());
    let source_column = syn::LitInt::new(&(start.column + 1).to_string(), name.span());
    let method = LitStr::new(&r_method(&c_name, &struct_settings, &fields), name.span());
    let docs = LitStr::new(&r_docs(&type_name, &struct_settings, &fields), name.span());

    Ok(quote! {
        #[doc = #c_doc]
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        // A field's clone is spanned at its type, so a `Copy` field is
        // linted in the user's crate.
        #[allow(clippy::clone_on_copy)]
        pub unsafe extern "C-unwind" fn #c_ident(
            x: ::miniextendr_api::SEXP,
            i: ::miniextendr_api::SEXP,
        ) -> ::miniextendr_api::SEXP {
            let __miniextendr_deferred_mark = ::miniextendr_api::deferred_condition::mark();
            let __miniextendr_value = ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                || {
                    let __mx_index: i32 = match ::miniextendr_api::TryFromSexp::try_from_sexp(i) {
                        ::core::result::Result::Ok(index) => index,
                        ::core::result::Result::Err(err) => {
                            ::core::panic!("`all.equal()` reads a field by its position: {err}")
                        }
                    };
                    match __mx_index {
                        #(#arms)*
                        _ => ::core::panic!(
                            "`all.equal()` compares no field {} of `{}`",
                            __mx_index,
                            #type_name,
                        ),
                    }
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

        #[doc(hidden)]
        #registration

        /// The `all.equal()` method's R source, via distributed slice.
        #[doc = #source_location_doc]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_ALL_EQUAL), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        static #entry_ident: ::miniextendr_api::registry::AllEqualEntry =
            ::miniextendr_api::registry::AllEqualEntry {
                rust_type: #type_name,
                method: #method,
                docs: #docs,
                source_file: ::core::file!(),
                source_line: #source_line,
                source_column: #source_column,
            };
    })
}

#[cfg(test)]
mod tests;
