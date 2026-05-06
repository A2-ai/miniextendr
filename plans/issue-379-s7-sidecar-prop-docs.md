# Issue #379 — S7 @prop emission for sidecar (r_data_accessors) properties

## Decision: Option 3 from the issue
Add `prop_doc` slot on `#[r_data_accessors(prop_doc = "...")]`. ExternalPtr
derive macro emits compile-time `MX_S7_SIDECAR_PROPS` distributed_slice
entries. S7 codegen reads them at write time, folds into the @prop loop.

## Implementation
1. `miniextendr-macros/src/externalptr_derive.rs`:
   - Add `prop_doc: Option<String>` to the `RDataAccessor` parsed-attr struct.
   - Parse from `#[r_data_accessors(prop_doc = "...")]`.
   - Emit `#[distributed_slice(MX_S7_SIDECAR_PROPS)]` per field on
     `#[miniextendr(s7, r_data_accessors)]` types only.
2. `miniextendr-api/src/registry.rs`: add `MX_S7_SIDECAR_PROPS`
   distributed_slice + `SidecarProp` struct.
3. `miniextendr-macros/src/miniextendr_impl/s7_class.rs`: at write time,
   query `MX_S7_SIDECAR_PROPS` for entries matching the current class's
   `type_name` and emit `#' @prop {field_name} {prop_doc}` (default
   `"(undocumented sidecar property)"` if omitted).
   Sidecar @prop lines come AFTER impl-block @prop lines (matches runtime
   `properties = c(list(...), .rdata_properties_<T>)` ordering).

## Tests
- Macro snapshot: derive struct with `prop_doc` + S7 impl with no impl-block
  props → `@prop {field} {doc}` line in output.
- Macro snapshot: derive struct + impl-block props → both kinds emit;
  impl-block first.
- rpkg integration: an existing or new `r_data_accessors` S7 fixture; verify
  rendered `man/<Type>.Rd` shows sidecar prop in the additional-properties
  section.
- Negative: missing `prop_doc` uses default text without error.

## Acceptance
- Sidecar props appear in `?<S7Type>` help via `@prop`.
- No regression for impl-block props.
- `just check && just clippy && just devtools-test && just devtools-document` clean.

## In-scope NIT (pickup from #379)
- `snapshot_s7_prop_tags` test currently happy-path only. Add coverage for
  default-valued constructor params and varargs in the same PR.

## Out of scope NIT
- Constructor-formal parsing brittleness in `s7_class.rs` (`split(", ")`
  then `split('=')`) — pre-existing, defer.
