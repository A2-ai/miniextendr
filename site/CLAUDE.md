# Site Documentation

Pages in `site/content/` are generated from `docs/` via the justfile (`just site-sync` or similar). **Edit files in `docs/`, not `site/content/`**. The justfile converts them to Zola-compatible format with TOML frontmatter.

If you need to update documentation content, edit the corresponding file in `docs/` at the repo root. The site build pipeline handles the rest.
