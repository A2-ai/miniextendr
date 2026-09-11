# PR 1485 rebase validation

Rebased the match-argument changes onto main after the S3 naming and roxygen
source-order changes landed. Preserved both hand-written generic documentation
blocks and regenerated the conflicting Rd, API inventory, and producer wrappers.

The installed-package S3 and help-page tests passed all 21 assertions. An extra
validation command then incorrectly asserted that `length(tools::undoc(...))`
should be zero. That function always returns four diagnostic categories, even
when every category is empty. Check `all(lengths(tools::undoc(...)) == 0L)`
instead; all four categories are empty after regeneration.
