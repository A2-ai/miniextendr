#!/usr/bin/env bash
# Run bindgen on all R packages with inst/include headers.
# Reports which packages can have their headers parsed by bindgen.
#
# Usage: bash dev/run_bindgen_corpus.sh [output_dir]
set -uo pipefail

OUTPUT_DIR="${1:-dev/bindgen-corpus-results}"
mkdir -p "$OUTPUT_DIR"

R_INCLUDE="$(Rscript -e 'cat(R.home("include"))')"
STAGING="$(pwd)/rv/library/4.5/arm64"
CSV="dev/002_pkgs_with_inst_include.csv"

# Results file
RESULTS="$OUTPUT_DIR/results.csv"
echo "package,n_headers,n_c_headers,bindgen_status,n_bindings,has_static_fns,error" > "$RESULTS"

# Build package list to a temp file (avoids subshell issues)
PKG_LIST=$(mktemp /tmp/bindgen_pkglist.XXXXXX)
awk -F',' 'NR>1 {gsub(/\r/,""); print $1}' "$CSV" | \
  grep -iv -E '^(Rcpp$|Rcpp[A-Z]|cpp11$|cpp11[a-z]|cpp4r|bindrcpp|cppcontainers|tidyCpp|rcpptimer|rcppmlpackexamples)' \
  > "$PKG_LIST"

TOTAL=$(wc -l < "$PKG_LIST" | tr -d ' ')
SUCCESS=0
FAIL=0
SKIP=0

echo "Processing $TOTAL packages..."

while IFS= read -r pkg; do
  pkg="${pkg%%[[:space:]]}"  # trim
  [ -z "$pkg" ] && continue
  PKG_INCLUDE="$STAGING/$pkg/include"

  if [ ! -d "$PKG_INCLUDE" ]; then
    echo "$pkg,0,0,not_installed,0,no," >> "$RESULTS"
    SKIP=$((SKIP + 1))
    continue
  fi

  # Count all headers and C-only headers
  n_headers=$(find "$PKG_INCLUDE" -type f \( -name "*.h" -o -name "*.hpp" -o -name "*.hh" \) 2>/dev/null | wc -l | tr -d ' ')
  n_c_headers=$(find "$PKG_INCLUDE" -type f -name "*.h" 2>/dev/null | wc -l | tr -d ' ')

  if [ "$n_c_headers" -eq 0 ]; then
    echo "$pkg,$n_headers,$n_c_headers,no_c_headers,0,no," >> "$RESULTS"
    SKIP=$((SKIP + 1))
    continue
  fi

  # Find .h files and create a wrapper header
  WRAPPER="/tmp/bindgen_wrapper_${pkg}.h"
  echo "#include <Rinternals.h>" > "$WRAPPER"

  # Include up to the first 20 .h files (avoid massive headers like BH)
  find "$PKG_INCLUDE" -type f -name "*.h" -maxdepth 3 2>/dev/null | head -20 | while read -r hdr; do
    rel="${hdr#$PKG_INCLUDE/}"
    echo "#include <$rel>" >> "$WRAPPER"
  done

  # Run bindgen
  STATIC_WRAPPERS="/tmp/bindgen_static_${pkg}.c"
  BINDGEN_OUT="$OUTPUT_DIR/${pkg}_ffi.rs"

  if bindgen \
    --merge-extern-blocks \
    --no-layout-tests \
    --no-doc-comments \
    --wrap-static-fns \
    --wrap-static-fns-path "$STATIC_WRAPPERS" \
    --blocklist-type 'SEXPREC' \
    --blocklist-type 'SEXP' \
    --raw-line 'use miniextendr_api::ffi::SEXP;' \
    "$WRAPPER" \
    -- \
    -I"$R_INCLUDE" \
    -I"$PKG_INCLUDE" \
    > "$BINDGEN_OUT" 2>/tmp/bindgen_err_${pkg}.txt; then

    n_bindings=$(grep -c 'pub fn\|pub static\|pub type' "$BINDGEN_OUT" 2>/dev/null || echo 0)
    has_static="no"
    [ -s "$STATIC_WRAPPERS" ] && has_static="yes"
    echo "$pkg,$n_headers,$n_c_headers,ok,$n_bindings,$has_static," >> "$RESULTS"
    SUCCESS=$((SUCCESS + 1))
    printf "  OK: %-30s (%d bindings, static=%s)\n" "$pkg" "$n_bindings" "$has_static"
  else
    err=$(head -1 /tmp/bindgen_err_${pkg}.txt 2>/dev/null | tr ',' ';' | head -c 200)
    echo "$pkg,$n_headers,$n_c_headers,error,0,no,$err" >> "$RESULTS"
    FAIL=$((FAIL + 1))
    printf "  FAIL: %-30s (%s)\n" "$pkg" "${err:-(unknown)}"
    rm -f "$BINDGEN_OUT"
  fi

  # Cleanup temp files
  rm -f "$WRAPPER" "$STATIC_WRAPPERS" "/tmp/bindgen_err_${pkg}.txt"
done < "$PKG_LIST"

rm -f "$PKG_LIST"

echo ""
echo "=== Summary ==="
echo "Total: $TOTAL"
echo "Success: $SUCCESS"
echo "Failed: $FAIL"
echo "Skipped: $SKIP"
echo "Results: $RESULTS"
