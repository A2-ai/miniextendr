# LabelledValue keeps its labels in a sidecar. The shared Tagged trait reaches
# them through owner receivers (ExternalPtrRef / ExternalPtrMut), which the
# generated .Call wrappers build from the receiver.

test_that("Tagged's owner receivers read and write the sidecar through .Call", {
  obj <- LabelledValue$new("first")
  expect_identical(LabelledValue$Tagged$labels(obj), "first")
  LabelledValue$Tagged$add_label(obj, "second")
  gc(full = TRUE)
  expect_identical(LabelledValue$Tagged$labels(obj), c("first", "second"))
})
