# Namespace assignments have no roxygen-visible usage. Their prose belongs
# with the named member, never with another member or the class description.
test_that("namespace help groups each member's documentation", {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  text <- function(x) trimws(gsub("[[:space:]]+", " ", paste(unlist(x), collapse = "")))
  nodes <- function(x, tag) {
    if (!is.list(x)) return(list())
    here <- if (identical(attr(x, "Rd_tag"), tag)) list(x) else list()
    c(here, unlist(lapply(x, nodes, tag = tag), recursive = FALSE))
  }
  page <- function(topic) {
    rd <- db[[paste0(topic, ".Rd")]]
    expect_false(is.null(rd), info = topic)
    rd
  }
  member <- function(rd, label) {
    sections <- Filter(function(x) identical(text(x[[1]]), "Methods"), nodes(rd, "\\section"))
    expect_identical(length(sections), 1L, info = label)
    items <- Filter(function(x) identical(text(x[[1]]), paste0(label, "()")), nodes(sections, "\\item"))
    expect_identical(length(items), 1L, info = label)
    text(items[[1]][[2]])
  }

  rd <- page("IntSet")
  description <- text(nodes(rd, "\\description"))
  expect_match(description, "IntSet inherent methods: membership test.", fixed = TRUE)
  expect_false(grepl("Membership checks", description, fixed = TRUE))
  expect_false(grepl("Check if the collection is empty.", description, fixed = TRUE))

  contains <- member(rd, "IntSet$contains")
  expect_match(contains, "Test whether an integer belongs to the set.", fixed = TRUE)
  expect_match(contains, "Membership checks leave the set unchanged.", fixed = TRUE)
  expect_match(contains, "Integer value to check for membership.", fixed = TRUE)
  expect_match(contains, "Returns: Whether the value belongs to the set.", fixed = TRUE)
  empty <- member(rd, "IntSet$RToVec$is_empty")
  expect_match(empty, "Check if the collection is empty.", fixed = TRUE)
  expect_false(grepl("membership", empty, fixed = TRUE))
  expect_match(text(nodes(rd, "\\examples")), "values$contains(2L)", fixed = TRUE)
  expect_false(grepl("Methods:", text(nodes(rd, "\\examples")), fixed = TRUE))

  rd <- page("EnvMatchArgCounter")
  expect_match(member(rd, "EnvMatchArgCounter$new"), "One or more of", fixed = TRUE)
  expect_match(member(rd, "EnvMatchArgCounter$reset"), "One or more of", fixed = TRUE)

  labels <- c(
    OmitPickS3 = "OmitPickS3$GradeDefaults$default_grade",
    OmitPickS7 = 'attr(OmitPickS7, "GradeDefaults")$default_grade',
    OmitPickEnv = "OmitPickEnv$GradeDefaults$default_grade"
  )
  for (topic in names(labels)) {
    docs <- member(page(topic), labels[[topic]])
    expect_match(docs, "Pick the default grade, or none.", fixed = TRUE)
    expect_match(docs, "One of", fixed = TRUE)
  }
})
