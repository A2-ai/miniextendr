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

1. Resolve the lockfile with the dev `[patch."<git-url>"]` override **active**,
   against the local workspace checkout. This is what makes a coordinated
   cross-crate change resolve correctly (see below). The framework crates land
   as local (no-`source`) entries at this point.
2. Run `cargo revendor` — it **stamps** `source = "git+https://...#<commit>"`
   back onto those entries (reconstructing the portable attribution that offline
   source replacement matches), and recomputes `.cargo-checksum.json` for each
   crate after CRAN-trim, so the lock's `checksum =` lines stay valid.

That's exactly what `just vendor` / `just update` (in this repo) and
`miniextendr::miniextendr_vendor()` (for scaffolded packages) do. There is no
longer a "move `.cargo/config.toml` aside and resolve against bare git" step —
that was what broke cross-crate renames (#883).

## The cross-crate cargo-surface case (solved by #883)

There was one corner case the **bare-git** regen flow (move
`.cargo/config.toml` aside, `cargo generate-lockfile`, move back) could never
solve: **a PR that changes the cargo surface of `miniextendr-{api,macros,lint}`
and the rpkg consumer in the same commit.** Typical example: renaming a feature
like `default-coerce` → `coerce-default` everywhere.

With the patch override moved aside, `cargo generate-lockfile` followed the
bare git URL (`https://github.com/A2-ai/miniextendr`, no rev) to origin's
default branch (`main`). main still had the *old* feature names; the PR's
`rpkg/src/rust/Cargo.toml` asked for the *new* names; cargo errored — across
`just vendor`, the `Bootstrap Vendor Test`, R CMD check, and the CRAN-like
check:

```
error: failed to select a version for `miniextendr-api`.
package `miniextendr` depends on `miniextendr-api` with feature `coerce-default`
but `miniextendr-api` does not have that feature.
 available features: ... default-coerce, default-r6, default-s7, default-strict, ...
```

### Why cargo can't record a git source from a local resolve

Cargo's lockfile model couples source attribution with resolution. To record
`source = "git+url#<sha>"`, cargo must *resolve* the dep against that git URL —
it can't be told to "resolve locally but record a git source." Every lever that
looks like it might bridge the two is rejected:

| Attempted lever | Cargo's response |
|---|---|
| `dep = { path = "...", git = "..." }` | rejected: *"specification is ambiguous. Only one of `git` or `path` is allowed."* |
| `[patch."url"] dep = { git = "url", rev = "..." }` (same URL) | rejected: *"patches must point to different sources."* |
| Same-URL patch via different scheme (`https` vs `ssh`) | URLs are normalized; same rejection. |
| Hand-inject `source = "git+url#<sha>"` into Cargo.lock | cargo strips it on the next `cargo metadata` / `cargo build` *while the patch is active* — the patch override is authoritative. |
| `cargo update -p <crate> --precise <sha>` | only works for already-resolved deps; can't bootstrap from the failing resolve. |

### How #883 resolves it

Stop pre-resolving against bare git. Keep the `[patch."<git-url>"]` override
**active** so cargo resolves the framework crates against the *local PR
checkout* — which has the renamed feature — and let them land as local
(no-`source`) entries. Then `cargo revendor` **stamps** the `git+<url>#<sha>`
attribution back on, *after* resolution is done.

This decouples *resolution* (local, sees the PR's surface) from *source
attribution* (git URL, what offline replacement needs). The hand-inject lever
above fails only because the patch is still authoritative during a later
re-resolve; the offline tarball build is **frozen** against `vendored-sources`,
so nothing re-resolves and the stamped source survives. The stamped sha is the
framework checkout's live `git HEAD` — cosmetic, since cargo's
`[source."git+<url>"]` replacement keys on the URL, not the commit.

`cargo revendor --stamp-lock` exposes the stamp step on its own (no full
re-vendor), for the lock-only `just update` recipe.

> Historically this was handled by **admin-merging the PR red** after eyeballing
> the diff — the CI failure was treated as a transitional state. PR #710
> (`default-* → *-default`) was the last PR to need that; PR #735 explored a
> `rev`-pin workaround (withdrawn). #883 removed the need for both.

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

**The `just` recipes now self-clean.** Every drifting maintainer recipe
auto-chains `just cargo-lock-restore` as its final step — the Rust-side
`check` / `build` / `clippy` / `test` and the R-side `rcmdinstall` /
`devtools-document` / `force-document` / `devtools-test` / `devtools-install` /
`devtools-load`. `devtools-test` restores even on a red test (via an `EXIT`
trap), so a failing suite never leaves the papercut behind. So after any of
these `just` recipes, `git status` shows the lock clean with no manual
`git checkout` needed; deliberate lock updates still go through `just update` /
`just vendor`, which re-stamp tarball-shape. Raw `R CMD INSTALL` (no `just`)
still drifts and relies on the pre-commit hook + `lock-shape-check` (#1052).

## Recovering a drifted lock

```bash
# Easiest path — full regen, also rebuilds inst/vendor.tar.xz
just vendor

# Lock-only regen (skips the heavy vendor/ + tarball step)
just update                # this repo
miniextendr::miniextendr_vendor()  # scaffolded packages

# Manual minimum (what `just update` does under the hood). Note the [patch]
# override stays ACTIVE — cargo resolves locally, then cargo-revendor stamps
# the git source. Run the cargo update from src/rust/ so cargo's CWD-relative
# config discovery picks up .cargo/config.toml.
( cd rpkg/src/rust && cargo update )
cargo revendor --manifest-path rpkg/src/rust/Cargo.toml --stamp-lock
# No checksum strip needed — checksum lines are retained.

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
