# Site Documentation

Pages in `site/content/manual/` are auto-generated from `docs/` by
`scripts/docs-to-site.sh` (1:1 conversion with TOML frontmatter derived
from each file's `# Title` line). **Edit files in `docs/`, not
`site/content/manual/`** — any direct edit there gets overwritten on
the next regeneration.

`site/content/manual/*.md` is gitignored except `_index.md`. CI
regenerates the manual via `bash scripts/docs-to-site.sh` before each
`zola build` in the pages workflow. Run the script locally before
`just site-build` / `just site-serve` to preview doc edits — the
generated files stay in your working tree but are no longer tracked.

After editing anything under `docs/`, commit only the `docs/` changes:

```bash
# Edit docs/SOMETHING.md, then:
bash scripts/docs-to-site.sh   # regenerate for local preview
just site-serve                # preview (optional)
git add docs/SOMETHING.md      # commit docs/ only — manual/ is gitignored
git commit
```

CI's existing regenerate-before-zola-build step keeps the deployed site
correct. You no longer need to `git add site/content/manual/`.

`just site-build` (`cd site && zola build`) and `just site-serve`
(`cd site && zola serve`) do **not** call `docs-to-site.sh`. Run the
script by hand when previewing doc edits locally.

Inter-doc links in `docs/*.md` use bare `UPPERCASE.md` references
(house style). `docs-to-site.sh` rewrites these to Zola-compatible
`../kebab-case/` paths during conversion, so `[SAFETY.md](SAFETY.md)`
in a doc becomes `[SAFETY.md](../safety/)` in the generated page.
Anchor fragments (`GAPS.md#41-r-...`) are preserved.

Verify links after editing:

```bash
just site-check   # = bash scripts/docs-to-site.sh + cd site && zola check
```

`zola check` validates both internal links (anchors, page paths) and
external URLs. Internal breakage means a cross-reference in `docs/`
points at a file that no longer exists; fix in `docs/`, not the
generated page. `site/config.toml` sets `[link_checker] internal_level =
"error"`, so a broken internal anchor fails the check, not just warns.

CI gates this on every PR that touches `docs/**`, `site/**`, or
`scripts/docs-to-site.sh` (the `Docs Link Check` job in `ci.yml` — separate
from the Pages deploy workflow, so a broken link fails the PR instead of
only being caught after merging to main).

**Cross-renderer anchor gotcha**: GitHub's heading slugifier and Zola's
disagree on several characters — most importantly, Zola always folds
underscores to hyphens (`Set_elt` → `set-elt`) while GitHub preserves them,
and the two disagree on em-dashes, slashes, and parentheses too. A heading
whose text is punctuation-free words joined by a colon or a bare hyphen
(`## NA and NULL handling`, `## Chunk-Based Fill`) slugifies identically on
both; anything with an underscore in the heading text cannot be made to
match on both renderers by rewording alone — retarget the link to a nearby
plain-word heading, or drop the link, rather than inventing a fake spelling
of the identifier. See `plans/2026-07-13-audit-doc-link-anchors.md` for the
full investigation.

Other files under `site/` (`templates/`, `sass/`, `static/`,
`config.toml`, landing pages like `content/_index.md` and
`content/getting-started.md`) are hand-written — edit those directly.
