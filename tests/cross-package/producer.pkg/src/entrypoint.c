#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

extern void R_init_producer_pkg_miniextendr(DllInfo *dll);

// NOTE: miniextendr_encoding_init() is disabled for R packages because it
// references non-API symbols (utf8locale, etc.) that aren't exported from libR.
// It only works when embedding R via miniextendr-engine.
// extern void miniextendr_encoding_init(void);
extern void miniextendr_assert_utf8_locale(void);
extern void miniextendr_panic_hook(void);
extern void miniextendr_runtime_init(void);

// Set ALTREP package name before ALTREP registration
extern void miniextendr_set_altrep_pkg_name(const char* name);

// Optional vctrs support initialization
// Returns: 0=success, 1=not available, 2=not main thread, 3=already initialized
// When vctrs feature is disabled, always returns 1 (not available)
extern int miniextendr_init_vctrs(void);

// Trait ABI C-callable registration (defined in mx_abi.c)
extern void mx_abi_register(void);

void R_init_producer_pkg(DllInfo *dll) {
    miniextendr_panic_hook();
    miniextendr_runtime_init();
    miniextendr_assert_utf8_locale();

    // Set ALTREP package name before ALTREP registration
    miniextendr_set_altrep_pkg_name("producer_pkg");

    // Register trait ABI C-callables (mx_wrap, mx_get, mx_query)
    mx_abi_register();

    // Optional: initialize vctrs C API support
    // Status 1 (not available) is fine - vctrs is optional
    (void)miniextendr_init_vctrs();

    R_init_producer_pkg_miniextendr(dll);

    R_useDynamicSymbols(dll, FALSE);
    R_forceSymbols(dll, TRUE);
}
