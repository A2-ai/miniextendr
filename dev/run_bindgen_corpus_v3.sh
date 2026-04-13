#!/usr/bin/env bash
# Run bindgen on all CRAN packages with inst/include headers.
# v3: resolves LinkingTo transitive deps, adds -isysroot, tries c++14 fallback.
#
# Improvements over v2:
#   - R_NO_REMAP to avoid R macro pollution
#   - -isysroot for C++ stdlib headers (macOS)
#   - Resolves LinkingTo dependencies for transitive include paths
#   - Falls back to -std=c++14 if c++17 fails (for deprecated APIs like auto_ptr)
#   - Better error categorization
#
# Usage: bash dev/run_bindgen_corpus_v3.sh [output_dir]
set -uo pipefail

OUTPUT_DIR="${1:-dev/bindgen-corpus-v3-results}"
mkdir -p "$OUTPUT_DIR"

R_INCLUDE="$(Rscript -e 'cat(R.home("include"))')"
LIB="$(pwd)/rv/library/4.5/arm64"
CSV="dev/002_pkgs_with_inst_include.csv"

# macOS SDK path for C++ stdlib
SDK_PATH=""
if command -v xcrun >/dev/null 2>&1; then
  SDK_PATH="$(xcrun --show-sdk-path 2>/dev/null)"
fi

# Pre-compute LinkingTo transitive deps for all packages
# Output: file with "pkg:dep1,dep2,dep3" per line
LINKING_TO_FILE=$(mktemp /tmp/linking_to.XXXXXX)
Rscript -e '
ap <- available.packages()
pkgs <- read.csv("dev/002_pkgs_with_inst_include.csv")$Package
for (p in pkgs) {
  if (p %in% rownames(ap)) {
    lt <- ap[p, "LinkingTo"]
    if (!is.na(lt) && nzchar(lt)) {
      d <- trimws(strsplit(lt, ",")[[1]])
      d <- sub(" [(].*", "", d)
      cat(p, ":", paste(d, collapse = ","), "\n", sep = "")
    }
  }
}
' > "$LINKING_TO_FILE" 2>/dev/null

RESULTS="$OUTPUT_DIR/results.csv"
echo "package,n_headers,mode,std,bindgen_status,n_lines,has_static_fns,error_category,error_detail" > "$RESULTS"

PKG_LIST=$(mktemp /tmp/bindgen_pkglist.XXXXXX)
awk -F',' 'NR>1 {gsub(/\r/,""); print $1}' "$CSV" | \
  grep -iv -E '^(Rcpp$|Rcpp[A-Z]|cpp11$|cpp11[a-z]|cpp4r|bindrcpp|cppcontainers|tidyCpp|rcpptimer|rcppmlpackexamples)' \
  > "$PKG_LIST"

TOTAL=$(wc -l < "$PKG_LIST" | tr -d ' ')
SUCCESS=0
FAIL=0
SKIP=0

echo "Processing $TOTAL packages..."
echo "SDK: $SDK_PATH"
echo ""

while IFS= read -r pkg; do
  pkg="${pkg%%[[:space:]]}"
  [ -z "$pkg" ] && continue
  PKG_INCLUDE="$LIB/$pkg/include"

  if [ ! -d "$PKG_INCLUDE" ]; then
    echo "$pkg,0,none,,not_installed,0,no,not_installed," >> "$RESULTS"
    SKIP=$((SKIP + 1))
    continue
  fi

  n_c=$(find "$PKG_INCLUDE" -type f -name "*.h" 2>/dev/null | wc -l | tr -d ' ')
  n_cpp=$(find "$PKG_INCLUDE" -type f \( -name "*.hpp" -o -name "*.hh" -o -name "*.hxx" \) 2>/dev/null | wc -l | tr -d ' ')
  n_all=$((n_c + n_cpp))

  if [ "$n_all" -eq 0 ]; then
    echo "$pkg,$n_all,none,,no_headers,0,no,no_headers," >> "$RESULTS"
    SKIP=$((SKIP + 1))
    continue
  fi

  # Decide mode
  if [ "$n_cpp" -gt 0 ]; then
    MODE="cpp"
  else
    # Check if any .h files include C++ headers (heuristic: look for #include <string> etc.)
    if grep -rlq '#include <\(string\|vector\|map\|iostream\|memory\|algorithm\|functional\)>' "$PKG_INCLUDE" 2>/dev/null; then
      MODE="cpp"
    else
      MODE="c"
    fi
  fi

  # Build include path list: PKG_INCLUDE + transitive LinkingTo deps
  INCLUDE_ARGS="-I$R_INCLUDE -I$PKG_INCLUDE"
  DEPS=$(grep "^${pkg}:" "$LINKING_TO_FILE" | sed "s/^${pkg}://")
  if [ -n "$DEPS" ]; then
    IFS=',' read -ra DEP_ARRAY <<< "$DEPS"
    for dep in "${DEP_ARRAY[@]}"; do
      dep=$(echo "$dep" | tr -d ' ')
      DEP_INCLUDE="$LIB/$dep/include"
      if [ -d "$DEP_INCLUDE" ]; then
        INCLUDE_ARGS="$INCLUDE_ARGS -I$DEP_INCLUDE"
      fi
    done
  fi

  # Build wrapper header
  WRAPPER="/tmp/bindgen_wrapper_${pkg}.h"
  printf '#define R_NO_REMAP\n#include <Rinternals.h>\n\n' > "$WRAPPER"
  find "$PKG_INCLUDE" -maxdepth 3 -type f \( -name "*.h" -o -name "*.hpp" \) 2>/dev/null | head -20 | while read -r hdr; do
    rel="${hdr#$PKG_INCLUDE/}"
    echo "#include <$rel>" >> "$WRAPPER"
  done

  STATIC_C="/tmp/bindgen_static_${pkg}.c"

  # Try bindgen
  run_bindgen() {
    local std="$1"
    local lang_flags=""
    local ns_flag=""
    local sysroot_flag=""

    if [ "$MODE" = "cpp" ]; then
      lang_flags="-x c++ -std=$std"
      ns_flag="--enable-cxx-namespaces"
      if [ -n "$SDK_PATH" ]; then
        sysroot_flag="-isysroot $SDK_PATH"
      fi
    fi

    # shellcheck disable=SC2086
    bindgen \
      $ns_flag \
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
      $lang_flags $sysroot_flag $INCLUDE_ARGS \
      2>/tmp/bindgen_err_${pkg}.txt
  }

  BINDGEN_OUT="$OUTPUT_DIR/${pkg}_ffi.rs"
  USED_STD=""
  BINDGEN_OK=false

  # Try c++17 first (for C++ mode), then fallback to c++14
  if [ "$MODE" = "cpp" ]; then
    if run_bindgen "c++17" > "$BINDGEN_OUT"; then
      BINDGEN_OK=true
      USED_STD="c++17"
    elif run_bindgen "c++14" > "$BINDGEN_OUT"; then
      BINDGEN_OK=true
      USED_STD="c++14"
    fi
  else
    if run_bindgen "" > "$BINDGEN_OUT"; then
      BINDGEN_OK=true
      USED_STD="c"
    fi
  fi

  if $BINDGEN_OK; then
    n_lines=$(wc -l < "$BINDGEN_OUT" | tr -d ' ')
    has_static="no"
    [ -s "$STATIC_C" ] && has_static="yes"
    echo "$pkg,$n_all,$MODE,$USED_STD,ok,$n_lines,$has_static,," >> "$RESULTS"
    SUCCESS=$((SUCCESS + 1))
    printf "  OK: %-30s (%s/%s, %d lines, static=%s)\n" "$pkg" "$MODE" "$USED_STD" "$n_lines" "$has_static"
  else
    # Categorize the error
    ERR_RAW=$(cat /tmp/bindgen_err_${pkg}.txt 2>/dev/null)
    ERR_CAT="unknown"
    ERR_DETAIL=""

    if echo "$ERR_RAW" | grep -q "panicked at"; then
      ERR_CAT="bindgen_panic"
      ERR_DETAIL="bindgen crashed on complex C++ templates"
    elif echo "$ERR_RAW" | grep -q "file not found"; then
      MISSING=$(echo "$ERR_RAW" | grep -o "'[^']*' file not found" | head -1)
      if echo "$MISSING" | grep -qiE "Rcpp|RcppArmadillo|RcppEigen"; then
        ERR_CAT="rcpp_dep"
        ERR_DETAIL="$MISSING"
      elif echo "$MISSING" | grep -qE "'(string|vector|map|iostream|memory|algorithm|thread|mutex|atomic|functional|cstddef|cstdint|cstdlib|cstring|cmath|cassert|cfloat|climits|typeinfo|stdexcept|sstream|fstream|iomanip)'" ; then
        ERR_CAT="cxx_stdlib"
        ERR_DETAIL="$MISSING"
      else
        ERR_CAT="missing_header"
        ERR_DETAIL="$MISSING"
      fi
    elif echo "$ERR_RAW" | grep -q "error:"; then
      ERR_LINE=$(echo "$ERR_RAW" | grep "error:" | grep -v "^clang diag:" | head -1 | sed 's|/Users/[^ ]*/||g' | cut -c1-150)
      ERR_CAT="compile_error"
      ERR_DETAIL=$(echo "$ERR_LINE" | tr ',' ';')
    fi

    echo "$pkg,$n_all,$MODE,,$ERR_CAT,0,no,$ERR_CAT,$ERR_DETAIL" >> "$RESULTS"
    FAIL=$((FAIL + 1))
    printf "  FAIL: %-30s (%s) [%s] %s\n" "$pkg" "$MODE" "$ERR_CAT" "${ERR_DETAIL:0:80}"
    rm -f "$BINDGEN_OUT"
  fi

  rm -f "$WRAPPER" "$STATIC_C" "/tmp/bindgen_err_${pkg}.txt"
done < "$PKG_LIST"

rm -f "$PKG_LIST" "$LINKING_TO_FILE"

echo ""
echo "=== Summary ==="
echo "Total: $TOTAL"
echo "Success: $SUCCESS"
echo "Failed: $FAIL"
echo "Skipped: $SKIP"
echo ""
echo "=== Error categories ==="
awk -F',' 'NR>1 && $5!="ok" && $5!="not_installed" && $5!="no_headers" {print $8}' "$RESULTS" | sort | uniq -c | sort -rn
echo ""
echo "Results: $RESULTS"
