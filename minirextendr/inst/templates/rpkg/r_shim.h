/* r_shim.h — local include that locally suppresses clang 21+'s
 * "-Wunknown-warning-option" warning emitted for upstream R's
 *   #pragma clang diagnostic ignored "-Wfixed-enum-extension"
 * in Boolean.h (R 4.5+). The pragma is upstream R, not us.
 *
 * Why local push/pop instead of -Wno-unknown-warning-option in PKG_CFLAGS:
 * R CMD check --as-cran flags any non-portable "warning suppressor" flag
 * in PKG_CFLAGS, producing a CI-blocking WARNING for downstream packages
 * that don't pin error-on='"error"'. Scoped pragma keeps the flag out of
 * PKG_CFLAGS — see issue #443.
 *
 * Use this header *instead of* including <Rinternals.h> directly in
 * package C sources.
 */
#ifndef MINIEXTENDR_R_SHIM_H
#define MINIEXTENDR_R_SHIM_H

#if defined(__clang__)
#  pragma clang diagnostic push
#  pragma clang diagnostic ignored "-Wunknown-warning-option"
#endif

#include <Rinternals.h>

#if defined(__clang__)
#  pragma clang diagnostic pop
#endif

#endif /* MINIEXTENDR_R_SHIM_H */
