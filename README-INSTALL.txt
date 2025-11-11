
HOW TO INTEGRATE THIS ALTREP SKELETON INTO YOUR PROJECT
======================================================

This drops into your existing "miniextendr" workspace and does **not** rely on libR-sys or extendr.
It uses only raw FFI declarations and your own minimal SEXP types.

Files provided:
--------------
1) miniextendr/miniextendr-api/src/altrep.rs     <-- NEW: trait-backed ALTREP core (INT, REAL, STRING)
2) miniextendr/rpkg/R/altrep.R                   <-- NEW: R wrappers that call the .Call() entry points

What it gives you:
------------------
- One ALTREP class per base kind (registered lazily): rust_altint, rust_altreal, rust_altstr.
- Traits any Rust struct can implement: IntBackend, RealBackend, StringBackend.
- Ready-made example backends:
    * CompactIntSeq  (start + i * step)
    * OwnedReal      (owned Box<[f64]> copy of a REALSXP)
    * Utf8Vec        (owned Vec<String> copy of a STRSXP)
- .Call()-style exported C functions:
    * C_altrep_compact_int(call, n, start, step)
    * C_altrep_from_doubles(call, x)
    * C_altrep_from_strings(call, x)

Steps:
------
A) Add the Rust module
   - Copy src/altrep.rs into miniextendr/miniextendr-api/src/altrep.rs
   - In miniextendr/miniextendr-api/src/lib.rs, add:

       pub mod altrep;

     (No other changes required; the C entry points are #[no_mangle] and will be exported.)

B) Add the R wrappers
   - Copy R/altrep.R into miniextendr/rpkg/R/altrep.R

C) Rebuild and install the R package
   - From the top-level "miniextendr" directory:
       R CMD INSTALL rpkg

   The package will build the Rust workspace and register the .Call() routines.

D) Try it in R
   library(rpkg)
   x <- altrep_compact_int(10L, 5L, 1L)
   x[1:5]
   as.integer(x)

   y <- altrep_from_doubles(runif(5))
   y[1:5]

   s <- altrep_from_strings(c("a","b",NA,"d"))
   s[1:4]

Notes:
------
- This implementation deliberately avoids exposing DATAPTR() unless the backend owns a contiguous buffer.
  For borrowed data, do not implement dataptr() — R will use Elt/Get_region.
- Serialization is omitted to keep it small. If you need save/load, mirror the "Serialized_state/Unserialize"
  declarations from R_ext/Altrep.h and set the corresponding setters in ensure_classes().
- LOGICAL / RAW / COMPLEX / LIST can be added by copying the pattern in altrep.rs and adjusting signatures.
- The String backend uses translateCharUTF8() and mkCharLen(). If you need explicit UTF-8 CHARSXPs, switch
  to mkCharLenCE(..., CE_UTF8) (declare the FFI for it and the cetype_t constant).

Security & Safety:
------------------
- Do not return a writable DATAPTR unless you control the memory. Otherwise, return NULL and let R fall back.
- The external pointer finalizers free the boxed trait objects.
- This code assumes your R wrappers pass the correct types (integers, doubles, characters). Keep it that way.
