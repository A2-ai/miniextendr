#!/usr/bin/env bash
# Emit a JSON array summarising each file under plans/ so the Zola
# roadmap template can render it at build time. For every plan:
#   - filename (basename, no extension)
#   - title (first-line `# …` heading, stripped)
#   - summary (first non-heading paragraph, plain-text)
#   - url (GitHub blob URL pointing at the plan on main)
#
# Invoke from the repo root: `bash scripts/plans-to-json.sh > site/data/plans.json`
set -euo pipefail

PLANS_DIR="${1:-plans}"
REPO_URL="${MINIEXTENDR_REPO_URL:-https://github.com/A2-ai/miniextendr}"
BRANCH="${MINIEXTENDR_DOCS_BRANCH:-main}"

json_escape() {
  python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()), end="")'
}

first() { printf '1'; }

printf '['
first=1
for plan in "$PLANS_DIR"/*.md; do
  [ -f "$plan" ] || continue
  basename=$(basename "$plan" .md)

  title=$(head -1 "$plan" | sed -E 's/^# +//')

  summary=$(
    awk '
      NR == 1 { next }
      /^[[:space:]]*$/ { if (capture) exit; next }
      /^#/            { if (capture) exit; next }
      /^```/          { if (capture) exit; next }
      /^[-*] /        { if (capture) exit; next }
      /^[0-9]+\. /    { if (capture) exit; next }
      { capture = 1; print }
    ' "$plan" | tr '\n' ' ' \
      | sed -E \
        -e 's/[[:space:]]+/ /g' \
        -e 's/^ //; s/ $//' \
        -e 's/\[([^]]+)\]\([^)]+\)/\1/g' \
        -e 's/`//g' \
        -e 's/\*\*([^*]+)\*\*/\1/g' \
        -e 's/\*([^*]+)\*/\1/g' \
        -e 's/_([^_]+)_/\1/g'
  )

  url="${REPO_URL}/blob/${BRANCH}/${PLANS_DIR}/${basename}.md"

  if [ "$first" -eq 1 ]; then
    first=0
  else
    printf ','
  fi

  printf '{'
  printf '"slug":%s,'    "$(printf '%s' "$basename" | json_escape)"
  printf '"title":%s,'   "$(printf '%s' "$title"    | json_escape)"
  printf '"summary":%s,' "$(printf '%s' "$summary"  | json_escape)"
  printf '"url":%s'      "$(printf '%s' "$url"      | json_escape)"
  printf '}'
done
printf ']'
