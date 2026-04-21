# Site Documentation

Pages in `site/content/manual/` are auto-generated from `docs/` by
`scripts/docs-to-site.sh` (1:1 conversion with TOML frontmatter derived
from each file's `# Title` line). **Edit files in `docs/`, not
`site/content/manual/`** — any direct edit there gets overwritten on
the next regeneration.

After editing anything under `docs/`, regenerate before committing:

```bash
bash scripts/docs-to-site.sh
git add docs/ site/content/manual/
```

CI runs `scripts/docs-to-site.sh` itself before each Zola build, so the
deployed site is always correct — but the in-repo `site/content/manual/`
drifts out of sync if you skip the local regenerate step, which makes
diffs noisy and masks unrelated site edits.

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
bash scripts/docs-to-site.sh
cd site && zola check
```

`zola check` validates both internal links (anchors, page paths) and
external URLs. Internal breakage means a cross-reference in `docs/`
points at a file that no longer exists; fix in `docs/`, not the
generated page.

Other files under `site/` (`templates/`, `sass/`, `static/`,
`config.toml`, landing pages like `content/_index.md` and
`content/getting-started.md`) are hand-written — edit those directly.
