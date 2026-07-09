# Shared helper for tests that must run in a fresh R subprocess.
#
# Several suites need to evaluate an expression in a clean R process with
# `miniextendr` freshly loaded — hazardous panic/thread/unwind paths that can
# corrupt the parent (test-subprocess-isolated.R) and the rayon thread-pool
# control tests, whose global pool builds once per process so each scenario
# needs its own process (test-rayon.R). This is the single implementation both
# use; `env_vars` lets callers pin/override environment variables (merged over
# `callr::rcmd_safe_env()`).
#
# NOT used by test-encoding.R's `run_load_in_locale`: that helper tests whether
# `library(miniextendr)` itself *fails* under a bad locale, so it wraps the load
# in tryCatch and returns a structured result — the opposite contract from
# "load succeeds, then eval expr". Folding it in would need a second code path
# and lose clarity, so it stays separate.
run_isolated <- function(expr, env_vars = character(), timeout = 30) {
  skip_if_not_installed("callr")
  env <- callr::rcmd_safe_env()
  if (length(env_vars)) {
    env[names(env_vars)] <- env_vars
  }
  callr::r(
    function(expr_to_eval) {
      library(miniextendr)
      eval(expr_to_eval)
    },
    args = list(expr_to_eval = substitute(expr)),
    env = env,
    timeout = timeout,
    error = "error"
  )
}
