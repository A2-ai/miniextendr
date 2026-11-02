# Summarise a bench.R CSV holding two labels as a Markdown table.
#
# Usage, from the repository root:
#   Rscript rpkg/dev/bench-native-borrow/compare.R <results.csv> <before> <after>
# Per case and label: the median over reps of the per-rep trimmed means, the
# min and max over reps, and the coefficient of variation over reps (the
# noise). `after / before` compares the medians.

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 3L) {
  stop("usage: Rscript rpkg/dev/bench-native-borrow/compare.R <results.csv> <before> <after>",
       call. = FALSE)
}
res <- read.csv(args[[1]], stringsAsFactors = FALSE)
before <- args[[2]]
after <- args[[3]]

summarise <- function(case, label) {
  t <- res$tmean_ns[res$case == case & res$label == label]
  c(med = stats::median(t), min = min(t), max = max(t), cv = stats::sd(t) / mean(t), n = length(t))
}
fmt <- function(s) {
  sprintf("%.0f (%.0f to %.0f, CV %.1f%%)", s[["med"]], s[["min"]], s[["max"]], 100 * s[["cv"]])
}

cases <- unique(res$case)
cat(sprintf("| case | %s ns | %s ns | %s / %s |\n", before, after, after, before))
cat("|---|---:|---:|---:|\n")
for (case in cases) {
  b <- summarise(case, before)
  a <- summarise(case, after)
  cat(sprintf("| `%s` | %s | %s | %.3f |\n", case, fmt(b), fmt(a), a[["med"]] / b[["med"]]))
}
cat(sprintf("\nReps per label: %d. Each value is a per-rep 5%%-trimmed mean of bench::mark iterations.\n",
            as.integer(summarise(cases[[1]], before)[["n"]])))
