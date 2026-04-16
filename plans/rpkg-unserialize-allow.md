# Plan: remove `#[allow(clippy::not_unsafe_ptr_arg_deref)]` from rpkg `LazyIntSeqData::unserialize` (#100)

## Goal

Delete the last `#[allow(clippy::not_unsafe_ptr_arg_deref)]` in the repo,
located at `rpkg/src/rust/lib.rs:619`, on
`impl AltrepSerialize for LazyIntSeqData :: fn unserialize`. Close out issue
#100 (the remaining instance after #92).

## Why the lint fires today

`SEXP` is a `#[repr(transparent)]` wrapper around `*mut SEXPREC`
(`miniextendr-api/src/ffi.rs:147`). Clippy's `not_unsafe_ptr_arg_deref`
triggers on `fn unserialize(state: SEXP)` because the non-`unsafe` function
receives a raw-pointer-bearing type and then dereferences it (through
`SexpExt::integer_elt` which calls `*self`).

Most other `AltrepSerialize::unserialize` impls in the codebase delegate to
`TryFromSexp::try_from_sexp(state)` and do not trigger the lint — their body
does not textually deref `state`; clippy's heuristic only flags when the
function body clearly derefs the raw-pointer parameter.

## Approach (minimal, idiomatic)

Rewrite `LazyIntSeqData::unserialize` to reconstruct the struct via
`TryFromSexp::try_from_sexp::<Vec<i32>>(state)` rather than hand-rolling
`integer_elt(0..2)` calls. The builtin `Vec<i32>` unserialize path already
exists (`miniextendr-api/src/altrep_data/builtins.rs:145`) and does not
deref `state` in its own body, so it doesn't trip the lint.

Pseudocode:

```rust
impl AltrepSerialize for LazyIntSeqData {
    fn serialized_state(&self) -> SEXP {
        vec![self.start, self.step, self.len as i32].into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        let v: Vec<i32> = TryFromSexp::try_from_sexp(state).ok()?;
        if v.len() != 3 {
            return None;
        }
        Some(LazyIntSeqData {
            start: v[0],
            step: v[1],
            len: v[2] as usize,
            materialized: None,
        })
    }
}
```

This removes both `#[allow]` sites naturally. Side bonus: `serialized_state`
also gets simpler.

## Work items (flat, prioritized)

1. **Rewrite `serialized_state`**: `vec![start, step, len as i32].into_sexp()`.
2. **Rewrite `unserialize`**: use `TryFromSexp::try_from_sexp::<Vec<i32>>`;
   drop the `#[allow(clippy::not_unsafe_ptr_arg_deref)]` attribute; return
   `None` on unexpected length or conversion failure.
3. **Verify with the CI clippy incantations** (from CLAUDE.md — both must
   pass on the worktree's toolchain):
   - `cargo clippy --workspace --all-targets --locked -- -D warnings`
     (manifest = `rpkg/src/rust/Cargo.toml`; that's where the allow lives)
   - `cargo clippy --workspace --all-targets --locked --features <CI list> -- -D warnings`
     (the feature list from CLAUDE.md's "Reproducing CI's clippy" section)
4. **Run the normal local lint recipes as well**:
   - `just clippy` — must be clean.
   - `just check` — must be clean.
5. **Rebuild & regen R wrappers** (because rpkg Rust changed):
   - `just configure`
   - `just rcmdinstall` (sandbox-disabled: the Bash tool call must pass
     `dangerouslyDisableSandbox: true` per CLAUDE.md "Sandbox Restrictions").
   - `just devtools-document` (same sandbox caveat).
   - Confirm `rpkg/R/miniextendr-wrappers.R` and `rpkg/NAMESPACE` are
     unchanged (behaviour-preserving edit — they should be identical).
6. **Regenerate vendor tarball** as the final commit (mandatory per
   CLAUDE.md "Before Opening a PR"):
   - `just vendor` (already passes `--force` since
     commit `5bee807d`).
   - Commit the regenerated `rpkg/inst/vendor.tar.xz`, `rpkg/vendor/`,
     `rpkg/src/rust/Cargo.toml`, `rpkg/src/rust/Cargo.lock`.
7. **Rebase onto `origin/main`** immediately before pushing.
8. **Delete this plan file** in the final commit.

## Non-goals

- Changing the `AltrepSerialize` trait signature (e.g. making `unserialize`
  take `&SEXP` or be `unsafe fn`). Scope creep; would affect every impl in
  the codebase plus cross-package ABI.
- Eliminating other `#[allow(clippy::...)]` attributes elsewhere. Only this
  one remains of the `not_unsafe_ptr_arg_deref` category per issue #100.

## Validation

- `just clippy` — clean.
- Both CI clippy invocations (see CLAUDE.md) — clean.
- `just check` — clean.
- `just test` — Rust unit tests pass (no test touches this path directly,
  but defensive).
- `just devtools-test` — the `test-altrep.R` suite round-trips `lazy_int_seq`
  through `base::serialize` / `base::unserialize`, which exercises this exact
  path. Must stay green.
- `rpkg/R/miniextendr-wrappers.R` / `rpkg/NAMESPACE` / `rpkg/man/*.Rd` —
  unchanged (pure internal refactor).
- `rpkg/inst/vendor.tar.xz` — regenerated.

## Branch / PR

- Branch: `fix/rpkg-unserialize-allow`
- Base: fresh `origin/main` (fetch + rebase immediately before push).
- PR title: `fix(rpkg): eliminate last not_unsafe_ptr_arg_deref allow in LazyIntSeqData (#100)`
- PR body: short summary, the reasoning above ("unserialize → TryFromSexp
  avoids manual deref"), and a checklist mirroring Validation.
  `Closes #100.`

## Expected diff shape

- `rpkg/src/rust/lib.rs` — ~15 lines changed (simpler `serialized_state`
  + rewritten `unserialize`, minus the `#[allow]`).
- `rpkg/inst/vendor.tar.xz`, `rpkg/vendor/**`, `rpkg/src/rust/Cargo.toml`,
  `rpkg/src/rust/Cargo.lock` — regenerated by `just vendor`.

If the diff touches anything else, stop and re-read the plan.
