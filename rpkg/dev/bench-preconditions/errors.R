# Error-path equivalence: the condition from the R guard vs the one from the
# Rust conversion (or Rust check), for the same bad input.
#
# Usage: Rscript rpkg/dev/bench-preconditions/errors.R <errors.csv> > errors.log
# Writes <errors.csv> (one row per input), <errors>-compact.txt (one line per
# input, headline counts first) and <errors>-inherits.csv (Rf_inherits vs
# base::inherits); the human-readable dump goes to stdout.
#
# Each call is built as `<fn>(<input>)` and evaluated in the namespace, so the
# condition's call reads as a user would write it. `call_same` compares the
# two calls with the function names unified.

out_csv <- commandArgs(trailingOnly = TRUE)
if (length(out_csv) < 1L) {
  stop("usage: Rscript rpkg/dev/bench-preconditions/errors.R <errors.csv>", call. = FALSE)
}
out_csv <- out_csv[[1]]
ns <- asNamespace("miniextendr")
`%||%` <- function(a, b) if (is.null(a)) b else a

capture <- function(fn, args) {
  cl <- as.call(c(as.name(fn), args))
  tryCatch({
    v <- eval(cl, ns)
    list(ok = TRUE, value = paste(format(v), collapse = " "), call_written = cl)
  }, error = function(e) list(ok = FALSE, e = e, call_written = cl))
}
fld <- function(r) {
  if (r$ok) {
    return(list(outcome = paste0("OK -> ", r$value), class = "", msg = "", param = "", rust_type = "", kind = "", call = ""))
  }
  e <- r$e
  cls <- setdiff(class(e), c("simpleError", "error", "condition"))
  list(
    outcome = "error",
    class = paste(cls, collapse = "/"),
    msg = conditionMessage(e),
    param = as.character(e$param %||% "<none>"),
    rust_type = as.character(e$rust_type %||% "<none>"),
    kind = as.character(e$kind %||% "<none>"),
    call = paste(deparse(conditionCall(e)), collapse = " ")
  )
}
unify <- function(s, a, b) gsub(paste0("\\b", b, "\\("), paste0(a, "("), s)

cases <- list()
add <- function(case, input_label, r_fn, rust_fn, args) {
  cases[[length(cases) + 1L]] <<- list(case = case, input = input_label, r_fn = r_fn, rust_fn = rust_fn, args = args)
}

# Scalars: default guards vs no_preconditions (the Rust conversion).
for (inp in list(
  list("\"a\"", list("a")), list("1:2", list(quote(1:2))), list("1.5", list(1.5)), list("2 (whole dbl)", list(2)),
  list("NA_integer_", list(NA_integer_)), list("NULL", list(NULL)), list("integer(0)", list(integer(0))),
  list("TRUE", list(TRUE)), list("factor(\"a\")", list(quote(factor("a")))), list("list(1L)", list(quote(list(1L))))
)) add("i32", inp[[1]], "pcb_i32", "pcb_i32_u", inp[[2]])
for (inp in list(
  list("\"a\"", list("a")), list("c(1, 2)", list(quote(c(1, 2)))), list("1L", list(1L)), list("NA_real_", list(NA_real_)),
  list("NA (logical)", list(NA)), list("NULL", list(NULL)), list("TRUE", list(TRUE)), list("numeric(0)", list(quote(numeric(0))))
)) add("f64", inp[[1]], "pcb_f64", "pcb_f64_u", inp[[2]])
for (inp in list(
  list("\"a\"", list("a")), list("c(TRUE, FALSE)", list(quote(c(TRUE, FALSE)))), list("1L", list(1L)),
  list("NA", list(NA)), list("NULL", list(NULL)), list("logical(0)", list(quote(logical(0))))
)) add("bool", inp[[1]], "pcb_bool", "pcb_bool_u", inp[[2]])
for (fn in c("str", "string")) {
  for (inp in list(
    list("1", list(1)), list("c(\"a\", \"b\")", list(quote(c("a", "b")))), list("NA_character_", list(NA_character_)),
    list("NULL", list(NULL)), list("factor(\"a\")", list(quote(factor("a")))), list("character(0)", list(quote(character(0))))
  )) add(if (fn == "str") "&str" else "String", inp[[1]], paste0("pcb_", fn), paste0("pcb_", fn, "_u"), inp[[2]])
}
for (inp in list(
  list("\"a\"", list("a")), list("1:2", list(quote(1:2))), list("1.5", list(1.5)), list("NA_integer_", list(NA_integer_)),
  list("NULL (ok)", list(NULL))
)) add("Option<i32>", inp[[1]], "pcb_opt_i32", "pcb_opt_i32_u", inp[[2]])
add("(a: i32, b: f64)", "a = \"x\", b = \"y\" (both bad)", "pcb_two", "pcb_two_u", list(a = "x", b = "y"))
add("(a: i32, b: f64)", "a = 1L, b = \"y\"", "pcb_two", "pcb_two_u", list(a = 1L, b = "y"))
add("(a: i32, b: f64)", "a = 1:2, b = 1L", "pcb_two", "pcb_two_u", list(a = quote(1:2), b = 1L))

# Vectors.
for (ty in c("slice_f64", "vec_f64")) {
  for (inp in list(
    list("\"a\"", list("a")), list("1:3", list(quote(1:3))), list("NULL", list(NULL)),
    list("list(1)", list(quote(list(1)))), list("TRUE", list(TRUE)), list("matrix(1, 2, 2)", list(quote(matrix(1, 2, 2)))),
    list("Sys.Date()", list(quote(Sys.Date())))
  )) add(if (ty == "slice_f64") "&[f64]" else "Vec<f64>", inp[[1]], paste0("pcb_", ty), paste0("pcb_", ty, "_u"), inp[[2]])
}
for (ty in c("slice_i32", "vec_i32")) {
  for (inp in list(
    list("c(1, 2)", list(quote(c(1, 2)))), list("\"a\"", list("a")), list("NULL", list(NULL)),
    list("TRUE", list(TRUE)), list("factor(c(\"a\", \"b\"))", list(quote(factor(c("a", "b"))))),
    list("1:3 (ok)", list(quote(1:3)))
  )) add(if (ty == "slice_i32") "&[i32]" else "Vec<i32>", inp[[1]], paste0("pcb_", ty), paste0("pcb_", ty, "_u"), inp[[2]])
}

# no_na: R anyNA() guard vs the Rust any_na (Either path, no R guard).
for (inp in list(
  list("c(1, NA)", list(quote(c(1, NA)))), list("c(1, NaN)", list(quote(c(1, NaN)))), list("\"a\"", list("a"))
)) add("no_na &[f64]", inp[[1]], "pcb_nona_slice_f64", "pcb_either_f64_nona", inp[[2]])
add("no_na &[f64]", "c(1, NA) [guard_na only vs Rust]", "pcb_nona_slice_f64_u", "pcb_either_f64_nona", list(quote(c(1, NA))))
add("no_na &[i32]", "c(1L, NA)", "pcb_nona_slice_i32", "pcb_either_i32_nona", list(quote(c(1L, NA))))

# coerce Vec<i32>: R whole-number guard vs the Rust element check.
for (inp in list(
  list("c(1.5, 2)", list(quote(c(1.5, 2)))), list("c(1.5, 2.5, 3)", list(quote(c(1.5, 2.5, 3)))),
  list("c(NA, 1)", list(quote(c(NA, 1)))), list("1e10", list(1e10)), list("NaN", list(NaN)), list("Inf", list(Inf)),
  list("\"a\"", list("a")), list("c(1, 2) (ok)", list(quote(c(1, 2))))
)) add("coerce Vec<i32>", inp[[1]], "pcb_coerce_vec_i32", "pcb_coerce_vec_i32_u", inp[[2]])

# inherits: R guard vs no guard (Rust has no class check on List today).
for (inp in list(
  list("list(1) (no class)", list(quote(list(1)))), list("1", list(1)),
  list("structure(1, class = \"mx_cls\")", list(quote(structure(1, class = "mx_cls"))))
)) {
  add("inherits List (guards: inherits+is.list vs inherits only)", inp[[1]], "pcb_inherits_list", "pcb_inherits_list_u", inp[[2]])
  add("inherits List (R inherits guard vs no check at all)", inp[[1]], "pcb_inherits_list_u", "pcb_list_u", inp[[2]])
}

# DataFrame: R inherits() guard vs the Rust DataFrame conversion.
for (inp in list(
  list("list(a = 1)", list(quote(list(a = 1)))), list("1", list(1)), list("NULL", list(NULL)),
  list("matrix(1, 2, 2)", list(quote(matrix(1, 2, 2)))),
  list("structure(list(a = 1:2, b = 1:3), class = \"data.frame\") (ragged)",
       list(quote(structure(list(a = 1:2, b = 1:3), class = "data.frame", row.names = 1:2))))
)) add("DataFrame", inp[[1]], "pcb_df_inherits", "pcb_df", inp[[2]])
add("DataFrame no_na", "data.frame(a = c(1, NA))", "pcb_df_nona", "pcb_df", list(quote(data.frame(a = c(1, NA)))))

# choices / match_arg: R helper vs the Rust MatchArg conversion (no helper).
for (inp in list(
  list("\"medium\"", list("medium")), list("1", list(1)), list("c(\"fast\", \"slow\")", list(quote(c("fast", "slow")))),
  list("NA_character_", list(NA_character_)), list("\"\"", list("")), list("NULL", list(NULL)),
  list("factor(\"slow\")", list(quote(factor("slow")))), list("\"sl\" (ok)", list("sl")),
  list("c(\"fast\", \"slow\", \"debug\") (the default vector)", list(quote(c("fast", "slow", "debug")))),
  list("<omitted>", list())
)) add("match_arg enum", inp[[1]], "pcb_match_arg", "pcb_mode_rust", inp[[2]])
add("choices &str (no Rust counterpart)", "\"medium\"", "pcb_choices", "pcb_str_u", list("medium"))

rows <- lapply(cases, function(k) {
  r <- fld(capture(k$r_fn, k$args))
  s <- fld(capture(k$rust_fn, k$args))
  data.frame(
    case = k$case, input = k$input,
    r_fn = k$r_fn, rust_fn = k$rust_fn,
    r_outcome = r$outcome, rust_outcome = s$outcome,
    r_class = r$class, rust_class = s$class, class_same = r$class == s$class,
    r_msg = r$msg, rust_msg = s$msg, msg_same = r$msg == s$msg,
    r_param = r$param, rust_param = s$param,
    r_rust_type = r$rust_type, rust_rust_type = s$rust_type,
    r_kind = r$kind, rust_kind = s$kind,
    r_call = r$call, rust_call = s$call,
    call_same = r$call == unify(s$call, k$r_fn, k$rust_fn),
    stringsAsFactors = FALSE
  )
})
out <- do.call(rbind, rows)
write.csv(out, out_csv, row.names = FALSE)

# One line per input, plus the headline counts over the inputs both paths refuse.
both <- out$r_outcome == "error" & out$rust_outcome == "error"
compact <- c(
  sprintf("both refuse: %d of %d inputs; class differs in %d, call differs in %d, param differs in %d, kind differs in %d",
          sum(both), nrow(out), sum(both & !out$class_same), sum(both & !out$call_same),
          sum(both & out$r_param != out$rust_param), sum(both & out$r_kind != out$rust_kind)),
  sprintf("%s | %s | %s/%s | R: %s | Rust: %s | rust_type=%s | class_same=%s call_same=%s",
          out$case, out$input, out$r_outcome, out$rust_outcome, out$r_msg, out$rust_msg,
          out$rust_rust_type, out$class_same, out$call_same)
)
writeLines(compact, sub("\\.csv$", "-compact.txt", out_csv))

# Human-readable dump.
for (i in seq_len(nrow(out))) {
  o <- out[i, ]
  cat(sprintf("\n## %s | input %s\n", o$case, o$input))
  cat(sprintf("  R    (%s): %s | class=%s | param=%s | rust_type=%s | kind=%s | call=%s\n        msg: %s\n",
              o$r_fn, o$r_outcome, o$r_class, o$r_param, o$r_rust_type, o$r_kind, o$r_call, o$r_msg))
  cat(sprintf("  Rust (%s): %s | class=%s | param=%s | rust_type=%s | kind=%s | call=%s\n        msg: %s\n",
              o$rust_fn, o$rust_outcome, o$rust_class, o$rust_param, o$rust_rust_type, o$rust_kind, o$rust_call, o$rust_msg))
  cat(sprintf("  same: class=%s msg=%s call=%s\n", o$class_same, o$msg_same, o$call_same))
}

# Rf_inherits (the C API) vs base::inherits: implicit classes, S4.
cat("\n\n# Rf_inherits vs base::inherits\n")
setClass("PcbBase", representation("VIRTUAL"))
setClass("PcbChild", contains = "PcbBase", representation(x = "numeric"))
obj4 <- new("PcbChild", x = 1)
probes <- list(
  list("1L", 1L, "integer"), list("1L", 1L, "numeric"), list("1", 1, "numeric"), list("1", 1, "double"),
  list("matrix(1)", matrix(1), "matrix"), list("matrix(1)", matrix(1), "array"),
  list("function(){}", function() NULL, "function"), list("data.frame()", data.frame(), "data.frame"),
  list("factor(\"a\")", factor("a"), "factor"), list("structure(1, class = \"mx_cls\")", structure(1, class = "mx_cls"), "mx_cls"),
  list("S4 PcbChild", obj4, "PcbChild"), list("S4 PcbChild", obj4, "PcbBase"),
  list("list()", list(), "list")
)
inh <- do.call(rbind, lapply(probes, function(p) {
  data.frame(x = p[[1]], class = p[[3]], base_inherits = inherits(p[[2]], p[[3]]),
             Rf_inherits = ns$pcb_rust_inherits(p[[2]], p[[3]]),
             is_methods = methods::is(p[[2]], p[[3]]))
}))
inh$same <- inh$base_inherits == inh$Rf_inherits
print(inh, right = FALSE)
write.csv(inh, sub("\\.csv$", "-inherits.csv", out_csv), row.names = FALSE)
