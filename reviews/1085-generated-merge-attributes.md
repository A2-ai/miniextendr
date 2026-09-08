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

Validation so far: 855 full minirextendr assertions passed, zero failures or
warnings, 11 existing skips. just fmt, just check, template synchronization,
and generated R help passed. Further verification is recorded before publishing.
