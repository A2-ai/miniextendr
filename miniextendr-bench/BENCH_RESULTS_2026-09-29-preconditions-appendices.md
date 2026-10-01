# R-side preconditions vs the Rust conversion: raw results

These are the raw per-variant results and derived deltas behind `BENCH_RESULTS_2026-09-29-preconditions.md` (added in #1663). Appendix A is the main run and Appendix B is the chunked run. The method, the environment and how to reproduce them are in that file's section 0 and its "Reproduce" section (`rpkg/dev/bench-preconditions/run.R`).

## Appendix A: per-variant summary, main run

The measured run (`summary.md` from `analyze.R`): per-variant medians, rep ranges, trimmed means and R heap, then the derived deltas that sections 2 to 5 quote.

<details>
<summary>Per-variant summary and derived deltas</summary>

### Raw per-variant results (median of 3 rep medians; [min-max of rep medians]; trimmed mean; R-heap bytes)


#### group: altrep

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| &[i32] compact | 1:n | 1000 | guarded | 1.35 µs | 1.35 µs–1.35 µs | 1.35 µs | 4.0 KB |
| &[i32] compact | 1:n | 1000 | unguarded | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 4.0 KB |
| &[i32] compact | 1:n | 100000 | guarded | 5.45 µs | 5.29 µs–5.53 µs | 5.51 µs | 390.7 KB |
| &[i32] compact | 1:n | 100000 | unguarded | 5.13 µs | 5.00 µs–5.21 µs | 5.17 µs | 390.7 KB |
| &[i32] compact | 1:n | 1e+07 | guarded | 402.3 µs | 397.1 µs–403.6 µs | 408.0 µs | 38.1 MB |
| &[i32] compact | 1:n | 1e+07 | unguarded | 402.9 µs | 397.6 µs–403.5 µs | 405.1 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1000 | guard_na | 1.39 µs | 1.39 µs–1.39 µs | 1.40 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | guard_type_na | 1.68 µs | 1.68 µs–1.68 µs | 1.68 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | no_check | 1.03 µs | 1.02 µs–1.07 µs | 1.06 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | rust_any_na | 6.35 µs | 6.27 µs–6.36 µs | 6.34 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | rust_either_base | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 100000 | guard_na | 5.41 µs | 5.33 µs–5.45 µs | 5.42 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | guard_type_na | 5.66 µs | 5.62 µs–5.74 µs | 5.69 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | no_check | 5.04 µs | 5.00 µs–5.08 µs | 5.04 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | rust_any_na | 545.2 µs | 526.6 µs–548.4 µs | 544.6 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | rust_either_base | 5.12 µs | 5.00 µs–5.21 µs | 5.14 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 1e+07 | guard_na | 401.3 µs | 396.0 µs–403.4 µs | 405.3 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | guard_type_na | 402.9 µs | 395.4 µs–403.2 µs | 410.3 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | no_check | 402.4 µs | 394.9 µs–403.4 µs | 403.9 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | rust_any_na | 53.75 ms | 52.73 ms–53.76 ms | 53.80 ms | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | rust_either_base | 402.6 µs | 393.8 µs–403.2 µs | 405.7 µs | 38.1 MB |
| scan compact dbl | as.double(1:n) | 1000 | altrep_floor | 779 ns | 738 ns–779 ns | 777 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | ctor_only | 123 ns | 123 ns–123 ns | 110 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | r_anyNA_guard | 1.11 µs | 1.11 µs–1.15 µs | 1.12 µs | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_capi | 410 ns | 410 ns–410 ns | 420 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_elt | 6.27 µs | 6.15 µs–6.31 µs | 6.31 µs | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_floor | 410 ns | 410 ns–410 ns | 406 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_slice | 1.07 µs | 1.03 µs–1.07 µs | 1.08 µs | 7.9 KB |
| scan compact dbl | as.double(1:n) | 100000 | altrep_floor | 779 ns | 779 ns–779 ns | 776 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | ctor_only | 123 ns | 123 ns–123 ns | 107 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | r_anyNA_guard | 1.11 µs | 1.07 µs–1.15 µs | 1.14 µs | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_capi | 410 ns | 410 ns–410 ns | 426 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_elt | 585.8 µs | 574.5 µs–596.6 µs | 591.8 µs | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_floor | 410 ns | 410 ns–410 ns | 404 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_slice | 38.9 µs | 38.0 µs–39.2 µs | 39.4 µs | 781.3 KB |
| scan compact dbl | as.double(1:n) | 1e+07 | altrep_floor | 779 ns | 738 ns–779 ns | 781 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | ctor_only | 123 ns | 123 ns–123 ns | 109 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | r_anyNA_guard | 1.11 µs | 1.11 µs–1.15 µs | 1.12 µs | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_capi | 410 ns | 410 ns–410 ns | 410 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_elt | 58.51 ms | 57.49 ms–58.52 ms | 58.55 ms | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_floor | 410 ns | 410 ns–410 ns | 397 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_slice | 3.83 ms | 3.78 ms–3.84 ms | 4.00 ms | 76.3 MB |
| scan compact int | 1:n | 1000 | altrep_floor | 697 ns | 697 ns–738 ns | 728 ns | 0 B |
| scan compact int | 1:n | 1000 | ctor_only | 82 ns | 82 ns–82 ns | 67 ns | 0 B |
| scan compact int | 1:n | 1000 | r_anyNA_guard | 1.11 µs | 1.07 µs–1.11 µs | 1.10 µs | 0 B |
| scan compact int | 1:n | 1000 | raw_capi | 369 ns | 369 ns–369 ns | 377 ns | 0 B |
| scan compact int | 1:n | 1000 | raw_elt | 6.23 µs | 6.23 µs–6.40 µs | 6.27 µs | 0 B |
| scan compact int | 1:n | 1000 | raw_floor | 369 ns | 369 ns–369 ns | 359 ns | 0 B |
| scan compact int | 1:n | 1000 | raw_slice | 615 ns | 574 ns–615 ns | 615 ns | 4.0 KB |
| scan compact int | 1:n | 100000 | altrep_floor | 697 ns | 697 ns–738 ns | 721 ns | 0 B |
| scan compact int | 1:n | 100000 | ctor_only | 82 ns | 82 ns–82 ns | 66 ns | 0 B |
| scan compact int | 1:n | 100000 | r_anyNA_guard | 1.07 µs | 1.07 µs–1.07 µs | 1.07 µs | 0 B |
| scan compact int | 1:n | 100000 | raw_capi | 369 ns | 369 ns–369 ns | 375 ns | 0 B |
| scan compact int | 1:n | 100000 | raw_elt | 584.3 µs | 573.5 µs–592.2 µs | 585.6 µs | 0 B |
| scan compact int | 1:n | 100000 | raw_floor | 369 ns | 328 ns–369 ns | 358 ns | 0 B |
| scan compact int | 1:n | 100000 | raw_slice | 8.69 µs | 8.49 µs–8.73 µs | 8.70 µs | 390.7 KB |
| scan compact int | 1:n | 1e+07 | altrep_floor | 697 ns | 697 ns–738 ns | 725 ns | 0 B |
| scan compact int | 1:n | 1e+07 | ctor_only | 82 ns | 82 ns–82 ns | 67 ns | 0 B |
| scan compact int | 1:n | 1e+07 | r_anyNA_guard | 1.07 µs | 1.03 µs–1.07 µs | 1.07 µs | 0 B |
| scan compact int | 1:n | 1e+07 | raw_capi | 369 ns | 369 ns–369 ns | 378 ns | 0 B |
| scan compact int | 1:n | 1e+07 | raw_elt | 58.52 ms | 57.48 ms–58.56 ms | 58.59 ms | 0 B |
| scan compact int | 1:n | 1e+07 | raw_floor | 369 ns | 328 ns–369 ns | 359 ns | 0 B |
| scan compact int | 1:n | 1e+07 | raw_slice | 918.7 µs | 888.9 µs–919.6 µs | 1.01 ms | 38.1 MB |

#### group: choices

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | choices_guard | 2.25 µs | 2.21 µs–2.26 µs | 2.25 µs | 0 B |
| choices &str / match_arg enum | "sl" (prefix) | 1 | match_arg_guard | 2.30 µs | 2.26 µs–2.30 µs | 2.30 µs | 0 B |
| choices &str / match_arg enum | "sl" (prefix) | 1 | rust_match_only | 738 ns | 738 ns–738 ns | 761 ns | 0 B |
| choices &str / match_arg enum | "sl" (prefix) | 1 | str_floor | 656 ns | 656 ns–697 ns | 677 ns | 0 B |
| choices &str / match_arg enum | "slow" | 1 | choices_guard | 2.26 µs | 2.17 µs–2.26 µs | 2.25 µs | 0 B |
| choices &str / match_arg enum | "slow" | 1 | choices_guard_u | 2.21 µs | 2.17 µs–2.21 µs | 2.23 µs | 0 B |
| choices &str / match_arg enum | "slow" | 1 | match_arg_guard | 2.26 µs | 2.21 µs–2.26 µs | 2.26 µs | 0 B |
| choices &str / match_arg enum | "slow" | 1 | rust_match_only | 697 ns | 656 ns–697 ns | 695 ns | 0 B |
| choices &str / match_arg enum | "slow" | 1 | str_floor | 656 ns | 656 ns–656 ns | 662 ns | 0 B |
| choices &str / match_arg enum | omitted (default) | 1 | choices_guard | 1.64 µs | 1.60 µs–1.68 µs | 1.65 µs | 0 B |
| choices &str / match_arg enum | omitted (default) | 1 | match_arg_guard | 1.64 µs | 1.64 µs–1.68 µs | 1.67 µs | 0 B |

#### group: df

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| DataFrame, 5 cols | data.frame | 10 | guard_anyNA | 4.51 µs | 4.43 µs–4.55 µs | 4.50 µs | 0 B |
| DataFrame, 5 cols | data.frame | 10 | guard_inherits | 1.43 µs | 1.39 µs–1.48 µs | 1.44 µs | 0 B |
| DataFrame, 5 cols | data.frame | 10 | no_guard | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | guard_anyNA | 6.35 µs | 6.23 µs–6.40 µs | 6.35 µs | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | guard_inherits | 1.39 µs | 1.39 µs–1.44 µs | 1.43 µs | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | no_guard | 1.03 µs | 1.02 µs–1.03 µs | 1.05 µs | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | guard_anyNA | 147.6 µs | 145.0 µs–147.8 µs | 148.0 µs | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | guard_inherits | 1.44 µs | 1.39 µs–1.44 µs | 1.44 µs | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | no_guard | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 0 B |

#### group: inherits

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| inherits List | list, class mx_cls | 3 | guard_inh | 1.07 µs | 1.07 µs–1.11 µs | 1.08 µs | 0 B |
| inherits List | list, class mx_cls | 3 | guard_inh_type | 1.39 µs | 1.35 µs–1.39 µs | 1.41 µs | 0 B |
| inherits List | list, class mx_cls | 3 | guard_type | 1.03 µs | 984 ns–1.07 µs | 1.05 µs | 0 B |
| inherits List | list, class mx_cls | 3 | no_check | 697 ns | 656 ns–697 ns | 695 ns | 0 B |
| inherits scan | list, 4 classes, match last | 3 | r_inherits_guard | 1.03 µs | 1.03 µs–1.07 µs | 1.05 µs | 0 B |
| inherits scan | list, 4 classes, match last | 3 | raw_floor | 287 ns | 287 ns–287 ns | 294 ns | 0 B |
| inherits scan | list, 4 classes, match last | 3 | raw_rf_inherits | 328 ns | 328 ns–328 ns | 326 ns | 0 B |
| inherits scan | list, 4 classes, match last | 3 | sexp_floor | 656 ns | 656 ns–656 ns | 668 ns | 0 B |
| inherits scan | list, class mx_cls | 3 | r_inherits_guard | 1.07 µs | 1.03 µs–1.07 µs | 1.06 µs | 0 B |
| inherits scan | list, class mx_cls | 3 | raw_floor | 287 ns | 287 ns–328 ns | 293 ns | 0 B |
| inherits scan | list, class mx_cls | 3 | raw_rf_inherits | 328 ns | 328 ns–328 ns | 323 ns | 0 B |
| inherits scan | list, class mx_cls | 3 | sexp_floor | 656 ns | 656 ns–656 ns | 663 ns | 0 B |

#### group: nona

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | guard_na | 1.03 µs | 984 ns–1.03 µs | 1.02 µs | 0 B |
| no_na &[f64] | runif(n) | 1 | guard_type_na | 1.31 µs | 1.31 µs–1.31 µs | 1.31 µs | 0 B |
| no_na &[f64] | runif(n) | 1 | no_check | 656 ns | 656 ns–697 ns | 669 ns | 0 B |
| no_na &[f64] | runif(n) | 1 | rust_any_na | 697 ns | 697 ns–697 ns | 702 ns | 0 B |
| no_na &[f64] | runif(n) | 1 | rust_either_base | 656 ns | 656 ns–697 ns | 670 ns | 0 B |
| no_na &[f64] | runif(n) | 1000 | guard_na | 1.27 µs | 1.27 µs–1.31 µs | 1.30 µs | 0 B |
| no_na &[f64] | runif(n) | 1000 | guard_type_na | 1.60 µs | 1.56 µs–1.60 µs | 1.60 µs | 0 B |
| no_na &[f64] | runif(n) | 1000 | no_check | 656 ns | 656 ns–697 ns | 676 ns | 0 B |
| no_na &[f64] | runif(n) | 1000 | rust_any_na | 984 ns | 984 ns–1.03 µs | 1.01 µs | 0 B |
| no_na &[f64] | runif(n) | 1000 | rust_either_base | 697 ns | 656 ns–697 ns | 687 ns | 0 B |
| no_na &[f64] | runif(n) | 100000 | guard_na | 27.6 µs | 27.6 µs–28.8 µs | 27.8 µs | 0 B |
| no_na &[f64] | runif(n) | 100000 | guard_type_na | 28.0 µs | 28.0 µs–29.2 µs | 28.2 µs | 0 B |
| no_na &[f64] | runif(n) | 100000 | no_check | 656 ns | 656 ns–656 ns | 680 ns | 0 B |
| no_na &[f64] | runif(n) | 100000 | rust_any_na | 27.8 µs | 27.3 µs–27.8 µs | 27.8 µs | 0 B |
| no_na &[f64] | runif(n) | 100000 | rust_either_base | 697 ns | 656 ns–697 ns | 684 ns | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | guard_na | 2.70 ms | 2.69 ms–2.75 ms | 2.71 ms | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | guard_type_na | 2.71 ms | 2.69 ms–2.72 ms | 2.72 ms | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | no_check | 697 ns | 697 ns–697 ns | 690 ns | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | rust_any_na | 2.69 ms | 2.69 ms–2.78 ms | 2.70 ms | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | rust_either_base | 697 ns | 656 ns–697 ns | 688 ns | 0 B |
| no_na &[i32] | int(n) | 1 | guard_na | 1.03 µs | 1.03 µs–1.03 µs | 1.02 µs | 0 B |
| no_na &[i32] | int(n) | 1 | guard_type_na | 1.31 µs | 1.31 µs–1.31 µs | 1.31 µs | 0 B |
| no_na &[i32] | int(n) | 1 | no_check | 656 ns | 656 ns–697 ns | 666 ns | 0 B |
| no_na &[i32] | int(n) | 1 | rust_any_na | 697 ns | 697 ns–738 ns | 707 ns | 0 B |
| no_na &[i32] | int(n) | 1 | rust_either_base | 656 ns | 656 ns–697 ns | 669 ns | 0 B |
| no_na &[i32] | int(n) | 1000 | guard_na | 1.31 µs | 1.31 µs–1.31 µs | 1.33 µs | 0 B |
| no_na &[i32] | int(n) | 1000 | guard_type_na | 1.60 µs | 1.60 µs–1.60 µs | 1.61 µs | 0 B |
| no_na &[i32] | int(n) | 1000 | no_check | 656 ns | 656 ns–656 ns | 679 ns | 0 B |
| no_na &[i32] | int(n) | 1000 | rust_any_na | 738 ns | 738 ns–738 ns | 763 ns | 0 B |
| no_na &[i32] | int(n) | 1000 | rust_either_base | 697 ns | 656 ns–697 ns | 686 ns | 0 B |
| no_na &[i32] | int(n) | 100000 | guard_na | 28.4 µs | 27.9 µs–28.7 µs | 28.6 µs | 0 B |
| no_na &[i32] | int(n) | 100000 | guard_type_na | 28.7 µs | 28.2 µs–28.7 µs | 28.9 µs | 0 B |
| no_na &[i32] | int(n) | 100000 | no_check | 656 ns | 656 ns–656 ns | 683 ns | 0 B |
| no_na &[i32] | int(n) | 100000 | rust_any_na | 4.72 µs | 4.63 µs–4.76 µs | 4.75 µs | 0 B |
| no_na &[i32] | int(n) | 100000 | rust_either_base | 656 ns | 656 ns–697 ns | 678 ns | 0 B |
| no_na &[i32] | int(n) | 1e+07 | guard_na | 2.71 ms | 2.70 ms–2.76 ms | 2.71 ms | 0 B |
| no_na &[i32] | int(n) | 1e+07 | guard_type_na | 2.70 ms | 2.70 ms–2.76 ms | 2.71 ms | 0 B |
| no_na &[i32] | int(n) | 1e+07 | no_check | 697 ns | 656 ns–697 ns | 685 ns | 0 B |
| no_na &[i32] | int(n) | 1e+07 | rust_any_na | 405.5 µs | 398.7 µs–482.3 µs | 408.6 µs | 0 B |
| no_na &[i32] | int(n) | 1e+07 | rust_either_base | 697 ns | 656 ns–697 ns | 685 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | r_anyNA_guard | 1.72 µs | 1.72 µs–1.76 µs | 1.74 µs | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | raw_capi | 287 ns | 287 ns–328 ns | 303 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | sexp_floor | 656 ns | 656 ns–656 ns | 665 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | r_anyNA_guard | 3.77 µs | 3.69 µs–3.77 µs | 3.84 µs | 4.0 KB |
| scan classed dbl | structure(runif(n), class) | 1000 | raw_capi | 615 ns | 574 ns–615 ns | 601 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 293 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | sexp_floor | 697 ns | 656 ns–697 ns | 694 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | r_anyNA_guard | 188.8 µs | 188.3 µs–189.1 µs | 192.2 µs | 390.7 KB |
| scan classed dbl | structure(runif(n), class) | 100000 | raw_capi | 27.2 µs | 27.2 µs–27.2 µs | 27.2 µs | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | sexp_floor | 656 ns | 656 ns–697 ns | 684 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | r_anyNA_guard | 19.05 ms | 18.68 ms–19.10 ms | 19.24 ms | 38.1 MB |
| scan classed dbl | structure(runif(n), class) | 1e+07 | raw_capi | 2.68 ms | 2.64 ms–2.70 ms | 2.68 ms | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | sexp_floor | 697 ns | 656 ns–697 ns | 691 ns | 0 B |
| scan dbl | runif(n) | 1 | r_anyNA_guard | 984 ns | 984 ns–1.03 µs | 1.01 µs | 0 B |
| scan dbl | runif(n) | 1 | raw_capi | 328 ns | 328 ns–328 ns | 314 ns | 0 B |
| scan dbl | runif(n) | 1 | raw_elt | 328 ns | 328 ns–328 ns | 319 ns | 0 B |
| scan dbl | runif(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| scan dbl | runif(n) | 1 | raw_slice | 287 ns | 287 ns–328 ns | 309 ns | 0 B |
| scan dbl | runif(n) | 1 | sexp_floor | 656 ns | 615 ns–656 ns | 647 ns | 0 B |
| scan dbl | runif(n) | 1000 | r_anyNA_guard | 1.27 µs | 1.27 µs–1.31 µs | 1.30 µs | 0 B |
| scan dbl | runif(n) | 1000 | raw_capi | 615 ns | 615 ns–615 ns | 601 ns | 0 B |
| scan dbl | runif(n) | 1000 | raw_elt | 615 ns | 574 ns–615 ns | 604 ns | 0 B |
| scan dbl | runif(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan dbl | runif(n) | 1000 | raw_slice | 615 ns | 574 ns–615 ns | 604 ns | 0 B |
| scan dbl | runif(n) | 1000 | sexp_floor | 656 ns | 656 ns–656 ns | 657 ns | 0 B |
| scan dbl | runif(n) | 100000 | r_anyNA_guard | 28.1 µs | 27.6 µs–28.2 µs | 28.1 µs | 0 B |
| scan dbl | runif(n) | 100000 | raw_capi | 27.2 µs | 26.7 µs–27.2 µs | 27.2 µs | 0 B |
| scan dbl | runif(n) | 100000 | raw_elt | 27.2 µs | 26.7 µs–27.2 µs | 27.3 µs | 0 B |
| scan dbl | runif(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| scan dbl | runif(n) | 100000 | raw_slice | 27.2 µs | 26.7 µs–27.2 µs | 27.1 µs | 0 B |
| scan dbl | runif(n) | 100000 | sexp_floor | 656 ns | 656 ns–656 ns | 650 ns | 0 B |
| scan dbl | runif(n) | 1e+07 | r_anyNA_guard | 2.69 ms | 2.69 ms–2.76 ms | 2.69 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_capi | 2.69 ms | 2.68 ms–2.74 ms | 2.69 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_elt | 2.69 ms | 2.69 ms–2.73 ms | 2.71 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_slice | 2.68 ms | 2.68 ms–2.68 ms | 2.68 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | sexp_floor | 656 ns | 656 ns–697 ns | 674 ns | 0 B |
| scan int | int(n) | 1 | r_anyNA_guard | 984 ns | 984 ns–1.03 µs | 1.00 µs | 0 B |
| scan int | int(n) | 1 | raw_capi | 328 ns | 328 ns–328 ns | 319 ns | 0 B |
| scan int | int(n) | 1 | raw_elt | 328 ns | 328 ns–328 ns | 321 ns | 0 B |
| scan int | int(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 294 ns | 0 B |
| scan int | int(n) | 1 | raw_slice | 328 ns | 287 ns–328 ns | 313 ns | 0 B |
| scan int | int(n) | 1 | sexp_floor | 656 ns | 656 ns–656 ns | 654 ns | 0 B |
| scan int | int(n) | 1000 | r_anyNA_guard | 1.31 µs | 1.27 µs–1.31 µs | 1.31 µs | 0 B |
| scan int | int(n) | 1000 | raw_capi | 369 ns | 328 ns–369 ns | 357 ns | 0 B |
| scan int | int(n) | 1000 | raw_elt | 369 ns | 369 ns–369 ns | 362 ns | 0 B |
| scan int | int(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan int | int(n) | 1000 | raw_slice | 369 ns | 328 ns–369 ns | 355 ns | 0 B |
| scan int | int(n) | 1000 | sexp_floor | 656 ns | 656 ns–656 ns | 660 ns | 0 B |
| scan int | int(n) | 100000 | r_anyNA_guard | 28.3 µs | 27.8 µs–28.3 µs | 28.2 µs | 0 B |
| scan int | int(n) | 100000 | raw_capi | 4.26 µs | 4.22 µs–4.30 µs | 4.28 µs | 0 B |
| scan int | int(n) | 100000 | raw_elt | 4.26 µs | 4.26 µs–4.26 µs | 4.28 µs | 0 B |
| scan int | int(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| scan int | int(n) | 100000 | raw_slice | 4.26 µs | 4.22 µs–4.26 µs | 4.26 µs | 0 B |
| scan int | int(n) | 100000 | sexp_floor | 656 ns | 656 ns–656 ns | 670 ns | 0 B |
| scan int | int(n) | 1e+07 | r_anyNA_guard | 2.72 ms | 2.71 ms–2.77 ms | 2.72 ms | 0 B |
| scan int | int(n) | 1e+07 | raw_capi | 413.5 µs | 407.4 µs–418.0 µs | 417.8 µs | 0 B |
| scan int | int(n) | 1e+07 | raw_elt | 411.0 µs | 410.2 µs–418.9 µs | 415.1 µs | 0 B |
| scan int | int(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| scan int | int(n) | 1e+07 | raw_slice | 405.0 µs | 401.1 µs–421.0 µs | 408.2 µs | 0 B |
| scan int | int(n) | 1e+07 | sexp_floor | 656 ns | 656 ns–656 ns | 666 ns | 0 B |

#### group: scalar

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| (i32, f64) | 1L, 2.5 | 1 | guarded | 2.13 µs | 2.09 µs–2.13 µs | 2.13 µs | 0 B |
| (i32, f64) | 1L, 2.5 | 1 | unguarded | 779 ns | 779 ns–820 ns | 783 ns | 0 B |
| &str | "abc" | 1 | guarded | 1.31 µs | 1.31 µs–1.35 µs | 1.34 µs | 0 B |
| &str | "abc" | 1 | unguarded | 656 ns | 656 ns–656 ns | 651 ns | 0 B |
| bool | TRUE | 1 | guarded | 1.31 µs | 1.31 µs–1.35 µs | 1.34 µs | 0 B |
| bool | TRUE | 1 | unguarded | 656 ns | 656 ns–656 ns | 648 ns | 0 B |
| f64 | 1.5 | 1 | guarded | 1.35 µs | 1.35 µs–1.35 µs | 1.37 µs | 0 B |
| f64 | 1.5 | 1 | unguarded | 697 ns | 656 ns–697 ns | 693 ns | 0 B |
| fast_i32 (existing) | 1L | 1 | guarded | 1.31 µs | 1.31 µs–1.35 µs | 1.35 µs | 0 B |
| fast_i32 (existing) | 1L | 1 | unguarded | 656 ns | 656 ns–656 ns | 677 ns | 0 B |
| fast_sum3 (existing) | 1L,2L,3L | 1 | guarded | 2.79 µs | 2.75 µs–2.87 µs | 2.78 µs | 0 B |
| fast_sum3 (existing) | 1L,2L,3L | 1 | unguarded | 861 ns | 820 ns–861 ns | 886 ns | 0 B |
| floors | 1.5 | 1 | r_guard_only | 574 ns | 574 ns–574 ns | 564 ns | 0 B |
| floors | 1.5 | 1 | raw_call | 287 ns | 287 ns–287 ns | 283 ns | 0 B |
| floors | 1.5 | 1 | raw_type_len | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| i32 | 1L | 1 | guarded | 1.35 µs | 1.31 µs–1.39 µs | 1.35 µs | 0 B |
| i32 | 1L | 1 | unguarded | 656 ns | 656 ns–656 ns | 660 ns | 0 B |
| Option<i32> | 1L | 1 | guarded | 1.39 µs | 1.39 µs–1.39 µs | 1.40 µs | 0 B |
| Option<i32> | 1L | 1 | unguarded | 656 ns | 656 ns–656 ns | 658 ns | 0 B |
| Option<i32> | NULL | 1 | guarded | 1.23 µs | 1.23 µs–1.27 µs | 1.26 µs | 0 B |
| Option<i32> | NULL | 1 | unguarded | 656 ns | 615 ns–656 ns | 652 ns | 0 B |
| String | "abc" | 1 | guarded | 1.35 µs | 1.35 µs–1.39 µs | 1.39 µs | 0 B |
| String | "abc" | 1 | unguarded | 697 ns | 656 ns–697 ns | 695 ns | 0 B |

#### group: vector

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| &[f64] | runif(n) | 1 | guarded | 984 ns | 943 ns–984 ns | 986 ns | 0 B |
| &[f64] | runif(n) | 1 | unguarded | 656 ns | 656 ns–656 ns | 675 ns | 0 B |
| &[f64] | runif(n) | 1000 | guarded | 984 ns | 984 ns–984 ns | 979 ns | 0 B |
| &[f64] | runif(n) | 1000 | unguarded | 656 ns | 656 ns–656 ns | 678 ns | 0 B |
| &[f64] | runif(n) | 100000 | guarded | 984 ns | 984 ns–984 ns | 987 ns | 0 B |
| &[f64] | runif(n) | 100000 | unguarded | 656 ns | 656 ns–697 ns | 667 ns | 0 B |
| &[f64] | runif(n) | 1e+07 | guarded | 943 ns | 943 ns–943 ns | 961 ns | 0 B |
| &[f64] | runif(n) | 1e+07 | unguarded | 656 ns | 656 ns–697 ns | 672 ns | 0 B |
| &[i32] | int(n) | 1 | guarded | 984 ns | 943 ns–984 ns | 981 ns | 0 B |
| &[i32] | int(n) | 1 | unguarded | 656 ns | 656 ns–656 ns | 672 ns | 0 B |
| &[i32] | int(n) | 1000 | guarded | 984 ns | 943 ns–984 ns | 986 ns | 0 B |
| &[i32] | int(n) | 1000 | unguarded | 656 ns | 656 ns–697 ns | 670 ns | 0 B |
| &[i32] | int(n) | 100000 | guarded | 984 ns | 943 ns–984 ns | 984 ns | 0 B |
| &[i32] | int(n) | 100000 | unguarded | 656 ns | 656 ns–697 ns | 667 ns | 0 B |
| &[i32] | int(n) | 1e+07 | guarded | 984 ns | 943 ns–984 ns | 980 ns | 0 B |
| &[i32] | int(n) | 1e+07 | unguarded | 656 ns | 656 ns–697 ns | 668 ns | 0 B |
| Vec<f64> | runif(n) | 1 | guarded | 984 ns | 984 ns–984 ns | 997 ns | 0 B |
| Vec<f64> | runif(n) | 1 | unguarded | 656 ns | 656 ns–697 ns | 686 ns | 0 B |
| Vec<f64> | runif(n) | 1000 | guarded | 1.15 µs | 1.15 µs–1.15 µs | 1.15 µs | 0 B |
| Vec<f64> | runif(n) | 1000 | unguarded | 820 ns | 820 ns–820 ns | 845 ns | 0 B |
| Vec<f64> | runif(n) | 100000 | guarded | 10.7 µs | 10.7 µs–10.9 µs | 10.8 µs | 0 B |
| Vec<f64> | runif(n) | 100000 | unguarded | 10.4 µs | 10.3 µs–10.6 µs | 10.7 µs | 0 B |
| Vec<f64> | runif(n) | 1e+07 | guarded | 1.23 ms | 1.23 ms–1.25 ms | 1.23 ms | 0 B |
| Vec<f64> | runif(n) | 1e+07 | unguarded | 1.23 ms | 1.23 ms–1.28 ms | 1.23 ms | 0 B |
| Vec<i32> | int(n) | 1 | guarded | 984 ns | 984 ns–1.03 µs | 999 ns | 0 B |
| Vec<i32> | int(n) | 1 | unguarded | 697 ns | 697 ns–697 ns | 705 ns | 0 B |
| Vec<i32> | int(n) | 1000 | guarded | 1.07 µs | 1.07 µs–1.11 µs | 1.10 µs | 0 B |
| Vec<i32> | int(n) | 1000 | unguarded | 779 ns | 779 ns–779 ns | 784 ns | 0 B |
| Vec<i32> | int(n) | 100000 | guarded | 6.11 µs | 6.07 µs–6.23 µs | 6.13 µs | 0 B |
| Vec<i32> | int(n) | 100000 | unguarded | 5.74 µs | 5.70 µs–5.86 µs | 5.80 µs | 0 B |
| Vec<i32> | int(n) | 1e+07 | guarded | 583.6 µs | 583.5 µs–605.7 µs | 584.9 µs | 0 B |
| Vec<i32> | int(n) | 1e+07 | unguarded | 582.0 µs | 581.5 µs–595.4 µs | 584.7 µs | 0 B |

#### group: whole

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | guarded | 7.95 µs | 7.83 µs–8.77 µs | 8.01 µs | 27.6 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | raw_capi | 1.03 µs | 984 ns–1.03 µs | 1.03 µs | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | raw_floor | 410 ns | 410 ns–410 ns | 407 ns | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | raw_slice | 1.19 µs | 1.11 µs–1.19 µs | 1.20 µs | 7.9 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | unguarded | 2.13 µs | 2.09 µs–2.13 µs | 2.16 µs | 7.9 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | guarded | 593.9 µs | 570.0 µs–696.0 µs | 614.3 µs | 2.7 MB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | raw_capi | 56.3 µs | 55.3 µs–56.5 µs | 56.5 µs | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | raw_floor | 410 ns | 410 ns–410 ns | 412 ns | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | raw_slice | 52.1 µs | 51.1 µs–53.2 µs | 52.8 µs | 781.3 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | unguarded | 119.4 µs | 116.7 µs–121.8 µs | 131.5 µs | 781.3 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | guarded | 60.30 ms | 57.59 ms–60.58 ms | 60.75 ms | 267.0 MB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | raw_capi | 5.58 ms | 5.49 ms–5.63 ms | 5.56 ms | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | raw_floor | 410 ns | 410 ns–410 ns | 414 ns | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | raw_slice | 5.15 ms | 5.06 ms–5.30 ms | 5.21 ms | 76.3 MB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | unguarded | 11.99 ms | 11.57 ms–12.06 ms | 12.26 ms | 76.3 MB |
| coerce Vec<i32> from doubles | whole doubles | 1 | guarded | 1.44 µs | 1.39 µs–1.44 µs | 1.44 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | r_guard_expr | 738 ns | 697 ns–738 ns | 730 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | raw_capi | 287 ns | 287 ns–287 ns | 308 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | raw_slice | 328 ns | 287 ns–328 ns | 316 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | unguarded | 697 ns | 697 ns–697 ns | 696 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | guarded | 6.56 µs | 6.35 µs–7.26 µs | 6.83 µs | 19.7 KB |
| coerce Vec<i32> from doubles | whole doubles | 1000 | r_guard_expr | 4.76 µs | 4.67 µs–4.76 µs | 4.79 µs | 19.7 KB |
| coerce Vec<i32> from doubles | whole doubles | 1000 | raw_capi | 738 ns | 697 ns–738 ns | 726 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | raw_slice | 738 ns | 697 ns–738 ns | 736 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | unguarded | 1.80 µs | 1.72 µs–1.80 µs | 1.80 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | guarded | 540.5 µs | 479.5 µs–580.3 µs | 556.4 µs | 1.9 MB |
| coerce Vec<i32> from doubles | whole doubles | 100000 | r_guard_expr | 365.4 µs | 353.4 µs–367.4 µs | 368.0 µs | 1.9 MB |
| coerce Vec<i32> from doubles | whole doubles | 100000 | raw_capi | 40.4 µs | 39.6 µs–40.4 µs | 40.5 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | raw_slice | 40.4 µs | 39.6 µs–40.4 µs | 40.3 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | unguarded | 107.3 µs | 105.2 µs–107.3 µs | 109.3 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | guarded | 48.93 ms | 46.53 ms–56.67 ms | 52.13 ms | 190.7 MB |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | r_guard_expr | 37.87 ms | 37.27 ms–38.17 ms | 41.24 ms | 190.7 MB |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | raw_capi | 4.08 ms | 3.94 ms–4.10 ms | 4.09 ms | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | raw_slice | 4.03 ms | 3.94 ms–4.09 ms | 4.04 ms | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | unguarded | 10.85 ms | 10.45 ms–19.47 ms | 11.01 ms | 0 B |


### Derived deltas (trimmed means)

#### Scalars: default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| (i32, f64) | 1L, 2.5 | 1 | 2.13 µs | 783 ns | 1.35 µs | 63% | 0 B | 0 B |
| &str | "abc" | 1 | 1.34 µs | 651 ns | 691 ns | 51% | 0 B | 0 B |
| bool | TRUE | 1 | 1.34 µs | 648 ns | 687 ns | 51% | 0 B | 0 B |
| f64 | 1.5 | 1 | 1.37 µs | 693 ns | 676 ns | 49% | 0 B | 0 B |
| fast_i32 (existing) | 1L | 1 | 1.35 µs | 677 ns | 669 ns | 50% | 0 B | 0 B |
| fast_sum3 (existing) | 1L,2L,3L | 1 | 2.78 µs | 886 ns | 1.90 µs | 68% | 0 B | 0 B |
| i32 | 1L | 1 | 1.35 µs | 660 ns | 690 ns | 51% | 0 B | 0 B |
| Option<i32> | 1L | 1 | 1.40 µs | 658 ns | 746 ns | 53% | 0 B | 0 B |
| Option<i32> | NULL | 1 | 1.26 µs | 652 ns | 607 ns | 48% | 0 B | 0 B |
| String | "abc" | 1 | 1.39 µs | 695 ns | 693 ns | 50% | 0 B | 0 B |

#### Vectors: default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| &[f64] | runif(n) | 1 | 986 ns | 675 ns | 311 ns | 32% | 0 B | 0 B |
| &[f64] | runif(n) | 1000 | 979 ns | 678 ns | 301 ns | 31% | 0 B | 0 B |
| &[f64] | runif(n) | 100000 | 987 ns | 667 ns | 320 ns | 32% | 0 B | 0 B |
| &[f64] | runif(n) | 1e+07 | 961 ns | 672 ns | 290 ns | 30% | 0 B | 0 B |
| &[i32] | int(n) | 1 | 981 ns | 672 ns | 309 ns | 31% | 0 B | 0 B |
| &[i32] | int(n) | 1000 | 986 ns | 670 ns | 316 ns | 32% | 0 B | 0 B |
| &[i32] | int(n) | 100000 | 984 ns | 667 ns | 317 ns | 32% | 0 B | 0 B |
| &[i32] | int(n) | 1e+07 | 980 ns | 668 ns | 312 ns | 32% | 0 B | 0 B |
| Vec<f64> | runif(n) | 1 | 997 ns | 686 ns | 310 ns | 31% | 0 B | 0 B |
| Vec<f64> | runif(n) | 1000 | 1.15 µs | 845 ns | 310 ns | 27% | 0 B | 0 B |
| Vec<f64> | runif(n) | 100000 | 10.8 µs | 10.7 µs | 90 ns | 1% | 0 B | 0 B |
| Vec<f64> | runif(n) | 1e+07 | 1.23 ms | 1.23 ms | 4.01 µs | 0% | 0 B | 0 B |
| Vec<i32> | int(n) | 1 | 999 ns | 705 ns | 294 ns | 29% | 0 B | 0 B |
| Vec<i32> | int(n) | 1000 | 1.10 µs | 784 ns | 315 ns | 29% | 0 B | 0 B |
| Vec<i32> | int(n) | 100000 | 6.13 µs | 5.80 µs | 332 ns | 5% | 0 B | 0 B |
| Vec<i32> | int(n) | 1e+07 | 584.9 µs | 584.7 µs | 203 ns | 0% | 0 B | 0 B |

#### no_na typed: R is.double/is.integer + anyNA vs no check

| case | input | n | R type + anyNA | no check | saved | share of R type + anyNA call | R heap R type + anyNA | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | 1.31 µs | 669 ns | 641 ns | 49% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1000 | 1.60 µs | 676 ns | 925 ns | 58% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 100000 | 28.2 µs | 680 ns | 27.5 µs | 98% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | 2.72 ms | 690 ns | 2.72 ms | 100% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1 | 1.31 µs | 666 ns | 647 ns | 49% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1000 | 1.61 µs | 679 ns | 930 ns | 58% | 0 B | 0 B |
| no_na &[i32] | int(n) | 100000 | 28.9 µs | 683 ns | 28.2 µs | 98% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1e+07 | 2.71 ms | 685 ns | 2.71 ms | 100% | 0 B | 0 B |

#### no_na typed: R anyNA only vs no check

| case | input | n | R anyNA | no check | saved | share of R anyNA call | R heap R anyNA | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | 1.02 µs | 669 ns | 349 ns | 34% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1000 | 1.30 µs | 676 ns | 625 ns | 48% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 100000 | 27.8 µs | 680 ns | 27.1 µs | 98% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | 2.71 ms | 690 ns | 2.71 ms | 100% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1 | 1.02 µs | 666 ns | 350 ns | 34% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1000 | 1.33 µs | 679 ns | 652 ns | 49% | 0 B | 0 B |
| no_na &[i32] | int(n) | 100000 | 28.6 µs | 683 ns | 27.9 µs | 98% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1e+07 | 2.71 ms | 685 ns | 2.71 ms | 100% | 0 B | 0 B |

#### no_na typed: Rust any_na (Either) vs Either without check

| case | input | n | Rust any_na | no check | saved | share of Rust any_na call | R heap Rust any_na | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | 702 ns | 670 ns | 33 ns | 5% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1000 | 1.01 µs | 687 ns | 327 ns | 32% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 100000 | 27.8 µs | 684 ns | 27.1 µs | 98% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | 2.70 ms | 688 ns | 2.70 ms | 100% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1 | 707 ns | 669 ns | 38 ns | 5% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1000 | 763 ns | 686 ns | 77 ns | 10% | 0 B | 0 B |
| no_na &[i32] | int(n) | 100000 | 4.75 µs | 678 ns | 4.07 µs | 86% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1e+07 | 408.6 µs | 685 ns | 407.9 µs | 100% | 0 B | 0 B |

#### NA scan, R anyNA guard vs bare-SEXP floor

| case | input | n | R anyNA | floor | saved | share of R anyNA call | R heap R anyNA | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan classed dbl | structure(runif(n), class) | 1 | 1.74 µs | 665 ns | 1.07 µs | 62% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | 3.84 µs | 694 ns | 3.15 µs | 82% | 4.0 KB | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | 192.2 µs | 684 ns | 191.5 µs | 100% | 390.7 KB | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | 19.24 ms | 691 ns | 19.23 ms | 100% | 38.1 MB | 0 B |
| scan dbl | runif(n) | 1 | 1.01 µs | 647 ns | 363 ns | 36% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 1.30 µs | 657 ns | 648 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 28.1 µs | 650 ns | 27.5 µs | 98% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.69 ms | 674 ns | 2.69 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 1.00 µs | 654 ns | 351 ns | 35% | 0 B | 0 B |
| scan int | int(n) | 1000 | 1.31 µs | 660 ns | 647 ns | 50% | 0 B | 0 B |
| scan int | int(n) | 100000 | 28.2 µs | 670 ns | 27.6 µs | 98% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 2.72 ms | 666 ns | 2.72 ms | 100% | 0 B | 0 B |

#### NA scan, raw C DATAPTR slice vs raw floor

| case | input | n | slice scan | floor | saved | share of slice scan call | R heap slice scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan dbl | runif(n) | 1 | 309 ns | 298 ns | 11 ns | 3% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 604 ns | 300 ns | 303 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 27.1 µs | 299 ns | 26.8 µs | 99% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.68 ms | 300 ns | 2.68 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 313 ns | 294 ns | 18 ns | 6% | 0 B | 0 B |
| scan int | int(n) | 1000 | 355 ns | 300 ns | 55 ns | 15% | 0 B | 0 B |
| scan int | int(n) | 100000 | 4.26 µs | 298 ns | 3.97 µs | 93% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 408.2 µs | 299 ns | 407.9 µs | 100% | 0 B | 0 B |

#### NA scan, raw C *_ELT (from_r::any_na shape) vs raw floor

| case | input | n | ELT scan | floor | saved | share of ELT scan call | R heap ELT scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan dbl | runif(n) | 1 | 319 ns | 298 ns | 21 ns | 7% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 604 ns | 300 ns | 304 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 27.3 µs | 299 ns | 27.0 µs | 99% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.71 ms | 300 ns | 2.71 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 321 ns | 294 ns | 26 ns | 8% | 0 B | 0 B |
| scan int | int(n) | 1000 | 362 ns | 300 ns | 62 ns | 17% | 0 B | 0 B |
| scan int | int(n) | 100000 | 4.28 µs | 298 ns | 3.98 µs | 93% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 415.1 µs | 299 ns | 414.8 µs | 100% | 0 B | 0 B |

#### NA scan, raw C NO_NA+OR_NULL+GET_REGION vs raw floor

| case | input | n | C-API scan | floor | saved | share of C-API scan call | R heap C-API scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan classed dbl | structure(runif(n), class) | 1 | 303 ns | 296 ns | 7 ns | 2% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | 601 ns | 293 ns | 308 ns | 51% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | 27.2 µs | 300 ns | 26.9 µs | 99% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | 2.68 ms | 299 ns | 2.68 ms | 100% | 0 B | 0 B |
| scan dbl | runif(n) | 1 | 314 ns | 298 ns | 15 ns | 5% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 601 ns | 300 ns | 301 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 27.2 µs | 299 ns | 26.9 µs | 99% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.69 ms | 300 ns | 2.69 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 319 ns | 294 ns | 25 ns | 8% | 0 B | 0 B |
| scan int | int(n) | 1000 | 357 ns | 300 ns | 57 ns | 16% | 0 B | 0 B |
| scan int | int(n) | 100000 | 4.28 µs | 298 ns | 3.99 µs | 93% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 417.8 µs | 299 ns | 417.5 µs | 100% | 0 B | 0 B |

#### ALTREP: R anyNA guard vs AltrepSexp floor

| case | input | n | R anyNA | floor | saved | share of R anyNA call | R heap R anyNA | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 1.12 µs | 777 ns | 345 ns | 31% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 1.14 µs | 776 ns | 368 ns | 32% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 1.12 µs | 781 ns | 342 ns | 30% | 0 B | 0 B |
| scan compact int | 1:n | 1000 | 1.10 µs | 728 ns | 376 ns | 34% | 0 B | 0 B |
| scan compact int | 1:n | 100000 | 1.07 µs | 721 ns | 345 ns | 32% | 0 B | 0 B |
| scan compact int | 1:n | 1e+07 | 1.07 µs | 725 ns | 346 ns | 32% | 0 B | 0 B |

#### ALTREP: raw C-API scan vs raw floor

| case | input | n | C-API scan | floor | saved | share of C-API scan call | R heap C-API scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 420 ns | 406 ns | 15 ns | 3% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 426 ns | 404 ns | 21 ns | 5% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 410 ns | 397 ns | 12 ns | 3% | 0 B | 0 B |
| scan compact int | 1:n | 1000 | 377 ns | 359 ns | 18 ns | 5% | 0 B | 0 B |
| scan compact int | 1:n | 100000 | 375 ns | 358 ns | 17 ns | 5% | 0 B | 0 B |
| scan compact int | 1:n | 1e+07 | 378 ns | 359 ns | 19 ns | 5% | 0 B | 0 B |

#### ALTREP: raw *_ELT scan vs raw floor

| case | input | n | ELT scan | floor | saved | share of ELT scan call | R heap ELT scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 6.31 µs | 406 ns | 5.90 µs | 94% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 591.8 µs | 404 ns | 591.4 µs | 100% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 58.55 ms | 397 ns | 58.55 ms | 100% | 0 B | 0 B |
| scan compact int | 1:n | 1000 | 6.27 µs | 359 ns | 5.91 µs | 94% | 0 B | 0 B |
| scan compact int | 1:n | 100000 | 585.6 µs | 358 ns | 585.3 µs | 100% | 0 B | 0 B |
| scan compact int | 1:n | 1e+07 | 58.59 ms | 359 ns | 58.58 ms | 100% | 0 B | 0 B |

#### ALTREP: raw DATAPTR slice scan vs raw floor

| case | input | n | slice scan | floor | saved | share of slice scan call | R heap slice scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 1.08 µs | 406 ns | 674 ns | 62% | 7.9 KB | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 39.4 µs | 404 ns | 39.0 µs | 99% | 781.3 KB | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 4.00 ms | 397 ns | 4.00 ms | 100% | 76.3 MB | 0 B |
| scan compact int | 1:n | 1000 | 615 ns | 359 ns | 256 ns | 42% | 4.0 KB | 0 B |
| scan compact int | 1:n | 100000 | 8.70 µs | 358 ns | 8.34 µs | 96% | 390.7 KB | 0 B |
| scan compact int | 1:n | 1e+07 | 1.01 ms | 359 ns | 1.01 ms | 100% | 38.1 MB | 0 B |

#### ALTREP: typed R anyNA vs no check

| case | input | n | R anyNA | no check | saved | share of R anyNA call | R heap R anyNA | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[i32] compact | 1:n | 1000 | 1.40 µs | 1.06 µs | 344 ns | 25% | 4.0 KB | 4.0 KB |
| no_na &[i32] compact | 1:n | 100000 | 5.42 µs | 5.04 µs | 378 ns | 7% | 390.7 KB | 390.7 KB |
| no_na &[i32] compact | 1:n | 1e+07 | 405.3 µs | 403.9 µs | 1.46 µs | 0% | 38.1 MB | 38.1 MB |

#### ALTREP: typed Rust any_na vs Either base

| case | input | n | Rust any_na | no check | saved | share of Rust any_na call | R heap Rust any_na | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[i32] compact | 1:n | 1000 | 6.34 µs | 1.04 µs | 5.30 µs | 84% | 4.0 KB | 4.0 KB |
| no_na &[i32] compact | 1:n | 100000 | 544.6 µs | 5.14 µs | 539.4 µs | 99% | 390.7 KB | 390.7 KB |
| no_na &[i32] compact | 1:n | 1e+07 | 53.80 ms | 405.7 µs | 53.39 ms | 99% | 38.1 MB | 38.1 MB |

#### ALTREP: &[i32] default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| &[i32] compact | 1:n | 1000 | 1.35 µs | 1.04 µs | 311 ns | 23% | 4.0 KB | 4.0 KB |
| &[i32] compact | 1:n | 100000 | 5.51 µs | 5.17 µs | 340 ns | 6% | 390.7 KB | 390.7 KB |
| &[i32] compact | 1:n | 1e+07 | 408.0 µs | 405.1 µs | 2.86 µs | 1% | 38.1 MB | 38.1 MB |

#### Whole-number: coerce default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | 8.01 µs | 2.16 µs | 5.85 µs | 73% | 27.6 KB | 7.9 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | 614.3 µs | 131.5 µs | 482.8 µs | 79% | 2.7 MB | 781.3 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | 60.75 ms | 12.26 ms | 48.49 ms | 80% | 267.0 MB | 76.3 MB |
| coerce Vec<i32> from doubles | whole doubles | 1 | 1.44 µs | 696 ns | 740 ns | 52% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | 6.83 µs | 1.80 µs | 5.03 µs | 74% | 19.7 KB | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | 556.4 µs | 109.3 µs | 447.1 µs | 80% | 1.9 MB | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | 52.13 ms | 11.01 ms | 41.13 ms | 79% | 190.7 MB | 0 B |

#### Whole-number: raw C slice scan vs raw floor

| case | input | n | slice scan | floor | saved | share of slice scan call | R heap slice scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | 1.20 µs | 407 ns | 788 ns | 66% | 7.9 KB | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | 52.8 µs | 412 ns | 52.4 µs | 99% | 781.3 KB | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | 5.21 ms | 414 ns | 5.21 ms | 100% | 76.3 MB | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | 316 ns | 298 ns | 18 ns | 6% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | 736 ns | 298 ns | 438 ns | 60% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | 40.3 µs | 299 ns | 40.0 µs | 99% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | 4.04 ms | 299 ns | 4.04 ms | 100% | 0 B | 0 B |

#### Whole-number: raw C OR_NULL/GET_REGION scan vs raw floor

| case | input | n | C-API scan | floor | saved | share of C-API scan call | R heap C-API scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | 1.03 µs | 407 ns | 626 ns | 61% | 0 B | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | 56.5 µs | 412 ns | 56.1 µs | 99% | 0 B | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | 5.56 ms | 414 ns | 5.56 ms | 100% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | 308 ns | 298 ns | 10 ns | 3% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | 726 ns | 298 ns | 429 ns | 59% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | 40.5 µs | 299 ns | 40.2 µs | 99% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | 4.09 ms | 299 ns | 4.09 ms | 100% | 0 B | 0 B |

#### inherits List: inherits+is.list vs no check

| case | input | n | R inherits+is.list | no check | saved | share of R inherits+is.list call | R heap R inherits+is.list | R heap no check |
|---|---|---|---|---|---|---|---|---|
| inherits List | list, class mx_cls | 3 | 1.41 µs | 695 ns | 713 ns | 51% | 0 B | 0 B |

#### inherits List: inherits only vs no check

| case | input | n | R inherits | no check | saved | share of R inherits call | R heap R inherits | R heap no check |
|---|---|---|---|---|---|---|---|---|
| inherits List | list, class mx_cls | 3 | 1.08 µs | 695 ns | 389 ns | 36% | 0 B | 0 B |

#### inherits: R guard vs bare-SEXP floor

| case | input | n | R inherits | floor | saved | share of R inherits call | R heap R inherits | R heap floor |
|---|---|---|---|---|---|---|---|---|
| inherits scan | list, 4 classes, match last | 3 | 1.05 µs | 668 ns | 378 ns | 36% | 0 B | 0 B |
| inherits scan | list, class mx_cls | 3 | 1.06 µs | 663 ns | 397 ns | 37% | 0 B | 0 B |

#### inherits: raw Rf_inherits vs raw floor

| case | input | n | Rf_inherits | floor | saved | share of Rf_inherits call | R heap Rf_inherits | R heap floor |
|---|---|---|---|---|---|---|---|---|
| inherits scan | list, 4 classes, match last | 3 | 326 ns | 294 ns | 32 ns | 10% | 0 B | 0 B |
| inherits scan | list, class mx_cls | 3 | 323 ns | 293 ns | 29 ns | 9% | 0 B | 0 B |

#### DataFrame: R inherits guard vs none

| case | input | n | R inherits | none | saved | share of R inherits call | R heap R inherits | R heap none |
|---|---|---|---|---|---|---|---|---|
| DataFrame, 5 cols | data.frame | 10 | 1.44 µs | 1.04 µs | 394 ns | 27% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | 1.43 µs | 1.05 µs | 383 ns | 27% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | 1.44 µs | 1.04 µs | 392 ns | 27% | 0 B | 0 B |

#### DataFrame: R anyNA guard vs none

| case | input | n | R anyNA | none | saved | share of R anyNA call | R heap R anyNA | R heap none |
|---|---|---|---|---|---|---|---|---|
| DataFrame, 5 cols | data.frame | 10 | 4.50 µs | 1.04 µs | 3.46 µs | 77% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | 6.35 µs | 1.05 µs | 5.30 µs | 83% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | 148.0 µs | 1.04 µs | 146.9 µs | 99% | 0 B | 0 B |

#### choices &str: R helper vs &str floor

| case | input | n | R helper | &str floor | saved | share of R helper call | R heap R helper | R heap &str floor |
|---|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | 2.25 µs | 677 ns | 1.57 µs | 70% | 0 B | 0 B |
| choices &str / match_arg enum | "slow" | 1 | 2.25 µs | 662 ns | 1.59 µs | 71% | 0 B | 0 B |

#### match_arg enum: R helper + Rust vs Rust matching only

| case | input | n | R helper + Rust | Rust only | saved | share of R helper + Rust call | R heap R helper + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | 2.30 µs | 761 ns | 1.54 µs | 67% | 0 B | 0 B |
| choices &str / match_arg enum | "slow" | 1 | 2.26 µs | 695 ns | 1.57 µs | 69% | 0 B | 0 B |

#### match_arg enum: Rust matching vs &str floor

| case | input | n | Rust match | &str floor | saved | share of Rust match call | R heap Rust match | R heap &str floor |
|---|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | 761 ns | 677 ns | 83 ns | 11% | 0 B | 0 B |
| choices &str / match_arg enum | "slow" | 1 | 695 ns | 662 ns | 32 ns | 5% | 0 B | 0 B |

</details>

## Appendix B: per-variant summary, chunked run

The chunked-scan run (`summary-chunked.md`): R's `anyNA()` and whole-number expression against the C-API scans, plain and chunked.

<details>
<summary>Per-variant summary</summary>

### Raw per-variant results (median of 3 rep medians; [min-max of rep medians]; trimmed mean; R-heap bytes)


#### group: chunked

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| NA scan dbl | runif(n) | 1 | r_anyNA | 1 ns | 1 ns–1 ns | 10 ns | 0 B |
| NA scan dbl | runif(n) | 1 | raw_capi | 287 ns | 287 ns–328 ns | 305 ns | 0 B |
| NA scan dbl | runif(n) | 1 | raw_chunked | 328 ns | 328 ns–328 ns | 311 ns | 0 B |
| NA scan dbl | runif(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 295 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | r_anyNA | 287 ns | 287 ns–287 ns | 285 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | raw_capi | 574 ns | 574 ns–574 ns | 590 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | raw_chunked | 451 ns | 451 ns–451 ns | 454 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 302 ns | 0 B |
| NA scan dbl | runif(n) | 100000 | r_anyNA | 26.3 µs | 26.3 µs–26.3 µs | 26.3 µs | 0 B |
| NA scan dbl | runif(n) | 100000 | raw_capi | 26.7 µs | 26.7 µs–26.7 µs | 26.7 µs | 0 B |
| NA scan dbl | runif(n) | 100000 | raw_chunked | 13.7 µs | 13.7 µs–13.9 µs | 13.7 µs | 0 B |
| NA scan dbl | runif(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 294 ns | 0 B |
| NA scan dbl | runif(n) | 1e+07 | r_anyNA | 2.64 ms | 2.64 ms–2.65 ms | 2.65 ms | 0 B |
| NA scan dbl | runif(n) | 1e+07 | raw_capi | 2.63 ms | 2.63 ms–2.63 ms | 2.64 ms | 0 B |
| NA scan dbl | runif(n) | 1e+07 | raw_chunked | 1.34 ms | 1.34 ms–1.36 ms | 1.35 ms | 0 B |
| NA scan dbl | runif(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 293 ns | 0 B |
| NA scan int | int(n) | 1 | r_anyNA | 1 ns | 1 ns–1 ns | 8 ns | 0 B |
| NA scan int | int(n) | 1 | raw_capi | 328 ns | 287 ns–328 ns | 310 ns | 0 B |
| NA scan int | int(n) | 1 | raw_chunked | 328 ns | 287 ns–328 ns | 310 ns | 0 B |
| NA scan int | int(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 297 ns | 0 B |
| NA scan int | int(n) | 1000 | r_anyNA | 287 ns | 287 ns–287 ns | 288 ns | 0 B |
| NA scan int | int(n) | 1000 | raw_capi | 328 ns | 328 ns–328 ns | 346 ns | 0 B |
| NA scan int | int(n) | 1000 | raw_chunked | 369 ns | 369 ns–369 ns | 361 ns | 0 B |
| NA scan int | int(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| NA scan int | int(n) | 100000 | r_anyNA | 26.4 µs | 26.4 µs–26.4 µs | 26.4 µs | 0 B |
| NA scan int | int(n) | 100000 | raw_capi | 4.14 µs | 4.14 µs–4.18 µs | 4.18 µs | 0 B |
| NA scan int | int(n) | 100000 | raw_chunked | 5.45 µs | 5.45 µs–5.45 µs | 5.48 µs | 0 B |
| NA scan int | int(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| NA scan int | int(n) | 1e+07 | r_anyNA | 2.65 ms | 2.65 ms–2.66 ms | 2.65 ms | 0 B |
| NA scan int | int(n) | 1e+07 | raw_capi | 386.3 µs | 385.2 µs–387.1 µs | 389.1 µs | 0 B |
| NA scan int | int(n) | 1e+07 | raw_chunked | 509.7 µs | 509.6 µs–509.7 µs | 511.3 µs | 0 B |
| NA scan int | int(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 295 ns | 0 B |
| whole-number scan | as.double(1:n) | 1000 | r_expr | 5.82 µs | 5.78 µs–5.82 µs | 5.85 µs | 35.4 KB |
| whole-number scan | as.double(1:n) | 1000 | raw_capi | 984 ns | 984 ns–1.03 µs | 1.00 µs | 0 B |
| whole-number scan | as.double(1:n) | 1000 | raw_chunked | 779 ns | 738 ns–779 ns | 765 ns | 0 B |
| whole-number scan | as.double(1:n) | 1000 | raw_floor | 410 ns | 410 ns–410 ns | 400 ns | 0 B |
| whole-number scan | as.double(1:n) | 100000 | r_expr | 474.5 µs | 474.5 µs–475.2 µs | 479.8 µs | 3.4 MB |
| whole-number scan | as.double(1:n) | 100000 | raw_capi | 55.2 µs | 55.2 µs–55.3 µs | 55.3 µs | 0 B |
| whole-number scan | as.double(1:n) | 100000 | raw_chunked | 31.8 µs | 31.8 µs–31.8 µs | 31.9 µs | 0 B |
| whole-number scan | as.double(1:n) | 100000 | raw_floor | 410 ns | 410 ns–410 ns | 401 ns | 0 B |
| whole-number scan | as.double(1:n) | 1e+07 | r_expr | 49.11 ms | 49.07 ms–50.17 ms | 51.89 ms | 343.3 MB |
| whole-number scan | as.double(1:n) | 1e+07 | raw_capi | 5.49 ms | 5.49 ms–5.49 ms | 5.49 ms | 0 B |
| whole-number scan | as.double(1:n) | 1e+07 | raw_chunked | 3.14 ms | 3.14 ms–3.14 ms | 3.14 ms | 0 B |
| whole-number scan | as.double(1:n) | 1e+07 | raw_floor | 410 ns | 410 ns–410 ns | 400 ns | 0 B |
| whole-number scan | whole doubles | 1 | r_expr | 205 ns | 205 ns–205 ns | 220 ns | 0 B |
| whole-number scan | whole doubles | 1 | raw_capi | 287 ns | 287 ns–287 ns | 304 ns | 0 B |
| whole-number scan | whole doubles | 1 | raw_chunked | 287 ns | 287 ns–328 ns | 307 ns | 0 B |
| whole-number scan | whole doubles | 1 | raw_floor | 287 ns | 287 ns–287 ns | 293 ns | 0 B |
| whole-number scan | whole doubles | 1000 | r_expr | 4.10 µs | 4.06 µs–4.55 µs | 4.19 µs | 19.7 KB |
| whole-number scan | whole doubles | 1000 | raw_capi | 697 ns | 697 ns–697 ns | 709 ns | 0 B |
| whole-number scan | whole doubles | 1000 | raw_chunked | 492 ns | 492 ns–492 ns | 482 ns | 0 B |
| whole-number scan | whole doubles | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 295 ns | 0 B |
| whole-number scan | whole doubles | 100000 | r_expr | 352.6 µs | 352.5 µs–352.6 µs | 360.9 µs | 1.9 MB |
| whole-number scan | whole doubles | 100000 | raw_capi | 39.6 µs | 39.6 µs–39.6 µs | 39.7 µs | 0 B |
| whole-number scan | whole doubles | 100000 | raw_chunked | 17.8 µs | 17.8 µs–17.8 µs | 17.8 µs | 0 B |
| whole-number scan | whole doubles | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| whole-number scan | whole doubles | 1e+07 | r_expr | 36.99 ms | 36.69 ms–37.96 ms | 42.30 ms | 190.7 MB |
| whole-number scan | whole doubles | 1e+07 | raw_capi | 3.94 ms | 3.94 ms–3.94 ms | 3.95 ms | 0 B |
| whole-number scan | whole doubles | 1e+07 | raw_chunked | 1.75 ms | 1.75 ms–1.75 ms | 1.75 ms | 0 B |
| whole-number scan | whole doubles | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 297 ns | 0 B |

</details>
