//! Build script for miniextendr-api
//!
//! Sets appropriate stack size linker flags for R-compatible binaries.
//! This affects tests, examples, and cdylib crates that depend on miniextendr-api.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use syn::parse::Parse;
use syn::{Attribute, Item, Macro, Token};

fn main() {
    // Only set stack size flags when nonapi feature is enabled
    // (since that's where thread utilities live)
    #[cfg(feature = "nonapi")]
    set_stack_size_flags();

    // Ensure rebuild on feature changes
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_NONAPI");

    if let Err(message) = run_miniextendr_lint() {
        panic!("{message}");
    }
}

#[cfg(feature = "nonapi")]
fn set_stack_size_flags() {
    // R requires larger stacks than Rust's default 2 MiB:
    // - Unix: typically 8 MiB
    // - Windows: 64 MiB since R 4.2
    //
    // We set 8 MiB as a reasonable default that works on all platforms.
    // Users needing Windows R's full 64 MiB can override via .cargo/config.toml.
    const STACK_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    match (target_os.as_str(), target_env.as_str()) {
        // Windows MSVC: /STACK:size
        ("windows", "msvc") => {
            println!("cargo::rustc-link-arg=/STACK:{STACK_SIZE}");
        }
        // Windows GNU (MinGW): --stack,size
        ("windows", "gnu") => {
            println!("cargo::rustc-link-arg=-Wl,--stack,{STACK_SIZE}");
        }
        // macOS: -stack_size (requires hex value)
        ("macos", _) => {
            println!("cargo::rustc-link-arg=-Wl,-stack_size,{STACK_SIZE:x}");
        }
        // Linux and other Unix: -z stack-size
        ("linux", _) | ("freebsd", _) | ("netbsd", _) | ("openbsd", _) => {
            println!("cargo::rustc-link-arg=-Wl,-z,stack-size={STACK_SIZE}");
        }
        // Unknown platform - skip
        _ => {}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum LintKind {
    Function,
    Impl,
    Struct,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct LintItem {
    kind: LintKind,
    name: String,
}

impl LintItem {
    fn new(kind: LintKind, name: String) -> Self {
        Self { kind, name }
    }

    fn display(&self) -> String {
        let prefix = match self.kind {
            LintKind::Function => "fn",
            LintKind::Impl => "impl",
            LintKind::Struct => "struct",
        };
        format!("{prefix} {}", self.name)
    }
}

fn run_miniextendr_lint() -> Result<(), String> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").map_err(|err| format!("env: {err}"))?,
    );
    let src_dir = manifest_dir.join("src");
    if !src_dir.is_dir() {
        return Ok(());
    }

    let mut rs_files = Vec::new();
    collect_rs_files(&src_dir, &mut rs_files)
        .map_err(|err| format!("miniextendr-lint: failed to read src: {err}"))?;

    let mut errors = Vec::new();
    for path in rs_files {
        println!("cargo::rerun-if-changed={}", path.display());
        if let Err(mut file_errors) = lint_file(&path) {
            errors.append(&mut file_errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let mut message = String::from("miniextendr-lint failed:\n");
        for err in errors {
            message.push_str("- ");
            message.push_str(&err);
            message.push('\n');
        }
        Err(message)
    }
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn lint_file(path: &Path) -> Result<(), Vec<String>> {
    let src = match fs::read_to_string(path) {
        Ok(src) => src,
        Err(err) => return Err(vec![format!("{}: failed to read: {err}", path.display())]),
    };

    let parsed = match syn::parse_file(&src) {
        Ok(parsed) => parsed,
        Err(err) => {
            return Err(vec![format!(
                "{}: failed to parse: {err}",
                path.display()
            )]);
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

    let mut missing: Vec<_> = miniextendr_items
        .iter()
        .filter(|item| !module_items.contains(*item))
        .map(|item| item.display())
        .collect();
    missing.sort();
    if !missing.is_empty() {
        errors.push(format!(
            "{}: #[miniextendr] items not listed in miniextendr_module!: {}",
            path.display(),
            missing.join(", ")
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_items(
    items: &[Item],
    path: &Path,
    miniextendr_items: &mut HashSet<LintItem>,
    module_items: &mut HashSet<LintItem>,
    errors: &mut Vec<String>,
) {
    for item in items {
        match item {
            Item::Fn(item_fn) => {
                if has_miniextendr_attr(&item_fn.attrs) {
                    miniextendr_items.insert(LintItem::new(
                        LintKind::Function,
                        item_fn.sig.ident.to_string(),
                    ));
                }
            }
            Item::Struct(item_struct) => {
                if has_miniextendr_attr(&item_struct.attrs) {
                    miniextendr_items.insert(LintItem::new(
                        LintKind::Struct,
                        item_struct.ident.to_string(),
                    ));
                }
            }
            Item::Impl(item_impl) => {
                if has_miniextendr_attr(&item_impl.attrs) {
                    match impl_type_name(&item_impl.self_ty) {
                        Some(name) => {
                            miniextendr_items.insert(LintItem::new(LintKind::Impl, name));
                        }
                        None => errors.push(format!(
                            "{}: #[miniextendr] impl type not supported by lint",
                            path.display()
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
}

fn has_miniextendr_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map_or(false, |seg| seg.ident == "miniextendr")
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
        .map_or(false, |seg| seg.ident == "miniextendr_module")
}

fn parse_miniextendr_module_items(mac: &Macro) -> syn::Result<Vec<LintItem>> {
    let parsed = syn::parse2::<MiniextendrModuleLite>(mac.tokens.clone())?;
    Ok(parsed.items)
}

struct MiniextendrModuleLite {
    items: Vec<LintItem>,
}

impl Parse for MiniextendrModuleLite {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let items: syn::punctuated::Punctuated<MiniextendrModuleItemLite, Token![;]> =
            syn::punctuated::Punctuated::parse_terminated_with(
                input,
                MiniextendrModuleItemLite::parse,
            )?;
        let mut out = Vec::new();

        for item in items {
            match item {
                MiniextendrModuleItemLite::Func(ident) => {
                    out.push(LintItem::new(LintKind::Function, ident.to_string()));
                }
                MiniextendrModuleItemLite::Struct(ident) => {
                    out.push(LintItem::new(LintKind::Struct, ident.to_string()));
                }
                MiniextendrModuleItemLite::Impl(ident) => {
                    out.push(LintItem::new(LintKind::Impl, ident.to_string()));
                }
                MiniextendrModuleItemLite::Module | MiniextendrModuleItemLite::Use => {}
            }
        }

        Ok(Self { items: out })
    }
}

enum MiniextendrModuleItemLite {
    Module,
    Use,
    Struct(syn::Ident),
    Func(syn::Ident),
    Impl(syn::Ident),
}

impl Parse for MiniextendrModuleItemLite {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _attrs = input.call(Attribute::parse_outer)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Token![mod]) {
            let _mod_token: syn::Token![mod] = input.parse()?;
            let _ident: syn::Ident = input.parse()?;
            Ok(Self::Module)
        } else if lookahead.peek(syn::Token![use]) {
            let _use_token: syn::Token![use] = input.parse()?;
            let _use_tree: syn::UseTree = input.parse()?;
            Ok(Self::Use)
        } else if lookahead.peek(syn::Token![struct]) {
            let _struct_token: syn::Token![struct] = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            Ok(Self::Struct(ident))
        } else if lookahead.peek(syn::Token![impl]) {
            let _impl_token: syn::Token![impl] = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            Ok(Self::Impl(ident))
        } else if lookahead.peek(syn::Token![fn]) || lookahead.peek(syn::Token![extern]) {
            if input.peek(syn::Token![extern]) {
                let _abi: syn::Abi = input.parse()?;
            }
            let _fn_token: syn::Token![fn] = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            Ok(Self::Func(ident))
        } else {
            Err(lookahead.error())
        }
    }
}
