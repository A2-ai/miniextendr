# Plan: Vendor Examples as R Test Fixtures

Implement upstream crate examples as R test fixtures in rpkg, organized by optional
integration crate. Each item is a new Rust `#[miniextendr]` function in the corresponding
`rpkg/src/rust/*_adapter_tests.rs` file, plus an R test in `rpkg/tests/testthat/`.

## Summary of Upstream Examples

| Crate | `examples/` files | Doc examples | Existing rpkg fixtures |
|-------|-------------------|-------------|----------------------|
| nalgebra | 17 | 0 | 10 functions (roundtrip, norm, dot, transpose, trace) |
| tabled | 54 | 0 | 6 functions (from_vecs, simple, empty, many_columns, special_chars, single_cell) |
| indicatif | 21 | 0 | 4 functions (rterm_debug, factories, hidden_bar, short_bar) |
| bitflags | 5 | 4 | 9 functions (roundtrip, strict/truncate, contains, union, intersect, empty, all) |
| toml | 3 | 0 | 8 functions (roundtrip, pretty, type_name, is_table, keys, nested, array_of_tables, parse_invalid, mixed_types) |
| borsh | 1 (serde_json_value) | 0 | 6 functions (roundtrip_doubles, roundtrip_string, tuple_size, nested, invalid, option) |
| rust_decimal | 1 (rkyv-remote) | 0 | 6 functions (roundtrip, add, mul, round, scale, is_zero) |
| uuid | 0 | 14 | 7 functions (roundtrip, roundtrip_vec, new_v4, nil, max, version, is_nil) |
| url | 0 | 37 | 9 functions (roundtrip, scheme, host, path, roundtrip_vec, is_valid, query, fragment, port, full_components) |
| rand | 0 | 3 | 10 functions (uniform, normal, exponential, int, interrupt, worker, guard, with_rng, sampler methods) |
| log | 0 | 10 | 5 functions (info, warn, error, set_level, debug) |
| either | 0 | 11 | 7 functions (int_or_str, dbl_or_vec, make_left, make_right, is_left, is_right, nested, zero) |
| ndarray | 0 | 0 | ~40 functions (NdVec/NdMat/NdDynArr wrappers with full ops) |
| regex | 0 | 1 | 6 functions (is_match, find, find_all, replace_first, replace_all, split) |
| Others (no examples) | 0 | 0-5 | Covered by existing adapters |

---

## 1. nalgebra (HIGH PRIORITY -- 17 upstream examples, many untested patterns)

**Existing coverage**: Basic DVector/DMatrix roundtrip, norm, dot, transpose, trace.

**New fixtures from upstream examples**:

1. `nalgebra_linear_system` -- Solve Ax=b via LU decomposition (from `linear_system_resolution.rs`)
   - `fn nalgebra_solve_4x4(a_flat: Vec<f64>, b: Vec<f64>) -> Vec<f64>`
   - Takes a flattened 4x4 matrix and a 4-vector, returns the solution vector
   - Tests: verify `a * x == b`

2. `nalgebra_matrix_construction` -- Build matrices from rows, columns, slices (from `matrix_construction.rs`)
   - `fn nalgebra_from_row_slice(data: Vec<f64>, nrow: i32, ncol: i32) -> DMatrix<f64>`
   - `fn nalgebra_from_fn(nrow: i32, ncol: i32) -> DMatrix<f64>` (fill with i*ncol+j)

3. `nalgebra_svector_roundtrip` -- Static (stack-allocated) vectors (from `scalar_genericity.rs`)
   - `fn nalgebra_svector3_roundtrip(x: f64, y: f64, z: f64) -> Vec<f64>`
   - Tests SVector<f64, 3> conversion (not yet tested)

4. `nalgebra_transforms` -- Isometry, rotation, scaling (from `transform_vector_point.rs`, `transform_matrix4.rs`)
   - `fn nalgebra_rotate_point_2d(angle: f64, px: f64, py: f64) -> Vec<f64>`
   - `fn nalgebra_scale_matrix(factor: f64) -> DMatrix<f64>`

5. `nalgebra_reshape` -- Reshape matrix without copy (from `reshaping.rs`)
   - `fn nalgebra_reshape(m: DMatrix<f64>, new_nrow: i32, new_ncol: i32) -> DMatrix<f64>`

6. `nalgebra_determinant` -- Compute determinant (from `linear_system_resolution.rs`)
   - `fn nalgebra_determinant(m: DMatrix<f64>) -> f64`

7. `nalgebra_inverse` -- Matrix inverse
   - `fn nalgebra_inverse(m: DMatrix<f64>) -> Option<DMatrix<f64>>`

8. `nalgebra_eigenvalues` -- Real eigenvalues of symmetric matrix
   - `fn nalgebra_eigenvalues(m: DMatrix<f64>) -> Vec<f64>`

**R tests**: `test-feature-adapters.R`, nalgebra section (add ~8 new test_that blocks)

---

## 2. tabled (HIGH PRIORITY -- 54 upstream examples, minimal coverage)

**Existing coverage**: Only `table_from_vecs` helper. No Builder, no styling, no formatting.

**New fixtures from upstream examples**:

1. `tabled_builder` -- Builder API (from `builder.rs`)
   - `fn tabled_builder_demo(items: Vec<String>) -> String`
   - Build a table row-by-row with `Builder::default()`, insert header, apply Style::markdown

2. `tabled_derive_struct` -- Tabled derive on a struct (from `table.rs`)
   - Define a `#[derive(Tabled)]` struct, create instances, format to string
   - `fn tabled_struct_table() -> String`

3. `tabled_styled` -- Different table styles (from `style_modern_rounded.rs`, `format.rs`)
   - `fn tabled_styled(style: &str) -> String` -- switch on style name: "ascii", "modern", "psql", "markdown", "rounded"
   - Uses `table_to_string_styled` helper

4. `tabled_column_width` -- Column width control (from `table_width.rs`, `table_width_2.rs`)
   - `fn tabled_with_max_width(max: i32) -> String`
   - Uses `Width::truncate(max)` or `Width::wrap(max)`

5. `tabled_alignment` -- Cell alignment (from `format.rs`)
   - `fn tabled_aligned() -> String`
   - Apply `Alignment::center()` to header row

6. `tabled_compact_table` -- CompactTable for simple output (from `compact_table.rs`)
   - `fn tabled_compact(data: Vec<Vec<String>>) -> String`

7. `tabled_span` -- Column/row spanning (from `span.rs`, `span_column.rs`)
   - `fn tabled_column_span() -> String`

8. `tabled_concat` -- Concatenate tables (from `concat.rs`)
   - `fn tabled_concat_horizontal() -> String`

**R tests**: `test-feature-adapters.R`, tabled section (add ~8 new test_that blocks)

---

## 3. indicatif (MEDIUM -- 21 upstream examples, but R console model limits patterns)

**Existing coverage**: RTerm construction, factory functions, hidden bar, short bar.

**New fixtures from upstream examples**:

1. `indicatif_spinner` -- Spinner (non-length) progress (from `long-spinner.rs`)
   - `fn indicatif_spinner_demo() -> String` -- create spinner, tick a few times, finish

2. `indicatif_custom_style` -- Custom template (from `download.rs`)
   - `fn indicatif_download_style(total: i32) -> String`
   - Set custom template with `{bytes}/{total_bytes}` style

3. `indicatif_message_update` -- Progress with dynamic messages (from `message.rs`)
   - `fn indicatif_with_messages(steps: Vec<String>) -> String`
   - Update message at each step

4. `indicatif_elapsed` -- Progress bar with elapsed time display (from `steady.rs`)
   - `fn indicatif_elapsed_demo() -> String`

Note: Multi-progress (`multi.rs`) requires threads, which is complex in R context.
Iterator wrapping (`iterator.rs`) uses `.progress()` which needs `TermLike` integration.
These are lower priority.

**R tests**: `test-feature-adapters.R`, indicatif section (add ~4 new test_that blocks)

---

## 4. bitflags (MEDIUM -- 5 upstream examples)

**Existing coverage**: Good coverage of RFlags, strict/truncate, contains, union, intersect, empty, all.

**New fixtures from upstream examples**:

1. `bitflags_display` -- Display/Debug formatting (from `fmt.rs`)
   - `fn bitflags_display(flags: RFlags<Perms>) -> String`
   - `fn bitflags_parse(s: &str) -> Option<RFlags<Perms>>` -- FromStr support

2. `bitflags_symmetric_difference` -- XOR operation (from `custom_bits_type.rs` concepts)
   - `fn bitflags_xor(a: RFlags<Perms>, b: RFlags<Perms>) -> RFlags<Perms>`

3. `bitflags_complement` -- NOT operation
   - `fn bitflags_complement(flags: RFlags<Perms>) -> RFlags<Perms>`

4. `bitflags_iter_names` -- Iterate over set flag names
   - `fn bitflags_names(flags: RFlags<Perms>) -> Vec<String>`

**R tests**: `test-feature-adapters.R`, bitflags section (add ~4 new test_that blocks)

---

## 5. toml (LOW -- 3 upstream examples, good existing coverage)

**Existing coverage**: roundtrip, pretty, type_name, is_table, keys, nested, array_of_tables, parse_invalid, mixed_types.

**New fixtures from upstream examples**:

1. `toml_decode_struct` -- Decode TOML into a typed structure (from `decode.rs`)
   - `fn toml_decode_config(input: String) -> Vec<String>` -- parse a config TOML, extract fields
   - Demonstrates serde + TOML interplay

2. `toml_enum_external` -- Enum variants in TOML (from `enum_external.rs`)
   - `fn toml_decode_enum(input: String) -> String` -- parse TOML with enum variants

3. `toml_to_json` -- TOML to JSON conversion (from `toml2json.rs`)
   - `fn toml_to_json(input: String) -> String` -- parse TOML, convert to JSON string
   - Requires serde_json feature

**R tests**: `test-feature-adapters.R`, toml section (add ~3 new test_that blocks)

---

## 6. borsh (LOW -- 1 upstream example, good coverage)

**Existing coverage**: Roundtrip doubles, strings, tuples, nested, invalid, option.

**New fixtures from upstream examples**:

1. `borsh_serde_json_value` -- Serialize serde_json::Value via Borsh (from `serde_json_value.rs`)
   - `fn borsh_json_value_roundtrip() -> bool` -- create a complex JSON value, serialize via Borsh, deserialize, verify equality
   - This is an advanced pattern showing interop between borsh and serde_json
   - Requires both `borsh` and `serde_json` features

2. `borsh_hashmap` -- HashMap serialization
   - `fn borsh_hashmap_roundtrip() -> bool` -- serialize/deserialize HashMap<String, i32>

**R tests**: `test-feature-adapters.R`, borsh section (add ~2 new test_that blocks)

---

## 7. nalgebra additional: SMatrix (static matrix)

**Not tested at all**: `SVector` and `SMatrix` types are exported but have no rpkg fixtures.

1. `nalgebra_smatrix_roundtrip` -- Static 3x3 matrix roundtrip
   - `fn nalgebra_smatrix3_new(data: Vec<f64>) -> DMatrix<f64>` (create SMatrix, return as DMatrix)
   - Tests the SMatrix conversion path

---

## 8. arrow (MEDIUM -- no upstream examples, but untested conversion paths)

**Existing coverage**: Float64Array, Int32Array, UInt8Array, BooleanArray, StringArray, RecordBatch roundtrips, nulls, sums, date, dictionary, timestamp.

**New fixtures for untested API surface**:

1. `arrow_zero_copy_verify` -- Verify zero-copy path
   - `fn arrow_is_zero_copy(v: Float64Array) -> bool` -- check `r_source().is_some()`
   - Tests `RSourced` trait

2. `arrow_posixct_to_timestamp` -- POSIXct to Arrow TimestampSecondArray
   - Already has `arrow_posixct_roundtrip` but could test more edge cases (epoch, negative timestamps)

3. `arrow_filter_nulls` -- Filter operation preserving null bitmap
   - `fn arrow_filter_non_null(v: Float64Array) -> Float64Array`

**R tests**: `test-arrow.R` (add ~3 new test_that blocks)

---

## 9. datafusion (MEDIUM -- no upstream examples, but untested API)

**Existing coverage**: SQL query, select, sort+limit, columns, chain, aggregate, global_agg, count.

**New fixtures from DataFusion capabilities**:

1. `datafusion_join` -- JOIN two tables
   - `fn test_df_join(left: RecordBatch, right: RecordBatch, sql: &str) -> RecordBatch`
   - Register two tables, run JOIN SQL

2. `datafusion_window` -- Window functions
   - `fn test_df_window(df: RecordBatch) -> RecordBatch`
   - SQL with `ROW_NUMBER() OVER (ORDER BY x)` or `SUM(y) OVER (PARTITION BY name)`

3. `datafusion_subquery` -- Subqueries
   - `fn test_df_subquery(df: RecordBatch) -> RecordBatch`
   - SQL with `WHERE x IN (SELECT ...)`

**R tests**: `test-datafusion.R` (add ~3 new test_that blocks)

---

## 10. rand (LOW -- good coverage, minor gaps)

**Existing coverage**: uniform, normal, exponential, int, interrupt, worker, guard, with_rng, sampler.

**New fixtures from doc examples**:

1. `rand_rrng_trait` -- Test RRng with rand crate traits
   - `fn rand_rrng_bool() -> bool` -- generate a random bool via RRng
   - `fn rand_rrng_range(min: f64, max: f64) -> f64` -- gen_range via RRng

2. `rand_distributions` -- R's native distributions via RDistributions trait
   - `fn rand_rdist_chi_sq(n: i32, df: f64) -> Vec<f64>` -- chi-squared
   - `fn rand_rdist_gamma(n: i32, shape: f64, scale: f64) -> Vec<f64>`

**R tests**: `test-rng.R` (add ~3 new test_that blocks)

---

## 11. serde_json (LOW -- good coverage, one gap)

**Existing coverage**: JsonValue roundtrip, type_name, is_object, object_keys, serialize_point, pretty.

**New fixtures**:

1. `json_from_sexp_options` -- Test JsonOptions (NaHandling, SpecialFloatHandling)
   - `fn json_with_na_null(sexp: SEXP) -> String` -- NaHandling::AsNull
   - `fn json_with_na_error(sexp: SEXP) -> String` -- NaHandling::Error
   - Tests the `json_from_sexp_with` path with different option combinations

2. `json_factor_handling` -- FactorHandling variants
   - `fn json_factor_as_int(sexp: SEXP) -> String` -- FactorHandling::AsInteger
   - `fn json_factor_as_string(sexp: SEXP) -> String` -- FactorHandling::AsString

**R tests**: `test-feature-adapters.R`, serde_json section (add ~3 new test_that blocks)

---

## 12. regex (LOW -- good coverage, one pattern missing)

**Existing coverage**: is_match, find, find_all, replace_first, replace_all, split.

**New fixtures**:

1. `regex_capture_groups` -- CaptureGroups from `regex_impl`
   - `fn regex_captures(pattern: &str, text: &str) -> Vec<String>` -- extract named/numbered capture groups
   - Uses `RCaptureGroups` trait

**R tests**: `test-feature-adapters.R`, regex section (add ~1 new test_that block)

---

## 13. Remaining crates with no upstream examples (already well covered)

These crates have **no** `examples/` directory or `[[example]]` entries, and existing
rpkg adapter tests already cover their core API. No additional fixtures needed:

- **num-bigint**: 5 functions (roundtrip, add, mul, factorial, is_positive) -- adequate
- **rust_decimal**: 6 functions (roundtrip, add, mul, round, scale, is_zero) -- adequate
- **ordered-float**: 9 functions (roundtrip, sort, special values) -- comprehensive
- **num-complex**: 9 functions (roundtrip, add, norm, conj, re, im, vec, is_finite, from_parts) -- comprehensive
- **num-traits**: 11 functions (is_zero, is_one, abs, signum, positive, negative, floor, ceil, sqrt, is_finite, is_nan, powi) -- comprehensive
- **indexmap**: 8 functions (roundtrip_int/str/dbl, keys, len, empty, duplicate_key, order_preserved, single) -- comprehensive
- **bytes**: 8 functions (roundtrip, len, mut_roundtrip, concat, slice, empty, large, all_values) -- comprehensive
- **bitvec**: 9 functions (roundtrip, ones, zeros, from_vec, to_vec, len, empty, all_ones, all_zeros, toggle) -- comprehensive
- **aho-corasick**: 8 functions (is_match, count, find_flat, replace, no_match, overlapping, unicode, replace_empty) -- comprehensive
- **tinyvec**: 8 functions (roundtrip_int/dbl, len, arrayvec_roundtrip_int/dbl, empty, at_capacity, over_capacity, arrayvec_empty) -- comprehensive
- **sha2**: 7 functions (sha256, sha512, lengths, known_hash, large, binary_content, different_inputs) -- comprehensive
- **url**: 10 functions (roundtrip, scheme, host, path, vec, is_valid, query, fragment, port, full_components) -- comprehensive
- **uuid**: 7 functions (roundtrip, vec, new_v4, nil, max, version, is_nil) -- comprehensive
- **either**: 8 functions (int_or_str, dbl_or_vec, make_left/right, is_left/right, nested, zero) -- comprehensive
- **log**: 5 functions (info, warn, error, set_level, debug) -- adequate
- **rayon**: 15 functions (sum, sqrt, filter, vec_collect, with_r_vec, with_r_vec_map, par_map/2/3, with_r_matrix, with_r_array, r_parallel_iterator, par_collect_sexp, dataframe_rayon) -- comprehensive

---

## Implementation Order (prioritized)

1. nalgebra: linear system solve, determinant, inverse, eigenvalues, SVector/SMatrix, matrix construction, transforms, reshape (~8 Rust fns, ~8 R tests)
2. tabled: builder, derive struct, styled, width, alignment, compact, span, concat (~8 Rust fns, ~8 R tests)
3. indicatif: spinner, custom style, messages, elapsed (~4 Rust fns, ~4 R tests)
4. bitflags: display, parse, xor, complement, iter_names (~5 Rust fns, ~4 R tests)
5. arrow: zero-copy verify, filter_nulls (~2 Rust fns, ~2 R tests)
6. datafusion: join, window, subquery (~3 Rust fns, ~3 R tests)
7. toml: decode_struct, enum, to_json (~3 Rust fns, ~3 R tests)
8. serde_json: options variants, factor handling (~4 Rust fns, ~3 R tests)
9. regex: capture groups (~1 Rust fn, ~1 R test)
10. rand: rrng bool/range, native distributions (~4 Rust fns, ~3 R tests)
11. borsh: json_value roundtrip, hashmap (~2 Rust fns, ~2 R tests)

**Total**: ~44 new Rust functions, ~41 new R test blocks.

---

## File Changes

**New/modified Rust files** (in `rpkg/src/rust/`):
- `nalgebra_adapter_tests.rs` -- add ~8 new functions
- `tabled_adapter_tests.rs` -- add ~8 new functions
- `indicatif_adapter_tests.rs` -- add ~4 new functions
- `bitflags_adapter_tests.rs` -- add ~5 new functions
- `arrow_adapter_tests.rs` -- add ~2 new functions
- `datafusion_tests.rs` -- add ~3 new functions
- `toml_adapter_tests.rs` -- add ~3 new functions
- `serde_json_adapter_tests.rs` -- add ~4 new functions
- `regex_adapter_tests.rs` -- add ~1 new function
- `rng_tests.rs` -- add ~4 new functions
- `borsh_adapter_tests.rs` -- add ~2 new functions

**New/modified R test files** (in `rpkg/tests/testthat/`):
- `test-feature-adapters.R` -- add sections for bitflags, tabled, toml, serde_json, regex, borsh, indicatif
- `test-ndarray.R` -- no changes needed (ndarray has separate comprehensive tests)
- `test-arrow.R` -- add ~2 new test_that blocks
- `test-datafusion.R` -- add ~3 new test_that blocks
- `test-rng.R` -- add ~3 new test_that blocks

**Build steps after implementation**:
```
just configure
just rcmdinstall
just devtools-document
just devtools-test
```
