# Plan: audit 2026-07-12 P1 — fix the 11 broken documentation links and gate `zola check`

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `docs/audit-doc-link-anchors`.

Covers 2026-07-12 audit worklist item 10. Distinct from the older on-disk
`plans/2026-07-01-docs-drift-sweep.md` (index/mechanism prose items — no
overlap).

## Verified state (all on 656e5cdd)

Why the split between GitHub rendering (mostly fine) and the site (`zola
check` fails): **GitHub's and Zola's heading slugifiers disagree** on
em-dashes, parentheses, underscores, and slashes. Also,
`site/config.toml:11-12` sets `[link_checker] internal_level = "warn"`,
which is why the Pages deploy (`zola build`, pages.yml:157) stays green
while `zola check` fails. Remember: `site/content/manual/` is generated
from `docs/` by `scripts/docs-to-site.sh` and gitignored — **edit
`docs/*.md` only**.

The 7 internal failures:

| Source link | Target heading | Status |
|---|---|---|
| `docs/DATAFRAME.md:189` → `#map-fields--parallel-list-column-expansion` | `DATAFRAME.md:191` `### Map fields — parallel list-column expansion` | GitHub OK (em-dash → `--`), Zola slug differs → site-broken |
| `docs/DATAFRAME.md:189` → `#nested-enum-fields--flatten--opt-outs` | `DATAFRAME.md:239` `### Nested enum fields — flatten + opt-outs` | same em-dash problem |
| `docs/RAYON.md:212` → `#with_r_vec` (evidence counts two refs — grep repo-wide) | `RAYON.md:100` `#### \`with_r_vec(len, f)\`: chunk-based parallel fill` | broken on BOTH renderers (slug has the full heading text) |
| `docs/SERDE_R.md:68` → `#nanull-handling` | heading `NA/NULL Handling` | GitHub OK (`/` dropped), Zola inserts a hyphen → site-broken |
| `docs/GC_PROTECT.md:17` → `#preserve-list` | **no such heading exists** (verified: heading list has no "Preserve" entry; the concept appears in prose at :276 and tables at :311-325) | broken everywhere |
| `docs/ALTREP.md:1057` → `#mutable-vectors-set_elt` | `ALTREP.md:593` `## Mutable Vectors (Set_elt)` | GitHub OK, Zola normalizes `_`/parens differently → site-broken |

The 4 external failures (GitHub line anchors `zola check` can't verify):

- `docs/RELEASE_WORKFLOW.md:140` and `docs/CRAN_COMPATIBILITY.md:295` →
  `r-devel/r-svn .../R-admin.texi#L5854` (unpinned `/master/` ref, so the
  line number also rots);
- `docs/RELEASE_WORKFLOW.md:178` → `.../setup-macos-tools/action.yml#L17-L28`
  and `:232` → `#L44-L53` (already commit-pinned `@ec72e88`, but the
  fragment still fails the checker).

## Work items (flat order)

1. **Fix the em-dash/slash/paren anchors by making the slugs
   renderer-stable.** Preferred: reword the four headings so GitHub and
   Zola produce the same slug (e.g. `### Map fields: parallel list-column
   expansion`, `### Nested enum fields: flatten and opt-outs`,
   `## NA and NULL handling`, `## Mutable vectors: set_elt`), then update
   every in-repo reference to each heading (`grep -rn` the old fragments
   across docs/, rustdoc comments, and skills). If rewording a heading is
   undesirable somewhere, the alternative is an explicit Zola anchor — but
   confirm docs-to-site.sh passes it through and GitHub tolerates the
   syntax before choosing it; do not mix strategies per file.
2. **Fix `#with_r_vec`**: retarget the RAYON.md:212 link (and the second
   reference the evidence saw — grep repo-wide for `#with_r_vec`) to the
   real heading's slug, or give the `with_r_vec` section a short stable
   heading. Verify on both renderers.
3. **Fix `#preserve-list`**: the target does not exist. Either add a
   proper `## Preserve list` section to GC_PROTECT.md (the content largely
   exists in the :272-325 ProtectPool comparison prose — a short section
   explaining `R_PreserveObject`-based rooting fits the doc's structure)
   or retarget the strategies-table link at :17 to where the concept
   actually lives. Adding the section is preferred: the table row promises
   a strategy the doc never describes.
4. **External line anchors**: replace the four with durable forms —
   pin the r-svn links to a commit (not `/master/`) and drop the `#L`
   fragment in favor of naming the section in the link text ("R-admin,
   'Building binary packages'"), same for the action.yml refs (keep the
   `@ec72e88` pin, name the step instead of line-ranging it). If any deep
   link is genuinely worth keeping, add
   `skip_anchor_prefixes = ["https://github.com/"]` under
   `[link_checker]` in `site/config.toml` — but prefer removing the rot.
5. **Flip `internal_level = "warn"` → `"error"`** in `site/config.toml`
   once items 1-3 pass, so future internal-anchor rot fails loudly.
6. **Gate it in CI.** Add a `just site-check` recipe (`just site-docs`
   to generate the manual, then `cd site && zola check`), and a step in
   `.github/workflows/pages.yml` (before the `zola build` at :157) or the
   ci.yml Sync Checks job that runs it. Note `zola check` performs network
   fetches for external links — if flakiness is a concern, split:
   internal-error level enforced in CI, external checking documented as
   the quarterly-audit command; state the choice in the PR body.

## Exact commands (worktree)

```bash
just site-docs && just site-build              # regenerate manual + build (do not commit output)
cd site && zola check 2>&1 > /tmp/audit-zola-check.log   # Read it — must be clean
just site-check                                 # new recipe, green
```

No R, no cargo.

## Must NOT touch

- `site/content/manual/**`, `site/public/**` (generated, gitignored —
  nothing to commit on the site side).
- Doc *content* beyond headings/links needed for anchor stability.
- `scripts/docs-to-site.sh` unless an explicit-anchor strategy (item 1
  alternative) requires a passthrough fix — if so, keep it minimal and
  test both renderers.

## Done criteria

- `zola check` on the freshly generated site reports zero broken internal
  anchors and zero of the four external failures.
- The same links resolve on GitHub's rendering of `docs/*.md` (spot-check
  each changed anchor).
- CI gate in place (`just site-check` + workflow step);
  `internal_level = "error"`.

## Escalation rule

If reality diverges from this plan — Zola's slugifier behaves differently
than described for a reworded heading, docs-to-site.sh mangles a chosen
anchor form, or `zola check`'s external fetching is too flaky to gate —
**stop, commit nothing further, and report back. Do not improvise.**
