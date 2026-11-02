# Compare two builds of miniextendr on the native-borrow bench (#1504).
#
# Usage, from the repository root (so rv activates the project library):
#   Rscript rpkg/dev/bench-native-borrow/run.R <out_dir> <before_lib> <after_lib> [reps]
# <before_lib> and <after_lib> are libraries that each hold one installed
# build of miniextendr (install one, copy rv/library/<ver>/<arch>/miniextendr
# into a directory, install the other). The two builds alternate, one rep at
# a time and each in its own Rscript, so drift in machine state hits both
# alike. reps defaults to 7; NBB_MIN_TIME is passed through. <out_dir> gets
# results.csv, env.log and summary.md (compare.R's table).

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 3L) {
  stop("usage: Rscript rpkg/dev/bench-native-borrow/run.R <out_dir> <before_lib> <after_lib> [reps]",
       call. = FALSE)
}
out_dir <- args[[1]]
libs <- c(before = args[[2]], after = args[[3]])
reps <- if (length(args) >= 4L) as.integer(args[[4]]) else 7L
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)
csv <- file.path(out_dir, "results.csv")
if (file.exists(csv)) file.remove(csv)

file_arg <- grep("^--file=", commandArgs(FALSE), value = TRUE)
here <- dirname(normalizePath(sub("^--file=", "", file_arg[[1]])))
rscript <- file.path(R.home("bin"), "Rscript")

writeLines(c(
  paste("date:", format(Sys.time(), "%Y-%m-%d %H:%M:%S %Z")),
  paste("R:", R.version.string),
  paste("bench:", as.character(utils::packageVersion("bench"))),
  paste("machine:", paste(Sys.info()[c("sysname", "release", "machine")], collapse = " ")),
  paste("load:", paste(suppressWarnings(system("uptime", intern = TRUE)), collapse = " ")),
  paste("before_lib:", libs[["before"]]),
  paste("after_lib:", libs[["after"]])
), file.path(out_dir, "env.log"))

for (rep in seq_len(reps)) {
  for (label in names(libs)) {
    cat(format(Sys.time(), "%H:%M:%S"), "rep", rep, label, "\n")
    status <- system2(rscript, c(file.path(here, "bench.R"), csv, label, rep),
                      env = paste0("NBB_LIB=", libs[[label]]))
    if (status != 0L) stop("bench.R failed for ", label, " rep ", rep, call. = FALSE)
  }
}

summary <- system2(rscript, c(file.path(here, "compare.R"), csv, "before", "after"), stdout = TRUE)
writeLines(summary, file.path(out_dir, "summary.md"))
writeLines(summary)
