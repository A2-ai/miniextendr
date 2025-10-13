add <- function(left, right) { .Call(C_add, left, right) }

add2 <- function(left, right, unused_dummy)
{ .Call(C_add2, left, right, unused_dummy) }

add3 <- function(left, right, unused_dummy)
{ invisible(.Call(C_add3, left, right, unused_dummy)) }

add4 <- function(left, right) { .Call(C_add4, left, right) }

add_panic <- function(unused_left, unused_right)
{ .Call(C_add_panic, unused_left, unused_right) }

add_r_error <- function(unused_left, unused_right)
{ .Call(C_add_r_error, unused_left, unused_right) }

add_left_mut <- function(left, right) { .Call(C_add_left_mut, left, right) }

add_right_mut <- function(left, right) { .Call(C_add_right_mut, left, right) }

add_left_right_mut <- function(left, right)
{ .Call(C_add_left_right_mut, left, right) }

unsafe_C_just_panic <- function() { .Call(C_just_panic) }

unsafe_C_panic_and_catch <- function() { .Call(C_panic_and_catch) }

greetings_with_named_dots <- function(unused_dots = ...)
{ invisible(.Call(C_greetings_with_named_dots, list(unused_dots))) }

greetings_with_nameless_dots <- function(...)
{ invisible(.Call(C_greetings_with_nameless_dots, list(...))) }

greetings_last_as_named_dots <-
function(unused_exclamations, unused_dots = ...)
{
    invisible(.Call(C_greetings_last_as_named_dots, unused_exclamations,
    list(unused_dots)))
}

greetings_last_as_nameless_dots <- function(unused_exclamations, ...)
{
    invisible(.Call(C_greetings_last_as_nameless_dots, unused_exclamations,
    list(...)))
}

