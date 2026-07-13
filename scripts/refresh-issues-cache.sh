#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE_DIR="${1:-$ROOT/ISSUES}"

mkdir -p "$CACHE_DIR"
cd "$ROOT"

open_json="$(gh issue list \
  --state open \
  --limit 1000 \
  --json number,title,state,author,createdAt,updatedAt,labels,body,url)"
closed_json="$(gh issue list \
  --state closed \
  --limit 1000 \
  --json number,title,closedAt)"

jq '
  sort_by(.number)
  | reverse
  | map({createdAt, labels, number, title, updatedAt})
' <<<"$open_json" > "$CACHE_DIR/_open-index.json"

jq '
  sort_by(.number)
  | map({closedAt, number, title})
' <<<"$closed_json" > "$CACHE_DIR/_closed-index.json"

open_issue_numbers=" "
while IFS= read -r issue; do
  number="$(jq -r '.number' <<<"$issue")"
  open_issue_numbers+="$number "
  jq -r '
    [
      "# #" + (.number | tostring) + " — " + .title,
      "state: " + (.state | ascii_downcase)
        + " | author: " + (.author.login // "unknown")
        + " | created: " + .createdAt
        + " | updated: " + .updatedAt,
      "labels: " + ([.labels[].name] | join(", ")),
      "",
      (.body // "")
    ]
    | join("\n")
  ' <<<"$issue" > "$CACHE_DIR/issue-$number.md"
done < <(jq -c 'sort_by(.number) | reverse[]' <<<"$open_json")

stale_issue_files=()
shopt -s nullglob
for issue_file in "$CACHE_DIR"/issue-*.md; do
  issue_name="${issue_file##*/}"
  issue_number="${issue_name#issue-}"
  issue_number="${issue_number%.md}"
  if [[ "$open_issue_numbers" != *" $issue_number "* ]]; then
    stale_issue_files+=("$issue_file")
  fi
done

if (( ${#stale_issue_files[@]} > 0 )); then
  if ! command -v trash >/dev/null 2>&1; then
    echo "ERROR: stale issue bodies found, but no trash utility is available:" >&2
    printf '  %s\n' "${stale_issue_files[@]}" >&2
    exit 1
  fi
  trash "${stale_issue_files[@]}"
fi

open_count="$(jq 'length' <<<"$open_json")"
closed_count="$(jq 'length' <<<"$closed_json")"
echo "Updated $CACHE_DIR ($open_count open issue bodies, $closed_count closed issue summaries, ${#stale_issue_files[@]} stale bodies trashed)."
