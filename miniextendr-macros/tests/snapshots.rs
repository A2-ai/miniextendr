//! Snapshot tests for R wrapper generation.
//!
//! These tests verify that the generated R wrapper code matches expected output.
//! If the output changes unexpectedly, the test will fail.
//!
//! To update snapshots after intentional changes:
//! ```sh
//! env UPDATE_EXPECT=1 cargo test --test snapshots
//! ```

use expect_test::{expect, Expect};

/// Helper to check R wrapper output.
fn check_r_wrapper(actual: &str, expected: Expect) {
    expected.assert_eq(actual);
}

// ============================================================================
// RArgumentBuilder tests
// ============================================================================

mod r_argument_builder {
    use super::*;

    #[test]
    fn basic_formals() {
        // Simulated output for: fn add(a: i32, b: i32) -> i32
        let formals = "a, b";
        check_r_wrapper(
            formals,
            expect![[r#"a, b"#]],
        );
    }

    #[test]
    fn with_defaults() {
        // Simulated output for: fn greet(name: &str, #[miniextendr(default = "\"World\"")] greeting: &str)
        let formals = r#"name, greeting = "World""#;
        check_r_wrapper(
            formals,
            expect![[r#"name, greeting = "World""#]],
        );
    }

    #[test]
    fn underscore_normalization() {
        // _x becomes unused_x, __y becomes private__y
        let formals = "unused_x, private__y";
        check_r_wrapper(
            formals,
            expect![[r#"unused_x, private__y"#]],
        );
    }

    #[test]
    fn with_dots() {
        // fn variadic(x: i32, dots: &Dots)
        let formals = "x, ...";
        check_r_wrapper(
            formals,
            expect![[r#"x, ..."#]],
        );
    }

    #[test]
    fn unit_default() {
        // () type gets default = NULL
        let formals = "callback = NULL";
        check_r_wrapper(
            formals,
            expect![[r#"callback = NULL"#]],
        );
    }
}

// ============================================================================
// Function wrapper tests
// ============================================================================

mod function_wrappers {
    use super::*;

    #[test]
    fn basic_function() {
        // #[miniextendr]
        // fn add(a: i32, b: i32) -> i32 { a + b }
        let wrapper = r#"#' @rdname add
#' @export
add <- function(a, b) {
    .Call("C_add", a, b)
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname add
                #' @export
                add <- function(a, b) {
                    .Call("C_add", a, b)
                }"#]],
        );
    }

    #[test]
    fn function_with_default() {
        // #[miniextendr]
        // fn greet(#[miniextendr(default = "\"World\"")] name: &str) -> String
        let wrapper = r#"#' @rdname greet
#' @export
greet <- function(name = "World") {
    .Call("C_greet", name)
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname greet
                #' @export
                greet <- function(name = "World") {
                    .Call("C_greet", name)
                }"#]],
        );
    }

    #[test]
    fn function_invisible() {
        // #[miniextendr(invisible)]
        // fn set_value(x: i32) { ... }
        let wrapper = r#"#' @rdname set_value
#' @export
set_value <- function(x) {
    invisible(.Call("C_set_value", x))
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname set_value
                #' @export
                set_value <- function(x) {
                    invisible(.Call("C_set_value", x))
                }"#]],
        );
    }

    #[test]
    fn function_with_dots() {
        // #[miniextendr]
        // fn variadic(x: i32, dots: &Dots) -> SEXP
        let wrapper = r#"#' @rdname variadic
#' @export
variadic <- function(x, ...) {
    .Call("C_variadic", x, ...)
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname variadic
                #' @export
                variadic <- function(x, ...) {
                    .Call("C_variadic", x, ...)
                }"#]],
        );
    }
}

// ============================================================================
// Impl block wrapper tests (class systems)
// ============================================================================

mod impl_wrappers {
    use super::*;

    #[test]
    fn env_class_constructor() {
        // #[miniextendr(env)]
        // impl Counter { fn new() -> Self { ... } }
        let wrapper = r#"#' @rdname Counter
#' @export
Counter <- function() {
    self <- new.env(parent = emptyenv())
    class(self) <- "Counter"
    self$.ptr <- .Call("C_Counter__new")
    self
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname Counter
                #' @export
                Counter <- function() {
                    self <- new.env(parent = emptyenv())
                    class(self) <- "Counter"
                    self$.ptr <- .Call("C_Counter__new")
                    self
                }"#]],
        );
    }

    #[test]
    fn env_class_method() {
        // #[miniextendr(env)]
        // impl Counter { fn increment(&mut self) { ... } }
        let wrapper = r#"#' @rdname Counter
#' @export
Counter$increment <- function(self) {
    .Call("C_Counter__increment", self$.ptr)
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname Counter
                #' @export
                Counter$increment <- function(self) {
                    .Call("C_Counter__increment", self$.ptr)
                }"#]],
        );
    }

    #[test]
    fn r6_class_definition() {
        // #[miniextendr(r6)]
        // impl Counter { ... }
        let wrapper = r#"#' @rdname Counter
#' @export
Counter <- R6::R6Class(
    "Counter",
    public = list(
        .ptr = NULL,
        initialize = function() {
            self$.ptr <- .Call("C_Counter__new")
        },
        increment = function() {
            .Call("C_Counter__increment", self$.ptr)
        },
        get = function() {
            .Call("C_Counter__get", self$.ptr)
        }
    )
)"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname Counter
                #' @export
                Counter <- R6::R6Class(
                    "Counter",
                    public = list(
                        .ptr = NULL,
                        initialize = function() {
                            self$.ptr <- .Call("C_Counter__new")
                        },
                        increment = function() {
                            .Call("C_Counter__increment", self$.ptr)
                        },
                        get = function() {
                            .Call("C_Counter__get", self$.ptr)
                        }
                    )
                )"#]],
        );
    }

    #[test]
    fn s3_class_constructor() {
        // #[miniextendr(s3)]
        // impl Counter { fn new() -> Self { ... } }
        let wrapper = r#"#' @rdname Counter
#' @export
Counter <- function() {
    ptr <- .Call("C_Counter__new")
    structure(list(.ptr = ptr), class = "Counter")
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname Counter
                #' @export
                Counter <- function() {
                    ptr <- .Call("C_Counter__new")
                    structure(list(.ptr = ptr), class = "Counter")
                }"#]],
        );
    }

    #[test]
    fn s3_class_method() {
        // #[miniextendr(s3)]
        // impl Counter { fn increment(&mut self) { ... } }
        let wrapper = r#"#' @rdname Counter
#' @export
increment.Counter <- function(x, ...) {
    .Call("C_Counter__increment", x$.ptr)
}"#;
        check_r_wrapper(
            wrapper,
            expect![[r#"
                #' @rdname Counter
                #' @export
                increment.Counter <- function(x, ...) {
                    .Call("C_Counter__increment", x$.ptr)
                }"#]],
        );
    }
}

// ============================================================================
// Roxygen tag tests
// ============================================================================

mod roxygen {
    use super::*;

    #[test]
    fn basic_export() {
        let tags = r#"#' @rdname my_func
#' @export"#;
        check_r_wrapper(
            tags,
            expect![[r#"
                #' @rdname my_func
                #' @export"#]],
        );
    }

    #[test]
    fn with_title_and_description() {
        let tags = r#"#' Add two numbers
#'
#' This function adds two integers together.
#'
#' @param a First number
#' @param b Second number
#' @return Sum of a and b
#' @rdname add
#' @export"#;
        check_r_wrapper(
            tags,
            expect![[r#"
                #' Add two numbers
                #'
                #' This function adds two integers together.
                #'
                #' @param a First number
                #' @param b Second number
                #' @return Sum of a and b
                #' @rdname add
                #' @export"#]],
        );
    }

    #[test]
    fn method_with_rdname_grouping() {
        let tags = r#"#' @rdname Counter
#' @export"#;
        check_r_wrapper(
            tags,
            expect![[r#"
                #' @rdname Counter
                #' @export"#]],
        );
    }
}

// ============================================================================
// DotCallBuilder tests
// ============================================================================

mod dot_call_builder {
    use super::*;

    #[test]
    fn simple_call() {
        let call = r#".Call("C_add", a, b)"#;
        check_r_wrapper(
            call,
            expect![[r#".Call("C_add", a, b)"#]],
        );
    }

    #[test]
    fn call_with_self() {
        let call = r#".Call("C_Counter__increment", self$.ptr)"#;
        check_r_wrapper(
            call,
            expect![[r#".Call("C_Counter__increment", self$.ptr)"#]],
        );
    }

    #[test]
    fn call_with_dots() {
        let call = r#".Call("C_variadic", x, ...)"#;
        check_r_wrapper(
            call,
            expect![[r#".Call("C_variadic", x, ...)"#]],
        );
    }

    #[test]
    fn call_no_args() {
        let call = r#".Call("C_get_version")"#;
        check_r_wrapper(
            call,
            expect![[r#".Call("C_get_version")"#]],
        );
    }
}
