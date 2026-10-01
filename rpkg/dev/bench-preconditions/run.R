# Run the whole precondition benchmark: the success-path timings, the chunked
# scans, their summaries, the error-path comparison and the probes.
# Results: miniextendr-bench/BENCH_RESULTS_2026-09-29-preconditions.md.
#
# Usage, from the repository root (so rv activates the project library):
#   just configure && just rcmdinstall
#   Rscript rpkg/dev/bench-preconditions/run.R <out_dir> [reps] [min_time]
# reps (default 3) and min_time (seconds, default 0.4) are the settings of the
# committed results. The full run takes about 20 minutes;
#   Rscript rpkg/dev/bench-preconditions/run.R target/bench-preconditions-smoke 1 0.05
# is a quick smoke run. A directory under `target/` stays out of git.
#
# Each script runs in its own Rscript process. <out_dir> gets:
#   env.log                       commit, R / bench / rustc versions, load
#   bench-results.csv, bench.log  main run (bench.R, default groups)
#   bench-chunked.csv, bench-chunked.log   chunked-scan run (bench.R chunked)
#   summary.md, summary-chunked.md         analyze.R over the two CSVs
#   errors.csv, errors-compact.txt, errors-inherits.csv, errors.log   errors.R
#   materialise.log, chunked_check.log, guard_forms.log, floor_parts.log

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 1L) {
  stop("usage: Rscript rpkg/dev/bench-preconditions/run.R <out_dir> [reps] [min_time]", call. = FALSE)
}
out_dir <- args[[1]]
Sys.setenv(
  PCB_REPS = if (length(args) >= 2L) args[[2]] else "3",
  PCB_MIN_TIME = if (length(args) >= 3L) args[[3]] else "0.4"
)
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)

file_arg <- grep("^--file=", commandArgs(FALSE), value = TRUE)
here <- dirname(normalizePath(sub("^--file=", "", file_arg[[1]])))
rscript <- file.path(R.home("bin"), "Rscript")
out <- function(name) file.path(out_dir, name)

load_avg <- function() paste(suppressWarnings(system("uptime", intern = TRUE)), collapse = " ")
writeLines(c(
  paste("date:", format(Sys.time(), "%Y-%m-%d %H:%M:%S %Z")),
  paste("commit:", paste(suppressWarnings(system("git rev-parse --short HEAD", intern = TRUE)), collapse = " ")),
  paste("system:", paste(Sys.info()[c("sysname", "release", "machine")], collapse = " ")),
  paste("R:", R.version.string),
  paste("bench:", as.character(utils::packageVersion("bench"))),
  paste("rustc:", paste(suppressWarnings(system("rustc --version", intern = TRUE)), collapse = " ")),
  paste("PCB_REPS:", Sys.getenv("PCB_REPS"), " PCB_MIN_TIME:", Sys.getenv("PCB_MIN_TIME")),
  paste("load before:", load_avg())
), out("env.log"))

# stderr joins the log, except for the markdown summaries, where it goes to the
# console so a warning cannot land in the table.
step <- function(script, script_args = character(), stdout_file, stderr_file = stdout_file) {
  cat(format(Sys.time(), "%H:%M:%S"), script, paste(script_args, collapse = " "), "\n")
  status <- system2(rscript, c(file.path(here, script), script_args),
                    stdout = stdout_file, stderr = stderr_file)
  if (!identical(status, 0L)) {
    stop(script, " failed (exit ", status, "); see ", stdout_file, call. = FALSE)
  }
}

step("bench.R", out("bench-results.csv"), out("bench.log"))
step("bench.R", c(out("bench-chunked.csv"), "chunked"), out("bench-chunked.log"))
step("analyze.R", out("bench-results.csv"), out("summary.md"), "")
step("analyze.R", out("bench-chunked.csv"), out("summary-chunked.md"), "")
step("errors.R", out("errors.csv"), out("errors.log"))
step("materialise.R", stdout_file = out("materialise.log"))
step("chunked_check.R", stdout_file = out("chunked_check.log"))
step("guard_forms.R", stdout_file = out("guard_forms.log"))
step("floor_parts.R", stdout_file = out("floor_parts.log"))

cat(paste("load after:", load_avg()), file = out("env.log"), sep = "\n", append = TRUE)
cat("done; results in", normalizePath(out_dir), "\n")
