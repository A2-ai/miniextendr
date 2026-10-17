# Aliases for `match_arg` choices: `#[match_arg(alias = "grey")]` on
# `Shade::Gray` lets a caller write "grey" for "gray". An alias selects its
# choice only when typed in full, so "gr" stays the prefix of "gray" it was.
# Every error that lists the choices names the alias after them,
# `(or "grey" for "gray")`, unless it is `listed = false`; the formal and the
# usage list the choices only.

shades <- c("red", "gray", "blue")

# A refused value is the argument error every choice check raises, listing the
# choices and the listed aliases.
expect_choices_error <- function(expr, param, message) {
  e <- tryCatch(expr, error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, param)
  expect_identical(conditionMessage(e), message)
}

one_of <- "'color' should be one of \"red\", \"gray\", \"blue\" (or \"grey\" for \"gray\")"

test_that("an alias typed in full selects its choice", {
  expect_identical(match_arg_alias_shade("grey"), "Gray")
  expect_identical(match_arg_alias_shade("gray"), "Gray")
  expect_identical(match_arg_alias_shade(factor("grey")), "Gray")
  expect_identical(match_arg_alias_shade(), "Red")
  expect_identical(match_arg_alias_shade(NULL), "Red")
})

test_that("a prefix matches the choices only", {
  # "gr" is still the unique prefix of "gray".
  expect_identical(match_arg_alias_shade("gr"), "Gray")
  # A prefix of the alias is no match, nor is another case.
  expect_choices_error(match_arg_alias_shade("gre"), "color", one_of)
  expect_choices_error(match_arg_alias_shade("GREY"), "color", one_of)
  expect_choices_error(match_arg_alias_shade("zzz"), "color", one_of)
  expect_choices_error(match_arg_alias_shade(NA_character_), "color", one_of)
})

test_that("the formal and the usage list the choices only", {
  expect_identical(formals(match_arg_alias_shade)$color, quote(c("red", "gray", "blue")))
  expect_identical(eval(formals(match_arg_alias_shade)$color), shades)
  expect_identical(eval(formals(match_arg_alias_shades)$colors), shades)
  expect_identical(eval(formals(match_arg_alias_shade_default)$color), c("blue", "red", "gray"))

  # load_all() registers the source directory, which has the generated man
  # pages but no installed help database.
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  rd_text <- function(page) paste(capture.output(print(db[[page]])), collapse = "\n")
  shade <- rd_text("match_arg_alias_shade.Rd")
  expect_match(shade, "match_arg_alias_shade(color = c(\"red\", \"gray\", \"blue\"))", fixed = TRUE)
  expect_false(grepl("grey", shade, fixed = TRUE))
  # The class page's generated choice text too.
  palette <- rd_text("AliasPalette.Rd")
  expect_match(palette, "One of \"red\", \"gray\", \"blue\".", fixed = TRUE)
  expect_false(grepl("grey", palette, fixed = TRUE))
  # A listed alias is named by the errors, not by the usage.
  route <- rd_text("match_arg_alias_route.Rd")
  expect_match(route, "match_arg_alias_route(route = c(\"extravascular\", \"iv_bolus\", \"iv_infusion\"))", fixed = TRUE)
  expect_false(grepl("oral", route, fixed = TRUE))
})

test_that("Option and Missing layers map the alias", {
  expect_identical(match_arg_alias_shade_optional("grey"), "Gray")
  expect_identical(match_arg_alias_shade_optional(), "none")
  expect_identical(match_arg_alias_shade_omitted("grey"), "Gray")
  expect_identical(match_arg_alias_shade_omitted(), "absent")
  expect_choices_error(match_arg_alias_shade_optional("gre"), "color", one_of)
})

test_that("several_ok maps the aliases element by element", {
  expect_identical(match_arg_alias_shades(c("grey", "red")), "Gray,Red")
  expect_identical(match_arg_alias_shades(c("gr", "grey", "b")), "Gray,Gray,Blue")
  expect_identical(match_arg_alias_shades(), "Red,Gray,Blue")
  expect_choices_error(
    match_arg_alias_shades(c("grey", "zzz")),
    "colors",
    "'colors' element 2 (\"zzz\") should be one of \"red\", \"gray\", \"blue\" (or \"grey\" for \"gray\")"
  )
})

test_that("a default = \"...\" parameter maps the alias, and its message follows the formal", {
  expect_identical(match_arg_alias_shade_default(), "Blue")
  expect_identical(match_arg_alias_shade_default("grey"), "Gray")
  expect_choices_error(
    match_arg_alias_shade_default("gre"),
    "color",
    "'color' should be one of \"blue\", \"red\", \"gray\" (or \"grey\" for \"gray\")"
  )
})

test_that("a parameter named c doesn't break the aliases argument", {
  expect_identical(match_arg_alias_shade_shadowed("grey", 1L), "Gray:1")
  expect_true(any(grepl("aliases = base::c(", deparse(body(match_arg_alias_shade_shadowed)), fixed = TRUE)))
})

test_that("the Rust matchers map the alias as the wrapper does", {
  # No R-side check: the derived `TryFromSexp`.
  expect_identical(match_arg_alias_shade_converted("grey"), "Gray")
  expect_identical(match_arg_alias_shade_converted("gr"), "Gray")
  expect_choices_error(match_arg_alias_shade_converted("gre"), "color", one_of)
  # A raw argument matched in the body.
  expect_identical(match_arg_alias_shade_param("grey"), "Gray")
  expect_identical(match_arg_alias_shade_param(factor("grey")), "Gray")
  expect_choices_error(match_arg_alias_shade_param("gre"), "color", one_of)
})

test_that("an impl-block method maps the alias", {
  p <- AliasPalette$new("grey")
  expect_identical(p$paint("grey"), "Gray")
  expect_identical(p$paint("b"), "Blue")
  expect_choices_error(p$paint("gre"), "color", one_of)
})

test_that("Either<Shade, f64> reads an alias as its choice", {
  skip_if_not(exists("match_arg_alias_shade_or_number"), "either feature not compiled in")
  expect_identical(match_arg_alias_shade_or_number("grey"), "Gray")
  expect_identical(match_arg_alias_shade_or_number(2), "number:2")
  expect_choices_error(match_arg_alias_shade_or_number("gre"), "color", one_of)
  expect_choices_error(
    match_arg_alias_shade_or_number(TRUE),
    "color",
    "'color' must be one of \"red\", \"gray\", \"blue\" (or \"grey\" for \"gray\"), or a number: got logical"
  )
})

# Listed and unlisted aliases: `Route` has the listed "oral" and "infusion"
# and the unlisted "ev"; `QuietShade`'s one alias is unlisted.
routes_one_of <- paste0(
  "'route' should be one of \"extravascular\", \"iv_bolus\", \"iv_infusion\" ",
  "(or \"oral\" for \"extravascular\", \"infusion\" for \"iv_infusion\")"
)

test_that("an unlisted alias is matched like a listed one", {
  expect_identical(match_arg_alias_route("oral"), "Extravascular")
  expect_identical(match_arg_alias_route("ev"), "Extravascular")
  expect_identical(match_arg_alias_route("infusion"), "IvInfusion")
  expect_identical(match_arg_alias_route("iv_b"), "IvBolus")
  expect_identical(match_arg_alias_routes(c("ev", "oral", "iv_bolus")), "Extravascular,Extravascular,IvBolus")
  expect_identical(match_arg_alias_route_converted("ev"), "Extravascular")
  expect_identical(match_arg_alias_route_param("ev"), "Extravascular")
  expect_identical(match_arg_alias_quiet_shade("grey"), "Gray")
  expect_identical(match_arg_alias_quiet_shade_converted("grey"), "Gray")
})

test_that("every check words the listed aliases alike, in the order of the choices", {
  # The wrapper's check, the derived `TryFromSexp`, and `match_arg_param()`.
  expect_choices_error(match_arg_alias_route("orl"), "route", routes_one_of)
  expect_choices_error(match_arg_alias_route_converted("orl"), "route", routes_one_of)
  expect_choices_error(match_arg_alias_route_param("orl"), "route", routes_one_of)
  expect_choices_error(match_arg_alias_route_param(factor("orl")), "route", routes_one_of)
  # `several_ok`, element by element.
  expect_choices_error(
    match_arg_alias_routes(c("oral", "orl")),
    "routes",
    paste0(
      "'routes' element 2 (\"orl\") should be one of \"extravascular\", \"iv_bolus\", \"iv_infusion\" ",
      "(or \"oral\" for \"extravascular\", \"infusion\" for \"iv_infusion\")"
    )
  )
})

test_that("Either<Route, f64> names the listed aliases before the other arm", {
  skip_if_not(exists("match_arg_alias_route_or_number"), "either feature not compiled in")
  expect_identical(match_arg_alias_route_or_number("ev"), "Extravascular")
  expect_choices_error(match_arg_alias_route_or_number("orl"), "route", routes_one_of)
  expect_choices_error(
    match_arg_alias_route_or_number(TRUE),
    "route",
    paste0(
      "'route' must be one of \"extravascular\", \"iv_bolus\", \"iv_infusion\" ",
      "(or \"oral\" for \"extravascular\", \"infusion\" for \"iv_infusion\"), ",
      "or a number: got logical"
    )
  )
})

test_that("a type whose aliases are all unlisted words its errors as without aliases", {
  quiet <- "'color' should be one of \"red\", \"gray\", \"blue\""
  expect_choices_error(match_arg_alias_quiet_shade("gre"), "color", quiet)
  expect_choices_error(match_arg_alias_quiet_shade_converted("gre"), "color", quiet)
})
