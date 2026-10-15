//! Shared crate index built from a single parse pass over all source files.
//!
//! All lint rules operate on this index rather than re-parsing files.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use syn::Item;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use crate::crate_root::CrateRoot;
use crate::description::{self, Description};
use crate::doc_links::{self, PkgLink};
use crate::helpers::{
    extract_cfg_attrs, extract_path_attr, extract_roxygen_tags, has_derive, has_miniextendr_attr,
    impl_type_name, is_altrep_struct, parse_miniextendr_impl_attrs,
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
    /// `self: ExternalPtrRef<'_, Self>`
    ExternalPtrRef,
    /// `self: ExternalPtrMut<'_, Self>`
    ExternalPtrMut,
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
                | Self::ExternalPtrMut
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
            Self::ExternalPtrRef => "self: ExternalPtrRef<'_, Self>",
            Self::ExternalPtrMut => "self: ExternalPtrMut<'_, Self>",
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

/// An inherent `#[miniextendr]` impl block, for the rules that look at a type
/// across all its impl blocks (MXL121).
#[derive(Clone, Debug)]
pub struct InherentImpl {
    pub type_name: String,
    pub class_system: String,
    /// 1-based line of the `impl` keyword.
    pub line: usize,
    /// 1-based line of the block's first attribute or doc comment (`line`
    /// without any).
    pub first_line: usize,
    /// True when `// mxl::allow(MXL121)` is on the block's header: one of
    /// its attribute and doc lines, the `impl` line, or the line above them.
    pub allows_mxl121: bool,
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
    pub line: usize,
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
    pub fn new(kind: LintKind, name: String, line: usize) -> Self {
        Self { kind, name, line }
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
    /// Every inherent `#[miniextendr]` impl block, in source order. (MXL121)
    pub inherent_impls: Vec<InherentImpl>,
    /// The type names a `#[miniextendr]` fn or method returns, bare, as
    /// `Self` or `ExternalPtr<T>`, or inside `Result` / `Option`. A method
    /// with a receiver doesn't count for its own type, whose object it needs
    /// already. (MXL121)
    pub returned_types: HashSet<String>,
    /// The type names the file builds `ExternalPtr`s of itself, in any item,
    /// `#[miniextendr]` or not (`HandBuilt`). (MXL121)
    pub hand_built_types: HashSet<String>,

    // Doc-comment roxygen tags per function/impl name
    /// Known roxygen tags: "@noRd", "@export", "@keywords internal"
    pub fn_doc_tags: HashMap<String, Vec<String>>,

    // Safety lint data
    /// Lines containing direct Rf_error/Rf_errorcall calls: (function_name, line_number).
    pub rf_error_calls: Vec<(String, usize)>,
    /// Lines containing `ffi::*_unchecked()` calls: (function_name, line_number).
    pub ffi_unchecked_calls: Vec<(String, usize)>,
    /// `into_sexp()` / `into_sexp_unchecked()` calls that appear *inside* a `vec!`/array
    /// literal: (call_name, line_number). This is the use-after-free idiom — each later
    /// `into_sexp` allocates and can GC an earlier, still-unprotected element of the same
    /// literal. (MXL302)
    pub vec_into_sexp_calls: Vec<(String, usize)>,
    /// Calls to, or declarations of, R's non-API `ATTRIB` / `SET_ATTRIB` (or their
    /// `_unchecked` forms): (function_name, line_number). (MXL304)
    pub nonapi_attrib_calls: Vec<(String, usize)>,
    /// The bodies the generated `#[miniextendr]` boundary runs: of `#[miniextendr]`
    /// fns, of the methods of `#[miniextendr]` impl blocks (inherent and trait), and
    /// the default methods of `#[miniextendr]` traits, as 1-based `(open, close)`
    /// brace lines. (MXL305)
    pub miniextendr_bodies: Vec<(usize, usize)>,
    /// Calls to `Rf_eval` / `R_forceAndCall` / `Rf_applyClosure` / `R_CheckUserInterrupt`
    /// (or their `_unchecked` forms) inside [`miniextendr_bodies`](Self::miniextendr_bodies):
    /// (function_name, line_number). (MXL305)
    pub raw_r_eval_calls: Vec<(String, usize)>,

    /// `pkg::topic` links in the roxygen-bound doc text of `#[miniextendr]`
    /// items. (MXL204)
    pub doc_pkg_links: Vec<PkgLink>,

    // R reserved-word parameter names
    /// Maps fn/method name → list of (param_name, line) for params that are R reserved words.
    /// Key for free functions is the function name; for impl methods it is `"TypeName::method_name"`.
    pub fn_param_names: HashMap<String, Vec<(String, usize)>>,
}
// endregion

// region: Crate index

/// Shared parsed state for all lint rules.
pub struct CrateIndex {
    /// All scanned Rust source files.
    pub files: Vec<PathBuf>,
    /// Per-file parsed data.
    pub file_data: HashMap<PathBuf, FileData>,
    /// The R package's `DESCRIPTION`, when the crate sits at `<pkg>/src/rust`
    /// of one ([`description::locate`]).
    pub description: Option<Description>,
    /// `roxygen_prose_links = "keep"` in `[package.metadata.miniextendr]`:
    /// leading prose then reaches roxygen2 with its links.
    pub prose_links_keep: bool,
}

/// Why [`CrateIndex::build`] failed, with the source files it had found.
#[derive(Debug)]
pub struct IndexError {
    /// What went wrong (every parse error, joined with `; `).
    pub message: String,
    /// The module-tree files found before the failure. A file that fails to
    /// parse is among them, so a build script can still watch it (#1738).
    /// Empty when the crate root file is missing or `[lib] path` is unreadable.
    pub files: Vec<PathBuf>,
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<String> for IndexError {
    fn from(message: String) -> Self {
        Self {
            message,
            files: Vec::new(),
        }
    }
}

impl CrateIndex {
    /// Build the index for the crate whose `Cargo.toml` sits in `manifest_dir`,
    /// walking the module tree from its root file ([`CrateRoot::resolve`]).
    pub fn build(manifest_dir: &Path) -> Result<Self, IndexError> {
        let root = CrateRoot::resolve(manifest_dir)?;

        let mut rs_files = Vec::new();
        collect_rs_files_from_module_tree(&root, &mut rs_files)?;
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
            return Err(IndexError {
                message: parse_errors.join("; "),
                files: rs_files,
            });
        }

        Ok(Self {
            files: rs_files,
            file_data,
            description: description::read(manifest_dir),
            prose_links_keep: crate::crate_root::prose_links_keep(manifest_dir),
        })
    }
}
// endregion

// region: File collection (module-tree walker)

/// Collect Rust source files by walking the module tree from the crate root,
/// following `mod child;` declarations and respecting `#[cfg(feature = "...")]`
/// gates via `CARGO_FEATURE_*` environment variables.
fn collect_rs_files_from_module_tree(
    root: &CrateRoot,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !root.file.is_file() {
        return Err(format!(
            "miniextendr-lint: cannot find crate root {} ({})",
            root.file.display(),
            root.origin
        ));
    }

    let active_features = collect_active_cargo_features();
    let mut seen = HashSet::new();
    walk_module_file(&root.file, None, &active_features, out, &mut seen);
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

/// Where the `mod` declarations of one scope resolve, tracked as rustc does
/// (`rustc_expand::module`).
struct ModDir {
    /// The directory `#[path]` attributes are relative to: the declaring file's
    /// own directory, extended by the enclosing inline modules.
    dir: PathBuf,
    /// The module name a non-mod-rs file adds before its child files: `mod b;`
    /// in `src/a.rs` is `src/a/b.rs`. `None` for a mod-rs file (the crate root,
    /// whatever its name, a `mod.rs`, or a file reached through `#[path]`) and
    /// inside inline modules.
    relative: Option<String>,
}

impl ModDir {
    /// The directory a plain `mod child;` looks in for `child.rs` / `child/mod.rs`.
    fn child_dir(&self) -> PathBuf {
        match &self.relative {
            Some(name) => self.dir.join(name),
            None => self.dir.clone(),
        }
    }
}

/// Recursively walk a module file, following `mod` declarations.
///
/// `relative` is the module name when `file` is a non-mod-rs file (see
/// [`ModDir::relative`]).
fn walk_module_file(
    file: &Path,
    relative: Option<String>,
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

    let Some(dir) = file.parent() else {
        return;
    };
    let scope = ModDir {
        dir: dir.to_path_buf(),
        relative,
    };

    discover_mod_declarations(&parsed.items, &scope, active_features, out, seen);
}

/// Walk parsed items looking for `mod child;` declarations and recurse.
fn discover_mod_declarations(
    items: &[Item],
    scope: &ModDir,
    active_features: &HashSet<String>,
    out: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) {
    for item in items {
        let Item::Mod(item_mod) = item else {
            continue;
        };

        // A cfg-gated module, inline or not, is visited only when its gate is active.
        let cfgs = extract_cfg_attrs(&item_mod.attrs);
        if !cfgs.is_empty() && !is_cfg_active(&cfgs, active_features) {
            continue;
        }

        // `mod r#type;` lives in `type.rs`.
        let mod_name = item_mod.ident.unraw().to_string();
        let path_attr = extract_path_attr(&item_mod.attrs);

        match (&item_mod.content, path_attr) {
            // `#[path = "dir"] mod name { ... }`: the attribute names the directory.
            (Some((_, child_items)), Some(dir)) => {
                let inner = ModDir {
                    dir: scope.dir.join(dir),
                    relative: None,
                };
                discover_mod_declarations(child_items, &inner, active_features, out, seen);
            }
            // `mod name { ... }`: its `mod` declarations look in `name/`.
            (Some((_, child_items)), None) => {
                let inner = ModDir {
                    dir: scope.child_dir().join(&mod_name),
                    relative: None,
                };
                discover_mod_declarations(child_items, &inner, active_features, out, seen);
            }
            // `#[path = "file.rs"] mod name;`: relative to the declaring scope's
            // directory, and the file is a mod-rs file.
            (None, Some(file_path)) => {
                let target = scope.dir.join(file_path);
                walk_module_file(&target, None, active_features, out, seen);
            }
            // `mod name;`: `name.rs`, else `name/mod.rs`.
            (None, None) => {
                let child_dir = scope.child_dir();
                let sibling = child_dir.join(format!("{mod_name}.rs"));
                if sibling.is_file() {
                    walk_module_file(&sibling, Some(mod_name), active_features, out, seen);
                } else {
                    let subdir_mod = child_dir.join(&mod_name).join("mod.rs");
                    walk_module_file(&subdir_mod, None, active_features, out, seen);
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

    let parsed = syn::parse_file(&src).map_err(|err| {
        let mut message = format!("{}: failed to parse: {err}", path.display());
        if let Some(hint) = crate::misplaced_dots::hint(&src) {
            message.push_str("; ");
            message.push_str(&hint);
        }
        message
    })?;

    let mut data = FileData::default();
    collect_items_recursive(&parsed.items, &mut data);
    let mut hand_built = HandBuilt::default();
    hand_built.visit_file(&parsed);
    data.hand_built_types = hand_built.types;

    // Both raw-source scanners need the line-split for is_suppressed look-behind.
    let lines: Vec<&str> = src.lines().collect();
    scan_rf_error_calls(&lines, &mut data);
    scan_ffi_unchecked_calls(&lines, &mut data);
    scan_vec_into_sexp_calls(&lines, &mut data);
    scan_nonapi_attrib_calls(&lines, &mut data);
    scan_raw_r_eval_calls(&lines, &mut data);
    for block in &mut data.inherent_impls {
        block.allows_mxl121 = header_allows(&lines, block.first_line, block.line, "MXL121");
    }

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

/// Record the `pkg::topic` links in the docs of a `#[miniextendr]` item and,
/// for an impl block, of its methods.
///
/// A block that renders no help page hands roxygen2 no links: the macro
/// strips every one of them there (#1818). That is a block with its own
/// `@noRd`, a `noexport` fn, and an impl block (with all its methods) under
/// `@noRd` or a plain `noexport`, by the macro's rules: `internal`, or on an
/// inherent impl the `@keywords internal` tag, keeps the page.
fn collect_doc_pkg_links(item: &Item, data: &mut FileData) {
    let attrs: &[syn::Attribute] = match item {
        Item::Fn(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        _ => return,
    };
    if !has_miniextendr_attr(attrs) {
        return;
    }
    let flags = parse_miniextendr_impl_attrs(attrs);
    let no_page = match item {
        Item::Fn(_) => flags.noexport || doc_links::has_tag(attrs, "noRd"),
        Item::Impl(item_impl) => {
            doc_links::has_tag(attrs, "noRd")
                || (flags.noexport
                    && !flags.internal
                    && (item_impl.trait_.is_some()
                        || !doc_links::has_tag(attrs, "keywords internal")))
        }
        _ => false,
    };
    if no_page {
        return;
    }
    data.doc_pkg_links.extend(doc_links::pkg_links(attrs));
    if let Item::Impl(item_impl) = item {
        for impl_item in &item_impl.items {
            if let syn::ImplItem::Fn(method) = impl_item
                && !doc_links::has_tag(&method.attrs, "noRd")
            {
                data.doc_pkg_links
                    .extend(doc_links::pkg_links(&method.attrs));
            }
        }
    }
}

/// Recursively collect all lint-relevant information from parsed items.
fn collect_items_recursive(items: &[Item], data: &mut FileData) {
    for item in items {
        collect_doc_pkg_links(item, data);
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

                if let syn::ReturnType::Type(_, ty) = &item_fn.sig.output {
                    collect_returned_types(ty, None, &mut data.returned_types);
                }

                data.miniextendr_bodies.push(block_lines(&item_fn.block));
            }
            Item::Trait(item_trait) if has_miniextendr_attr(&item_trait.attrs) => {
                // A default method runs when an impl doesn't override it (MXL305)
                for trait_item in &item_trait.items {
                    if let syn::TraitItem::Fn(method) = trait_item
                        && let Some(block) = &method.default
                    {
                        data.miniextendr_bodies.push(block_lines(block));
                    }
                }
            }
            Item::Struct(item_struct) => {
                let is_miniextendr_altrep =
                    has_miniextendr_attr(&item_struct.attrs) && is_altrep_struct(item_struct);
                let is_derive_altrep = has_derive(&item_struct.attrs, "Altrep");
                if is_miniextendr_altrep || is_derive_altrep {
                    let line = item_struct.ident.span().start().line;
                    data.miniextendr_items.push(LintItem::new(
                        LintKind::Struct,
                        item_struct.ident.to_string(),
                        line,
                    ));
                }
                if has_derive(&item_struct.attrs, "ExternalPtr") {
                    data.types_with_external_ptr
                        .insert(item_struct.ident.to_string());
                }
                if has_derive(&item_struct.attrs, "Vctrs") {
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

                    // Method bodies, inherent and trait (MXL305)
                    for impl_item in &item_impl.items {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            data.miniextendr_bodies.push(block_lines(&method.block));
                        }
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
                                data.miniextendr_items.push(LintItem::new(
                                    LintKind::Impl,
                                    type_name.clone(),
                                    line,
                                ));
                                let impl_line = item_impl.impl_token.span.start().line;
                                data.inherent_impls.push(InherentImpl {
                                    type_name: type_name.clone(),
                                    class_system: class_system.clone(),
                                    line: impl_line,
                                    first_line: item_impl
                                        .attrs
                                        .first()
                                        .map_or(impl_line, |attr| attr.span().start().line),
                                    allows_mxl121: false,
                                });

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

                            // The types its methods return (MXL121)
                            for impl_item in &item_impl.items {
                                let syn::ImplItem::Fn(method) = impl_item else {
                                    continue;
                                };
                                let syn::ReturnType::Type(_, ty) = &method.sig.output else {
                                    continue;
                                };
                                let mut returned = HashSet::new();
                                collect_returned_types(ty, Some(&type_name), &mut returned);
                                if detect_receiver_kind(&method.sig) != MethodReceiverKind::None {
                                    returned.remove(&type_name);
                                }
                                data.returned_types.extend(returned);
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

/// Whether `// mxl::allow(<code>)` is on an item's header: its lines
/// `first_line..=line` (1-based: attributes, doc comments and the item's own
/// first line) or the line above them.
fn header_allows(lines: &[&str], first_line: usize, line: usize, code: &str) -> bool {
    (first_line.saturating_sub(1).max(1)..=line).any(|n| {
        lines
            .get(n - 1)
            .is_some_and(|text| line_has_allow(text, code))
    })
}

/// Add to `out` the type `ty` hands to R as a class object: its last path
/// segment, `self_type` for `Self`, through `Result`, `Option` and
/// `ExternalPtr`. Other names (`i32`, `Vec`, ...) are added too; they match no
/// class.
fn collect_returned_types(ty: &syn::Type, self_type: Option<&str>, out: &mut HashSet<String>) {
    let syn::Type::Path(path) = ty else {
        return;
    };
    let Some(last) = path.path.segments.last() else {
        return;
    };
    if ["Result", "Option", "ExternalPtr"]
        .iter()
        .any(|wrapper| last.ident == wrapper)
    {
        if let syn::PathArguments::AngleBracketed(args) = &last.arguments
            && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
        {
            collect_returned_types(inner, self_type, out);
        }
    } else {
        out.extend(path_type_name(&path.path, self_type));
    }
}

/// The type `path` names: its last segment, `self_type` for `Self`.
fn path_type_name(path: &syn::Path, self_type: Option<&str>) -> Option<String> {
    let last = path.segments.last()?;
    if last.ident == "Self" {
        self_type.map(str::to_string)
    } else {
        Some(last.ident.to_string())
    }
}

/// The types a file builds `ExternalPtr`s of itself (MXL121), in any item:
///
/// - the struct in `ExternalPtr::new(T { .. })` / `ExternalPtr::new(T(..))`
///   (or `new_unchecked`), with `Self` inside `impl T`, or in
///   `ExternalPtr::new(x)` after `let x = T { .. }` / `let x = T(..)` /
///   `let x: T = ..` in the same function body (the last `let x` wins);
/// - `T` in an `ExternalPtr<T>` written as a type or a turbofish
///   (`ExternalPtr::<T>::new(..)`): return types, `let` annotations, fields,
///   generic arguments.
///
/// A function's parameters, its receiver among them, are objects the caller
/// has already, and are skipped. So is the impl's own type in a method of a
/// `#[miniextendr]` impl block that takes `self`, as for
/// [`FileData::returned_types`]. A value from a call whose type isn't written
/// (`let record = make_record(inner); ExternalPtr::new(record)`), a value
/// from another function, and code inside macro calls (`vec![..]`) stay
/// invisible.
#[derive(Default)]
struct HandBuilt {
    types: HashSet<String>,
    /// The type `Self` names here: the enclosing impl's.
    self_type: Option<String>,
    /// Inside a `#[miniextendr]` impl block.
    miniextendr_impl: bool,
    /// The type of each `let` binding of the current function body whose
    /// value is a struct literal or whose annotation names it.
    locals: HashMap<String, String>,
}

impl HandBuilt {
    /// Visit a function with bindings of its own.
    fn in_fn(&mut self, visit_fn: impl FnOnce(&mut Self)) {
        let locals = std::mem::take(&mut self.locals);
        visit_fn(self);
        self.locals = locals;
    }

    /// The type a struct literal `T { .. }` or tuple-struct call `T(..)` builds.
    fn literal_type(&self, expr: &syn::Expr) -> Option<String> {
        let path = match expr {
            syn::Expr::Struct(literal) => &literal.path,
            syn::Expr::Call(call) => match &*call.func {
                syn::Expr::Path(ctor) => &ctor.path,
                _ => return None,
            },
            _ => return None,
        };
        path_type_name(path, self.self_type.as_deref())
    }
}

impl<'ast> Visit<'ast> for HandBuilt {
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let self_type = std::mem::replace(&mut self.self_type, impl_type_name(&item.self_ty));
        let miniextendr_impl = std::mem::replace(
            &mut self.miniextendr_impl,
            has_miniextendr_attr(&item.attrs),
        );
        visit::visit_item_impl(self, item);
        self.self_type = self_type;
        self.miniextendr_impl = miniextendr_impl;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.in_fn(|this| visit::visit_item_fn(this, item));
    }

    fn visit_trait_item_fn(&mut self, method: &'ast syn::TraitItemFn) {
        self.in_fn(|this| visit::visit_trait_item_fn(this, method));
    }

    fn visit_impl_item_fn(&mut self, method: &'ast syn::ImplItemFn) {
        if !self.miniextendr_impl || detect_receiver_kind(&method.sig) == MethodReceiverKind::None {
            self.in_fn(|this| visit::visit_impl_item_fn(this, method));
            return;
        }
        let outer = std::mem::take(&mut self.types);
        self.in_fn(|this| visit::visit_impl_item_fn(this, method));
        let mut built = std::mem::replace(&mut self.types, outer);
        if let Some(self_type) = &self.self_type {
            built.remove(self_type);
        }
        self.types.extend(built);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        visit::visit_local(self, local);
        let (pat, annotated) = match &local.pat {
            syn::Pat::Type(typed) => {
                let annotated = match &*typed.ty {
                    syn::Type::Path(ty) => path_type_name(&ty.path, self.self_type.as_deref()),
                    _ => None,
                };
                (&*typed.pat, annotated)
            }
            pat => (pat, None),
        };
        let syn::Pat::Ident(binding) = pat else {
            return;
        };
        let name = binding.ident.to_string();
        let ty = annotated.or_else(|| {
            local
                .init
                .as_ref()
                .and_then(|init| self.literal_type(&init.expr))
        });
        match ty {
            Some(ty) => self.locals.insert(name, ty),
            None => self.locals.remove(&name),
        };
    }

    fn visit_fn_arg(&mut self, _: &'ast syn::FnArg) {}

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(func) = &*call.func
            && is_external_ptr_new(&func.path)
            && let Some(arg) = call.args.first()
        {
            let built = match arg {
                syn::Expr::Path(local) => local
                    .path
                    .get_ident()
                    .and_then(|ident| self.locals.get(&ident.to_string()).cloned()),
                literal => self.literal_type(literal),
            };
            self.types.extend(built);
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        for segment in &path.segments {
            if segment.ident == "ExternalPtr"
                && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
            {
                collect_returned_types(inner, self.self_type.as_deref(), &mut self.types);
            }
        }
        visit::visit_path(self, path);
    }
}

/// Whether `path` ends in `ExternalPtr::new` or `ExternalPtr::new_unchecked`,
/// with or without a turbofish.
fn is_external_ptr_new(path: &syn::Path) -> bool {
    let mut segments = path.segments.iter().rev();
    segments
        .next()
        .is_some_and(|last| last.ident == "new" || last.ident == "new_unchecked")
        && segments
            .next()
            .is_some_and(|segment| segment.ident == "ExternalPtr")
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
            // the typed forms `self: ExternalPtrRef<'_, Self>`, `self: ExternalPtrMut<'_, Self>`
            // and `self: ExternalPtr<Self>`.  When a colon token is present the receiver
            // has an explicit type in `recv.ty`; otherwise `recv.reference` / `recv.mutability`
            // describe the shorthand `(&)(&mut) self`.
            if recv.colon_token.is_some() {
                // Typed form: `self: <ty>`.  Classify by inspecting `recv.ty`.
                match recv.ty.as_ref() {
                    syn::Type::Reference(r) if r.mutability.is_some() => MethodReceiverKind::RefMut,
                    syn::Type::Reference(_) => MethodReceiverKind::Ref,
                    ty => match self_handle_ty(ty) {
                        Some("ExternalPtr") => MethodReceiverKind::ExternalPtrValue,
                        Some("ExternalPtrRef") => MethodReceiverKind::ExternalPtrRef,
                        Some("ExternalPtrMut") => MethodReceiverKind::ExternalPtrMut,
                        _ => MethodReceiverKind::None,
                    },
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

/// The handle `ty` names over `Self`: `"ExternalPtr"` for `ExternalPtr<Self>`,
/// `"ExternalPtrRef"` / `"ExternalPtrMut"` for the views (`ExternalPtrRef<'_,
/// Self>`), matched on the last path segment with a `Self` type argument.
fn self_handle_ty(ty: &syn::Type) -> Option<&'static str> {
    let syn::Type::Path(p) = ty else {
        return None;
    };
    let last = p.path.segments.last()?;
    let name = ["ExternalPtr", "ExternalPtrRef", "ExternalPtrMut"]
        .into_iter()
        .find(|name| last.ident == name)?;
    let syn::PathArguments::AngleBracketed(ref args) = last.arguments else {
        return None;
    };
    args.args
        .iter()
        .any(|arg| {
            matches!(arg, syn::GenericArgument::Type(syn::Type::Path(tp)) if tp.path.is_ident("Self"))
        })
        .then_some(name)
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

// region: MXL304 — non-API `ATTRIB` / `SET_ATTRIB`

/// R's non-API attribute-pairlist accessors. `R CMD check` (R >= 4.6) reports them.
const NONAPI_ATTRIB_FNS: &[&str] = &["ATTRIB", "SET_ATTRIB"];

/// Scan raw source text for calls to, or declarations of, `ATTRIB` / `SET_ATTRIB` and
/// their `_unchecked` forms.
///
/// Only a whole identifier followed by `(` counts, so the API entry points that share
/// the suffix (`ANY_ATTRIB`, `CLEAR_ATTRIB`, `DUPLICATE_ATTRIB`,
/// `SHALLOW_DUPLICATE_ATTRIB`) are not flagged. A declaration in an `extern` block
/// (`fn ATTRIB(x: SEXP) -> SEXP;`) is flagged too, because that is how the symbol
/// reaches the package's shared object.
fn scan_nonapi_attrib_calls(lines: &[&str], data: &mut FileData) {
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
        for ident in called_idents(code_part) {
            let base = ident.strip_suffix("_unchecked").unwrap_or(ident);
            if NONAPI_ATTRIB_FNS.contains(&base) && !is_suppressed(lines, line_idx, "MXL304") {
                data.nonapi_attrib_calls
                    .push((ident.to_string(), line_idx + 1));
            }
        }
    }
}

/// The whole identifiers in `code` that a `(` follows (after optional whitespace):
/// the names of the functions it calls or declares. `DUPLICATE_ATTRIB(` yields
/// `DUPLICATE_ATTRIB`, never `ATTRIB`.
fn called_idents(code: &str) -> Vec<&str> {
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';
    let mut idents = Vec::new();
    let mut chars = code.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        if !is_ident_char(c) {
            continue;
        }
        let mut end = start + c.len_utf8();
        while let Some(&(idx, next)) = chars.peek() {
            if !is_ident_char(next) {
                break;
            }
            end = idx + next.len_utf8();
            chars.next();
        }
        if code[end..].trim_start().starts_with('(') {
            idents.push(&code[start..end]);
        }
    }
    idents
}

// endregion

// region: MXL305 — raw R evaluation in a `#[miniextendr]` body

/// R entry points that run R code, or R's interrupt handling, and longjmp out on an
/// R error or a pending interrupt.
const RAW_R_EVAL_FNS: &[&str] = &[
    "Rf_eval",
    "R_forceAndCall",
    "Rf_applyClosure",
    "R_CheckUserInterrupt",
];

/// The 1-based lines of `block`'s `{` and `}`.
fn block_lines(block: &syn::Block) -> (usize, usize) {
    let span = &block.brace_token.span;
    (span.open().start().line, span.close().end().line)
}

/// Scan the [`FileData::miniextendr_bodies`] for calls to `Rf_eval`,
/// `R_forceAndCall`, `Rf_applyClosure` and `R_CheckUserInterrupt`, and their
/// `_unchecked` forms,
/// through any path (`sys::Rf_eval(`, `miniextendr_api::sys::Rf_eval(`, an
/// imported `Rf_eval(`).
///
/// The generated wrapper runs such a body inside one `R_UnwindProtect`, holding
/// the receiver's borrow guard. An R error or interrupt raised by one of these
/// calls jumps to that `R_UnwindProtect` over every Rust frame of the body: no
/// destructor runs, and the receiver stays borrowed for the rest of the session
/// (#2093). Wrapping the call in `with_r_unwind_protect` (or its `_or_raise`
/// form) doesn't help: it resumes the R exit with `R_ContinueUnwind`, which jumps
/// over the body's frames all the same. So no context inside a body is exempt;
/// the `expression` evaluators (`RCall::eval`, `eval_with_handlers`, ...) are the
/// way to run R code there.
///
/// Raw text, like MXL304: a call inside a macro invocation is seen, a helper fn
/// the body calls is not (only the bodies themselves are scanned).
fn scan_raw_r_eval_calls(lines: &[&str], data: &mut FileData) {
    let mut in_body = vec![false; lines.len()];
    for &(open, close) in &data.miniextendr_bodies {
        for flag in in_body.iter_mut().take(close).skip(open - 1) {
            *flag = true;
        }
    }
    for (line_idx, line) in lines.iter().enumerate() {
        if !in_body[line_idx] {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        // Strip inline comments to avoid false positives
        let code_part = match trimmed.find("//") {
            Some(pos) => &trimmed[..pos],
            None => trimmed,
        };
        for ident in called_idents(code_part) {
            let base = ident.strip_suffix("_unchecked").unwrap_or(ident);
            if RAW_R_EVAL_FNS.contains(&base) && !is_suppressed(lines, line_idx, "MXL305") {
                data.raw_r_eval_calls
                    .push((ident.to_string(), line_idx + 1));
            }
        }
    }
}

// endregion

// region: MXL302 — `into_sexp()` inside a `vec!`/array literal (use-after-free idiom)

/// `into_sexp` call spellings that build a fresh, unprotected SEXP.
const INTO_SEXP_CALLS: &[&str] = &["into_sexp(", "into_sexp_unchecked("];

/// Scan raw source text for `into_sexp()` / `into_sexp_unchecked()` calls that appear
/// *inside* a `vec!` or `&[...]` literal — the use-after-free idiom (#307, #1025).
///
/// Each `into_sexp` allocates a new SEXP. When several appear as elements of one literal
/// (`vec![(k, a.into_sexp()), (k, b.into_sexp())]`), building `b` can trigger a GC that
/// collects the still-unprotected `a` (and vice versa), because nothing roots the earlier
/// elements until the whole `Vec` is handed to `List::from_raw_pairs`. The fix is to route
/// each element through the protected builder path (`__scope.protect_raw(x.into_sexp())`
/// with a `ProtectScope`), exactly as the `IntoList` / `DataFrameRow` derives now do.
///
/// # Heuristic and scope
///
/// This is a deliberately narrow raw-text scanner (consistent with MXL300/MXL301):
/// it tracks bracket depth opened by a `vec![` or `&[` literal and flags an `into_sexp(`
/// call only while that depth is open. This is what makes it precise:
///
/// - **Flags** `vec![ ... into_sexp() ... ]` — the call is an element *inside* the literal.
/// - **Does NOT flag** `vec![1, 2, 3].into_sexp()` — the `]` closes the literal before the
///   `.into_sexp()` call, so depth is back to 0 (the whole `Vec` is converted as one SEXP,
///   which is safe — there are no sibling unprotected SEXPs).
///
/// The protected builder path is treated as a true negative: an `into_sexp(` whose element
/// routes through a `protect_raw(...)` / `protect(...)` / `protect_with_index(...)` call
/// (`__scope.protect_raw(x.into_sexp())`) is *not* flagged. This is exactly the form the
/// `IntoList` / `DataFrameRow` derives emit, and the recommended hand-written fix.
///
/// Known limits (false negatives, by design — to keep zero false positives):
/// - Only `vec![` and `&[` literal opens are tracked; bare `[ ... ]` array literals are
///   not, because a leading `[` is ambiguous with indexing (`arr[i]`). Array-literal sites
///   are rare in this codebase; promote the scanner if one appears.
/// - The protect-detection is per-element and line-local in spirit: an element whose
///   `protect(...)`/`protect_raw(...)` wrapper sits on an earlier line than its `into_sexp(`
///   is still recognised (the flag persists until the next element-separating `,`), but a
///   contrived element that opens a protect call yet smuggles an *unprotected* sibling
///   `into_sexp(` after the same comma-free span would be missed. No such shape exists in
///   the corpus.
fn scan_vec_into_sexp_calls(lines: &[&str], data: &mut FileData) {
    // Bracket depth opened specifically by a `vec![` / `&[` literal. Nested `[` inside the
    // literal increment it too; `]` decrements. We never let it go negative.
    let mut literal_depth: i32 = 0;
    // Whether the *current literal element* already routes its value through a protect call
    // (`protect_raw(...)` / `protect(...)`). Reset at each element-separating `,` and at
    // each literal open. This is what makes the protected builder path (the form emitted by
    // the `IntoList` / `DataFrameRow` derives) a true negative.
    let mut element_protected = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        // Strip inline `//` comments so commented-out idioms / doc prose don't trip us.
        let code_part = match line.find("//") {
            Some(pos) => &line[..pos],
            None => *line,
        };

        let bytes = code_part.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            // Detect a `vec![` / `&[` literal open (look-behind).
            if bytes[i] == b'[' {
                // Byte-slice matching: `i` walks bytes, so `&str` slicing would
                // panic mid-codepoint on non-ASCII source (e.g. a `"α"` literal).
                let opens_literal =
                    bytes[..i].ends_with(b"vec!") || (i > 0 && bytes[i - 1] == b'&');
                if opens_literal || literal_depth > 0 {
                    // Either a fresh literal open, or a nested `[` while already inside one.
                    literal_depth += 1;
                    element_protected = false;
                }
                i += 1;
                continue;
            }
            if bytes[i] == b']' {
                if literal_depth > 0 {
                    literal_depth -= 1;
                }
                i += 1;
                continue;
            }
            if literal_depth > 0 {
                // An element separator resets the per-element protected flag.
                if bytes[i] == b',' {
                    element_protected = false;
                    i += 1;
                    continue;
                }
                // A `protect_raw(` / `protect(` call before `into_sexp(` in this element
                // means the value is rooted as it is built — the protected builder path.
                if bytes[i..].starts_with(b"protect_raw(")
                    || bytes[i..].starts_with(b"protect(")
                    || bytes[i..].starts_with(b"protect_with_index(")
                {
                    element_protected = true;
                }
                // Flag a raw `into_sexp(` element call (not wrapped in a protect call).
                for pattern in INTO_SEXP_CALLS {
                    if bytes[i..].starts_with(pattern.as_bytes())
                        && !element_protected
                        && !is_suppressed(lines, line_idx, "MXL302")
                    {
                        let call_name = &pattern[..pattern.len() - 1];
                        data.vec_into_sexp_calls
                            .push((call_name.to_string(), line_idx + 1));
                    }
                }
            }
            i += 1;
        }
    }
}
// endregion
// endregion
