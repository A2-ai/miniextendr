#!/usr/bin/env bash
# Run bindgen on CRAN packages with inst/include headers and report parse results.
#
# Three modes mirror the three generations of this script:
#
#   --mode c     (legacy v1) C-only headers, no R_NO_REMAP, no isysroot, no
#                LinkingTo resolution.  Output: bindgen-corpus-c-results/
#
#   --mode cpp   (legacy v2) C+C++ headers, R_NO_REMAP, -std=c++17,
#                --enable-cxx-namespaces.  No isysroot, no LinkingTo
#                resolution, no c++14 fallback.
#                Output: bindgen-corpus-cpp-results/
#
#   --mode full  (legacy v3, DEFAULT) Everything: R_NO_REMAP, isysroot,
#                LinkingTo transitive resolution, c++14 fallback, enriched
#                error categorisation.  Output: bindgen-corpus-full-results/
#
# Input corpus: dev/002_pkgs_with_inst_include.csv
# Staging tree: rv/library/4.5/arm64/<pkg>/include/
#
# Usage:
#   bash dev/run_bindgen_corpus.sh [--mode c|cpp|full] [output_dir]
#   bash dev/run_bindgen_corpus.sh --help
#
# Examples:
#   bash dev/run_bindgen_corpus.sh                            # full mode, default dir
#   bash dev/run_bindgen_corpus.sh --mode=cpp                 # cpp mode, default dir
#   bash dev/run_bindgen_corpus.sh --mode c /tmp/c-out        # c mode, custom dir
set -uo pipefail

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
MODE="full"
OUTPUT_DIR=""

print_usage() {
  sed -n '2,/^set -uo/p' "$0" | grep '^#' | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)   MODE="$2"; shift 2 ;;
    --mode=*) MODE="${1#*=}"; shift ;;
    -h|--help) print_usage; exit 0 ;;
    -*) echo "Unknown option: $1" >&2; print_usage >&2; exit 2 ;;
    *)  OUTPUT_DIR="$1"; shift ;;
  esac
done

case "$MODE" in
  c|cpp|full) ;;
  *) echo "Unknown --mode: $MODE (expected c, cpp, or full)" >&2; exit 2 ;;
esac

OUTPUT_DIR="${OUTPUT_DIR:-dev/bindgen-corpus-${MODE}-results}"
mkdir -p "$OUTPUT_DIR"

# ---------------------------------------------------------------------------
# Common setup
# ---------------------------------------------------------------------------
R_INCLUDE="$(Rscript -e 'cat(R.home("include"))')"
LIB="$(pwd)/rv/library/4.5/arm64"
CSV="dev/002_pkgs_with_inst_include.csv"

# ---------------------------------------------------------------------------
# Mode: full — pre-compute LinkingTo transitive deps and macOS SDK path
# ---------------------------------------------------------------------------
SDK_PATH=""
LINKING_TO_FILE=""

if [[ "$MODE" == "full" ]]; then
  if command -v xcrun >/dev/null 2>&1; then
    SDK_PATH="$(xcrun --show-sdk-path 2>/dev/null)"
  fi

  LINKING_TO_FILE=$(mktemp /tmp/linking_to.XXXXXX)
  # shellcheck disable=SC2016
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
fi

# ---------------------------------------------------------------------------
# CSV header — differs by mode
# ---------------------------------------------------------------------------
RESULTS="$OUTPUT_DIR/results.csv"
if [[ "$MODE" == "c" ]]; then
  echo "package,n_headers,n_c_headers,bindgen_status,n_bindings,has_static_fns,error" > "$RESULTS"
elif [[ "$MODE" == "cpp" ]]; then
  echo "package,n_all_headers,n_c_headers,n_cpp_headers,mode,bindgen_status,n_lines,has_static_fns,error" > "$RESULTS"
else
  echo "package,n_headers,mode,std,bindgen_status,n_lines,has_static_fns,error_category,error_detail" > "$RESULTS"
fi

# ---------------------------------------------------------------------------
# Build package list (filter known C++-only wrappers that always fail)
# ---------------------------------------------------------------------------
PKG_LIST=$(mktemp /tmp/bindgen_pkglist.XXXXXX)
awk -F',' 'NR>1 {gsub(/\r/,""); print $1}' "$CSV" | \
  grep -iv -E '^(Rcpp$|Rcpp[A-Z]|cpp11$|cpp11[a-z]|cpp4r|bindrcpp|cppcontainers|tidyCpp|rcpptimer|rcppmlpackexamples)' \
  > "$PKG_LIST"

TOTAL=$(wc -l < "$PKG_LIST" | tr -d ' ')
SUCCESS=0
FAIL=0
SKIP=0

if [[ "$MODE" == "c" ]]; then
  echo "Processing $TOTAL packages (C-only mode)..."
elif [[ "$MODE" == "cpp" ]]; then
  echo "Processing $TOTAL packages (C + C++ mode)..."
else
  echo "Processing $TOTAL packages (full mode)..."
  echo "SDK: $SDK_PATH"
  echo ""
fi

# ---------------------------------------------------------------------------
# Per-package loop
# ---------------------------------------------------------------------------
while IFS= read -r pkg; do
  pkg="${pkg%%[[:space:]]}"
  [[ -z "$pkg" ]] && continue
  PKG_INCLUDE="$LIB/$pkg/include"

  # --- not installed ---
  if [[ ! -d "$PKG_INCLUDE" ]]; then
    if [[ "$MODE" == "c" ]]; then
      echo "$pkg,0,0,not_installed,0,no," >> "$RESULTS"
    elif [[ "$MODE" == "cpp" ]]; then
      echo "$pkg,0,0,0,none,not_installed,0,no," >> "$RESULTS"
    else
      echo "$pkg,0,none,,not_installed,0,no,not_installed," >> "$RESULTS"
    fi
    SKIP=$((SKIP + 1))
    continue
  fi

  # --- count headers ---
  if [[ "$MODE" == "c" ]]; then
    n_headers=$(find "$PKG_INCLUDE" -type f \( -name "*.h" -o -name "*.hpp" -o -name "*.hh" \) 2>/dev/null | wc -l | tr -d ' ')
    n_c_headers=$(find "$PKG_INCLUDE" -type f -name "*.h" 2>/dev/null | wc -l | tr -d ' ')

    if [[ "$n_c_headers" -eq 0 ]]; then
      echo "$pkg,$n_headers,$n_c_headers,no_c_headers,0,no," >> "$RESULTS"
      SKIP=$((SKIP + 1))
      continue
    fi
  else
    n_c=$(find "$PKG_INCLUDE" -type f -name "*.h" 2>/dev/null | wc -l | tr -d ' ')
    n_cpp=$(find "$PKG_INCLUDE" -type f \( -name "*.hpp" -o -name "*.hh" -o -name "*.hxx" \) 2>/dev/null | wc -l | tr -d ' ')
    n_all=$((n_c + n_cpp))

    if [[ "$n_all" -eq 0 ]]; then
      if [[ "$MODE" == "cpp" ]]; then
        echo "$pkg,$n_all,$n_c,$n_cpp,none,no_headers,0,no," >> "$RESULTS"
      else
        echo "$pkg,$n_all,none,,no_headers,0,no,no_headers," >> "$RESULTS"
      fi
      SKIP=$((SKIP + 1))
      continue
    fi
  fi

  # --- decide language mode (cpp and full only) ---
  if [[ "$MODE" == "c" ]]; then
    PKG_LANG="c"
  elif [[ "$MODE" == "cpp" ]]; then
    if [[ "$n_cpp" -gt 0 ]]; then
      PKG_LANG="cpp"
    else
      PKG_LANG="c"
    fi
  else
    # full: check .hpp/.hh presence OR C++ stdlib includes in .h files
    if [[ "$n_cpp" -gt 0 ]]; then
      PKG_LANG="cpp"
    elif grep -rlq '#include <\(string\|vector\|map\|iostream\|memory\|algorithm\|functional\)>' "$PKG_INCLUDE" 2>/dev/null; then
      PKG_LANG="cpp"
    else
      PKG_LANG="c"
    fi
  fi

  # --- build wrapper header ---
  WRAPPER="/tmp/bindgen_wrapper_${pkg}.h"
  if [[ "$MODE" == "c" ]]; then
    echo "#include <Rinternals.h>" > "$WRAPPER"
    find "$PKG_INCLUDE" -type f -name "*.h" -maxdepth 3 2>/dev/null | head -20 | while read -r hdr; do
      rel="${hdr#"$PKG_INCLUDE"/}"
      echo "#include <$rel>" >> "$WRAPPER"
    done
  else
    printf '#define R_NO_REMAP\n#include <Rinternals.h>\n\n' > "$WRAPPER"
    find "$PKG_INCLUDE" -maxdepth 3 -type f \( -name "*.h" -o -name "*.hpp" \) 2>/dev/null | head -20 | while read -r hdr; do
      rel="${hdr#"$PKG_INCLUDE"/}"
      echo "#include <$rel>" >> "$WRAPPER"
    done
  fi

  STATIC_C="/tmp/bindgen_static_${pkg}.c"
  BINDGEN_OUT="$OUTPUT_DIR/${pkg}_ffi.rs"

  # --- build include args (full mode adds LinkingTo deps) ---
  INCLUDE_ARGS="-I$R_INCLUDE -I$PKG_INCLUDE"
  if [[ "$MODE" == "full" && -n "$LINKING_TO_FILE" ]]; then
    DEPS=$(grep "^${pkg}:" "$LINKING_TO_FILE" | sed "s/^${pkg}://")
    if [[ -n "$DEPS" ]]; then
      IFS=',' read -ra DEP_ARRAY <<< "$DEPS"
      for dep in "${DEP_ARRAY[@]}"; do
        dep=$(echo "$dep" | tr -d ' ')
        DEP_INCLUDE="$LIB/$dep/include"
        if [[ -d "$DEP_INCLUDE" ]]; then
          INCLUDE_ARGS="$INCLUDE_ARGS -I$DEP_INCLUDE"
        fi
      done
    fi
  fi

  # ---------------------------------------------------------------------------
  # Mode c: simple single bindgen run, count pub fn/static/type
  # ---------------------------------------------------------------------------
  if [[ "$MODE" == "c" ]]; then
    # shellcheck disable=SC2086
    if bindgen \
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
        -I"$R_INCLUDE" \
        -I"$PKG_INCLUDE" \
        > "$BINDGEN_OUT" 2>/tmp/bindgen_err_${pkg}.txt; then

      n_bindings=$(grep -c 'pub fn\|pub static\|pub type' "$BINDGEN_OUT" 2>/dev/null || echo 0)
      has_static="no"
      [[ -s "$STATIC_C" ]] && has_static="yes"
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

  # ---------------------------------------------------------------------------
  # Mode cpp: single run with optional C++ flags, count lines
  # ---------------------------------------------------------------------------
  elif [[ "$MODE" == "cpp" ]]; then
    if [[ "$PKG_LANG" == "cpp" ]]; then
      LANG_FLAGS="-x c++ -std=c++17"
      NS_FLAG="--enable-cxx-namespaces"
    else
      LANG_FLAGS=""
      NS_FLAG=""
    fi

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
      [[ -s "$STATIC_C" ]] && has_static="yes"
      echo "$pkg,$n_all,$n_c,$n_cpp,$PKG_LANG,ok,$n_lines,$has_static," >> "$RESULTS"
      SUCCESS=$((SUCCESS + 1))
      printf "  OK: %-30s (%s, %d lines, static=%s)\n" "$pkg" "$PKG_LANG" "$n_lines" "$has_static"
    else
      err=$(grep -v "^clang diag:" /tmp/bindgen_err_${pkg}.txt 2>/dev/null | grep -v "^$" | head -1 | tr ',' ';' | head -c 200)
      echo "$pkg,$n_all,$n_c,$n_cpp,$PKG_LANG,error,0,no,$err" >> "$RESULTS"
      FAIL=$((FAIL + 1))
      printf "  FAIL: %-30s (%s) %s\n" "$pkg" "$PKG_LANG" "${err:-(unknown)}"
      rm -f "$BINDGEN_OUT"
    fi

  # ---------------------------------------------------------------------------
  # Mode full: c++17 → c++14 fallback, isysroot, enriched error categories
  # ---------------------------------------------------------------------------
  else
    run_bindgen() {
      local std="$1"
      local lang_flags=""
      local ns_flag=""
      local sysroot_flag=""

      if [[ "$PKG_LANG" == "cpp" ]]; then
        lang_flags="-x c++ -std=$std"
        ns_flag="--enable-cxx-namespaces"
        if [[ -n "$SDK_PATH" ]]; then
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

    USED_STD=""
    BINDGEN_OK=false

    if [[ "$PKG_LANG" == "cpp" ]]; then
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
      [[ -s "$STATIC_C" ]] && has_static="yes"
      echo "$pkg,$n_all,$PKG_LANG,$USED_STD,ok,$n_lines,$has_static,," >> "$RESULTS"
      SUCCESS=$((SUCCESS + 1))
      printf "  OK: %-30s (%s/%s, %d lines, static=%s)\n" "$pkg" "$PKG_LANG" "$USED_STD" "$n_lines" "$has_static"
    else
      ERR_RAW=$(cat "/tmp/bindgen_err_${pkg}.txt" 2>/dev/null)
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
        elif echo "$MISSING" | grep -qE "'(string|vector|map|iostream|memory|algorithm|thread|mutex|atomic|functional|cstddef|cstdint|cstdlib|cstring|cmath|cassert|cfloat|climits|typeinfo|stdexcept|sstream|fstream|iomanip)'"; then
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

      echo "$pkg,$n_all,$PKG_LANG,,$ERR_CAT,0,no,$ERR_CAT,$ERR_DETAIL" >> "$RESULTS"
      FAIL=$((FAIL + 1))
      printf "  FAIL: %-30s (%s) [%s] %s\n" "$pkg" "$PKG_LANG" "$ERR_CAT" "${ERR_DETAIL:0:80}"
      rm -f "$BINDGEN_OUT"
    fi
  fi

  # --- cleanup temp files ---
  rm -f "$WRAPPER" "$STATIC_C" "/tmp/bindgen_err_${pkg}.txt"
done < "$PKG_LIST"

rm -f "$PKG_LIST"
[[ -n "${LINKING_TO_FILE:-}" ]] && rm -f "$LINKING_TO_FILE"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=== Summary ==="
echo "Total: $TOTAL"
echo "Success: $SUCCESS"
echo "Failed: $FAIL"
echo "Skipped: $SKIP"

if [[ "$MODE" == "full" ]]; then
  echo ""
  echo "=== Error categories ==="
  awk -F',' 'NR>1 && $5!="ok" && $5!="not_installed" && $5!="no_headers" {print $8}' "$RESULTS" | sort | uniq -c | sort -rn
fi

echo ""
echo "Results: $RESULTS"
