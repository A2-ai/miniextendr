# S7: a method for a not-yet-loaded package's S3 generic needs a bare-name binding

- **Upstream**: RConsortium/S7. No issue filed. Fixed on the main branch by
  [RConsortium/S7#721](https://github.com/RConsortium/S7/pull/721) (merged
  2026-07-08), not yet released: the latest release is 0.2.2 (2026-04-22).
  Related, not the same bug:
  [RConsortium/S7#573](https://github.com/RConsortium/S7/issues/573) (methods
  for classes of a suggested package).
- **Our issue**: none yet (not filed from the PR that added this note).
  We have not commented upstream, and no upstream post is needed.

## Impact

A package registers an S7 method for an S3 generic of a package it only
suggests, through `S7::new_external_generic()`. `S7::methods_register()` in
`.onLoad` defers the registration until that package loads. When the hook
fires, S7 0.2.2's `register_S3_method()` looks up the generic's bare name in
the registering package's namespace to find the generic's package
(`get0(generic$name, envir = envir)`). If the external generic was bound
anywhere else, for example inside `local()`, the lookup finds nothing,
`registerS3method()` then fails with "object 'tidy' not found" inside the
load hook (which `loadNamespace()` wraps in `try()`), and dispatch falls
through to "no applicable method".

Found in PR #1598, whose S7 codegen for `s7(generic = "pkg::name")` binds the
external generic under the bare name for this reason.

Reproduction: two pure-R probe packages (`Imports: S7`, `Suggests: generics`,
`.onLoad` calling `S7::methods_register()`, `export(Thing)`), installed into a
private library and called from a fresh `Rscript` with `generics` not loaded.
The failing one binds the generic inside `local()`:

```r
Thing <- new_class("Thing", package = "localbind")

# The external generic is bound only inside local().
local({
  tidy <- new_external_generic("generics", "tidy", "x")
  method(tidy, Thing) <- function(x, ...) "tidied a Thing"
})
```

The working one is identical except for a top-level
`tidy <- new_external_generic("generics", "tidy", "x")`. The driver script:

```r
pkg <- commandArgs(TRUE)[[1]]
cat("S7", format(packageVersion("S7")), "\n")
library(pkg, character.only = TRUE)
cat("generics loaded before the call:", isNamespaceLoaded("generics"), "\n")
x <- Thing()
print(generics::tidy(x))
```

Output on R 4.6.1 with S7 0.2.2:

```
===== localbind
S7 0.2.2
generics loaded before the call: FALSE
Error in get(genname, envir = envir) : object 'tidy' not found
Error in UseMethod("tidy") :
  no applicable method for 'tidy' applied to an object of class "c('localbind::Thing', 'S7_object')"
Calls: print -> <Anonymous>
Execution halted
===== topbind
S7 0.2.2
generics loaded before the call: FALSE
[1] "tidied a Thing"
```

With S7 main at `bee740f` (0.2.2.9000), both probes print
`[1] "tidied a Thing"`: #721 made `register_S3_method()` register into
`environment(generic$generic)`, the generic's own namespace.

## Workaround

Bind the external generic at the top level of the package under the generic's
bare name, as the wrappers generated under PR #1598 do.

## Closes when

S7 releases a version after 0.2.2 that includes #721, and miniextendr raises
its S7 floor to it. The bare-name binding can then stop being load-bearing.

## Last checked

2026-09-24: no upstream issue. #721 is merged on main; the latest release is
still 0.2.2.
