add <- function(left, right) { .Call(C_add, left, right) }

add2 <- function(left, right, _dummy) { .Call(C_add2, left, right, _dummy) }

add3 <- function(left, right, _dummy)
{ invisible(.Call(C_add3, left, right, _dummy)) }

add4 <- function(left, right) { .Call(C_add4, left, right) }

add_panic <- function(_left, _right) { .Call(C_add_panic, _left, _right) }

add_r_error <- function(_left, _right) { .Call(C_add_r_error, _left, _right) }

add_left_mut <- function(left, right) { .Call(C_add_left_mut, left, right) }

add_right_mut <- function(left, right) { .Call(C_add_right_mut, left, right) }

add_left_right_mut <- function(left, right)
{ .Call(C_add_left_right_mut, left, right) }

C_just_panic <- function() { .Call(C_just_panic) }

C_panic_and_catch <- function() { .Call(C_panic_and_catch) }

greetings_with_named_dots <- function(_dots)
{ invisible(.Call(C_greetings_with_named_dots, _dots)) }

greetings_with_nameless_dots <- function()
{ invisible(.Call(C_greetings_with_nameless_dots)) }

greetings_last_as_named_dots <- function(_exclamations, _dots)
{ invisible(.Call(C_greetings_last_as_named_dots, _exclamations, _dots)) }

greetings_last_as_nameless_dots <- function(_exclamations)
{ invisible(.Call(C_greetings_last_as_nameless_dots, _exclamations)) }

