# Satellite-bridge batch — #1067, #1068, #1069, #1070, #1071, #1072

Follow-ups to the serde satellite-crate harness (PR #1066). Re-verified
2026-07-11 against main @ 17f634d8 (original pass: 88c493fd): the PRECONDITION
below is still unresolved (`satellite` appears in NEITHER
`rpkg/tools/detect-features.R` nor any `ci.yml` feature list — but it is also
not denylisted, so configure-time auto-detection MAY enable it; the gate's
step 1 decides). Branches:
- SAT-1 #1068+#1069 → `test/1068-1069-satellite-shape-surface`
- SAT-2 #1071+#1072 → `feat/1071-1072-satellite-json-hybrid`
- SAT-3 #1070 → `docs/1070-satellite-serde-unification`
- SAT-4 #1067 → `feat/1067-satellite-bridge-macro` (LAST; defer if batch runs long)

All units: `rig default 4.6` first; `just worktree-sync` FIRST; regen loop ×2
for new exports (`just configure && just rcmdinstall && just force-document
&& just rcmdinstall`); `just devtools-test 2>&1 > /tmp/sat-devtools.log` then
`grep -E '\[ FAIL [0-9]+'` (devtools::test always exits 0); three clippy legs
per ci.yml; `cargo fmt --all`. Escalation rule: if reality diverges from this
plan — the precondition resolves to "cannot be enabled in CI without
structural work", an anchor doesn't match — **stop, commit nothing further,
and report back. Do not improvise.** The satellite crate is `rpkg/src/rust/satellite/` (sealed
workspace, excluded — `rpkg/src/rust/Cargo.toml:11`), bridged via
`rpkg/src/rust/satellite_bridge.rs`, feature `satellite = ["serde", "dep:satellite"]`.
Existing bridge fns: `satellite_readings_df`, `satellite_stations_df`,
`satellite_readings_list` (AsSerialize), `satellite_events_split`,
`satellite_echo_reading` (from_r). Samples: `sample_readings/stations/events`.

## PRECONDITION — verify CI actually compiles the satellite feature

The issues (#1068–#1071) assert "CI compiles the satellite feature (it's in the
detect-features set)." **This is unverified and looks false**: `satellite` is NOT
in the clippy feature lists (`ci.yml:328,341`). Before relying on CI to guard any
new fixture:
1. Check `rpkg/tools/detect-features.R` output includes `satellite`
   (`Rscript rpkg/tools/detect-features.R | tr ',' '\n' | grep satellite`).
2. If absent, EITHER add it to the detect set OR add `satellite` to a clippy leg
   feature list, as the first commit — otherwise every fixture below compiles
   locally and silently rots in CI.
This gate blocks the whole batch's value; do it first, report the finding.

## Suggested shipping order (build the surface by hand, then automate)

**PR 1 — #1068 + #1069** (exercise the shape surface + add coverage; they pair
naturally since #1069 tests what #1068 adds):

#1068 — add bridge fixtures for the rest of the serde→R surface:
- `result_to_dataframe` (Ok/Err partition) — needs a `sample_result`-shaped
  satellite sample; extend `satellite/src/lib.rs`.
- `map_to_dataframe` / `hashmap_to_dataframe` — needs a map sample.
- `dataframe_to_vec` / `SerdeRows` — the REVERSE direction (R data.frame →
  `Vec<satellite::Reading>`); this is the one with a live gctorture concern.
- `vec_to_dataframe_split` Collated + PerVariantListWithTag shapes (the existing
  fixture only does `PerVariantList`).
- `par_iter_to_dataframe` (rayon path) for a satellite type — gate on `rayon`.
- Each is a small `#[miniextendr]` fn over `sample_*` data. Grep the serde module
  for exact fn names/signatures before writing: `grep -rn "pub fn result_to_dataframe\|pub fn map_to_dataframe\|pub fn dataframe_to_vec\|pub fn par_iter_to_dataframe" miniextendr-api/src/serde/`.

#1069 — R-side coverage for the bridge:
- `rpkg/tests/testthat/test-satellite-bridge.R` (does not exist yet): call every
  bridge fn; assert column types (NA in `note`, i64 `readings_taken` as numeric,
  nested `site_lat`/`site_lon`), enum split yields per-variant data.frames,
  `satellite_echo_reading(list(...))` round-trips.
- gctorture fixture per the #430 convention: no-arg `gc_stress_satellite()` in
  `rpkg/src/rust/gc_stress_fixtures.rs` driving `vec_to_dataframe(&satellite::sample_readings())`
  AND the split path — the serde columnar path stores SEXPs across allocations in
  generic-list buffers, so this is required. Verify it fails on a deliberately
  broken (unrooted) variant, passes fixed.
- Regen loop ×2 for the new exports; commit NAMESPACE + man in sync.

**PR 2 — #1071 + #1072** (JSON bridge + hybrid pattern doc; both small, both
extend the demonstration surface):

#1071 — serde_json bridge fixture:
- `satellite_readings_json() -> AsJson<Vec<satellite::Reading>>` + a `FromJson`
  round-trip fn, gated on `serde_json`. Note: enabling `satellite` pulls `serde`
  only, not `serde_json` — the fixture must `#[cfg(feature = "serde_json")]` and
  rely on the detect-features set enabling both (verify per the precondition).
- Confirm the JSON adapter names: `grep -rn "AsJson\b\|FromJson\|AsJsonVec" miniextendr-api/src/`.

#1072 — document the hybrid data+handle pattern:
- `docs/SERDE_R.md` section + a fixture: a package-crate newtype wrapping
  `satellite::Reading`, `#[derive(ExternalPtr)]` on the wrapper (live handle,
  mutated in place) PLUS serde bridge fns (snapshot its data to/from R). Shows
  escalating from "data only" to "live object" without the satellite crate
  depending on miniextendr.

**PR 3 — #1070** (doc + optional guard; standalone, no fixture surface):
- `docs/SERDE_R.md` "Satellite crates" troubleshooting note: the serde
  version-unification requirement (cargo unifies satellite's `serde = "1"` with
  api's serde for any semver-compatible `^1`), the failure signature if an
  incompatible major coexists (`E0277: the trait Serialize is not implemented`),
  and how to diagnose.
- Optional: a `minirextendr_doctor()` check warning when a bridged path dep
  resolves serde to a different major than miniextendr-api. Nice-to-have; the
  doc note is the must-ship. If doing the check, locate doctor's dep-resolution
  probes first (`grep -n "doctor\|cargo metadata" minirextendr/R/doctor.R`).

**PR 4 — #1067** (codegen macro — LAST, after the surface is proven by hand):
- `miniextendr_serde_bridge! { satellite::Reading => { df: readings_df, list: readings_list, from_r: echo_reading } }`
  expanding to the `#[miniextendr]` bridge fns. Only worth building once #1068's
  hand-written fixtures prove exactly what the macro must emit.
- Also evaluate the `TryFrom<&[T]>/TryFrom<Vec<T>> for DataFrame` sugar
  (compiles; `TryFrom<T>` hits E0119 vs core's reflexive blanket) so a bridge
  body can write `rows.try_into()`. Doesn't remove the named-export requirement,
  so only valuable combined with the macro.
- This is the biggest of the batch and pure ergonomics — defer if the batch
  runs long; the hand-written bridges from PR 1 are fully functional without it.

## Cross-cutting

- All four PRs: run under the satellite feature locally
  (`CARGO_FEATURES="...satellite serde_json..." just rcmdinstall && just devtools-test`
  — export in the SAME shell for install AND test; MEMORY:
  lesson_cargo_features_env_per_shell).
- Doubles as living documentation of `docs/SERDE_R.md`'s capability table —
  keep that table in sync as fixtures land.

Done per issue: `Closes #1068/#1069` (PR1), `#1071/#1072` (PR2), `#1070` (PR3),
`#1067` (PR4), with the CI-compiles-satellite precondition resolved up front.
