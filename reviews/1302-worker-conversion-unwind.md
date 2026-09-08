# Worker input conversion cleanup (#1302)

The initial regression created a resource in an earlier argument and another in
an R binding converter, then triggered an R error while reading the binding.
Both worker and main-thread wrappers reported two resources created, zero dropped,
and zero calls dispatched. Wrapping the entire conversion body in the existing
helper would not fix this: R_UnwindProtect invokes its cleanup callback after
SETJMP has already skipped that body's Rust frames.

The worker fix fences individual checked R API calls below converter locals.
A private panic transports the preserved R continuation through Rust cleanup to
the generated wrapper, which finishes RNG cleanup before resuming R. Checked
calls suspend inherited conversion fencing while R evaluates callbacks, and the
scope ends before worker dispatch. Nested legacy guards forward this private
payload instead of treating it as a Rust panic.

Each pending error owns a separate continuation. Successful calls recycle tokens;
error tokens are retired under R's protection stack before continuation resume,
because on.exit code can reenter worker wrappers before R installs the original
return value. Tokens are allocated at wrapper entry, never just before protecting
a fresh conversion result. R's deferred simple-error message is copied before
Rust destructors can overwrite the global error buffer, then restored behind a
private exiting handler after Rust and RNG cleanup.

The broader skipped-local problem in main-thread bodies, worker callbacks, and
unchecked calls is tracked in [#1507](https://github.com/A2-ai/miniextendr/issues/1507).
The regression keeps a main-thread control, but the worker fix does not claim that
an outer guard now makes arbitrary unchecked R calls safe.
