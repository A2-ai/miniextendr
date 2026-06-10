# ALTREP Path (a): declarative macros → proc-macro direct emit (#711)

Spike date: 2026-06-09. Tracks Path (a) from #682 (follow-up to the Path (b)
consolidation landed in PR #713).

## Goal

Make `#[derive(AltrepInteger)]` (+ Real/Logical/Raw/String/Complex/List) parse
the `#[altrep(dataptr|serialize|subset|…)]` attributes and **emit the underlying
trait impls directly**, removing the `impl_alt<family>_from_data!`
declarative-macro indirection. The declarative macros stay public (hand-rolled
`optionals/{arrow,nalgebra,ndarray}_impl.rs` call sites depend on them); the
proc-macro just stops *delegating* to them.

## What this spike actually did

Migrated **one family — integer** — as a feasibility proof. The other 6
families still delegate to their declarative macros (`emit_direct: false`).

### The 6-arm option matrix → proc-macro emit

Every `impl_alt<family>_from_data!` arm expands to exactly four items. The
mapping the proc-macro now reproduces directly:

| Arm | `impl Altrep` | `impl AltVec` | `impl Alt<Family>` | `impl InferBase` |
|---|---|---|---|---|
| `($ty)` | `__impl_altrep_base!(ty, RUnwind)` | `__impl_altvec_integer_dataptr!` (materializing) | `__impl_altinteger_methods!` | `impl_inferbase_integer!` |
| `($ty, dataptr)` | `__impl_altrep_base!(ty, RUnwind)` | `__impl_altvec_dataptr!(ty, i32)` | ″ | ″ |
| `($ty, serialize)` | `__impl_altrep_base!(ty, RUnwind, with_serialize)` | materializing | ″ | ″ |
| `($ty, subset)` | `__impl_altrep_base!(ty, RUnwind)` | `__impl_altvec_extract_subset!` | ″ | ″ |
| `($ty, dataptr, serialize)` | `…RUnwind, with_serialize` | `__impl_altvec_dataptr!(ty, i32)` | ″ | ″ |
| `($ty, subset, serialize)` | `…RUnwind, with_serialize` | `__impl_altvec_extract_subset!` | ″ | ″ |

So the option matrix reduces to three orthogonal booleans driving small,
independent emit decisions:

- `has_serialize` → flips the base macro to the `with_serialize` arm.
- `has_dataptr` vs `has_subset` vs neither → selects the `AltVec` impl.
- (family is fixed per derive entry point → `methods_macro` + `inferbase_macro`).

`validate_options` already rejects `dataptr + subset` and `subset` on
unsupported families at the derive call site, so the proc-macro never has to
emit an illegal combination.

### Implementation shape (the shared emit helper the other 6 families reuse)

The emit lives in `AltrepAttrs::generate_lowlevel_direct` in
`miniextendr-macros/src/altrep_derive.rs`. It is **fully family-generic** — it
reads everything family-specific off the existing `AltrepFamilyConfig`:

- `dataptr_macro: Option<(&str, Option<TokenStream>)>` — the typed dataptr macro
  + element type (already present).
- `methods_macro`, `inferbase_macro`, `default_guard` — already present.
- `materializing_dataptr_macro: &str` — **new** field naming the per-family
  materializing dataptr macro (`__impl_altvec_<family>_dataptr`) used on the
  default/serialize arms.
- `emit_direct: bool` — **new** per-family switch. `generate_lowlevel` early-
  returns into `generate_lowlevel_direct` when set.

To migrate a remaining family, flip `emit_direct: true` and set
`materializing_dataptr_macro` on its `AltrepFamilyConfig` — no new emit code.
The helper already handles every arm for every atomic family. Complex/raw/real/
logical are structurally identical to integer; only the string and list
families need extra attention (see below).

### Folding the two prior code paths into one

Pre-#711 the derive had two paths:

1. **Simple path** (default `RUnwind` guard, no `subset`) → delegate to
   `impl_alt<family>_from_data!`.
2. **Expanded path** (non-default guard OR `subset`) → emit the `__impl_*`
   macros directly.

The expanded path's `AltVec` choice for the *no-dataptr/no-subset* case was a
**bare `impl AltVec {}`** (non-materializing) — different from the simple
path's materializing macro. This was an accident of the split, only reachable
with a non-default guard. The direct path preserves that exact behavior
(branch on `has_non_default_guard()`) so the migration is a pure de-indirection
with zero semantic drift. No production fixture uses a non-default guard on any
ALTREP derive, so this branch is exercised only by the proc-macro unit tests.
(A future cleanup could make non-default-guard + no-options materialize like
everything else — arguably a latent bug fix — but that is out of scope for a
behaviour-preserving spike.)

## Byte-equivalence argument

The emitted trait surface is byte-equivalent to delegating to
`impl_altinteger_from_data!` for every default-guard arm:

- `__impl_altrep_base!(ty)` ≡ `__impl_altrep_base!(ty, RUnwind)` (the no-guard
  arm forwards to the `RUnwind` arm), and `__impl_altrep_base!(ty,
  with_serialize)` ≡ `__impl_altrep_base!(ty, RUnwind, with_serialize)`.
- the `AltVec` / methods / inferbase macros are emitted with identical
  arguments.

Empirical confirmation: `git status` shows `rpkg/R/miniextendr-wrappers.R`,
`rpkg/NAMESPACE`, and `rpkg/src/rust/wasm_registry.rs` **all unchanged** after
the migration. Those artifacts are generated from the emitted trait surface; if
the surface had drifted, they would have changed. `cargo check -p
miniextendr-api -p miniextendr-macros` passes and the full
`cargo test -p miniextendr-macros` suite (324 tests, incl. trybuild UI tests)
is green.

## Estimated effort / risk per remaining family

Per the issue, the heavy work is the first family; the rest reuse
`generate_lowlevel_direct`.

| Family | Effort | Risk | Notes |
|---|---|---|---|
| **real** | trivial | low | structurally identical to integer; flip `emit_direct`, set `materializing_dataptr_macro`. |
| **logical** | trivial | low | same; logical `elt` quirk lives in `__impl_altlogical_methods`, not the emit path. |
| **raw** | trivial | low | same; raw has no `sum/min/max`, irrelevant to emit. |
| **complex** | trivial | low | same; element type is `::miniextendr_api::Rcomplex`. |
| **string** | small | medium | `dataptr_macro = None`, `string_dataptr = true`. The default/dataptr arms both route through `__impl_altvec_string_dataptr!` (no typed contiguous pointer). `generate_lowlevel_direct` needs a string branch: when `string_dataptr`, the `has_dataptr` and default arms both emit `__impl_altvec_string_dataptr!`. Modest extra branch; raw `Rf_protect`/`Rf_unprotect` discipline already lives inside the macro, untouched. |
| **list** | small–medium | medium | List has its **own** derive entry (`derive_altrep_list`) that does not use `AltrepFamilyConfig` at all and already emits `impl Altrep`/`AltVec`/`AltList`/`InferBase` inline for the serialize case. Migrating list means either folding it into the generic helper (it rejects `dataptr`/`subset`, so only the default + `serialize` arms exist) or leaving it as-is (it is already mostly "direct"). Lowest value of the seven. |

Cross-cutting risk for the whole migration: **gctorture sweep**. The integer
spike changed no emitted bytes, so the existing gctorture coverage still
applies. Once a family's emit genuinely diverges (it should not, if done as a
pure de-indirection), run the `gctorture(TRUE)` harness over the ALTREP
fixtures per `docs/GCTORTURE_TESTING.md`.

## Recommendation

**Proceed with the migration via the follow-up issue.** The spike proves the
approach is clean, the emitted surface is byte-equivalent, and the shared
`generate_lowlevel_direct` helper makes the remaining 5 atomic families nearly
mechanical. The declarative macros stay public for the `optionals/*` hand-rolled
call sites, exactly as #711 requires. The only families needing real thought
are string (extra `string_dataptr` branch) and list (separate entry point, low
value). This is the right de-indirection; #711 should be closed by the
follow-up that finishes the remaining families.
