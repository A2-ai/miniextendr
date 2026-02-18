# Benchmark Results — 2026-02-18

## Environment

| Field | Value |
|-------|-------|
| **Date** | 2026-02-18 |
| **Commit** | `d479886` (branch: `main`) |
| **Rust** | rustc 1.93.0 (254b59607 2026-01-19) |
| **OS** | macOS 15.3 (Darwin 25.2.0) arm64 |
| **CPU** | Apple M3 Max |
| **RAM** | 36 GB |

## Summary

29 benchmark binaries executed across the main suite + lint benchmarks.
1 bench skipped: `wrappers` (needs rpkg installed). Connections benchmarks now pass after fixing open-state and GC protection.

### Key Findings

1. **Worker thread round-trip**: ~4-5 us per `run_on_worker` call; `with_r_thread` from main thread is nearly free (10 ns).
2. **Unwind protection**: `with_r_unwind_protect` adds ~31-35 ns overhead over direct call. Nesting 5 layers costs ~170 ns. `catch_unwind` success path is 0.5 ns; panic path is ~5 us.
3. **ALTREP vs plain**: Element access 200 ns (ALTREP) vs 9 ns (plain INTSXP) — ~22x overhead per element. DATAPTR materialization at 64K elements: 16-19 us (ALTREP) vs 9 ns (plain).
4. **Guard modes**: `unsafe` guard ≈ `default` (rust_unwind) at 64K elements (15-16 ms full scan). `r_unwind` guard adds ~25% overhead (20 ms). Plain INTSXP: 258 us.
5. **Trait ABI dispatch**: vtable query ~1 ns (cache-hot). Full dispatch through view: ~53-62 ns. Multi-method (5-method) vtable query same cost as 2-method (~1 ns). Multi-method dispatch all-4: ~417 ns.
6. **ExternalPtr creation**: Small (8B) ~65 ns vs Box ~12 ns (~5x). Large (64KB) ~727 ns vs Box ~512 ns (~1.4x). Overhead decreases with payload size.
7. **Allocator**: R allocator 60-90 ns for small allocs vs system 10-18 ns (~5x slower). Gap narrows at 64KB.
8. **Strict mode**: Negligible overhead for scalar conversions (~42-45 ns vs ~40-43 ns normal). Vec<i64> at 10K: strict 7.9 us vs normal 7.1 us (~11% overhead).
9. **String ALTREP**: Materialization (DATAPTR_RO force) at 64K strings: 6.6 ms. Element access without materialization: 2.6 ms (same as creating). Vs plain STRSXP element: 4.5 ms.
10. **Typed list validation**: 3 fields: 660 ns. 10 fields: 2 us. 50 fields: 12 us. Linear scaling.
11. **Panic telemetry**: RwLock read (no hook) ~1.5 ns. Fire with hook: ~65 ns. Negligible hot-path cost.
12. **Vector scaling (1M elements)**: i32 into_sexp: 675 us. f64: 1.6 ms. String: 276 ms (dominated by CHARSXP allocation). Option<i32> 50% NA: 934 us.

## Rust-Side Results (divan)

### allocator

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| rallocator_alloc | 8 | 71 ns | 70 ns |
| rallocator_alloc | 1024 | 155 ns | 162 ns |
| rallocator_alloc | 65536 | 867 ns | 2.9 us |
| system_alloc | 8 | 16 ns | 14 ns |
| system_alloc | 1024 | 22 ns | 22 ns |
| system_alloc | 65536 | 521 ns | 534 ns |

### altrep (basic)

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| altrep_int_dataptr | 0 (1) | 167 ns | 278 ns |
| altrep_int_dataptr | 4 (64K) | 15.1 us | 16.1 us |
| altrep_int_elt | 0 (1) | 211 ns | 210 ns |
| altrep_int_elt | 4 (64K) | 16.2 us | 17.2 us |
| plain_int_dataptr | any | 9.9 ns | 10 ns |
| plain_int_elt | any | 9.2 ns | 9.2 ns |

### altrep_advanced

| Group | Benchmark | Size Idx | Median | Mean |
|-------|-----------|----------|--------|------|
| complex_altrep | dataptr | 0 | 166 ns | 357 ns |
| complex_altrep | dataptr | 4 | 65.1 us | 66.5 us |
| complex_altrep | full_scan_elt | 4 | 5.1 ms | 5.1 ms |
| creation | altrep_int | 4 | 15.8 us | 16.9 us |
| creation | altrep_string | 4 | 2.6 ms | 2.6 ms |
| creation | constant_real | 4 | 232 ns | 228 ns |
| guard_modes | unsafe | 4 | 15.7 ms | 15.8 ms |
| guard_modes | default | 4 | 16.2 ms | 16.5 ms |
| guard_modes | r_unwind | 4 | 20.2 ms | 20.2 ms |
| guard_modes | plain_intsxp | 4 | 258 us | 259 us |
| materialization | altrep_dataptr_ro | 4 | 18.6 us | 19.6 us |
| materialization | plain_dataptr_ro | any | 8.9 ns | 8.9 ns |
| string_altrep | create | 4 | 2.6 ms | 2.6 ms |
| string_altrep | elt | 4 | 2.6 ms | 2.6 ms |
| string_altrep | elt_with_na | 4 | 2.4 ms | 2.4 ms |
| string_altrep | force_materialize | 4 | 6.6 ms | 6.8 ms |
| zero_alloc | constant_elt | any | ~510 ns | ~563 ns |
| zero_alloc | constant_full_scan | 4 | 16.3 ms | 16.4 ms |

### coerce

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| vec_int_direct_slice | 4 | 8.9 ns | 9.0 ns |
| vec_int_to_float_r_coerce | 4 | 32.3 us | 35.2 us |
| vec_float_to_int_rust_coerce | 4 | 11.7 us | 12.1 us |
| vec_lgl_direct | 4 | 8.9 ns | 8.9 ns |
| vec_lgl_to_int_r_coerce | 4 | 33.3 us | 33.3 us |

### dataframe

| Benchmark | Rows | Median | Mean |
|-----------|------|--------|------|
| transpose/point3_3col | 100 | 26.2 us | 27.3 us |
| transpose/point3_3col | 100000 | 14.9 ms | 15.2 ms |
| transpose/wide10_10col | 100 | 44.2 us | 44.4 us |
| transpose/wide10_10col | 100000 | 26.9 ms | 27.3 ms |
| full_pipeline/mixed_5col | 100 | 34.1 us | 34.3 us |
| full_pipeline/event_2var_enum | 100 | 42.3 us | 42.3 us |

### externalptr

| Benchmark | Median | Mean |
|-----------|--------|------|
| create_small_payload (8B) | 65 ns | 67 ns |
| create_medium_payload (1KB) | 77 ns | 83 ns |
| create_large_payload (64KB) | 727 ns | 747 ns |
| baseline_box_small | 12 ns | 12 ns |
| baseline_box_large | 512 ns | 528 ns |
| access_as_ref | 0.5 ns | 0.5 ns |
| access_deref | 0.5 ns | 0.5 ns |
| erased_is_hit | 5.5 ns | 5.7 ns |
| erased_is_miss | 4.2 ns | 4.3 ns |
| erased_downcast_ref_hit | 5.6 ns | 5.8 ns |
| create_sized_payload(8) | 64 ns | 69 ns |
| create_sized_payload(4096) | 85 ns | 93 ns |
| create_sized_payload(65536) | 780 ns | 787 ns |
| create_n_ptrs(100) | 6.6 us | 6.9 us |
| create_n_ptrs(1000) | 62 us | 66 us |

### ffi_calls

| Benchmark | Median | Mean |
|-----------|--------|------|
| rf_scalar_integer | 14 ns | 15 ns |
| rf_scalar_real | 14 ns | 14 ns |
| rf_alloc_vector_int_1000 | 114 ns | 126 ns |
| rf_protect_unprotect | 5.2 ns | 5.4 ns |
| rf_install_cached | 2.1 ns | 2.1 ns |

### from_r

| Benchmark | Size Idx | Median | Mean |
|-----------|----------|--------|------|
| scalar_i32 | - | 3.1 ns | 3.1 ns |
| scalar_f64 | - | 3.0 ns | 3.1 ns |
| slice_i32 | 4 (64K) | 8.0 ns | 8.3 ns |
| hashset_i32 | 2 (4K) | 68 us | 71 us |
| btreeset_i32 | 2 (4K) | 117 us | 118 us |

### gc_protect

| Benchmark | Median | Mean |
|-----------|--------|------|
| owned_protect_create_drop | 9.2 ns | 9.7 ns |
| scope_protect_single | 8.9 ns | 9.6 ns |
| manual_protect_unprotect | 5.3 ns | 5.4 ns |
| owned_protect_100_sequential | 854 ns | 925 ns |
| scope_protect_100_batch | 816 ns | 829 ns |
| manual_protect_100_sequential | 508 ns | 547 ns |

### into_r

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| scalar_i32 | - | 12.5 ns | 12.6 ns |
| scalar_f64 | - | 12.5 ns | 12.5 ns |
| vec_i32 | 64K | 13.3 us | 14.5 us |
| vec_f64 | 64K | 22.5 us | 22.7 us |
| vec_string | 64K | 3.95 ms | 3.96 ms |
| vec_option_i32_50pct_na | 64K | 14.1 us | 14.9 us |
| **scale_vec_i32** | **1M** | **675 us** | **698 us** |
| **scale_vec_f64** | **1M** | **1.6 ms** | **1.7 ms** |
| **scale_vec_string** | **1M** | **276 ms** | **277 ms** |
| **scale_vec_option_i32_50pct_na** | **1M** | **934 us** | **954 us** |

### panic_telemetry

| Benchmark | Median | Mean |
|-----------|--------|------|
| read_lock_no_hook | 1.6 ns | 1.6 ns |
| read_lock_with_hook | 1.5 ns | 1.5 ns |
| fire_with_hook | 65 ns | 67 ns |

### raw_access

| Group | Benchmark | Size | Median | Mean |
|-------|-----------|------|--------|------|
| integer | safe_r_slice | 64K | 9.4 ns | 9.6 ns |
| integer | unchecked_slice | 64K | 9.2 ns | 9.5 ns |
| integer | raw_ffi_pointer | 64K | 9.4 ns | 9.3 ns |
| real | safe_r_slice | 64K | 9.4 ns | 9.5 ns |
| real | raw_ffi_pointer | 64K | 9.2 ns | 9.5 ns |
| conversion | try_from_sexp_i32 | 64K | 9.1 ns | 9.2 ns |
| conversion | manual_ptr_i32 | 64K | 8.9 ns | 9.1 ns |

### strict

| Group | Benchmark | Size | Median | Mean |
|-------|-----------|------|--------|------|
| scalar_input | normal_i64 | - | 3.2 ns | 3.3 ns |
| scalar_input | strict_i64 | - | 3.2 ns | 3.5 ns |
| vec_input | normal_vec_i64 | 10K | 23.3 us | 24.2 us |
| vec_input | strict_vec_i64 | 10K | 24.5 us | 24.7 us |
| vec_output | normal_vec_i64 | 10K | 7.9 us | 8.2 us |
| vec_output | strict_vec_i64 | 10K | 7.9 us | 32.7 us* |

*outlier in strict_vec_i64 output (mean skewed by one 2.5 ms spike)

### strings

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| from_r_cstr (UTF-8) | 64K chars | 2.3 us | 2.3 us |
| into_r_str_mkcharlen | 64K chars | 60.4 us | 60.4 us |
| tryfromsexp_str (UTF-8) | 64K chars | 1.2 us | 1.3 us |

### trait_abi

| Benchmark | Median | Mean |
|-----------|--------|------|
| mx_query_vtable | 1.0 ns | 1.0 ns |
| mx_query_multi_method_vtable | 1.0 ns | 1.0 ns |
| query_vtable_miss | 1.0 ns | 1.0 ns |
| view_value_only | 55 ns | 60 ns |
| dispatch_self_value | 62 ns | 66 ns |
| dispatch_self_mut_increment | 32 ns | 32 ns |
| dispatch_multi_method_all | 417 ns | 865 ns |
| dispatch_multi_method_hot (10x) | 1.1 us | 1.4 us |
| dispatch_repeated_hot (10x) | 580 ns | 684 ns |

### typed_list

| Group | Benchmark | Fields | Median | Mean |
|-------|-----------|--------|--------|------|
| numeric_validation | validate_pass | 3 | 659 ns | 690 ns |
| numeric_validation | validate_pass | 50 | 12.4 us | 13.0 us |
| numeric_validation | validate_strict_pass | 50 | 12.9 us | 13.3 us |
| failure_paths | wrong_type_first_field | 3 | 640 ns | 688 ns |
| failure_paths | missing_required_field | 50 | 13.3 us | 14.0 us |
| spec_construction | build_spec | 50 | 2.7 us | 2.8 us |

### unwind_protect

| Benchmark | Median | Mean |
|-----------|--------|------|
| direct_noop | 0 ns | 0 ns |
| unwind_protect_noop | 31 ns | 29 ns |
| unwind_r_call | 39 ns | 42 ns |
| catch_unwind_success | 0.5 ns | 0.5 ns |
| catch_unwind_panic | 5.3 us | 6.2 us |
| unwind_nested_2 | 83 ns | 202 ns |
| unwind_nested_5 | 170 ns | 190 ns |

### worker

| Benchmark | Args | Median | Mean |
|-----------|------|--------|------|
| run_on_worker_no_r | - | 5.0 us | 7.6 us |
| run_on_worker_with_r_thread | - | 7.4 us | 9.4 us |
| with_r_thread_main | - | 10 ns | 18 ns |
| worker_channel_saturation | 1 | 4.0 us | 5.1 us |
| worker_channel_saturation | 20 | 64 us | 70 us |
| worker_channel_saturation | 100 | 387 us | 392 us |
| worker_batching | 1 | 13.6 us | 12.9 us |
| worker_batching | 10 | 72 us | 73 us |
| worker_batching | 50 | 326 us | 327 us |
| worker_payload_size | 8 | 4.3 us | 4.7 us |
| worker_payload_size | 65536 | 4.9 us | 7.1 us |

## Lint Benchmarks (miniextendr-lint)

| Group | Benchmark | Median | Mean |
|-------|-----------|--------|------|
| full_scan | small_10_modules | 1.8 ms | 1.8 ms |
| full_scan | medium_100_modules | 15.6 ms | 15.6 ms |
| full_scan | large_500_modules | 82.1 ms | 84.6 ms |
| full_scan | dense_10_modules_50fns | 5.9 ms | 5.9 ms |
| impl_scan | small_10_types | 1.8 ms | 1.9 ms |
| impl_scan | medium_100_types | 16.1 ms | 16.4 ms |
| index_build | small_10_modules | 1.9 ms | 1.9 ms |
| index_build | large_500_modules | 79.4 ms | 79.9 ms |
| scaling | fns_500_files_10 | 5.8 ms | 5.8 ms |
| scaling | fns_500_files_100 | 16.1 ms | 16.8 ms |
| scaling | fns_500_files_500 | 64.1 ms | 64.5 ms |

### connections (feature-gated)

| Benchmark | Size | Median | Mean |
|-----------|------|--------|------|
| connection_build | — | 542 ns | 1.2 us |
| connection_write | 128 B | 29 ns | 33 ns |
| connection_read | 64 | 21 ns | 22 ns |
| connection_read | 256 | 22 ns | 22 ns |
| connection_read | 1024 | 22 ns | 26 ns |
| connection_read | 4096 | 86 ns | 129 ns |
| connection_read | 16384 | 1.2 us | 1.3 us |
| connection_write_sized | 64 | 24 ns | 24 ns |
| connection_write_sized | 256 | 42 ns | 64 ns |
| connection_write_sized | 1024 | 85 ns | 99 ns |
| connection_write_sized | 4096 | 319 ns | 369 ns |
| connection_write_sized | 16384 | 1.0 us | 1.2 us |
| connection_burst_write | 1 | 39 ns | 49 ns |
| connection_burst_write | 10 | 245 ns | 368 ns |
| connection_burst_write | 50 | 1.1 us | 1.2 us |

## Skipped / Known Issues

- **wrappers**: Requires `rpkg` installed to provide R class methods. Skipped in this run.
- **R-side benchmarks** (D1-D4): Require rpkg installed + `bench` R package. Run separately with `just bench-r`.
- **Macro compile-time** (B3): Run separately with `just bench-compile` (uses synthetic crates).

## Reproducing

```bash
# Full Rust suite
cargo bench --manifest-path=miniextendr-bench/Cargo.toml

# Lint benchmarks
cargo bench --manifest-path=miniextendr-lint/Cargo.toml --bench lint_scan

# Feature-gated (connections)
cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features connections --bench connections

# Save structured baseline
just bench-save
```
