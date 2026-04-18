# miniextendr-macros refactor

Branch: `review/macros-refactor`

Tracks 20 tasks from the review of `miniextendr-macros` and
`miniextendr-macros-core`. Priority order (cheap → structural):

1. Delete dead `force_main_thread` field (#1)
2. Delete empty `impl ThreadStrategy {}` (#2)
3. Move `class_ref_or_verbatim` to shared module (#5)
4. Feature-gate `VCTRS_GENERICS` (#3)
5. Add `normalize_r_arg_str` (#6)
6. Extract `FN_OPTIONS_HELP` constant (#7)
7. Validate package name in `miniextendr_init!` (#8)
8. Extract match_arg placeholder key helpers (#4)
9. Move util helpers out of `lib.rs` (#12)
10. Remove duplicate struct initializer in `MiniextendrFnAttrs::parse` (#11)
11. Fuse three `@param` generation loops (#10)
12. Move extern signature validation before codegen (#9)
13. Extract `parse_lit_str` + data-drive bool flags + delete paren-bool form (#13, #14)
14. Unify `MatchArg` registry entries (#15)
15. Decompose `miniextendr` fn body (#17)
16. Retarget `miniextendr` fn codegen at `CWrapperContext` (#19)
17. Collapse `per_param_*` into `ParamAttrs` map (#16)
18. Split `MethodAttrs` per class system (#20)
19. Absorb `miniextendr-macros-core` (#18)

Commit in phases; green-build between each phase.
