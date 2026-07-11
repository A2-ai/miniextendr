# Plan: #1261 items 2+4 — Rd duplicate-argument emission bug + 2147483648L test literal

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1261-rd-arg-dedup-int-literal`.

Scope: two of the four standing `R CMD check` WARNINGs from #1261. Item 1
(`.mx_gen` leak) rides the #1248 PR (`plans/2026-07-11-issue-1248-s3-nongeneric-collision.md`);
item 3 (`Rf_findVarInFrame`) has its own plan
(`plans/2026-07-11-issue-1261-item3-findvarinframe.md`). Do NOT touch those here.

## Item 2 — duplicated `\argument` entries `b`, `c` in `fast_fixtures.Rd`

### Root cause (verified)

`rpkg/man/fast_fixtures.Rd:38-44` currently contains:

```
\item{x}{Input value.}
\item{a, b, c}{Numeric scalars.}
\item{b}{(no documentation available)}
\item{c}{(no documentation available)}
```

The user doc on `fast_sum3_default`/`fast_sum3_fast`
(`rpkg/src/rust/fast_fixtures.rs:57,65`) is `/// @param a,b,c Numeric scalars.`
— a roxygen comma-list documenting three params in one tag. The macro's
auto-filler decides "is this param already documented?" by **string prefix
match**:

- `miniextendr-macros/src/lib.rs:1194-1196`:
  `.any(|t| t.trim_start().starts_with(&format!("@param {r_name}")))`

For param `b`, `"@param a,b,c Numeric scalars."` does not start with
`"@param b"` → filler `@param b (no documentation available)` is emitted →
roxygen2 merges the blocks under the shared Rd page → duplicate `\item{b}`
→ `checking Rd \usage sections` WARNING. (`a` escapes only because
`"@param a,b,c…"` happens to start with `"@param a"` — the same mechanism is
also a false-positive generator: a documented `@param x2` would wrongly count
param `x` as documented.)

The identical prefix-match appears at 7 sites total (all must be fixed —
same defect class):

| Site | Form |
|---|---|
| `miniextendr-macros/src/lib.rs:1196` | `.any(starts_with)` |
| `miniextendr-macros/src/r_class_formatter.rs:890` | `.any(starts_with)` |
| `miniextendr-macros/src/miniextendr_impl_trait/r_wrappers.rs:748` | `.any(starts_with)` |
| `miniextendr-macros/src/miniextendr_impl/r6_class.rs:191` | `.any(starts_with)` |
| `miniextendr-macros/src/miniextendr_impl/r6_class.rs:303` | `.any(starts_with)` |
| `miniextendr-macros/src/miniextendr_impl/s7_class.rs:431` | `.any(starts_with)` |
| `miniextendr-macros/src/miniextendr_impl/s7_class.rs:440` | `.find(starts_with)` — returns the matched tag, not a bool |

There is already a canonical name extractor:
`roxygen::extract_param_names` (`miniextendr-macros/src/roxygen.rs:749-761`),
used by the R6 generators for class-level param suppression — but it takes
the whole first whitespace token (`a,b,c`) as ONE name, so it shares the
comma-list defect.

### Fix

1. **`roxygen.rs`**: make `extract_param_names` split the name token on `,`
   (roxygen2's multi-param syntax is comma-separated names without spaces;
   trim each piece defensively anyway; skip empties). Update its rustdoc.
2. **`roxygen.rs`**: add
   `pub(crate) fn param_documented(tags: &[String], name: &str) -> bool`
   implemented as exact-membership against the (comma-split) names of each
   `@param` tag. Do NOT build a HashSet per call inside loops if trivially
   avoidable, but correctness > micro-perf here; a simple per-call scan that
   splits each tag's first token on commas and compares exact strings is fine.
3. Replace the 6 `.any(starts_with)` sites with `param_documented(...)`.
4. `s7_class.rs:440` (`.find(...)` — it *reuses* the matched tag's text):
   replace the predicate with "tag's comma-split name list contains
   `param_name`" so a comma-list tag is found for every name it documents;
   keep the surrounding logic unchanged.
5. Unit tests in the macros crate (beside existing roxygen tests — grep
   `extract_param_names` in `miniextendr-macros/src` for the current test
   module): `extract_param_names(["@param a,b,c desc"])` yields `{a,b,c}`;
   `param_documented` true for `b`, false for `d`; false-positive case:
   `@param x2 desc` does NOT document `x`.

### Regeneration and proof

- Full regen loop (commands below). `rpkg/man/fast_fixtures.Rd` must lose the
  `\item{b}`/`\item{c}` filler lines (the *committed* diff is the proof).
  Other `man/*.Rd` files may lose spurious fillers too — that's the same fix
  working; review the diff, commit all of it.
- `just r-cmd-check` log: the `checking Rd \usage sections ... WARNING`
  (duplicated `\argument` entries) line must be gone.

## Item 4 — `2147483648L` parse warning in a test file

`rpkg/tests/testthat/test-fast-fixtures.R:60`:

```r
e <- tryCatch(fast_i32_default(x = -2147483648L - 1),
```

`2147483648L` is one past `.Machine$integer.max`; the parser warns
`non-integer value 2147483648L qualified with L; using numeric value` and
demotes to double → `checking for unstated dependencies in tests` WARNING
surface. The expression's *value* (`-2147483649`, a double below i32::MIN)
is exactly what the test wants. Replace the expression with the literal
`-2147483649` (no `L`) and keep everything else on that line unchanged.
This is the only such literal in the repo's R sources (verified:
`grep -rn '2147483648L' rpkg/` → only this line; the two hits without `L`
in test comments are prose, untouched).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1       # must print 4.6.x
just worktree-sync                              # FIRST
just configure && just rcmdinstall && just force-document
# (no NEW exports — single install suffices; wrappers.R regen is enough)
cargo test -p miniextendr-macros 2>&1 > /tmp/1261-macros-test.log   # Read it
just test 2>&1 > /tmp/1261-rust-test.log
just devtools-test 2>&1 > /tmp/1261-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1261-devtools.log  # devtools::test always exits 0
just r-cmd-check 2>&1 > /tmp/1261-rcmdcheck.log  # then Read; verify both WARNINGs gone
cargo clippy --workspace --all-targets --locked -- -D warnings   # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Snapshot rebaselining: insta snapshots under `miniextendr-macros/src/**/snapshots/`
may change where fixtures use comma-list `@param` (also check
`snapshot_r6_active_bindings.snap` — it contains a filler line). Diff each
`.snap.new` vs `.snap`; if the only delta is removed spurious fillers, `mv`
over and re-run. trybuild `.stderr`: not expected to change; if one does,
stop and report (never `TRYBUILD=overwrite` locally — #1239).

## Must NOT touch

- Do NOT hand-edit `rpkg/man/fast_fixtures.Rd` — it must change only via the
  regen loop (roxygen2 output).
- `rpkg/R/miniextendr-wrappers.R` / `wasm_registry.rs` (generated, gitignored).
- Items 1 and 3 of #1261 (other PRs). Do not "fix" `.mx_gen` or
  `Rf_findVarInFrame` here even though the same check log shows them.
- Do not flip CI `error-on:` to `"warning"` — that's #1261's close-out step,
  decided by the maintainer after all four items land.

## Done criteria

- `fast_fixtures.Rd` regenerates without duplicate `\item{b}`/`\item{c}`;
  `R CMD check` no longer reports the duplicated-`\argument` WARNING nor the
  `2147483648L` parse warning (2 of the 4 #1261 WARNINGs gone; the log will
  still show the other two — expected, reference their PRs in the PR body).
- Macros unit tests for comma-split param docs pass; all 7 prefix-match
  sites replaced; suites + three clippy legs green.
- PR body references #1261 (partial fix — do NOT `Fixes #1261`).

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, the Rd diff
shows changes beyond removed fillers, a snapshot changes in a way not
explained by the fix — **stop, commit nothing further, and report back.
Do not improvise.**
