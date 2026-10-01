# Which calls expand a compact ALTREP sequence? For each call, a fresh
# compact vector is passed and its state is read back with inspect().
#
# Usage: Rscript rpkg/dev/bench-preconditions/materialise.R > materialise.log
ns <- asNamespace("miniextendr")
state <- function(x) {
  s <- paste(capture.output(.Internal(inspect(x))), collapse = " ")
  if (grepl("expanded", s)) "EXPANDED" else if (grepl("compact", s)) "compact" else "not-altrep"
}
probe <- function(label, f, make) {
  x <- make()
  before <- state(x)
  r <- tryCatch({ v <- f(x); paste("ok:", if (length(v) == 1L) format(v) else paste0("<len ", length(v), ">")) }, error = function(e) paste("error:", conditionMessage(e)))
  after <- state(x)
  data.frame(call = label, input = deparse(body(make)), before = before, after = after, result = r)
}
int_make <- function() 1:1000
dbl_make <- function() as.double(1:1000)
g <- function(n) get(n, ns)
rows <- list(
  probe("R: is.integer(x)", function(x) is.integer(x), int_make),
  probe("R: length(x) == 1L", function(x) length(x) == 1L, int_make),
  probe("R: anyNA(x)", function(x) anyNA(x), int_make),
  probe("R: anyNA(x)", function(x) anyNA(x), dbl_make),
  probe("R: all(is.na(x) | x == trunc(x))", function(x) all(is.na(x) | x == trunc(x)), dbl_make),
  probe("R: inherits(x, 'mx_cls')", function(x) inherits(x, "mx_cls"), int_make),
  probe("R: sum(x)", function(x) sum(x), int_make),
  probe("R: x + 0L", function(x) x + 0L, int_make),
  probe("#[miniextendr] x: SEXP (no guard)", g("pcb_sexp_floor"), int_make),
  probe("#[miniextendr] x: AltrepSexp (no guard)", g("pcb_altrep_floor"), int_make),
  probe("#[miniextendr] no_na x: AltrepSexp (R anyNA guard)", g("pcb_altrep_rguard_nona"), int_make),
  probe("raw C: nothing", g("unsafe_C_pcb_raw_floor"), int_make),
  probe("raw C: TYPEOF+XLENGTH", g("unsafe_C_pcb_type_len"), dbl_make),
  probe("raw C: NA scan, NO_NA+OR_NULL+GET_REGION", g("unsafe_C_pcb_na_capi"), int_make),
  probe("raw C: NA scan, NO_NA+OR_NULL+GET_REGION", g("unsafe_C_pcb_na_capi"), dbl_make),
  probe("raw C: NA scan, *_ELT (from_r::any_na shape)", g("unsafe_C_pcb_na_framework"), int_make),
  probe("raw C: NA scan, *_ELT (from_r::any_na shape)", g("unsafe_C_pcb_na_framework"), dbl_make),
  probe("raw C: NA scan, DATAPTR_RO slice", g("unsafe_C_pcb_na_slice"), int_make),
  probe("raw C: whole-number scan, OR_NULL+GET_REGION", g("unsafe_C_pcb_whole_capi"), dbl_make),
  probe("raw C: whole-number scan, DATAPTR_RO slice", g("unsafe_C_pcb_whole_slice"), dbl_make),
  probe("raw C: Rf_inherits", g("unsafe_C_pcb_inherits"), int_make),
  probe("#[miniextendr] &[i32]", g("pcb_slice_i32_u"), int_make),
  probe("#[miniextendr] Vec<i32>", g("pcb_vec_i32_u"), int_make),
  probe("#[miniextendr] &[f64]", g("pcb_slice_f64_u"), dbl_make),
  probe("#[miniextendr] no_na &[i32] (R: is.integer + anyNA)", g("pcb_nona_slice_i32"), int_make),
  probe("#[miniextendr] no_na Either<&[i32], String> (Rust any_na)", g("pcb_either_i32_nona"), int_make),
  probe("#[miniextendr] coerce Vec<i32> (R whole guard + Rust)", g("pcb_coerce_vec_i32"), dbl_make),
  probe("#[miniextendr] coerce Vec<i32>, no_preconditions (Rust only)", g("pcb_coerce_vec_i32_u"), dbl_make)
)
out <- do.call(rbind, rows)
old <- options(width = 250)
print(out, right = FALSE)
options(old)
