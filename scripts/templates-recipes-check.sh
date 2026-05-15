#!/usr/bin/env bash
# Verify that shared recipe bodies in the two template justfiles stay aligned.
#
# Context: minirextendr/inst/templates/ has two template flavours:
#   rpkg/justfile     - standalone R package
#   monorepo/justfile - Rust workspace with an embedded R package
#
# Several recipes are intentionally identical modulo a {{rpkg}}/ path prefix
# that the monorepo template prepends (because the R package lives in a
# subdirectory). This script checks that those recipes have not drifted.
#
# Normalisation applied to the monorepo body before diffing:
#   - Strip leading "{{rpkg}}/" from path arguments
#   - Replace standalone "{{rpkg}}" (without trailing slash) with "."
#
# If a new shared recipe is added to both templates, append its name to
# SHARED_RECIPES below.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RPKG_JF="$REPO_ROOT/minirextendr/inst/templates/rpkg/justfile"
MONO_JF="$REPO_ROOT/minirextendr/inst/templates/monorepo/justfile"

# Recipes that must be structurally identical (modulo {{rpkg}}/ prefix).
# Recipes with intentional differences (e.g. vendor, rbuild, rtest, load,
# devtools-install) are deliberately excluded.
SHARED_RECIPES=(
    cran-prep
    rcheck
    rdoc
    devtools-check
)

# extract_recipe <file> <name>
# Prints the recipe header + all indented/continuation lines until the next
# top-level definition.  Comment lines (# ...) at column 0 are excluded:
# they vary intentionally between templates (signpost comments, doc strings
# of sibling recipes) and must not influence the comparison.
# Works with both BSD awk (macOS) and GNU awk.
extract_recipe() {
    local file="$1" name="$2"
    awk -v recipe="$name" '
        # Stash just attribute annotations ([script(...)], [private], etc.)
        /^[[:space:]]*\[/ { attr = $0; next }
        # Skip top-level comment lines (not inside a recipe)
        /^#/ { if (!in_recipe) { attr = ""; next } }
        $0 ~ ("^" recipe "([[:space:]\\*:]|$)") {
            if (attr != "") { print attr; attr = "" }
            in_recipe = 1
            print
            next
        }
        # Clear stashed attribute on any non-attribute, non-matching top-level line
        /^[a-zA-Z]/ { attr = "" }
        in_recipe {
            # Stop at the next top-level definition (letter or [ at col 0)
            if (/^[a-zA-Z\[]/ ) { in_recipe = 0; exit }
            # Skip comment lines inside the recipe body too (signpost comments
            # differ between templates by design)
            if (/^#/) { next }
            print
        }
    ' "$file" | sed -e 's/[[:space:]]*$//'
}

# normalise_mono <body>
# Strip "{{rpkg}}/" prefix from paths; replace bare "{{rpkg}}" with ".".
normalise_mono() {
    sed \
        -e 's|{{rpkg}}/||g' \
        -e 's|{{rpkg}}|.|g'
}

fail=0

for recipe in "${SHARED_RECIPES[@]}"; do
    rpkg_body="$(extract_recipe "$RPKG_JF" "$recipe")"
    mono_body="$(extract_recipe "$MONO_JF" "$recipe" | normalise_mono)"

    if [ -z "$rpkg_body" ]; then
        printf 'ERROR: recipe "%s" not found in %s\n' "$recipe" "$RPKG_JF" >&2
        fail=1
        continue
    fi
    if [ -z "$mono_body" ]; then
        printf 'ERROR: recipe "%s" not found in %s\n' "$recipe" "$MONO_JF" >&2
        fail=1
        continue
    fi

    diff_out="$(diff <(printf '%s\n' "$rpkg_body") <(printf '%s\n' "$mono_body") || true)"
    if [ -n "$diff_out" ]; then
        printf 'DRIFT in recipe "%s":\n' "$recipe" >&2
        printf '%s\n' "$diff_out" >&2
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then
    printf '\nShared recipe drift detected. Edit both template justfiles to keep them in sync.\n' >&2
    printf 'See scripts/templates-recipes-check.sh for the canonical list.\n' >&2
    exit 1
fi

printf 'OK: all shared template justfile recipes are in sync.\n'
