# miniextendr-macros refactor

Branch: `review/macros-refactor`

Tracks 20 refactor tasks from the review of `miniextendr-macros` and
(now-defunct) `miniextendr-macros-core`.

## Done (17 / 20)

1. [#1] Delete dead `force_main_thread` field.
2. [#2] Delete empty `impl ThreadStrategy {}`.
3. [#3] Feature-gate / move `VCTRS_GENERICS` out of lib.rs.
4. [#4] Extract match_arg placeholder key helpers (`match_arg_keys.rs`).
5. [#5] Move `class_ref_or_verbatim` / `is_bare_identifier` to shared module.
6. [#6] Add `normalize_r_arg_string` to skip `Ident` round-trip.
7. [#7] Extract `FN_BOOL_FLAGS_HELP` / `FN_NESTED_OPTIONS_HELP` constants.
8. [#8] Validate package name in `miniextendr_init!` (compile_error, not panic).
9. [#9] Move extern signature validation before codegen.
10. [#10] Fuse three `@param` generation loops into one.
11. [#11] Remove duplicate struct initializer in `MiniextendrFnAttrs::parse`.
12. [#12] Move util helpers out of lib.rs (`util.rs`, `type_inspect.rs`).
13. [#13] Extract `parse_lit_str` helper and data-drive bool flags.
14. [#14] Delete paren-bool form (`strict(true)`) from fn attrs.
15. [#15] Share `MatchArg` entry token builders between lib.rs and miniextendr_impl.rs.
16. [#16] Collapse five per-param fields on `MiniextendrFunctionParsed` into a
    single `HashMap<String, ParamAttrs>`.
17. [#18] Absorb `miniextendr-macros-core` into `miniextendr-macros`; drop the
    crate, its vendor copy, workspace member, and every leaf manifest listing.

## Deferred (3 / 20)

### [#17] Decompose `miniextendr` fn body — **partial**

Extracted `build_match_arg_helpers` (~55 lines lifted out). The entry
point is still large (~1100 lines): the R wrapper string assembly,
registration emission, and worker-thread wrapper selection all live
inline.

Full decomposition likely subsumes #19 — see below.

### [#19] Retarget standalone-fn codegen at `CWrapperContext`

Currently lib.rs hand-rolls four variants of the `extern "C-unwind"`
wrapper (main-thread × rng × error_in_r), mirroring what
`c_wrapper_builder::CWrapperContext` already does for impl methods.
Routing the fn path through the builder would delete ~400 duplicated
lines. Not attempted this sweep — requires careful handling of the
Dots/SEXP-arg thread-strategy constraints that are currently open-coded
in the fn body.

### [#20] Split `MethodAttrs` per class system

`miniextendr_impl.rs::MethodAttrs` has 40+ fields mixing cross-cutting
attrs with class-system-specific ones (R6 active/setter, S7
getter/setter/prop/default/…/convert, vctrs protocol, S3 as_coercion, …).
An enum `ClassSystemMethodAttrs { R6(…), S7(…), … }` would make
class-system specific fields a type error outside the right branch.

Not attempted this sweep — touches all six class generators and
every destructuring site.
