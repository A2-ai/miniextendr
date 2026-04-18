# miniextendr-macros refactor

Branch: `review/macros-refactor`

Tracks 20 refactor items from the review of `miniextendr-macros` and
(now-defunct) `miniextendr-macros-core`.

## Done (19 / 20)

### Quick cleanups
1. [#1] Delete dead `force_main_thread` field.
2. [#2] Delete empty `impl ThreadStrategy {}`.
3. [#3] Feature-gate / move `VCTRS_GENERICS` out of lib.rs.
4. [#5] Move `class_ref_or_verbatim` / `is_bare_identifier` to shared module.
5. [#6] Add `normalize_r_arg_string` to skip `Ident` round-trip.
6. [#8] Validate package name in `miniextendr_init!` (compile_error, not panic).

### Attribute parsing
7. [#7] Extract `FN_BOOL_FLAGS_HELP` / `FN_NESTED_OPTIONS_HELP` constants.
8. [#11] Remove duplicate struct initializer in `MiniextendrFnAttrs::parse`.
9. [#13] Extract `parse_lit_str` helper and data-drive bool flags.
10. [#14] Delete paren-bool form (`strict(true)`) from fn attrs.

### match_arg pipeline
11. [#4] Extract match_arg placeholder key helpers (`match_arg_keys.rs`).
12. [#15] Share `MatchArg` entry token builders between lib.rs and miniextendr_impl.rs.

### Codegen path
13. [#9] Move extern signature validation before codegen.
14. [#10] Fuse three `@param` generation loops into one.
15. [#12] Move util helpers out of lib.rs (`util.rs`, `type_inspect.rs`).
16. [#17] Decompose `miniextendr` fn body — extracted `build_match_arg_helpers`
    and `build_fn_c_wrapper` (with collapsed worker-thread variants).

### Data shapes
17. [#16] Collapse five per-param fields on `MiniextendrFunctionParsed` into a
    single `HashMap<String, ParamAttrs>`.
18. [#20] Split `MethodAttrs` per class system — partial:
    - Per-param fields (`per_param_match_arg`/`_several_ok`/`_choices`)
      collapsed into a single `HashMap<String, ParamAttrs>`.
    - S7-specific fields (13) moved into `S7MethodAttrs` sub-struct
      (`method_attrs.s7.*`).
    - R6-prefixed fields (`r6_setter`, `r6_prop`, `active_span`) moved into
      `R6MethodAttrs` sub-struct (`method_attrs.r6.*`).
    - Remaining R6 booleans (`active`/`private`/`finalize`/`deep_clone`) and
      S3/S4-related fields stay at the top level because they're read through
      cross-cutting accessors (`is_active`, `is_private`, `is_finalizer`) or
      are shared between class systems (`generic`, `class`, `as_coercion`).

### Crate structure
19. [#18] Absorb `miniextendr-macros-core` into `miniextendr-macros`; drop the
    crate, its vendor copy, workspace member, and every leaf manifest listing.

## Deferred (1 / 20)

### [#19] Retarget standalone-fn codegen at `CWrapperContext`

`lib.rs::build_fn_c_wrapper` still hand-rolls three variants of the
`extern "C-unwind"` wrapper (main-thread × `{rng, no-rng}` × error_in_r
collapsed into a single-branch shape), mirroring what
`c_wrapper_builder::CWrapperContext` already does for impl methods.

Routing the fn path through the builder is not purely mechanical:

- `CWrapperContext::build_c_params` mangles parameter names to `arg_0`,
  `arg_1`, … in the C wrapper signature. The standalone-fn path preserves
  the user's original identifiers, which are visible in rustdoc. Switching
  would be a user-observable rename.
- `CWrapperContext` always emits wrappers with the default visibility;
  the fn path honors `#vis` from the user's source. Would need a builder
  hook for visibility.
- The fn path has dots-specific handling (`Dots` / `SEXP` inputs pin the
  function to the main thread) that `CWrapperContext` handles via
  `thread_strategy` set at build time; the routing site would need to
  reproduce the current auto-detection logic.

None of these are showstoppers, but the change needs a dedicated pass
with its own trybuild coverage to verify the C-wrapper signature /
rustdoc diff and any call-site fallout. Left for a follow-up PR.
