//! Lint helpers for miniextendr usage in a crate.
//!
//! # Usage in build.rs
//!
//! ```ignore
//! fn main() {
//!     miniextendr_lint::build_script();
//! }
//! ```
//!
//! # Architecture Note: Parser Duplication
//!
//! This crate contains a **copy** of `miniextendr_module.rs` from `miniextendr-macros`.
//!
//! - **Source**: `miniextendr-macros/src/miniextendr_module.rs`
//! - **Copy**: `miniextendr-lint/src/miniextendr_module.rs` (this file)
//! - **Requirement**: The parser imports `call_method_def_ident_for` and
//!   `r_wrapper_const_ident_for` from `crate::`, so we must define stubs below
//!
//! **Why duplicate instead of sharing?**
//! - Allows miniextendr-lint to be published independently to crates.io
//! - `#[path = "../../"]` includes don't work in published packages
//! - Trade-off: Code duplication vs. independent publishing
//!
//! **Keeping in sync:**
//! When `miniextendr-macros/src/miniextendr_module.rs` changes, manually copy to
//! `miniextendr-lint/src/miniextendr_module.rs`. Changes are infrequent.

// Parser module (copied from miniextendr-macros for independent publishing).
#[allow(dead_code)]
mod miniextendr_module;

// TODO: Check how many miniextendr_module! calls there is in a module
// atmost 1

// TODO: check how many reflections a type has; is it externalptr? is it an impl-block?
// is it altrep? is it too much?

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::{Attribute, Item, Macro};

/// Required by `miniextendr_module.rs` shared parser (not called by lint).
#[allow(dead_code)]
fn call_method_def_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("call_method_def_{rust_ident}")
}

/// Required by `miniextendr_module.rs` shared parser (not called by lint).
#[allow(dead_code)]
fn r_wrapper_const_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let rust_ident_upper = rust_ident.to_string().to_uppercase();
    quote::format_ident!("R_WRAPPER_{rust_ident_upper}")
}

fn cargo_warning(message: &str) {
    let message = message.replace(['\n', '\r'], " ");
    println!("cargo::warning={}", message.trim());
}

/// Entry point for build.rs. Runs the lint and prints cargo directives.
///
/// Controlled by `MINIEXTENDR_LINT` env var (enabled by default).
/// Set to `0`, `false`, `no`, or `off` to disable.
pub fn build_script() {
    println!("cargo::rerun-if-env-changed=MINIEXTENDR_LINT");

    let enabled = match lint_enabled("MINIEXTENDR_LINT") {
        Ok(enabled) => enabled,
        Err(message) => {
            cargo_warning(&message);
            return;
        }
    };

    if !enabled {
        return;
    }

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(err) => {
            cargo_warning(&format!("CARGO_MANIFEST_DIR: {err}"));
            return;
        }
    };

    let report = match run(&manifest_dir) {
        Ok(report) => report,
        Err(message) => {
            cargo_warning(&message);
            return;
        }
    };

    for path in &report.files {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    if !report.errors.is_empty() {
        cargo_warning("miniextendr-lint found issues");
        for err in &report.errors {
            cargo_warning(err);
        }
    }
}

#[derive(Debug, Default)]
pub struct LintReport {
    pub files: Vec<PathBuf>,
    pub errors: Vec<String>,
}

/// Returns whether the lint should run based on the given env var.
///
/// Defaults to `true` when the var is unset. Set to 0/false/no/off to disable.
pub fn lint_enabled(env_var: &str) -> Result<bool, String> {
    match env::var(env_var) {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "0" | "false" | "no" | "off" | "" => Ok(false),
                "1" | "true" | "yes" | "on" => Ok(true),
                _ => Err(format!(
                    "{env_var} has invalid value '{value}'; use 1/0, true/false, yes/no, on/off"
                )),
            }
        }
        Err(env::VarError::NotPresent) => Ok(true),
        Err(err) => Err(format!("{env_var}: {err}")),
    }
}

/// Run the lint against the crate rooted at `root`.
///
/// If `root/src` exists, that directory is scanned. Otherwise `root` is scanned.
pub fn run(root: impl AsRef<Path>) -> Result<LintReport, String> {
    let root = root.as_ref();
    let src_dir = if root.join("src").is_dir() {
        root.join("src")
    } else {
        root.to_path_buf()
    };

    if !src_dir.is_dir() {
        return Err(format!(
            "miniextendr-lint: root is not a directory: {}",
            src_dir.display()
        ));
    }

    let mut rs_files = Vec::new();
    collect_rs_files(&src_dir, &mut rs_files)
        .map_err(|err| format!("miniextendr-lint: failed to read src: {err}"))?;
    rs_files.sort();

    let mut errors = Vec::new();
    for path in &rs_files {
        if let Err(mut file_errors) = lint_file(path) {
            errors.append(&mut file_errors);
        }
    }

    Ok(LintReport {
        files: rs_files,
        errors,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum LintKind {
    Function,
    Impl,
    Struct,
    TraitImpl,
}

#[derive(Clone, Debug)]
struct LintItem {
    kind: LintKind,
    name: String,
    line: usize,
}

impl PartialEq for LintItem {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name
    }
}

impl Eq for LintItem {}

impl std::hash::Hash for LintItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.name.hash(state);
    }
}

impl LintItem {
    fn new(kind: LintKind, name: String, line: usize) -> Self {
        Self { kind, name, line }
    }

    fn display(&self) -> String {
        match self.kind {
            LintKind::Function => format!("fn {}", self.name),
            LintKind::Impl => format!("impl {}", self.name),
            LintKind::Struct => format!("struct {}", self.name),
            LintKind::TraitImpl => format!("impl {}", self.name), // "impl Trait for Type"
        }
    }
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            collect_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(name, "target" | "ra_target" | ".cargo" | ".git" | "vendor")
}

fn lint_file(path: &Path) -> Result<(), Vec<String>> {
    let src = match fs::read_to_string(path) {
        Ok(src) => src,
        Err(err) => return Err(vec![format!("{}: failed to read: {err}", path.display())]),
    };

    let parsed = match syn::parse_file(&src) {
        Ok(parsed) => parsed,
        Err(err) => {
            return Err(vec![format!("{}: failed to parse: {err}", path.display())]);
        }
    };

    let mut miniextendr_items = HashSet::new();
    let mut module_items = HashSet::new();
    let mut errors = Vec::new();

    collect_items(
        &parsed.items,
        path,
        &mut miniextendr_items,
        &mut module_items,
        &mut errors,
    );

    if !errors.is_empty() {
        return Err(errors);
    }

    if !miniextendr_items.is_empty() && module_items.is_empty() {
        errors.push(format!(
            "{}: #[miniextendr] items found but no miniextendr_module! in file",
            path.display()
        ));
    }

    if miniextendr_items.is_empty() && !module_items.is_empty() {
        errors.push(format!(
            "{}: miniextendr_module! present but no #[miniextendr] items in file",
            path.display()
        ));
    }

    // Check for #[miniextendr] items missing from module
    for item in &miniextendr_items {
        if !module_items.contains(item) {
            errors.push(format!(
                "{}:{}: #[miniextendr] {} not listed in miniextendr_module!",
                path.display(),
                item.line,
                item.display()
            ));
        }
    }

    // Check for module items without #[miniextendr] attribute (bidirectional)
    for item in &module_items {
        if !miniextendr_items.contains(item) {
            errors.push(format!(
                "{}:{}: {} listed in miniextendr_module! but has no #[miniextendr] attribute",
                path.display(),
                item.line,
                item.display()
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        Err(errors)
    }
}

/// Parse the class system from #[miniextendr(...)] attribute.
/// Returns Some("s3"), Some("s4"), etc. or None for default (env).
fn parse_class_system(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr
            .path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "miniextendr")
        {
            // Try to parse the attribute arguments
            if let syn::Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                let tokens = tokens.trim();
                if !tokens.is_empty() {
                    return Some(tokens.to_string());
                }
            }
        }
    }
    None
}

fn collect_items(
    items: &[Item],
    path: &Path,
    miniextendr_items: &mut HashSet<LintItem>,
    module_items: &mut HashSet<LintItem>,
    errors: &mut Vec<String>,
) {
    // Track inherent impl class systems for compatibility checking
    let mut inherent_impl_class_systems: std::collections::HashMap<String, (String, usize)> =
        std::collections::HashMap::new();
    // Track trait impls to check compatibility after all items are processed
    let mut trait_impls_to_check: Vec<(String, String, Option<String>, usize)> = Vec::new();

    for item in items {
        match item {
            Item::Fn(item_fn) => {
                if has_miniextendr_attr(&item_fn.attrs) {
                    let line = item_fn.sig.ident.span().start().line;
                    miniextendr_items.insert(LintItem::new(
                        LintKind::Function,
                        item_fn.sig.ident.to_string(),
                        line,
                    ));
                }
            }
            Item::Struct(item_struct) => {
                if has_miniextendr_attr(&item_struct.attrs) {
                    let line = item_struct.ident.span().start().line;
                    miniextendr_items.insert(LintItem::new(
                        LintKind::Struct,
                        item_struct.ident.to_string(),
                        line,
                    ));
                }
            }
            Item::Impl(item_impl) => {
                if has_miniextendr_attr(&item_impl.attrs) {
                    let line = item_impl.self_ty.span().start().line;
                    let class_system = parse_class_system(&item_impl.attrs);

                    match impl_type_name(&item_impl.self_ty) {
                        Some(type_name) => {
                            // Check if this is a trait impl (impl Trait for Type)
                            if let Some((_, trait_path, _)) = &item_impl.trait_ {
                                // Get trait name from path
                                if let Some(trait_seg) = trait_path.segments.last() {
                                    let trait_name = trait_seg.ident.to_string();
                                    let full_name = format!("{} for {}", trait_name, type_name);
                                    miniextendr_items.insert(LintItem::new(
                                        LintKind::TraitImpl,
                                        full_name,
                                        line,
                                    ));

                                    // Store for compatibility checking
                                    trait_impls_to_check.push((
                                        type_name.clone(),
                                        trait_name,
                                        class_system,
                                        line,
                                    ));
                                }
                            } else {
                                // Regular impl block - track its class system
                                inherent_impl_class_systems.insert(
                                    type_name.clone(),
                                    (class_system.unwrap_or_default(), line),
                                );

                                miniextendr_items.insert(LintItem::new(
                                    LintKind::Impl,
                                    type_name,
                                    line,
                                ));
                            }
                        }
                        None => errors.push(format!(
                            "{}:{}: #[miniextendr] impl type not supported by lint",
                            path.display(),
                            line
                        )),
                    }
                }
            }
            Item::Macro(item_macro) => {
                if is_miniextendr_module_macro(&item_macro.mac) {
                    match parse_miniextendr_module_items(&item_macro.mac) {
                        Ok(items) => {
                            module_items.extend(items);
                        }
                        Err(err) => errors.push(format!(
                            "{}: failed to parse miniextendr_module!: {err}",
                            path.display()
                        )),
                    }
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, items)) = &item_mod.content {
                    collect_items(items, path, miniextendr_items, module_items, errors);
                }
            }
            _ => {}
        }
    }

    // Check class system compatibility for trait impls
    // Env-style trait impls (default) require Env-style inherent impls
    // because they use Type$Trait$method() patterns that need an environment
    for (type_name, trait_name, trait_class_system, line) in trait_impls_to_check {
        let trait_style = trait_class_system.as_deref().unwrap_or("env");

        // Env trait impl requires Env inherent impl
        if trait_style == "env"
            && let Some((inherent_style, _inherent_line)) =
                inherent_impl_class_systems.get(&type_name)
            && !inherent_style.is_empty()
            && inherent_style != "env"
        {
            errors.push(format!(
                "{}:{}: #[miniextendr] impl {} for {} uses Env-style (default) which requires \
                Env-style inherent impl, but {} uses #[miniextendr({})]. \
                Env-style trait impls generate Type$Trait$method() patterns that need \
                the type to be an environment. Either change the trait impl to use \
                #[miniextendr({})] or change the inherent impl to #[miniextendr].",
                path.display(),
                line,
                trait_name,
                type_name,
                type_name,
                inherent_style,
                inherent_style
            ));
        }

        // S3/S4/S7/R6 trait impls are compatible with Env inherent impls
        // because they use their own dispatch mechanisms (generics, methods, etc.)
    }
}

fn has_miniextendr_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "miniextendr")
    })
}

fn impl_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string()),
        syn::Type::Reference(type_ref) => impl_type_name(&type_ref.elem),
        _ => None,
    }
}

fn is_miniextendr_module_macro(mac: &Macro) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "miniextendr_module")
}

fn parse_miniextendr_module_items(mac: &Macro) -> syn::Result<Vec<LintItem>> {
    let parsed = syn::parse2::<miniextendr_module::MiniextendrModule>(mac.tokens.clone())?;
    let mut items = Vec::new();

    for func in parsed.functions {
        let line = func.ident.span().start().line;
        items.push(LintItem::new(
            LintKind::Function,
            func.ident.to_string(),
            line,
        ));
    }

    for strukt in parsed.structs {
        let line = strukt.ident.span().start().line;
        items.push(LintItem::new(
            LintKind::Struct,
            strukt.ident.to_string(),
            line,
        ));
    }

    for impl_block in parsed.impls {
        let line = impl_block.ident.span().start().line;
        items.push(LintItem::new(
            LintKind::Impl,
            impl_block.ident.to_string(),
            line,
        ));
    }

    // Trait implementations (impl Trait for Type;)
    for trait_impl in parsed.trait_impls {
        let line = trait_impl.type_ident.span().start().line;
        // Get trait name from path
        let trait_name = trait_impl
            .trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let full_name = format!("{} for {}", trait_name, trait_impl.type_ident);
        items.push(LintItem::new(LintKind::TraitImpl, full_name, line));
    }

    Ok(items)
}
