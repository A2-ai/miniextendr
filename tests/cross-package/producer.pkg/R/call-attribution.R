# Hand-written delegates for the crate-level `call_attribution = "caller"`
# default (#1566) probed by `test-call-attribution.R`. Package-internal; tests
# reach them through `:::`.

# Delegates to a `noexport` entry point with no per-item `call = ...`: the
# crate default makes it report this function's call.
producer_attributed <- function(value) {
  producer_attributed_probe_impl(value)
}

# Delegates to a `call = wrapper` entry point: the per-item attribute wins and
# the error names the bridge.
producer_self_attributed <- function(value) {
  producer_self_attributed_probe_impl(value)
}
