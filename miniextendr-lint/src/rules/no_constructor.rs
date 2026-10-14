//! MXL121: a class R code can never create.
//!
//! A type's objects come from its constructor, from the functions that
//! return one, or from pointers the crate builds itself. The constructor is
//! the method marked `#[miniextendr(<system>(constructor))]`, else the
//! receiver-less method returning bare `Self` the macro infers, in any of the
//! type's inherent `#[miniextendr]` impl blocks. A factory is any
//! `#[miniextendr]` fn or method returning `T`, `Self` or `ExternalPtr<T>`,
//! bare or inside `Result` / `Option` (`returned_types` in the index). A
//! hand-built pointer is an `ExternalPtr::new(T { .. })` / `ExternalPtr::new(T(..))`,
//! or an `ExternalPtr<T>` written as a type or turbofish, anywhere in the
//! crate, `#[miniextendr]` or not (`hand_built_types`): a wrapper whose
//! hand-written `IntoR` builds the pointer is a factory too. Without any of
//! these, every method of the class needs an object R code has no way to get.
//!
//! The type's own methods taking `self` don't count: they need an object
//! already. Nor do parameters: the caller has the object. vctrs classes are
//! skipped: the package builds their vectors in R from the payload, not from
//! a `T`. `ExternalPtr::new(value)` where the value's type is written nowhere
//! is invisible; `// mxl::allow(MXL121)` covers it.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use crate::crate_index::{CrateIndex, InherentImpl};
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    let mut returned: HashSet<&str> = HashSet::new();
    let mut marked: HashSet<&str> = HashSet::new();
    let mut blocks: BTreeMap<&str, Vec<(&PathBuf, &InherentImpl)>> = BTreeMap::new();
    for (path, data) in &index.file_data {
        returned.extend(data.returned_types.iter().map(String::as_str));
        returned.extend(data.hand_built_types.iter().map(String::as_str));
        for (type_name, methods) in &data.impl_methods {
            if methods.iter().any(|m| m.has_constructor_attr) {
                marked.insert(type_name);
            }
        }
        for block in &data.inherent_impls {
            blocks
                .entry(&block.type_name)
                .or_default()
                .push((path, block));
        }
    }

    for (type_name, blocks) in blocks {
        if returned.contains(type_name)
            || marked.contains(type_name)
            || blocks
                .iter()
                .any(|(_, b)| b.class_system == "vctrs" || b.allows_mxl121)
        {
            continue;
        }
        let Some((path, first)) = blocks.iter().min_by_key(|(path, b)| (*path, b.line)) else {
            continue;
        };
        diagnostics.push(
            Diagnostic::new(
                LintCode::MXL121,
                path.as_path(),
                first.line,
                format!(
                    "R code can never create a `{type_name}`: no impl block of `{type_name}` \
                     has a constructor, no `#[miniextendr]` function or method returns one, \
                     and the crate builds no `ExternalPtr<{type_name}>`"
                ),
            )
            .with_help(format!(
                "mark a constructor with `#[miniextendr(<system>(constructor))]`, or add a \
                 `#[miniextendr]` function returning `{type_name}` or `ExternalPtr<{type_name}>`; \
                 where the crate builds the pointer from a value whose type it never writes, \
                 name it (`let ptr: ExternalPtr<{type_name}> = ...`) or put \
                 `// mxl::allow(MXL121)` on the impl"
            )),
        );
    }
}
