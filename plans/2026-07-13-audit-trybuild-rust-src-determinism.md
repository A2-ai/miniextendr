# Plan: audit 2026-07-12 P1 — make `just test` deterministic across contributor toolchains (trybuild / rust-src)

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `ci/audit-trybuild-rust-src-determinism`.

Covers 2026-07-12 audit worklist item 9 — **with a corrected diagnosis**.

## Corrected diagnosis (the audit's framing is partially wrong)

The audit attributed the 5 failing macro UI snapshots to "Rust 1.97 added
source text under core/src/panic.rs" on the floating `stable` channel.
The actual, previously-nailed mechanism (established while fixing PR
#1239's CI breakage; recorded in project memory) is the **rust-src
component**, not the channel/version: toolchains with `rust-src` installed
render stdlib source snippets (e.g.
`$crate::panicking::panic_fmt($crate::const_format_args!($($t)+));`) into
trybuild `.stderr` output; CI's `dtolnay/rust-toolchain@stable` minimal
profile (`.github/workflows/ci.yml:226-227` and siblings) has no rust-src
and renders bare `= note:` fallbacks. Verified on 656e5cdd:

- No committed snapshot contains the stdlib span text
  (`grep -rn 'panicking::panic_fmt' miniextendr-macros/tests/ui/` → empty),
  i.e. the committed baselines are the CI (no-rust-src) flavor.
- `rust-toolchain.toml` pins only `channel = "stable"` (2 lines, no
  profile/components field — and a profile field could not *remove* an
  already-installed component anyway).
- CI passed on the same commit the audit's local run failed — consistent
  with component skew, not version skew.
- The 5 files (audit evidence): `derive_dataframe_enum_nested_enum_b1_custom_tag_sibling.rs`,
  `derive_dataframe_enum_nested_inner_variant_field.rs`,
  `derive_dataframe_enum_nested_inner_variant_field_custom_tag.rs`,
  `derive_dataframe_enum_struct_field_no_derive.rs`,
  `derive_dataframe_enum_three_level_nesting_payload_collision.rs` —
  all panic-adjacent derives whose diagnostics quote stdlib spans.

So: **CI is authoritative and healthy; the defect is that the advertised
local `just test` (justfile:307-318, plain `cargo test --workspace` on the
active toolchain) is non-deterministic across contributor component sets**
— which still violates the project's no-known-issues principle. The audit
finding survives, narrowed to that residue.

## Work items (flat order)

1. **Deterministic UI-test toolchain in the recipe.** Teach the test path
   to run `cargo test -p miniextendr-macros --test ui` under a toolchain
   that matches CI's component set:
   - detect the hazard: `rustup component list --installed | grep -q
     rust-src` on the active toolchain;
   - when present, run the UI test under a *version-named* toolchain
     installed with `--profile minimal` (a version-named toolchain, e.g.
     `1.97.0`, is a separate rustup toolchain from `stable` even at the
     same version, so it carries no rust-src). Resolve the version from
     the active stable (`rustc --version`), `rustup toolchain install
     <ver> --profile minimal` if missing, then
     `RUSTUP_TOOLCHAIN=<ver> cargo test -p miniextendr-macros --test ui`;
   - when absent, run in place (today's behavior — matches CI).
   Implement as a small `test-ui` recipe that `just test` invokes (keep
   the rest of justfile:307-318's workspace/cross-package/rpkg legs
   unchanged; exclude the ui test from the first leg or let the double-run
   stand — implementer picks the cheaper wiring and says so in the PR).
2. **Fallback posture if item 1 proves too brittle** (e.g. offline
   environments where rustup can't install): degrade to detect-and-warn —
   `just test` prints a loud, actionable notice ("rust-src detected; UI
   snapshots will not match CI — run `just test-ui` / see docs") and skips
   only the trybuild target, never silently failing. Do NOT silently pass.
3. **Document the mechanism where contributors will hit it.** CLAUDE.md's
   "UI test snapshots" section (and the AGENTS.md mirror — hand-kept)
   currently says regenerate with `TRYBUILD=overwrite` with no rust-src
   caveat. Add: never `TRYBUILD=overwrite` under a rust-src-equipped
   toolchain when stdlib spans are involved; CI is authoritative; how to
   reproduce CI exactly (`rustup toolchain install <ver> --profile
   minimal` + `RUSTUP_TOOLCHAIN=...`).
4. **Leave `rust-toolchain.toml` floating** (`channel = "stable"`) — the
   audit's alternative (pinning the snapshot toolchain repo-wide) trades
   one skew for a standing version-bump chore and doesn't fix the
   component-set problem. State this trade-off in the PR body. If the
   maintainer prefers a hard pin, that is a one-line follow-up, not this
   plan.

## Exact commands (worktree)

```bash
just test 2>&1 > /tmp/audit-trybuild-test.log     # Read it — must be green on a rust-src toolchain
rustup component list --installed | grep rust-src  # confirm the hazard is present locally
just test-ui                                       # new recipe, green
cargo clippy --workspace --all-targets --locked -- -D warnings   # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Acceptance evidence: run `just test` twice, once on the rust-src-equipped
default toolchain and once with `RUSTUP_TOOLCHAIN=<ver-minimal>` — both
green, zero snapshot churn.

## Must NOT touch

- The 5 (or any) `.stderr` snapshots — no `TRYBUILD=overwrite`; the
  committed baselines are correct for CI.
- CI's toolchain steps (ci.yml:226-227 etc.) — CI is the reference
  behavior, unchanged.
- `rust-toolchain.toml` (per item 4).

## Done criteria

- `just test` is green on a rust-src-equipped stable toolchain AND on a
  minimal-profile toolchain, with identical snapshot results.
- CLAUDE.md + AGENTS.md carry the rust-src caveat and the exact CI-repro
  command.
- No snapshot files changed; three clippy legs green.

## Escalation rule

If reality diverges from this plan — the version-named-toolchain trick
doesn't isolate rust-src on some platform, trybuild output differs for a
*second* reason (true version skew), or `just test`'s recipe structure
resists the split — **stop, commit nothing further, and report back. Do
not improvise.**
