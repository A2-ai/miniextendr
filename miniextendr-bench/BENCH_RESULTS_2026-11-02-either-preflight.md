# Native-borrow preflight that follows an `Either`'s arm (#1504)

Date: 2026-11-02
Before: the parent of the commit that adds this file: main at `87bd9c1f6` plus the commit that adds the benchmark fixtures (`bench(native-borrow): add preflight benchmarks for #1504`). On main an `Either` argument declared no native borrow, so the alias check skipped it.
After: the commit that adds this file (#1504). An `Either` borrows the union of its arms; when that union meets another argument, the wrapper prepares the `Either` to learn its arm and checks again.

**Question.** What does the new check cost on the success path: for calls without an `Either` (plain, borrowed vectors, borrowed strings) and for calls with one, on each arm?

**Answer.** Calls without an `Either` cost what they did, within the noise. At the wrapper level, an `Either` next to a borrowed vector costs 50 to 70 ns more, because the check now runs for it (it was skipped, which let an alias through). An `Either` alone costs about 5 ns more, for the check that its two arms cannot alias. At the R call level (about 1 µs per call), all of these are within 2% to 8%.

## Environment and method

| | |
|---|---|
| Machine | Apple M3 Max, 14 cores, 36 GB, macOS 26.6.2 |
| R | 4.6.1, `bench` 1.1.4, the rv-managed project library |
| Rust | rustc 1.99.0; rpkg release profile, divan `bench` profile |
| Load (1-min average) | about 2 during both runs; other sessions were active |

- **Fixtures.** The same signatures in two places:
  - `rpkg/src/rust/native_borrow_bench_fixtures.rs` (`nbb_*`, `noexport`), measured through their R wrappers;
  - `miniextendr-bench/benches/native_borrow.rs`, which calls the generated C wrappers directly (no R wrapper, no `.Call()`).
- **Cases.**
  - `plain`: `x: i32`.
  - `slice`: `&[f64]`.
  - `slices`: `&[f64], &mut [f64], &[i32]`, so the alias check runs.
  - `str`: `&str`.
  - `strs`: `&str, &str, Vec<&str>`.
  - `either_left` / `either_right`: `Either<i32, &mut [f64]>` next to a `&[f64]` of another vector, on each arm.
  - `either_alone_left` / `either_alone_right`: `Either<i32, &[f64]>` alone.
  - `either_left_aliased`: `Either<f64, &mut [f64]>` and `&[f64]` given one double scalar. The owned left arm takes it, so the call is valid: the check prepares the `Either` and checks again.
- **Alternation.** The before and after builds alternate, one rep at a time, each in its own process, 7 reps each.
- **R level.** `bench::mark` with `min_time = 0.5 s`, after one warm-up call. Each rep's value is the 5%-trimmed mean of its iterations (the macOS timer's 41.7 ns tick quantises a median). The table shows the median of the 7 reps, then their range and CV.
- **Wrapper level.** divan with 100 samples of 2000 iterations. Each rep's value is divan's median. The table shows the median of the 7 reps, then their range and CV.

## R call level

| case | before ns | after ns | after / before |
|---|---:|---:|---:|
| `plain` | 1267 (1236 to 1305, CV 2.1%) | 1292 (1255 to 1306, CV 1.4%) | 1.020 |
| `slice` | 945 (814 to 1022, CV 8.4%) | 983 (901 to 995, CV 3.5%) | 1.041 |
| `slices` | 1791 (1775 to 1819, CV 0.8%) | 1812 (1631 to 1828, CV 3.9%) | 1.012 |
| `str` | 1288 (1236 to 1329, CV 2.3%) | 1308 (1275 to 1322, CV 1.4%) | 1.016 |
| `strs` | 2602 (2388 to 2631, CV 3.3%) | 2634 (2609 to 2696, CV 1.1%) | 1.012 |
| `either_left` | 1070 (1051 to 1102, CV 1.5%) | 1131 (1106 to 1144, CV 1.2%) | 1.058 |
| `either_right` | 1079 (951 to 1098, CV 4.9%) | 1149 (1066 to 1166, CV 3.0%) | 1.065 |
| `either_alone_left` | 638 (435 to 660, CV 13.0%) | 654 (643 to 658, CV 0.8%) | 1.024 |
| `either_alone_right` | 653 (646 to 671, CV 1.2%) | 664 (652 to 675, CV 1.2%) | 1.017 |
| `either_left_aliased` | 1082 (1027 to 1107, CV 2.6%) | 1164 (1023 to 1185, CV 4.9%) | 1.076 |

An earlier run of the same comparison, with an intermediate build of the branch, gave `plain` 0.983, `slice` 1.007, `slices` 0.996, `str` 0.979 and `strs` 1.006. So the non-`Either` cases sit within about ±2% of 1.0 from run to run, and their before and after ranges overlap.

## Wrapper level (divan)

| case | before ns | after ns | after / before |
|---|---:|---:|---:|
| `plain/plain` | 62.7 (61.8 to 65.0, CV 2.0%) | 64.7 (64.4 to 69.9, CV 2.8%) | 1.033 |
| `borrowed/slice` | 64.7 (63.4 to 66.0, CV 1.3%) | 65.4 (64.6 to 66.2, CV 0.8%) | 1.011 |
| `borrowed/slices` | 135.8 (134.6 to 136.9, CV 0.5%) | 133.5 (131.8 to 138.7, CV 1.6%) | 0.983 |
| `borrowed/str` | 79.8 (77.4 to 81.0, CV 1.4%) | 80.8 (78.2 to 81.0, CV 1.1%) | 1.012 |
| `borrowed/strs` | 210.6 (207.3 to 211.4, CV 0.7%) | 212.6 (210.0 to 213.8, CV 0.6%) | 1.009 |
| `either/left` | 76.8 (76.3 to 78.1, CV 0.7%) | 126.8 (125.2 to 127.8, CV 0.8%) | 1.651 |
| `either/right` | 80.0 (77.7 to 82.8, CV 1.9%) | 140.1 (138.0 to 146.0, CV 1.8%) | 1.752 |
| `either/alone_left` | 63.9 (62.8 to 68.9, CV 3.0%) | 69.0 (66.8 to 71.4, CV 2.0%) | 1.078 |
| `either/alone_right` | 64.1 (63.4 to 65.6, CV 1.1%) | 68.3 (67.5 to 71.3, CV 2.0%) | 1.066 |
| `either/left_aliased` | 77.2 (75.8 to 79.0, CV 1.4%) | 146.4 (144.0 to 150.4, CV 1.5%) | 1.895 |

- **`plain`.** The plain wrapper compiles to the same code before and after: the disassembly of its functions has the same length and makes the same calls. Its ratio over five runs of three builds of the branch was 0.974, 0.996, 0.997, 1.033 and 1.051, so the 1.033 above is code layout and noise, not work.
- **`slices`.** A first build of the branch measured 1.072 here, because the metadata walk had grown: `conflicts_with`, `needs_internal_check` and `visit_leaves` no longer inlined into the check. Each now has an inlined plain-vector path with the nested shapes out of line, which brought the case back to 0.983.
- **`either/left`, `either/right`.** The check now walks both arguments (about 50 to 60 ns). Before, the `Either` declared nothing and the walk was skipped, which also let `Either<i32, &mut [f64]>` alias a `&[f64]` of the same vector.
- **`either/left_aliased`.** The first pass finds the conflict through the `Either`'s right arm, the preparation converts the left arm (and keeps the value), and the second pass finds nothing: about 70 ns.
- **`either/alone_*`.** About 5 ns for `needs_check` to see that one `Either`'s arms cannot alias each other (one arm converts).

## Reproduce

From the repository root, with the before and after builds installed into two libraries:

```sh
# Install the before build (check out the parent of the commit that adds this file), then save it:
just configure && just rcmdinstall
mkdir -p /tmp/nbb/before && cp -R rv/library/4.6/arm64/miniextendr /tmp/nbb/before/
# Install the after build (this commit), then save it:
just configure && just rcmdinstall
mkdir -p /tmp/nbb/after && cp -R rv/library/4.6/arm64/miniextendr /tmp/nbb/after/
# R call level: alternating, 7 reps; writes results.csv, env.log and summary.md
Rscript rpkg/dev/bench-native-borrow/run.R /tmp/nbb/r /tmp/nbb/before /tmp/nbb/after 7

# Wrapper level: build each commit's binary, then alternate them
cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features either --bench native_borrow --no-run
# (copy target/release/deps/native_borrow-* for each commit), then per rep:
R_HOME="$(R RHOME)" ./native_borrow-before --bench --sample-count 100 --sample-size 2000
R_HOME="$(R RHOME)" ./native_borrow-after  --bench --sample-count 100 --sample-size 2000
```

`just bench-features` also runs the `native_borrow` bench, with divan's default sampling.
