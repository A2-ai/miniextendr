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

unsafe_drop_on_panic <- function() { .Call(drop_on_panic) }

unsafe_drop_on_panic_with_move <- function()
{ .Call(drop_on_panic_with_move) }

greetings_with_named_dots <- function(dots = ...)
{ invisible(.Call(C_greetings_with_named_dots, list(dots))) }

greetings_with_named_and_unused_dots <- function(unused_dots = ...)
{
    invisible(.Call(C_greetings_with_named_and_unused_dots,
    list(unused_dots)))
}

greetings_with_nameless_dots <- function(...)
{ invisible(.Call(C_greetings_with_nameless_dots, list(...))) }

greetings_last_as_named_dots <- function(unused_exclamations, dots = ...)
{
    invisible(.Call(C_greetings_last_as_named_dots, unused_exclamations,
    list(dots)))
}

greetings_last_as_named_and_unused_dots <-
function(unused_exclamations, unused_dots = ...)
{
    invisible(.Call(C_greetings_last_as_named_and_unused_dots,
    unused_exclamations, list(unused_dots)))
}

greetings_last_as_nameless_dots <- function(unused_exclamations, ...)
{
    invisible(.Call(C_greetings_last_as_nameless_dots, unused_exclamations,
    list(...)))
}

invisibly_return_no_arrow <- function()
{ invisible(.Call(C_invisibly_return_no_arrow)) }

invisibly_return_arrow <- function()
{ invisible(.Call(C_invisibly_return_arrow)) }

invisibly_option_return_none <- function()
{ invisible(.Call(C_invisibly_option_return_none)) }

invisibly_option_return_some <- function()
{ invisible(.Call(C_invisibly_option_return_some)) }

invisibly_result_return_ok <- function()
{ invisible(.Call(C_invisibly_result_return_ok)) }

