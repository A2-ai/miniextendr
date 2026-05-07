# Issue #318 — `call_attribution_demo` fixture pollutes public namespace

## Decision
Apply option 1 from the issue (`@keywords internal`), not option 3 (move to
testthat). Keep the doc transcript in `docs/CALL_ATTRIBUTION.md` runnable but
require `miniextendr:::call_attr_with(...)` (triple-colon prefix).

## Changes
1. `rpkg/src/rust/call_attribution_demo.rs`:
   - Replace each `/// @export` line with `/// @keywords internal`.
   - Keep both function bodies and signatures unchanged.
2. `docs/CALL_ATTRIBUTION.md`:
   - Update the runnable transcript to invoke `miniextendr:::call_attr_with(...)`
     and `miniextendr:::unsafe_C_call_attr_without(...)`.
   - Add a one-line note explaining these are internal demo fixtures.
3. Regenerate `NAMESPACE` and `man/*.Rd` via
   `bash ./configure && just rcmdinstall && just devtools-document` (run from rpkg/).

## Acceptance
- `Rscript -e 'getNamespaceExports("miniextendr")' | grep -c call_attr` returns 0.
- `man/call_attribution_demo.Rd` either disappears or carries `\keyword{internal}`.
- Doc transcript still works against an installed package (manual verify).
- Regenerated `R/miniextendr-wrappers.R`, `NAMESPACE`, `man/` committed in sync.
