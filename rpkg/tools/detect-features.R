# Feature detection for miniextendr (rpkg demo package)
# Called by ./configure to auto-detect available features.
# Output: comma-separated list of Cargo features to enable.
#
# Users can override by setting the MINIEXTENDR_FEATURES environment variable.
# Add rules with: minirextendr::add_feature_rule()

features <- character()

## BEGIN RULES (do not edit this line)

# Core features — always enable
features <- c(features, "worker-thread")
features <- c(features, "rayon")
features <- c(features, "rand")
features <- c(features, "rand_distr")
features <- c(features, "either")
features <- c(features, "ndarray")
features <- c(features, "nalgebra")
features <- c(features, "serde")
features <- c(features, "serde_json")
features <- c(features, "num-bigint")
features <- c(features, "rust_decimal")
features <- c(features, "ordered-float")
features <- c(features, "uuid")
features <- c(features, "regex")
features <- c(features, "indexmap")
features <- c(features, "time")
features <- c(features, "num-traits")
features <- c(features, "bytes")
features <- c(features, "num-complex")
features <- c(features, "url")
features <- c(features, "sha2")
features <- c(features, "bitflags")
features <- c(features, "bitvec")
features <- c(features, "aho-corasick")
features <- c(features, "toml")
features <- c(features, "tabled")
features <- c(features, "tinyvec")
features <- c(features, "raw_conversions")
features <- c(features, "borsh")
features <- c(features, "arrow")
features <- c(features, "log")

# vctrs — only if the R package is installed
if (requireNamespace("vctrs", quietly = TRUE)) {
  features <- c(features, "vctrs")
}

## END RULES (do not edit this line)

cat(paste(features, collapse = ","))
