//! Shared crate index built from a single parse pass over all source files.
//!
//! All lint rules operate on this index rather than re-parsing files.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use miniextendr_macros_core::miniextendr_module;
use syn::Item;
use syn::spanned::Spanned;

use crate::helpers::{
    extract_cfg_attrs, extract_path_attr, extract_roxygen_tags, has_altrep_derive,
    has_external_ptr_derive, has_miniextendr_attr, has_vctrs_derive, impl_type_name,
    is_altrep_struct, is_miniextendr_module_macro, parse_miniextendr_impl_attrs,
};

// ── Lint item types ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LintKind {
    Function,
    Impl,
    Struct,
    TraitImpl,
    Vctrs,
}

#[derive(Clone, Debug)]
pub struct LintItem {
    pub kind: LintKind,
    pub name: String,
    pub label: Option<String>,
    pub line: usize,
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
    pub fn new(kind: LintKind, name: String, line: usize) -> Self {
        Self {
            kind,
            name,
            label: None,
            line,
        }
    }

    pub fn with_label(kind: LintKind, name: String, label: Option<String>, line: usize) -> Self {
        Self {
            kind,
            name,
            label,
            line,
        }
    }

    /// Renders the item in module-declaration syntax for diagnostics.
    pub fn display(&self) -> String {
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
            LintKind::TraitImpl => format!("impl {}", self.name),
            LintKind::Vctrs => format!("vctrs {}", self.name),
        }
    }

    /// Normalized key for duplicate detection (ignores line number).
    pub fn dedup_key(&self) -> String {
        match self.kind {
            LintKind::Function => format!("fn:{}", self.name),
            LintKind::Impl => match &self.label {
                Some(label) => format!("impl:{}:{}", self.name, label),
                None => format!("impl:{}", self.name),
            },
            LintKind::Struct => format!("struct:{}", self.name),
            LintKind::TraitImpl => format!("trait_impl:{}", self.name),
            LintKind::Vctrs => format!("vctrs:{}", self.name),
        }
    }
}

// ── Trait impl entries from miniextendr_module! ─────────────────────────────

#[derive(Clone, Debug)]
pub struct TraitImplEntry {
    pub trait_name: String,
    pub type_name: String,
    pub type_name_sanitized: String,
    pub is_generic: bool,
    pub line: usize,
    pub cfgs: Vec<String>,
}

// ── Attributed trait impls from source ──────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AttributedTraitImpl {
    pub type_name: String,
    pub trait_name: String,
    pub class_system: Option<String>,
    pub line: usize,
}

// ── Per-file parsed data ────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct FileData {
    // Source items
    pub miniextendr_items: Vec<LintItem>,

    // Module macro data
    pub module_items: Vec<LintItem>,
    pub module_macro_lines: Vec<usize>,
    pub module_name: Option<String>,
    pub module_uses: Vec<String>,

    // Type/derive information
    pub types_with_external_ptr: HashSet<String>,
    pub types_with_typed_external: HashSet<String>,

    // Impl block details
    pub impl_type_entries: Vec<(String, usize)>,
    pub trait_impl_entries: Vec<TraitImplEntry>,
    pub inherent_impl_class_systems: HashMap<String, (String, usize)>,
    pub attributed_trait_impls: Vec<AttributedTraitImpl>,
    pub impl_blocks_per_type: HashMap<String, Vec<(Option<String>, usize)>>,

    // Function details
    pub fn_visibility: HashMap<String, bool>,

    // cfg tracking
    pub item_cfgs: HashMap<String, Vec<String>>,
    pub module_entry_cfgs: HashMap<String, Vec<String>>,

    // Module tree
    /// Simple `mod child;` declarations (by ident name).
    pub declared_child_mods: Vec<String>,
    /// `#[path = "file.rs"] mod name;` declarations: (mod_name, file_path_str).
    pub path_redirected_mods: Vec<(String, String)>,
    /// cfg attrs on `mod child;` declarations: mod_name -> cfg strings.
    pub mod_decl_cfgs: HashMap<String, Vec<String>>,
    /// cfg attrs on `use child;` entries in `miniextendr_module!`: use_name -> cfg strings.
    pub use_entry_cfgs: HashMap<String, Vec<String>>,

    // Export control
    pub export_control: HashMap<String, (bool, bool)>,

    // Doc-comment roxygen tags per function/impl name
    /// Known roxygen tags: "@noRd", "@export", "@keywords internal"
    pub fn_doc_tags: HashMap<String, Vec<String>>,

    // Safety lint data
    /// Lines containing direct Rf_error/Rf_errorcall calls: (function_name, line_number).
    pub rf_error_calls: Vec<(String, usize)>,
    /// Lines containing `ffi::*_unchecked()` calls: (function_name, line_number).
    pub ffi_unchecked_calls: Vec<(String, usize)>,
}

impl FileData {
    /// Returns miniextendr_items as a HashSet for O(1) lookup.
    pub fn miniextendr_items_set(&self) -> HashSet<&LintItem> {
        self.miniextendr_items.iter().collect()
    }

    /// Returns module_items as a HashSet for O(1) lookup.
    pub fn module_items_set(&self) -> HashSet<&LintItem> {
        self.module_items.iter().collect()
    }

    /// Whether this file has a miniextendr_module! macro.
    pub fn has_module_macro(&self) -> bool {
        !self.module_macro_lines.is_empty()
    }
}

// ── Crate index ─────────────────────────────────────────────────────────────

/// Shared parsed state for all lint rules.
pub struct CrateIndex {
    /// All scanned Rust source files.
    pub files: Vec<PathBuf>,
    /// Per-file parsed data.
    pub file_data: HashMap<PathBuf, FileData>,
}

impl CrateIndex {
    /// Build the index from a crate root directory.
    pub fn build(root: &Path) -> Result<Self, String> {
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
        collect_rs_files_from_module_tree(&src_dir, &mut rs_files)?;
        rs_files.sort();

        let mut file_data = HashMap::new();
        let mut parse_errors = Vec::new();

        for path in &rs_files {
            match parse_file(path) {
                Ok(data) => {
                    file_data.insert(path.clone(), data);
                }
                Err(err) => parse_errors.push(err),
            }
        }

        if !parse_errors.is_empty() {
            return Err(parse_errors.join("; "));
        }

        Ok(Self {
            files: rs_files,
            file_data,
        })
    }

    /// All files that have a `miniextendr_module!`.
    pub fn files_with_module(&self) -> Vec<&Path> {
        self.file_data
            .iter()
            .filter(|(_, data)| data.has_module_macro())
            .map(|(path, _)| path.as_path())
            .collect()
    }

    /// Collect all module names across all files.
    pub fn all_module_names(&self) -> Vec<(&Path, &str)> {
        self.file_data
            .iter()
            .filter_map(|(path, data)| {
                data.module_name
                    .as_deref()
                    .map(|name| (path.as_path(), name))
            })
            .collect()
    }

    /// Returns the set of files that are in cfg-gated alternative modules.
    ///
    /// A file is cfg-gated if its parent declares the same module name multiple times
    /// (via `declared_child_mods` and/or `path_redirected_mods`), indicating that
    /// only one alternative is active at compile time.
    pub fn cfg_gated_module_files(&self) -> HashSet<PathBuf> {
        let mut result = HashSet::new();

        for (parent_path, parent_data) in &self.file_data {
            let parent_dir = match parent_path.parent() {
                Some(dir) => dir,
                None => continue,
            };

            // Count total declarations per module name
            let mut name_counts: HashMap<&str, usize> = HashMap::new();
            for name in &parent_data.declared_child_mods {
                *name_counts.entry(name.as_str()).or_default() += 1;
            }
            for (mod_name, _) in &parent_data.path_redirected_mods {
                *name_counts.entry(mod_name.as_str()).or_default() += 1;
            }

            // For names with 2+ declarations, mark all target files
            // (old pattern: #[cfg(feat)] mod foo; #[cfg(not(feat))] mod foo;)
            for (mod_name, count) in &name_counts {
                if *count < 2 {
                    continue;
                }

                if parent_data
                    .declared_child_mods
                    .iter()
                    .any(|n| n == mod_name)
                {
                    result.insert(parent_dir.join(format!("{mod_name}.rs")));
                    result.insert(parent_dir.join(mod_name).join("mod.rs"));
                }

                for (redir_name, file_path) in &parent_data.path_redirected_mods {
                    if redir_name == mod_name {
                        result.insert(parent_dir.join(file_path));
                    }
                }
            }

            // Also mark modules that have #[cfg] on their mod declaration
            // (new pattern: #[cfg(feat)] mod foo; — single declaration, no stub)
            for mod_name in parent_data.mod_decl_cfgs.keys() {
                result.insert(parent_dir.join(format!("{mod_name}.rs")));
                result.insert(parent_dir.join(mod_name).join("mod.rs"));
            }
        }

        result
    }

    /// Collect all types with ExternalPtr derive or TypedExternal impl across all files.
    pub fn all_typed_external_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        for data in self.file_data.values() {
            types.extend(data.types_with_external_ptr.iter().cloned());
            types.extend(data.types_with_typed_external.iter().cloned());
        }
        types
    }
}

// ── File collection (module-tree walker) ────────────────────────────────────

/// Collect Rust source files by walking the module tree from `lib.rs`,
/// following `mod child;` declarations and respecting `#[cfg(feature = "...")]`
/// gates via `CARGO_FEATURE_*` environment variables.
fn collect_rs_files_from_module_tree(src_dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let lib_rs = src_dir.join("lib.rs");
    if !lib_rs.is_file() {
        return Err(format!(
            "miniextendr-lint: cannot find lib.rs in {}",
            src_dir.display()
        ));
    }

    let active_features = collect_active_cargo_features();
    walk_module_file(&lib_rs, &active_features, out);
    Ok(())
}

/// Collect the set of active Cargo features from `CARGO_FEATURE_*` env vars.
/// Feature names are normalized: `CARGO_FEATURE_FOO_BAR` → `"foo-bar"`.
fn collect_active_cargo_features() -> HashSet<String> {
    std::env::vars()
        .filter_map(|(key, _)| {
            key.strip_prefix("CARGO_FEATURE_")
                .map(|suffix| suffix.to_lowercase().replace('_', "-"))
        })
        .collect()
}

/// Recursively walk a module file, following `mod` declarations.
fn walk_module_file(file: &Path, active_features: &HashSet<String>, out: &mut Vec<PathBuf>) {
    if !file.is_file() {
        return;
    }

    // Avoid duplicates (e.g., mod.rs referenced multiple ways)
    let file_buf = file.to_path_buf();
    if out.contains(&file_buf) {
        return;
    }

    out.push(file.to_path_buf());

    // Parse the file to discover mod declarations
    let Ok(src) = fs::read_to_string(file) else {
        return;
    };
    let Ok(parsed) = syn::parse_file(&src) else {
        return;
    };

    let parent_dir = match file.parent() {
        Some(dir) => dir,
        None => return,
    };

    // Determine the stem-based subdirectory for non-lib/mod files.
    // For `foo.rs`, child modules live in `foo/`.
    // For `lib.rs` or `mod.rs`, child modules live in the same directory.
    let child_dir = {
        let stem = file.file_stem().and_then(|s| s.to_str());
        match stem {
            Some("lib" | "mod") => parent_dir.to_path_buf(),
            Some(name) => parent_dir.join(name),
            None => parent_dir.to_path_buf(),
        }
    };

    discover_mod_declarations(&parsed.items, &child_dir, active_features, out);
}

/// Walk parsed items looking for `mod child;` declarations and recurse.
fn discover_mod_declarations(
    items: &[Item],
    child_dir: &Path,
    active_features: &HashSet<String>,
    out: &mut Vec<PathBuf>,
) {
    for item in items {
        let Item::Mod(item_mod) = item else {
            continue;
        };

        if let Some((_, child_items)) = &item_mod.content {
            // Inline module — recurse into its items (same file)
            discover_mod_declarations(child_items, child_dir, active_features, out);
        } else {
            // Out-of-line module declaration: `mod child;`
            // Check if cfg-gated and whether the gate is active
            let cfgs = extract_cfg_attrs(&item_mod.attrs);
            if !cfgs.is_empty() && !is_cfg_active(&cfgs, active_features) {
                continue; // Feature not enabled, skip this module
            }

            let mod_name = item_mod.ident.to_string();

            // Check for #[path = "file.rs"] attribute
            let path_attr = extract_path_attr(&item_mod.attrs);

            if let Some(file_path) = path_attr {
                let target = child_dir.join(&file_path);
                walk_module_file(&target, active_features, out);
            } else {
                // Try child.rs first, then child/mod.rs
                let sibling = child_dir.join(format!("{mod_name}.rs"));
                if sibling.is_file() {
                    walk_module_file(&sibling, active_features, out);
                } else {
                    let subdir_mod = child_dir.join(&mod_name).join("mod.rs");
                    walk_module_file(&subdir_mod, active_features, out);
                }
            }
        }
    }
}

/// Evaluate whether a set of `#[cfg(...)]` attributes is active given the current features.
///
/// Handles:
/// - `#[cfg(feature = "foo")]` → true if "foo" is in active_features
/// - `#[cfg(not(feature = "foo"))]` → true if "foo" is NOT in active_features
/// - Multiple cfg attrs → all must be satisfied (AND semantics, matching rustc)
///
/// Unknown cfg predicates (e.g., `cfg(target_os = "linux")`) are treated as active
/// (conservative: include the module rather than exclude it).
fn is_cfg_active(cfgs: &[String], active_features: &HashSet<String>) -> bool {
    for cfg_str in cfgs {
        if let Some(result) = eval_cfg_str(cfg_str, active_features)
            && !result
        {
            return false;
        }
        // Unknown cfg → treat as active (conservative)
    }
    true
}

/// Try to evaluate a single cfg string like `cfg(feature = "foo")` or `cfg(not(feature = "foo"))`.
/// The string comes from `syn::Meta::to_token_stream().to_string()` which may insert spaces
/// (e.g., `cfg (feature = "foo")` with a space after `cfg`).
/// Returns `Some(bool)` if it can evaluate, `None` if it can't parse the predicate.
fn eval_cfg_str(cfg_str: &str, active_features: &HashSet<String>) -> Option<bool> {
    // Normalize: remove all spaces to handle varying token stream formatting
    let normalized: String = cfg_str.chars().filter(|c| !c.is_whitespace()).collect();

    // Extract the inner content of cfg(...)
    let inner = normalized
        .strip_prefix("cfg(")
        .and_then(|s| s.strip_suffix(')'))?;

    // Handle `not(feature="foo")`
    if let Some(not_inner) = inner.strip_prefix("not(").and_then(|s| s.strip_suffix(')')) {
        if let Some(feat) = extract_feature_name(not_inner) {
            return Some(!active_features.contains(&feat));
        }
        return None; // Can't evaluate complex not()
    }

    // Handle `feature="foo"`
    if let Some(feat) = extract_feature_name(inner) {
        return Some(active_features.contains(&feat));
    }

    None // Unknown predicate
}

/// Extract the feature name from a string like `feature="foo"` (already whitespace-normalized).
fn extract_feature_name(s: &str) -> Option<String> {
    let rest = s.strip_prefix("feature")?;
    let rest = rest.strip_prefix('=')?;
    // Strip quotes: "foo" or \"foo\"
    let name = rest.trim_matches('"').trim_matches('\\');
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

// ── Single-file parsing ─────────────────────────────────────────────────────

fn parse_file(path: &Path) -> Result<FileData, String> {
    let src = fs::read_to_string(path)
        .map_err(|err| format!("{}: failed to read: {err}", path.display()))?;

    let parsed = syn::parse_file(&src)
        .map_err(|err| format!("{}: failed to parse: {err}", path.display()))?;

    let mut data = FileData::default();
    collect_items_recursive(&parsed.items, &mut data);

    // Scan raw source for direct Rf_error/Rf_errorcall calls (MXL300)
    scan_rf_error_calls(&src, &mut data);

    // Scan for ffi::*_unchecked() calls (MXL301)
    scan_ffi_unchecked_calls(&src, &mut data);

    Ok(data)
}

/// Recursively collect all lint-relevant information from parsed items.
fn collect_items_recursive(items: &[Item], data: &mut FileData) {
    for item in items {
        match item {
            Item::Fn(item_fn) => {
                // Track visibility for all functions with #[miniextendr]
                if has_miniextendr_attr(&item_fn.attrs) {
                    let line = item_fn.sig.ident.span().start().line;
                    let name = item_fn.sig.ident.to_string();

                    data.miniextendr_items.push(LintItem::new(
                        LintKind::Function,
                        name.clone(),
                        line,
                    ));

                    // Track visibility
                    let is_pub = matches!(item_fn.vis, syn::Visibility::Public(_));
                    data.fn_visibility.insert(name.clone(), is_pub);

                    // Track cfg attrs
                    let cfgs = extract_cfg_attrs(&item_fn.attrs);
                    if !cfgs.is_empty() {
                        data.item_cfgs.insert(format!("fn:{}", name), cfgs);
                    }

                    // Track export control
                    let attrs = parse_miniextendr_impl_attrs(&item_fn.attrs);
                    if attrs.internal || attrs.noexport {
                        data.export_control
                            .insert(name.clone(), (attrs.internal, attrs.noexport));
                    }

                    // Track doc-comment roxygen tags
                    let doc_tags = extract_roxygen_tags(&item_fn.attrs);
                    if !doc_tags.is_empty() {
                        data.fn_doc_tags.insert(name, doc_tags);
                    }
                }
            }
            Item::Struct(item_struct) => {
                // Only 1-field structs without explicit mode attrs are ALTREP
                // (need `struct Name;` in miniextendr_module!). Multi-field structs
                // and structs with list/dataframe/externalptr attrs generate derives
                // that don't need module entries.
                // ALTREP structs via #[miniextendr] or #[derive(Altrep)]
                let is_miniextendr_altrep =
                    has_miniextendr_attr(&item_struct.attrs) && is_altrep_struct(item_struct);
                let is_derive_altrep = has_altrep_derive(&item_struct.attrs);
                if is_miniextendr_altrep || is_derive_altrep {
                    let line = item_struct.ident.span().start().line;
                    data.miniextendr_items.push(LintItem::new(
                        LintKind::Struct,
                        item_struct.ident.to_string(),
                        line,
                    ));
                }
                if has_external_ptr_derive(&item_struct.attrs) {
                    data.types_with_external_ptr
                        .insert(item_struct.ident.to_string());
                }
                if has_vctrs_derive(&item_struct.attrs) {
                    let line = item_struct.ident.span().start().line;
                    data.miniextendr_items.push(LintItem::new(
                        LintKind::Vctrs,
                        item_struct.ident.to_string(),
                        line,
                    ));
                }
            }
            Item::Impl(item_impl) => {
                // Check for impl TypedExternal for Type
                if let Some((_, trait_path, _)) = &item_impl.trait_
                    && let Some(last_seg) = trait_path.segments.last()
                    && last_seg.ident == "TypedExternal"
                    && let Some(type_name) = impl_type_name(&item_impl.self_ty)
                {
                    data.types_with_typed_external.insert(type_name);
                }

                if has_miniextendr_attr(&item_impl.attrs) {
                    let line = item_impl.self_ty.span().start().line;
                    let impl_attrs = parse_miniextendr_impl_attrs(&item_impl.attrs);

                    match impl_type_name(&item_impl.self_ty) {
                        Some(type_name) => {
                            if let Some((_, trait_path, _)) = &item_impl.trait_ {
                                // Trait impl
                                if let Some(trait_seg) = trait_path.segments.last() {
                                    let trait_name = trait_seg.ident.to_string();
                                    let full_name = format!("{} for {}", trait_name, type_name);
                                    data.miniextendr_items.push(LintItem::new(
                                        LintKind::TraitImpl,
                                        full_name,
                                        line,
                                    ));
                                    data.attributed_trait_impls.push(AttributedTraitImpl {
                                        type_name: type_name.clone(),
                                        trait_name,
                                        class_system: impl_attrs.class_system.clone(),
                                        line,
                                    });
                                }
                            } else {
                                // Inherent impl
                                data.inherent_impl_class_systems.insert(
                                    type_name.clone(),
                                    (impl_attrs.class_system.clone().unwrap_or_default(), line),
                                );
                                data.impl_blocks_per_type
                                    .entry(type_name.clone())
                                    .or_default()
                                    .push((impl_attrs.label.clone(), line));
                                data.miniextendr_items.push(LintItem::with_label(
                                    LintKind::Impl,
                                    type_name.clone(),
                                    impl_attrs.label.clone(),
                                    line,
                                ));

                                // Track cfg attrs
                                let cfgs = extract_cfg_attrs(&item_impl.attrs);
                                let key = match &impl_attrs.label {
                                    Some(label) => format!("impl:{}:{}", type_name, label),
                                    None => format!("impl:{}", type_name),
                                };
                                if !cfgs.is_empty() {
                                    data.item_cfgs.insert(key, cfgs);
                                }

                                // Track export control
                                if impl_attrs.internal || impl_attrs.noexport {
                                    data.export_control.insert(
                                        type_name,
                                        (impl_attrs.internal, impl_attrs.noexport),
                                    );
                                }
                            }
                        }
                        None => { /* unsupported impl type, skip */ }
                    }
                }
            }
            Item::Macro(item_macro) => {
                if is_miniextendr_module_macro(&item_macro.mac) {
                    let line = item_macro.mac.path.span().start().line;
                    data.module_macro_lines.push(line);

                    if let Ok(parsed) = syn::parse2::<miniextendr_module::MiniextendrModule>(
                        item_macro.mac.tokens.clone(),
                    ) {
                        // Module name
                        data.module_name = Some(parsed.module_name.ident.to_string());

                        // Uses
                        for use_entry in &parsed.uses {
                            let name = use_entry.use_name.ident.to_string();
                            let cfgs = extract_cfg_attrs(&use_entry.attrs);
                            if !cfgs.is_empty() {
                                data.use_entry_cfgs.insert(name.clone(), cfgs);
                            }
                            data.module_uses.push(name);
                        }

                        // Functions
                        for func in &parsed.functions {
                            let fn_line = func.ident.span().start().line;
                            let name = func.ident.to_string();
                            data.module_items.push(LintItem::new(
                                LintKind::Function,
                                name.clone(),
                                fn_line,
                            ));
                            let cfgs = extract_cfg_attrs(&func.attrs);
                            if !cfgs.is_empty() {
                                data.module_entry_cfgs.insert(format!("fn:{}", name), cfgs);
                            }
                        }

                        // Structs
                        for strukt in &parsed.structs {
                            let s_line = strukt.ident.span().start().line;
                            data.module_items.push(LintItem::new(
                                LintKind::Struct,
                                strukt.ident.to_string(),
                                s_line,
                            ));
                        }

                        // Vctrs
                        for vctrs in &parsed.vctrs {
                            let v_line = vctrs.ident.span().start().line;
                            data.module_items.push(LintItem::new(
                                LintKind::Vctrs,
                                vctrs.ident.to_string(),
                                v_line,
                            ));
                        }

                        // Impls
                        for impl_block in &parsed.impls {
                            let i_line = impl_block.ident.span().start().line;
                            let name = impl_block.ident.to_string();
                            data.module_items.push(LintItem::with_label(
                                LintKind::Impl,
                                name.clone(),
                                impl_block.label.clone(),
                                i_line,
                            ));
                            data.impl_type_entries.push((name.clone(), i_line));
                            let cfgs = extract_cfg_attrs(&impl_block.attrs);
                            let key = match &impl_block.label {
                                Some(label) => {
                                    format!("impl:{}:{}", name, label)
                                }
                                None => format!("impl:{}", name),
                            };
                            if !cfgs.is_empty() {
                                data.module_entry_cfgs.insert(key, cfgs);
                            }
                        }

                        // Trait impls
                        for trait_impl in &parsed.trait_impls {
                            let t_line = trait_impl.impl_type.span().start().line;
                            let trait_name = trait_impl
                                .trait_path
                                .segments
                                .last()
                                .map(|s| s.ident.to_string())
                                .unwrap_or_default();
                            let type_name = trait_impl.type_name_sanitized();
                            let full_name = format!("{} for {}", trait_name, type_name);
                            data.module_items.push(LintItem::new(
                                LintKind::TraitImpl,
                                full_name,
                                t_line,
                            ));

                            let is_generic = trait_impl.simple_type_ident().is_none();
                            data.trait_impl_entries.push(TraitImplEntry {
                                trait_name: trait_name.clone(),
                                type_name: type_name.clone(),
                                type_name_sanitized: trait_impl.type_name_sanitized(),
                                is_generic,
                                line: t_line,
                                cfgs: extract_cfg_attrs(&trait_impl.attrs),
                            });
                        }
                    }
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, child_items)) = &item_mod.content {
                    // Inline module
                    collect_items_recursive(child_items, data);
                } else {
                    // Out-of-line module declaration
                    let mod_name = item_mod.ident.to_string();

                    // Track cfg attrs on the mod declaration
                    let cfgs = extract_cfg_attrs(&item_mod.attrs);
                    if !cfgs.is_empty() {
                        data.mod_decl_cfgs.insert(mod_name.clone(), cfgs);
                    }

                    // Check for #[path = "file.rs"] attribute
                    let path_attr = extract_path_attr(&item_mod.attrs);
                    if let Some(file_path) = path_attr {
                        data.path_redirected_mods.push((mod_name, file_path));
                    } else {
                        data.declared_child_mods.push(mod_name);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Patterns that indicate direct Rf_error/Rf_errorcall calls in user code.
const RF_ERROR_PATTERNS: &[&str] = &[
    "Rf_error(",
    "Rf_error_unchecked(",
    "Rf_errorcall(",
    "Rf_errorcall_unchecked(",
];

/// Check if a lint code is suppressed via `// mxl::allow(MXL...)` comment.
///
/// Checks the current line for a trailing `// mxl::allow(MXLnnn)` comment,
/// and also checks the immediately preceding line for a standalone
/// `// mxl::allow(MXLnnn)` comment. Multiple codes can be comma-separated:
/// `// mxl::allow(MXL300, MXL301)`.
fn is_suppressed(lines: &[&str], line_idx: usize, code: &str) -> bool {
    // Check current line for trailing comment
    if line_has_allow(lines[line_idx], code) {
        return true;
    }
    // Check preceding line for standalone allow comment
    if line_idx > 0 && line_has_allow(lines[line_idx - 1], code) {
        return true;
    }
    false
}

/// Check if a single line contains `// mxl::allow(...)` matching the given code.
fn line_has_allow(line: &str, code: &str) -> bool {
    const PREFIX: &str = "// mxl::allow(";
    if let Some(pos) = line.find(PREFIX) {
        let after = &line[pos + PREFIX.len()..];
        if let Some(end) = after.find(')') {
            let codes = &after[..end];
            return codes.split(',').any(|c| c.trim() == code);
        }
    }
    false
}

/// Scan raw source text for `ffi::*_unchecked()` calls.
fn scan_ffi_unchecked_calls(src: &str, data: &mut FileData) {
    let lines: Vec<&str> = src.lines().collect();
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Skip comment lines
        if trimmed.starts_with("//") {
            continue;
        }
        // Skip #[link_name] attributes (generated by #[r_ffi_checked])
        if trimmed.starts_with("#[") {
            continue;
        }
        // Find ffi::*_unchecked( patterns
        let mut search_from = 0;
        while let Some(ffi_pos) = trimmed[search_from..].find("ffi::") {
            let abs_pos = search_from + ffi_pos;
            let after_ffi = &trimmed[abs_pos + 5..];
            // Extract identifier after "ffi::"
            let ident_end = after_ffi
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(after_ffi.len());
            let ident = &after_ffi[..ident_end];
            if ident.ends_with("_unchecked")
                && after_ffi[ident_end..].starts_with('(')
                && !is_suppressed(&lines, line_idx, "MXL301")
            {
                data.ffi_unchecked_calls
                    .push((ident.to_string(), line_idx + 1));
            }
            search_from = abs_pos + 5 + ident_end;
        }
    }
}

/// Scan raw source text for direct Rf_error/Rf_errorcall calls.
fn scan_rf_error_calls(src: &str, data: &mut FileData) {
    let lines: Vec<&str> = src.lines().collect();
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Skip comment lines
        if trimmed.starts_with("//") {
            continue;
        }
        for pattern in RF_ERROR_PATTERNS {
            if trimmed.contains(pattern) && !is_suppressed(&lines, line_idx, "MXL300") {
                let fn_name = &pattern[..pattern.len() - 1];
                data.rf_error_calls
                    .push((fn_name.to_string(), line_idx + 1));
            }
        }
    }
}
