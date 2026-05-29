# Tier-3 webR Node smoke hangs after a successful run (exit 124)

**PR:** #750 (`feat/webr-tier3-node-smoke`) — closes #492
**Date:** 2026-05-29

## What was attempted

Add a tier-3 step to the `webr-install` job that boots a webR Node session
against the wasm install tier-2 produced and drives `library(miniextendr)`:
`timeout 900 node smoke.mjs`.

## What went wrong

The job failed with **exit code 124** (the `timeout` SIGTERM code). The smoke
runner itself succeeded — the CI log shows the full happy path:

```
[tier3] OK: library(miniextendr) loaded.
[tier3] miniextendr version: 0.1.0
[tier3] Tier-3 PASSED.
##[error]Process completed with exit code 124.
```

`Tier-3 PASSED` printed at 17:26:41; the process was then killed at 17:41:30 —
exactly 900s after the step started. So the runner did all its work and then
**hung for the full timeout** instead of exiting.

## Root cause

`smoke.mjs`'s `main()` resolved but the Node process never exited. webR boots a
dedicated worker via Node `worker_threads` to host the R runtime; that worker
(plus the channel listeners and the open NODEFS mount) keeps Node's event loop
alive after `main()` returns. The old tail —

```js
main().catch((e) => { ...; process.exitCode = 1; });
```

— set `process.exitCode` but never terminated the worker and never called
`process.exit()`, so Node stayed up until `timeout` killed it. This is purely a
process-lifecycle bug; it has nothing to do with whether miniextendr loads (it
did).

## Fix

Terminate webR and exit explicitly in a top-level `.finally()`:

```js
let webR;                       // module-scoped so teardown sees it on error paths
...
main()
  .catch((e) => { ...; process.exitCode = 1; })
  .finally(() => {
    if (webR) { try { webR.close(); } catch {} }   // worker.terminate()
    process.exit(process.exitCode ?? 0);
  });
```

`webR.close()` → `worker.terminate()` releases the loop-keeper; the explicit
`process.exit()` guarantees a prompt, deterministic exit on both success and
error paths regardless of any residual handles. `close()` is wrapped in
try/catch so the exit is robust even if it throws or is absent.

## Verification (arm64-native, no emulation)

The CI image (`ghcr.io/a2-ai/webr-mirror`, == upstream `ghcr.io/r-wasm/webr`)
is amd64-only; running it under qemu on the arm64 dev box was rejected as
unacceptable. webR's Node runtime is JS + portable wasm, so it runs natively on
arm64 via the npm `webr` package — no container, no emulation.

A harness mirroring `smoke.mjs`'s boot → `installPackages` → `library` → exit
shape (with `cli` standing in for the CI-proven miniextendr load) was run with
both tails:

| Tail | Behaviour | Exit | Wall |
|------|-----------|------|------|
| old (no close/exit) | does all work, then **hangs** (watchdog-killed) | **124** | 45s (cap) |
| new (close + exit) | does all work, exits cleanly | **0** | ~3s |

The old tail reproduces the CI symptom exactly (work completes, then exit 124).
The new tail is the fix.

## Note on the "full" local run

Running the *literal* `library(miniextendr)` locally would need a wasm-built
miniextendr, which needs a webR cross-toolchain (WEBR_ROOT). The vendored
`.webr` checkout (v0.5.8) only builds from source (`make webr` — multi-hour,
downloads a full emscripten toolchain) and the host emcc / npm-webr versions
don't match it, so a from-source build risks side-module ABI skew. Since the
wasm *load* is already green on this exact branch's CI run and is orthogonal to
the exit-lifecycle bug, that build was deliberately not undertaken: fixed exit
path + already-green load ⇒ the runner passes.
