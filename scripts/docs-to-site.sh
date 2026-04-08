#!/usr/bin/env bash
# Convert docs/*.md files into Zola site content.
# Each doc gets frontmatter derived from its # Title line.
# Preserves _index.md (the landing page) untouched.
set -euo pipefail

DOCS_DIR="${1:-docs}"
CONTENT_DIR="${2:-site/content/manual}"

escape_toml_string() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

extract_description() {
  local doc="$1"
  local raw

  raw=$(
    tail -n +2 "$doc" | awk '
      BEGIN { capture = 0 }
      /^[[:space:]]*$/ {
        if (capture) exit
        next
      }
      /^#/ {
        if (capture) exit
        next
      }
      /^```/ {
        if (capture) exit
        next
      }
      /^[-*] / {
        if (capture) exit
        next
      }
      /^[0-9]+\. / {
        if (capture) exit
        next
      }
      {
        capture = 1
        print
      }
    '
  )

  if [ -z "$raw" ]; then
    return 0
  fi

  printf '%s' "$raw" \
    | tr '\n' ' ' \
    | sed -E \
      -e 's/[[:space:]]+/ /g' \
      -e 's/^ //; s/ $//' \
      -e 's/\[([^]]+)\]\([^)]+\)/\1/g' \
      -e 's/`//g' \
      -e 's/\*\*([^*]+)\*\*/\1/g' \
      -e 's/\*([^*]+)\*/\1/g' \
      -e 's/_([^_]+)_/\1/g'
}

# Weight mapping for sidebar ordering — important pages first.
# Anything not listed gets weight 50 (alphabetical within that tier).
declare -A WEIGHTS=(
  [GETTING_STARTED]=1
  [ARCHITECTURE]=2
  [TYPE_CONVERSIONS]=3
  [CLASS_SYSTEMS]=4
  [ALTREP]=5
  [EXTERNALPTR]=6
  [ERROR_HANDLING]=7
  [THREADS]=8
  [VENDOR]=9
  [FEATURES]=10
  [MINIEXTENDR_ATTRIBUTE]=11
  [ENTRYPOINT]=12
  [ADAPTER_TRAITS]=13
  [ADAPTER_COOKBOOK]=14
  [ARROW]=15
  [RAYON]=16
  [SAFETY]=17
  [GC_PROTECT]=18
  [FFI_GUARD]=19
  [COERCE]=20
  [AS_COERCE]=21
  [CONVERSION_MATRIX]=22
  [DATAFRAME]=23
  [CONNECTIONS]=24
  [EXPRESSION_EVAL]=25
  [ENUMS_AND_FACTORS]=26
  [DOTS_TYPED_LIST]=27
  [S3_METHODS]=28
  [ALTREP_EXAMPLES]=29
  [ALTREP_QUICKREF]=30
  [ALTREP_GUARDS]=31
  [ALTREP_SEXP]=32
  [SPARSE_ITERATOR_ALTREP]=33
  [RARRAY]=34
  [RAW_CONVERSIONS]=35
  [SERDE_R]=36
  [VCTRS]=37
  [RNG]=38
  [ENCODING]=39
  [LIFECYCLE]=40
  [STRICT_MODE]=41
  [FEATURE_DEFAULTS]=42
  [NONAPI]=43
  [EXTENDING_MINIEXTENDR]=44
  [TRAIT_ABI]=45
  [TRAIT_AS_R]=46
  [LINKING]=47
  [R_BUILD_SYSTEM]=48
  [TEMPLATES]=49
  [ENGINE]=50
  [ALLOCATOR]=51
  [CACHED_SEXPS]=52
  [PREFER_DERIVES]=53
  [TRACK_CALLER]=54
  [MACRO_ERRORS]=55
  [PROGRESS]=56
  [ENVIRONMENT_VARIABLES]=57
  [DEVELOPER_WORKFLOW]=58
  [MAINTAINER]=59
  [BENCHMARKS]=60
  [TROUBLESHOOTING]=61
  [SMOKE_TEST]=62
  [MINIREXTENDR]=63
  [ORPHAN_RULE_CHALLENGES]=64
  [GAPS]=65
  [FEATURE_BACKLOG]=66
)

# Remove old generated content (but keep _index.md)
find "$CONTENT_DIR" -maxdepth 1 -name '*.md' ! -name '_index.md' -delete

for doc in "$DOCS_DIR"/*.md; do
  basename=$(basename "$doc" .md)

  # Skip README — it's the repo readme, not a doc page
  [ "$basename" = "README" ] && continue

  # Extract title from first # heading
  title=$(head -1 "$doc" | sed 's/^# *//')
  description=$(extract_description "$doc")

  # Convert UPPER_CASE filename to kebab-case slug
  slug=$(echo "$basename" | tr '[:upper:]' '[:lower:]' | tr '_' '-')

  weight=${WEIGHTS[$basename]:-50}

  # Write frontmatter + content (skip the # Title line since Zola renders the title)
  {
    echo "+++"
    echo "title = \"$(escape_toml_string "$title")\""
    echo "weight = ${weight}"
    if [ -n "$description" ]; then
      echo "description = \"$(escape_toml_string "$description")\""
    fi
    echo "+++"
    echo ""
    # Skip the first line (# Title) and any blank line immediately after
    tail -n +2 "$doc" | sed '1{
/^$/d
}'
  } > "$CONTENT_DIR/${slug}.md"
done

echo "Generated $(find "$CONTENT_DIR" -maxdepth 1 -name '*.md' ! -name '_index.md' | wc -l | tr -d ' ') pages from docs/"
