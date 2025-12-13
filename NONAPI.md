# Non-API R Functions Tracking

This document tracks usage of non-API R functions in miniextendr.
Non-API functions are detected by `R CMD check` via `tools:::.check_so_symbols`.

Reference: <https://svn.r-project.org/R/trunk/src/library/tools/R/sotools.R>

## Currently Used Non-API Functions

| Function | Location | Feature Gate | Notes |
|----------|----------|--------------|-------|
| (none currently) | | | |

## Feature-Gated Non-API Functions

| Function | Location | Feature Gate | Notes |
|----------|----------|--------------|-------|
| `DATAPTR` | `ffi.rs:416` | `nonapi` | Mutable data pointer - prefer `DATAPTR_RO` or `DATAPTR_OR_NULL` |
| `R_CStackStart` | `ffi.rs:618` | `nonapi` | Stack top address - needed for thread safety |
| `R_CStackLimit` | `ffi.rs:624` | `nonapi` | Stack limit - set to `usize::MAX` to disable checking |
| `R_CStackDir` | `ffi.rs:630` | `nonapi` | Stack growth direction (-1 = down, 1 = up) |

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
- `SET_FORMALS`, `SET_BODY`, `SET_CLOENV`, `Rf_findVarInFrame3`
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

### R_ext/Connections.h non-API

- `R_new_custom_connection`, `R_ReadConnection`
- `R_WriteConnection`, `R_GetConnection`

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
