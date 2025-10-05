#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

/* Register native routines (.Call shown; add others if needed) */
static const R_CallMethodDef CallEntries[] = {
    /* {"C_fn", (DL_FUNC) &C_fn, 2}, */
    {NULL, NULL, 0}
};

void R_init_rpkgs(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
    R_forceSymbols(dll, TRUE);
}
