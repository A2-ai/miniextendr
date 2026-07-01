# satellite/ — the miniextendr-FREE crate

A deliberately plain Rust data crate used to prove and exercise the "satellite"
architecture: a third-party crate that derives `serde` gets full R interop
through the package crate's bridge, **without ever depending on miniextendr**.

## The one invariant: stay miniextendr-free

**This crate must never depend on miniextendr, R, or any FFI.** Its entire
`Cargo.toml` `[dependencies]` is `serde` (plus serde-ecosystem data crates if a
type genuinely needs them — `chrono`, `uuid`, etc.). The point of the experiment
collapses the moment `miniextendr-api` (or `-macros`, `-lint`) enters this
crate's dependency tree.

Concretely, in `src/**`:
- **No** `use miniextendr_api::...`, no `#[miniextendr]`, no `#[derive(ExternalPtr)]`,
  no `SEXP`, no `IntoR`/`TryFromSexp`.
- **No** `#[serde(crate = "...")]` — this crate owns its own `serde` dependency
  and derives normally. (rpkg's *test* modules use `#[serde(crate = "crate::serde")]`
  because they borrow miniextendr's re-exported serde; this crate does not.)
- Only `#[derive(Serialize, Deserialize)]` and ordinary Rust.

Verify after any dependency change:

```bash
cd rpkg/src/rust/satellite
cargo tree | grep -E 'miniextendr' && echo "INVARIANT BROKEN" || echo ok
# (the repo path contains "miniextendr"; check for a crate line, not the path)
```

## Where the R glue lives

All miniextendr-aware code is in `../satellite_bridge.rs` (the rpkg crate). That
is the *only* place that mentions both worlds. When you add a type here, you add
a `#[miniextendr]` free function there — never the reverse. See
`docs/SERDE_R.md` → "Satellite crates" for the full pattern.

## Build / experiment loop

This crate is a path dependency of `rpkg/src/rust` behind the optional
`satellite` cargo feature. Like the other integration features, it is
auto-enabled by `tools/detect-features.R`, so rpkg's normal build (and CI's
`R CMD INSTALL` + `wrappers-sync-check`) compiles it and generates R wrappers
for the `satellite_*` bridge exports. The fast Rust-only loop needs no R install:

```bash
cd rpkg/src/rust && cargo check --features satellite
```

It is sealed as its own `[workspace]` and excluded from rpkg's workspace, so it
belongs to neither the miniextendr root workspace nor rpkg's.

## What this can and cannot demonstrate

serde moves **values**: structs ↔ lists, `Vec<struct>` → data.frame (with nested
flatten), enums, `Option` → NA, maps, and R → Rust. It cannot demonstrate
**behaviour/identity** — `ExternalPtr` handles, R6/S3/S4/S7 classes and methods,
ALTREP, connections, conditions, dots — those require miniextendr-native code in
the package crate and are out of scope here by design.
