# Regression tests for #1115 (two r6 impls of one trait must not collide) and
# #1141 resp-4 (trait `-> Self` factory methods must re-wrap into a classed
# object). Fixtures: rpkg/src/rust/trait_r6_collision.rs.

test_that("two r6 impls of one trait dispatch via class-scoped namespaces (#1115)", {
  a <- DoublerA$new(3L)
  b <- DoublerB$new(3L)
  expect_true(inherits(a, "DoublerA"))
  expect_true(inherits(b, "DoublerB"))

  # Class-scoped trait wrappers: DoublerA$Doubler$doubled vs
  # DoublerB$Doubler$doubled — distinct, no shared r6_trait_Doubler_doubled.
  expect_equal(DoublerA$Doubler$doubled(a), 6L)
  expect_equal(DoublerB$Doubler$doubled(b), 30L)

  # The pre-fix unqualified standalone wrapper must be gone from the namespace.
  exports <- getNamespaceExports("miniextendr")
  expect_false("r6_trait_Doubler_doubled" %in% exports)
})

test_that("r6 trait -> Self factory methods return a classed object (#1141)", {
  a <- DoublerA$new(7L)

  # instance -> Self
  a2 <- DoublerA$Doubler$duplicate(a)
  expect_true(inherits(a2, "DoublerA"))
  expect_equal(a2$value(), 7L)

  # static -> Self
  a3 <- DoublerA$Doubler$spawn(9L)
  expect_true(inherits(a3, "DoublerA"))
  expect_equal(a3$value(), 9L)
})

test_that("env trait -> Self factory methods return a classed object (#1141)", {
  e <- DoublerEnv$new(4L)

  # instance -> Self: re-wrapped via class(.val) <- "DoublerEnv"
  e2 <- DoublerEnv$Doubler$duplicate(e)
  expect_true(inherits(e2, "DoublerEnv"))
  expect_equal(DoublerEnv$Doubler$doubled(e2), 8L)

  # static -> Self
  e3 <- DoublerEnv$Doubler$spawn(5L)
  expect_true(inherits(e3, "DoublerEnv"))
  expect_equal(DoublerEnv$Doubler$doubled(e3), 10L)
})
