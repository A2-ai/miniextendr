//! Shared crate index built from a single parse pass over all source files.
//!
//! All lint rules operate on this index rather than re-parsing files.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use syn::Item;
use syn::spanned::Spanned;

use crate::helpers::{
    extract_cfg_attrs, extract_path_attr, extract_roxygen_tags, has_altrep_derive,
    has_external_ptr_derive, has_miniextendr_attr, has_vctrs_derive, impl_type_name,
    is_altrep_struct, parse_miniextendr_impl_attrs,
};

// region: Impl method entry

/// Receiver kind for an impl method, mirroring `ReceiverKind` in `miniextendr-macros`.
///
/// Mirror: `miniextendr-macros/src/miniextendr_impl.rs` — `ReceiverKind`.
/// Keep both in sync: if the macro relaxes one receiver kind, update this enum too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MethodReceiverKind {
    /// No self — static / associated function.
    None,
    /// `&self`
    Ref,
    /// `&mut self`
    RefMut,
    /// `self` (consuming)
    Value,
    /// `self: &ExternalPtr<Self>`
    ExternalPtrRef,
    /// `self: &mut ExternalPtr<Self>`
    ExternalPtrRefMut,
    /// `self: ExternalPtr<Self>`
    ExternalPtrValue,
}

impl MethodReceiverKind {
    /// Returns true if this is an instance receiver (any form of `self`).
    ///
    /// Mirrors `ReceiverKind::is_instance` in `miniextendr-macros/src/miniextendr_impl.rs`.
    /// `Value` (consuming `self`) is **excluded** — the macro treats consuming-`self` methods
    /// separately: they are either constructors (`returns Self` or `#[miniextendr(constructor)]`)
    /// or finalizers, not ordinary instance calls.  Including `Value` here would produce a
    /// false-positive for a vctrs method with `#[miniextendr(constructor)]` that consumes `self`.
    pub fn is_instance(self) -> bool {
        matches!(
            self,
            Self::Ref
                | Self::RefMut
                | Self::ExternalPtrRef
                | Self::ExternalPtrRefMut
                | Self::ExternalPtrValue
        )
    }

    /// Human-readable spelling used in diagnostic messages.
    pub fn spelling(self) -> &'static str {
        match self {
            Self::None => "(none)",
            Self::Ref => "&self",
            Self::RefMut => "&mut self",
            Self::Value => "self",
            Self::ExternalPtrRef => "self: &ExternalPtr<Self>",
            Self::ExternalPtrRefMut => "self: &mut ExternalPtr<Self>",
            Self::ExternalPtrValue => "self: ExternalPtr<Self>",
        }
    }
}

/// Per-method data collected during the crate-index pass for impl-method lint rules.
#[derive(Clone, Debug)]
pub struct ImplMethodEntry {
    pub method_name: String,
    pub line: usize,
    pub class_system: String,
    /// Stringified return type tokens (empty string = `()` / no explicit return).
    pub return_type_str: String,
    /// Receiver kind detected from the method signature.
    pub receiver_kind: MethodReceiverKind,
    /// True when the method carries `#[miniextendr(constructor)]`.
    pub has_constructor_attr: bool,
}

// endregion

// region: Lint item types

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
}
// endregion

// region: Attributed trait impls from source

#[derive(Clone, Debug)]
pub struct AttributedTraitImpl {
    pub type_name: String,
    pub trait_name: String,
    pub class_system: Option<String>,
    pub line: usize,
}
// endregion

// region: Per-file parsed data

#[derive(Debug, Default)]
pub struct FileData {
    // Source items (functions, impls, structs with #[miniextendr])
    pub miniextendr_items: Vec<LintItem>,

    // Type/derive information
    pub types_with_external_ptr: HashSet<String>,
    pub types_with_typed_external: HashSet<String>,

    // Impl block details
    pub inherent_impl_class_systems: HashMap<String, (String, usize)>,
    pub attributed_trait_impls: Vec<AttributedTraitImpl>,
    pub impl_blocks_per_type: HashMap<String, Vec<(Option<String>, usize)>>,

    // Function details
    pub fn_visibility: HashMap<String, bool>,

    // Module tree (for file discovery)
    /// Simple `mod child;` declarations (by ident name).
    pub declared_child_mods: Vec<String>,
    /// `#[path = "file.rs"] mod name;` declarations: (mod_name, file_path_str).
    pub path_redirected_mods: Vec<(String, String)>,
    /// cfg attrs on `mod child;` declarations: mod_name -> cfg strings.
    pub mod_decl_cfgs: HashMap<String, Vec<String>>,

    // Export control
    /// (has_internal, has_noexport, line)
    pub export_control: HashMap<String, (bool, bool, usize)>,

    // Impl method details for per-method lint rules
    /// Methods per inherent impl type: type_name → `Vec<ImplMethodEntry>`.
    pub impl_methods: HashMap<String, Vec<ImplMethodEntry>>,

    // Doc-comment roxygen tags per function/impl name
    /// Known roxygen tags: "@noRd", "@export", "@keywords internal"
    pub fn_doc_tags: HashMap<String, Vec<String>>,

    // Safety lint data
    /// Lines containing direct Rf_error/Rf_errorcall calls: (function_name, line_number).
    pub rf_error_calls: Vec<(String, usize)>,
    /// Lines containing `ffi::*_unchecked()` calls: (function_name, line_number).
    pub ffi_unchecked_calls: Vec<(String, usize)>,

    // R reserved-word parameter names
    /// Maps fn/method name → list of (param_name, line) for params that are R reserved words.
    /// Key for free functions is the function name; for impl methods it is `"TypeName::method_name"`.
    pub fn_param_names: HashMap<String, Vec<(String, usize)>>,

    // Lifetime parameter lint (MXL112)
    /// `#[miniextendr]` functions or impl blocks that carry explicit lifetime params.
    /// Each entry is `(name, line)` where `name` is the function or type name.
    pub lifetime_param_items: Vec<(String, usize)>,
}
// endregion

// region: Crate index

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
}
// endregion

// region: File collection (module-tree walker)

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
    let mut seen = HashSet::new();
    walk_module_file(&lib_rs, &active_features, out, &mut seen);
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
fn walk_module_file(
    file: &Path,
    active_features: &HashSet<String>,
    out: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) {
    if !file.is_file() {
        return;
    }

    let file_buf = file.to_path_buf();
    if !seen.insert(file_buf.clone()) {
        return;
    }

    out.push(file_buf);

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

    discover_mod_declarations(&parsed.items, &child_dir, active_features, out, seen);
}

/// Walk parsed items looking for `mod child;` declarations and recurse.
fn discover_mod_declarations(
    items: &[Item],
    child_dir: &Path,
    active_features: &HashSet<String>,
    out: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) {
    for item in items {
        let Item::Mod(item_mod) = item else {
            continue;
        };

        if let Some((_, child_items)) = &item_mod.content {
            // Inline module — recurse into its items (same file)
            discover_mod_declarations(child_items, child_dir, active_features, out, seen);
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
                walk_module_file(&target, active_features, out, seen);
            } else {
                // Try child.rs first, then child/mod.rs
                let sibling = child_dir.join(format!("{mod_name}.rs"));
                if sibling.is_file() {
                    walk_module_file(&sibling, active_features, out, seen);
                } else {
                    let subdir_mod = child_dir.join(&mod_name).join("mod.rs");
                    walk_module_file(&subdir_mod, active_features, out, seen);
                }
            }
        }
    }
}

/// Evaluate whether a set of `#[cfg(...)]` attributes is active given the current features.
fn is_cfg_active(cfgs: &[String], active_features: &HashSet<String>) -> bool {
    for cfg_str in cfgs {
        if let Some(result) = eval_cfg_str(cfg_str, active_features)
            && !result
        {
            return false;
        }
    }
    true
}

/// Try to evaluate a single cfg string like `cfg(feature = "foo")`.
fn eval_cfg_str(cfg_str: &str, active_features: &HashSet<String>) -> Option<bool> {
    let normalized: String = cfg_str.chars().filter(|c| !c.is_whitespace()).collect();

    let inner = normalized
        .strip_prefix("cfg(")
        .and_then(|s| s.strip_suffix(')'))?;

    if let Some(not_inner) = inner.strip_prefix("not(").and_then(|s| s.strip_suffix(')')) {
        if let Some(feat) = extract_feature_name(not_inner) {
            return Some(!active_features.contains(&feat));
        }
        return None;
    }

    if let Some(feat) = extract_feature_name(inner) {
        return Some(active_features.contains(&feat));
    }

    None
}

/// Extract the feature name from a string like `feature="foo"`.
fn extract_feature_name(s: &str) -> Option<String> {
    let rest = s.strip_prefix("feature")?;
    let rest = rest.strip_prefix('=')?;
    let name = rest.trim_matches('"').trim_matches('\\');
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
// endregion

// region: Single-file parsing

fn parse_file(path: &Path) -> Result<FileData, String> {
    let src = fs::read_to_string(path)
        .map_err(|err| format!("{}: failed to read: {err}", path.display()))?;

    let parsed = syn::parse_file(&src)
        .map_err(|err| format!("{}: failed to parse: {err}", path.display()))?;

    let mut data = FileData::default();
    collect_items_recursive(&parsed.items, &mut data);

    // Both raw-source scanners need the line-split for is_suppressed look-behind.
    let lines: Vec<&str> = src.lines().collect();
    scan_rf_error_calls(&lines, &mut data);
    scan_ffi_unchecked_calls(&lines, &mut data);

    Ok(data)
}

/// Extract named parameter names (and their 1-based line numbers) from a function signature.
///
/// Skips `self` / `&self` / `&mut self` receiver parameters. Skips unnamed (`_`) parameters.
fn extract_param_names(sig: &syn::Signature) -> Vec<(String, usize)> {
    let mut params = Vec::new();
    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input
            && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
        {
            let name = pat_ident.ident.to_string();
            // Skip `_` (bare anonymous). Named `_foo` patterns are kept because
            // the proc-macro forwards the name verbatim (stripping only the leading
            // underscore in some codegen paths), so they can still collide with R
            // reserved words.
            if name == "_" {
                continue;
            }
            let line = pat_ident.ident.span().start().line;
            params.push((name, line));
        }
    }
    params
}

/// Recursively collect all lint-relevant information from parsed items.
fn collect_items_recursive(items: &[Item], data: &mut FileData) {
    for item in items {
        match item {
            Item::Fn(item_fn) if has_miniextendr_attr(&item_fn.attrs) => {
                let line = item_fn.sig.ident.span().start().line;
                let name = item_fn.sig.ident.to_string();

                data.miniextendr_items
                    .push(LintItem::new(LintKind::Function, name.clone(), line));

                // Track visibility
                let is_pub = matches!(item_fn.vis, syn::Visibility::Public(_));
                data.fn_visibility.insert(name.clone(), is_pub);

                // Track export control
                let attrs = parse_miniextendr_impl_attrs(&item_fn.attrs);
                if attrs.internal || attrs.noexport {
                    data.export_control
                        .insert(name.clone(), (attrs.internal, attrs.noexport, line));
                }

                // Track doc-comment roxygen tags
                let doc_tags = extract_roxygen_tags(&item_fn.attrs);
                if !doc_tags.is_empty() {
                    data.fn_doc_tags.insert(name.clone(), doc_tags);
                }

                // Track parameter names for R reserved-word check (MXL110)
                let params = extract_param_names(&item_fn.sig);
                if !params.is_empty() {
                    data.fn_param_names.insert(name.clone(), params);
                }

                // Track explicit lifetime params for MXL112
                let has_lifetime = item_fn
                    .sig
                    .generics
                    .params
                    .iter()
                    .any(|p| matches!(p, syn::GenericParam::Lifetime(_)));
                if has_lifetime {
                    data.lifetime_param_items.push((name.clone(), line));
                }
            }
            Item::Struct(item_struct) => {
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

                    // Track explicit lifetime params on impl blocks for MXL112
                    let impl_has_lifetime = item_impl
                        .generics
                        .params
                        .iter()
                        .any(|p| matches!(p, syn::GenericParam::Lifetime(_)));
                    if impl_has_lifetime && let Some(type_name) = impl_type_name(&item_impl.self_ty)
                    {
                        data.lifetime_param_items.push((type_name, line));
                    }

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
                                let class_system =
                                    impl_attrs.class_system.clone().unwrap_or_default();
                                data.inherent_impl_class_systems
                                    .insert(type_name.clone(), (class_system.clone(), line));
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

                                // Collect method names for per-method rules (e.g. MXL111, MXL120)
                                let methods =
                                    data.impl_methods.entry(type_name.clone()).or_default();
                                for impl_item in &item_impl.items {
                                    if let syn::ImplItem::Fn(method) = impl_item {
                                        let method_name = method.sig.ident.to_string();
                                        let method_line = method.sig.ident.span().start().line;
                                        let return_type_str =
                                            extract_return_type_str(&method.sig.output);
                                        let receiver_kind = detect_receiver_kind(&method.sig);
                                        let has_constructor_attr =
                                            has_constructor_attr(&method.attrs);
                                        methods.push(ImplMethodEntry {
                                            method_name,
                                            line: method_line,
                                            class_system: class_system.clone(),
                                            return_type_str,
                                            receiver_kind,
                                            has_constructor_attr,
                                        });
                                    }
                                }

                                // Track export control
                                if impl_attrs.internal || impl_attrs.noexport {
                                    data.export_control.insert(
                                        type_name.clone(),
                                        (impl_attrs.internal, impl_attrs.noexport, line),
                                    );
                                }
                            }

                            // Track parameter names for all methods in the impl block (MXL110)
                            for impl_item in &item_impl.items {
                                if let syn::ImplItem::Fn(method) = impl_item {
                                    let method_name = method.sig.ident.to_string();
                                    let key = format!("{}::{}", type_name, method_name);
                                    let params = extract_param_names(&method.sig);
                                    if !params.is_empty() {
                                        data.fn_param_names.insert(key, params);
                                    }
                                }
                            }
                        }
                        None => { /* unsupported impl type, skip */ }
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
fn is_suppressed(lines: &[&str], line_idx: usize, code: &str) -> bool {
    if line_has_allow(lines[line_idx], code) {
        return true;
    }
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

/// Scan raw source text for `sys::*_unchecked()` calls.
fn scan_ffi_unchecked_calls(lines: &[&str], data: &mut FileData) {
    const PREFIX: &str = "sys::";
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("#[") {
            continue;
        }
        // Strip inline comments to avoid false positives
        let code_part = match trimmed.find("//") {
            Some(pos) => &trimmed[..pos],
            None => trimmed,
        };
        let mut search_from = 0;
        while let Some(sys_pos) = code_part[search_from..].find(PREFIX) {
            let abs_pos = search_from + sys_pos;
            let after = &code_part[abs_pos + PREFIX.len()..];
            let ident_end = after
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(after.len());
            let ident = &after[..ident_end];
            if ident.ends_with("_unchecked")
                && after[ident_end..].starts_with('(')
                && !is_suppressed(lines, line_idx, "MXL301")
            {
                data.ffi_unchecked_calls
                    .push((ident.to_string(), line_idx + 1));
            }
            search_from = abs_pos + PREFIX.len() + ident_end;
        }
    }
}

// region: Impl method helpers (MXL120 and future per-method rules)

/// Stringify a `syn::ReturnType` to a compact token string.
///
/// Returns an empty string for `-> ()` / no explicit return (both mean unit).
fn extract_return_type_str(output: &syn::ReturnType) -> String {
    use quote::ToTokens;
    match output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
    }
}

/// Detect the receiver kind from a method signature.
///
/// Mirror: `miniextendr-macros/src/miniextendr_impl.rs` — `detect_receiver_kind`.
/// Keep both in sync: if the macro adds a new receiver variant, update this function too.
fn detect_receiver_kind(sig: &syn::Signature) -> MethodReceiverKind {
    let first = match sig.inputs.first() {
        Some(arg) => arg,
        None => return MethodReceiverKind::None,
    };
    match first {
        syn::FnArg::Receiver(recv) => {
            // syn 2.x parses *all* `self` receiver forms as `FnArg::Receiver`, including
            // the typed forms `self: &ExternalPtr<Self>`, `self: &mut ExternalPtr<Self>`,
            // and `self: ExternalPtr<Self>`.  When a colon token is present the receiver
            // has an explicit type in `recv.ty`; otherwise `recv.reference` / `recv.mutability`
            // describe the shorthand `(&)(&mut) self`.
            if recv.colon_token.is_some() {
                // Typed form: `self: <ty>`.  Classify by inspecting `recv.ty`.
                match recv.ty.as_ref() {
                    syn::Type::Reference(r) => {
                        if is_external_ptr_self_ty(r.elem.as_ref()) {
                            if r.mutability.is_some() {
                                MethodReceiverKind::ExternalPtrRefMut
                            } else {
                                MethodReceiverKind::ExternalPtrRef
                            }
                        } else if r.mutability.is_some() {
                            MethodReceiverKind::RefMut
                        } else {
                            MethodReceiverKind::Ref
                        }
                    }
                    ty if is_external_ptr_self_ty(ty) => MethodReceiverKind::ExternalPtrValue,
                    _ => MethodReceiverKind::None,
                }
            } else {
                // Shorthand form: `self`, `&self`, `&mut self`.
                if recv.mutability.is_some() {
                    MethodReceiverKind::RefMut
                } else if recv.reference.is_some() {
                    MethodReceiverKind::Ref
                } else {
                    MethodReceiverKind::Value
                }
            }
        }
        syn::FnArg::Typed(_) => {
            // In syn 2.x, typed `self:` forms are represented as `FnArg::Receiver`, so
            // this arm is only reached for genuinely non-`self` parameters.
            MethodReceiverKind::None
        }
    }
}

/// Returns true if `ty` is `ExternalPtr<Self>` (last path segment = `ExternalPtr`,
/// single type argument = `Self`).
fn is_external_ptr_self_ty(ty: &syn::Type) -> bool {
    let syn::Type::Path(p) = ty else {
        return false;
    };
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    if last.ident != "ExternalPtr" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(ref args) = last.arguments else {
        return false;
    };
    matches!(
        args.args.first(),
        Some(syn::GenericArgument::Type(syn::Type::Path(tp)))
            if tp.path.is_ident("Self")
    )
}

/// Returns true when the attribute list contains `#[miniextendr(constructor)]` or
/// `#[miniextendr(r6(constructor))]` / `#[miniextendr(s3(constructor))]` etc.
fn has_constructor_attr(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr
            .path()
            .segments
            .last()
            .is_none_or(|seg| seg.ident != "miniextendr")
        {
            continue;
        }
        if let syn::Meta::List(meta_list) = &attr.meta {
            let tokens = meta_list.tokens.to_string();
            // Accept both `constructor` at top level and inside `r6(...)`, `s3(...)`, etc.
            if tokens
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|t| t == "constructor")
            {
                return true;
            }
        }
    }
    false
}

// endregion

/// Scan raw source text for direct Rf_error/Rf_errorcall calls.
fn scan_rf_error_calls(lines: &[&str], data: &mut FileData) {
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        // Strip inline comments to avoid false positives
        let code_part = match trimmed.find("//") {
            Some(pos) => &trimmed[..pos],
            None => trimmed,
        };
        for pattern in RF_ERROR_PATTERNS {
            if code_part.contains(pattern) && !is_suppressed(lines, line_idx, "MXL300") {
                let fn_name = &pattern[..pattern.len() - 1];
                data.rf_error_calls
                    .push((fn_name.to_string(), line_idx + 1));
            }
        }
    }
}
// endregion
