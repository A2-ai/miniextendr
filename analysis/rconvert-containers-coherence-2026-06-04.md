# Can `RConvert` newtypes gain `Vec<T>` / `Option<T>` conversions?

**Date:** 2026-06-04
**Issue:** [#844](https://github.com/A2-ai/miniextendr/issues/844) (investigation), part of #835, context from #842
**Toolchain:** `rustc 1.95.0`
**Verdict:** **Mostly yes** — the issue's expected "definitive no" is wrong. Five of the
six container shapes are coherence-sound with a plain api-side blanket. Exactly **one**
shape (`IntoR for Vec<MyNewtype>` — returning a vector of newtypes to R) is blocked, and
that one is solvable by folding one existing blanket into a shared marker.

---

## TL;DR

`#[derive(RConvert)]` (#842) is scalar-only today, and the issue asked whether an
api-side **marker-trait + container blanket** could lift that to `Vec<T>` / `Option<T>` /
`Vec<Option<T>>`. The expectation, recorded in the issue, was that it "cannot."

Empirically (every claim below is a compiled `rustc 1.95` result, repros in the appendix):

| Direction | Container shape | Blanket "slot" today | Verdict |
|---|---|---|---|
| `TryFromSexp` (R→Rust) | `Vec<T>` | empty | ✅ **compiles** |
| `TryFromSexp` | `Option<T>` | empty | ✅ **compiles** |
| `TryFromSexp` | `Vec<Option<T>>` | empty | ✅ **compiles** |
| `IntoR` (Rust→R) | `Option<T>` | empty | ✅ **compiles** |
| `IntoR` | `Vec<Option<T>>` | structurally clear of `MatchArg` | ✅ **compiles** |
| `IntoR` | **`Vec<T>`** | **occupied** by `impl<T: MatchArg> IntoR for Vec<T>` | ❌ **E0119** |

The original PR (#842) and this issue both reasoned from "the orphan rule forbids
`impl TryFromSexp for Vec<MyNewtype>` *in the user crate*." That is true and not the
question here. The question is whether **api-side** blankets keyed on a marker trait
(which the user-crate derive implements — orphan-legal) can do it. They can, for all but
one shape.

---

## The single rule that explains the whole table

> Coherence permits **at most one** open-bound blanket `impl<T: Bound> Trait for Vec<T>`
> (likewise for `Option<T>`). It freely permits that blanket to coexist with any number of
> **concrete** `impl Trait for Vec<i32>` / `Vec<f64>` / … impls.

Two consequences:

1. **Concrete-vs-blanket is fine.** The issue worried that `impl<T: RConvertForward>
   TryFromSexp for Vec<T>` would conservatively conflict with the concrete `Vec<i32>` /
   `Vec<f64>` / … impls because coherence "can't prove `i32: !RConvertForward`." It can.
   `RConvertForward` is **api-local** and `i32` is **foreign**, so by the orphan rule only
   `miniextendr-api` could ever add `impl RConvertForward for i32`, and it hasn't — so the
   compiler *does* perform the negative reasoning. Proof: the existing
   `impl<T: MatchArg> IntoR for Vec<T>` (`match_arg.rs:276`) already coexists today with
   ~30 concrete `impl IntoR for Vec<…>` impls (`into_r.rs`). That is the same shape.

2. **Blanket-vs-blanket is the real wall.** Two open-bound `Vec<T>` blankets with
   *different* marker bounds (`MatchArg` vs `RConvertForward`) overlap, because a
   *downstream* type could implement **both** markers — coherence cannot prove disjointness,
   and stable Rust has no negative bounds. This is identical to the documented reason the
   crate has no `impl<T: RNativeType> IntoR for Vec<T>` blanket (`into_r.rs:343-352`).

So the surviving constraint is purely: **how many `Vec<T>` blanket slots are already
taken, and by what.**

- `TryFromSexp for Vec<T>` slot: **empty** today (the only generic `Vec<…>` `TryFromSexp`
  impls are over *wrapped* types — `Vec<Vec<T>>`, `Vec<ExternalPtr<T>>`, `Vec<RFlags<T>>`,
  `Vec<Map<String,V>>` — which are structurally disjoint from a bare newtype). → A newtype
  blanket can take it.
- `IntoR for Vec<T>` slot: **occupied** by `impl<T: MatchArg> IntoR for Vec<T>`. → A second
  newtype blanket here is E0119.
- `Option<T>` slots (both directions): **empty** (every generic `Option<…>` impl is over a
  wrapped type — `Option<Vec<T>>`, `Option<Map>`, `Option<ExternalPtr<T>>`, …). → Free.

### Why `Vec<Option<T>>` `IntoR` is *not* blocked even though `MatchArg` reaches it

`impl<T: MatchArg> IntoR for Vec<T>` structurally covers `Vec<Option<U>>` (take
`T = Option<U>`). One might expect a newtype `impl<U: RConvertForward> IntoR for
Vec<Option<U>>` to collide. It does **not** (test E). Overlap would require some `U` with
both `U: RConvertForward` **and** `Option<U>: MatchArg`. The latter is provably impossible:
`MatchArg` is api-local and `Option<_>` is a non-`#[fundamental]` foreign type, so only
`miniextendr-api` could write `impl … MatchArg for Option<_>`, and it never will
(`MatchArg: Sized + Copy + 'static` is for unit-ish enums, not `Option`). The compiler
discharges the negative and the impls are disjoint. So only the **bare** `Vec<U>` newtype
blanket — where `U` itself could be `MatchArg` — actually clashes.

---

## Proposed design (agreed direction: derive the traits directly)

Rather than the `#[derive(RConvert)]` umbrella with `#[rconvert(from = …, into = …)]`
toggles, derive the conversion traits **by name**, so directionality is just *which*
derives you list (this also retires the boolean attributes):

```rust
#[derive(TryFromSexp)]              // R -> Rust only (e.g. compiled regex::Regex)
struct Pattern(regex::Regex);

#[derive(TryFromSexp, IntoR)]       // round-trip
struct UserId(uuid::Uuid);
// Vec<UserId>, Option<UserId>, Vec<Option<UserId>> all work — see below.
```

Each derive emits, in the **user** crate (all orphan-legal — local `Self`):

- the **scalar** forwarding impl (`impl TryFromSexp for UserId` / `impl IntoR for UserId`),
  exactly as `#[derive(RConvert)]` does today; **plus**
- a small **marker** impl `impl RConvertForward for UserId { type Inner = uuid::Uuid; … }`
  (foreign trait + local type → legal).

`miniextendr-api` then carries the **container blankets**, keyed on the marker, in the slots
that are free:

```rust
// in miniextendr-api — all coherence-verified (tests B, D, E):
impl<T: RConvertForward> TryFromSexp for Vec<T>          where Vec<T::Inner>: TryFromSexp { … }
impl<T: RConvertForward> TryFromSexp for Option<T>       where Option<T::Inner>: TryFromSexp { … }
impl<T: RConvertForward> TryFromSexp for Vec<Option<T>>  where Vec<Option<T::Inner>>: TryFromSexp { … }
impl<T: RConvertForward> IntoR       for Option<T>       where Option<T::Inner>: IntoR { … }
impl<T: RConvertForward> IntoR       for Vec<Option<T>>  where Vec<Option<T::Inner>>: IntoR { … }
```

Verified end-to-end in a two-crate setup (test G): a downstream `struct UserId(u64)`
resolves `Vec<UserId>: TryFromSexp` (R→Rust) through these blankets with no user-crate
container impl.

### The one remaining blocker and its fix

`impl<T: RConvertForward> IntoR for Vec<T>` (returning `Vec<MyNewtype>` to R) is **E0119**
against `impl<T: MatchArg> IntoR for Vec<T>`. `MatchArg` is the **sole** blocker —
remove it and the newtype blanket compiles (test F). Two ways forward:

1. **Unify the `Vec<T>` `IntoR` slot (recommended if we want full symmetry).** Introduce a
   single api-local element-marker, e.g.

   ```rust
   pub trait RVecElementIntoR: Sized { fn elements_into_sexp(v: Vec<Self>) -> SEXP; }
   impl<T: RVecElementIntoR> IntoR for Vec<T> { fn into_sexp(self) -> SEXP { T::elements_into_sexp(self) } }
   ```

   and route **both** the `MatchArg` derive (its STRSXP-by-variant strategy) **and** the
   newtype derive (forward to `Vec<Inner>`) through `RVecElementIntoR` instead of each owning
   a competing blanket. One slot, two feeders → no overlap (test C compiles; end-to-end
   `Vec<UserId>` `IntoR` works in test G). **Cost:** touches `match_arg.rs` and any other
   future would-be `Vec<T>` `IntoR` blanket must also funnel through this marker.

2. **Document the workaround (zero code).** Returning `Vec<MyNewtype>` is the only gap;
   the body unwraps to the inner type's vector (`fn ids() -> Vec<Uuid>`), which already has
   `IntoR`. This is the status quo answer for *all* shapes today; under this proposal it
   would shrink to just this one shape.

---

## Architectural note: occupying the slots

Adding the `TryFromSexp` `Vec<T>` / `Option<T>` / `Vec<Option<T>>` blankets **consumes**
those (currently empty) blanket slots. After this lands, any *future* feature wanting its
own `impl<T: OtherMarker> TryFromSexp for Vec<T>` blanket will hit the same E0119 wall and
must instead funnel through `RConvertForward` (or a shared element-marker, as in option 1).
That is an acceptable, explicit trade — it is the same single-slot discipline `MatchArg`
already imposes on the `IntoR` side — but it should be called out in the api docs next to
each blanket so the constraint is discoverable rather than rediscovered.

---

## Appendix: reproductions

All compiled with `rustc 1.95.0`, `--crate-type=lib --edition 2021`. Each models the real
`miniextendr-api` impl topology (concrete `Vec<i32>`/`Option<i32>`, the structural
`Vec<Vec<T>>`/`Vec<ExternalPtr<T>>`/`Option<Vec<T>>` blankets, and `match_arg.rs:276`).

**Test A — `IntoR for Vec<T>` newtype blanket vs `MatchArg` blanket → E0119:**
```rust
impl<T: MatchArg> IntoR for Vec<T> { /* … */ }                       // existing
impl<T: RConvertForward> IntoR for Vec<T> where Vec<T::Inner>: IntoR { /* … */ }  // proposed
// error[E0119]: conflicting implementations of trait `IntoR` for type `Vec<_>`
```

**Test B — `TryFromSexp for Vec<T>` newtype blanket vs concrete `Vec<i32/f64/String/bool>`
+ structural `Vec<Vec<T>>` / `Vec<ExternalPtr<T>>` / `Vec<RFlags<T>>` → compiles.**

**Test C — single unified `impl<T: RVecElementIntoR> IntoR for Vec<T>` blanket, fed by both
a match-arg enum and a newtype, alongside concrete `Vec<i32/f64/String>` → compiles.**

**Test D — `TryFromSexp`/`IntoR for Option<T>` newtype blankets vs concrete `Option<i32/String>`
+ structural `Option<Vec<T>>` / `Option<ExternalPtr<T>>` → both compile.**

**Test E — `TryFromSexp`/`IntoR for Vec<Option<T>>` newtype blankets vs `MatchArg` `Vec<T>`
blanket → both compile** (`Option<U>` provably never `MatchArg`).

**Test F — Test A with the `MatchArg` blanket removed → compiles** (confirms `MatchArg` is
the sole blocker for the `Vec<T>` `IntoR` shape).

**Test G — two crates (`api` + downstream `user`):** `struct UserId(u64)` with an
orphan-legal `impl RConvertForward for UserId` resolves `Vec<UserId>: TryFromSexp`
(empty-slot blanket) and `Vec<UserId>: IntoR` (unified element-marker) end-to-end, with no
container impl in the `user` crate.
