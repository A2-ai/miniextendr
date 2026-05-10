//! Host-time generator of `wasm_registry.rs` — the WASM-side replacement for
//! `linkme`'s runtime distributed-slice gather.
//!
//! On native builds, the cdylib runs [`write_wasm_registry_to_file`] to emit
//! Rust source listing every `MX_CALL_DEFS` / `MX_ALTREP_REGISTRATIONS` /
//! `MX_TRAIT_DISPATCH` entry as `extern "C" {}` declarations + ordinary
//! `&[T]` static slices. On `wasm32-*` targets, the user crate compiles that
//! file in place of the linkme distributed_slices (gating happens in step 5
//! of `plans/webr-support.md`).
//!
//! The writer is intentionally pure-text-formatting — no `syn`, no
//! `proc-macro2`, no template engine. Output is small, append-only, and
//! deterministic so `git diff --exit-code` works as the regen check.

use crate::abi::mx_tag;
use crate::registry::{
    AltrepRegistration, MX_ALTREP_REGISTRATIONS, MX_CALL_DEFS, MX_TRAIT_DISPATCH,
    TraitDispatchEntry,
};
use std::ffi::CStr;
use std::fmt::Write as _;

// Bumped whenever the generated-file shape changes (struct fields, import
// paths, macro names). The receiving `build.rs` (step 5) refuses to compile
// a `wasm_registry.rs` whose header doesn't match.
const GENERATOR_VERSION: u32 = 1;

/// Pre-extracted, cdylib-side view of one `R_CallMethodDef`.
///
/// `R_CallMethodDef` carries `name` as a raw `*const c_char`; safely walking
/// it requires `unsafe`. The formatter takes already-extracted, owned values
/// so it can be unit-tested without globals.
pub struct CallDefRow {
    pub name: String,
    pub num_args: i32,
}

/// Pre-extracted view of one `MX_ALTREP_REGISTRATIONS` entry.
pub struct AltrepRegRow {
    pub symbol: String,
}

/// Pre-extracted view of one `MX_TRAIT_DISPATCH` entry.
pub struct TraitDispatchRow {
    pub concrete_tag: mx_tag,
    pub trait_tag: mx_tag,
    pub vtable_symbol: String,
}

/// Format a `wasm_registry.rs` source file from extracted runtime data.
///
/// Output structure:
/// ```text
/// // header (auto-generated marker, generator-version, content-hash)
/// use ...;
/// unsafe extern "C-unwind" { fn <wrapper>(...); ... }
/// unsafe extern "C" { fn <altrep_reg>(); ... static <vtable>: u8; ... }
/// pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[ ... ];
/// pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &[ ... ];
/// pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[ ... ];
/// ```
///
/// Every fn / static referenced from a slice gets a matching `extern` decl —
/// the WASM linker resolves them against the user crate's `#[no_mangle]`
/// exports.
pub fn format_wasm_registry(
    call_defs: &[CallDefRow],
    altrep_regs: &[AltrepRegRow],
    trait_dispatches: &[TraitDispatchRow],
) -> String {
    let body = format_body(call_defs, altrep_regs, trait_dispatches);
    let content_hash = fnv1a_64(body.as_bytes());

    let mut out = String::new();
    writeln!(&mut out, "// AUTO-GENERATED — DO NOT EDIT.").unwrap();
    writeln!(&mut out, "//").unwrap();
    writeln!(
        &mut out,
        "// Produced on host by `miniextendr_write_wasm_registry`. Compiled on"
    )
    .unwrap();
    writeln!(
        &mut out,
        "// wasm32-* targets in place of the linkme distributed_slices."
    )
    .unwrap();
    writeln!(&mut out, "//").unwrap();
    writeln!(&mut out, "// generator-version: {GENERATOR_VERSION}").unwrap();
    writeln!(&mut out, "// content-hash:      {content_hash:016x}").unwrap();
    writeln!(&mut out).unwrap();
    out.push_str(&body);
    out
}

fn format_body(
    call_defs: &[CallDefRow],
    altrep_regs: &[AltrepRegRow],
    trait_dispatches: &[TraitDispatchRow],
) -> String {
    let mut out = String::new();

    writeln!(&mut out, "use ::miniextendr_api::abi::mx_tag;").unwrap();
    writeln!(
        &mut out,
        "use ::miniextendr_api::ffi::{{R_CallMethodDef, SEXP}};"
    )
    .unwrap();
    writeln!(
        &mut out,
        "use ::miniextendr_api::registry::{{AltrepRegistration, TraitDispatchEntry}};"
    )
    .unwrap();
    writeln!(&mut out, "use ::core::ffi::c_void;").unwrap();
    writeln!(&mut out).unwrap();

    format_extern_unwind_block(&mut out, call_defs);
    format_extern_c_block(&mut out, altrep_regs, trait_dispatches);
    format_call_defs_slice(&mut out, call_defs);
    format_altrep_regs_slice(&mut out, altrep_regs);
    format_trait_dispatch_slice(&mut out, trait_dispatches);

    out
}

fn format_extern_unwind_block(out: &mut String, call_defs: &[CallDefRow]) {
    writeln!(out, "unsafe extern \"C-unwind\" {{").unwrap();
    for row in call_defs {
        let params = sexp_param_list(row.num_args);
        writeln!(out, "    pub fn {}({params}) -> SEXP;", row.name).unwrap();
    }
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
}

fn format_extern_c_block(
    out: &mut String,
    altrep_regs: &[AltrepRegRow],
    trait_dispatches: &[TraitDispatchRow],
) {
    writeln!(out, "unsafe extern \"C\" {{").unwrap();
    // ALTREP register fns get the 2024-edition `safe` qualifier so the fn
    // type stays `extern "C" fn()` (not `unsafe extern "C" fn()`), allowing
    // direct assignment to `AltrepRegistration.register`. Semantically the
    // register fns are safe to call from anywhere — they wrap a OnceLock
    // init and don't take SEXP arguments.
    for row in altrep_regs {
        writeln!(out, "    pub safe fn {}();", row.symbol).unwrap();
    }
    // Vtable shape is opaque from wasm_registry.rs' perspective — we only
    // need the address. Declaring as `u8` is the convention used in the
    // plan sketch and keeps the file independent of trait-specific types.
    for row in trait_dispatches {
        writeln!(out, "    pub static {}: u8;", row.vtable_symbol).unwrap();
    }
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
}

fn format_call_defs_slice(out: &mut String, call_defs: &[CallDefRow]) {
    writeln!(out, "pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[").unwrap();
    for row in call_defs {
        // The transmute target signature here is positional-only — Rust's
        // `extern fn` *type* doesn't carry parameter names, only types — so
        // it differs from the `extern { fn ... }` declaration form, which
        // does require named or `_`-bound parameters.
        let arg_types = sexp_type_list(row.num_args);
        writeln!(out, "    R_CallMethodDef {{").unwrap();
        writeln!(out, "        name: c\"{}\".as_ptr(),", row.name).unwrap();
        writeln!(
            out,
            "        fun: Some(unsafe {{ ::core::mem::transmute::<unsafe extern \"C-unwind\" fn({arg_types}) -> SEXP, _>({}) }}),",
            row.name
        )
        .unwrap();
        writeln!(out, "        numArgs: {},", row.num_args).unwrap();
        writeln!(out, "    }},").unwrap();
    }
    writeln!(out, "];").unwrap();
    writeln!(out).unwrap();
}

fn format_altrep_regs_slice(out: &mut String, altrep_regs: &[AltrepRegRow]) {
    writeln!(
        out,
        "pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &["
    )
    .unwrap();
    for row in altrep_regs {
        writeln!(out, "    AltrepRegistration {{").unwrap();
        writeln!(out, "        register: {},", row.symbol).unwrap();
        writeln!(out, "        symbol: {:?},", row.symbol).unwrap();
        writeln!(out, "    }},").unwrap();
    }
    writeln!(out, "];").unwrap();
    writeln!(out).unwrap();
}

fn format_trait_dispatch_slice(out: &mut String, trait_dispatches: &[TraitDispatchRow]) {
    writeln!(
        out,
        "pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &["
    )
    .unwrap();
    for row in trait_dispatches {
        writeln!(out, "    TraitDispatchEntry {{").unwrap();
        writeln!(
            out,
            "        concrete_tag: mx_tag::new(0x{:016x}, 0x{:016x}),",
            row.concrete_tag.lo, row.concrete_tag.hi
        )
        .unwrap();
        writeln!(
            out,
            "        trait_tag: mx_tag::new(0x{:016x}, 0x{:016x}),",
            row.trait_tag.lo, row.trait_tag.hi
        )
        .unwrap();
        writeln!(
            out,
            "        vtable: unsafe {{ ::core::ptr::from_ref(&{}).cast::<c_void>() }},",
            row.vtable_symbol
        )
        .unwrap();
        writeln!(out, "        vtable_symbol: {:?},", row.vtable_symbol).unwrap();
        writeln!(out, "    }},").unwrap();
    }
    writeln!(out, "];").unwrap();
}

/// Comma-joined `_: SEXP` parameter list for an `extern { fn ...; }` decl.
///
/// Extern fn declarations require parameter bindings — bare `SEXP, SEXP`
/// is parsed as pattern-typed which fails. Each slot is `_: SEXP`.
fn sexp_param_list(num_args: i32) -> String {
    if num_args <= 0 {
        return String::new();
    }
    std::iter::repeat_n("_: SEXP", num_args as usize)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Comma-joined `SEXP` type list for an `extern fn(...)` *type* expression
/// (e.g. inside `transmute::<...>`). Function pointer types don't carry
/// parameter names, so `_:` would be invalid here.
fn sexp_type_list(num_args: i32) -> String {
    if num_args <= 0 {
        return String::new();
    }
    std::iter::repeat_n("SEXP", num_args as usize)
        .collect::<Vec<_>>()
        .join(", ")
}

/// FNV-1a 64-bit hash. Matches the implementation in
/// `miniextendr-macros/src/miniextendr_impl_trait.rs::type_to_uppercase_name`
/// so a future build.rs check can recompute it portably.
fn fnv1a_64(data: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;
    let mut h = OFFSET_BASIS;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Read the live linkme distributed slices and return rows safe to pass to
/// [`format_wasm_registry`].
fn read_runtime_slices() -> (Vec<CallDefRow>, Vec<AltrepRegRow>, Vec<TraitDispatchRow>) {
    let call_defs: Vec<CallDefRow> = MX_CALL_DEFS
        .iter()
        .map(|d| {
            // SAFETY: every emission site sets `name` from a static CStr literal
            // (see `c_wrapper_builder.rs` and friends), so the pointer is valid
            // for the program lifetime and points to a NUL-terminated UTF-8
            // ASCII string.
            let name = unsafe { CStr::from_ptr(d.name) }
                .to_str()
                .expect("MX_CALL_DEFS.name is not valid UTF-8")
                .to_string();
            CallDefRow {
                name,
                num_args: d.numArgs,
            }
        })
        .collect();

    let altrep_regs: Vec<AltrepRegRow> = MX_ALTREP_REGISTRATIONS
        .iter()
        .map(|r: &AltrepRegistration| AltrepRegRow {
            symbol: r.symbol.to_string(),
        })
        .collect();

    let trait_dispatches: Vec<TraitDispatchRow> = MX_TRAIT_DISPATCH
        .iter()
        .map(|t: &TraitDispatchEntry| TraitDispatchRow {
            concrete_tag: t.concrete_tag,
            trait_tag: t.trait_tag,
            vtable_symbol: t.vtable_symbol.to_string(),
        })
        .collect();

    (call_defs, altrep_regs, trait_dispatches)
}

/// Read the live distributed slices, format `wasm_registry.rs`, and write it
/// to `path`. No-op when content is unchanged (matches `write_r_wrappers_to_file`).
pub fn write_wasm_registry_to_file(path: &str) {
    let (call_defs, altrep_regs, trait_dispatches) = read_runtime_slices();
    let content = format_wasm_registry(&call_defs, &altrep_regs, &trait_dispatches);

    let existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing == content {
        return;
    }

    std::fs::write(path, content.as_bytes())
        .unwrap_or_else(|e| panic!("failed to write {path}: {e}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_inputs() -> (Vec<CallDefRow>, Vec<AltrepRegRow>, Vec<TraitDispatchRow>) {
        let call_defs = vec![
            CallDefRow {
                name: "miniextendr_my_fn".into(),
                num_args: 0,
            },
            CallDefRow {
                name: "miniextendr_other".into(),
                num_args: 3,
            },
        ];
        let altrep_regs = vec![AltrepRegRow {
            symbol: "__mx_altrep_reg_MyType".into(),
        }];
        let trait_dispatches = vec![TraitDispatchRow {
            concrete_tag: mx_tag::new(0xdead_beef_dead_beef, 0x1234_5678_1234_5678),
            trait_tag: mx_tag::new(0xcafe_babe_cafe_babe, 0xfeed_face_feed_face),
            vtable_symbol: "__VTABLE_COUNTER_FOR_MYTYPE".into(),
        }];
        (call_defs, altrep_regs, trait_dispatches)
    }

    #[test]
    fn header_carries_generator_version_and_content_hash() {
        let (a, b, c) = sample_inputs();
        let out = format_wasm_registry(&a, &b, &c);
        assert!(
            out.contains(&format!("generator-version: {GENERATOR_VERSION}")),
            "expected generator-version line; got:\n{out}"
        );
        assert!(
            out.contains("content-hash:      "),
            "expected content-hash line; got:\n{out}"
        );
    }

    #[test]
    fn content_hash_is_deterministic() {
        let (a, b, c) = sample_inputs();
        let first = format_wasm_registry(&a, &b, &c);
        let second = format_wasm_registry(&a, &b, &c);
        assert_eq!(first, second);
    }

    #[test]
    fn content_hash_changes_when_body_changes() {
        let (a, b, c) = sample_inputs();
        let mut a2 = a.clone_into_unique();
        a2.push(CallDefRow {
            name: "different_fn".into(),
            num_args: 1,
        });
        let first = format_wasm_registry(&a, &b, &c);
        let second = format_wasm_registry(&a2, &b, &c);
        assert_ne!(first, second);
    }

    #[test]
    fn emits_extern_decls_for_every_referenced_symbol() {
        let (a, b, c) = sample_inputs();
        let out = format_wasm_registry(&a, &b, &c);
        // call wrappers in the C-unwind block — params bound to `_` so the
        // declaration parses (extern fn decls require parameter bindings).
        assert!(out.contains("pub fn miniextendr_my_fn() -> SEXP;"));
        assert!(out.contains("pub fn miniextendr_other(_: SEXP, _: SEXP, _: SEXP) -> SEXP;"));
        // altrep registrations + vtables in the C block
        assert!(out.contains("pub safe fn __mx_altrep_reg_MyType();"));
        assert!(out.contains("pub static __VTABLE_COUNTER_FOR_MYTYPE: u8;"));
    }

    #[test]
    fn emits_named_slice_constants() {
        let (a, b, c) = sample_inputs();
        let out = format_wasm_registry(&a, &b, &c);
        assert!(out.contains("pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef]"));
        assert!(out.contains("pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration]"));
        assert!(out.contains("pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry]"));
    }

    #[test]
    fn renders_mx_tag_with_const_constructor() {
        let (a, b, c) = sample_inputs();
        let out = format_wasm_registry(&a, &b, &c);
        assert!(
            out.contains("mx_tag::new(0xdeadbeefdeadbeef, 0x1234567812345678)"),
            "expected concrete_tag literal; got:\n{out}"
        );
        assert!(
            out.contains("mx_tag::new(0xcafebabecafebabe, 0xfeedfacefeedface)"),
            "expected trait_tag literal; got:\n{out}"
        );
    }

    #[test]
    fn empty_inputs_produce_empty_slices() {
        let out = format_wasm_registry(&[], &[], &[]);
        assert!(out.contains("pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[\n];"));
        assert!(
            out.contains("pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &[\n];")
        );
        assert!(out.contains("pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[\n];"));
    }

    #[test]
    fn param_list_uses_underscore_bindings() {
        assert_eq!(sexp_param_list(0), "");
        assert_eq!(sexp_param_list(1), "_: SEXP");
        assert_eq!(sexp_param_list(3), "_: SEXP, _: SEXP, _: SEXP");
    }

    #[test]
    fn type_list_is_bare_types() {
        assert_eq!(sexp_type_list(0), "");
        assert_eq!(sexp_type_list(1), "SEXP");
        assert_eq!(sexp_type_list(3), "SEXP, SEXP, SEXP");
    }

    #[test]
    fn altrep_register_decls_use_safe_keyword() {
        let (a, b, c) = sample_inputs();
        let out = format_wasm_registry(&a, &b, &c);
        assert!(
            out.contains("pub safe fn __mx_altrep_reg_MyType()"),
            "expected `safe fn` so the fn type matches AltrepRegistration.register; got:\n{out}"
        );
    }

    // Helper: clone a Vec<CallDefRow> by re-creating each row (CallDefRow
    // doesn't impl Clone — keeping it minimal). Used only in
    // `content_hash_changes_when_body_changes`.
    trait CloneIntoUnique {
        fn clone_into_unique(&self) -> Vec<CallDefRow>;
    }
    impl CloneIntoUnique for Vec<CallDefRow> {
        fn clone_into_unique(&self) -> Vec<CallDefRow> {
            self.iter()
                .map(|r| CallDefRow {
                    name: r.name.clone(),
                    num_args: r.num_args,
                })
                .collect()
        }
    }
}
