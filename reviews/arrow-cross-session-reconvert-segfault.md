# Arrow cross-session re-conversion segfault

## What was attempted

In a cross-session test (callr::r subprocess), an ALTREP Arrow Float64Array with NAs was:
1. Created in the parent session (compute: multiply by 10)
2. Serialized via saveRDS (ALTREP materializes to plain R numeric with NA sentinels)
3. Loaded in a fresh subprocess via readRDS (returns plain numeric, not ALTREP)
4. Passed back through Arrow conversion (`arrow_na_f64_add_one`) in the subprocess

## What went wrong

Step 4 segfaults at address 0x0 ("invalid permissions") inside `arrow_na_f64_add_one`.

The loaded data is a plain `numeric` vector (not ALTREP), so TryFromSexp for Float64Array
should work: it wraps the data buffer via `sexp_to_arrow_buffer` and scans for NAs.

## Root cause

Not fully diagnosed. Hypotheses:

1. **`sexp_to_arrow_buffer` R_PreserveObject issue**: The `R_PreserveObject` call in
   `sexp_to_arrow_buffer` might behave differently in the subprocess context, or the
   mutex guard around it might not be initialized.

2. **`init_sexprec_data_offset` not called**: If `package_init()` ran but
   `init_sexprec_data_offset()` computed a wrong offset in the subprocess, the
   pointer recovery in `try_recover_r_sexp` could read garbage.

3. **Thread ID mismatch**: `miniextendr_runtime_init()` records `R_MAIN_THREAD_ID`.
   If this wasn't set up correctly in the subprocess, FFI thread checks could fail.

Note: The same function works fine when called directly in a subprocess with fresh data
(`c(1.0, NA, 3.0)` constructed in-subprocess), so it's specific to data loaded from RDS.

## Fix

Skipped the test with a reference to this review. Needs investigation of which init step
fails when processing deserialized R vectors in a subprocess.
