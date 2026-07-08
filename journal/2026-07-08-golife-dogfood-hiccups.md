# golife dogfood session — hiccups log

Building a demo package (`golife`, Conway's Game of Life) at
`/Users/elea/Documents/GitHub/golife` as an end user of `minirextendr` +
`miniextendr`, exercising as many advanced features as reasonably fit
(ALTREP, DataFrameRow in+out, typed_list! dots, trait-impl methods,
serde-based returns). Logging anything that trips up a fresh user as we go;
convert to `gh issue create` afterward per CLAUDE.md's deferred-items rule.

## Hiccups

### 1. Fresh scaffold + configure before `git init` silently lands in CRAN tarball mode

`usethis::create_package()` does not `git init` by default (it just prints a
hint to run `usethis::use_git()`). `minirextendr::use_miniextendr()` also
warns "No .git directory found" but proceeds anyway. Running `bash
./configure` on that scaffold before ever running `git init` trips the
auto-vendor self-repair (`no inst/vendor.tar.xz` + `cargo-revendor` on PATH +
`no .git ancestor` → auto-vendors), which flips the package straight into
CRAN-style offline tarball mode. Nothing in the configure output says *why*
it chose tarball mode — it just prints `configure: install mode = tarball
install (offline, vendored)`, indistinguishable-looking from an intentional
CRAN prep step. A first-time user who scaffolds and immediately runs
configure (very natural order) gets silently steered into the mode where
`R CMD INSTALL .` / `devtools::document()` skip wrapper regeneration, with no
signal that `minirextendr::miniextendr_build()` is now required instead of
the plain commands the getting-started skill itself shows as step 3
(`bash ./configure && R CMD INSTALL .`).

Worked around by `git init`-ing before configuring and deleting the
accidentally-produced `inst/vendor.tar.xz` + `vendor/` to fall back to normal
source mode. Possible fix: configure could print one line explaining *why*
it picked tarball mode when the auto-vendor branch fires (e.g. "no .git
ancestor found, assuming CRAN-style build"), or the getting-started skill's
step 1 could recommend `git init` immediately after `use_miniextendr()`.

### 2. `#[miniextendr(dots = typed_list!(...))]` sugar is function-only, silently absent inside impl blocks

The attribute sugar documented by the `miniextendr-dots` skill and
`dots_tests.rs` (`#[miniextendr(dots = typed_list!(x => numeric()))]` on a
free `pub fn`) is implemented only in the standalone-function attribute
parser (`miniextendr_fn.rs`). The impl-block method attribute parser
(`miniextendr_impl.rs`, used for `#[miniextendr(r6)] impl Board { ... }`
methods including constructors) has a completely different, disjoint option
set — `env, r6, s3, s4, s7, vctrs, defaults, unsafe, check_interrupt, coerce,
no_coerce, rng, unwrap_in_r, as, lifecycle, r_name, r_entry, r_post_checks,
r_on_exit, noexport, internal` — with no `dots` key at all. Trying it on a
class constructor fails at compile time with `error: unknown attribute;
expected one of: ...` (no mention that this is a function-vs-method split;
reads like a typo). The raw `_dots: ...` parameter itself works fine in impl
methods (confirmed via `is_dots_type` handling in `miniextendr_impl.rs`), so
the workaround is just to call `.typed(typed_list!(...))` manually in the
method body instead of relying on the sugar's auto-generated `dots_typed`
binding — no loss of functionality, just more boilerplate and a surprising
first encounter. Worth either wiring the sugar into the impl-block parser too,
or having the skill/docs call out explicitly that it's function-only.

### 3. ALTREP derive macros/traits aren't in `miniextendr_api::prelude`

The `miniextendr-altrep` skill's own minimal example writes `use
miniextendr_api::prelude::*;` then `#[derive(AltrepInteger)]` and expects
`AltrepLen`/`AltIntegerData` to be in scope. In practice this fails to
compile (`cannot find derive macro AltrepInteger`, `cannot find trait
AltrepLen`) — real fixtures (`altrep_condition_tests.rs`) import explicitly:
`use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};` and `use
miniextendr_api::{AltrepInteger, ...};`. The prelude does not re-export the
ALTREP derive macros or the per-family data traits. Either the prelude should
include them, or the skill's own code sample needs the explicit imports
added — as written it doesn't compile.

### 4. Bare `dots: ...` parameter is fully broken inside impl-block methods

After switching away from the `dots =` attribute sugar (hiccup #2) to a
manual `dots.typed(typed_list!(...))` call, the bare `dots: ...` parameter
itself on an impl-block method fails too — not with a miniextendr error, but
a raw rustc error, meaning the macro passed the literal `...` token straight
through to the compiler instead of rewriting it:
```
error: `...` is not supported for non-extern functions
   --> board.rs:100:73
    = help: only `extern "C"` and `extern "C-unwind"` functions may have a C
      variable argument list
```
Grepping all of `rpkg/src/rust/*.rs` (the framework's own exhaustive fixture
suite) for any impl-block method using a dots parameter turns up **zero**
matches — every `dots: ...` example in the repo (`dots_tests.rs`,
`vctrs_class_example.rs`) is a standalone `#[miniextendr] pub fn`, never a
method inside `#[miniextendr(r6)] impl` (or any other class-system impl
block). `miniextendr_impl.rs` does reference `is_dots_type` (borrowed from
`miniextendr_fn.rs`), so there's some partial awareness, but nothing rewrites
the token for the impl-block codegen path — the `...` survives unchanged
into the final expansion. Net effect: **dots/`...` params appear to be
completely unsupported on class methods/constructors today**, not just
missing the attribute sugar. Worked around by dropping dots entirely from
`Board::new()` in favor of plain `Option<Vec<i32>>` parameters, and moving
the `typed_list!` showcase to a standalone free function instead. Given how
natural "configurable constructor options via dots" is for a class, this
looks like a real gap worth a `gh issue create` rather than just a doc note.

### 5. `TypedList::get`/`get_opt` can't retrieve vector fields even though `typed_list!` can validate them

`typed_list!(survive => integer())` happily validates that `survive` is an
integer vector, but `TypedList::get::<Vec<i32>>("survive")` fails to compile:
```
error[E0271]: type mismatch resolving `<Vec<i32> as TryFromSexp>::Error == SexpError`
   |
   = note: required by a bound in `TypedList::get`
   |    T: TryFromSexp<Error = SexpError>,
```
(`miniextendr-api/src/typed_list.rs:300-318`). `Vec<i32>`'s `TryFromSexp`
impl uses `SexpTypeError`, not `SexpError` — scalars (`f64`, `i32`, `String`,
`bool`) use `SexpError` but vector conversions use the richer
`SexpTypeError`, and `get`/`get_opt` are hard-bound to the scalar error type.
So the only fields you can actually pull out with the ergonomic `.get()`/
`.get_opt()` are scalars; anything the schema declares as `numeric()`/
`integer()`/`character()` (i.e. a vector) must go through `get_raw()` +
manual `Vec::<T>::try_from_sexp(raw)`. This isn't documented as a
scalar-only restriction anywhere in the `miniextendr-dots` skill or
`docs/DOTS_TYPED_LIST.md` — the skill's own reference table lists
`numeric()`/`integer()`/`character()` as ordinary type specs with no hint
that retrieval requires a different code path than scalars. Likely fix:
relax the bound on `get`/`get_opt` to accept any `TryFromSexp` and map both
error variants into `TypedListError::WrongType`.

### 6. `minirextendr::miniextendr_build()` needs two passes for a new export — FALSE ALARM, root cause is hiccup #1

Originally logged as the same "second install needed" gotcha that root
`CLAUDE.md` documents for the monorepo's maintainer-only `just rcmdinstall
&& just force-document` recipe, silently recurring in
`minirextendr::miniextendr_build()` itself. A repo-wide audit (dispatched
after this session) found that's wrong: `miniextendr_build()`
(`minirextendr/R/workflow.R:136-224`) already does exactly the right thing —
it snapshots `NAMESPACE` before/after `devtools::document()` and, if it
changed, runs a same-call reinstall (lines 199-224, documented in the
`@section Why a conditional reinstall` comment at lines 92-108). This is
issue **#860** ("`miniextendr_build()` installs before `document()` → first
build exports nothing"), closed 2026-06-09, with a dedicated regression test
(`minirextendr/tests/testthat/test-templates.R:637-706`) that calls
`miniextendr_build()` **exactly once** and asserts the new export is
callable.

What golife actually hit was a knock-on effect of hiccup #1: once the
scaffold silently latched into tarball mode (no `.git` ancestor at configure
time), install skips wrapper regeneration entirely
(`minirextendr/R/workflow.R:114-118`), which defeats the #860
document()-diff-triggered reinstall regardless — `miniextendr_build()` had
nothing to diff against because the wrappers never regenerated in the first
place. The skill edit made in this repo
(`.claude/skills/miniextendr-getting-started/SKILL.md` Step 3/4,
recommending `miniextendr_build()` as the primary standalone-package path)
is still a reasonable improvement on its own merits, but the "two-pass"
framing and the `gh issue create` this entry originally called for are
retracted — there is no bug left in `miniextendr_build()` to file. See
hiccup #1's fix instead.

### 7. R6 trait-impl methods are flat free functions, not `obj$method()` — genuine gap, not a documented tradeoff

`#[miniextendr(r6)] impl Summarize for Board { fn describe(&self) -> String
{...} }` does not produce `board$describe()`, `board$Summarize$describe()`,
or `Board$Summarize$describe(board)` — only a disconnected top-level
function `r6_trait_Summarize_describe(x)` that isn't wired into the R6
class's `public = list(...)` at all. This contradicts the
`miniextendr-class-systems` skill's "Trait methods across class systems"
section, which describes universal "dual calling"
(`Type$Trait$method(obj)` / `obj$Trait$method()`) as if it applied across
class systems.

Investigated with an agent to confirm this isn't a local misunderstanding:
- `miniextendr-macros/src/miniextendr_impl_trait/r_wrappers.rs:862-871` has
  a doc comment that already **self-acknowledges** the limitation: "R6
  classes are defined monolithically (all methods in `R6Class()`), so trait
  methods cannot be injected into the class definition. Instead, they are
  generated as standalone exported functions."
- Dual calling (`Type$Trait <- new.env(...)`, `` `$.ClassName` `` searching
  trait namespaces and binding `self`) is real **only for the Env class
  system** (`env_class.rs:184-229,193-196,216-218`). S3, S4, and S7 all emit
  flat functions too (`generic.Type` / `setGeneric`+`setMethod` / S7
  generics) — which is idiomatic for those systems' own generic-dispatch
  conventions, but R6 users reasonably expect `obj$method()`, so a bare
  `r6_trait_X_Y(obj)` free function is a real UX gap for R6 specifically,
  not a deliberate design choice.
- A plausible fix path exists and is unused today: R6 supports post-hoc
  method injection via `ClassName$set("public", name, fn)`. The R6 code
  generator is already R6-`$set()`-aware for other purposes (per issue
  #369, docs-only), just not repurposed to let a *later* trait-impl macro
  invocation inject into an *earlier* class definition.
- No existing tracked issue covers this (checked `ISSUES/_open-index.json` /
  `_closed-index.json` — closest tangential hits are #369, #987, #495,
  #161). Worth a `gh issue create`: either wire R6 trait dispatch through
  `$set()`, or explicitly narrow the class-systems skill's "dual calling"
  claim to Env-only so it stops overpromising for R6/S3/S4/S7.

### 8. Fresh `use_miniextendr()` scaffold's `.gitignore` doesn't match where generated files actually live

Three generated artifacts that this repo's own `CLAUDE.md` says must be
gitignored are **not actually covered** by the `.gitignore` that
`minirextendr::use_miniextendr()` writes for a fresh standalone package
(verified via `git status` / `git check-ignore` after a real build):

- **`src/rust/.cargo/config.toml`** (the real, nested location `configure`
  writes to) vs. the gitignore pattern `.cargo/config.toml` (no leading
  path — anchors to `<repo-root>/.cargo/config.toml` only, per gitignore
  semantics for a pattern containing a `/`). `git check-ignore -v
  src/rust/.cargo/config.toml` returns exit 1 (not ignored); `git status`
  shows it as untracked (`??`) after every fresh `configure` run.
- **`R/golife-wrappers.R`** (real location, matching `rpkg/R/miniextendr-wrappers.R`
  in this repo) vs. the gitignore pattern `src/*-wrappers.R` (looks under
  `src/`, not `R/`). This is the file CLAUDE.md calls out most emphatically
  as gitignored-because-it-caused-"constant merge conflicts" — yet a fresh
  scaffold tracks it by default. `git add -A` staged it as a new file
  (`A  R/golife-wrappers.R`) in this session. Note: the sibling
  `.Rbuildignore` pattern `^src/.*-wrappers\.R$` should stay as-is (also
  wrong path, but *harmlessly* — CLAUDE.md says wrappers.R must ship in the
  built tarball from disk, i.e. NOT be Rbuildignored, same treatment as
  `wasm_registry.rs` below, so only the `.gitignore` side needs the `R/`
  path fix, not `.Rbuildignore`).
- **`src/rust/wasm_registry.rs`** has no gitignore entry at all (correctly
  absent from `.Rbuildignore`, per CLAUDE.md — it should ship in the built
  tarball from disk — but it should still be gitignored in the source repo,
  regenerated every host install). `git add -A` staged it too
  (`A  src/rust/wasm_registry.rs`).

Worked around by hand-patching golife's own `.gitignore`. This looks like
the scaffold template drifted from the two generated-file conventions
documented in root `CLAUDE.md` (which itself was presumably updated after
the templates were last synced) — worth a `gh issue create` to fix
`minirextendr`'s `.gitignore` template (`minirextendr/inst/templates/`) to
match current reality, plus a `.Rbuildignore` fix for the wrappers.R path.

### Correction: builder-pattern support exists and is more first-class than first assessed

Initially told the user "no dedicated builder-pattern feature, just void
`-> ()` methods getting `invisible(self)`." That undersold it. Found
`rpkg/src/rust/pipe_builder_tests.rs` + `rpkg/tests/testthat/test-pipe-builder.R`
afterward: `&mut self -> &mut Self` is a **deliberately supported, tested
idiom across all six class systems** (S3, R6, S4, S7, Env, and implicitly
Vctrs), not just an R6 side effect:
- S3/S4/S7 generate free functions (`builder_set_name(x, ...)`, `s4_add(x,
  ...)`, `s7_add(x, ...)`) whose body returns the receiver — composes
  directly under R's native `|>` pipe.
- R6/Env return the same object identity through the chain — deliberately
  guarded: the R6 fixture's comment calls out that codegen must NOT
  re-wrap via `R6PipeBuilder$new(.ptr = .val)` (would mint a new R6
  environment around the same pointer and break identity) — see issue
  #769.
- `PatternBuilder`'s `add_*` methods were switched from `-> ()` to
  `-> &mut Self` to match this canonical idiom once found.

**Docs gap**: `docs/S3_METHODS.md` ("Functional builders (native pipe `|>`)",
~L130-190, plus a dispatch-table row at L396) documents the idiom well but
is scoped to the S3 doc file only — it never mentions the same idiom is also
supported (and fixture-tested) for R6, S4, S7, and Env. Someone reading only
`miniextendr-class-systems` (no builder mention at all, until this session's
edit) or only `docs/S3_METHODS.md` (S3-only framing) would miss that this is
a general, cross-class-system feature. **Fixed** the skill side (added the
"In-place builder idiom" subsection to `miniextendr-class-systems`
`SKILL.md`, cross-referencing all six systems and `pipe_builder_tests.rs`).
The `docs/` side could still use a cross-reference from `S3_METHODS.md` to
R6/S4/S7/Env, or a dedicated page — left as-is for now since `docs/` edits
require a `scripts/docs-to-site.sh` regen pass and this is a nice-to-have,
not a defect; worth a `gh issue create` if the user wants it tracked.

**Design suggestion (not implemented, for later)**: today, whether a void
`&mut self -> ()` R6 method returns `invisible(self)` vs `invisible(NULL)`
is an unconditional blanket default baked into `build_r6_body` — there's no
per-method opt-out (see the dispatched sub-agent investigation below once it
lands). A cleaner alternative: introduce a marker return type, e.g.
`Invisible<T>` (a thin newtype), so the *method signature itself* declares
chainability intent instead of it being inferred implicitly from "returns
unit": `fn mutate(&mut self) -> Invisible<Self>` opts into
`invisible(self)`, while a plain `fn mutate(&mut self)` (real `-> ()`) could
then safely mean "return `invisible(NULL)`, not chainable" without a
behavior change surprising anyone relying on void-as-void. `Invisible<()>`
would cover the "invisible NULL, not self" case some non-chainable void
methods might actually want. **Filed as
[#1213](https://github.com/A2-ai/miniextendr/issues/1213)**, broadened into a
brainstorm of the whole "marker newtype controls R wrapper generation" design
space (`Invisible<T>`, `WrapAs<T>` as an alternative to hiccup #9's
heuristic, `Raw<T>`, a shared dispatch-table mechanism) — not an immediate
fix, needs design discussion on scope and whether it's additive or a
breaking change to the existing void-method default.

Investigated why this is unconditional (agent, `method_return_builder.rs`):
the check (`ReturnStrategy::for_method`) is purely syntactic — `&mut self`
receiver + literal `()` return, no attribute gating, no opt-out anywhere in
`MethodAttrs`. Introduced in the founding codegen commit (`a766508f`,
2025-12-20) uniformly across all 5 hand-rolled generators with just a
one-line comment; the "avoids `obj$mutate()` printing NULL while enabling
chaining" rationale wasn't written up until the class-systems skill, well
after the fact. One concrete, undocumented asymmetry that strengthens the
case for an explicit marker: `&mut self -> Result<(), E>` (a *fallible* void
mutator) does **not** qualify as "unit" by the syntactic check, so it falls
through to a bare-`NULL` `Direct` return instead of `invisible(self)` — a
mutator's chainability today silently depends on whether its error path is
modeled as `Result` or as a panic, which has nothing to do with intent. An
explicit `Invisible<T>` marker would make chainability independent of error
handling style.

### 9. A method returning a DIFFERENT ExternalPtr class isn't auto-wrapped into that class's R6 instance

`PatternBuilder::build(&self, width: i32, height: i32) -> Board` (a method
on one R6 class returning a *different* ExternalPtr-backed R6 class,
finally materializing the accumulated plan) generates a wrapper that just
returns the naked pointer:
```r
build = function(width, height) {
  ...
  .val <- .Call(C_PatternBuilder__build, .call = match.call(), private$.ptr, width, height)
  if (inherits(.val, "rust_condition_value") && ...) return(...)
  .val   # <-- bare externalptr, NOT wrapped as Board$new(.ptr = .val)
}
```
So `plan$build(40L, 40L)$population()` fails with `object of type
'externalptr' is not subsettable` — the natural, most obvious way to use a
builder whose whole point is "materialize into a *different*, richer
object" doesn't work out of the box.

The fix mechanism already exists and works when invoked by hand: every R6
`initialize()` this session generated accepts a `.ptr = NULL` escape hatch
specifically for wrapping a pre-built pointer (confirmed for both `Board`
and `PatternBuilder` — e.g. `Board$new(width, height, ..., .ptr = NULL)`
skips all argument validation and just does `private$.ptr <- .ptr` when
`.ptr` is supplied). Because the skipped formals (`width`, `height`, etc.)
have no default and are never forced in that code path, R's lazy argument
evaluation means `Board$new(.ptr = raw_ptr)` works fine with no other
arguments — confirmed as the workaround:
```r
raw_ptr <- plan$build(40L, 40L)
board <- Board$new(.ptr = raw_ptr)   # now a real, usable Board
```
So the `.ptr` factory-wrapping mechanism is general (not scoped to "a
class's own static method returning `Self`", as the class-systems skill's
existing description implies) — it just isn't wired up for the "instance
method on type A returns type B" case. This looks like exactly the missing
piece needed to make cross-type builder `build()` methods usable directly;
worth a `gh issue create` to have the codegen detect a method's return type
naming a *different* known `#[miniextendr]`-registered ExternalPtr class and
emit `ReturnType::new(.ptr = .val)` instead of a bare `.val`.

### rust-target — checked, NOT a bug

`.Rbuildignore`'s `^rust-target$` and `.gitignore`'s `rust-target/` both
correctly match the real `CARGO_TARGET_DIR`
(`/Users/elea/Documents/GitHub/golife/rust-target`, confirmed in `configure`
output). No fix needed here — flagging only because it was worth checking
given hiccup #8 found two sibling patterns that *were* broken.

