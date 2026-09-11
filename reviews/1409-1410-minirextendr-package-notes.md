# minirextendr source-package check notes (#1409, #1410)

Checking the built package reproduced the non-standard top-level `AGENTS.md`
note. `.Rbuildignore` excluded its sibling `CLAUDE.md` but omitted the required
agent guidance file. Added an anchored exclusion while retaining both files in
the repository.

The release-workflow help linked to `miniextendr::assert_utf8_locale_now`, a
fixture in the separate framework package, not a topic in `minirextendr`.
The proposed same-package link would also be unresolved. An installed framework
masked this during local R CMD check; enabling R's declared-package xref check
reported the undeclared dependency. Replaced the reference with the published
release-workflow guide, whose page includes the UTF-8 rationale.

The regression builds the actual source tarball, checks its contents, extracts
it, and runs R's Rd resolver with declared-package checks enabled. Before the
fix it failed both the maintainer-file and help-link assertions. The first test
attempt also used an unsupported `info` argument to `expect_length`; switching
to `expect_identical` exposed the intended Rd failure directly.

The initial built-package check failed inside the new test because R's Rd
resolver scans all recommended-package help, while R CMD check shadows
undeclared test dependencies with dummy packages. `MASS` was installed in
`.Library` but hidden by that test library. The artifact test now temporarily
puts `.Library` first for the metadata scan; declared-package link checking
remains enabled. This does not add a runtime dependency on recommended packages.

The follow-up template audit found the same omission in both scaffold
`Rbuildignore` templates: neither excluded `AGENTS.md` or `CLAUDE.md`.
`templates-check` had passed because `justfile` did not map either
`Rbuildignore` template to `rpkg/.Rbuildignore`. Added both mappings, ported the
existing rpkg exclusions, and regenerated the approved delta. The agent-skill
installer now also excludes the `AGENTS.md` it writes when used directly in an
existing R package.

The extended regressions failed seven assertions before the fix: actual
standalone and monorepo template tarballs contained both agent files, and the
skill installer left its AGENTS.md unignored. After the fix, the focused ignore,
skill, and source-package suites pass 91 assertions with no failures, warnings,
or skips. The tarball tests retain README.md, so the exclusions cannot pass by
blanket-filtering Markdown files. Re-running the installer retains user-owned
AGENTS.md content and deduplicates the exclusion.

After rebasing onto current main, the complete minirextendr suite passes 805
assertions (11 existing skips, no failures or warnings). The built-package
check with declared-package Rd checking reports zero errors, zero warnings,
and zero notes. All three CI Clippy configurations and template drift checks
also pass.
