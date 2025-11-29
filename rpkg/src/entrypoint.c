// NOTICE: Any changes to this file, must also be applied to configure.ac's embedded version of this file!
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

extern void R_init_rpkg_miniextendr(DllInfo *dll);
extern void miniextendr_altrep_init(void);
extern void miniextendr_panic_hook(void);
extern void miniextendr_worker_init(void);

void R_init_rpkg(DllInfo *dll) {
    miniextendr_panic_hook();
    miniextendr_altrep_init();
    miniextendr_worker_init();
    R_init_rpkg_miniextendr(dll);

    R_useDynamicSymbols(dll, FALSE);
    R_forceSymbols(dll, TRUE);
}
