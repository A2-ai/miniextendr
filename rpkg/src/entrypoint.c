// NOTICE: Any changes to this file, must also be applied to configure.ac's embedded version of this file!
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

extern void R_init_rpkg_miniextendr(DllInfo *dll);
extern void miniextendr_altrep_init(void);
extern void register_miniextendr_panic_hook(void);

void R_init_rpkg(DllInfo *dll) {
    register_miniextendr_panic_hook();
    miniextendr_altrep_init();
    R_init_rpkg_miniextendr(dll);

    R_useDynamicSymbols(dll, FALSE);
    R_forceSymbols(dll, TRUE);
}
