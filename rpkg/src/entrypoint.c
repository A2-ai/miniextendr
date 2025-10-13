// NOTICE: Any changes to this file, must also be applied to configure.ac's embedded version of this file!
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

extern void R_init_rpkg_miniextendr(DllInfo * dll);

void R_init_rpkg(DllInfo *dll) {
    R_init_rpkg_miniextendr(dll);

    R_useDynamicSymbols(dll, FALSE);
    R_forceSymbols(dll, TRUE);
}
