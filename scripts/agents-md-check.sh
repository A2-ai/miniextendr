#!/usr/bin/env bash
# CI check for the CLAUDE.md <-> AGENTS.md sibling invariant (issue #1253).
#
# PR #1236 put an AGENTS.md beside every CLAUDE.md so non-Claude agents
# (codex/cursor) inherit the same per-directory guidance. The convention is
# documented in the root CLAUDE.md "Orientation" section; this script keeps
# it from silently regressing.
#
# WHAT IT CHECKS
# --------------
# All file inventory comes from `git ls-files` ONLY — never the working
# tree — so untracked/ignored trees (background/, rv/, target/) can never
# trip it, and scaffolded-package files under minirextendr/inst/templates/
# stay out of scope by construction (none are tracked CLAUDE.md/AGENTS.md).
#
#   1. SIBLING PRESENCE: every tracked CLAUDE.md has a tracked AGENTS.md in
#      the same directory.
#   2. NO ORPHANS: every tracked AGENTS.md has a tracked CLAUDE.md in the
#      same directory.
#   3. IMPORT LINE: every tracked *subdirectory* AGENTS.md contains a line
#      that is exactly `@CLAUDE.md` (codex's include mechanism — the file
#      inherits its sibling CLAUDE.md verbatim, no drift). The ROOT
#      AGENTS.md is exempt: it is a hand-kept mirror, not an import.
#
# NOT CHECKED: prose parity of the root AGENTS.md mirror against the root
# CLAUDE.md. That is inherently hand-maintained (see the root CLAUDE.md
# "Orientation" section — a project-wide rule changed in one must be
# mirrored in the other by hand).
#
# EXIT CODE
# ---------
# Non-zero iff any rule fired; every violation is printed with a one-line
# fix hint. A clean tree exits 0 silently (CI-friendly).
#
# USAGE
#   bash scripts/agents-md-check.sh       # or: just agents-md-check
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

violations=0

# Directory prefixes ("" for the repo root, "dir/" otherwise) holding each
# file. The `grep -E '(^|/)CLAUDE\.md$'` guard keeps a hypothetical
# FOO-CLAUDE.md from matching; `|| true` keeps an empty match set from
# tripping pipefail.
claude_dirs="$(git ls-files '*CLAUDE.md' | { grep -E '(^|/)CLAUDE\.md$' || true; } | sed 's|CLAUDE\.md$||' | sort)"
agents_dirs="$(git ls-files '*AGENTS.md' | { grep -E '(^|/)AGENTS\.md$' || true; } | sed 's|AGENTS\.md$||' | sort)"

# Rule 1: dirs with a CLAUDE.md but no AGENTS.md.
while IFS= read -r dir; do
    echo "VIOLATION: ${dir}CLAUDE.md has no sibling ${dir}AGENTS.md" \
        "— add one (subdirectory: a one-line @CLAUDE.md import; see the root CLAUDE.md 'Orientation' section)."
    violations=1
done < <(comm -23 <(printf '%s\n' "$claude_dirs") <(printf '%s\n' "$agents_dirs") | sed '/^$/d')

# Rule 2: orphan AGENTS.md (no sibling CLAUDE.md).
while IFS= read -r dir; do
    echo "VIOLATION: ${dir}AGENTS.md has no sibling ${dir}CLAUDE.md" \
        "— every AGENTS.md pairs with a CLAUDE.md; add the CLAUDE.md or remove the orphan (see the root CLAUDE.md 'Orientation' section)."
    violations=1
done < <(comm -13 <(printf '%s\n' "$claude_dirs") <(printf '%s\n' "$agents_dirs") | sed '/^$/d')

# Rule 3: every subdirectory AGENTS.md carries the literal import line.
# The root AGENTS.md (hand-kept mirror) is exempt.
while IFS= read -r f; do
    [ "$f" = "AGENTS.md" ] && continue
    if ! grep -qx '@CLAUDE.md' "$f"; then
        echo "VIOLATION: $f lacks the exact import line '@CLAUDE.md'" \
            "— subdirectory AGENTS.md must import its sibling CLAUDE.md verbatim (see the root CLAUDE.md 'Orientation' section)."
        violations=1
    fi
done < <(git ls-files '*AGENTS.md' | { grep -E '(^|/)AGENTS\.md$' || true; })

exit "$violations"
