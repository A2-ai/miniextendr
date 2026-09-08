# #1252: borrowed list elements bypassed the alias guard

Attempted: reject shared R vector storage when a mutable slice is bound through
a list argument and the same vector is also bound through another argument.

Failure: code-generation regressions fail because Vec parameters have no alias
guard, and the existing direct-slice guard disappears in release builds.

Root cause: classification recognizes only direct slice references and Option;
it compares top-level parameter SEXPs, so a list and its element look distinct.

Fix: classify borrowed slice leaves with their native element type and Vec
nesting depth. Check raw leaf identities before conversion, including duplicate
leaves inside a mutable list. Skip NULL, empty vectors, wrong-type leaves, and
shared/shared pairs; collect all conflicting parameter pairs in one diagnostic.
Keep current list elements protected while visiting another list so allocating
ALTREP access cannot collect a live leaf. Protection depth follows type nesting.
