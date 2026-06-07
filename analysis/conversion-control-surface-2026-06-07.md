# The `As*` / `Prefer*` conversion-control surface: taxonomy and reshape

**Date:** 2026-06-07
**Issue:** investigation (no issue yet); follows the #835 conversion-dedup sprint and the
#844 RConvert reshape (`analysis/rconvert-containers-coherence-2026-06-04.md`)
**Toolchain:** `rustc 1.95.0`
**Decision locked (this session):** keep the `convert::As*` *wrapper* family; **rename the
`as_coerce::As*` S3-coercion *trait* family** out of the `As*` namespace. The rename is what
unblocks a symmetric matrix, because it frees `AsDataFrame` / `AsFactor` / … for the wrapper
side.

---

## TL;DR

"Which R representation does this Rust type take?" is currently answered by **four
overlapping mechanisms**, and the `As*` prefix is used for **two unrelated** ones. The
literal questions that prompted this — *is there an `AsExternalPtr`? a `PreferExternalPtr`?*
— both answer **yes, and both work** (`convert.rs:346`, `list_derive.rs:354`). ExternalPtr is
in fact the **best-covered** representation. The problems are:

1. **`As*` is overloaded.** `convert::AsList` (a newtype that forces `IntoR` → R list) and
   `as_coerce::AsList` (a trait that exposes `as.list(x)` to R callers) are different
   mechanisms with the same name in the same crate. `as_coerce.rs:14` already warns about the
   footgun.
2. **Coverage is asymmetric.** Factor and Vctrs — the exact enum representations we care
   about — are wired into *neither* the wrapper family *nor* the `Prefer*` family.
3. **The `Prefers*` marker traits are vestigial** — declared, derived, re-exported, but never
   used as a trait bound (`0` `T: PrefersX` sites). Each `Prefer*` derive emits its `IntoR`
   impl *directly*; the marker carries no dispatch weight.
4. Minor: `prefer = "native"` is wired oddly, `prefer=` accepts no `factor`/`vctrs`, derive is
   `Prefer*` but marker is `Prefers*`, and `Collect*` wrappers skip the `As*` convention.

The fix is mechanical and low-risk. Renaming the S3 family frees the namespace so the
wrapper / `Prefer*` / `prefer=` axes can be made symmetric over the representations that don't
already cover themselves.

---

## Shipped (2026-06-07)

Implemented on `refactor/rcoerce-and-conversion-symmetry`. Two refinements to the §4 plan
surfaced during implementation and are recorded here:

- **Factor needs nothing.** `#[derive(RFactor)]` already emits `impl IntoR` → factor
  (`factor_derive.rs:259`), so it *is* the factor representation default. An `AsFactor` wrapper
  would be pointless (the type already converts to a factor unwrapped) and a `PreferFactor`
  derive would emit a conflicting second `IntoR`. So the factor row of the matrix is filled by
  `RFactor`, not by new `As*`/`Prefer*` constructs.
- **Vctrs goes through the derive, not `prefer=`.** `#[derive(Vctrs)]` requires its own
  `#[vctrs(class = …)]` metadata, so a self-contained `#[miniextendr(prefer = "vctrs")]` mode
  (like `list`/`dataframe`/`externalptr`) is impossible. Vctrs is wired via `#[derive(Vctrs,
  PreferVctrs)]` (type default) and `AsVctrs<T>` (call-site) instead.

What landed:

1. **§4.1 rename** — `as_coerce` → `r_coerce`; the 15 `As*` traits → `RCoerce*`; `AsCoerceError`
   → `RCoerceError`. Method names (`as_list`, …) and the `#[miniextendr(as = "…")]` attribute are
   unchanged. The crate-root `AsList as AsListCoerce` alias hack is gone.
2. **§4.3 markers removed** — all four `Prefers*` traits deleted; the `Prefer*` derives keep
   emitting their `IntoR` impl directly.
3. **§4.2 gaps** — added `convert::AsDataFrame<T: IntoDataFrame>` (T = `Vec<Row>`),
   `convert::AsVctrs<T: IntoVctrs>` (vctrs-gated, `Error = VctrsBuildError`), and the
   `PreferVctrs` derive. `NamedList`/`NamedVector`/`Serialize`/`Json` remain call-site-only
   (A-axis) by design — a type-level "always serialize via serde / always a named vector"
   default is rarely wanted and easy to add later if a use case appears.

Final coverage after this PR — every representation now reachable at the call site **and** as a
type-level default, except the deliberately A-only ones:

| Representation | call-site | type default |
|---|---|---|
| List | `AsList` | `PreferList` / `#[miniextendr(list)]` |
| DataFrame | `AsDataFrame` ✅ new | `PreferDataFrame` / `#[miniextendr(dataframe)]` |
| ExternalPtr | `AsExternalPtr` | `PreferExternalPtr` / `#[miniextendr(externalptr)]` |
| RNative scalar | `AsRNative` | `PreferRNativeType` / `prefer = "native"` |
| Factor | (returned directly) | `#[derive(RFactor)]` |
| Vctrs | `AsVctrs` ✅ new | `#[derive(Vctrs, PreferVctrs)]` ✅ new |
| NamedList / NamedVector / Serialize / Json | `AsNamedList` / `AsNamedVector` / `AsSerialize` / `AsJson*` | — (A-only by design) |

---

## 1. The four control surfaces

| # | Mechanism | Where | Scope | Drives | Direction |
|---|---|---|---|---|---|
| A | `convert::As*<T>` newtype wrappers | `convert.rs` | one return value | `IntoR` | Rust→R |
| B | `#[derive(Prefer*)]` | `list_derive.rs` | type-level default | emits `IntoR` directly | Rust→R |
| C | `#[miniextendr(prefer="…"/list/dataframe/…)]` | `struct_enum_dispatch.rs` | type-level | selects a derive path | Rust→R |
| D | `as_coerce::As*` traits | `as_coerce.rs` | exposes `as.X()` to **R callers** | S3 dispatch via `#[miniextendr(as="…")]` | (method surface) |

**A — wrappers** (`struct AsFoo<T>(pub T)` implementing `IntoR`):
`AsList` (`convert.rs:86`), `AsExternalPtr` (`:346`), `AsRNative` (`:398`), `AsNamedList`
(`:470`), `AsNamedVector` (`:565`), `AsDisplay`/`AsDisplayVec` (`:785`/`:812`),
`AsFromStr`/`AsFromStrVec` (`:846`/`:888`, these are inbound `TryFromSexp`), and the
non-conforming `Collect`/`CollectStrings`/`CollectNA*` iterator adapters (`:957`+). Plus the
serde wrappers `AsSerialize` (`serde/traits.rs`) and `AsJson`/`AsJsonPretty`/`AsJsonVec`
(`serde/json_string.rs`). Ergonomic `.wrap_list()` / `.wrap_external_ptr()` / … extension
traits live alongside (`convert.rs:673`+).

**B — `Prefer*` derives.** Each emits the marker **and** a full `IntoR` impl:
`PreferList`→`into_list().into_sexp()` (`list_derive.rs:314`),
`PreferExternalPtr`→`ExternalPtr::new(self).into_sexp()` (`:354`),
`PreferDataFrame`→`ColumnSource::into_column_list(self)` (`:394`),
`PreferRNativeType`→`AsRNative(self).into_sexp()` (`:435`).

**C — type attribute.** `parse_attrs` (`struct_enum_dispatch.rs:55`) accepts path flags
`list`/`dataframe`/`externalptr`/`match_arg`/`factor` and `prefer="externalptr"|"list"|
"dataframe"|"native"` (`:44`, validated `:252`).

**D — S3 coercion traits** (`trait AsFoo { fn as_foo(&self) -> Result<_, AsCoerceError> }`):
the 15 in `as_coerce.rs:192`–`:325`, dispatched by the `#[miniextendr(as="data.frame")]`
attribute, with the generic→method table at `as_coerce.rs:338` and the codegen mapping at
`miniextendr_impl.rs:3019`.

---

## 2. The coverage matrix (the asymmetry)

| Representation | A: `As*` wrapper | B: `Prefer*` derive | C: `prefer=`/flag | D: S3 `as_coerce` |
|---|---|---|---|---|
| List | ✅ `AsList` | ✅ `PreferList` | ✅ `list` | ✅ `AsList` |
| DataFrame | ❌ *(name taken by D)* | ✅ `PreferDataFrame` | ✅ `dataframe` | ✅ `AsDataFrame` |
| ExternalPtr | ✅ `AsExternalPtr` | ✅ `PreferExternalPtr` | ✅ `externalptr` | ❌ |
| RNative scalar | ✅ `AsRNative` | ✅ `PreferRNativeType` | ✅ `native` | ❌ |
| **Factor** | ❌ *(name taken by D)* | ❌ *(separate `RFactor` derive)* | ⚠️ `factor` flag, **no** `prefer="factor"` | ✅ `AsFactor` |
| **Vctrs** | ❌ | ❌ | ❌ *(separate `Vctrs` derive)* | ❌ |
| NamedList / NamedVector | ✅ | ❌ | ❌ | ❌ |
| Serialize / Json | ✅ `AsSerialize`/`AsJson*` | ❌ | ❌ | ❌ |
| character/numeric/integer/logical/matrix/vector/Date/POSIXct/complex/raw/environment/function | ❌ | ❌ | ❌ | ✅ |

Reading the table: ExternalPtr and List+RNative are the only fully-covered outbound paths.
DataFrame can only be defaulted (B/C) — there is **no call-site wrapper** to force it on one
return value, because `AsDataFrame` the name is occupied by the D trait. **Factor and Vctrs**
sit entirely outside the A/B/C axes despite being the canonical multi-representation enum
targets.

---

## 3. The four inconsistencies, precisely

**3.1 — `As*` namespace collision (headline).** `AsList` is *both* a `struct` (A,
`convert.rs:86`) and a `trait` (D, `as_coerce.rs:216`). `AsDataFrame`/`AsFactor`/… exist only
as D traits today, which is exactly why A can't offer `AsDataFrame`/`AsFactor` wrappers. The D
module already documents the confusion: *"an `AsList` impl will not satisfy a `fn foo(_:
MyType)` argument coming from R — that needs `TryFromSexp`"* (`as_coerce.rs:14`).

**3.2 — Factor/Vctrs gap.** The motivating use case ("an enum could be a factor, a vector, a
list, or a vctrs") is only half-wired. `RFactor` and `Vctrs` are standalone derives with no
`As*` wrapper, no `Prefer*` derive, and no `prefer=` value. You can't force factor/vctrs on a
single return, nor express "this enum defaults to factor" through the same surface as the
others.

**3.3 — Vestigial `Prefers*` markers.** Verified: `PrefersList`/`PrefersDataFrame`/
`PrefersExternalPtr`/`PrefersRNativeType` (`markers.rs:200`–`:215`) are **never used as a
bound** anywhere — only declared, impl'd by the derives, and re-exported (`lib.rs:736`). The
`IntoR` impl is emitted *directly* by each derive (§1.B), not via a blanket
`impl<T: PrefersX> IntoR for T`. **This is forced, not accidental:** four marker-keyed
blankets would mutually overlap (a type could implement two markers) — the same
blanket-vs-blanket E0119 wall documented in the #844 analysis. So the markers *cannot*
cheaply become load-bearing for dispatch. Today they're pure tags, and the "currently
informational" caveat is applied to only 2 of the 4 (`markers.rs:209`, `:214`) when it is true
of all 4 — a documentation inconsistency.

**3.4 — Minor.**
- `prefer = "native"` maps to "ExternalPtr + PreferRNative marker" (`struct_enum_dispatch.rs:189`,
  `:313`), which reads inconsistently against the `AsRNative` scalar path.
- Stacking two `Prefer*` derives on one type is a (desirable) mutual-exclusion, but surfaces as
  a cryptic E0119 on `IntoR` rather than a guided error.
- Derive is `Prefer*`; marker trait is `Prefers*` (third-person `s`).
- `Collect*` (A) are representation-forcing wrappers that don't follow the `As*` naming.

---

## 4. Proposed reshape

### 4.1 Rename the S3 coercion family out of `As*` (the unblocking move)

Decision: **keep `convert::As*` wrappers as-is**; move the D traits to an unambiguous prefix.
Recommended `RCoerce*` (ties to the existing `AsCoerceError` and to "R's coercion generics";
avoids colliding with the lax `coerce::Coerce` path):

| Today (D) | Proposed |
|---|---|
| `as_coerce::AsList` … `AsFunction` (15 traits) | `r_coerce::RCoerceList` … `RCoerceFunction` |
| `as_coerce::AsCoerceError` | `r_coerce::RCoerceError` |
| module `as_coerce` | module `r_coerce` |

Keep the **method names** (`as_list`, `as_data_frame`, `as_factor`, …) and the
`#[miniextendr(as = "…")]` attribute spelling — both correctly mirror R's `as.X()` generics
and are not the source of confusion. The only mechanical edits:

- `as_coerce.rs` → `r_coerce.rs`: rename the 15 traits + error enum (definitions).
- `lib.rs:736`-area re-exports (`pub use as_coerce::{…}` and the `AsCoerceError` re-export).
- `miniextendr_impl.rs:3019`–`3046`: the `(trait_name, trait_method)` table and the two
  `as_coerce::AsCoerceError` paths in the generated return types.

Blast radius confirmed by grep — only those three files reference the D traits / `AsCoerceError`.

### 4.2 Fill the matrix using the freed names

With D vacated, A and B can be made symmetric:

- **Add `convert::AsDataFrame<T>`** wrapper (name now free) → call-site `-> AsDataFrame<T>`,
  routing through `ColumnSource::into_column_list` (mirrors `PreferDataFrame`).
- **Add factor + vctrs to every axis:** `convert::AsFactor<T>` / `convert::AsVctrs<T>`
  wrappers; `PreferFactor` / `PreferVctrs` derives that wrap the existing `RFactor` / `Vctrs`
  derive output in an `IntoR` impl; `prefer = "factor"` / `prefer = "vctrs"` values in
  `struct_enum_dispatch.rs` (and fold the existing bare `factor` flag into the unified set).
- **Decide on `NamedList`/`NamedVector`/`Serialize`/`Json`:** these have wrappers (A) but no
  `Prefer*`. They're arguably call-site-only by nature (named pairs / serde framing rarely
  want to be a *type's* permanent default), so leaving them A-only is defensible — but call it
  out explicitly so the asymmetry is a decision, not an oversight.

### 4.3 Resolve the vestigial markers

Pick one (this is the main open decision — see §6):
- **(a) Remove** `Prefers*` (4 trait decls + 4 derive lines + re-export). Simplest; the
  derives keep working unchanged since they emit `IntoR` directly.
- **(b) Keep as reflection tags**, but make the "informational" note uniform across all 4 and
  document that they are intentionally not dispatch bounds (coherence forbids it — link the
  #844 reasoning).

### 4.4 Fix the `prefer=` quirks

Normalize the `prefer=` value set to exactly mirror the representation list (add `factor`,
`vctrs`; reconcile `native`), and emit a guided compile error when two type-level preferences
are requested rather than letting it fall through to a raw E0119.

### 4.5 Naming nits

Rename `Collect*` → conforming wrapper names or document why they're exempt; consider aligning
the `Prefer*` derive / `Prefers*` marker spelling if the markers survive §4.3.

---

## 5. Blast radius summary

| Change | Files touched | Risk |
|---|---|---|
| §4.1 rename D → `RCoerce*` | `as_coerce.rs`, `lib.rs`, `miniextendr_impl.rs` | low (3 files; regenerate wrappers + UI snapshots) |
| §4.2 add wrappers/derives/values | `convert.rs`, `list_derive.rs`, `struct_enum_dispatch.rs`, `lib.rs`, macro `lib.rs` derive registrations | medium (new codegen; needs fixtures) |
| §4.3 markers | `markers.rs`, `list_derive.rs`, `lib.rs` | low |

Because the project carries no backwards-compatibility constraint (unreleased), the rename can
be a clean break. Standard post-change loop applies: `just configure && just rcmdinstall &&
just force-document`, regenerate `tests/ui/*.stderr`, and commit `wrappers.R`/`NAMESPACE`/`man`
in lockstep. New `As*`/`Prefer*` paths that store SEXPs across allocations need a no-arg
`gc_stress_*` fixture (per #430).

---

## 6. Decisions (resolved)

1. **§4.3 — markers:** **removed** all four `Prefers*` traits (no consumer existed).
2. **§4.2 — scope:** **one PR**, filling the gaps that don't cover themselves (DataFrame
   call-site wrapper, vctrs both axes). Factor needed nothing (RFactor); NamedList/serde left
   A-only.
3. **§4.1 — prefix:** **`RCoerce*`** (avoids confusion with the lax `coerce::Coerce` trait).
