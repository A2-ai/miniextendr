# Non-API R Functions Tracking

This document tracks usage of non-API R functions in miniextendr.
Non-API functions are detected by `R CMD check` via `tools:::.check_so_symbols`.

Reference: <https://svn.r-project.org/R/trunk/src/library/tools/R/sotools.R>

## Currently Used Non-API Functions

| Function | Location | Feature Gate | Notes |
|----------|----------|--------------|-------|
| (none currently) | | | |

## Feature-Gated Non-API Functions

Locations intentionally omit line numbers (they drift); use search on the symbol name.

| Function | Location | Feature Gate | Notes |
|----------|----------|--------------|-------|
| `DATAPTR` | `miniextendr-api/src/sys.rs` | `nonapi` | Mutable data pointer - prefer `DATAPTR_RO` or `DATAPTR_OR_NULL` |
| `ATTRIB` | `miniextendr-api/src/sys.rs` | `nonapi` | Attribute pairlist. Test for attributes with `SexpExt::has_attributes()`, read one with `SexpExt::get_attr()`, compare objects with `SexpExt::is_identical()`. Lint MXL304 flags uses and hand-written declarations |
| `SET_ATTRIB` | `miniextendr-api/src/sys.rs` | `nonapi` | Replace the attribute pairlist. Use `SexpExt::set_attr()` for one attribute, `DUPLICATE_ATTRIB` / `SHALLOW_DUPLICATE_ATTRIB` to copy all of them. MXL304 |
| `OBJECT`, `SET_OBJECT` | `miniextendr-api/src/sys.rs` | `nonapi` | Object bit. Use `SexpExt::is_object()` (`Rf_isObject`); setting a `class` attribute sets the bit |
| `LEVELS`, `SETLEVELS` | `miniextendr-api/src/sys.rs` | `nonapi` | The `gp` header field, not factor levels (those are the `levels` attribute, `SexpExt::get_levels()`) |
| `R_CStackStart` | `miniextendr-api/src/sys.rs` (`nonapi_stack`) | `nonapi` | Process-global stack top; package code must not rewrite it for secondary-thread R calls |
| `R_CStackLimit` | `miniextendr-api/src/sys.rs` (`nonapi_stack`) | `nonapi` | Process-global stack limit; disabling it does not make R's API thread-safe |
| `R_CStackDir` | `miniextendr-api/src/sys.rs` (`nonapi_stack`) | `nonapi` | Process-global stack growth direction (-1 = down, 1 = up) |
| `ptr_R_WriteConsoleEx` | `miniextendr-api/src/progress.rs` | `indicatif` (implies `nonapi`) | R console hook used by indicatif TermLike backend |

Note: `miniextendr-engine` is entirely non-API (uses Rembedded.h/Rinterface.h for embedding R) and is not tracked here.

## Test Utilities (Not Part of R Package)

The file `miniextendr-api/tests/r_test_utils.rs` contains a `disable_r_stack_checking()` function
that directly accesses `R_CStackLimit`. This code is **never part of the R package** because:

1. It's in the `tests/` directory of `miniextendr-api`, which is only compiled for `cargo test`
2. It uses `miniextendr-engine` to embed R for Rust unit tests
3. The example R package (`rpkg/`) only vendors `miniextendr-api/src/` and `miniextendr-macros/src/`
4. Test directories are excluded from vendoring

This test utility exists because `cargo test` embeds R in a non-standard way
(not via `R CMD`). It is test-harness infrastructure, not permission for an R
package to call R APIs from secondary threads. The package-facing helpers and
fixtures that currently suggest otherwise are tracked for removal in #1352.

## API Functions (Safe to Use)

These functions are NOT in the non-API list and are safe:

| Function | Status |
|----------|--------|
| `DATAPTR_RO` | API (safe) |
| `DATAPTR_OR_NULL` | API (safe) |
| `R_ExternalPtrAddr` | API (safe) |
| `R_ExternalPtrTag` | API (safe) |
| `R_ExternalPtrProtected` | API (safe) |
| `R_MakeExternalPtr` | API (safe) |
| `Rf_allocVector` | API (safe) |
| `Rf_protect` / `Rf_unprotect` | API (safe) |
| `Rf_install` | API (safe) |
| `Rf_xlength` | API (safe) |
| `STRING_ELT` | API (safe) |
| `R_CHAR` | API (safe) |
| `TYPEOF` | API (safe) |
| `R_getVarEx` | API (safe, R >= 4.5.0) — single-frame/inherited variable lookup with an `ifnotfound` default; replaces the now-removed `Rf_findVarInFrame` |
| `ANY_ATTRIB` | API (safe, R >= 4.5.0) — whether `ATTRIB(x)` is non-empty. R 4.4's libR does not export it, so `SexpExt::has_attributes()` evaluates `attributes(quote(x))` instead, which works on every supported R |
| `R_compute_identical` | API (safe) — R's `identical()`; `SexpExt::is_identical()` passes `identical()`'s default flags (16) |
| `DUPLICATE_ATTRIB` / `SHALLOW_DUPLICATE_ATTRIB` | API (safe) — copy every attribute (and the object bit) from one object to another |

`TRUELENGTH` is not declared at all: R 4.5's `R CMD check` reports it as
non-API, and R 4.6's libR no longer exports it, so a package referencing it
cannot load there.

### Choosing between `ANY_ATTRIB` and `attributes()`

Writing R Extensions recommends `ANY_ATTRIB` (added in R 4.5.0) in place of
`ATTRIB` for testing whether an object has attributes. miniextendr supports
R 4.4, whose libR does not export `ANY_ATTRIB`, and R loads a package's shared
object with every symbol resolved up front. A safe wrapper that called
`ANY_ATTRIB` would make every package built with it fail to load on R 4.4,
whether or not the wrapper ever ran. So `SexpExt::has_attributes()` evaluates
`attributes(quote(x))` on every R version. That allocates a call and, when
attributes exist, the list `attributes()` returns.

The two answers differ in two cases. `attributes()` reports the tag names of
a pairlist as a `names` attribute, while `ANY_ATTRIB` ignores them;
`has_attributes()` follows `attributes()`, so `pairlist(a = 1)` has
attributes. A `CHARSXP` keeps R's string-cache chain in its attribute slot, so
`ANY_ATTRIB` can be non-zero for one; `has_attributes()` returns `false`, as
`identical()` skips that slot too.

To reproduce `identical(x, FALSE)`, call
`x.is_identical(SEXP::scalar_logical(false))` rather than combining a value
check with an attribute check.

## Full Non-API List (R 4.5.x trunk)

<details>
<summary>Click to expand complete list</summary>

### Obsolete

- `chol_`, `chol2inv_`, `cg_`, `ch_`, `rg_`
- `fft_factor`, `fft_work`, `Brent_fmin`, `optif0`

### Non-API in headers or no header

- `OutDec`, `PRIMOFFSET`, `RC_fopen`, `R_CollectFromIndex`
- `R_CompiledFileName`, `R_FileExists`
- `R_FreeStringBuffer`, `R_FunTab`, `R_GE_setVFontRoutines`
- `R_GetVarLocMISSING`, `R_MethodsNamespace`, `R_NewHashedEnv`
- `R_OpenCompiledFile`, `R_PV`, `R_ParseContext`
- `R_ParseContextLast`, `R_ParseContextLine`
- `R_ParseError`, `R_ParseErrorMsg`, `R_SrcfileSymbol`
- `R_SrcrefSymbol`, `R_Visible`, `R_addTaskCallback`
- `R_cairoCdynload`, `R_data_class`
- `R_deferred_default_method`, `R_execMethod`
- `R_findVarLocInFrame`, `R_fopen`, `R_gc_torture`
- `R_getTaskCallbackNames`, `R_get_arith_function`
- `R_gzclose`, `R_gzgets`, `R_gzopen`, `R_ignore_SIGPIPE`
- `R_isForkedChild`, `R_isMethodsDispatchOn`
- `R_moduleCdynload`, `R_primitive_generic`
- `R_primitive_methods`, `R_print`, `R_removeTaskCallback`
- `R_running_as_main_program`, `R_setInternetRoutines`
- `R_setLapackRoutines`, `R_setX11Routines`
- `R_set_prim_method`, `R_set_quick_method_check`
- `R_set_standardGeneric_ptr`, `R_strtod4`
- `R_subassign3_dflt`, `R_taskCallbackRoutine`
- `Rconn_fgetc`, `Rconn_printf`, `Rdownload`
- `Rf_EncodeComplex`, `Rf_EncodeElement`
- `Rf_EncodeEnvironment`, `Rf_EncodeInteger`
- `Rf_EncodeLogical`, `Rf_EncodeReal`, `Rf_GPretty`
- `Rf_NewEnvironment`, `Rf_PrintDefaults`
- `Rf_ReplIteration`, `Rf_Seql`, `Rf_addTaskCallback`
- `Rf_begincontext`, `Rf_callToplevelHandlers`
- `Rf_checkArityCall`, `Rf_con_pushback`
- `Rf_copyMostAttribNoTs`, `Rf_deparse1`, `Rf_deparse1line`
- `Rf_dpptr`, `Rf_endcontext`, `Rf_envlength`
- `Rf_formatComplex`, `Rf_formatInteger`
- `Rf_formatLogical`, `Rf_formatReal`, `Rf_init_con`
- `Rf_isProtected`, `Rf_mbrtowc`, `Rf_mkFalse`
- `Rf_printNamedVector`, `Rf_printRealVector`
- `Rf_printVector`, `Rf_removeTaskCallbackByIndex`
- `Rf_removeTaskCallbackByName`, `Rf_set_iconv`
- `Rf_sortVector`, `Rf_strIsASCII`, `Rf_strchr`
- `Rf_strrchr`, `Rf_ucstomb`, `Rf_utf8towcs`
- `Rf_wcstoutf8`, `Rg_PolledEvents`, `Rg_set_col_ptrs`
- `Rf_wait_usec`, `Ri18n_*`, `Rsock*`, `Runzip`
- `UNIMPLEMENTED_TYPE`, `baseRegisterIndex`
- `Rf_csduplicated`, `Rf_currentTime`
- `dcar`, `dcdr`, `do_*`, `dqrrsd_`, `dqrxb_`, `dtype`
- `dummy_*`, `epslon_`, `extR_*`, `fdhess`
- `getConnection`, `getPRIMNAME`, `known_to_be_latin1`
- `locale2charset`, `match5`, `matherr`
- `max_contour_segments`, `Rf_mbcsToUcs2`, `Rf_memtrace_report`
- `parseError`, `pythag_`, `rs_`, `rwarnc_`
- `tql2_`, `tqlrat_`, `tred1_`, `tred2_`, `utf8locale`, `yylloc`
- `R_opendir`, `R_readdir`, `R_closedir`

### Rinternals.h non-API

- `ENSURE_NAMEDMAX`, `IS_ASCII`, `IS_UTF8`, `SET_PRSEEN`, `ddfind`
- `SET_TYPEOF`, `SET_OBJECT`, `SET_S4_OBJECT`, `UNSET_S4_OBJECT`
- `SETLENGTH`, `SET_TRUELENGTH`, `SETLEVELS`
- `SET_ENVFLAGS`, `SET_FRAME`, `SET_ENCLOS`, `SET_HASHTAB`
- `SET_PRENV`, `SET_PRVALUE`, `SET_PRCODE`, `STDVEC_DATAPTR`
- `IS_GROWABLE`, `SET_GROWABLE_BIT`, `SET_NAMED`
- `R_PromiseExpr`, `R_tryWrap`
- `DDVAL`, `NAMED`, `INTERNAL`, `SYMVALUE`, `PRSEEN`
- `INTEGER0`, `LOGICAL0`, `RAW0`, `REAL0`, `COMPLEX0`
- `LEVELS`, `FRAME`, `HASHTAB`, `ENVFLAGS`, `RDEBUG`, `SET_RDEBUG`
- `STRING_PTR`, `VECTOR_PTR`, `DATAPTR`, `STDVEC_DATAPTR`
- `SET_FORMALS`, `SET_BODY`, `SET_CLOENV`
- `PRCODE`, `PRENV`, `PRVALUE`, `R_nchar`
- `Rf_NonNullStringMatch`, `TRUELENGTH`, `XLENGTH_EX`
- `XTRUELENGTH`, `Rf_gsetVar`
- `Rf_isValidString`, `Rf_isValidStringF`
- `R_shallow_duplicate_attr`
- `EXTPTR_PROT`, `EXTPTR_TAG`, `EXTPTR_PTR`
- `OBJECT`, `IS_S4_OBJECT`
- `Rf_allocSExp`, `Rf_isFrame`
- `BODY`, `FORMALS`, `CLOENV`, `ENCLOS`
- `R_Pretty`

### Rinterface.h / Rembedded.h non-API

- `AllDevicesKilled`, `R_CStackLimit`, `R_CStackStart`
- `R_ClearerrConsole`, `R_CleanTempDir`, `R_Consolefile`
- `R_DefCallbacks`, `R_DefParams`, `R_DefParamsEx`
- `R_DirtyImage`, `R_GUIType`, `R_GlobalContext`
- `R_HistoryFile`, `R_HistorySize`, `R_Home`, `R_HomeDir`
- `R_Interactive`, `R_Outputfile`
- `R_PolledEvents`, `R_ReplDLLdo1`, `R_ReplDLLinit`
- `R_RestoreGlobalEnv`, `R_RestoreGlobalEnvFromFile`
- `R_RestoreHistory`, `R_RunExitFinalizers`, `R_SaveGlobalEnv`
- `R_SaveGlobalEnvToFile`, `R_SelectEx`, `R_SetParams`
- `R_SetWin32`, `R_SignalHandlers`, `R_SizeFromEnv`, `R_NoEcho`
- `R_Suicide`, `R_TempDir`, `R_checkActivity`
- `R_checkActivityEx`, `R_runHandlers`
- `R_setStartTime`, `R_set_command_line_arguments`
- `R_setupHistory`, `R_timeout_handler`, `R_timeout_val`
- `R_wait_usec`, `RestoreAction`, `Rf_CleanEd`
- `Rf_KillAllDevices`, `Rf_endEmbeddedR`, `Rf_initEmbeddedR`
- `Rf_initialize_R`, `Rf_jump_to_toplevel`, `Rf_mainloop`
- `SaveAction`, `editorcleanall`, `fpu_setup`
- `freeRUser`, `free_R_HOME`
- `getDLLVersion`, `getRUser`, `get_R_HOME`
- `getSelectedHandler`, `initStdinHandler`
- `process_site_Renviron`, `process_system_Renviron`
- `process_user_Renviron`, `ptr_R_*`
- `readconsolecfg`, `run_Rmainloop`, `setup_Rmainloop`

### R_ext/Connections.h non-API (before R 4.6)

- `R_new_custom_connection`, `R_ReadConnection`
- `R_WriteConnection`, `R_GetConnection`

R 4.6 lists these as experimental API (`tools:::funAPI()` gives `eapi`). R
4.4 and 4.5 export the same symbols and ship the same header, but
`R CMD check` notes them there as non-API calls. Built against R < 4.6, the
`connections` feature reaches a connection's state and bytes through R
functions instead (`CONNECTIONS.md`, "R versions before 4.6"), and only
`R_new_custom_connection`, which has no R-level alternative, remains.

### Deprecated/removed

- `call_R` (removed in R 4.5.0)
- `Rf_setSVector`

</details>

## Feature Gate Strategy

When using non-API functions, wrap them in a `nonapi` feature:

```rust
#[cfg(feature = "nonapi")]
pub fn DATAPTR(x: SEXP) -> *mut c_void;
```

This allows users to opt-in to non-API usage while keeping the default build CRAN-compatible.

## R-version cfgs

A package is compiled against the R it installs into, so the build knows the
R version. `miniextendr-api`'s `build.rs` reads `R_MAJOR` / `R_MINOR` from
`$R_HOME/include/Rversion.h` and sets one cfg per listed version at or below
it, cumulatively: `r_4_5` on R 4.5 and later, `r_4_6` on R 4.6 and later
(#1905). A build that finds no header (a wasm32 target, CI's lint job with an
empty `R_HOME`) sets them all.

Use them for an entry point that a newer R adds, or that a newer R makes
API:

- **R adds it.** Gate the binding in `sys.rs` on the cfg (`ANY_ATTRIB` and
  `R_getVarEx` are `#[cfg(r_4_5)]`), and keep a fallback beside every use for
  the older R. Referencing an entry point the R lacks fails at load, because
  packages load with `dyn.load(now = TRUE)`.
- **R makes it API.** The entry point exists before that version, but
  `R CMD check` notes it as non-API there. Gate the call on the cfg when the
  older R offers another route to the same result, and keep the entry point
  on the newer side: `connection/access.rs` reads a connection through
  `R_GetConnection` under `r_4_6` and through `summary.connection()` /
  `readBin()` / `writeBin()` before.

Gate a **public** item on a cfg only in the first case. There a missing
entry point would fail at load, and the gate turns that into a compile error.
In the second, the entry point works on the older R, so keep the public
signature the same on every R and switch the implementation inside it:
packages compile on each user's machine against that user's R, so an item
that exists only on newer R breaks the install for a user on an older R while
its author, on a newer one, never sees it. Gate a public item there only when
the older R has no route to what it returns at all (`get_connection`, the raw
handle, is R >= 4.6 only).

To add a version, append it to `R_VERSION_CFGS` in `miniextendr-api/build.rs`.
Test both sides: CI runs `R CMD check` on R 4.4 and release on every PR, and
on oldrel too on main or when the PR carries the `oldrel` label (add it to a
PR that touches these cfgs). The lint job compiles the newest side.

