#' A list-like S3 class the package builds itself
#'
#' `handmade_rec` is the shape of a package that builds its own objects: the
#' Rust impl block of `SidecarHandmade` has no constructor, so
#' `new_handmade_rec()` classes the pointer [rdata_sidecar_handmade_new()]
#' returns. `s3(r_data_accessors = "get")` generates the readers of its
#' `#[r_data]` fields (`$`, `[[`, `names()`, `as.list()` and
#' `.DollarNames()`), which look the fields up as a list's do.
#'
#' @param id The id of the record; its `ids` are `seq_len(id)`.
#' @return A `handmade_rec` object: the classed external pointer.
#' @rdname handmade_rec
#' @export
new_handmade_rec <- function(id) {
  structure(rdata_sidecar_handmade_new(id), class = "handmade_rec")
}
