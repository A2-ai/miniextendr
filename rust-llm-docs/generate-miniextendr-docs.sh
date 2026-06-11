#!/usr/bin/env bash
# Regenerate the LLM-parseable doc corpus for every miniextendr workspace crate.
#
#   rust-llm-docs/generate-miniextendr-docs.sh
#
# Outputs to rust-llm-docs/generated/:
#   <crate>.md                     — single-file API digest (rustdoc_megadoc.py)
#   <crate>-impl-inventory.md      — every trait impl grouped by trait + span
#   conversion-impl-inventory.md   — conversion traits only, the dedup-audit view
#
# Requires a nightly-capable rustc (we use RUSTC_BOOTSTRAP=1 on stable) and
# python3 (no third-party deps for the local-crate path).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
GEN="$HERE/generated"
mkdir -p "$GEN"

# miniextendr-api gets the broad feature set so every feature-gated conversion
# impl is visible (full minus the heavy datafusion/tokio stack, plus jiff).
API_FEATURES="serde,serde_json,num-complex,uuid,url,aho-corasick,bitflags,bitvec,arrow,toml,time,ndarray,nalgebra,indexmap,tinyvec,bytes,raw_conversions,vctrs,borsh,ordered-float,num-bigint,rust_decimal,regex,num-traits,tabled,rayon,sha2,rand,rand_distr,either,log,worker-thread,macro-coverage,growth-debug,jiff"

cd "$ROOT"
echo ">> cargo doc (api, broad features)"
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" \
  cargo doc --no-deps -p miniextendr-api --features "$API_FEATURES"

echo ">> cargo doc (macros, engine, lint, cli)"
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" \
  cargo doc --no-deps -p miniextendr-macros -p miniextendr-engine -p miniextendr-lint -p miniextendr-cli

# json basename -> output basename  (cli's lib crate is named `miniextendr`)
gen() {
  local json="$1" out="$2"
  python3 "$HERE/rustdoc_megadoc.py"        "target/doc/${json}.json" "$GEN/${out}.md" >/dev/null
  python3 "$HERE/rustdoc_impl_inventory.py" "target/doc/${json}.json" --out "$GEN/${out}-impl-inventory.md" >/dev/null
  echo "   ${out}.md + ${out}-impl-inventory.md"
}

echo ">> rendering markdown"
gen miniextendr_api    miniextendr-api
gen miniextendr_macros miniextendr-macros
gen miniextendr_engine miniextendr-engine
gen miniextendr_lint   miniextendr-lint
gen miniextendr        miniextendr-cli

# Conversion-only inventory — the dedup-audit lens.
python3 "$HERE/rustdoc_impl_inventory.py" target/doc/miniextendr_api.json \
  --traits TryFromSexp,IntoR,Coerce,TryCoerce,IntoRAs,RSerializeNative,RDeserializeNative,IntoRAltrep,AltrepSerialize \
  --out "$GEN/conversion-impl-inventory.md" >/dev/null
echo "   conversion-impl-inventory.md"

# Manual-vs-macro lens — hand-rolled impls a macro could absorb.
python3 "$HERE/rustdoc_manual_vs_macro.py" target/doc/miniextendr_api.json \
  --out "$GEN/conversion-manual-vs-macro.md" >/dev/null
echo "   conversion-manual-vs-macro.md"
echo ">> done -> $GEN"
