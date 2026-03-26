# Plan: Fix All Audit Issues

## Critical (9) — fix now

1. expression.rs: protect msg_sexp in get_r_error_message()
2. columnar.rs: -(nrow as i32) → checked i32 conversion
3. connection.rs: debug_assert → runtime assert in get_state()
4. ndarray_impl.rs: .product() → checked_mul chain
5. nalgebra_impl.rs: argmin/argmax skip NaN
6. factor.rs: bounds check in level()
7. cargo-revendor strip.rs: fix TOML section prefix matching
8. cargo-revendor vendor.rs: normalize Windows paths in freeze
9. cargo-revendor Cargo.toml: verify edition (2024 = Rust 1.85+, intentional)

## Medium (11) — fix now

10. Arrow buffer guard: document allocation failure behavior
11. Arrow RecordBatch: add column length pre-validation
12. DataFusion runtime: return Result instead of expect
13. toml_impl: protect integer/float array allocations
14. serde_impl: replace .expect() with .map_err() for user data
15. encoding.rs: add is_r_main_thread() assert
16. DataFrame transpose: fix GC protection in error path
17. bytes_impl: check remaining_mut() overflow
18. indexmap_impl: migrate to OwnedProtect
19. GC_PROTECT.md: add ProtectPool section
20. Optionals as-casts: document 15 lossy casts

## Test coverage — defer to separate session
