# CI broken: duplicate YAML `env:` key

**Date**: 2026-03-12

## What was attempted

Merging the linkme v2 PRs (#19, #20, #24) into main.

## What went wrong

After merging, all CI runs on main showed "This run likely failed because of a workflow file issue" with zero jobs. No actual CI jobs ran at all.

## Root cause

The CRAN check step in `.github/workflows/ci.yml` had `env: NOT_CRAN: false` twice — once before the `run:` key (from main's existing code) and once after `run:` (added by the packages commit during rebase conflict resolution). YAML mappings don't allow duplicate keys; GitHub Actions silently rejects the entire workflow file.

```yaml
# BROKEN — duplicate `env:` key in same step mapping
- name: R CMD check (CRAN-like, from tarball)
  env:
    NOT_CRAN: false       # ← first
  run: |
    R CMD check --as-cran --no-manual ${{ env.TARBALL }}
  env:
    NOT_CRAN: false       # ← second (duplicate!)
```

## Fix

Removed the first `env:` block, keeping only the one after `run:`. PR #25.

## Lesson

When resolving rebase/merge conflicts in YAML workflow files, always check for duplicate mapping keys — YAML parsers silently pick one (usually the last), but GitHub Actions rejects the file entirely. `ruby -ryaml` and `python -c 'import yaml'` won't catch this because YAML spec says last-key-wins for duplicates.
