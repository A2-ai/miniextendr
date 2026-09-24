# Register S7 methods when the package loads. S7 records methods for
# generics from other packages (base generics such as format(), operators
# such as `[[` and `+`) when the package is built; S7::methods_register()
# registers them again in every new session.
# suppressMessages(): under devtools::load_all() methods for another
# package's S7 generic are registered twice, and S7 reports the second
# registration as "Overwriting method".
.onLoad <- function(libname, pkgname) {
  suppressMessages(S7::methods_register())
}
