# `miniextendr`

Experimental repository.

## Setup / Configuration

For development in this monorepo, run:

```shell
just configure
```

This vendors deps for `rpkg` and runs `rpkg/configure` (autoconf).

## What’s in this repo

- `miniextendr-api/` - runtime crate (R FFI, worker-thread pattern, ALTREP, conversions)
- `miniextendr-macros/` - proc-macros (`#[miniextendr]`, `miniextendr_module!`, `#[r_ffi_checked]`)
- `rpkg/` - example R package that exercises the features

See `docs.md`, `altrep.md`, `COERCE.md`, `THREADS.md`, and `NONAPI.md`.

## Developer configuration

### `justfile`

Use `just` recipes to keep the Rust crates + the example R package in sync:

```shell
just --list
```

Common ones:

```shell
just check
just test
just clippy
just devtools-load
just r-cmd-check
```

## Threading model (two distinct stories)

1) **Recommended (default): worker-thread pattern**

`#[miniextendr]` typically runs Rust code on a worker thread to safely catch panics and run Drops
even when R errors via longjmp. R API calls are marshalled back to the main thread via
`miniextendr_api::worker::with_r_thread`.

2) **Opt-in: calling R from non-main threads**

`miniextendr_api::thread` (feature `nonapi`) disables R’s C stack checking so you can call R off
the main thread. This is sharp: you must still serialize R access (R is not thread-safe). See
`THREADS.md` and `NONAPI.md`.

## Note on release builds

This workspace keeps `debug-assertions = true` in the release profile (see `Cargo.toml`), so
thread checks from `#[r_ffi_checked]` stay on even under `--release`.
