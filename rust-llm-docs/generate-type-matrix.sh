#!/usr/bin/env bash
# Build the type matrix: every type miniextendr defines or converts, and how
# each one behaves across the framework.
#
#   rust-llm-docs/generate-type-matrix.sh [nightly-toolchain]
#
# Outputs to rust-llm-docs/generated/ (gitignored; see rust-llm-docs/README.md):
#   type-matrix.json       the data (rustdoc_type_matrix.py)
#   type-matrix.html       self-contained interactive page (rustdoc_type_matrix_html.py)
#   type-matrix.pdf        both tables, one page each (type-matrix.typ)
#   type-matrix-<p>.svg    the same pages as SVG
#
# Requires an installed nightly toolchain (nightly by default), python3 (no
# third-party deps) and typst.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
GEN="$HERE/generated"
mkdir -p "$GEN"

DOC_TOOLCHAIN="${1:-nightly}"
# shellcheck source=rust-llm-docs/rustdoc-json.sh
. "$HERE/rustdoc-json.sh"

if ! command -v typst >/dev/null 2>&1; then
  echo "error: typst is not installed (https://github.com/typst/typst#installation)" >&2
  exit 1
fi

cd "$ROOT"
TARGET="$(cargo_target_dir)"

doc_api_full
echo ">> cargo doc (engine)"
cargo "+$DOC_TOOLCHAIN" doc "${DOC_FLAGS[@]}" -p miniextendr-engine

echo ">> building the matrix"
python3 "$HERE/rustdoc_type_matrix.py" \
  --api "$TARGET/doc/miniextendr_api.json" \
  --engine "$TARGET/doc/miniextendr_engine.json" \
  --repo "$ROOT" \
  --out "$GEN/type-matrix.json" \
  --html "$GEN/type-matrix.html"

echo ">> typst"
rm -f "$GEN"/type-matrix-*.svg
typst compile "$HERE/type-matrix.typ" "$GEN/type-matrix.pdf"
typst compile "$HERE/type-matrix.typ" "$GEN/type-matrix-{p}.svg"
echo ">> done -> $GEN/type-matrix.{json,html,pdf} + type-matrix-*.svg"
