#include "cli_wrapper.h"

// Static wrappers

void cli_progress_add__extern(SEXP bar, double inc) { cli_progress_add(bar, inc); }
SEXP cli_progress_bar__extern(double total, SEXP config) { return cli_progress_bar(total, config); }
void cli_progress_done__extern(SEXP bar) { cli_progress_done(bar); }
void cli_progress_init_timer__extern(void) { cli_progress_init_timer(); }
int cli_progress_num__extern(void) { return cli_progress_num(); }
void cli_progress_set__extern(SEXP bar, double set) { cli_progress_set(bar, set); }
void cli_progress_set_clear__extern(SEXP bar, int clear) { cli_progress_set_clear(bar, clear); }
void cli_progress_set_name__extern(SEXP bar, const char *name) { cli_progress_set_name(bar, name); }
void cli_progress_set_status__extern(SEXP bar, const char *status) { cli_progress_set_status(bar, status); }
void cli_progress_set_type__extern(SEXP bar, const char *type) { cli_progress_set_type(bar, type); }
void cli_progress_update__extern(SEXP bar, double set, double inc, int force) { cli_progress_update(bar, set, inc, force); }
void cli_progress_sleep__extern(int s, long ns) { cli_progress_sleep(s, ns); }
