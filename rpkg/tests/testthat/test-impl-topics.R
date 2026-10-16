# The defining impl's author topic must also receive secondary impl members.
# R/zz_impl_topics.R sorts after the wrappers: @order must keep its name,
# title and parameter text authoritative regardless of file order.
impl_topic_rd <- function() {
  pkg_path <- getNamespaceInfo(asNamespace("miniextendr"), "path")
  if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
}

test_that("all class systems keep their members on the impl topic", {
  db <- impl_topic_rd()
  page <- db[["impl_topics.Rd"]]
  expect_false(is.null(page))
  tags <- vapply(page, function(x) attr(x, "Rd_tag"), character(1))
  aliases <- vapply(page[tags == "\\alias"], paste0, character(1), collapse = "")
  expected <- c(
    "TopicENV", "TopicENV$new", "TopicENV$topic_env_value", "TopicENV$origin",
    "TopicENV$topic_as_integer", "as.integer.TopicENV",
    "TopicR6", "TopicR6$origin", "TopicS3", "topic_s3_value.TopicS3", "topics3_origin",
    "TopicS4", "s4_topic_s4_value,TopicS4-method", "TopicS4_origin",
    "TopicS7", "TopicS7$topic_s7_value", "TopicS7_topic_s7_value", "TopicS7_origin",
    "TopicS7Peer", "TopicS7_from_peer", "convert-TopicS7-to-TopicS7Peer",
    "convert-TopicS7Peer-to-TopicS7",
    "new_topicvctrs", "topicvctrs_origin", "vec_ptype_abbr.TopicVctrs",
    "vec_ptype2.TopicVctrs.TopicVctrs", "vec_cast.TopicVctrs.TopicVctrs"
  )
  for (alias in expected) expect_true(alias %in% aliases, info = alias)
  expect_false(any(paste0(c("TopicENV", "TopicR6", "TopicS3", "TopicS4", "TopicS7", "TopicS7Peer", "TopicVctrs"), ".Rd") %in% names(db)))
  text <- paste0(as.character(page), collapse = "")
  expect_match(text, "Classes sharing an author-defined topic", fixed = TRUE)
  expect_match(text, "Offset, documented by the topic.", fixed = TRUE)
  expect_match(text, "Numeric amounts, documented by the topic.", fixed = TRUE)
  expect_false(grepl("undocumented|no documentation available", text))
  expect_false(grepl(".__MX_CLASS_MEMBER_DEFAULT__", text, fixed = TRUE))
  override <- paste0(as.character(db[["impl_topic_override.Rd"]]), collapse = "")
  expect_match(override, "topic_override.TopicS3", fixed = TRUE)
  expect_false("topic_override.TopicS3" %in% aliases)
})

test_that("topic inheritance preserves callable constructors and secondary members", {
  expect_identical(TopicENV$new(4L)$topic_env_value(), 4L)
  expect_identical(TopicR6$new(4L)$topic_r6_value(0L), 4L)
  expect_identical(topic_s3_value(TopicS3$new(4L)), 4L)
  expect_identical(s4_topic_s4_value(TopicS4(4L)), 4L)
  expect_identical(topic_s7_value(TopicS7(4L)), 4L)
  expect_identical(topic_override(TopicS3$new(4L)), 4L)
  for (origin in list(TopicENV$origin, TopicR6$origin, topics3_origin,
                     TopicS4_origin, TopicS7_origin, topicvctrs_origin)) {
    expect_identical(origin(7L), 7L)
  }
  expect_identical(vctrs::vec_data(new_topicvctrs(c(1, 2))), c(1, 2))
  expect_identical(as.integer(TopicENV$new(4L)), 4L)
  peer <- S7::convert(TopicS7(4L), TopicS7Peer)
  expect_true(S7::S7_inherits(peer, TopicS7Peer))
  expect_identical(topic_s7_value(S7::convert(peer, TopicS7)), 4L)
})
