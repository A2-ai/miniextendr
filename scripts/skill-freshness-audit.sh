#!/usr/bin/env bash
# Quarterly freshness audit for Claude Code skills (.claude/skills/<slug>/SKILL.md).
#
# Flight 14 (#174) shipped a set of skills that cite file paths, function/type/
# macro names, and line numbers drawn from the source tree, CLAUDE.md, and
# MEMORY.md. Those citations drift as the code evolves. This script flags the
# drift so a maintainer can repair the skill (source wins — fix the SKILL.md,
# not the code). See issue #626.
#
# WHAT IT CHECKS, per .claude/skills/*/SKILL.md
# ---------------------------------------------
#   1. PATH existence  (BLOCKING)
#      Every backtick-wrapped token that contains a "/" and ends in a known
#      code/file extension is treated as a repo-relative path. We resolve it
#      against the repo root AND against rpkg/ (skills frequently use
#      package-relative paths like `src/Makevars.in` or `tools/foo.R`). A token
#      that resolves under neither root, and is not a known generated/gitignored
#      artifact or external reference, is a BLOCKING miss for that skill.
#
#   2. SYMBOL existence  (WARN)
#      Backtick-wrapped tokens shaped like real code identifiers
#      (snake_case fns, CamelCase types, ALL_CAPS consts, `macro!`) are searched
#      across the whole tracked repo with `git grep -wF`. A token found NOWHERE
#      is reported as a WARN (maintainer triage) — never BLOCKING, because prose
#      and illustrative names share this shape.
#
#   3. LINE-NUMBER cites  (WARN)
#      Tokens of the form `file.ext:NNN` are checked: if the resolved file has
#      fewer than NNN lines, the cite is out of range and reported as a WARN.
#
#   4. CLAUDE.md contradiction  (informational)
#      Not auto-decided here — too prose-shaped to diff mechanically without
#      false positives. The audit prints a reminder; a maintainer eyeballs any
#      skill that restates a CLAUDE.md fact (build-pipeline order, attribute
#      defaults, MXL numbers). Source wins.
#
# EXIT CODE
# ---------
#   Non-zero iff any BLOCKING miss (a cited path resolves nowhere). WARN-only
#   runs exit 0. This lets the script gate a CI job on path drift without
#   tripping on the inherently-noisy symbol heuristic.
#
# KNOWN FALSE-POSITIVE MODES (be conservative; prefer WARN over BLOCKING)
# ----------------------------------------------------------------------
#   * Illustrative names. Skills use placeholder paths/idents in examples
#     (`foo.rs`, `foo/mod.rs`, `my_rule.rs`, `MyStruct`, `s4_something`,
#     `fn_unchecked` where `fn` stands in for a real function name). These are
#     denylisted (ILLUSTRATIVE_TOKENS) or simply surface as WARN.
#   * R functions that look like paths (`match.arg`, `match.call`, `dyn.load`,
#     `self.value`). These contain a "." but no "/", so the path check ignores
#     them (only slash-containing tokens are paths).
#   * Prose abbreviations of real symbols (`write_wrappers` for
#     `miniextendr_write_wrappers`). Word-bounded grep won't find the short form
#     -> shows as a symbol WARN. Harmless; maintainer ignores.
#   * External R headers (`R_ext/Connections.h`) live in the gitignored
#     `background/` reference tree; matched by EXTERNAL_PATH_PREFIXES -> WARN.
#   * Generated / gitignored artifacts (`inst/vendor.tar.xz`,
#     `R/miniextendr-wrappers.R`, `.cargo/config.toml`) won't exist in a clean
#     tree; matched by GENERATED_PATH_SUFFIXES -> WARN, not BLOCKING.
#
# USAGE
#   bash scripts/skill-freshness-audit.sh            # human report
#   bash scripts/skill-freshness-audit.sh --quiet    # only misses + summary
#   bash scripts/skill-freshness-audit.sh --user-layout <scaffold-root>
#       Audits the END-USER skill set (minirextendr/inst/claude/skills/) with
#       paths resolved against <scaffold-root> — a freshly scaffolded package
#       tree (minirextendr::create_miniextendr_package()). There, paths like
#       `src/rust/lib.rs` are real files, not this repo's layout. Symbol
#       existence is still checked against THIS repo (the framework source).
#
# CADENCE: run quarterly (see CLAUDE.md "Skill freshness audit"). Repair drift
# in the same pass; source wins.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

QUIET=0
USER_LAYOUT=0
SCAFFOLD_ROOT=""
while [ $# -gt 0 ]; do
    case "$1" in
        --quiet) QUIET=1 ;;
        --user-layout)
            USER_LAYOUT=1
            SCAFFOLD_ROOT="${2:-}"
            if [ -z "$SCAFFOLD_ROOT" ] || [ ! -d "$SCAFFOLD_ROOT" ]; then
                echo "usage: $0 --user-layout <scaffold-root>  (scaffold-root must be an existing directory)" >&2
                exit 2
            fi
            shift
            ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

SKILLS_GLOB=".claude/skills/*/SKILL.md"

# File extensions we treat as "this token is a path", to avoid mistaking R
# function calls (match.arg) or version strings for paths.
PATH_EXT_RE='\.(rs|R|toml|lock|c|h|md|def|patch|yml|yaml|sh|in|R|cfg)$'

# Tokens that are illustrative placeholders, never real repo objects.
ILLUSTRATIVE_TOKENS=(
    'foo.rs' 'foo/mod.rs' 'my_rule.rs' 'bar.rs'
    'MyStruct' 'MyType' 'MyTrait'
    'fn_unchecked' 's4_something' 's4_s4_something'
    's4_my_method' 's4_s4_my_method'
)

# Generated / gitignored artifacts: absence in a clean tree is expected.
# Matched as a *suffix* of the cited path.
GENERATED_PATH_SUFFIXES=(
    'inst/vendor.tar.xz'
    'R/miniextendr-wrappers.R'
    '.cargo/config.toml'
    'src/rust/wasm_registry.rs'
    'Cargo.lock'
)

# External reference trees (gitignored background/) — cited but not in-repo.
EXTERNAL_PATH_PREFIXES=(
    'R_ext/'
)

# Scaffolded-end-user-package layout. The getting-started / scaffolding skills
# describe the structure of a package minirextendr GENERATES, not a path inside
# this repo (this repo's live example lives under rpkg/src/rust/, with no nested
# src/). Matched as prefix -> WARN, not BLOCKING.
SCAFFOLD_PATH_PREFIXES=(
    'src/rust/src/'
)

# Roots a package-relative path may resolve against (skills mix repo-root and
# rpkg-relative paths freely).
RESOLVE_ROOTS=( '.' 'rpkg' )

# Files a scaffolded package only gains after an opt-in helper or a build has
# run (use_configure_feature_detection(), ./configure). Cited by the user
# skills as "where X lives when enabled" — absence in a bare scaffold is
# expected. User-layout mode only.
OPTIONAL_SCAFFOLD_PATHS=(
    'tools/detect-features.R'
    'src/Makevars'
)

if [ "$USER_LAYOUT" -eq 1 ]; then
    SKILLS_GLOB="minirextendr/inst/claude/skills/*/SKILL.md"
    RESOLVE_ROOTS=( "$SCAFFOLD_ROOT" )
    # No background/-style external trees in a scaffold; scaffold-layout paths
    # ARE the real layout there, so that prefix class doesn't apply either.
    EXTERNAL_PATH_PREFIXES=()
    SCAFFOLD_PATH_PREFIXES=()
fi

# Pathspecs for the symbol search: the whole tracked tree minus vendored
# crates, the rv R library, and the skills themselves (we don't want a symbol
# to "exist" only because the skill names it).
SYMBOL_PATHSPECS=(
    ':!**/vendor/**'
    ':!rv/**'
    ':!.claude/skills/**'
    ':!minirextendr/inst/claude/skills/**'
    ':!target/**'
)

c_red=''; c_yellow=''; c_green=''; c_dim=''; c_reset=''
if [ -t 1 ]; then
    c_red=$'\033[31m'; c_yellow=$'\033[33m'; c_green=$'\033[32m'
    c_dim=$'\033[2m'; c_reset=$'\033[0m'
fi

in_list() {
    local needle="$1"; shift
    local x
    for x in "$@"; do [ "$x" = "$needle" ] && return 0; done
    return 1
}

has_suffix() {
    local s="$1" suf
    shift
    for suf in "$@"; do
        case "$s" in *"$suf") return 0 ;; esac
    done
    return 1
}

has_prefix() {
    local s="$1" pre
    shift
    for pre in "$@"; do
        case "$s" in "$pre"*) return 0 ;; esac
    done
    return 1
}

# Resolve a repo path against each candidate root; echo "ok" if it exists.
path_exists() {
    local p="$1" root
    for root in "${RESOLVE_ROOTS[@]}"; do
        [ -e "$root/$p" ] && return 0
    done
    return 1
}

total_block=0
total_warn=0
audited=0

for skill in $SKILLS_GLOB; do
    [ -f "$skill" ] || continue
    audited=$((audited + 1))
    slug="$(basename "$(dirname "$skill")")"

    block_lines=()
    warn_lines=()

    # ---- collect backtick tokens once -------------------------------------
    # Strip leading list-prefix; pull every `...` span. The single quotes are
    # deliberate — we match literal backticks, nothing should expand.
    # shellcheck disable=SC2016
    mapfile -t tokens < <(grep -oE '`[^`]+`' "$skill" | sed 's/^`//; s/`$//' | sort -u)

    for tok in "${tokens[@]}"; do
        # --- line-number cite: file.ext:NNN ---
        if [[ "$tok" =~ ^([A-Za-z0-9_./-]+):([0-9]+)$ ]]; then
            file="${BASH_REMATCH[1]}"
            lineno="${BASH_REMATCH[2]}"
            # Only meaningful if it looks like a path with an extension.
            if [[ "$file" =~ $PATH_EXT_RE ]]; then
                resolved=""
                for root in "${RESOLVE_ROOTS[@]}"; do
                    if [ -f "$root/$file" ]; then resolved="$root/$file"; break; fi
                    # bare filename: try to locate a unique tracked match
                done
                if [ -z "$resolved" ] && [[ "$file" != */* ]]; then
                    # bare filename line cite (e.g. miniextendr_trait.rs:808)
                    match="$(git ls-files "**/$file" 2>/dev/null | head -1)"
                    [ -n "$match" ] && resolved="$match"
                fi
                if [ -n "$resolved" ]; then
                    nlines="$(wc -l < "$resolved" | tr -d ' ')"
                    if [ "$lineno" -gt "$nlines" ]; then
                        warn_lines+=("line-cite OUT OF RANGE: $tok (file has $nlines lines: $resolved)")
                    fi
                else
                    warn_lines+=("line-cite file not found: $tok")
                fi
            fi
            continue
        fi

        # --- path token (must contain a slash + known extension) ---
        if [[ "$tok" == */* ]] && [[ "$tok" =~ $PATH_EXT_RE ]]; then
            # Skip non-paths that happen to embed a slashed filename:
            #   - command fragments (contain whitespace): `Rscript tools/foo.R`
            #   - glob patterns (contain *): `R/*-wrappers.R`, `tools/*.R`
            #   - template placeholders (contain < >): `.../<system>_class.rs`
            case "$tok" in
                *[[:space:]]* | *'*'* | *'<'* | *'>'*) continue ;;
            esac
            if in_list "$tok" "${ILLUSTRATIVE_TOKENS[@]}"; then
                continue
            fi
            if [ "${#EXTERNAL_PATH_PREFIXES[@]}" -gt 0 ] && has_prefix "$tok" "${EXTERNAL_PATH_PREFIXES[@]}"; then
                warn_lines+=("external ref (not in repo): $tok")
                continue
            fi
            if [ "${#SCAFFOLD_PATH_PREFIXES[@]}" -gt 0 ] && has_prefix "$tok" "${SCAFFOLD_PATH_PREFIXES[@]}"; then
                warn_lines+=("scaffolded-package layout (not a repo path): $tok")
                continue
            fi
            if path_exists "$tok"; then
                continue
            fi
            if [ "$USER_LAYOUT" -eq 1 ] && in_list "$tok" "${OPTIONAL_SCAFFOLD_PATHS[@]}"; then
                warn_lines+=("optional scaffold file (created by an opt-in helper or ./configure): $tok")
                continue
            fi
            if has_suffix "$tok" "${GENERATED_PATH_SUFFIXES[@]}"; then
                warn_lines+=("generated/gitignored path (absent in clean tree): $tok")
                continue
            fi
            block_lines+=("MISSING PATH: $tok")
            continue
        fi

        # --- symbol token (code-shaped identifier, no slash, no dot-path) ---
        if [[ "$tok" != */* ]] && [[ "$tok" != *.* ]]; then
            base="${tok%!}"
            shaped=0
            # snake_case fn (>=1 underscore, lowercase head)
            [[ "$base" =~ ^[a-z][a-z0-9]*(_[a-z0-9]+)+$ ]] && shaped=1
            # CamelCase type (multi-word)
            [[ "$base" =~ ^[A-Z][a-z0-9]+[A-Z][A-Za-z0-9]*$ ]] && shaped=1
            # ALL_CAPS const (>=2 segments)
            [[ "$base" =~ ^[A-Z][A-Z0-9]*(_[A-Z0-9]+)+$ ]] && shaped=1
            if [ "$shaped" -eq 1 ]; then
                if in_list "$tok" "${ILLUSTRATIVE_TOKENS[@]}" || in_list "$base" "${ILLUSTRATIVE_TOKENS[@]}"; then
                    continue
                fi
                if ! git grep -qwF "$base" -- "${SYMBOL_PATHSPECS[@]}" 2>/dev/null; then
                    warn_lines+=("symbol not found in repo: $tok")
                fi
            fi
            continue
        fi
    done

    # ---- per-skill report -------------------------------------------------
    nblock=${#block_lines[@]}
    nwarn=${#warn_lines[@]}
    total_block=$((total_block + nblock))
    total_warn=$((total_warn + nwarn))

    if [ "$nblock" -eq 0 ] && [ "$nwarn" -eq 0 ]; then
        [ "$QUIET" -eq 1 ] || printf '%s== %-32s%s %sOK%s\n' "$c_dim" "$slug" "$c_reset" "$c_green" "$c_reset"
        continue
    fi

    printf '== %-32s ' "$slug"
    if [ "$nblock" -gt 0 ]; then
        printf '%s%d BLOCKING%s' "$c_red" "$nblock" "$c_reset"
    fi
    if [ "$nwarn" -gt 0 ]; then
        [ "$nblock" -gt 0 ] && printf ', '
        printf '%s%d WARN%s' "$c_yellow" "$nwarn" "$c_reset"
    fi
    printf '\n'

    for l in "${block_lines[@]}"; do printf '   %s[BLOCK]%s %s\n' "$c_red" "$c_reset" "$l"; done
    for l in "${warn_lines[@]}"; do printf '   %s[warn]%s  %s\n' "$c_yellow" "$c_reset" "$l"; done
done

printf '\n'
printf -- '----------------------------------------\n'
printf 'Audited %d skill(s): %s%d BLOCKING%s, %s%d WARN%s\n' \
    "$audited" \
    "$c_red" "$total_block" "$c_reset" \
    "$c_yellow" "$total_warn" "$c_reset"
printf '%sContradiction check (CLAUDE.md vs skill restated facts) is manual — source wins.%s\n' "$c_dim" "$c_reset"

if [ "$total_block" -gt 0 ]; then
    printf '%sBLOCKING path misses found — repair the cited SKILL.md (source wins).%s\n' "$c_red" "$c_reset"
    exit 1
fi
exit 0
