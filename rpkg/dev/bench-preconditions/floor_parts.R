# What is in the ~660 ns no-guard floor? Byte-compiled hand variants of the
# generated `pcb_i32_u` wrapper, dropping one piece at a time.
#
# Usage: Rscript rpkg/dev/bench-preconditions/floor_parts.R > floor_parts.log
# PCB_REPS (default 3) and PCB_MIN_TIME (default 0.4) as in bench.R.
suppressPackageStartupMessages(library(bench))
REPS <- as.integer(Sys.getenv("PCB_REPS", "3"))
MIN_TIME <- as.numeric(Sys.getenv("PCB_MIN_TIME", "0.4"))
ns <- asNamespace("miniextendr")
# The hand copies live in the namespace, like the generated wrapper, so they
# resolve the native symbol and the raise helper the same way.
cmp <- function(f) {
  environment(f) <- ns
  compiler::cmpfun(f)
}
generated <- ns$pcb_i32_u
full <- cmp(function(x) {
  .val <- .Call(C_miniextendr_pcb_i32_u, .call = sys.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))) return(.miniextendr_raise_condition(.val, sys.call()))
  .val
})
# The comparison is only meaningful while the hand copy is the generated body.
same <- identical(deparse(body(full)), deparse(body(generated)))
cat("hand copy matches the generated pcb_i32_u:", same, "\n")
if (!same) {
  cat("generated wrapper is now:\n")
  print(generated)
}
no_post <- cmp(function(x) .Call(C_miniextendr_pcb_i32_u, .call = sys.call(), x))
no_syscall <- cmp(function(x) {
  .val <- .Call(C_miniextendr_pcb_i32_u, .call = NULL, x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))) return(.miniextendr_raise_condition(.val, sys.call()))
  .val
})
bare <- cmp(function(x) .Call(C_miniextendr_pcb_i32_u, .call = NULL, x))
res <- list()
for (rep in seq_len(REPS)) {
  bm <- bench::mark(generated = generated(1L), hand_full = full(1L), no_post_check = no_post(1L),
                    no_sys_call = no_syscall(1L), bare_dotcall = bare(1L),
                    check = FALSE, min_time = MIN_TIME, max_iterations = 1e6)
  res[[rep]] <- data.frame(variant = as.character(bm$expression),
                           tmean_ns = vapply(bm$time, function(t) mean(as.numeric(t), trim = 0.05), numeric(1)) * 1e9)
}
a <- aggregate(tmean_ns ~ variant, do.call(rbind, res), median)
print(a[order(-a$tmean_ns), ], row.names = FALSE, digits = 4)
