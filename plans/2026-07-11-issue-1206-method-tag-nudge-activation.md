# Plan: #1206 — activate the impl-level method-tag nudge (sweep first, then activate)

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1206-method-tag-nudge`.

Decision baked in (per the addendum): **FIX, not delete** — activate the
nudge. Two commits in one PR, in this order (bisectable; CI failure points at
real stragglers):

1. commit 1 — mechanical sweep deleting every impl-level method-only tag;
2. commit 2 — activation in `roxygen.rs` + tests.

Related: #1118 (made the const names unique — merged), #1242 (sibling static
family — PR #1277 in flight, `miniextendr_impl.rs` overlap is possible; if
#1277 merges first, rebase).

## Mechanism (verified)

`strip_method_tags` (`miniextendr-macros/src/roxygen.rs:697-740`) and
`strip_method_tags_r6` (`:821`, same emission shape at `:850`) emit, per
stripped method-only tag, a `#[deprecated] #[doc(hidden)] #[allow(dead_code)]
const _MINIEXTENDR_IMPL_METHOD_TAG_WARN_<Type>_<block>_<n>: () = ();`.
An unused deprecated const warns NOWHERE — dead code implying a feature.
Call sites: `miniextendr_impl.rs:2470,2477`,
`miniextendr_impl_trait.rs:355`, `miniextendr_impl_trait/vtable.rs:147`
(all consume the returned `warnings` TokenStream — no call-site changes
needed; activation is internal to the two roxygen.rs fns).

## Commit 1 — sweep (mechanical, behavior-preserving)

1. Discovery procedure (do this FIRST, with the activation change applied
   locally but NOT committed):
   - Apply the item-3 activation edit to `roxygen.rs`.
   - `cargo clippy --workspace --all-targets --locked 2>&1 | grep -n "has no effect" > /tmp/1206-inventory.log`
     — also run the `clippy_all` and `clippy_all_s7` feature legs (lists from
     `.github/workflows/ci.yml` `clippy_all` step) and append, since
     feature-gated fixture modules only compile there. Then ALSO build the
     cross-package crates (they are separate workspaces):
     `cd tests/cross-package/producer.pkg/src/rust && cargo clippy --all-targets 2>&1 | grep "has no effect"`,
     same for `consumer.pkg`. Read the logs.
   - `git stash` the activation edit (or `git checkout -- miniextendr-macros/src/roxygen.rs`)
     before making sweep edits, so commit 1 contains ONLY deletions.
2. For every inventory hit: **DELETE the flagged tag line(s)** from the impl
   block's roxygen. Do NOT move tags onto methods — moving changes generated
   docs; deleting is exactly behavior-preserving (the tags are already
   stripped). If a deleted tag reads like documentation worth keeping, list
   it in the PR body under "candidates to re-add on methods" — do not act.
   Known sites (issue body; expect ~40 tags total): `rpkg/src/rust/class_system_matrix.rs`
   (`@param x` on an S4 trait impl), `rpkg/src/rust/s3_tests.rs`,
   `rpkg/src/rust/pipe_builder_tests.rs`,
   `tests/cross-package/producer.pkg/src/rust/lib.rs` (`@examples` on impl
   blocks). The inventory log is authoritative, not this list.
   NOTE: R6 class-level `@param` is INTENTIONALLY kept by
   `strip_method_tags_r6` (roxygen2 8.0.0 inherits class-level params) — the
   nudge consts are only emitted for tags the fns actually strip, so the
   inventory can't contain legitimate R6 class-level `@param`s. Trust the
   inventory.
3. Behavior-preservation proof: run the regen loop (commands below) and
   verify `git status` shows NO diff in `NAMESPACE` / `man/*.Rd`
   (wrappers.R is gitignored; tracked artifacts must be byte-identical).
   Commit 1 = fixture/test-crate deletions only.

## Commit 2 — activation

3. In BOTH `strip_method_tags` (`roxygen.rs:731-737`) and
   `strip_method_tags_r6` (same shape at `:850` area): after the existing
   WARN const, emit a sibling use-const per the issue's sketch:
   ```rust
   #[doc(hidden)]
   #[allow(dead_code, non_upper_case_globals)]
   const _MINIEXTENDR_IMPL_METHOD_TAG_USE_<same suffix>: () = _MINIEXTENDR_IMPL_METHOD_TAG_WARN_<same suffix>;
   ```
   and add `#[allow(non_upper_case_globals)]` to the WARN const's attrs
   (names embed mixed-case type names). Keep `quote_spanned!{span}` so the
   warning points at the impl block.
4. Update `roxygen/tests.rs` (region at `:660`) — assert the warnings
   TokenStream now contains both consts and the USE initializer references
   the WARN ident.
5. New trybuild fixture in `miniextendr-macros/tests/ui/`:
   `#![deny(deprecated)]` crate with `/// @param x nope` on a
   `#[miniextendr]` impl block → compile error whose message contains
   `has no effect — move it to the method`. Generate the `.stderr` via
   `TRYBUILD=overwrite cargo test -p miniextendr-macros` (allowed for NEW
   fixtures; if CI's stderr differs from local — the rust-src span lesson,
   #1239 — take CI's output verbatim, never re-overwrite locally to fight it).
6. All three clippy legs + both cross-package crates clippy-clean (the
   activation must produce ZERO warnings after commit 1's sweep — any
   remaining warning is a straggler; delete its tag in commit 1, amend).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document
# (sweep+activation change no exports — single install; verify no NAMESPACE/man diff)
cargo test -p miniextendr-macros 2>&1 > /tmp/1206-macros.log   # Read it
just test 2>&1 > /tmp/1206-rust-test.log
just devtools-test 2>&1 > /tmp/1206-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1206-devtools.log  # devtools::test always exits 0
just cross-install && just cross-test 2>&1 > /tmp/1206-cross.log
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- The R6 class-level `@param` retention logic in `strip_method_tags_r6`.
- `METHOD_ONLY_TAGS` membership (no new tags in/out of the strip set).
- Generated artifacts must show zero diff after commit 1 (that IS the check).
- No edits to `#1242`'s class-name-entry statics (`miniextendr_impl.rs:3225`
  area) — separate PR #1277.

## Done criteria

- A method-only tag on an impl block produces a visible deprecation warning
  at the impl-block span (trybuild fixture pins message + span shape).
- Workspace + rpkg + cross-package carry zero such tags; three clippy legs
  green; regen loop produces no tracked-artifact diff; `Fixes #1206`.

## Escalation rule

If reality diverges from this plan — the use-const trick does not fire the
lint on the current toolchain, the inventory shows a tag whose deletion is
not behavior-preserving, NAMESPACE/man diff appears after the sweep — **stop,
commit nothing further, and report back. Do not improvise.**
