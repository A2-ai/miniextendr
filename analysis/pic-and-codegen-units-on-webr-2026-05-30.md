# PIC, `relocation-model`, and `codegen-units = 1` under webR — a full audit

**Date:** 2026-05-30
**Scope:** Two questions, both about flags miniextendr does (or doesn't) pass to `rustc` when building for `wasm32-unknown-emscripten` / webR.

1. *Is PIC necessary in webR?* — and why did we drop `-C relocation-model=pic` (#745).
2. *Why do we have `codegen-units = 1`, and is it necessary for webR support?*

Everything below is backed by primary sources: the R source tree (`r-devel/r-svn`),
the webR build system (`r-wasm/webr`), the webR package builder (`r-wasm/rwasm`),
savvy (`yutannihilation/savvy`), and reproducible `rustc` / `emcc` runs on this
machine (`rustc 1.95.0` stable, `1.97.0-nightly`; `emcc 5.0.7-git`).

---

## TL;DR — the verdict

| Claim | Verdict | One-line reason |
|---|---|---|
| "PIC is not necessary in webR" | **False as stated, true with one word changed.** | webR *is* a dynamic-linking environment (MAIN_MODULE + SIDE_MODULE); PIC is **foundational** to it. What's unnecessary is the **explicit Rust `-C relocation-model=pic` flag**. |
| `-C relocation-model=pic` RUSTFLAG | **Verified no-op. Correctly dropped (#745/#749).** | `wasm32-unknown-emscripten` already compiles at **PIC Level 2** by default. Adding the flag produces byte-identical LLVM IR. |
| `-Zdefault-visibility=hidden` RUSTFLAG | **Necessary. Keep it.** | Without it, `-s SIDE_MODULE=1` exports ~3000 mangled Rust symbols and webR's `dyn.load` chokes. Distinct from PIC. |
| `codegen-units = 1` | **Necessary on *native* (linkme). Inert / unnecessary for *webR*.** | It exists to stop the linker dropping unreferenced linkme `#[distributed_slice]` archive members. On wasm there is no linkme — the registry is reached by ordinary symbol references — so the failure mode it guards against cannot occur. |

**Bottom line for the comment you pasted:** the comment is *accurate about why the
flag exists* (native linkme archive-member selection). It is **not** evidence that
`codegen-units = 1` is needed for webR. It isn't. It's set unconditionally via
`[profile.*]`, so it also applies to the wasm build, where it is harmless and inert.

---

## 0. Disambiguating "PIC" (this is the whole ballgame)

"PIC" gets used for two different things and the question collapses them:

- **PIC the model** — code that accesses globals and makes calls *relative to a base
  that's only known at load time*, so the module can be mapped anywhere. This is what
  every shared library is. **webR depends on it completely** (next sections).
- **`-C relocation-model=pic` the Rust flag** — an *explicit request* to rustc to emit
  PIC. The question of whether *we need to pass it* is separate from whether PIC
  *happens*. The answer is no, because the emscripten target already emits PIC without
  being asked.

Keep these apart and the entire confusion dissolves.

---

## 1. PIC 101 and why native R cannot live without it

An R package's compiled code ships as a shared object — `.so` (Linux), `.dylib`
(macOS), `.dll` (Windows) — that R loads at runtime with `dyn.load()` into the address
space of an already-running R process. The OS dynamic loader maps it at whatever base
address is free (ASLR, other libraries already loaded). For that to work, the code
cannot contain absolute addresses baked in at compile time; it must reference its own
globals and functions *relative to a runtime base*. That is Position-Independent Code.

R bakes this into its build configuration. From the R source tree:

```
background/r-svn/configure.ac:1438   cpicflags="-fPIC"
background/r-svn/configure.ac:1528   darwin_pic="-fPIC"
background/r-svn/configure.ac:1714   fpicflags="-fPIC"
background/r-svn/configure.ac:1930   AC_SUBST(FPICFLAGS)
background/r-svn/Makeconf.in:41      FPICFLAGS = @FPICFLAGS@
background/r-svn/etc/Makeconf.in:179 ALL_FFLAGS = ... $(FPICFLAGS) ...
```

`configure` probes the platform, picks `-fPIC` (or `-fpic` where the GOT is small
enough), and threads `FPICFLAGS` into every `ALL_*FLAGS` that compiles package code.
**On every native platform R targets, PIC is mandatory and non-negotiable.** It is the
price of `dyn.load`.

The mental model to carry into webR: *nothing about webR removes the `dyn.load` model.*
R still loads each package as a separate shared object at runtime. So the demand for PIC
doesn't go away — it just gets satisfied by a different toolchain.

---

## 2. WebAssembly has two linking models, and only one needs PIC

WebAssembly has no absolute code addresses. Functions are referenced by *index* into a
module's table; the linker (`wasm-ld`) assigns those indices. There are two ways to
produce a final artifact:

### 2a. Static linking — the default, one self-contained module
Everything is resolved at link time into a single `.wasm`. No runtime relocation, no
GOT, no base register. This is what `wasm32-unknown-unknown` targets, and its Rust
target spec says so explicitly:

```
$ rustc +nightly -Z unstable-options --target wasm32-unknown-unknown --print target-spec-json | grep relocation
  "relocation-model": "static",
```

Static modules don't need PIC because nothing moves.

### 2b. Dynamic linking — Emscripten `MAIN_MODULE` + `SIDE_MODULE`
Here a *main* module (`MAIN_MODULE`) is loaded first, and *side* modules
(`SIDE_MODULE`) are `dlopen`'d at runtime. A side module does **not** know, at compile
time, where its data will live in the main module's linear memory, nor what table
indices its functions will get. So it:

- imports `__memory_base` and `__table_base` (the runtime offsets for its data and
  table),
- accesses its globals through the **GOT** (`GOT.mem` / `GOT.func` imports), and
- calls through the shared `__indirect_function_table`.

This is *exactly* the ELF shared-library story re-expressed in wasm. **It requires
PIC.** Emscripten turns it on with `-fPIC` at compile and `-sSIDE_MODULE`/`-sMAIN_MODULE`
at link.

---

## 3. webR is a dynamic-linking world — PIC is foundational, not optional

This is the load-bearing fact the original question gets backwards. webR does **not**
statically bake packages into one module. It builds **R itself as the `MAIN_MODULE`**
and **each R package as a `SIDE_MODULE`**, and `dlopen`s packages at runtime — the
faithful wasm reimplementation of native `dyn.load`.

Straight from the webR build system (`r-wasm/webr`, clone at `d90f76b`):

```
.webr/R/Makefile:98    STAGE2_CFLAGS  += -fPIC
.webr/R/Makefile:110   STAGE2_FFLAGS  += -fPIC
.webr/R/Makefile:118     'GOT',          # GOT exported as a runtime method
.webr/R/Makefile:133   MAIN_LDFLAGS  := -s MAIN_MODULE=1
.webr/R/Makefile:138   MAIN_LDFLAGS  += -s ERROR_ON_UNDEFINED_SYMBOLS=0
.webr/R/Makefile:154   SHLIB_LDFLAGS := -s SIDE_MODULE=1     # ← packages are side modules
```

And from the package builder that R CMD INSTALL actually uses under webR
(`r-wasm/rwasm`'s `inst/webr-vars.mk`, clone at `38a0508`):

```
.rwasm/inst/webr-vars.mk:23   WASM_COMMON_FLAGS += -fPIC
.rwasm/inst/webr-vars.mk:46   WASM_LDFLAGS      += -s SIDE_MODULE=1
.rwasm/inst/webr-vars.mk:58   C_VISIBILITY       = -fvisibility=hidden   # see §5
.rwasm/inst/webr-vars.mk:81   FPICFLAGS          = -fPIC
.rwasm/inst/webr-vars.mk:119  override ALL_FFLAGS = ... $(FPICFLAGS) ...
```

So: **every C, C++, and Fortran object in webR — R core and every package — is compiled
`-fPIC`.** PIC isn't merely "necessary in webR," it's the substrate the whole runtime
stands on. Anyone who says "PIC isn't needed in webR" is describing a world that
doesn't exist.

### Empirical confirmation (local `emcc 5.0.7-git`)

Build a trivial C side module and inspect its imports:

```
$ emcc -fPIC -c a.c -o a_pic.o && emcc -sSIDE_MODULE=1 a_pic.o -o a_pic.wasm
$ strings a_pic.wasm | grep -E '__memory_base|__table_base'
__memory_base
env__table_base
```

The side module imports `__memory_base` and `__table_base` — the runtime relocation
hooks. That's PIC, made visible. (`-sSIDE_MODULE=1` implies `-sRELOCATABLE` at link, so
emcc 5.x doesn't hard-*error* on a non-`-fPIC` object in the trivial case, but the
`-fPIC` object is the one that correctly GOT-relativizes global access once
`__memory_base ≠ 0`; webR compiles `-fPIC` everywhere precisely so real code with real
globals doesn't mis-address at load.)

---

## 4. The Rust `-C relocation-model=pic` flag was a verified no-op — proof

Given §3, the side module that contains our Rust code **must** be PIC. So how can it be
correct that we dropped `-C relocation-model=pic` from the Rust build? Because the Rust
`wasm32-unknown-emscripten` target **already emits PIC by default** — the flag was
redundant. Here's the proof, three ways.

### 4a. The target spec doesn't pin a relocation model — it inherits LLVM's PIC default

```
$ rustc +nightly -Z unstable-options --target wasm32-unknown-emscripten --print target-spec-json
  "crt-static-default": true,
  "dll-suffix": ".wasm",
  "dynamic-linking": true,
  "tls-model": "local-exec"
  # note: NO "relocation-model" key at all
```

Contrast `wasm32-unknown-unknown`, which *does* pin it (`"relocation-model": "static"`).
The emscripten target omits the key, so it takes LLVM's default for the target — which
is PIC.

### 4b. The generated LLVM IR carries `PIC Level 2`, and the flag changes nothing

This is the centerpiece. Compile the same crate for emscripten twice — once with
defaults, once with the flag — and diff the LLVM IR:

```rust
// lib.rs
#![crate_type = "cdylib"]
static TABLE: [u32; 4] = [1, 2, 3, 4];
#[no_mangle] pub extern "C" fn read_table(i: usize) -> u32 { unsafe { *TABLE.get_unchecked(i) } }
#[no_mangle] pub extern "C" fn indirect() -> u32 { read_table(2) }
```

```
$ rustc --target wasm32-unknown-emscripten --emit=llvm-ir -O lib.rs -o emcc-default.ll
$ rustc --target wasm32-unknown-emscripten --emit=llvm-ir -O -C relocation-model=pic lib.rs -o emcc-pic.ll

$ grep 'PIC Level' emcc-default.ll
!0 = !{i32 8, !"PIC Level", i32 2}          # ← full PIC, by default, no flag asked

$ diff emcc-default.ll emcc-pic.ll && echo IDENTICAL
IDENTICAL                                    # ← the flag is a no-op

# for contrast, the static target emits no PIC level at all:
$ rustc --target wasm32-unknown-unknown --emit=llvm-ir -O lib.rs | grep 'PIC Level' || echo "(none — static)"
(none — static)
```

`PIC Level 2` is LLVM's "full PIC." The byte-identical diff is the empirical
disproof of the idea that we needed the flag.

### 4c. A Rust emscripten object links into a SIDE_MODULE with zero relocation flags

```
$ rustc --target wasm32-unknown-emscripten --emit=obj --crate-type=staticlib lib.rs -o rust_emcc.o
$ emcc -sSIDE_MODULE=1 -sWASM_BIGINT rust_emcc.o -o rust_side.wasm   # no -C relocation-model, no -fPIC
$ echo $?; strings rust_side.wasm | grep -E '__memory_base|__table_base|__indirect_function_table'
0
__indirect_function_table
__memory_base
env__table_base
```

The Rust object drops straight into a relocatable side module and produces all the PIC
import hooks — without anyone passing a relocation flag. QED.

### 4d. The project's own history says the same thing

The flag's life-cycle, from the issue tracker:

- **#494** proposed `-C relocation-model=pic -C link-args=-s SIDE_MODULE=1` for the wasm
  path, *unvalidated*, copied from a plan doc.
- **#751** shipped the wasm RUSTFLAGS block but **retained** `relocation-model=pic`,
  explicitly noting *"it's likely the wasm32-emscripten default anyway; dropping it is
  deferred until tier-3 is green."*
- **#745** asked the direct question and noted two priors: wasm is position-independent
  by nature, and *savvy doesn't set the flag*.
- **#749** ran the experiment — dropped the flag, tier-2 (emcc side-module link) stayed
  green, tier-3 (#492, `library()` load under webR) stayed green. Verdict: redundant,
  removed.

This is now codified in `rpkg/configure.ac:266`:

```
dnl `-C relocation-model=pic` used to be set here too but was dropped (#745):
dnl PR #749 proved the emcc side-module link succeeds without it, and tier-3
dnl (#492) confirms the module still loads at runtime. wasm32-unknown-emscripten
dnl is position-independent by default, so the explicit flag was a no-op. Do not
dnl re-add without re-running the #749 experiment.
```

---

## 5. The flag that *is* required (and is not PIC): `-Zdefault-visibility=hidden`

It's easy to conflate the dropped flag with the kept flag. They solve unrelated
problems. The kept one:

```
rpkg/configure.ac:271   if test "$is_wasm_install" = true; then
rpkg/configure.ac:272     ENV_RUSTFLAGS="$ENV_RUSTFLAGS -Zdefault-visibility=hidden"
rpkg/configure.ac:273   fi
```

The problem it solves: `-s SIDE_MODULE=1` drags **~3000 mangled stdlib/dependency
symbols** into the side module's EXPORT table. webR's JS-side loader then fails —
`TypeError: Cannot read properties of undefined` on pinned emcc, or a hard
`emcc: error: invalid export name` on emcc 4.0.8+. Hiding symbols by default leaves only
our `#[no_mangle] extern "C"` entry points (`R_init_<pkg>`, the `C_*` wrappers)
exported. This is **savvy's approach** (`yutannihilation/savvy#372`), endorsed by webR's
maintainer (`r-wasm/webr#532`). It's a nightly `-Z` flag, but the wasm build already
mandates nightly via `-Z build-std`, so it adds no new constraint.

**The `force_link` wrinkle (#756).** Hiding all symbols broke one thing: the
`miniextendr_force_link` anchor (the native linkme drag-anchor — see §7) was a
`#[no_mangle] static`, and hidden statics become unresolvable `GOT.mem` imports on wasm
(`bad export type for 'miniextendr_force_link': undefined` at `dlopen`). The fix was to
emit it as a `#[no_mangle] extern "C" fn` instead, which stays exported under hidden
visibility (functions are kept; statics aren't). Visible today at
`miniextendr-macros/src/lib.rs:2763`:

```rust
pub extern "C" fn miniextendr_force_link() {}
```

This is the one place the native and wasm worlds genuinely intersect, and it's worth
internalizing: the visibility flag, not PIC, is what made webR loading work.

---

## 6. Comparison: what savvy does

savvy (`yutannihilation/savvy`) is the closest reference implementation — Rust ↔ R, with
real webR support. Its wasm Makevars
(`savvy-bindgen/src/codegen/templates/Makevars.in`) is the apples-to-apples comparison:

```make
$(STATLIB):
	export PATH="$(PATH):$(HOME)/.cargo/bin" && \
	  export CC="$(CC)" && export CFLAGS="$(CFLAGS)" && \
	  export RUSTFLAGS="$(RUSTFLAGS)" && \
	  if [ "$(TARGET)" != "wasm32-unknown-emscripten" ]; then \
	    cargo build $(CARGO_BUILD_ARGS); \
	  else \
	    export CARGO_PROFILE_DEV_PANIC="abort" && \
	    export CARGO_PROFILE_RELEASE_PANIC="abort" && \
	    export RUSTFLAGS="$(RUSTFLAGS) -Zdefault-visibility=hidden" && \
	    cargo +nightly build $(CARGO_BUILD_ARGS) --target $(TARGET) -Zbuild-std=panic_abort,std; \
	  fi
```

Read what's there and what's **not**:

| Knob | savvy (wasm) | miniextendr (wasm) | Same conclusion? |
|---|---|---|---|
| `-Zdefault-visibility=hidden` | ✅ set | ✅ set | yes — export hygiene |
| `panic = abort` | ✅ set | (target default via `-Zbuild-std=...,panic_abort`) | yes |
| `-Zbuild-std` | ✅ `panic_abort,std` | ✅ `std,panic_abort` | yes |
| **`-C relocation-model=pic`** | ❌ **never set** | ❌ **dropped (#745)** | **yes — both agree it's unnecessary** |
| **`codegen-units = 1`** | ❌ **never set** | ✅ set (native reasons, §7) | savvy doesn't need it at all |

**Why savvy never needs `codegen-units`.** savvy uses **explicit, generated C
registration** — `savvy update` runs `savvy-cli`, which emits an `init.c` containing a
literal `static const R_CallMethodDef CallEntries[]` table
(`savvy-bindgen/src/codegen/c.rs`). Every wrapper is named and referenced from
hand-generated C. There is **no linkme, no `#[distributed_slice]`** anywhere in savvy.
With explicit references, the linker's archive-member selection pulls every wrapper
naturally — the failure mode that `codegen-units = 1` defends against (next section)
simply cannot arise. savvy bought registration ergonomics differently (a codegen step
the user re-runs) and paid a different price; miniextendr bought *automatic* registration
via linkme and pays for it with `codegen-units = 1`.

---

## 7. `codegen-units = 1` — why it exists, and whether webR needs it

The comment under audit:

```toml
# rpkg/src/rust/Cargo.toml:81
# codegen-units = 1 ensures the entire user crate compiles into a single .o file
# inside the staticlib archive. This is required for linkme's distributed_slice:
# the linker only pulls archive members that resolve undefined symbols, so with
# multiple .o files, distributed_slice entries in unreferenced .o files get dropped.
# With a single .o, pulling in R_init_<pkg> brings everything along.
[profile.dev]
codegen-units = 1
[profile.release]
codegen-units = 1
```

### 7a. Why it exists (native) — and the comment is correct

miniextendr registers `#[miniextendr]` items automatically via linkme
`#[distributed_slice]`. Those slice entries live in **custom link sections** and are
**not referenced by any symbol** — the linker is supposed to gather them by section, not
by reachability. But a static-library link (`.a`) only pulls in an archive *member*
(`.o`) if that member **defines a symbol some already-included object left undefined**.
So if the crate is split into many codegen units → many `.o` in the archive, a `.o`
whose *only* contribution is unreferenced `#[distributed_slice]` entries is **never
pulled in**, and its registrations **silently vanish**. You get a package that compiles,
links, loads — and is missing half its functions.

`codegen-units = 1` collapses the entire user crate into a **single** `.o`. Now one
reference into that object — the `miniextendr_force_link` anchor that `stub.c` takes the
address of — drags the whole thing in, distributed-slice sections and all. This is the
cheaper replacement for `-Wl,-force_load` (macOS) / `-Wl,--whole-archive` (Linux); the
Makevars confirms no force-load is used:

```make
# rpkg/src/Makevars.in:26
# codegen-units = 1 in Cargo.toml ensures the user crate compiles into a single .o
# inside the staticlib archive. The linker pulls it in (via R_init_<pkg>), which
# brings all linkme distributed_slice entries along — no force_load needed.
PKG_LIBS = $(CARGO_AR)
```

So on native: **`codegen-units = 1` is load-bearing.** Remove it and registrations get
dropped under any multi-CGU build. This is real and the comment is right.

### 7b. Is it necessary for webR? No — the failure mode can't occur on wasm

Here is the part the comment doesn't address, and the direct answer to your question.

**On wasm there is no linkme.** Every `#[distributed_slice]` in the runtime is gated
out:

```
miniextendr-api/src/registry.rs:10   #[cfg(not(target_arch = "wasm32"))] use linkme::distributed_slice;
miniextendr-api/src/registry.rs:16   // linkme::distributed_slice does not compile for wasm32-* targets —
miniextendr-api/src/registry.rs:31   #[cfg(not(target_arch = "wasm32"))] #[distributed_slice]
   ... (every slice + its writer is cfg(not(target_arch = "wasm32")))
```

In its place, the host emits a `wasm_registry.rs` snapshot that the macro pulls in only
on wasm (`miniextendr-macros/src/lib.rs:2715`, `#[path="wasm_registry.rs"] mod ...`). And
crucially, that snapshot is **plain, referenced code** — not section-collected entries:

```rust
// rpkg/src/rust/wasm_registry.rs (generated)
unsafe extern "C-unwind" {
    pub fn C_validate_class_args(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_do_nothing(_: SEXP) -> SEXP;
    // ... every wrapper, declared as an ordinary extern fn
}
// ... then arrays of R_CallMethodDef that take the addresses of those fns,
//     consumed directly by R_init_<pkg>.
```

So the wasm reachability graph is **fully connected by ordinary symbol references**:

```
R_init_<pkg>  →  MX_CALL_DEFS_WASM[] (a normal static array, same crate)
              →  &C_foo, &C_bar, ...  (address-taken extern fns)
              →  the .o that defines each C_foo
```

`R_init_<pkg>` is an exported root (R looks it up at `dlopen`). It references the
registry array; the array references each `C_*` by address; each reference is a normal
*undefined-symbol* that `wasm-ld` resolves by pulling the defining archive member. **No
member is reachable only via an unreferenced custom section** — because on wasm there are
no such sections. Therefore the exact thing `codegen-units = 1` exists to prevent is
*structurally impossible* on wasm, regardless of how many codegen units the wasm
staticlib is split into.

**Conclusion:** `codegen-units = 1` is **not necessary for webR support.** It is set
unconditionally in `[profile.*]`, so it does apply to the wasm build, where its effect is
limited to the ordinary trade-off (less compile parallelism, more cross-function
inlining, marginally smaller/faster output) — *not* registration correctness. webR works
because of the `wasm_registry.rs` + `R_init` reference chain, not because of the single
codegen unit.

### 7c. Honesty box — what would falsify §7b

The argument above is a *mechanism* argument (cfg-gating + reference graph), verified by
reading the generated code, not by a CI run that builds the wasm staticlib with
`codegen-units = 16` and confirms `library()` still loads. That experiment has **not**
been run. It would be cheap to run in the `mxe-wasm` container and would convert "should
be inert" into "is inert." Until then, the safe and zero-cost status quo is to leave
`codegen-units = 1` in place — it's required for native and harmless for wasm. If you
ever want the wasm build to parallelize, the change is to make the setting
native-conditional (e.g. a wasm profile override), *then* run that load test before
trusting it.

### 7d. "Is there really no flag that retains the slice without `codegen-units = 1`?"

Short answer: **`codegen-units = 1` is not the only fix — it's the cheapest surgical one.**
The problem ("the linker drops archive members that define only unreferenced
section-collected statics") is a known, tracked linker behavior (`rust-lang/rust#67209`,
`dtolnay/linkme#36`); linkme's own README doesn't even document a workaround. Here's the
full menu, after walking `rustc -C help` and `rustc +nightly -Z help`:

**Things that genuinely work:**

| Lever | Where | Works? | Cost |
|---|---|---|---|
| `codegen-units = 1` | Cargo profile | ✅ | Stable, one line, **surgical** — only the user crate becomes one code object; std stays demand-loaded. |
| `-Wl,--whole-archive` (GNU/lld) / `-Wl,-force_load` (macOS) | **Makevars `PKG_LIBS`** | ✅ | Per-platform flag, and **force-loads the *entire* staticlib** — see the bloat note below. This is literally what the Makevars comment says cu=1 replaced. |
| `-C lto=fat` | rustc | ✅ (measured: collapses user crate to **1** object) | Whole-program opt is slow; complicates the cdylib↔staticlib double-link. A heavier hammer for the same retention. |

**Things that *look* like fixes but are not:**

- **`-C link-dead-code=yes`** — about `--gc-sections` at the *final* link, i.e. keeping
  dead code in objects that are *already included*. Archive-member *selection* happens
  *before* GC; an un-pulled member never becomes a GC candidate. Measured: it changed the
  emitted object set (401 members) but does not force-pull unreferenced members.
- **`#[used]` / `#[used(linker)]`** — linkme already marks entries `#[used]`. It protects
  a symbol from GC *inside an included object*; it does nothing for archive selection.
- **`-Z combine-cgu`** — does **not exist** in current rustc (verified against
  `rustc +nightly -Z help`; I had misremembered it). No `-Z` flag force-retains members.
- **`-Wl,--undefined=SYM` / `-u SYM`** — works *per symbol*: forces the member defining
  `SYM` to be pulled. But you'd have to name every entry's symbol — which is just
  reinventing explicit registration (next point), and linkme entries have no stable names.

**The real architectural escape hatch:** drop the section-collection dependency entirely
and emit an explicit table that *names* every wrapper symbol, reached from a root
(`R_init`). With explicit references there is no archive-drop and **no `codegen-units`
constraint at all**. This is exactly what **savvy** does (generated `init.c`) and what
**miniextendr already does on wasm** (`wasm_registry.rs`). On native, linkme is kept for
the ergonomics (no codegen step for the user), and `codegen-units = 1` is its tax.

### 7e. Why cu=1 beats `--whole-archive` — measured

A Rust `staticlib` doesn't contain just your crate; it bundles **all of std** as object
files. Measured on this machine (16-function test crate, `aarch64-apple-darwin`):

```
                 user-crate code objects   total archive members
codegen-units=16          4                        388   (std + core + alloc +
codegen-units=1           1 (+1 alloc shim)         386    compiler_builtins + libc +
-C lto=fat                1                          367    object/gimli/addr2line/...)
```

So the archive has ~**386 members**, of which only a handful are *your* code. Consequences:

- `-Wl,--whole-archive $(CARGO_AR)` force-loads **all ~386** — you drag the entire Rust
  standard library and its deps into every package `.so`. That's the bloat cu=1 avoids.
- `codegen-units = 1` makes *your* crate exactly one code object. The `R_init_<pkg>` /
  `miniextendr_force_link` reference pulls that one object by ordinary symbol resolution,
  carrying all `#[distributed_slice]` entries with it — while std stays demand-loaded
  member-by-member as usual. Surgical, not nuclear.

And note *where* each lever applies: the final `.so` link is done by **R's Makevars**
(`PKG_LIBS = $(CARGO_AR)`), not by rustc — so a `--whole-archive`/`-force_load` fix lives
in `Makevars`, and a rustc `-C link-arg` would only affect the *cdylib* (wrapper-gen)
link, not the installed shared object. `codegen-units` is the one lever that's set once,
in the crate, and covers both links on every platform.

### 7f. Considered alternative: route native through a generated table too (rejected)

The clean way to delete the `codegen-units = 1` constraint *entirely* would be to stop
consuming linkme in the staticlib and have native `R_init` consume an explicit generated
registry — exactly what wasm already does with `wasm_registry.rs`. It is ~90% built
already. **Rejected for native.** Reasoning recorded here so it isn't re-proposed cold.

**It does not actually remove linkme.** `wasm_registry.rs` is *generated by the cdylib
walking the linkme slices at runtime* (`miniextendr_write_wasm_registry`). You'd still
need linkme to *produce* the table — you'd only stop *consuming* it in the staticlib.

**Why native even has a choice (and wasm doesn't).** The archive-drop problem is
exclusive to the `staticlib` crate-type: a cdylib hands the crate's CGU objects straight
to rustc's own link line (always included), whereas a staticlib packs them into a `.a`
whose members the *downstream* link (R's Makevars) selects. So linkme works in the cdylib
regardless of `codegen-units`; the staticlib is the only place it can drop entries. On
wasm, linkme doesn't compile at all → the generated table is *forced*. On native, linkme
works → link-time collection is available, and it has a property the generated table
cannot match: **it cannot drift.** Registration is gathered from the actually-compiled
code at link time, so a fresh build is correct by construction — there is no committed
artifact to keep fresh.

**What adopting it on native would cost:**
1. **Bootstrap / build-order inversion.** Today the staticlib builds first and is
   self-contained (linkme); the cdylib generates files afterward. If the staticlib
   *consumed* the registry, you'd need registry-before-staticlib — chicken-and-egg on a
   clean tree, and downstream `install_github` consumers would need the committed registry
   fresh. That is exactly the constraint that makes the wasm path fragile
   (`rpkg/configure.ac:216` hard-errors if `wasm_registry.rs` is absent), now extended to
   every platform.
2. **A sync contract everywhere.** The ~11.8K-line generated file would become
   load-bearing for *native* correctness too — regenerated on every macro-surface change,
   enforced by tooling. Native is currently immune to that whole class of "generated file
   went stale" bug.
3. **linkme stays anyway** (for generation), so the dependency isn't shed.

**What it would buy:** codegen parallelism (faster staticlib builds). That's the entire
upside, and it's modest for a typical user crate.

**Verdict:** keep linkme + `codegen-units = 1` on native. The current per-platform split
is correct — each target uses the aggregation mechanism that fits its constraints:
link-time collection where linkme compiles (can't-drift, costs build parallelism), a
generated table where it doesn't (forced). **Revisit only if** single-unit codegen
becomes a *measured* build-time bottleneck on a large crate; then scope `codegen-units = 1`
to non-wasm and route native through the same generated registry — paying the
sync/bootstrap cost knowingly.

---

## 8. Recommendations

1. **Leave `-C relocation-model=pic` gone.** It's a proven no-op (§4). The
   `configure.ac:266` comment already guards against a well-meaning re-add — keep it.
2. **Keep `-Zdefault-visibility=hidden`.** It's the actual webR-enabling flag, unrelated
   to PIC (§5). Removing it reintroduces the ~3000-symbol export failure.
3. **Keep `codegen-units = 1` as-is for now, but fix the comment.** The current comment
   implies a blanket necessity; it's a *native linkme* necessity. Suggested tweak:
   > `codegen-units = 1` forces the user crate into one `.o` so the linker can't drop
   > unreferenced linkme `#[distributed_slice]` archive members (native registration
   > path). Inert on `wasm32-*`, which uses the referenced `wasm_registry.rs` snapshot
   > instead of linkme.
4. **If wasm compile time ever matters**, scope `codegen-units = 1` to non-wasm and run
   the §7c falsification test once. Low priority.
5. **One-liner for anyone who asks "does webR need PIC?"**: *webR runs entirely on PIC
   (R is a MAIN_MODULE, packages are SIDE_MODULEs); what it doesn't need is for us to
   pass `-C relocation-model=pic`, because the emscripten Rust target emits PIC Level 2
   by default.*

---

## Appendix A — Reproduce every run

All commands run on macOS (`darwin 25.5.0`), `rustc 1.95.0` stable /
`1.97.0-nightly`, `emcc 5.0.7-git`, with `wasm32-unknown-emscripten` and
`wasm32-unknown-unknown` targets installed.

```bash
# --- §4a: target relocation models ---
rustc +nightly -Z unstable-options --target wasm32-unknown-emscripten --print target-spec-json | grep -E 'relocation|dynamic-link|crt-static'
rustc +nightly -Z unstable-options --target wasm32-unknown-unknown   --print target-spec-json | grep relocation   # -> "static"

# --- §4b: PIC Level + no-op diff ---
cat > lib.rs <<'EOF'
#![crate_type = "cdylib"]
static TABLE: [u32; 4] = [1, 2, 3, 4];
#[no_mangle] pub extern "C" fn read_table(i: usize) -> u32 { unsafe { *TABLE.get_unchecked(i) } }
#[no_mangle] pub extern "C" fn indirect() -> u32 { read_table(2) }
EOF
rustc --target wasm32-unknown-emscripten --emit=llvm-ir -O                          lib.rs -o emcc-default.ll
rustc --target wasm32-unknown-emscripten --emit=llvm-ir -O -C relocation-model=pic  lib.rs -o emcc-pic.ll
grep 'PIC Level' emcc-default.ll          # !0 = !{i32 8, !"PIC Level", i32 2}
diff emcc-default.ll emcc-pic.ll && echo IDENTICAL

# --- §4c: Rust object -> SIDE_MODULE, no relocation flag ---
rustc --target wasm32-unknown-emscripten --emit=obj --crate-type=staticlib lib.rs -o rust_emcc.o
emcc -sSIDE_MODULE=1 -sWASM_BIGINT rust_emcc.o -o rust_side.wasm
strings rust_side.wasm | grep -E '__memory_base|__table_base|__indirect_function_table'

# --- §3 empirical: C side module imports the PIC hooks ---
printf 'static int T[4]={1,2,3,4}; int rd(int i){return T[i];}\n' > a.c
emcc -fPIC -c a.c -o a_pic.o && emcc -sSIDE_MODULE=1 a_pic.o -o a_pic.wasm
strings a_pic.wasm | grep -E '__memory_base|__table_base'
```

## Appendix B — Source map

**This repo**
- `rpkg/src/rust/Cargo.toml:81-90` — the `codegen-units = 1` comment under audit
- `rpkg/src/Makevars.in:26-29` — "no force_load needed"; `PKG_LIBS = $(CARGO_AR)`
- `rpkg/configure.ac:252-273` — wasm RUSTFLAGS: `-Zdefault-visibility=hidden` kept,
  `-C relocation-model=pic` dropped with #745 rationale
- `miniextendr-api/src/registry.rs:10,16,31…` — linkme is `cfg(not(target_arch="wasm32"))`
- `miniextendr-macros/src/lib.rs:2715,2763` — wasm registry `mod`; `force_link` as a fn
- `rpkg/src/rust/wasm_registry.rs` — generated `extern "C-unwind"` decls + CallDef arrays

**Issues / PRs**
- #494 (proposed the flag, unvalidated) · #751 (added `-Zdefault-visibility=hidden`, kept
  PIC) · #745 (validate PIC necessity) · #749 (experiment: dropped it, CI green) · #492
  (tier-3 `library()` load validator) · #756 (`force_link` static→fn under hidden vis)

**Upstream**
- `r-devel/r-svn` — `configure.ac` `FPICFLAGS`, `Makeconf.in` (native PIC is mandatory)
- `r-wasm/webr` @ `d90f76b` — `R/Makefile` (`MAIN_MODULE=1`, `SIDE_MODULE=1`, `-fPIC`)
- `r-wasm/rwasm` @ `38a0508` — `inst/webr-vars.mk` (`-fPIC`, `SIDE_MODULE=1`, hidden vis)
- `yutannihilation/savvy` — `savvy-bindgen/src/codegen/templates/Makevars.in`
  (`-Zdefault-visibility=hidden`, no `relocation-model`, no `codegen-units`);
  `savvy-bindgen/src/codegen/c.rs` (explicit `R_CallMethodDef` init.c, no linkme);
  PR #372 (the visibility fix)
- Emscripten dynamic linking: `MAIN_MODULE`/`SIDE_MODULE` ⇒ relocatable/PIC modules
  importing `__memory_base` / `__table_base` / GOT.
