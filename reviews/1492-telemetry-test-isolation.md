# Telemetry assertions in parallel FFI guard tests (#1492)

While running the normal parallel Rust suite for #1429, the fallback test's
`boom` panic reached the process-global hook installed by the neighboring
telemetry test. That hook asserted that every message contained `test panic`.
The assertion failed, but `panic_telemetry::fire` correctly caught the callback
panic, so both tests still passed and the assertion appeared only as chatter.

The hook-owning test now lives in its own integration-test binary, with one
controlled lifecycle. Its callback only records reports through a channel.
Assertions run after clearing the hook, outside the suppression boundary.
Controlled calls on two threads verify exact messages, distinct sources and
fallback values without depending on event ordering. Successful calls and
calls after clearing the hook must produce no telemetry. Production code and
normal test parallelism are unchanged.

Validation passed: `just fmt`, `just check`, the full `just test` suite,
`just clippy -- -D warnings`, and all three CI Clippy feature configurations.
The R configure/install/force-document loop and `just llm-docs-check` also
passed without generated-file changes.
