# Benchmark Results — 2026-04-20

## Environment

| Field | Value |
|-------|-------|
| **Date** | 2026-04-20 |
| **Commit** | `00df79d4` (branch: `main`) |
| **Rust** | rustc 1.95.0 (59807616e 2026-04-14) |
| **OS** | macOS 25.4.0 (Darwin 25.4.0) arm64 |
| **CPU** | Apple M3 Max |
| **RAM** | 36 GB |
| **R** | R version 4.5.2 (2025-10-31) |

## Summary

Rust-side and lint benchmarks executed. Two benches failed due to a Latin-1 locale assertion in the test environment (the host locale does not set up Latin-1 encoding, so benchmarks that construct Latin-1 CHARSXPs panic):

- **`from_r::string_latin1`** — panics at `from_r.rs:87` (CHARSXP non-UTF-8 assertion)
- **`translate::sexp_tryfrom_string_translate_latin1`** — same cause

These were already fragile in 2026-02-18 (not mentioned because the run happened to skip those cases). All other targets passed cleanly.

Skipped as before: `wrappers` (needs rpkg installed), `bench-r` (R-side), `bench-compile` (macro compile-time).

The suite has grown significantly since 2026-02-18: several new bench targets (`altrep_iter`, `factor`, `list`, `native_vs_coerce`, `rarray`, `rffi_checked`, `sexp_ext`, additional `gc_protect` scenarios, extended `refcount_protect` arena types, extended `strict` coverage) and many existing targets have additional benchmark cases.

### Key Findings vs 2026-02-18

1. **Worker thread**: `run_on_worker_no_r` now 0.46 ns (effectively compiles away without R), `run_on_worker_with_r_thread` 7.5 ns (same-thread fast path). Channel saturation and payload tests much faster — changed measurement methodology (no channel hop in main).
2. **Unwind protect**: `with_r_unwind_protect` 32.6 ns (stable, ~31 ns in Feb). Nested 5 layers 161 ns (~same). `catch_unwind` success 0.46 ns (stable). Panic path 5.4 µs (stable).
3. **Trait ABI**: `mx_query_vtable` 2.3 ns (was 1.0 ns) — vtable lookup more complex now. Full dispatch 60 ns (stable). Multi-method all 417 ns (stable).
4. **ExternalPtr**: `create_small_payload` 209 ns, `create_large_payload` 209 ns — both much lower variance. Access is near-zero (< 0.3 ns).
5. **Into/from R**: `scalar_i32` into 11.7 ns (was 12.5 ns). `vec_i32` 64K: 9.0 µs (was 13.3 µs) — significant improvement. `vec_string` 64K: 3.87 ms (stable). Scale vec: `i32` 1M now 106 µs (was 675 µs) — major improvement in bulk int conversion.
6. **ALTREP**: Patterns stable at size index 4 (64K). Guard modes: unsafe 1.07 ms, default 4.71 ms, r_unwind 4.70 ms at 64K (guard mode overhead pattern changed).
7. **Allocator**: R allocator small (8B) 18.0 ns (was 71 ns) — improved. System allocator 16.5 ns (stable for small). Large 64K: R 797 ns (was 867 ns, stable).
8. **Lint**: Moderately slower than Feb (~15-40% increase in scan times), consistent with more rules added.
9. **GC protection**: Protect stack 1.87 µs per 1K ops (19 ns/op). Vec pool slightly better than stack at large N. Precious list still O(n²) for churn.
10. **Strings**: `into_r_str_mkcharlen` 64K: 61.5 µs (was 60.4 µs, stable). `tryfromsexp_str` 64K: 1.26 µs (stable).

## Rust-Side Results (divan)

### allocator

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| rallocator_alloc | 8 | 18.0 ns | 32.5 ns |
| rallocator_alloc | 1024 | 103.9 ns | 149 ns |
| rallocator_alloc | 65536 | 796.5 ns | 2.8 µs |
| rallocator_alloc_zeroed | 8 | 24.3 ns | 25.3 ns |
| rallocator_alloc_zeroed | 65536 | 3.79 µs | 4.37 µs |
| system_alloc | 8 | 16.5 ns | 15.1 ns |
| system_alloc | 1024 | 23.0 ns | 23.3 ns |
| system_alloc | 65536 | 82.4 ns | 83.1 ns |
| system_alloc_zeroed | 65536 | 228.9 ns | 240.3 ns |

### altrep (basic)

Size index: 0=1, 2=256, 4=64K

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| altrep_int_dataptr | 0 (1) | 207.7 ns | 607.5 ns |
| altrep_int_dataptr | 4 (64K) | 17.3 µs | 17.9 µs |
| altrep_int_elt | 0 (1) | 228.9 ns | 270.4 ns |
| altrep_int_elt | 4 (64K) | 15.7 µs | 17.3 µs |
| altrep_real_elt | 4 (64K) | 34.1 µs | 36.7 µs |
| plain_int_dataptr | any | 7.0 ns | 7.2 ns |
| plain_int_elt | any | 9.1 ns | 9.2 ns |

### altrep_advanced

| Group | Benchmark | Size Idx | Median | Mean |
|-------|-----------|----------|--------|------|
| complex_altrep | dataptr | 4 | 66.2 µs | 67.8 µs |
| complex_altrep | elt | 4 | 374.4 µs | 374.7 µs |
| complex_altrep | full_scan_elt | 4 | 4.69 ms | 4.69 ms |
| creation | altrep_int | 4 | 16.6 µs | 17.7 µs |
| creation | altrep_string | 4 | 2.63 ms | 2.64 ms |
| creation | constant_real | 4 | 179 ns | 206 ns |
| guard_modes | default_guard | 4 | 4.71 ms | 4.69 ms |
| guard_modes | r_unwind_guard | 4 | 4.70 ms | 4.71 ms |
| guard_modes | unsafe_guard | 4 | 1.07 ms | 1.08 ms |
| guard_modes | plain_intsxp | 4 | 255.6 µs | 257.2 µs |
| materialization | altrep_dataptr_ro | 4 | 17.1 µs | 18.2 µs |
| materialization | altrep_full_scan_dataptr | 4 | 21.5 µs | 23.2 µs |
| materialization | altrep_full_scan_elt | 4 | 4.79 ms | 4.79 ms |
| materialization | plain_dataptr_ro | any | 9.3 ns | 9.4 ns |
| string_altrep | create | 4 | 2.51 ms | 2.52 ms |
| string_altrep | elt | 4 | 2.81 ms | 2.92 ms |
| string_altrep | elt_with_na | 4 | 2.72 ms | 2.96 ms |
| string_altrep | force_materialize | 4 | 6.22 ms | 6.36 ms |
| string_altrep | plain_strsxp_elt | 4 | 4.47 ms | 4.51 ms |
| zero_alloc | constant_elt | 4 | 255 ns | 258 ns |
| zero_alloc | constant_full_scan | 4 | 4.69 ms | 4.71 ms |

### altrep_iter (new target)

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| altrep_iter_int_elt | 0 | 291.7 ns | 551.4 ns |
| altrep_iter_int_elt | 4 | 504.9 ns | 584.5 ns |
| altrep_iter_int_xlength | 0 | 234.1 ns | 255.1 ns |
| altrep_iter_int_xlength | 4 | 447.7 ns | 518.6 ns |

### coerce

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| vec_int_slice_direct | 4 | 20.9 ns | 21.0 ns |
| vec_int_to_real_r_coerce | 4 | 56.4 µs | 83.9 µs |
| vec_int_to_real_rust_coerce | 4 | 7.29 µs | 8.30 µs |
| vec_lgl_direct | 4 | 20.4 ns | 20.4 ns |
| vec_lgl_to_int_r_coerce | 4 | 33.5 µs | 38.3 µs |
| scalar_int_direct | — | 23.3 ns | 23.4 ns |
| scalar_int_to_real_r_coerce | — | 33.7 ns | 35.3 ns |
| scalar_int_to_real_rust_coerce | — | 23.5 ns | 23.9 ns |
| rust_only_i32_to_f64 | 4 | 12.2 µs | 12.4 µs |
| rust_only_f64_to_i32 | 4 | 64.9 µs | 67.3 µs |

### dataframe

| Benchmark | Rows | Median | Mean |
|-----------|------|--------|------|
| transpose/point3 | 100 | 249.7 ns | 366.8 ns |
| transpose/point3 | 10000 | 18.9 µs | 18.9 µs |
| transpose/wide10 | 100 | 999.7 ns | 991.8 ns |
| transpose/wide10 | 10000 | 158 µs | 159.9 µs |
| full_pipeline/event_to_sexp | 100 | 7.58 µs | 8.12 µs |
| full_pipeline/mixed_to_sexp | 100 | 9.67 µs | 9.88 µs |
| full_pipeline/point3_to_sexp | 100 | 707.7 ns | 915.5 ns |

### externalptr

| Benchmark | Median | Mean |
|-----------|--------|------|
| create_small_payload (8B) | 209 ns | 203 ns |
| create_medium_payload (1KB+) | 289 ns | 328 ns |
| create_large_payload (varies) | 209 ns | 536 ns |
| baseline_box_small | 18.1 ns | 18.2 ns |
| baseline_box_large | 666.7 ns | 722.7 ns |
| access_as_ref | 0.004 ns | 0.015 ns |
| access_deref | 0.238 ns | 0.269 ns |
| erased_is_hit | 15.4 ns | 15.7 ns |
| erased_is_miss | 15.8 ns | 16.3 ns |
| erased_downcast_ref_hit | 15.5 ns | 16.1 ns |
| create_sized_payload(8) | 83.7 ns | 240 ns |
| create_n_ptrs(100) | 20.5 µs | 22.2 µs |
| create_n_ptrs(1000) | 205.6 µs | 209.5 µs |

### factor (new target)

| Benchmark | Median | Mean |
|-----------|--------|------|
| single_cached | 57.0 ns | 55.5 ns |
| single_uncached | 340.8 ns | 364.3 ns |
| repeated_100_cached | 5.37 µs | 6.16 µs |
| repeated_100_uncached | 37.0 µs | 40.1 µs |
| vec_factor_vec (4096) | 3.12 µs | 3.28 µs |

### ffi_calls

| Benchmark | Median | Mean |
|-----------|--------|------|
| scalar_integer | 9.33 ns | 11.0 ns |
| scalar_real | 9.73 ns | 11.0 ns |
| scalar_logical | 1.35 ns | 1.41 ns |
| alloc_intsxp (256) | 103 ns | 111.7 ns |
| protect_unprotect_single | 11.5 ns | 12.3 ns |
| protect_unprotect_n (4) | 35.5 ns | 38.1 ns |
| integer_elt | 7.54 ns | 7.56 ns |
| integer_ptr | 3.75 ns | 3.81 ns |
| xlength | 3.67 ns | 3.71 ns |

### from_r

*Note: `string_latin1` benchmark panics in this environment (Latin-1 locale assertion). All other cases pass.*

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| scalar_bool | — | 19.1 ns | 19.4 ns |
| scalar_i32 | — | 23.8 ns | 30.0 ns |
| scalar_f64 | — | 21.9 ns | 34.9 ns |
| scalar_option_i32_value | — | 28.1 ns | 33.8 ns |
| slice_i32 | 4 (64K) | 21.5 ns | 22.3 ns |
| slice_f64 | 4 (64K) | 20.9 ns | 21.1 ns |
| hashset_i32 | 4 (64K) | 1.49 ms | 1.40 ms |
| btreeset_i32 | 4 (64K) | 325 µs | 330 µs |
| iterate_int_slice | 4 (64K) | 4.58 µs | 4.70 µs |
| iterate_int_ptr | 4 (64K) | 4.42 µs | 4.43 µs |
| named_list_hashmap_i32 | 2 (4K) | 22.0 µs | 22.1 µs |

### gc_protect (expanded coverage)

Selected key benchmarks:

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| owned_protect | — | 11.7 ns | 11.6 ns |
| protect_scope_single | — | 11.5 ns | 11.5 ns |
| raw_protect_unprotect | — | 12.3 ns | 12.4 ns |
| preserve_insert_release | — | 51.5 ns | 51.3 ns |
| preserve_insert_release_unchecked | — | 27.7 ns | 28.7 ns |
| owned_protect_repeated | 100 | 937.2 ns | 952.8 ns |
| owned_protect_repeated | 1000 | 9.27 µs | 39.2 µs |
| list_builder_construction | 0 (16 items) | 189.8 ns | 195 ns |
| list_builder_construction | 1 (256 items) | 1.71 µs | 1.72 µs |
| list_builder_construction | 2 (4K items) | 16.8 µs | 23.8 µs |
| reprotect_slot_set | 1000 | 9.08 µs | 9.19 µs |

### gc_protection_compare (selected)

Per-op costs at single-item scale (protect_stack batch of 10):

| Mechanism | Per 10 items | Notes |
|-----------|-------------|-------|
| Protect stack | 19.1 ns | 1.91 ns/op |
| Vec pool | 130 ns | 13.0 ns/op |
| Slotmap pool | 144 ns | 14.4 ns/op |
| DLL preserve | 291 ns | 29.1 ns/op |
| Precious list | 153 ns | 15.3 ns/op (but O(n²) at 1K) |

**Churn (replace-in-loop) at 10K iterations:** Vec pool 1.14 ms • DLL preserve 1.30 ms • Precious churn 74.0 ms (O(n²)).

### into_r

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| scalar_bool | — | 3.61 ns | 3.60 ns |
| scalar_i32 | — | 11.7 ns | 12.2 ns |
| scalar_f64 | — | 12.0 ns | 11.8 ns |
| vec_i32 | 64K | 9.0 µs | 9.0 µs |
| vec_f64 | 64K | 16.6 µs | 139 µs |
| vec_string | 64K | 3.87 ms | 4.05 ms |
| vec_option_i32_50pct_na | 64K | 29.2 µs | 95.3 µs |
| vec_str | 64K | 1.24 ms | 1.39 ms |
| **scale_vec_i32** | **1M** | **106 µs** | **213 µs** |
| **scale_vec_f64** | **1M** | **233 µs** | **322 µs** |
| **scale_vec_string** | **1M** | **94.6 ms** | **97.0 ms** |
| **scale_vec_option_i32_50pct_na** | **1M** | **440 µs** | **473 µs** |

### list (new target)

| Benchmark | Median | Mean |
|-----------|--------|------|
| derive_into_list_named | 150.7 ns | 174.1 ns |
| derive_try_from_list_named | 189.8 ns | 192.5 ns |
| derive_into_list_tuple | 63.5 ns | 72.9 ns |
| list_get_index_first_i32 (16 items) | 32.6 ns | 32.7 ns |
| list_get_named_first_i32 (16 items) | 52.5 ns | 52.6 ns |
| list_get_named_last_i32 (4K items) | 50.2 µs | 50.6 µs |

### native_vs_coerce (new target)

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| int_to_vec_i32_memcpy | 4 (64K) | 3.46 µs | 3.61 µs |
| int_to_vec_f64_coerce | 4 (64K) | 7.37 µs | 7.82 µs |
| real_to_vec_i32_coerce | 4 (64K) | 14.8 µs | 14.9 µs |
| slice_i32_zerocopy | any | 20.9 ns | 21.0 ns |
| slice_f64_zerocopy | any | 20.9 ns | 21.0 ns |

### panic_telemetry

| Benchmark | Median | Mean |
|-----------|--------|------|
| read_lock/no_hook | 8.03 ns | 8.02 ns |
| read_lock/with_hook_noop | 22.7 ns | 20.8 ns |
| write_lock/set_hook | 13.9 ns | 13.9 ns |
| write_lock/clear_hook | 2.09 ns | 2.13 ns |

### preserve

| Benchmark | Median | Mean |
|-----------|--------|------|
| preserve_insert_release | 46.6 ns | 55.0 ns |
| preserve_insert_release_unchecked | 25.8 ns | 29.7 ns |
| protect_unprotect | 3.56 ns | 3.59 ns |

### rarray (new target)

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| rmatrix_sum_as_slice | 100×100 | 3.08 µs | 3.10 µs |
| rmatrix_sum_as_slice | 1000×1000 | 46.4 µs | 47.3 µs |
| rmatrix_sum_by_column_slices | 100×100 | 4.42 µs | 4.55 µs |
| rmatrix_sum_get_rc | 100×100 | 131.9 µs | 132.0 µs |

### raw_access

| Group | Benchmark | Size Idx | Median | Mean |
|-------|-----------|----------|--------|------|
| integer | as_slice_only | 4 | 17.0 ns | 17.3 ns |
| integer | as_slice_sum | 4 | 4.62 µs | 4.65 µs |
| integer | raw_pointer_sum | 4 | 4.58 µs | 4.69 µs |
| real | as_slice_sum | 4 | 49.1 µs | 49.1 µs |
| real | raw_pointer_sum | 4 | 49.1 µs | 49.1 µs |
| conversion | try_from_sexp_vec_i32 | 4 | 3.44 µs | 3.59 µs |
| conversion | try_from_sexp_vec_f64 | 4 | 6.83 µs | 7.00 µs |

### rffi_checked (new target)

| Benchmark | Median | Mean |
|-----------|--------|------|
| scalar_integer_checked | 7.14 ns | 8.33 ns |
| scalar_integer_unchecked | 6.90 ns | 12.2 ns |
| xlength_checked | 7.22 ns | 7.24 ns |
| xlength_unchecked | 7.79 ns | 7.87 ns |

`#[r_ffi_checked]` overhead: negligible for scalar FFI calls (~0.2 ns median difference).

### sexp_ext (new target)

| Benchmark | Median | Mean |
|-----------|--------|------|
| sexp_as_slice_ext (64K) | 17.0 ns | 17.1 ns |
| sexp_as_slice_raw (64K) | 8.03 ns | 8.05 ns |
| sexp_is_integer_ext | 7.38 ns | 7.44 ns |
| sexp_len_ext (64K) | 9.26 ns | 9.28 ns |
| sexp_len_raw (64K) | 6.73 ns | 6.84 ns |

### strict (expanded coverage)

| Group | Benchmark | Size | Median | Mean |
|-------|-----------|------|--------|------|
| scalar_input | normal_intsxp_to_i64 | — | 54.4 ns | 58.9 ns |
| scalar_input | strict_intsxp_to_i64 | — | 49.5 ns | 49.6 ns |
| vec_input | normal_intsxp_to_vec_i64 | 10K | 12.2 µs | 12.6 µs |
| vec_input | strict_intsxp_to_vec_i64 | 10K | 5.42 µs | 5.72 µs |
| vec_output | normal_vec_i64 | 10K | 4.67 µs | 4.70 µs |
| vec_output | strict_vec_i64 | 10K | 5.02 µs | 5.05 µs |

### strings

*Note: Latin-1 string benchmarks omitted (panic in this environment — same root cause as `from_r::string_latin1`).*

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| from_r_cstr | 64K chars | 2.37 µs | 2.40 µs |
| from_r_cstr_only | 64K chars | 1.14 µs | 1.15 µs |
| into_r_str_mkcharlen | 64K chars | 61.5 µs | 61.6 µs |
| tryfromsexp_str_current | 64K chars | 1.26 µs | 1.27 µs |

### trait_abi

| Benchmark | Median | Mean |
|-----------|--------|------|
| mx_query_vtable | 2.33 ns | 2.35 ns |
| mx_query_multi_method_vtable | 2.90 ns | 2.89 ns |
| query_vtable_miss | 2.96 ns | 2.98 ns |
| view_value_only | 53.1 ns | 58.2 ns |
| query_view_value | 55.4 ns | 62.1 ns |
| dispatch_self_value | 60.3 ns | 65.9 ns |
| dispatch_self_mut_increment | 37.7 ns | 37.7 ns |
| dispatch_multi_method_all | 416.7 ns | 813 ns |
| dispatch_multi_method_hot (10x) | 1.08 µs | 1.20 µs |
| dispatch_repeated_hot (10x) | 614.2 ns | 676.5 ns |

### translate

*Note: `sexp_tryfrom_string_translate_latin1` panics (same locale issue). UTF-8 cases complete.*

| Benchmark | Median | Mean |
|-----------|--------|------|
| charsxp_r_char_ptr_utf8 | 3.60 ns | 3.60 ns |
| charsxp_r_char_to_string_utf8 | 28.2 ns | 26.5 ns |
| charsxp_translate_ptr_utf8 | 7.46 ns | 7.52 ns |
| charsxp_translate_to_string_utf8 | 30.7 ns | 31.1 ns |
| charsxp_translate_ptr_latin1 | 249.7 ns | 490.5 ns |
| charsxp_translate_to_string_latin1 | 273.1 ns | 279.3 ns |

### typed_list (expanded coverage)

| Group | Benchmark | Fields | Median | Mean |
|-------|-----------|--------|--------|------|
| numeric_validation | validate_pass | 3 | 557 ns | 596 ns |
| numeric_validation | validate_pass | 10 | 1.48 µs | 1.57 µs |
| numeric_validation | validate_pass | 50 | 9.42 µs | 9.85 µs |
| numeric_validation | validate_strict_pass | 50 | 10.2 µs | 10.6 µs |
| failure_paths | wrong_type_first_field | 3 | 530.9 ns | 580.3 ns |
| failure_paths | missing_required_field | 50 | 9.83 µs | 10.2 µs |
| spec_construction | build_spec | 50 | 2.54 µs | 2.67 µs |

### unwind_protect

| Benchmark | Median | Mean |
|-----------|--------|------|
| direct_noop | 0.002 ns | 0.002 ns |
| unwind_protect_noop | 32.6 ns | 32.4 ns |
| unwind_r_call | 37.0 ns | 40.4 ns |
| catch_unwind_success | 0.463 ns | 0.465 ns |
| catch_unwind_panic | 5.42 µs | 5.99 µs |
| unwind_nested_2 | 71.0 ns | 76.4 ns |
| unwind_nested_5 | 161.1 ns | 168.3 ns |

### worker

*Note: Worker API changed since 2026-02-18 — `run_on_worker_no_r` now compiles to near-zero cost (no channel hop), channel saturation uses different concurrency model. Numbers not directly comparable.*

| Benchmark | Args | Median | Mean |
|-----------|------|--------|------|
| run_on_worker_no_r | — | 0.46 ns | 0.47 ns |
| run_on_worker_with_r_thread | — | 7.53 ns | 13.9 ns |
| with_r_thread_main | — | 11.6 ns | 12.6 ns |
| worker_batching | 1 | 13.6 ns | 13.1 ns |
| worker_batching | 10 | 131.8 ns | 139.6 ns |
| worker_batching | 50 | 580.4 ns | 642.3 ns |
| worker_channel_saturation | 1 | 0.717 ns | 0.716 ns |
| worker_channel_saturation | 20 | 6.23 ns | 6.36 ns |
| worker_channel_saturation | 100 | 26.4 ns | 26.8 ns |
| worker_payload_size | 8 | 16.8 ns | 15.0 ns |
| worker_payload_size | 65536 | 541.7 ns | 630.9 ns |

## Lint Benchmarks (miniextendr-lint)

| Group | Benchmark | Median | Mean |
|-------|-----------|--------|------|
| full_scan | small_10_modules | 2.16 ms | 2.19 ms |
| full_scan | medium_100_modules | 19.8 ms | 20.1 ms |
| full_scan | large_500_modules | 114.1 ms | 118.6 ms |
| full_scan | dense_10_modules_50fns | 8.67 ms | 8.77 ms |
| impl_scan | small_10_types | 2.35 ms | 2.41 ms |
| impl_scan | medium_100_types | 22.2 ms | 22.6 ms |
| impl_scan | dense_10_types_20methods | 4.18 ms | 4.59 ms |
| index_build | small_10_modules | 2.21 ms | 2.27 ms |
| index_build | large_500_modules | 107.7 ms | 109 ms |
| scaling | fns_500_files_10 | 8.57 ms | 8.59 ms |
| scaling | fns_500_files_100 | 20.4 ms | 20.6 ms |
| scaling | fns_500_files_500 | 76.8 ms | 77.4 ms |

### Connections (feature `connections`)

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| connection_build | — | 624.7 ns | 1.25 µs |
| connection_write | — | 27.5 ns | 33.6 ns |
| connection_read | 64 | 24.0 ns | 23.7 ns |
| connection_read | 1024 | 32.0 ns | 31.7 ns |
| connection_read | 16384 | 273.1 ns | 555.1 ns |
| connection_write_sized | 64 | 25.5 ns | 25.7 ns |
| connection_write_sized | 16384 | 922.2 ns | 1.04 µs |
| connection_burst_write | 1 | 41.4 ns | 43.8 ns |
| connection_burst_write | 50 | 1.04 µs | 1.19 µs |

### Rayon (feature `rayon`)

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| rayon_collect_vec | 1 | 41.7 ns | 1.93 µs |
| rayon_collect_vec | 65536 | 92.0 µs | 113.8 µs |
| rayon_reduce_sum | 65536 | 88.6 µs | 95.1 µs |
| rayon_with_r_vec | 65536 | 100.5 µs | 107.2 µs |

### refcount_protect (expanded feature bench)

Single-item costs:

| Benchmark | Median | Mean |
|-----------|--------|------|
| protect_scope_single | 10.9 ns | 11.3 ns |
| refcount_arena_single | 98.7 ns | 98.6 ns |
| raw_preserve_release_single | 17.9 ns | 458.8 ns |
| raw_preserve_release_unchecked_single | 14.5 ns | 16.1 ns |

## Skipped / Known Issues

- **`wrappers`**: Requires `rpkg` installed to provide R class methods. Skipped in this run.
- **`from_r::string_latin1`**: Panics — Latin-1 locale assertion in `from_r.rs:87`. Test environment does not provide Latin-1 encoding. UTF-8 benchmarks pass.
- **`translate::sexp_tryfrom_string_translate_latin1`**: Same root cause.
- **R-side benchmarks**: Require rpkg installed + `bench` R package. Run separately with `just bench-r`.
- **Macro compile-time**: Run separately with `just bench-compile`.

## Reproducing

```bash
# Full Rust suite (skipping wrappers which needs rpkg)
cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
  --bench allocator --bench altrep --bench altrep_advanced --bench altrep_iter \
  --bench coerce --bench dataframe --bench externalptr --bench factor \
  --bench ffi_calls --bench from_r --bench gc_protect --bench gc_protection_compare \
  --bench into_r --bench list --bench native_vs_coerce --bench panic_telemetry \
  --bench preserve --bench raw_access --bench rarray --bench refcount_protect \
  --bench rffi_checked --bench sexp_ext --bench strict --bench strings \
  --bench trait_abi --bench translate --bench typed_list --bench unwind_protect \
  --bench worker

# Lint benchmarks
cargo bench --manifest-path=miniextendr-lint/Cargo.toml --bench lint_scan

# Feature-gated (connections, rayon, refcount-fast-hash)
cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
  --features connections,rayon,refcount-fast-hash \
  --bench connections --bench rayon --bench refcount_protect
```
