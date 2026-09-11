# Generated artifacts were entering textual merges (#1085)

The R and CLI scaffolders wrote ignore files but no Git attributes for the
generated artifacts that packages keep tracked. Git therefore used text merges
for NAMESPACE, generated R wrappers, configure, and Rd files. Rerere could record
and replay resolutions of those generated conflicts after their source changed.

The initial public-scaffolder regressions reproduced seven missing-attribute
failures. The first upgrade fixture also lacked the existing package detector's
required Cargo.toml and Makevars.in files; completing that fixture let it exercise
the real upgrader. The initial merge test could not run before the new attribute
writer existed. With the writer and scaffold wiring in place, all 64 focused
assertions passed without warnings or skips.

Add the four -merge rules in rpkg/.gitattributes and both package templates.
The monorepo root template uses recursive patterns so arbitrary package paths
are covered. All three templates are mapped to the master in justfile and the
approved patch is regenerated. R and CLI scaffolding append missing lines while
preserving existing rules; R upgrades add the package rules as well.

The Git regression creates two branches with different generated contents,
enables rerere, and performs a conflicting merge. All four files stay unmerged
in the index, retain the current branch's contents without conflict markers,
and produce no rerere preimages. Handwritten R/Rust files and configure.ac retain
the normal merge behavior. The upgrade help and template README explain that
users must regenerate from the merged sources and git add the results.

Validation: the full minirextendr suite passes 933 assertions after stacking
on PR #1495, with zero failures/warnings and 11 existing skips. The original
standalone branch passed 855 assertions. just fmt, just check, full just test,
just clippy with -D warnings, and all three exact CI Clippy configurations pass.
The R configure/install/force-document loop and minirextendr installation pass.
Generated R help and the CLI API corpus are committed in sync.

This PR is stacked on #1495 so its package check inherits the generated-fixture
dependency scanner fix and the two package-note fixes, rather than duplicating
those changes in its review diff. Rebasing produced the expected conflict in
patches/templates.patch: select a starting copy, regenerate with
just templates-approve, and verify with just templates-check. No generated
patch hunks were hand-merged.

The built minirextendr tarball passes R CMD check --as-cran with 0 errors,
0 warnings, and 0 notes, with recursive dependency scanning and declared-package
Rd checking enabled. Final formatting, template checks, and all nine API-corpus
renderer tests pass without drift.
