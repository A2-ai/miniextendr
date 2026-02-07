# `rpkg/configure.ac` Review (2026-02-07)

## Scope Reviewed

- `rpkg/configure.ac`
- Generated script behavior via `rpkg/configure`
- Consumer templates:
  - `rpkg/src/Makevars.in`
  - `rpkg/src/rust/cargo-config.toml.in`

## Findings (Ordered by Severity)

### [P1] `CARGO_PROFILE=debug` is documented but broken at runtime

- `rpkg/configure.ac:195`
- `rpkg/src/Makevars.in:81`

`configure.ac` says `CARGO_PROFILE` supports `debug` or `release`, but Makevars passes it to `cargo ... --profile $(CARGO_PROFILE)`. Cargo rejects `--profile debug` because `debug` is reserved; the development profile name is `dev`.

Impact: builds fail if users follow the documented `debug` value.

### [P2] Cross-compilation default target can be invalid for Rust

- `rpkg/configure.ac:210`

When cross-compiling, `CARGO_BUILD_TARGET` defaults to Autoconf `$host`. That host triple is not always a valid Rust target triple (example: `x86_64-pc-linux-gnu` vs Rust `x86_64-unknown-linux-gnu`).

Impact: cross builds may fail unless users manually override `CARGO_BUILD_TARGET`.

### [P2] `post-vendor` lockfile update can drift versions and hides failures

- `rpkg/configure.ac:448`
- `rpkg/configure.ac:449`

`cargo update --workspace` may update dependency versions, but comment text claims no version change. The command is also masked with `|| true`, and success is always logged.

Impact: silent lockfile drift and false-positive success logs.

### [P3] `FORCE_VENDOR` and `VENDOR_SYNC_EXTRA` are currently no-ops

- `rpkg/configure.ac:278`
- `rpkg/configure.ac:284`
- `rpkg/configure.ac:432`

Both vars are declared and passed into command environments, but vendoring logic does not branch on them.

Impact: confusing knobs that imply behavior that does not exist.

## Remediation Plan

1. Fix `CARGO_PROFILE` contract.
- Accept `dev` and `release` as canonical values.
- Optionally map legacy `debug` -> `dev` with a warning for compatibility.
- Update docs/messages and any comments mentioning `debug`.

2. Make cross-target derivation Rust-aware.
- Prefer explicit `CARGO_BUILD_TARGET` when provided.
- Otherwise map common Autoconf triples to Rust target triples, or skip auto-setting and emit a clear notice requiring explicit override for cross builds.

3. Make lockfile maintenance deterministic.
- Replace `cargo update --workspace` with a non-upgrading operation or remove this step if unnecessary.
- Remove `|| true`; fail loudly when lockfile sync action fails.
- Update comments so they match actual behavior.

4. Resolve no-op vendor flags.
- Either implement `FORCE_VENDOR` / `VENDOR_SYNC_EXTRA` behavior in `cargo-vendor` command logic, or remove these variables to reduce config surface.

## Validation Checklist After Changes

- `autoconf -Wall -Werror configure.ac` passes.
- `autoreconf -fi` passes.
- `./configure` succeeds in:
  - Dev mode (`NOT_CRAN=true`)
  - CRAN/offline mode (`NOT_CRAN=false`) with vendored input
- `CARGO_PROFILE=dev` and `CARGO_PROFILE=release` both build successfully.
- Cross-compilation path either:
  - uses a valid Rust target triple automatically, or
  - emits actionable guidance requiring explicit `CARGO_BUILD_TARGET`.

## Notes

- No code changes were made as part of this review artifact.
