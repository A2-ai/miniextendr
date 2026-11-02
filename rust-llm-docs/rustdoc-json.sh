# shellcheck shell=bash
# Shared by the rustdoc JSON generators in this directory; sourced, not run.
#
#   DOC_TOOLCHAIN=nightly
#   . rust-llm-docs/rustdoc-json.sh
#
# rustdoc JSON is unstable, so every build goes through `cargo +$DOC_TOOLCHAIN`
# (an installed nightly). `-D warnings` keeps a broken intra-doc link from
# landing silently.

export RUSTDOCFLAGS="-D warnings -Z unstable-options --output-format json"
DOC_FLAGS=(--no-deps --document-private-items)

# Cargo target directories are configurable globally, per workspace, and via
# `CARGO_TARGET_DIR`. Resolve them from the same working directory as each
# `cargo doc` invocation so the renderer reads the JSON Cargo actually wrote.
cargo_target_dir() {
  cargo "+$DOC_TOOLCHAIN" metadata --format-version 1 --no-deps \
    | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])'
}

# miniextendr-api with every optional integration (`--features full`), the
# build both the corpus and the type matrix read.
doc_api_full() {
  echo ">> cargo doc (api, full features)"
  cargo "+$DOC_TOOLCHAIN" doc "${DOC_FLAGS[@]}" -p miniextendr-api --features full
}
