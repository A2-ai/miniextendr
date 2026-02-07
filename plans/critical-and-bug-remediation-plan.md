# Critical and Bug Remediation Plan (Current Review Context)

## Scope
This plan captures concrete bugs and contract mismatches found during the current macro/macro-coverage review work:
1. `#[r_ffi_checked]` pointer-returning wrapper behavior mismatch (safety-sensitive)
2. S7 fallback dispatch/runtime behavior bugs
3. `miniextendr_module!` parser diagnostics inconsistency

Primary code areas:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl.rs`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl/tests.rs`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/worker.rs`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros-core/src/miniextendr_module.rs`

---

## Finding 1 [P1]: `r_ffi_checked` pointer-return behavior is inconsistent with documented contract

### Evidence
- Docs state pointer-returning functions cannot be routed and should panic off main thread:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs:2145`
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/worker.rs:30`
- Generated wrapper branch currently routes pointer-returning functions through `with_r_thread` and returns pointer back:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs:2265`
- The dedicated guard helper exists but is not used by generated wrappers:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/worker.rs:142`

### Task 1.1: Lock the contract
Decide and record one canonical rule. Recommended:
- pointer-returning `#[r_ffi_checked]` wrappers are **main-thread-only**
- off-main-thread calls panic with a clear message
- value-returning wrappers continue to route through `with_r_thread`

**Done when**:
- contract text is identical across macro docs and worker docs.

### Task 1.2: Fix pointer-return wrapper generation
Update pointer-return branch in `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs` to enforce main-thread-only behavior:

```rust
// Pointer-returning functions must stay on main thread.
#vis unsafe fn #fn_name(#inputs) #output {
    ::miniextendr_api::worker::assert_r_main_thread_for_pointer_api(stringify!(#fn_name));
    unsafe { #unchecked_name(#(#arg_names),*) }
}
```

This removes the current `with_r_thread(... Sendable(ptr) ...)` path for raw pointers.

**Done when**:
- generated pointer-return wrappers call `assert_r_main_thread_for_pointer_api`.
- no pointer-return branch routes via `with_r_thread`.

### Task 1.3: Add regression tests for macro output contract
Add tests that assert generated code shape for both branches:
- pointer return (`-> *mut T`) emits `assert_r_main_thread_for_pointer_api(...)`
- non-pointer return emits `with_r_thread(...)`

Recommended test location:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs` (unit test module) or dedicated tests file in `miniextendr-macros`.

**Done when**:
- tests fail on current implementation and pass after fix.

### Task 1.4: Update docs/examples
Align docs in:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-api/src/worker.rs`

Specifically remove any ambiguity that suggests routed pointer values are safe to use from worker thread.

**Done when**:
- all pointer-return sections consistently state main-thread-only + panic off-thread.

---

## Finding 2 [P2]: S7 fallback has real runtime/dispatch bugs

### Evidence
- hardcoded call path always uses `x@.ptr`:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl.rs:2919`
- generic override branch ignores fallback class:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl.rs:2953`
- fallback class is only applied in non-override path:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl.rs:3014`
- test is currently shallow:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl/tests.rs:1225`

### Task 2.1: Execute existing remediation plan
Use and complete:
- `/Users/elea/Documents/GitHub/miniextendr/plans/s7-fallback-remediation-plan.md`

That plan already contains self-contained code tasks, runtime tests, and docs updates.

**Done when**:
- all acceptance criteria in `/Users/elea/Documents/GitHub/miniextendr/plans/s7-fallback-remediation-plan.md` are met.

---

## Finding 3 [P3]: `miniextendr_module!` docs/diagnostic mismatch about item ordering

### Evidence
- docs say items may be in any order:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros-core/src/miniextendr_module.rs:467`
- missing-mod error text says `mod` is required as first item:
  - `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros-core/src/miniextendr_module.rs:613`
- parser logic itself does not enforce first position; it only requires presence.

### Task 3.1: Make diagnostics match actual parser behavior
Change missing-mod error text to remove “first item” claim:

```rust
"missing `mod <name>;` declaration in miniextendr_module!"
```

Keep semantics: exactly one `mod <name>;`, order-independent.

**Done when**:
- user-facing parser diagnostics and docs describe the same rule.

### Task 3.2: Add parser test for non-first `mod`
Add/extend parser test to confirm this remains valid:

```rust
miniextendr_module! {
    fn f;
    mod mypkg;
}
```

Expected: parses (or expands) without order-related error.

**Done when**:
- test protects against accidental future “mod must be first” regressions.

---

## Execution Order
1. Fix Finding 1 first (safety/contract risk).
2. Complete S7 fallback remediation plan (Finding 2).
3. Clean up parser diagnostics/tests (Finding 3).

---

## Verification Checklist
Run at minimum:

```sh
cargo test -p miniextendr-macros
cargo test -p miniextendr-macros-core
```

If S7 runtime tests are part of the change set:

```sh
just devtools-test FILTER="class-systems"
```

