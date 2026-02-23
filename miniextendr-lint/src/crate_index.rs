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
    extract_cfg_attrs, extract_path_attr, extract_roxygen_tags, has_external_ptr_derive,
    has_miniextendr_attr, impl_type_name, is_miniextendr_module_macro,
    parse_miniextendr_impl_attrs, should_skip_dir,
};

// ── Lint item types ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LintKind {
    Function,
    Impl,
    Struct,
    TraitImpl,
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
        collect_rs_files(&src_dir, &mut rs_files)
            .map_err(|err| format!("miniextendr-lint: failed to read src: {err}"))?;
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

// ── File collection ─────────────────────────────────────────────────────────

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

// ── Single-file parsing ─────────────────────────────────────────────────────

fn parse_file(path: &Path) -> Result<FileData, String> {
    let src = fs::read_to_string(path)
        .map_err(|err| format!("{}: failed to read: {err}", path.display()))?;

    let parsed = syn::parse_file(&src)
        .map_err(|err| format!("{}: failed to parse: {err}", path.display()))?;

    let mut data = FileData::default();
    collect_items_recursive(&parsed.items, &mut data);
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
                if has_miniextendr_attr(&item_struct.attrs) {
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
