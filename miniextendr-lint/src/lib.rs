//! miniextendr-lint: internal build-time lint helpers for the workspace.
//!
//! This crate scans Rust sources for miniextendr macro usage and emits
//! cargo warnings with actionable diagnostics. It is intended for local
//! development and CI, not as a public API.
//!
//! ## Usage in build.rs
//!
//! ```ignore
//! fn main() {
//!     miniextendr_lint::build_script();
//! }
//! ```
//!
//! ## Configuration
//! - Controlled by the `MINIEXTENDR_LINT` env var (enabled by default).
//! - Set it to `0`, `false`, `no`, or `off` to disable.
//!
//! ## Architecture note: parser duplication
//!
//! This crate contains a **copy** of `miniextendr_module.rs` from
//! `miniextendr-macros` so it can be published independently.
//!
//! - **Source**: `miniextendr-macros/src/miniextendr_module.rs`
//! - **Copy**: `miniextendr-lint/src/miniextendr_module.rs` (this file)
//! - **Requirement**: the parser imports `call_method_def_ident_for` and
//!   `r_wrapper_const_ident_for` from `crate::`, so we define stubs below
//!
//! **Why duplicate instead of sharing?**
//! - Allows `miniextendr-lint` to be published independently to crates.io
//! - `#[path = "../../"]` includes do not work in published packages
//!
//! **Keeping in sync:**
//! When `miniextendr-macros/src/miniextendr_module.rs` changes, manually copy to
//! `miniextendr-lint/src/miniextendr_module.rs`. Changes are infrequent.

// Parser module (copied from miniextendr-macros for independent publishing).
// Allow dead_code since we only use parsing, not code generation helpers.
#[allow(dead_code)]
mod miniextendr_module;

// Stubs required by miniextendr_module.rs (which imports these from crate::)
// The lint only uses parsing, not code generation, so these return dummy idents.
#[allow(dead_code)]
pub(crate) fn call_method_def_ident_for(_ident: &syn::Ident) -> syn::Ident {
    syn::Ident::new("__stub", proc_macro2::Span::call_site())
}

#[allow(dead_code)]
pub(crate) fn r_wrapper_const_ident_for(_ident: &syn::Ident) -> syn::Ident {
    syn::Ident::new("__stub", proc_macro2::Span::call_site())
}

// TODO: check how many reflections a type has; is it externalptr? is it an impl-block?
// is it altrep? is it too much?

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::{Attribute, Item, Macro};

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
    /// Optional label for impl blocks with multiple impl blocks for the same type.
    label: Option<String>,
    line: usize,
}

impl PartialEq for LintItem {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name && self.label == other.label
    }
}

impl Eq for LintItem {}

impl std::hash::Hash for LintItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.name.hash(state);
        self.label.hash(state);
    }
}

impl LintItem {
    fn new(kind: LintKind, name: String, line: usize) -> Self {
        Self {
            kind,
            name,
            label: None,
            line,
        }
    }

    fn with_label(kind: LintKind, name: String, label: Option<String>, line: usize) -> Self {
        Self {
            kind,
            name,
            label,
            line,
        }
    }

    fn display(&self) -> String {
        match self.kind {
            LintKind::Function => format!("fn {}", self.name),
            LintKind::Impl => {
                if let Some(ref label) = self.label {
                    format!("impl {} as \"{}\"", self.name, label)
                } else {
                    format!("impl {}", self.name)
                }
            }
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
    let mut module_macro_locations = Vec::new();
    let mut errors = Vec::new();

    collect_items(
        &parsed.items,
        path,
        &mut miniextendr_items,
        &mut module_items,
        &mut module_macro_locations,
        &mut errors,
    );

    if !errors.is_empty() {
        return Err(errors);
    }

    // Check for multiple miniextendr_module! macros (at most 1 per file)
    if module_macro_locations.len() > 1 {
        errors.push(format!(
            "{}: multiple miniextendr_module! macros found (at most 1 allowed per file). \
             Found at lines: {}",
            path.display(),
            module_macro_locations
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
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

/// Resolve file path for an out-of-line module declaration.
///
/// For `mod foo;` in `/path/to/bar.rs`, tries:
/// - `/path/to/foo.rs`
/// - `/path/to/foo/mod.rs`
///
/// Returns None if neither exists.
fn resolve_file_module(parent_path: &Path, mod_ident: &syn::Ident) -> Option<PathBuf> {
    let parent_dir = parent_path.parent()?;
    let mod_name = mod_ident.to_string();

    // Try foo.rs
    let sibling = parent_dir.join(format!("{}.rs", mod_name));
    if sibling.exists() {
        return Some(sibling);
    }

    // Try foo/mod.rs
    let subdir_mod = parent_dir.join(&mod_name).join("mod.rs");
    if subdir_mod.exists() {
        return Some(subdir_mod);
    }

    None
}

/// Parse a module file and collect items from it.
fn collect_items_from_file(
    mod_path: &Path,
    miniextendr_items: &mut HashSet<LintItem>,
    module_items: &mut HashSet<LintItem>,
    module_macro_locations: &mut Vec<usize>,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let src = fs::read_to_string(mod_path).map_err(|e| format!("failed to read: {}", e))?;

    let parsed = syn::parse_file(&src).map_err(|e| format!("failed to parse: {}", e))?;

    collect_items(
        &parsed.items,
        mod_path,
        miniextendr_items,
        module_items,
        module_macro_locations,
        errors,
    );
    Ok(())
}

/// Parsed miniextendr attribute information for an impl block.
#[derive(Debug, Default)]
struct MiniextendrImplAttrs {
    /// Class system (e.g., "r6", "s3", "s4", "s7", or empty for env)
    class_system: Option<String>,
    /// Optional label for distinguishing multiple impl blocks of the same type
    label: Option<String>,
}

/// Parse the #[miniextendr(...)] attribute to extract class system and label.
///
/// Handles:
/// - `#[miniextendr]` → default (env), no label
/// - `#[miniextendr(env)]` → explicit env, no label
/// - `#[miniextendr(r6)]` → r6 class system, no label
/// - `#[miniextendr(label = "foo")]` → default (env), labeled
/// - `#[miniextendr(env, label = "foo")]` → explicit env, labeled
/// - `#[miniextendr(r6, label = "foo")]` → r6 class system, labeled
fn parse_miniextendr_impl_attrs(attrs: &[Attribute]) -> MiniextendrImplAttrs {
    let mut result = MiniextendrImplAttrs::default();

    for attr in attrs {
        if attr
            .path()
            .segments
            .last()
            .is_none_or(|seg| seg.ident != "miniextendr")
        {
            continue;
        }

        // Try to parse the attribute arguments
        if let syn::Meta::List(meta_list) = &attr.meta {
            let tokens = meta_list.tokens.to_string();
            let tokens = tokens.trim();
            if tokens.is_empty() {
                continue;
            }

            // Parse tokens like: "r6, label = \"methods\"" or "label = \"foo\""
            for part in tokens.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }

                if part.starts_with("label") {
                    // Extract label value: label = "..."
                    if let Some(eq_pos) = part.find('=') {
                        let value = part[eq_pos + 1..].trim();
                        // Remove quotes
                        let value = value.trim_matches('"').trim_matches('\'');
                        result.label = Some(value.to_string());
                    }
                } else if !part.contains('=') {
                    // This is a class system identifier (env, r6, s3, s4, s7)
                    // Note: "env" is valid even though it's the default
                    result.class_system = Some(part.to_string());
                }
                // Skip other key=value pairs (like class = "CustomName")
            }
        }
    }

    result
}

/// Parse the class system from #[miniextendr(...)] attribute.
/// Returns Some("s3"), Some("s4"), etc. or None for default (env).
#[allow(dead_code)]
fn parse_class_system(attrs: &[Attribute]) -> Option<String> {
    parse_miniextendr_impl_attrs(attrs).class_system
}

fn collect_items(
    items: &[Item],
    path: &Path,
    miniextendr_items: &mut HashSet<LintItem>,
    module_items: &mut HashSet<LintItem>,
    module_macro_locations: &mut Vec<usize>,
    errors: &mut Vec<String>,
) {
    // Track inherent impl class systems for compatibility checking
    let mut inherent_impl_class_systems: std::collections::HashMap<String, (String, usize)> =
        std::collections::HashMap::new();
    // Track trait impls to check compatibility after all items are processed
    let mut trait_impls_to_check: Vec<(String, String, Option<String>, usize)> = Vec::new();
    // Track impl blocks per type for multiple impl block validation
    // Maps type_name -> Vec<(label, line)>
    let mut impl_blocks_per_type: std::collections::HashMap<String, Vec<(Option<String>, usize)>> =
        std::collections::HashMap::new();

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
                    let impl_attrs = parse_miniextendr_impl_attrs(&item_impl.attrs);
                    let class_system = impl_attrs.class_system.clone();
                    let label = impl_attrs.label.clone();

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

                                // Track for multiple impl block validation
                                impl_blocks_per_type
                                    .entry(type_name.clone())
                                    .or_default()
                                    .push((label.clone(), line));

                                miniextendr_items.insert(LintItem::with_label(
                                    LintKind::Impl,
                                    type_name,
                                    label,
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
                    let line = item_macro.mac.path.span().start().line;
                    module_macro_locations.push(line);

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
                    // Inline module: mod foo { ... }
                    collect_items(
                        items,
                        path,
                        miniextendr_items,
                        module_items,
                        module_macro_locations,
                        errors,
                    );
                } else {
                    // File module: mod foo;
                    // Resolve and parse the file, then recursively collect
                    // Note: Each file module gets its own module_macro_locations
                    // because "at most 1 per file" applies per-file, not per-crate
                    if let Some(mod_path) = resolve_file_module(path, &item_mod.ident) {
                        let mut child_module_macro_locations = Vec::new();
                        if let Err(e) = collect_items_from_file(
                            &mod_path,
                            miniextendr_items,
                            module_items,
                            &mut child_module_macro_locations,
                            errors,
                        ) {
                            errors.push(format!(
                                "{}: failed to process module {}: {}",
                                path.display(),
                                item_mod.ident,
                                e
                            ));
                        }
                        // Check child file for multiple modules
                        if child_module_macro_locations.len() > 1 {
                            errors.push(format!(
                                "{}: multiple miniextendr_module! macros found (at most 1 allowed per file). \
                                 Found at lines: {}",
                                mod_path.display(),
                                child_module_macro_locations
                                    .iter()
                                    .map(|l| l.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                    }
                    // Silently skip if module file can't be resolved (might be cfg'd out or generated)
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

    // Validate multiple impl blocks: if a type has 2+ impl blocks, all must have labels
    for (type_name, impl_blocks) in &impl_blocks_per_type {
        if impl_blocks.len() > 1 {
            // Check if any impl block is missing a label
            let missing_labels: Vec<_> = impl_blocks
                .iter()
                .filter(|(label, _)| label.is_none())
                .map(|(_, line)| *line)
                .collect();

            if !missing_labels.is_empty() {
                errors.push(format!(
                    "{}:{}: type `{}` has {} impl blocks but some are missing labels. \
                     When a type has multiple #[miniextendr] impl blocks, all must have \
                     distinct labels using #[miniextendr(label = \"...\")]. \
                     Unlabeled impl blocks at lines: {}",
                    path.display(),
                    impl_blocks[0].1, // First occurrence line
                    type_name,
                    impl_blocks.len(),
                    missing_labels
                        .iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }

            // Check for duplicate labels
            let mut seen_labels: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for (label, line) in impl_blocks {
                if let Some(label) = label {
                    if let Some(first_line) = seen_labels.get(label.as_str()) {
                        errors.push(format!(
                            "{}:{}: duplicate label \"{}\" for type `{}`. \
                             First occurrence at line {}. Each impl block must have a unique label.",
                            path.display(),
                            line,
                            label,
                            type_name,
                            first_line
                        ));
                    } else {
                        seen_labels.insert(label.as_str(), *line);
                    }
                }
            }
        }
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
        items.push(LintItem::with_label(
            LintKind::Impl,
            impl_block.ident.to_string(),
            impl_block.label.clone(),
            line,
        ));
    }

    // Trait implementations (impl Trait for Type;)
    for trait_impl in parsed.trait_impls {
        let line = trait_impl.impl_type.span().start().line;
        // Get trait name from path
        let trait_name = trait_impl
            .trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        // Use type_name_sanitized for consistent display of generic types
        let type_name = trait_impl.type_name_sanitized();
        let full_name = format!("{} for {}", trait_name, type_name);
        items.push(LintItem::new(LintKind::TraitImpl, full_name, line));
    }

    Ok(items)
}
