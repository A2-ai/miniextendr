test_that("field helpers are emitted only for classes with field syntax", {
  namespace <- asNamespace("miniextendr")
  types <- c(
    "SidecarEnv", "SidecarS3", "SidecarS3Get", "SidecarComputed",
    "SidecarHandmade", "SidecarStrict", "SidecarS4",
    "FallbackS3", "FallbackList", "FallbackS4", "FallbackEnv",
    "ComputedResult",
    "ReadOnlyS3", "FrozenS3", "FrozenS4", "FrozenEnv",
    "NullS3"
  )
  # The read-only form (`r_data_accessors = "read_only"`) writes no
  # `.rdata_set_<Type>`: its `$<-` / `[[<-` refuse every field, and its setter
  # routines are not registered (#2292).
  read_only <- c("FrozenS3", "FrozenS4", "FrozenEnv")
  for (kind in c("fields", "get", "set", "no_field", "read_only", "error_class", "every_read", "live")) {
    prefix <- paste0(".rdata_", kind, "_")
    bindings <- ls(namespace, all.names = TRUE)
    expected <- if (kind == "set") setdiff(types, read_only) else types
    expect_setequal(
      bindings[startsWith(bindings, prefix)],
      paste0(prefix, expected)
    )
  }
})
