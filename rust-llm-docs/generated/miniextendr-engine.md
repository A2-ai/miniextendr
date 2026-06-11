# miniextendr_engine v0.1.0

miniextendr-engine: standalone R embedding for Rust binaries and tests.

This crate centralizes `libR` linking (via `build.rs`), R initialization, and
a minimal runtime handle for processing events and interrupts. It is intended
for Rust-only executables and integration tests that embed R.

**Not for R packages:** this crate uses non-API R internals
(`Rembedded.h`, `Rinterface.h`). For R packages, depend on `miniextendr-api`
and keep `nonapi` disabled.

## When to use
- Rust binaries that embed R.
- Integration tests or benchmarks that need full control over R startup.

## Quick start

```ignore
// SAFETY: Must be called once, from the main thread.
let engine = unsafe {
    miniextendr_engine::REngine::build()
        .with_args(&["R", "--quiet", "--vanilla"])
        .init()
        .expect("Failed to initialize R")
};

// ... use R APIs from the main thread ...

std::mem::forget(engine); // optional: intentionally leak the handle
```

## Initialization details
- Ensures `R_HOME` (via `R RHOME`) if missing.
- Calls `Rf_initialize_R` directly to avoid double `setup_Rmainloop()`.
- Calls `setup_Rmainloop()` exactly once after initialization.

## Runtime sentinel

```ignore
if miniextendr_engine::r_initialized_sentinel() {
    // R has been initialized in this process.
}
```

## Safety

- Must only be initialized once per process.
- Must be called from the main thread.
- No shutdown: `Rf_endEmbeddedR` is intentionally not called because the
  cleanup path is not reentrant-safe. The OS reclaims resources on exit.

---

## Structs

### `REngine`

Handle to an initialized R runtime.

This is a marker type indicating R has been initialized for this process.
R cleanup (via `Rf_endEmbeddedR`) is intentionally NOT called because it
performs non-reentrant operations that can crash if called during Drop
or concurrent with other cleanup. The OS reclaims all resources on process exit.

**Methods:**

#### `build`

```rust
build() -> REngineBuilder
```

Create a new builder for configuring R initialization.

#### `check_interrupt`

```rust
unsafe check_interrupt(self: &Self)
```

Check for user interrupts (Ctrl+C).

##### Safety

Must be called from the thread that initialized R.

#### `process_events`

```rust
unsafe process_events(self: &Self)
```

Process pending R events.

Call this periodically to allow R to handle events, especially
when running a long computation.

##### Safety

Must be called from the thread that initialized R.

### `REngineBuilder`

Builder for configuring and initializing the R runtime.

#### Example

```ignore
let engine = REngine::build()
    .with_args(&["R", "--quiet", "--no-save"])
    .interactive(false)
    .signal_handlers(false)
    .init()?;
```

**Methods:**

#### `init`

```rust
unsafe init(self: Self) -> Result<REngine, REngineError>
```

Initialize the R runtime with the configured settings.

##### Safety

- Must only be called once per process
- Must be called from the main thread
- R cannot be safely shutdown and reinitialized

##### Errors

Returns an error if R initialization fails.

#### `interactive`

```rust
interactive(self: Self, interactive: bool) -> Self
```

Set whether R should run in interactive mode.

Default is `false`.

#### `new`

```rust
new() -> Self
```

Create a new R engine builder with default settings.

#### `r_home`

```rust
r_home(self: Self, path: impl Into<PathBuf>) -> Self
```

Set the R_HOME directory explicitly.

By default, R_HOME is auto-detected by running `R RHOME` or reading
the `R_HOME` environment variable. Use this method to override that
behavior with an explicit path.

##### Example

```ignore
let engine = REngine::build()
    .r_home("/opt/R/4.4.0/lib/R")
    .init()
    .expect("Failed to initialize R");
```

#### `signal_handlers`

```rust
signal_handlers(self: Self, enable: bool) -> Self
```

Set whether R should install signal handlers.

Default is `false`. Set to `true` if you want R to handle Ctrl+C etc.

#### `with_args`

```rust
with_args(self: Self, args: &[&str]) -> Self
```

Set the command-line arguments for R initialization.

Default is `["R", "--quiet", "--vanilla"]`.

---

## Enums

### `REngineError`

Errors that can occur during R engine initialization.

**Variants:**

- `RHomeNotFound { ... }`
  - Could not determine / set `R_HOME` for embedding.
- `InitializationFailed`
  - R initialization failed.
- `AlreadyInitialized`
  - R is already initialized. Re-initialization is not supported.

---

## Functions

### `r_initialized_sentinel`

```rust
r_initialized_sentinel() -> bool
```

Check whether `Rf_initialize_R` has run by inspecting stack sentinels.

`R_CStackStart`/`R_CStackDir` are set during R initialization on the main
thread. A zero or `usize::MAX` value indicates "not initialized".
