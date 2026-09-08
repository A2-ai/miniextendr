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

Validation on R 4.6.1 passed 1,198 assertions across nine targeted R suites and a
GC protection fixture, with no failures, warnings, or skips. The new suite accounts
for 336 assertions covering real binding/promise/missing-argument errors, checked
warnings promoted to errors, nested guards, allocating destructors, original
messages and condition data, on.exit reentry, RNG cleanup, and GC pressure. The
worker stress regression also completed 2,000 error/reuse cycles.

The first temporary R runner accidentally muffled warnings in its outer observer,
preventing `warn = 2` from producing an error. Leaving those warnings untouched
restored R's normal behavior; the package code and regression test needed no
changes. A standalone C/R probe also verified exact error-buffer restoration for
nine message cases, including Unicode, percent signs, and R's 8,190-byte payload
limit, without notifying outer or global calling handlers. Compiler MIR verified
that consuming the boxed private payload through a normal Rust return frees its
allocation before the final R longjmp.

The full `just check` and `just test` recipes passed, including both UI suites.
`just clippy -- -D warnings`, all three exact CI clippy feature configurations,
`just lint`, formatting, template/AGENTS checks, and LLM documentation sync also
passed. The installed package has no undocumented R exports and its R API symbols
pass the installed R 4.6.1 non-API and registration checks.
