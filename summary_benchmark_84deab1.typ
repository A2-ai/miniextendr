// Benchmark Summary for miniextendr
// Commit: 84deab1
// Date: 2026-01-13
// Platform: darwin (macOS)

#set page(margin: 1.5cm)
#set text(font: "New Computer Modern", size: 10pt)

= miniextendr Benchmark Summary
#text(gray)[Commit `84deab1` | 2026-01-13 | macOS Darwin 25.2.0]

== GC Protection (gc_protect.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`ProtectScope::protect` (single)], [13.8 ns], [14.21 ns], [14.42 ns],
  [`OwnedProtect::new`], [14.73 ns], [15.19 ns], [16.67 ns],
  [`ReprotectSlot::set`], [10.05 ns], [15.93 ns], [14.16 ns],
  [`scope.collect` (1000 i32s)], [1.457 µs], [1.52 µs], [1.597 µs],
  [`scope.collect` (10000 i32s)], [13.79 µs], [14.58 µs], [14.57 µs],
  [`ListAccumulator::push` (100 items)], [3.916 µs], [4.395 µs], [4.491 µs],
)

== Unwind Protection (unwind_protect.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`with_r_unwind_protect` (noop)], [32.93 ns], [33.58 ns], [33.74 ns],
  [Direct call (baseline)], [0.002 ns], [0.002 ns], [0.002 ns],
)

_Note: The ~33ns overhead is the cost of `R_UnwindProtect` wrapping for panic safety._

== FFI Checked vs Unchecked (rffi_checked.rs)

The `_unchecked` variants skip the thread safety check (`is_r_main_thread()`).

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`Rf_ScalarInteger` (checked)], [7.455 ns], [7.781 ns], [8.099 ns],
  [`Rf_ScalarInteger_unchecked`], [7.781 ns], [10.18 ns], [10.94 ns],
  [`Rf_xlength` (checked)], [9.0 ns], [9.08 ns], [9.34 ns],
  [`Rf_xlength_unchecked`], [6.56 ns], [6.64 ns], [6.73 ns],
  [`Rf_allocVector` (1 elem, checked)], [6.2 ns], [6.26 ns], [12.3 ns],
  [`Rf_allocVector_unchecked` (1 elem)], [4.9 ns], [7.38 ns], [8.66 ns],
  [`Rf_allocVector` (65536 elem, checked)], [213 ns], [224 ns], [5.19 µs],
  [`Rf_allocVector_unchecked` (65536 elem)], [212 ns], [216 ns], [1.48 µs],
)

_Observation: Checked overhead is ~2-3ns for simple functions. Variance in large allocations is due to GC._

== Protection Strategy Comparison (refcount_protect.rs)

Comparing different approaches for protecting many values.

=== R's Protect Stack (ProtectScope)
Limited by R's `--max-ppsize` (default 50000).

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Count*], [*Fastest*], [*Median*], [*Mean*],
  [10,000], [111.5 µs], [141.7 µs], [167.5 µs],
  [30,000], [315.7 µs], [348.6 µs], [391.8 µs],
  [49,000], [514.4 µs], [551.8 µs], [603.2 µs],
  [49,900], [532.4 µs], [548.9 µs], [638.1 µs],
)

=== Thread-Local Arena (no ppsize limit)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Count*], [*Fastest*], [*Median*], [*Mean*],
  [50,000], [3.691 ms], [4.103 ms], [4.353 ms],
  [100,000], [7.599 ms], [8.523 ms], [8.788 ms],
  [200,000], [15.85 ms], [18.2 ms], [18.69 ms],
  [500,000], [43.1 ms], [51.62 ms], [52.23 ms],
)

=== RefCount Arena (no ppsize limit)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Count*], [*Fastest*], [*Median*], [*Mean*],
  [50,000], [3.524 ms], [4.042 ms], [4.137 ms],
  [100,000], [7.607 ms], [8.593 ms], [9.033 ms],
  [200,000], [15.46 ms], [17.71 ms], [18.5 ms],
  [500,000], [57.25 ms], [75.46 ms], [75.22 ms],
)

== Zero-Copy Slice Access (native_vs_coerce.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`slice_i32_zerocopy` (any size)], [20.73 ns], [20.89 ns], [22.26 ns],
  [`slice_f64_zerocopy` (any size)], [20.73 ns], [20.89 ns], [22.56 ns],
  [`slice_u8_zerocopy` (any size)], [20.73 ns], [25.69 ns], [24.28 ns],
  [`int_to_vec_i32_memcpy` (100k)], [3.749 µs], [3.916 µs], [4.344 µs],
  [`vec_i32_rnative` (100k)], [3.749 µs], [3.791 µs], [3.815 µs],
)

_Key insight: Zero-copy slicing is O(1) at ~21ns regardless of vector size._

== Worker Thread (worker.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`run_on_worker` (no R calls)], [2.749 µs], [5.603 µs], [6.196 µs],
  [`run_on_worker` (with R thread)], [6.208 µs], [11.47 µs], [11.44 µs],
  [`with_r_thread` (already on main)], [11.53 ns], [16.25 ns], [16.17 ns],
)

== Preserve/Release (preserve.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`preserve_insert_release`], [53.1 ns], [54.41 ns], [68.2 ns],
  [`preserve_insert_release_unchecked`], [29.02 ns], [40.58 ns], [41.4 ns],
  [`protect_unprotect` (R stack)], [14.05 ns], [18.28 ns], [19.96 ns],
)

== String Operations (strings.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`from_r_cstr` (short)], [14.95 ns], [17.43 ns], [17.25 ns],
  [`from_r_cstr` (1000 chars)], [185.9 ns], [195 ns], [213.2 ns],
  [`from_r_cstr` (10000 chars)], [2.354 µs], [2.999 µs], [2.771 µs],
  [`into_r_empty` (BlankString)], [0.003 ns], [0.042 ns], [0.043 ns],
  [`into_r_str_mkcharlen` (short)], [8.278 ns], [8.603 ns], [9.099 ns],
  [`into_r_str_mkcharlen` (65536 chars)], [61.45 µs], [68.54 µs], [68.11 µs],
)

== Encoding Translation (translate.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`charsxp_r_char_ptr_utf8`], [7.302 ns], [7.384 ns], [7.461 ns],
  [`charsxp_translate_ptr_utf8`], [7.302 ns], [7.424 ns], [7.432 ns],
  [`charsxp_translate_ptr_latin1`], [207.7 ns], [249.7 ns], [352.2 ns],
  [`charsxp_translate_to_string_utf8`], [31.63 ns], [31.96 ns], [32.12 ns],
  [`charsxp_translate_to_string_latin1`], [249.7 ns], [290.7 ns], [403.9 ns],
)

_UTF-8 is fast (no conversion needed). Latin-1 requires translation (~250ns)._

== Trait ABI / Cross-Package (trait_abi.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark*], [*Fastest*], [*Median*], [*Mean*],
  [`mx_query_vtable`], [0.812 ns], [0.832 ns], [0.843 ns],
  [`query_view_value`], [25.78 ns], [33.91 ns], [33.62 ns],
  [`view_value_only`], [24.63 ns], [29.59 ns], [30.85 ns],
  [Direct increment (baseline)], [0.003 ns], [0.016 ns], [0.013 ns],
)

== Matrix Operations (rarray.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark (100x100 matrix)*], [*Fastest*], [*Median*], [*Mean*],
  [`sum_as_slice`], [2.978 µs], [3.062 µs], [3.054 µs],
  [`sum_by_column_slices`], [4.915 µs], [4.999 µs], [5.196 µs],
  [`sum_get_rc` (row/col indexing)], [132.3 µs], [132.4 µs], [133 µs],
  [`to_vec`], [364.2 ns], [403.3 ns], [452.5 ns],
)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Benchmark (1000x1000 matrix)*], [*Fastest*], [*Median*], [*Mean*],
  [`sum_as_slice`], [48.54 µs], [49.37 µs], [49.43 µs],
  [`sum_by_column_slices`], [55.45 µs], [55.79 µs], [55.9 µs],
  [`sum_get_rc` (row/col indexing)], [2.119 ms], [2.123 ms], [2.142 ms],
  [`to_vec`], [6.916 µs], [7.041 µs], [7.97 µs],
)

_Key insight: `as_slice()` is 40x faster than row/col indexing for sum operations._

== R Allocator (allocator.rs)

#table(
  columns: (auto, auto, auto, auto),
  align: (left, right, right, right),
  [*Size*], [*Fastest*], [*Median*], [*Mean*],
  [8 bytes], [58.32 ns], [75.9 ns], [84.19 ns],
  [64 bytes], [60.27 ns], [77.85 ns], [81.86 ns],
  [4096 bytes], [60.6 ns], [78.17 ns], [82.56 ns],
  [65536 bytes], [164.2 ns], [172.5 ns], [217.5 ns],
)

== Summary

#table(
  columns: (auto, auto),
  align: (left, left),
  [*Operation*], [*Typical Cost*],
  [Zero-copy slice access], [~21 ns (constant)],
  [Thread safety check], [~2-3 ns],
  [R_UnwindProtect wrapper], [~33 ns],
  [Single PROTECT/UNPROTECT], [~14-18 ns],
  [Worker thread roundtrip], [~6-11 µs],
  [String from R (short)], [~15-17 ns],
  [String to R (short)], [~8-9 ns],
  [VTable lookup], [~0.8 ns],
  [Cross-package view], [~25-30 ns],
)
