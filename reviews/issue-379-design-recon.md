# Issue #379 Design Recon

## Status of @prop tag emission (as of this branch)

PR #377 added the infrastructure for `@prop` docs on S7 properties defined via
`#[miniextendr(s7(getter))]` methods. But looking at the actual code:

- `ClassDocBuilder.build()` does NOT emit `@prop` tags at all.
- `generate_s7_r_wrapper` iterates `properties.values()` to emit `new_property()`
  definitions but never emits `#' @prop <name> <doc>` before them.
- The `S7Property` struct (local to `generate_s7_r_wrapper`) has no `doc` field.
- Conclusion: `@prop` tag emission for impl-block properties is ALSO missing,
  not just for sidecar ones. We need to add both.

## Where to add @prop emission for impl-block properties

`s7_class.rs` around line 500-506:

```rust
if prop_parts.is_empty() {
    prop_items.push(format!("    {} = S7::new_property()", prop.name));
} else {
    prop_items.push(format!("    {} = S7::new_property({})", ...));
}
```

The `@prop` lines should go into the class-level roxygen block, BEFORE the
`S7::new_class(...)` call. The best insertion point is after the constructor
`@param` lines emitted at lines ~337-376 and before `S7::new_class(...)` at
lines ~379-391.

The `S7Property` struct needs a `doc: Option<String>` field, populated from the
getter method's doc_tags (first non-empty line that isn't a tag, or the doc
comment on the method).

For the getter's doc, look at `method.doc_tags` on the getter method — these
are already the parsed `/// text` values stripped of the `///` prefix. The
first non-tag line is the description.

## Where prop_doc parsing should slot in externalptr_derive.rs

In `parse_sidecar_info` (lines 288-339), for each non-selector `#[r_data]` field,
we need to parse an optional `prop_doc` attribute from the field's attrs:

```rust
#[r_data(prop_doc = "...")]
pub prop_int: i32,
```

The `#[r_data]` attribute already has no meta-arguments in the current code —
`has_r_data_attr` just checks for presence. We need to add a parser for
`r_data(prop_doc = "...")`.

The `SidecarSlot` struct (lines 218-230) should gain a `prop_doc: Option<String>`
field.

## New distributed_slice: MX_S7_SIDECAR_PROPS

In `miniextendr-api/src/registry.rs`, add:

```rust
pub struct SidecarPropEntry {
    pub rust_type: &'static str,   // e.g. "SidecarS7"
    pub field_name: &'static str,  // e.g. "prop_int"
    pub prop_doc: &'static str,    // e.g. "Integer sidecar property" or "(undocumented sidecar property)"
}

#[distributed_slice]
pub static MX_S7_SIDECAR_PROPS: [SidecarPropEntry];
```

## Where emit entries in externalptr_derive.rs

In `generate_sidecar_accessors` (line 856+), for each public slot,
emit a `#[distributed_slice(MX_S7_SIDECAR_PROPS)]` static ONLY when
`info.class_system == ClassSystem::S7`. This mirrors how `MX_CALL_DEFS` and
`MX_R_WRAPPERS` entries are emitted.

## Where the @prop loop is in s7_class.rs (where to fold in sidecar entries)

The `@prop` loop doesn't exist yet — we're adding it. The insertion point is
the roxygen header block, between the constructor `@param` lines and the
`S7::new_class(...)` call. Steps:

1. Emit `#' @prop {name} {doc}` for each impl-block property (from `properties.values()`).
2. If `parsed_impl.r_data_accessors` is true, also emit `#' @prop {name} {doc}` for
   each matching entry in `MX_S7_SIDECAR_PROPS` where `rust_type == type_ident.to_string()`.
3. Sidecar `@prop` lines come AFTER impl-block `@prop` lines (matching runtime ordering).

## Key design deviation from original plan

The plan said "emit `#[distributed_slice]` per field on `#[miniextendr(s7, r_data_accessors)]` types only". But the `#[derive(ExternalPtr)]` macro doesn't know if the `impl` block will have `r_data_accessors`. The safest approach is to emit entries regardless of class system (filtering only by `S7` class system from `#[externalptr(s7)]`), and have the S7 codegen gate on `parsed_impl.r_data_accessors`.

Alternatively: emit for ALL class systems but have the S7 codegen only look them up when `r_data_accessors` is true. This keeps the derive macro simpler.

## No need for MX_S7_SIDECAR_PROPS to exist at macro expansion time

`generate_s7_r_wrapper` is called at cdylib runtime (when `miniextendr_write_wrappers`
is called), not at proc-macro expansion time. By then, `MX_S7_SIDECAR_PROPS` is
fully populated. So the distributed_slice approach is sound.
