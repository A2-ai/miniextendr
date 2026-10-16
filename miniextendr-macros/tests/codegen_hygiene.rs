//! Guard every generated branch against consumer-side standard-name shadows.
//!
//! Compile fixtures exercise behavior; this source audit also covers branches
//! no fixture expands, without enabling mutually exclusive Cargo features.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use std::path::{Path, PathBuf};
use syn::visit_mut::{self, VisitMut};

const PRELUDE_NAMES: &[&str] = &[
    "AsMut",
    "AsRef",
    "AsyncFn",
    "AsyncFnMut",
    "AsyncFnOnce",
    "Box",
    "Clone",
    "Copy",
    "Default",
    "DoubleEndedIterator",
    "Drop",
    "Eq",
    "Err",
    "ExactSizeIterator",
    "Extend",
    "Fn",
    "FnMut",
    "FnOnce",
    "From",
    "FromIterator",
    "Future",
    "Into",
    "IntoFuture",
    "IntoIterator",
    "Iterator",
    "None",
    "Ok",
    "Option",
    "Ord",
    "PartialEq",
    "PartialOrd",
    "Result",
    "Send",
    "Sized",
    "Some",
    "String",
    "Sync",
    "ToOwned",
    "ToString",
    "TryFrom",
    "TryInto",
    "Unpin",
    "Vec",
    "drop",
];
const STD_MACROS: &[&str] = &[
    "assert",
    "assert_eq",
    "assert_ne",
    "cfg",
    "column",
    "compile_error",
    "concat",
    "dbg",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "env",
    "eprint",
    "eprintln",
    "file",
    "format",
    "format_args",
    "include",
    "include_bytes",
    "include_str",
    "line",
    "matches",
    "module_path",
    "option_env",
    "panic",
    "print",
    "println",
    "stringify",
    "thread_local",
    "todo",
    "unimplemented",
    "unreachable",
    "vec",
    "write",
    "writeln",
];
const QUOTE_MACROS: &[&str] = &[
    "quote",
    "quote_spanned",
    "parse_quote",
    "parse_quote_spanned",
];

#[derive(Clone, Copy)]
enum Surface {
    Quotes,
    ExportedRules,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Finding {
    line: usize,
    column: usize,
    name: String,
}

struct Audit {
    surface: Surface,
    bodies: usize,
    findings: Vec<Finding>,
}

fn punct(tokens: &[TokenTree], index: usize, ch: char) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Punct(p)) if p.as_char() == ch)
}

fn ident(tokens: &[TokenTree], index: usize, name: &str) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Ident(i)) if i == name)
}

fn after_path(tokens: &[TokenTree], index: usize) -> bool {
    index >= 2 && punct(tokens, index - 2, ':') && punct(tokens, index - 1, ':')
}

impl Audit {
    fn source(source: &str, surface: Surface) -> Self {
        // Parsing Rust items lets us exclude #[cfg(test)] modules. Macro bodies
        // remain token streams: quote interpolations need not parse as Rust.
        let tokens: TokenStream = source.parse().expect("source must tokenize");
        let mut file: syn::File = syn::parse2(tokens).expect("source must parse");
        let mut audit = Self {
            surface,
            bodies: 0,
            findings: Vec::new(),
        };
        audit.visit_file_mut(&mut file);
        audit.findings.sort();
        audit.findings.dedup();
        audit
    }

    fn quote_body(&mut self, name: &str, stream: TokenStream) {
        let mut tokens: Vec<_> = stream.into_iter().collect();
        if name.ends_with("_spanned") {
            // The span expression before => executes in the macro crate.
            let arrow = tokens
                .windows(2)
                .position(|pair| punct(pair, 0, '=') && punct(pair, 1, '>'))
                .expect("spanned quote must have =>");
            tokens.drain(..arrow + 2);
        }
        self.bodies += 1;
        self.generated(&tokens, false);
    }

    fn find_quotes(&mut self, stream: TokenStream) {
        let tokens: Vec<_> = stream.into_iter().collect();
        let mut index = 0;
        while index < tokens.len() {
            if let TokenTree::Ident(name) = &tokens[index]
                && QUOTE_MACROS.contains(&name.to_string().as_str())
                && punct(&tokens, index + 1, '!')
                && let Some(TokenTree::Group(group)) = tokens.get(index + 2)
            {
                self.quote_body(&name.to_string(), group.stream());
                index += 3;
                continue;
            }
            if let TokenTree::Group(group) = &tokens[index] {
                self.find_quotes(group.stream());
            }
            index += 1;
        }
    }

    fn exported_rules(&mut self, stream: TokenStream) {
        let tokens: Vec<_> = stream.into_iter().collect();
        let mut arms = 0;
        for rule in tokens.windows(4) {
            if matches!(rule[0], TokenTree::Group(_))
                && punct(rule, 1, '=')
                && punct(rule, 2, '>')
                && let TokenTree::Group(body) = &rule[3]
            {
                // Matchers name literal tokens and metavariables, not emitted
                // Rust references. Inspect only the transcriber after =>.
                self.generated(&body.stream().into_iter().collect::<Vec<_>>(), false);
                self.bodies += 1;
                arms += 1;
            }
        }
        assert!(arms > 0, "unrecognized exported macro_rules body");
    }

    fn find_exported_rules(&mut self, stream: TokenStream) {
        let tokens: Vec<_> = stream.into_iter().collect();
        let mut exported = false;
        let mut index = 0;
        while index < tokens.len() {
            if punct(&tokens, index, '#')
                && let Some(TokenTree::Group(attr)) = tokens.get(index + 1)
                && attr.delimiter() == Delimiter::Bracket
            {
                exported |= ident(
                    &attr.stream().into_iter().collect::<Vec<_>>(),
                    0,
                    "macro_export",
                );
                index += 2;
                continue;
            }
            if exported
                && ident(&tokens, index, "macro_rules")
                && punct(&tokens, index + 1, '!')
                && let Some(TokenTree::Group(body)) = tokens.get(index + 3)
            {
                self.exported_rules(body.stream());
                index += 4;
                exported = false;
                continue;
            }
            exported = false;
            if let TokenTree::Group(group) = &tokens[index] {
                self.find_exported_rules(group.stream());
            }
            index += 1;
        }
    }

    fn generated(&mut self, tokens: &[TokenTree], inferbase_args: bool) {
        for (index, token) in tokens.iter().enumerate() {
            match token {
                TokenTree::Group(group) => {
                    // __impl_inferbase!'s second argument is an RBase variant,
                    // not a Rust type. Do not exempt arbitrary $crate macro args.
                    let inferbase = index >= 6
                        && punct(tokens, index - 6, '$')
                        && ident(tokens, index - 5, "crate")
                        && after_path(tokens, index - 2)
                        && ident(tokens, index - 2, "__impl_inferbase")
                        && punct(tokens, index - 1, '!');
                    self.generated(&group.stream().into_iter().collect::<Vec<_>>(), inferbase);
                }
                TokenTree::Ident(name) => {
                    let raw = name.to_string();
                    let name_text = raw.strip_prefix("r#").unwrap_or(&raw);
                    let qualified = after_path(tokens, index)
                        || (index > 0
                            && ['.', '#', '$']
                                .iter()
                                .any(|&ch| punct(tokens, index - 1, ch)));
                    let variant = inferbase_args
                        && name_text == "String"
                        && tokens[..index]
                            .iter()
                            .filter(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ','))
                            .count()
                            == 1;
                    // The ABI's `drop:` field is not the prelude function.
                    // Do not exempt types before ':' (where String: Send).
                    let label = name_text == "drop"
                        && matches!(tokens.get(index + 1),
                        Some(TokenTree::Punct(p)) if p.as_char() == ':' && p.spacing() == Spacing::Alone);
                    if qualified || variant || label {
                        continue;
                    }
                    let relative_root = ["std", "core", "alloc"].contains(&name_text)
                        && punct(tokens, index + 1, ':')
                        && punct(tokens, index + 2, ':');
                    let std_macro =
                        STD_MACROS.contains(&name_text) && punct(tokens, index + 1, '!');
                    if relative_root || std_macro || PRELUDE_NAMES.contains(&name_text) {
                        let pos = name.span().start();
                        let suffix = if relative_root {
                            "::"
                        } else if std_macro {
                            "!"
                        } else {
                            ""
                        };
                        self.findings.push(Finding {
                            line: pos.line,
                            column: pos.column + 1,
                            name: format!("{name_text}{suffix}"),
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

fn requires_test(meta: &syn::Meta) -> bool {
    match meta {
        syn::Meta::Path(path) => path.is_ident("test"),
        syn::Meta::List(list) if list.path.is_ident("all") || list.path.is_ident("any") => {
            let parts = list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .expect("cfg predicate must parse");
            if list.path.is_ident("all") {
                parts.iter().any(requires_test)
            } else {
                !parts.is_empty() && parts.iter().all(requires_test)
            }
        }
        _ => false,
    }
}

fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("test")
            || (attr.path().is_ident("cfg")
                && requires_test(&attr.parse_args().expect("cfg must parse")))
    })
}

impl VisitMut for Audit {
    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        if !test_only(&item.attrs) {
            visit_mut::visit_item_mod_mut(self, item);
        }
    }

    fn visit_item_fn_mut(&mut self, item: &mut syn::ItemFn) {
        if !test_only(&item.attrs) {
            visit_mut::visit_item_fn_mut(self, item);
        }
    }

    fn visit_item_macro_mut(&mut self, item: &mut syn::ItemMacro) {
        if test_only(&item.attrs) {
            return;
        }
        if matches!(self.surface, Surface::ExportedRules)
            && item.mac.path.is_ident("macro_rules")
            && item.attrs.iter().any(|a| a.path().is_ident("macro_export"))
        {
            self.exported_rules(item.mac.tokens.clone());
        } else {
            self.visit_macro_mut(&mut item.mac);
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        match self.surface {
            Surface::Quotes => {
                let name = mac.path.segments.last().unwrap().ident.to_string();
                if QUOTE_MACROS.contains(&name.as_str()) {
                    self.quote_body(&name, mac.tokens.clone());
                } else {
                    self.find_quotes(mac.tokens.clone());
                }
            }
            Surface::ExportedRules => self.find_exported_rules(mac.tokens.clone()),
        }
    }
}

fn source_files(dir: &Path, paths: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().unwrap() != "tests" {
                source_files(&path, paths);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && path.file_name().unwrap() != "tests.rs"
        {
            paths.push(path);
        }
    }
}

#[test]
fn generated_standard_names_are_qualified() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut failures = Vec::new();
    for (krate, surface) in [
        ("miniextendr-macros", Surface::Quotes),
        ("miniextendr-api", Surface::ExportedRules),
    ] {
        let mut files = Vec::new();
        source_files(&root.join(krate).join("src"), &mut files);
        files.sort();
        let mut bodies = 0;
        for path in &files {
            let audit = Audit::source(&std::fs::read_to_string(path).unwrap(), surface);
            bodies += audit.bodies;
            for f in audit.findings {
                failures.push(format!(
                    "{}:{}:{}: unqualified {}",
                    path.strip_prefix(root).unwrap().display(),
                    f.line,
                    f.column,
                    f.name
                ));
            }
        }
        assert!(bodies > 0, "{krate}: no generated bodies inspected");
        println!(
            "{krate}: inspected {bodies} generated bodies in {} source files",
            files.len()
        );
    }
    assert!(
        failures.is_empty(),
        "generated Rust must use absolute standard-library paths:\n{}",
        failures.join("\n")
    );
}

fn names(source: &str, surface: Surface) -> Vec<String> {
    Audit::source(source, surface)
        .findings
        .into_iter()
        .map(|f| f.name)
        .collect()
}

#[test]
fn all_quote_forms_reject_bare_names_macros_and_relative_roots() {
    for name in QUOTE_MACROS {
        let span = if name.ends_with("_spanned") {
            "Some(span).unwrap() => "
        } else {
            ""
        };
        let source = format!(
            "fn emit() {{ quote::{name}!({span} Vec<Some<FnMut()>>; format!(); std::vec::Vec; core::option::Option; alloc::boxed::Box;); }}"
        );
        assert_eq!(
            names(&source, Surface::Quotes),
            [
                "Vec", "Some", "FnMut", "format!", "std::", "core::", "alloc::"
            ]
        );
    }
}

#[test]
fn qualified_paths_members_interpolations_and_literals_are_allowed() {
    let source = r##"
        fn emit() {
            quote! {
                ::std::vec::Vec<::core::option::Option<#String>>;
                ::core::option::Option::Some(#Vec);
                value.Some; Self::String; #(#Some)*; $String;
                ::std::format!("Vec format! std::core");
                #[doc = "Option and vec!"] struct Example;
            }
        }
    "##;
    assert!(names(source, Surface::Quotes).is_empty());
}

#[test]
fn field_names_do_not_hide_bare_function_references() {
    let source = r#"
        fn emit() {
            quote! {
                VTable { drop: ::core::mem::drop };
                fn check(drop: usize) where String: ::core::marker::Send {}
                drop(value);
                let f = drop;
            }
        }
    "#;
    assert_eq!(names(source, Surface::Quotes), ["String", "drop", "drop"]);
}

#[test]
fn source_walk_covers_feature_arms_and_macro_tokens_but_not_test_modules() {
    let source = r#"
        fn host() { let _: Vec<String>; }
        #[cfg(feature = "optional")] fn enabled() { quote!(Vec); }
        macro_rules! helper { () => { quote!(String); }; }
        #[cfg(test)] mod tests { fn test() { quote!(Option); } }
        #[cfg(all(test, feature = "optional"))] mod more_tests { fn test() { quote!(Some); } }
        #[cfg(any(test, feature = "optional"))] mod production_too { fn emit() { quote!(None); } }
    "#;
    assert_eq!(names(source, Surface::Quotes), ["Vec", "String", "None"]);
}

#[test]
fn exported_rules_scan_transcribers_including_repetitions() {
    let source = r#"
        macro_rules! private { () => { String }; }
        #[macro_export] macro_rules! exposed {
            (String, $String:ty) => { $String; Vec<$String>; };
            ($($t:ty),*) => { $(Option<$t>),* };
        }
        #[cfg(test)] mod tests { #[macro_export] macro_rules! test_only { () => { Some }; } }
    "#;
    assert_eq!(names(source, Surface::ExportedRules), ["Vec", "Option"]);
}

#[test]
fn exported_rules_inside_other_macro_tokens_are_inspected() {
    let source = r#"
        macro_rules! define {
            () => {
                #[macro_export] #[doc(hidden)]
                macro_rules! generated { () => { vec![String]; }; }
            };
        }
    "#;
    assert_eq!(names(source, Surface::ExportedRules), ["vec!", "String"]);
}

#[test]
fn inferbase_variant_exception_does_not_hide_type_arguments() {
    let source = r#"
        #[macro_export] macro_rules! family {
            ($ty:ty) => {
                $crate::__impl_inferbase!($ty, String, $crate::make, install);
                $crate::other!(String);
                $crate::__impl_inferbase!(String, String, $crate::make, install);
            };
        }
    "#;
    assert_eq!(names(source, Surface::ExportedRules), ["String", "String"]);
}

#[test]
fn inventory_covers_the_compile_fixture_shadows() {
    let file = syn::parse_file(include_str!("ui/pass/shadowed_standard_names.rs")).unwrap();
    for item in file.items {
        match item {
            syn::Item::Struct(item) => assert!(
                PRELUDE_NAMES.contains(&item.ident.to_string().as_str()),
                "{}",
                item.ident
            ),
            syn::Item::Trait(item) => assert!(
                PRELUDE_NAMES.contains(&item.ident.to_string().as_str()),
                "{}",
                item.ident
            ),
            syn::Item::Macro(item) if item.mac.path.is_ident("macro_rules") => {
                let name = item.ident.unwrap().to_string();
                assert!(STD_MACROS.contains(&name.as_str()), "{name}");
            }
            _ => {}
        }
    }
}
