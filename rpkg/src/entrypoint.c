#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

SEXP C_add(SEXP _left, SEXP _right);
SEXP C_add(SEXP _left, SEXP _right);
SEXP C_add_panic(SEXP _left, SEXP _right);
SEXP C_add_r_error(SEXP _left, SEXP _right);

/* Register native routines (.Call shown; add others if needed) */
static const R_CallMethodDef CallEntries[] = {
    /* {"C_fn", (DL_FUNC) &C_fn, 2}, */
    {"C_add", (DL_FUNC) &C_add, 2},
    {"C_add_r_error", (DL_FUNC) &C_add_r_error, 2},
    {"C_add_panic", (DL_FUNC) &C_add_panic, 2},
    // {"C_unwind_add", (DL_FUNC) &C_unwind_add, 2},
    {NULL, NULL, 0}
};

void R_init_rpkg(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
    R_forceSymbols(dll, TRUE);
}
