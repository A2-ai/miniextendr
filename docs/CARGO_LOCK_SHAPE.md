# Cargo.lock shape: why it's not just a Cargo.lock

The committed `Cargo.lock` in a miniextendr-based R package is **not** a vanilla
Cargo.lock. It's in a specific shape — *tarball-shape* — that the offline
install path needs. Every R package built with miniextendr (the example `rpkg/`
in this repo, and any package scaffolded via `minirextendr`) ships its
`src/rust/Cargo.lock` in this shape.

If you've never thought about it, it's because the maintainer recipes
(`just vendor`, `miniextendr_vendor()`) produce the right shape automatically.
But every cargo build that runs with the dev `[patch."git+url"]` override
silently dirties it, and the canonical regen is `just vendor` or
`miniextendr::miniextendr_vendor()`. The pre-commit hook +
`just lock-shape-check` keep you honest.

This page explains what the shape is, why it exists, and how to recover when
it drifts.

## What "tarball-shape" means

One invariant on `src/rust/Cargo.lock`:

1. **No `source = "path+..."` entries** for any crate that's published or
   workspace-internal to the miniextendr framework
   (`miniextendr-api`, `miniextendr-lint`, `miniextendr-macros`).
   These crates *must* carry `source = "git+https://github.com/A2-ai/miniextendr#<commit>"`.

> **Note:** `checksum = "..."` lines are now **allowed** in the committed
> lock. `cargo-revendor` recomputes valid `.cargo-checksum.json` files
> after CRAN-trim, with the original `package` field (matching the registry
> checksum) preserved and the `files` map updated to reflect post-trim disk
> contents. Cargo's offline source-replacement verifies both successfully.

`just lock-shape-check` (and the equivalent pre-commit hook) asserts the
`path+` invariant only.

## Why the invariant

### `source = "git+url#commit"` for framework crates

The dev workflow uses cargo's `[patch."https://github.com/A2-ai/miniextendr"]`
mechanism (in `src/rust/.cargo/config.toml`) to redirect
`miniextendr-{api,lint,macros}` to either monorepo siblings (in this repo) or
to a checked-out copy. When cargo resolves the lock under that patch, it
records the resolved entries with `source = "path+file:///..."`.

That `path+...` entry is **fatal at offline install time**: the install
machine doesn't have `/home/your-username/checkout/...`. Even if it did,
the path would be different. The lock has to record a portable identifier
that source replacement can match against vendored sources — and that's the
git URL plus commit hash.

So the regen flow is:

1. Move `.cargo/config.toml` aside (so the patch override is inactive).
2. Regenerate the lockfile against the bare git URL — entries for
   miniextendr crates resolve to `source = "git+https://...#<commit>"`.
3. Restore `.cargo/config.toml`.
4. Run `cargo revendor` — it recomputes `.cargo-checksum.json` for each
   crate after CRAN-trim, so the lock's `checksum =` lines stay valid.

That's exactly what `just vendor` (in this repo) and
`miniextendr::miniextendr_vendor()` (for scaffolded packages) do.

## The cross-crate cargo-surface chicken-and-egg

The regen flow above (move `.cargo/config.toml` aside, `cargo generate-lockfile`,
move back) has one corner case it can't solve: **a PR that changes the cargo
surface of `miniextendr-{api,macros,lint}` and the rpkg consumer in the same
commit.** Typical example: renaming a feature like `default-coerce` →
`coerce-default` everywhere.

Symptom — `just vendor`, `Bootstrap Vendor Test`, R CMD check, CRAN-like check
all fail with:

```
error: failed to select a version for `miniextendr-api`.
package `miniextendr` depends on `miniextendr-api` with feature `coerce-default`
but `miniextendr-api` does not have that feature.
 available features: ... default-coerce, default-r6, default-s7, default-strict, ...
```

Why: with the patch override moved aside, `cargo generate-lockfile` follows
the bare git URL (`https://github.com/A2-ai/miniextendr`, no rev) to origin's
default branch (`main`). main still has the *old* feature names; the PR's
rpkg/src/rust/Cargo.toml asks for the *new* names; cargo errors. The fix
would be to make cargo resolve the api crate at the PR's own HEAD instead
of main — but cargo has no clean mechanism for that.

### Why cargo can't fix it cleanly

Cargo's lockfile model fundamentally couples source attribution with
resolution. To get `source = "git+url#<sha>"` recorded, cargo must
*resolve* the dep against that git URL — it can't be told to "resolve
locally but record a git source." Specifically:

| Attempted lever | Cargo's response |
|---|---|
| `dep = { path = "...", git = "..." }` | rejected: *"specification is ambiguous. Only one of `git` or `path` is allowed."* |
| `[patch."url"] dep = { git = "url", rev = "..." }` (same URL) | rejected: *"patches must point to different sources."* |
| Same-URL patch via different scheme (`https` vs `ssh`) | URLs are normalized; same rejection. |
| Hand-inject `source = "git+url#<sha>"` into Cargo.lock | cargo strips it on the next `cargo metadata` / `cargo build` — the patch override is authoritative for source attribution. |
| `cargo update -p <crate> --precise <sha>` | only works for already-resolved deps; can't bootstrap from the failing resolve. |

The one lever that *does* work is modifying the dep declaration itself to
include `rev = "<HEAD_SHA>"` — but that means mutating
`rpkg/src/rust/Cargo.toml` (a tracked file) for the duration of the regen,
which is its own correctness hazard (trap-restore, SIGKILL window, accidental
commit). Not worth permanent machinery.

### Policy: admin-merge after eyeballing

Cross-crate cargo-surface changes happen rarely (feature renames, removing
a public re-export, etc.). When the bootstrap test fails on such a PR:

1. **Read the diff.** Confirm the rename is coordinated — the new feature/symbol
   exists on miniextendr-api *and* is what rpkg/src/rust/Cargo.toml asks for.
   Check `git log --stat` for the PR shows changes to both crates.
2. **Admin-merge.** The CI failure is a transitional state, not a real defect.
   Once the PR lands on main, the next bootstrap test on any unrelated PR
   will pass — the rename is now what main has.
3. **Do not** add temporary `rev` pins to `rpkg/src/rust/Cargo.toml`,
   modify `just vendor` to inject rev pins, or otherwise build permanent
   infrastructure to make this single class of PR pass CI. The Bootstrap
   Vendor Test failing on cross-crate renames is a *correct signal* that
   deserves human eyeballing, not automated bypass.

This was explored as part of PR #710 (`default-* → *-default` rename) and
PR #735 (the rev-pin workaround, withdrawn). See those PRs for the
investigation trail.

## When does the lock drift?

Any cargo invocation that runs with the patch override active will rewrite
the lock:

- `just check` / `just clippy` / `just test` (rpkg variants)
- `cargo build --manifest-path rpkg/src/rust/Cargo.toml`
- `R CMD INSTALL` in source mode (no `inst/vendor.tar.xz`)
- `devtools::document()` / `devtools::install()` / `devtools::test()`
- `just devtools-document` (because it shells out to the above)

After any of these, you'll see (under `git diff`):

- `source = "git+...#<commit>"` lines deleted from
  `miniextendr-{api,lint,macros}` (they become path deps via `[patch]`)

`checksum = "..."` lines may also be added/changed by cargo build, but
those are now harmless — `just vendor` will put them back in sync.

**This drift is expected and harmless for local iteration. Don't commit it.**
The pre-commit hook will block the `path+` drift. Re-run the canonical
regen (`just vendor` or `just update`) before staging.

## Recovering a drifted lock

```bash
# Easiest path — full regen, also rebuilds inst/vendor.tar.xz
just vendor

# Lock-only regen (skips the heavy vendor/ + tarball step)
just update                # this repo
miniextendr::miniextendr_vendor()  # scaffolded packages

# Manual minimum (what the recipes do under the hood)
mv rpkg/src/rust/.cargo/config.toml /tmp/cargo-config.toml.bak
rm rpkg/src/rust/Cargo.lock
cargo generate-lockfile --manifest-path rpkg/src/rust/Cargo.toml
mv /tmp/cargo-config.toml.bak rpkg/src/rust/.cargo/config.toml
# No checksum strip needed — cargo-revendor handles it during `just vendor`

# Verify
just lock-shape-check
```

## Verifying

`just lock-shape-check` in this repo, or for any miniextendr-based package:

```bash
# Equivalent shell check
grep -q 'source = "path+' src/rust/Cargo.lock && echo "BAD: contains path+ sources"
```

## What about `inst/vendor.tar.xz`?

The vendor tarball is a separate artifact:

- This repo: **gitignored**. Regenerated by CI's `just vendor` before every
  R CMD check. Never committed (binary blob, 22 MB/commit historically).
- Scaffolded packages: **typically also gitignored** — generated by
  `miniextendr_vendor()` only at release time. CRAN submissions ship the
  tarball *inside* the source `.tar.gz` (because it lives at
  `inst/vendor.tar.xz`), but it's regenerated, not tracked.

The lockfile's tarball-shape is independent of whether `vendor.tar.xz`
currently exists. The lock just has to be in the shape that *would* work
when the tarball is present and source replacement kicks in. The pre-commit
hook + `lock-shape-check` enforce this even when the tarball is absent.

## See also

- [CRAN compatibility][cran-compat] — the install-mode decision tree, what
  triggers source vs tarball mode, the maintainer release workflow.
- [`cargo-revendor` README][cargo-revendor] — the vendoring tool that
  produces the matched `vendor/` tree from a tarball-shape lock.
- Cargo book: [source replacement][cargo-source-replacement] — the offline
  install mechanism that depends on the lock being in this shape.

[cran-compat]: ./CRAN_COMPATIBILITY.md
[cargo-revendor]: ../cargo-revendor/README.md
[cargo-source-replacement]: https://doc.rust-lang.org/cargo/reference/source-replacement.html
