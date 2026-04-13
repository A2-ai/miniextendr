#!/usr/bin/env bash
# Run bindgen on all CRAN packages with inst/include headers, using C++ mode.
# Tests both C and C++ headers with improved flags:
#   - R_NO_REMAP to avoid R macro pollution (length, error, etc.)
#   - -x c++ -std=c++17 for C++ parsing
#   - --enable-cxx-namespaces for C++ namespace support
#   - --wrap-static-fns for static inline functions
#
# Usage: bash dev/run_bindgen_corpus_cpp.sh [output_dir]
set -uo pipefail

OUTPUT_DIR="${1:-dev/bindgen-corpus-cpp-results}"
mkdir -p "$OUTPUT_DIR"

R_INCLUDE="$(Rscript -e 'cat(R.home("include"))')"
LIB="$(pwd)/rv/library/4.5/arm64"
CSV="dev/002_pkgs_with_inst_include.csv"

RESULTS="$OUTPUT_DIR/results.csv"
echo "package,n_all_headers,n_c_headers,n_cpp_headers,mode,bindgen_status,n_lines,has_static_fns,error" > "$RESULTS"

PKG_LIST=$(mktemp /tmp/bindgen_pkglist.XXXXXX)
awk -F',' 'NR>1 {gsub(/\r/,""); print $1}' "$CSV" | \
  grep -iv -E '^(Rcpp$|Rcpp[A-Z]|cpp11$|cpp11[a-z]|cpp4r|bindrcpp|cppcontainers|tidyCpp|rcpptimer|rcppmlpackexamples)' \
  > "$PKG_LIST"

TOTAL=$(wc -l < "$PKG_LIST" | tr -d ' ')
SUCCESS=0
FAIL=0
SKIP=0

echo "Processing $TOTAL packages (C + C++ mode)..."

while IFS= read -r pkg; do
  pkg="${pkg%%[[:space:]]}"
  [ -z "$pkg" ] && continue
  PKG_INCLUDE="$LIB/$pkg/include"

  if [ ! -d "$PKG_INCLUDE" ]; then
    echo "$pkg,0,0,0,none,not_installed,0,no," >> "$RESULTS"
    SKIP=$((SKIP + 1))
    continue
  fi

  n_c=$(find "$PKG_INCLUDE" -type f -name "*.h" 2>/dev/null | wc -l | tr -d ' ')
  n_cpp=$(find "$PKG_INCLUDE" -type f \( -name "*.hpp" -o -name "*.hh" -o -name "*.hxx" \) 2>/dev/null | wc -l | tr -d ' ')
  n_all=$((n_c + n_cpp))

  if [ "$n_all" -eq 0 ]; then
    echo "$pkg,$n_all,$n_c,$n_cpp,none,no_headers,0,no," >> "$RESULTS"
    SKIP=$((SKIP + 1))
    continue
  fi

  # Decide mode: if any .hpp/.hh, use C++; otherwise C
  if [ "$n_cpp" -gt 0 ]; then
    MODE="cpp"
    LANG_FLAGS="-x c++ -std=c++17"
    NS_FLAG="--enable-cxx-namespaces"
  else
    MODE="c"
    LANG_FLAGS=""
    NS_FLAG=""
  fi

  # Build wrapper header: R_NO_REMAP + first ≤20 headers
  WRAPPER="/tmp/bindgen_wrapper_${pkg}.h"
  printf '#define R_NO_REMAP\n#include <Rinternals.h>\n\n' > "$WRAPPER"

  find "$PKG_INCLUDE" -maxdepth 3 -type f \( -name "*.h" -o -name "*.hpp" \) 2>/dev/null | head -20 | while read -r hdr; do
    rel="${hdr#$PKG_INCLUDE/}"
    echo "#include <$rel>" >> "$WRAPPER"
  done

  STATIC_C="/tmp/bindgen_static_${pkg}.c"
  BINDGEN_OUT="$OUTPUT_DIR/${pkg}_ffi.rs"

  # shellcheck disable=SC2086
  if bindgen \
    $NS_FLAG \
    --merge-extern-blocks \
    --no-layout-tests \
    --no-doc-comments \
    --wrap-static-fns \
    --wrap-static-fns-path "$STATIC_C" \
    --blocklist-type 'SEXPREC' \
    --blocklist-type 'SEXP' \
    --raw-line 'use miniextendr_api::ffi::SEXP;' \
    "$WRAPPER" \
    -- \
    $LANG_FLAGS \
    -I"$R_INCLUDE" \
    -I"$PKG_INCLUDE" \
    > "$BINDGEN_OUT" 2>/tmp/bindgen_err_${pkg}.txt; then

    n_lines=$(wc -l < "$BINDGEN_OUT" | tr -d ' ')
    has_static="no"
    [ -s "$STATIC_C" ] && has_static="yes"
    echo "$pkg,$n_all,$n_c,$n_cpp,$MODE,ok,$n_lines,$has_static," >> "$RESULTS"
    SUCCESS=$((SUCCESS + 1))
    printf "  OK: %-30s (%s, %d lines, static=%s)\n" "$pkg" "$MODE" "$n_lines" "$has_static"
  else
    err=$(grep -v "^clang diag:" /tmp/bindgen_err_${pkg}.txt 2>/dev/null | grep -v "^$" | head -1 | tr ',' ';' | head -c 200)
    echo "$pkg,$n_all,$n_c,$n_cpp,$MODE,error,0,no,$err" >> "$RESULTS"
    FAIL=$((FAIL + 1))
    printf "  FAIL: %-30s (%s) %s\n" "$pkg" "$MODE" "${err:-(unknown)}"
    rm -f "$BINDGEN_OUT"
  fi

  rm -f "$WRAPPER" "$STATIC_C" "/tmp/bindgen_err_${pkg}.txt"
done < "$PKG_LIST"

rm -f "$PKG_LIST"

echo ""
echo "=== Summary ==="
echo "Total: $TOTAL"
echo "Success: $SUCCESS"
echo "Failed: $FAIL"
echo "Skipped: $SKIP"
echo "Results: $RESULTS"
