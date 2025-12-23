# Key Performance Highlights

## FFI Overhead is Minimal

- Basic FFI calls (integer_elt, real_ptr, xlength): 7-8ns
- Scalar creation: 8-11ns
- This is near-native performance

  ExternalPtr is Fast

- Access (as_ref, deref, as_ptr): 3.6ns - essentially pointer dereference
- Creation: 85-165ns for small/medium payloads
- Well-optimized for the common case

  ALTREP Overhead

- Plain vector element access: ~7ns
- ALTREP element access: ~170-500ns (20-70x slower)
- This is expected - ALTREP trades speed for memory/laziness
- The iterator-based ALTREP shows good consistency across sizes

Coercion: Rust Wins

| Operation           | R Coerce | Rust Coerce         |
|---------------------|----------|---------------------|
| scalar int→real     | 28ns     | 22ns                |
| vec[65536] int→real | 35-67µs  | 53µs                |
| vec[65536] raw→int  | 18µs     | 4.7µs (3.8x faster) |

Protection Overhead

- R's PROTECT/UNPROTECT: 16ns
- Preserve list (checked): 100ns
- Preserve (unchecked): 44ns
- R_UnwindProtect: 31ns

Allocator Comparison

R's allocator is 3-5x slower than system for small allocs, but handles R's GC integration.

Worker Thread

- Cross-thread dispatch: 5-10µs
- Main thread fast-path: 13ns

String Performance

- UTF-8 (no translation): 7ns pointer, 31ns to String
- Latin1 (needs translation): 250ns - 35x slower

Recommendations

1. Prefer slices over iterators when possible - iterate_int_slice is 100x faster than iterate_int_elt for large vectors
2. Use unchecked FFI in hot paths after validating inputs - saves ~2-3ns per call
3. ALTREP is best for large/lazy data - the overhead is amortized over many elements
4. UTF-8 strings are fast - Latin1 translation is the bottleneck

## RNative vs Coercion Path Performance

Size 4 (65,536 elements) - Most significant difference

| Operation                | Time    | Notes                                    |
|--------------------------|---------|------------------------------------------|
| slice_i32_zerocopy       | 19 ns   | Zero-copy, O(1) baseline                 |
| vec_i32_rnative          | 3.8 µs  | RNative: memcpy                          |
| vec_i64_coerce           | 6.9 µs  | Coercion: 1.8x slower than memcpy        |
| vec_u32_coerce_unchecked | 5.3 µs  | Unchecked coercion                       |
| vec_u32_coerce_checked   | 24.8 µs | TryCoerce with bounds check: 6.5x slower |
| vec_usize_coerce         | 24.8 µs | TryCoerce (same as u32 checked)          |

Key Insights

1. Zero-copy slice is O(1): ~19ns regardless of size - always prefer &[i32] when possible
2. RNative Vec uses memcpy: 3.8µs for 64K elements - fast memory copy
3. Widening coercion (i32→i64) adds ~80% overhead: 6.9µs vs 3.8µs

   - The extra cost is from element-by-element conversion

4. Checked coercion is expensive: TryCoerce<u32> for i32 values takes 24.8µs (6.5x slower)

   - The bounds checking (if x < 0 { return Err }) prevents vectorization
   - Use unchecked cast when you know values are valid

5. Size scaling:

   - Small vectors (1-16): Overhead is dominated by allocation (~35-45ns)
   - Large vectors (64K): Memcpy wins - 3.8µs vs 6.9µs for coercion

Recommendation

When possible:

- Use `&[i32] / &[f64]` for read-only access (zero-copy)
- Use `Vec<i32> / Vec<f64>` when you need ownership (memcpy)
- Avoid `Vec<i64> / Vec<u32>` with checked coercion in hot paths
