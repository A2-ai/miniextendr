# Plan: #1060 + #1061 — serde enum reader symmetry + configurable tag-column name

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `feat/1060-1061-serde-enum-reader`.

One PR, two issues: #1060 is the substantive reader work; #1061's tag-name
config must land with it so the name is honored on BOTH sides from day one.

## Verified state

- Writers: `vec_to_dataframe_flatten_enums`
  (`miniextendr-api/src/serde/columnar.rs:3941`) hard-codes per-field tag
  column `<field>_variant`; top-level `SplitShape::Collated { column }`
  (`:3123`) already lets callers name the top-level tag column.
- Reader: `dataframe_de.rs` forwards `deserialize_enum` into the erroring
  `deserialize_any` (`:324` + the `forward_to_deserialize_any!` list at
  `:353`; the row/cell deserializer sits at `:522-629`). Neither top-level
  Collated enums nor nested flattened enum fields round-trip.

## Work items (flat order)

1. **Reader — nested enum fields** (#1060, `dataframe_de.rs`): implement
   `deserialize_enum` on the row deserializer instead of forwarding it.
   Mechanism: for field `f` with variants list from serde, read the tag cell
   from column `<f>_variant` (or the configured name, item 3) at the current
   row; yield an `EnumAccess` whose `VariantAccess` deserializes the payload
   from the prefixed columns `<f>_<subfield>` for that row (struct variants),
   the single `<f>_<...>` payload column (newtype variants — match whatever
   column shape the WRITER emits; read the writer's emission in
   `columnar.rs` around `:3941`ff and the #1056 tests to fix the exact
   column-name contract), and no columns (unit variants). Follow the file's
   existing `MapAccess`/cell-deserializer structure — this must slot into the
   row-wise model, not restructure it.
2. **Reader — top-level Collated** (#1060): `dataframe_to_vec`-family reads
   of a Collated frame: tag column named by the caller (symmetric with
   `SplitShape::Collated { column }`). Follow how the writer's Collated shape
   is parameterized and mirror it on the read entry point (a `_collated`
   variant or an options param — copy the file's existing option-plumbing
   convention; do NOT invent a new config style).
3. **Tag-name config** (#1061): add
   `vec_to_dataframe_flatten_enums_with_tags<T: Serialize>(rows, tags: &[(&str, &str)])`
   (field → tag-column name; unlisted fields keep `<field>_variant`), with
   the existing fn delegating to it with an empty mapping. Mirror the same
   mapping parameter on the reader entry point from item 1 so custom names
   round-trip.
4. **Round-trip tests** (api integration tests — extend
   `miniextendr-api/tests/serde_columnar.rs` / the #1056 test home):
   - nested enum field: write via `vec_to_dataframe_flatten_enums`, read
     back via the reader, assert `Vec<T>` equality — unit, newtype, and
     struct variants; `Option`-bearing payload fields with `None` rows.
   - top-level Collated: write `vec_to_dataframe_split(Collated{column})`,
     read back with the same column name.
   - custom tag names both ways (`tags = [("state", "state_tag")]`).
   - error cases: missing tag column → clear error naming the expected
     column; unknown variant string in the tag cell → serde unknown-variant
     error listing variants.
5. rpkg surface: if `rpkg/src/rust/` has flatten-enum fixtures (grep
   `flatten_enums`), add ONE reverse fixture (df → Vec<enum-bearing T>) +
   testthat row proving the R-facing round trip; new export → ×2 install.
6. Docs: `docs/SERDE_R.md` capability table — flip the reader cells for
   split/flattened enums from "serialize-only" to supported, and document
   the `_with_tags` mapping.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/1060-api.log     # Read it
just configure && just rcmdinstall && just force-document && just rcmdinstall
just devtools-test 2>&1 > /tmp/1060-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1060-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- Writer output shapes (column names, variant casing, `_type` column
  conventions) — the reader adapts to the writer, never the reverse.
- `serde/de.rs` (the non-dataframe deserializer) beyond what enum plumbing
  genuinely requires; `dataframe_derive.rs` (the derive is a separate
  surface).
- The `DataFrameRow` HashMap `unzip()` lock-step invariant.

## Done criteria

- Every #1056 writer shape round-trips back to `Vec<T>` (tests in item 4
  all green); custom tag names honored on both sides; suites + three clippy
  legs green; `Fixes #1060, fixes #1061`.

## Escalation rule

If reality diverges from this plan — the row-wise deserializer cannot host
`EnumAccess` without restructuring, the writer's newtype/tuple-variant column
contract is ambiguous, any writer change looks necessary — **stop, commit
nothing further, and report back. Do not improvise.**
