# Worker-thread removal — scoping document

**Date:** 2026-06-14
**Refs:** #989 (and the broader architecture question behind it)
**Status:** Investigation / scoping only. No code changed. The decision and any
implementation remain open.

## The question

> "Do we really need this worker-thread thing? I'd remove it entirely, and stick
> to the solution that Rust errors become R condition objects, and R errors get
> captured then the Rust call terminated early (via panic?) and that error
> surfaced again via the R wrapper."

This document inventories the worker-thread mechanism, asks what it actually buys
today, lays out the maintainer's proposed end-state against what already exists,
sketches a flat removal plan with blast radius, and ends with a direct
recommendation plus the decision points the maintainer must rule on.

**Headline recommendation: remove it.** The worker thread is opt-in
(`#[miniextendr(worker)]`), exercised by exactly one production-shipped function
and a handful of test fixtures, compiled-out on wasm already (a working
proof-of-concept of "no worker"), and uncovered by any CI job. The panic-safety
and error-transport guarantees the maintainer wants are all provided by the
*main-thread* path (`with_r_unwind_protect` + `catch_unwind` +
`extern "C-unwind"`), independent of the worker. Removal dissolves #989 by
construction and collapses two error-transport paths into one. The genuinely
hard part is not the worker itself — it is the `Sendable<T>`/`is_r_main_thread()`
surface that leaked into `externalptr.rs`, `sys.rs`, `error.rs`, `encoding.rs`,
and `pump.rs`, plus the public re-exports and the rayon story.

---

## 1. Inventory

### 1.1 The core mechanism (`miniextendr-api/src/worker.rs`)

The whole worker lives in one file, almost entirely behind
`#[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]`:

- `Sendable<T>` (`worker.rs:80`) — `#[repr(transparent)]` newtype with a blanket
  `unsafe impl<T> Send`. The macro-facing escape hatch for ferrying `!Send`
  values (notably `SEXP`) across the channel.
- `is_r_main_thread()` (`worker.rs:89`) — compares `current().id()` to the
  `OnceLock<ThreadId>` recorded at init.
- `with_r_thread(f)` (`worker.rs:126`) — runs `f()` inline if already on main;
  otherwise routes to main via the channel. Note the cfg at `worker.rs:139` and
  `worker.rs:151`: *without* the feature (or on wasm), calling it from a non-main
  thread is a `panic!`, not a route — because there is nowhere to route to.
- `run_on_worker(f)` (`worker.rs:199`) — dispatches a closure to the worker;
  without the feature (or on wasm) it is literally `Ok(f())` inline
  (`worker.rs:212`).
- `miniextendr_runtime_init()` (`worker.rs:231`) — records the main thread id and
  (feature on, non-wasm) spawns the worker. Without the feature it *only* records
  the id (`worker.rs:280`).
- `miniextendr_runtime_shutdown()` (`worker.rs:305`) — joins the worker; without
  the feature, only uninstalls the panic hook.
- `worker_channel` submodule (`worker.rs:357`–`775`) — the entire bidirectional
  channel machinery: `WorkerMsg`/`WorkerMessage`, the `WORKER` mutex, the
  `dispatch_to_worker` main-thread event loop with its own `R_UnwindProtect`
  trampoline + cleanup handler (`worker.rs:622`–`719`), re-entry guard
  (`worker.rs:513`), and the `#730` main-thread precondition `debug_assert`
  (`worker.rs:529`).

Note that `dispatch_to_worker` *reimplements* the same `R_UnwindProtect`
trampoline/cleanup-handler dance that `unwind_protect.rs` already has — a second,
parallel copy of the hardest code in the runtime (`worker.rs:615`–`719` vs
`unwind_protect.rs:260`–`360`).

### 1.2 The `worker-thread` cargo feature

- Declared `worker-thread = []` (`miniextendr-api/Cargo.toml:128`); `worker-default`
  forwards it to the macros crate and turns it on (`Cargo.toml:129`).
- Mirrored in `rpkg/src/rust/Cargo.toml:65`–`66`, and included in rpkg's `full`
  aggregate (`rpkg/src/rust/Cargo.toml:68`–`74`).
- In `miniextendr-api`, `worker-thread` is pulled into `full-codegen` via
  `worker-default` (`miniextendr-api/Cargo.toml:169`–`177`), and `full-codegen`
  feeds `full` (`Cargo.toml:179`).
- **CI coverage: none.** `grep -n worker .github/workflows/ci.yml` returns zero
  hits. The clippy_all job lints `full-codegen` (`ci.yml:319`), so the worker
  *compiles* under clippy, but no test job runs with `worker-thread` enabled and
  no test exercises a worker-routed `#[miniextendr]` wrapper end-to-end. This is
  the same class of gap MEMORY.md flags for `macro-coverage` ("NO CI job compiles
  it"). The worker's correctness rests on `miniextendr-api/tests/worker_*.rs`
  unit tests, which only run if someone enables the feature locally.

### 1.3 Codegen: how a wrapper chooses main vs worker

This is the load-bearing finding. From `miniextendr-macros/src/lib.rs:910`–`915`:

```rust
let requires_main_thread = returns_sexp || has_sexp_inputs || has_dots || check_interrupt;
let use_main_thread = !force_worker || requires_main_thread;
```

- **The default is main-thread.** A function only runs on the worker if the user
  writes `#[miniextendr(worker)]` (`force_worker`) **and** the signature has no
  SEXP in/out, no dots, no interrupt check.
- "SEXP isn't Send → main thread" is encoded as `has_sexp_inputs || returns_sexp`
  forcing `use_main_thread` regardless of `force_worker`. So *any* function that
  takes or returns a raw `SEXP` can never be a worker function — it is structurally
  forced to main (`lib.rs:914`).
- The worker wrapper (`c_wrapper_builder.rs:470`–`563`) splits conversions:
  SEXP→Rust on main (`pre_closure_stmts`), the call on the worker
  (`run_on_worker`, `c_wrapper_builder.rs:540`), Rust→SEXP back on main via
  `with_r_unwind_protect`. It emits a `compile_error!` if `worker` is requested
  without the feature (`c_wrapper_builder.rs:487`–`490`).
- The default main-thread wrapper (`c_wrapper_builder.rs:437`–`455`) is just
  `with_r_unwind_protect(|| { conversions; call; return_handling }, call)`. No
  worker, no `catch_unwind` at the outer layer (the `catch_unwind` is *inside*
  `with_r_unwind_protect`'s trampoline at `unwind_protect.rs:281`).

### 1.4 Who actually opts in

Across the entire tree, production `#[miniextendr(worker)]` usage is:

- `rpkg/src/rust/rng_tests.rs:55` — `rng_worker_uniform`, itself
  `#[cfg(feature = "worker-thread")]`, i.e. it does not exist in a default build.
- `rpkg/src/rust/worker_tests.rs` — test fixtures (`test_extptr_from_worker`,
  `test_nested_helper_from_worker`, `test_multiple_extptrs_from_worker`),
  referenced in `wasm_registry.rs:3813`/`3873`/`3903` because the registry is
  generated from a worker-enabled native build.

That is the complete production footprint. No class-system method, no conversion,
no DataFrame path, nothing in the cross-package trait ABI uses the worker.

### 1.5 The leaked surface (worker concepts used *outside* worker.rs)

This is where removal gets real, because the worker model bled into the runtime:

- `Sendable<T>` is re-exported publicly (`lib.rs:445`) and used in
  `externalptr.rs:183`/`191`/`198`/`555`/`693`/`702` to batch
  `NonNull<Box<dyn Any>>` / `Vec<SEXP>` ownership transfer, and has an `IntoR`
  impl and `From` for `SEXP` (`into_r.rs:146`/`162`).
- `is_r_main_thread()` is the runtime backbone of the checked-FFI layer:
  `sys.rs:175`/`195`/`210` (the `Rf_error`/`Rf_errorcall`/`Rf_warning` checked
  wrappers panic if off-main), and `error.rs:68`/`88`/`106`/`122`,
  `encoding.rs:57`, `pump.rs:372` (log routing decides main vs queue by it).
- Public re-exports: `pub use worker::{Sendable, is_r_main_thread, with_r_thread}`
  (`lib.rs:445`).

### 1.6 The 16 MB stack constraint

The worker is spawned with a 16 MB stack (documented in `worker.rs:48`). This is
why CLAUDE.md tells you to use `TestRunner { fork: false }` instead of the
`proptest!` macro and keep `Vec<Option<T>>` strategies to ~10 cases — deep
proptest recursion overflows the worker stack. ALTREP callbacks explicitly do
**not** run on the worker (they receive non-Send SEXP args and run on main —
MEMORY.md, `altrep_bridge.rs`).

### 1.7 The wasm story (#758) — a working "no worker" proof-of-concept

#758 (MERGED) keeps `worker-thread` *enabled* on wasm but gates every spawn path
`not(target_family = "wasm")`, so on wasm:

- `miniextendr_runtime_init` falls through to the inline branch and never spawns
  (`worker.rs:236`–`283`).
- `run_on_worker(f)` is `Ok(f())` inline (`worker.rs:210`–`213`).
- `with_r_thread(f)` runs inline (already on the single R thread) — the route
  branch is `unreachable` and compiled out (`worker.rs:137`–`155`).
- The `worker_channel` module is omitted entirely (`worker.rs:357`).

**This is exactly the proposed end-state, already shipping and verified** (rpkg
loads and runs under webR, tier-3 Node). The only reason the feature stays *on*
for wasm is registry parity: `wasm_registry.rs` is generated from a native
worker-enabled build, so disabling the feature would compile out ~31
worker-gated routines and leave dangling registry entries (`miniextendr-api/
CLAUDE.md`, "Don't drop worker-thread on wasm"). That parity constraint
*disappears* if the worker is removed everywhere, because then the native build
generating the registry has no worker-gated routines either.

---

## 2. What does the worker thread actually buy today?

Assessed claim by claim.

### 2.1 Panic safety — NOT provided by the worker

The FFI boundary is `extern "C-unwind"` on every generated wrapper
(`c_wrapper_builder.rs:444`, `:533`). Panic containment comes from `catch_unwind`,
which exists on **both** paths:

- Main-thread path: `with_r_unwind_protect` → `run_r_unwind_protect` → the
  trampoline wraps the closure in `catch_unwind(AssertUnwindSafe(f))`
  (`unwind_protect.rs:281`). Panics become a tagged-condition SEXP
  (`unwind_protect.rs:558`–`615`).
- Worker path: an *additional* outer `catch_unwind` (`c_wrapper_builder.rs:535`)
  plus the worker's own `catch_unwind` inside the job (`worker.rs:575`).

The worker does not add panic safety; it adds a *second* catch layer. The
"unwinding across the FFI boundary is UB" problem was solved by `-C panic=unwind`
+ `extern "C-unwind"` + `catch_unwind`, not by being on another thread. This is
exactly extendr's model: extendr runs user code on R's own thread and wraps it in
`catch_unwind` at the `#[extendr]` boundary; there is no worker thread.

### 2.2 R-error capture — already main-thread, already what the maintainer describes

`unwind_protect.rs` *is* the "capture R error, terminate Rust early, resurface in
the wrapper" mechanism, and it runs on main:

- `R_UnwindProtect_C_unwind` with a cleanup handler (`unwind_protect.rs:293`–`317`).
- On R longjmp the cleanup handler `panic_any(RErrorMarker)` (`:296`), which
  unwinds Rust frames (running destructors), then `run_r_unwind_protect` detects
  the marker and calls `R_ContinueUnwind` to resume R's unwind to top level
  (`unwind_protect.rs:346`–`351`).

The maintainer's "R errors get captured then the Rust call terminated early (via
panic?)" is *precisely* this, and it has nothing to do with the worker. (The
worker's `dispatch_to_worker` has its own copy of this dance at
`worker.rs:643`–`719` purely to ferry the error back across the channel — which
is the source of #989, see §2.4.)

### 2.3 Rust-errors-as-conditions — already main-thread

The tagged-SEXP transport is fully main-thread:
`make_rust_condition_value` (`error_value.rs`), the `RCondition` enum +
`error!`/`warning!`/`message!`/`condition!` (`condition.rs`), recognised in
`with_r_unwind_protect` (`unwind_protect.rs:566`–`601`) and re-raised by the
generated R wrapper as `stop(structure(..., class = c("rust_error", ...)))`. No
worker involvement.

### 2.4 Does anything *require* a separate OS thread? No — and the worker can't even call R

The decisive point is `docs/THREADS.md`: R's C API checks `R_CStackStart`/
`R_CStackLimit` against the current stack on most API calls, so **calling any R
API from a non-main thread segfaults**. The worker therefore *cannot* call R. It
proves this itself: to touch R, worker code must call `with_r_thread`, which
routes the closure **back to the main thread** (`worker.rs:153`,
`worker.rs:464`–`494`). So R work is always on main either way.

What the worker buys is narrow: pure-Rust compute that (a) the author explicitly
opts into with `worker`, (b) has no SEXP in its signature, and (c) wants run on a
thread other than R's so a panic unwinds Rust frames "off to the side." But
`catch_unwind` on the main thread already contains the panic safely, so even this
is not a hard requirement — it is a stylistic isolation that the default path
provides without a thread.

**Verdict: the worker thread is vestigial.** It is a second, parallel
implementation of panic-catching and R-unwind-capture that the main-thread path
already does, reachable only by explicit opt-in, structurally barred from the
common SEXP-carrying case, and the cause of the one bug (#989) that the redesign
would dissolve.

---

## 3. The proposed end-state

Concretely, the target is what wasm already runs (§1.7) but on every target:

| Concern | Mechanism | Already this way? |
|---|---|---|
| Rust runs on R's main thread | direct call inside the C wrapper | **Yes** for all default/SEXP/dots functions; only `worker` opt-ins differ |
| Panic caught at the boundary | `extern "C-unwind"` + `catch_unwind` in `with_r_unwind_protect` | **Yes** (`unwind_protect.rs:281`) |
| Rust error → R condition | tagged-SEXP transport, R wrapper raises `stop(structure(...))` | **Yes** (`unwind_protect.rs:558`, `error_value.rs`) |
| R error → capture + early-terminate Rust + resurface | `R_UnwindProtect` + cleanup-handler `panic_any` + `R_ContinueUnwind` | **Yes** (`unwind_protect.rs:293`–`351`) |
| `with_r_thread` from user code | runs inline (already on main) | becomes a trivial identity / can be removed |
| `run_on_worker` | runs inline `Ok(f())` | already the no-feature behaviour; can be removed |

So the proposed end-state is **already the behaviour of a no-`worker-thread`
build, and already the behaviour on wasm regardless of feature.** Removal is
mostly deletion of the alternate branch plus untangling the leaked surface
(§1.5), not new design.

---

## 4. Removal plan (flat priority order)

1. **Delete the codegen branch.** Remove `force_worker` / `use_main_thread` /
   `requires_main_thread` logic (`lib.rs:910`–`915`) and
   `generate_worker_thread_wrapper` + `generate_worker_return_handling`
   (`c_wrapper_builder.rs:470`–`563`, `:748`+). Drop the `worker` attribute from
   the parser (`miniextendr_fn.rs:1070`, `:1217`, `:1269`–`1310`) and from
   `miniextendr_impl_trait.rs:128`. Every `#[miniextendr]` then emits the
   main-thread wrapper unconditionally. **Blast radius:** macro tests, the
   `compile_error!` UI snapshot for "worker requires feature" (regenerate
   `.stderr`). This is the largest single deletion and the highest-leverage step.

2. **Resolve `Sendable<T>`.** It is genuinely used by `externalptr.rs` to batch
   ownership transfer (`:183`/`:702`) — but in a no-worker world nothing crosses a
   thread, so `Sendable` becomes a no-op wrapper. Either (a) delete it and inline
   the raw `NonNull`/`Vec` (preferred — "no backwards compat" principle), or (b)
   keep a private no-op shim if the batching code reads cleaner with it. Also drop
   the public re-export (`lib.rs:445`) and the `IntoR`/`From` impls
   (`into_r.rs:146`/`162`) if unused after. **Blast radius:** `externalptr.rs`
   (6 sites), `into_r.rs`, any downstream that imported `Sendable` (none in-tree
   besides re-export). **Ordering: do this before #4** so the externalptr edits
   land against a stable API.

3. **Delete `worker.rs`'s worker machinery.** Remove `run_on_worker`,
   `with_r_thread`'s routing branch, `worker_channel`, `init_worker`,
   `worker_loop`, `dispatch_to_worker` (and its duplicate `R_UnwindProtect`
   dance), `has_worker_context`, the `WorkerMsg`/`WorkerMessage` types. Keep
   `miniextendr_runtime_init`/`_shutdown` as thin functions (init records main id
   + installs panic hook; shutdown uninstalls it) — they are still called from
   `R_init_<pkg>` / `R_unload_<pkg>` via `package_init` (`init.rs`). **Blast
   radius:** `init.rs`, `worker_*.rs` integration tests (delete), rpkg
   `test-worker.R` / `test-worker-longjmp.R` (delete or repurpose to assert the
   main-thread longjmp path).

4. **Collapse `is_r_main_thread()`.** With no worker, the only thread that ever
   reaches the runtime is main. The checked-FFI guards in `sys.rs:175`/`195`/`210`
   and the branches in `error.rs`, `encoding.rs`, `pump.rs:372` become either
   always-true (so the guard is dead) or should degrade to `debug_assert!` to
   still catch a user who spawns their *own* `std::thread` and calls R from it
   (which `THREADS.md` says segfaults anyway). **Decision point** — see §5.
   Keep `is_r_main_thread()` itself (cheap, useful as a `debug_assert` and for
   rayon, §5), but it may no longer need to be `pub`.

5. **Remove the `worker-thread` / `worker-default` features.** Delete from
   `miniextendr-api/Cargo.toml:128`–`129`, `rpkg/src/rust/Cargo.toml:65`–`66` and
   the `full`/`full-codegen` aggregates (`Cargo.toml:169`–`179`,
   `rpkg/.../Cargo.toml:68`–`74`). **Blast radius:** `docs/FEATURE_DEFAULTS.md`,
   `docs/FEATURES.md`, CI's `clippy_all` derives `full-codegen` so it follows
   automatically (no manual ci.yml feature edit needed, by design — #677).

6. **Delete worker-only fixtures + regenerate the wasm registry.** Remove
   `rpkg/src/rust/worker_tests.rs` and `rng_worker_uniform`
   (`rng_tests.rs:48`–`58`). Then regenerate `wasm_registry.rs` from the now
   worker-free native build — the ~31 worker-gated routines simply stop existing,
   and the §1.7 parity constraint evaporates. Run `just configure &&
   just rcmdinstall && just force-document` to regenerate wrappers/NAMESPACE/man.

7. **Docs + skills sweep.** `docs/THREADS.md` (rewrite around "R is single-thread;
   we run on it; here's how to use your own threads safely"), `docs/RAYON.md`
   (the `with_r_thread`-inside-rayon guidance changes — §5), `docs/ARCHITECTURE.md`,
   `docs/SAFETY.md`, `docs/FFI_GUARD.md`, `docs/ERROR_HANDLING.md`,
   `docs/CONDITIONS.md`. Delete/rewrite `.claude/skills/miniextendr-worker/SKILL.md`
   and de-reference it from the worker mentions in the `externalptr`/`altrep`/
   `macros`/`ffi`/`lint` skills. Update root `CLAUDE.md` ("Worker thread: Rust
   runs on a worker thread for panic safety" is now false) and
   `miniextendr-api/CLAUDE.md`.

8. **Close #989** (dissolved) and the MXL301 wording that lists `with_r_thread`
   as a safe `_unchecked` context (`miniextendr-lint`, `sys.rs` doc, `lib.rs:1629`)
   — `with_r_thread` bodies stop existing, leaving ALTREP callbacks and
   `with_r_unwind_protect` as the two remaining safe contexts.

**Genuinely hard parts / ordering constraints.** The hard parts are #2 (Sendable
is real, not vestigial — externalptr's batched ownership transfer relies on it
crossing the macro's `run_on_worker` closure boundary today; once that boundary is
gone the `Send` requirement evaporates, but the edit must be careful about
provenance per the `cached_ptr` rule) and #4 (the `is_r_main_thread` guards are a
real safety net for users who spawn their own threads — don't just delete them,
demote them). Order is strict: 1 → 2 → 3 (3 depends on 1+2 removing all callers
of `run_on_worker`/`with_r_thread`/`Sendable`), then 4, 5, 6 (6 must follow 1 so
the registry regenerates worker-free), then 7, 8.

---

## 5. Risks & open questions / decision points

**R1 — Stack size.** The worker has a 16 MB stack (`worker.rs:48`). R's main
thread stack is governed by `R_CStackLimit` (`THREADS.md`) — typically 8 MB
default `ulimit -s` on Linux, but R reserves a margin and *checks* it, where the
worker's 16 MB was unchecked headroom. Removing the worker means deep Rust
recursion runs on R's checked stack and can trip `R_SignalCStackOverflow` (a
clean R error) rather than a hard crash — arguably *better* behaviour, but a
behavioural change. The proptest guidance in CLAUDE.md (fork:false, ~10 cases)
was a worker-stack workaround; on main it is governed by R's limit instead.
*Mitigation:* none needed for production code; update the proptest note.

**R2 — `is_r_main_thread` guards: delete or demote? (DECISION POINT 1.)**
The checked-FFI panics (`sys.rs:175`+) and log-routing branch (`pump.rs:372`)
exist to catch off-main R calls. With no worker, the only way to be off-main is a
user-spawned thread (rayon/std::thread) — which `THREADS.md` says segfaults in R
anyway. Options: (a) keep them as `debug_assert!` safety nets (cheap, catches the
rayon footgun early); (b) delete them as dead weight. Recommend (a) — they cost
one atomic load and turn a segfault into a panic for the rayon misuse case.

**R3 — Rayon / user threads. (DECISION POINT 2.)** `docs/RAYON.md:603` already
documents "with_r_thread inside a rayon closure PANICS." In the worker world, a
`#[miniextendr(worker)]` fn could call `with_r_thread` to route rayon results back
to main. Without the worker, `with_r_thread` from a non-main thread has nowhere to
route — it must panic (as it does today without the feature, `worker.rs:139`).
**Question for the maintainer:** is there any real use case where Rust code on a
*user* thread needs to call back into R? If yes, that is the ONE capability the
worker provided that nothing else does — but note it currently only works for
`worker`-opted functions, is unused in production, and the rayon docs already warn
against it. Recommend: drop it; document "collect rayon results to owned Rust
data, return to main, convert there" (which RAYON.md already teaches).

**R4 — `Sendable<T>` semantics. (DECISION POINT 3.)** Delete entirely (inline raw
pointers/Vec in externalptr.rs) vs keep a private no-op shim. Recommend delete per
"no backwards compat," but it touches pointer-provenance-sensitive code
(`externalptr.rs:693`–`702`) so it needs a `gctorture(TRUE)` pass (the
`gc_stress_*` convention) after the edit.

**R5 — #277 / Windows DLL-unload.** The worker's careful synchronous shutdown
(`worker.rs:415`–`433`, the `WorkerMsg::Shutdown` design) exists *because* the
worker thread could resume in unmapped DLL pages after `library.dynam.unload`. No
worker → this entire class of bug is gone, and `miniextendr_runtime_shutdown`
simplifies to just the panic-hook uninstall. The `atexit` discussion
(`worker.rs:260`–`274`) becomes moot. Net safety *improvement*.

**R6 — Cross-package trait ABI.** Trait shims use `with_r_unwind_protect_shim`
(`unwind_protect.rs:483`) and the View boundary re-panic — all main-thread,
no worker. Removing the worker does not touch the trait ABI. Confirmed: no
`run_on_worker`/`with_r_thread` in `miniextendr_trait.rs` codegen paths.

**R7 — #989 / #931 / #345 interaction.** #989 (bare `simpleError` from worker
jobs) exists *only* because `dispatch_to_worker`'s cleanup handler resumes the R
longjmp via `R_ContinueUnwind` (`worker.rs:712`) directly to top level, bypassing
the tagged-condition transport — `run_on_worker` never returns, so the wrapper
can't re-tag. **No worker boundary → no bypass → #989 cannot exist.** #931 (the
~8-byte leak + re-usability characterisation) was worker-longjmp-specific and
becomes irrelevant. #345 (route trait-ABI/ALTREP panics through tagged path) is
*orthogonal and already merged* — it concerns the no-R-wrapper guard sites, not
the worker.

**R8 — `miniextendr_runtime_init` contract.** It still must run from
`R_init_<pkg>` to record the main thread id (used by R2's debug_asserts and rayon
safety). Keep it; it just stops spawning. **Decision point (minor):** if all
`is_r_main_thread` guards are deleted (R2 option b), even recording the id becomes
optional — but keeping it is near-free and preserves the rayon safety net.

---

## 6. Recommendation

**Remove the worker thread.** This is the rare case where the maintainer's
instinct and the evidence align cleanly:

- **It is vestigial.** Every guarantee the framework advertises — panic safety,
  Rust-errors-as-conditions, R-error capture-and-resurface — is delivered by the
  main-thread `with_r_unwind_protect` + `catch_unwind` + `extern "C-unwind"` path
  (§2). The worker is a second, parallel implementation of the same two
  mechanisms, plus a thread that *cannot even call R* (§2.4).
- **It is barely used and uncovered.** One `#[cfg]`-gated production function, a
  few test fixtures, zero CI jobs (§1.2, §1.4). The risk of removing untested code
  is far lower than the risk of keeping an untested concurrency mechanism in a
  framework whose entire safety story is "R is single-threaded."
- **It is already removed on wasm** and that build works (§1.7). The proposed
  end-state is not speculative; it ships today on one target.
- **It dissolves #989** by construction and unifies error transport on one path
  (§5/R7), removing the duplicate `R_UnwindProtect` dance in `worker.rs`.

**Complexity / LoC delta (estimate):** net deletion. `worker.rs` is ~945 lines,
of which the worker machinery is ~600 (the rest — `Sendable`, `is_r_main_thread`,
init/shutdown stubs — partly survives). Add the codegen branch (~150 lines of
`generate_worker_thread_wrapper` + return-handling + parser), the worker
integration tests (~2 files), `worker_tests.rs`, two rpkg test files, and the
feature plumbing. Conservatively **~1,200–1,500 lines deleted**, against perhaps
~100 lines of edits to demote `is_r_main_thread` guards and rewrite docs. The
duplicate `R_UnwindProtect` trampoline collapses to one copy. The framework's
threading mental model collapses from "two-thread with a routing channel" to
"single-thread, like extendr" — a large conceptual simplification that the docs,
skills, and CLAUDE.md currently spend significant prose defending.

**What to be careful about:** the `Sendable`/`externalptr` provenance edit (R4,
needs gctorture) and the `is_r_main_thread` demotion (R2, keep as debug_assert).
Neither is a blocker; both are well-scoped.

**Top 3 decision points for the maintainer:**
1. `is_r_main_thread` checked-FFI guards — demote to `debug_assert!` (recommended)
   or delete? (R2)
2. Is "call R from a user-spawned (rayon/std) thread, routed back to main" a real
   requirement worth preserving? (R3) — recommend no.
3. `Sendable<T>` — delete and inline, or keep as a private no-op shim? (R4) —
   recommend delete.

**Biggest risk:** R2/R3 combined — silently dropping the off-main-thread *safety
net*. Today, a user who spawns a rayon thread and calls an R API gets a clean
panic from the `sys.rs` checked guard; if removal deletes those guards rather than
demoting them, that footgun becomes a segfault (`R_CheckStack` false overflow per
`THREADS.md`). The mitigation is trivial — keep `is_r_main_thread()` and demote
the guards to `debug_assert!` — but it must be a conscious decision, not an
incidental casualty of deleting the worker.
