# Progress Bars in R via indicatif

miniextendr provides an adapter that routes [indicatif](https://docs.rs/indicatif) progress bar output through R's console, so progress bars render correctly in R terminals, RStudio, and other R frontends.

**Feature flag:** `indicatif` (implies `nonapi`)

```toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["indicatif"] }
```

## Table of Contents

- [Why a Custom Terminal Adapter](#why-a-custom-terminal-adapter)
- [Quick Start](#quick-start)
- [RTerm and RStream](#rterm-and-rstream)
- [Convenience Constructors](#convenience-constructors)
- [Refresh Rate Tuning](#refresh-rate-tuning)
- [Main Thread Safety](#main-thread-safety)
- [ANSI Cursor Support](#ansi-cursor-support)
- [Complete Example](#complete-example)
- [Limitations](#limitations)

## Why a Custom Terminal Adapter

R's console output is **not** stdout or stderr. R uses its own output functions (`Rprintf`, `REprintf`, or the `ptr_R_WriteConsoleEx` hook) to route text to the active frontend (terminal, RStudio, Jupyter, etc.). Writing directly to stdout/stderr would bypass R entirely -- output might not appear in RStudio's console, or could interleave incorrectly with R's own output.

indicatif's `ProgressBar` needs a `TermLike` implementation to know how to write text, move the cursor, and clear lines. `RTerm` implements `TermLike` by routing all output through R's console hooks, making indicatif work seamlessly inside R.

## Quick Start

```rust
use miniextendr_api::progress::term_like_stderr;
use indicatif::{ProgressBar, ProgressStyle};
use miniextendr_api::miniextendr;

#[miniextendr]
pub fn process_data(x: &[f64]) -> Vec<f64> {
    let pb = ProgressBar::with_draw_target(x.len() as u64, term_like_stderr(80));
    pb.set_style(
        ProgressStyle::with_template("{bar:40} {pos}/{len} [{elapsed_precise}]").unwrap()
    );

    let result: Vec<f64> = x.iter().map(|&val| {
        pb.inc(1);
        val.sqrt()
    }).collect();

    pb.finish_with_message("done");
    result
}
```

From R:

```r
result <- process_data(runif(100000))
#> ======================================== 100000/100000 [00:00:01]
```

## RTerm and RStream

### RStream

`RStream` selects the output target:

```rust
pub enum RStream {
    Stdout,  // R's standard output (otype = 0)
    Stderr,  // R's standard error (otype = 1)
}
```

Use `Stderr` for progress bars (the conventional choice), so they don't interfere with data piped to stdout.

### RTerm

`RTerm` is the `TermLike` implementation. It takes a stream and a fixed column width:

```rust
use miniextendr_api::progress::{RTerm, RStream};
use indicatif::ProgressDrawTarget;

let term = RTerm::new(RStream::Stderr, 80);
let target = ProgressDrawTarget::term_like(Box::new(term));
```

The `width` parameter controls how indicatif formats the progress bar. Use 80 as a safe default. If you need dynamic width detection, query `getOption("width")` from R and pass it in.

### Output Routing

`RTerm` uses the following strategy:

1. **Primary:** `ptr_R_WriteConsoleEx` -- the hook used by R frontends (RStudio, Jupyter, etc.)
2. **Fallback:** `Rprintf` / `REprintf` -- standard R output functions (used when the hook is unavailable)

All output is chunked to respect `i32::MAX` byte limits per call, and NUL bytes are stripped from the fallback path.

## Convenience Constructors

For common use cases, these functions create `ProgressDrawTarget` values directly:

```rust
use miniextendr_api::progress::*;

// Basic targets (default refresh rate)
let target = term_like_stdout(80);
let target = term_like_stderr(80);

// With custom refresh rate
let target = term_like_stdout_with_hz(80, 10);
let target = term_like_stderr_with_hz(80, 10);

// Full control
let target = term_like_with_hz_and_stream(RStream::Stderr, 120, 5);
```

All return `ProgressDrawTarget`, ready to pass to `ProgressBar::with_draw_target()`.

## Refresh Rate Tuning

The `_with_hz` variants let you control how often indicatif redraws the progress bar:

```rust
// Update at 5 Hz (every 200ms) - less overhead for slow connections
let target = term_like_stderr_with_hz(80, 5);

// Update at 20 Hz (every 50ms) - smoother appearance for fast operations
let target = term_like_stderr_with_hz(80, 20);
```

| Scenario | Suggested Hz |
|----------|-------------|
| Long-running computation (minutes) | 2-5 |
| Medium operations (seconds) | 10 (default) |
| Fast operations where smoothness matters | 15-20 |
| Remote / slow terminal connections | 2-3 |

Higher refresh rates mean more R console calls, which adds overhead. For tight loops where each iteration is very fast, consider using `pb.inc(batch_size)` to update in batches rather than increasing the refresh rate.

## Main Thread Safety

All `RTerm` output is **silently dropped** when called from a non-main thread:

```rust
fn write_console(&self, bytes: &[u8]) -> io::Result<()> {
    if !crate::worker::is_r_main_thread() {
        return Ok(());  // Silent no-op
    }
    // ... actual output ...
}
```

This means:
- Progress bars drawn from **non-main threads** (worker thread, rayon threads) produce no output.
- Only code running on the **main R thread** can display progress.

For `#[miniextendr]` functions, this works naturally -- by default your function runs on the
main thread, so progress bar output works directly. If using the `worker-thread` feature,
operations that need R console access should use `unsafe(main_thread)` or run the progress
bar update on the main thread. In practice, since indicatif batches draws, the simplest
approach is to create the progress bar and call `pb.inc()` from your function directly.

See [THREADS.md](THREADS.md) for details on the worker thread model.

## ANSI Cursor Support

`RTerm` implements all cursor movement operations via ANSI escape sequences:

| Operation | ANSI Sequence | Method |
|-----------|--------------|--------|
| Move cursor up N lines | `\x1b[NA` | `move_cursor_up(n)` |
| Move cursor down N lines | `\x1b[NB` | `move_cursor_down(n)` |
| Move cursor right N columns | `\x1b[NC` | `move_cursor_right(n)` |
| Move cursor left N columns | `\x1b[ND` | `move_cursor_left(n)` |
| Clear current line | `\r\x1b[2K` | `clear_line()` |

These work in terminals that support ANSI escape codes (most modern terminals, including macOS Terminal, iTerm2, and the RStudio terminal pane). The `flush()` method calls `R_FlushConsole()` to ensure output is displayed immediately.

## Complete Example

A `#[miniextendr]` function with a styled progress bar:

```rust
use miniextendr_api::miniextendr;
use miniextendr_api::progress::term_like_stderr;
use indicatif::{ProgressBar, ProgressStyle};

#[miniextendr]
pub fn compute_with_progress(n: i32) -> Vec<f64> {
    let n = n as usize;
    let pb = ProgressBar::with_draw_target(n as u64, term_like_stderr(80));
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})"
        )
        .unwrap()
        .progress_chars("=>-")
    );

    let mut results = Vec::with_capacity(n);
    for i in 0..n {
        // Simulate expensive computation
        let value = ((i as f64) * 0.001).sin().powi(2);
        results.push(value);
        pb.inc(1);
    }

    pb.finish_with_message("computation complete");
    results
}
```

From R:

```r
result <- compute_with_progress(1000000L)
#> / [00:00:02] [========================================] 1000000/1000000 (0s)
```

### Multi-Progress Bar Example

indicatif's `MultiProgress` works too:

```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use miniextendr_api::progress::{RTerm, RStream};

#[miniextendr]
pub fn parallel_progress() {
    let term = RTerm::new(RStream::Stderr, 80);
    let mp = MultiProgress::with_draw_target(
        indicatif::ProgressDrawTarget::term_like(Box::new(term))
    );

    let style = ProgressStyle::with_template("{prefix:.bold} {bar:30} {pos}/{len}").unwrap();

    let pb1 = mp.add(ProgressBar::new(100));
    pb1.set_style(style.clone());
    pb1.set_prefix("Task 1");

    let pb2 = mp.add(ProgressBar::new(200));
    pb2.set_style(style);
    pb2.set_prefix("Task 2");

    for i in 0..200 {
        if i < 100 { pb1.inc(1); }
        pb2.inc(1);
    }

    pb1.finish();
    pb2.finish();
}
```

## Limitations

- **Main thread only.** Output from non-main threads is silently dropped. This is by design -- R's console functions are not thread-safe.
- **Fixed width.** `RTerm` uses a fixed column width (no dynamic terminal size detection). Pass the desired width when constructing.
- **ANSI support varies.** RStudio's console pane has limited ANSI support. Multi-line progress bars may not render correctly in all R frontends. Single-line progress bars work everywhere.
- **No input.** `RTerm` is output-only. It cannot read user input (e.g., for interactive prompts).
- **Non-interactive sessions.** In `Rscript` or batch mode (`R CMD BATCH`), console output may be buffered or suppressed depending on the frontend. Progress bars still work but may not display in real-time.
- **Implies `nonapi`.** The `indicatif` feature enables the `nonapi` feature because `ptr_R_WriteConsoleEx` is a non-API symbol. This means `R CMD check` will note the use of non-API entry points.

## See Also

- [FEATURES.md](FEATURES.md) -- Feature flags reference (`indicatif`)
- [THREADS.md](THREADS.md) -- Worker thread architecture and main thread safety
- [indicatif documentation](https://docs.rs/indicatif) -- Full indicatif API reference
