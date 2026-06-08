# Cargo feature hygiene (closes #678 + #676)

Bundled because both are tiny and live in the same Cargo.toml / cfg surface.

## Part 1: rename `default-*` → `*-default` (closes #678)

Today in `miniextendr-api/Cargo.toml`:
```toml
default-worker = ["miniextendr-macros/default-worker", "worker-thread"]
```

The name suggests "worker is on by default" but it isn't in the `default` feature
set. The issue prefers `worker-default` (codegen-default selector pattern).

Since the project follows "no backwards compatibility", do all five renames in
one shot for consistency:

- `default-worker` → `worker-default`
- `default-r6` → `r6-default`
- `default-s7` → `s7-default`
- `default-strict` → `strict-default`
- `default-coerce` → `coerce-default`

### Sites (per `grep` survey)

- `miniextendr-api/Cargo.toml`
- `miniextendr-macros/Cargo.toml`
- `rpkg/src/rust/Cargo.toml`
- `.github/workflows/ci.yml` (clippy_all feature list)
- `miniextendr-macros/src/lib.rs` (existing `cfg(all(...))` compile_error guard)
- `miniextendr-macros/src/miniextendr_impl.rs` (cfg checks at L800, L802, L1133, L1689, L1691)
- `miniextendr-macros/src/miniextendr_fn.rs` (cfg checks at L1569, L1572, L1581)
- `miniextendr-macros/src/miniextendr_impl_trait.rs` (L763)
- `miniextendr-macros/src/miniextendr_impl_trait/vtable.rs` (L783)
- `miniextendr-macros/src/c_wrapper_builder.rs` (L481)
- `miniextendr-api/src/lib.rs` (rustdoc table)
- `miniextendr-api/README.md` (features table)
- `miniextendr-api/CLAUDE.md`
- `README.md`
- `AGENTS.md`
- `CLAUDE.md` (root)
- `docs/FEATURE_DEFAULTS.md`
- `docs/FEATURES.md`
- `docs/MINIEXTENDR_ATTRIBUTE.md`
- `docs/STRICT_MODE.md`
- `docs/BENCHMARKS.md`
- `site/content/features.md`
- `.claude/skills/miniextendr-architecture/SKILL.md`
- `.claude/skills/miniextendr-macros/SKILL.md`
- `.claude/skills/miniextendr-worker/SKILL.md`
- `rpkg/tests/testthat/test-rng.R` (comment + skip message)

Use `Edit` with `replace_all: true` per file (NOT `sed` — `feedback_sed_substring_rename_gotcha`).

## Part 2: enforce mutex via compile_error! (closes #676)

There's already a `compile_error!` for `default-r6`/`default-s7` at
`miniextendr-macros/src/lib.rs:205-206`. Update the message to match #676's
proposed text (more helpful: explains the fallback) and re-key on the new
feature names.

`strict` and `coerce` are NOT mutually exclusive (docs/FEATURE_DEFAULTS.md
explicitly demonstrates `#[miniextendr(strict, coerce)]` is the default
when both features enabled). No second compile_error needed; called out in PR body.

## Verification

1. `just check` clean
2. `just clippy` clean
3. CI `clippy_all` reproduced locally with new feature names
4. `cargo check --features "r6-default,s7-default" -p miniextendr-macros` fails with new message
5. `just devtools-test` clean
