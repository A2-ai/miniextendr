# Site documentation build emitted warnings

## What was attempted

Generate the site-facing manual from `docs/` and build it with Zola after the
documentation drift sweep.

## What went wrong

The build exited successfully but reported seven broken internal anchors and
four unsupported syntax-highlighting language labels.

## Root cause

Several hand-written anchors still reflected older GitHub-style slug spelling
(`_`, `/`, and doubled punctuation), while Zola generated normalized hyphenated
IDs. A few fenced blocks used `default` or `m4`, neither of which is present in
the site's configured syntax set. The GC guide also linked to a preserve-list
section that had never been added.

## Fix

Point links at the generated Zola slugs, add the missing Preserve List section,
and mark unhighlighted output/m4 snippets as `text`. A second `just site-docs`
and `just site-build` pass completed without warnings.
