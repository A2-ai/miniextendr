#' @export
add <- function(left, right) {
    .Call(C_add, .call = match.call(), left, right)
}

#' @export
add2 <- function(left, right, unused_dummy = NULL) {
    .Call(C_add2, .call = match.call(), left, right, unused_dummy)
}

#' @export
add3 <- function(left, right, unused_dummy = NULL) {
    .Call(C_add3, .call = match.call(), left, right, unused_dummy)
}

#' @export
add4 <- function(left, right) {
    .Call(C_add4, .call = match.call(), left, right)
}

#' @export
add_panic <- function(unused_left, unused_right) {
    .Call(C_add_panic, .call = match.call(), unused_left, unused_right)
}

#' @export
add_r_error <- function(unused_left, unused_right) {
    .Call(C_add_r_error, .call = match.call(), unused_left, unused_right)
}

#' @export
add_panic_heap <- function(unused_left, unused_right) {
    .Call(C_add_panic_heap, .call = match.call(), unused_left, unused_right)
}

#' @export
add_r_error_heap <- function(unused_left, unused_right) {
    .Call(C_add_r_error_heap, .call = match.call(), unused_left, unused_right)
}

unsafe_C_unwind_protect_normal <- function() {
    .Call(C_unwind_protect_normal)
}

unsafe_C_unwind_protect_r_error <- function() {
    .Call(C_unwind_protect_r_error)
}

unsafe_C_unwind_protect_lowlevel_test <- function() {
    .Call(C_unwind_protect_lowlevel_test)
}

#' @export
add_left_mut <- function(left, right) {
    .Call(C_add_left_mut, .call = match.call(), left, right)
}

#' @export
add_right_mut <- function(left, right) {
    .Call(C_add_right_mut, .call = match.call(), left, right)
}

#' @export
add_left_right_mut <- function(left, right) {
    .Call(C_add_left_right_mut, .call = match.call(), left, right)
}

take_and_return_nothing <- function() {
    invisible(.Call(C_take_and_return_nothing, .call = match.call()))
}

unsafe_C_just_panic <- function() {
    .Call(C_just_panic)
}

unsafe_C_panic_and_catch <- function() {
    .Call(C_panic_and_catch)
}

#' @export
drop_message_on_success <- function() {
    .Call(C_drop_message_on_success, .call = match.call())
}

#' @export
drop_on_panic <- function() {
    invisible(.Call(C_drop_on_panic, .call = match.call()))
}

#' @export
drop_on_panic_with_move <- function() {
    invisible(.Call(C_drop_on_panic_with_move, .call = match.call()))
}

#' @export
greetings_with_named_dots <- function(dots = ...) {
    invisible(.Call(C_greetings_with_named_dots, .call = match.call(), list(dots)))
}

#' @export
greetings_with_named_and_unused_dots <- function(unused_dots = ...) {
    invisible(.Call(C_greetings_with_named_and_unused_dots, .call = match.call(), list(unused_dots)))
}

#' @export
greetings_with_nameless_dots <- function(...) {
    invisible(.Call(C_greetings_with_nameless_dots, .call = match.call(), list(...)))
}

#' @export
greetings_last_as_named_dots <- function(unused_exclamations, dots = ...) {
    invisible(.Call(C_greetings_last_as_named_dots, .call = match.call(), unused_exclamations, list(dots)))
}

#' @export
greetings_last_as_named_and_unused_dots <- function(unused_exclamations, unused_dots = ...) {
    invisible(.Call(C_greetings_last_as_named_and_unused_dots, .call = match.call(), unused_exclamations, list(unused_dots)))
}

#' @export
greetings_last_as_nameless_dots <- function(unused_exclamations, ...) {
    invisible(.Call(C_greetings_last_as_nameless_dots, .call = match.call(), unused_exclamations, list(...)))
}

#' @export
invisibly_return_no_arrow <- function() {
    invisible(.Call(C_invisibly_return_no_arrow, .call = match.call()))
}

#' @export
invisibly_return_arrow <- function() {
    invisible(.Call(C_invisibly_return_arrow, .call = match.call()))
}

#' @export
invisibly_option_return_none <- function() {
    invisible(.Call(C_invisibly_option_return_none, .call = match.call()))
}

#' @export
invisibly_option_return_some <- function() {
    invisible(.Call(C_invisibly_option_return_some, .call = match.call()))
}

#' @export
invisibly_result_return_ok <- function() {
    invisible(.Call(C_invisibly_result_return_ok, .call = match.call()))
}

#' @export
force_invisible_i32 <- function() {
    invisible(.Call(C_force_invisible_i32, .call = match.call()))
}

#' @export
force_visible_unit <- function() {
    .Call(C_force_visible_unit, .call = match.call())
}

#' @export
with_interrupt_check <- function(x) {
    .Call(C_with_interrupt_check, .call = match.call(), x)
}

unsafe_C_r_error <- function() {
    .Call(C_r_error)
}

unsafe_C_r_error_in_catch <- function() {
    .Call(C_r_error_in_catch)
}

unsafe_C_r_error_in_thread <- function() {
    .Call(C_r_error_in_thread)
}

unsafe_C_r_print_in_thread <- function() {
    .Call(C_r_print_in_thread)
}

unsafe_C_check_interupt_after <- function() {
    .Call(C_check_interupt_after)
}

unsafe_C_check_interupt_unwind <- function() {
    .Call(C_check_interupt_unwind)
}

#' @export
unsafe_C_worker_drop_on_success <- function() {
    .Call(C_worker_drop_on_success)
}

#' @export
unsafe_C_worker_drop_on_panic <- function() {
    .Call(C_worker_drop_on_panic)
}

#' @export
unsafe_C_test_worker_simple <- function() {
    .Call(C_test_worker_simple)
}

#' @export
unsafe_C_test_worker_with_r_thread <- function() {
    .Call(C_test_worker_with_r_thread)
}

#' @export
unsafe_C_test_worker_multiple_r_calls <- function() {
    .Call(C_test_worker_multiple_r_calls)
}

#' @export
unsafe_C_test_worker_panic_simple <- function() {
    .Call(C_test_worker_panic_simple)
}

#' @export
unsafe_C_test_worker_panic_with_drops <- function() {
    .Call(C_test_worker_panic_with_drops)
}

#' @export
unsafe_C_test_worker_panic_in_r_thread <- function() {
    .Call(C_test_worker_panic_in_r_thread)
}

#' @export
unsafe_C_test_worker_panic_in_r_thread_with_drops <- function() {
    .Call(C_test_worker_panic_in_r_thread_with_drops)
}

#' @export
unsafe_C_test_worker_r_error_in_r_thread <- function() {
    .Call(C_test_worker_r_error_in_r_thread)
}

#' @export
unsafe_C_test_worker_r_error_with_drops <- function() {
    .Call(C_test_worker_r_error_with_drops)
}

#' @export
unsafe_C_test_worker_r_calls_then_error <- function() {
    .Call(C_test_worker_r_calls_then_error)
}

#' @export
unsafe_C_test_worker_r_calls_then_panic <- function() {
    .Call(C_test_worker_r_calls_then_panic)
}

#' @export
test_worker_return_i32 <- function() {
    .Call(C_test_worker_return_i32, .call = match.call())
}

#' @export
test_worker_return_string <- function() {
    .Call(C_test_worker_return_string, .call = match.call())
}

#' @export
test_worker_return_f64 <- function() {
    .Call(C_test_worker_return_f64, .call = match.call())
}

#' @export
unsafe_C_test_extptr_from_worker <- function() {
    .Call(C_test_extptr_from_worker)
}

#' @export
unsafe_C_test_multiple_extptrs_from_worker <- function() {
    .Call(C_test_multiple_extptrs_from_worker)
}

#' @export
test_main_thread_r_api <- function() {
    .Call(C_test_main_thread_r_api, .call = match.call())
}

#' @export
test_main_thread_r_error <- function() {
    .Call(C_test_main_thread_r_error, .call = match.call())
}

#' @export
test_main_thread_r_error_with_drops <- function() {
    .Call(C_test_main_thread_r_error_with_drops, .call = match.call())
}

#' @export
unsafe_C_test_wrong_thread_r_api <- function() {
    .Call(C_test_wrong_thread_r_api)
}

#' @export
unsafe_C_test_nested_helper_from_worker <- function() {
    .Call(C_test_nested_helper_from_worker)
}

#' @export
unsafe_C_test_nested_multiple_helpers <- function() {
    .Call(C_test_nested_multiple_helpers)
}

#' @export
unsafe_C_test_nested_with_r_thread <- function() {
    .Call(C_test_nested_with_r_thread)
}

#' @export
unsafe_C_test_call_worker_fn_from_main <- function() {
    .Call(C_test_call_worker_fn_from_main)
}

#' @export
unsafe_C_test_nested_worker_calls <- function() {
    .Call(C_test_nested_worker_calls)
}

#' @export
unsafe_C_test_nested_with_error <- function() {
    .Call(C_test_nested_with_error)
}

#' @export
unsafe_C_test_nested_with_panic <- function() {
    .Call(C_test_nested_with_panic)
}

#' @export
unsafe_C_test_deep_with_r_thread_sequence <- function() {
    .Call(C_test_deep_with_r_thread_sequence)
}

#' @export
test_i32_identity <- function(x) {
    .Call(C_test_i32_identity, .call = match.call(), x)
}

#' @export
test_i32_add_one <- function(x) {
    .Call(C_test_i32_add_one, .call = match.call(), x)
}

#' @export
test_i32_sum <- function(a, b, c) {
    .Call(C_test_i32_sum, .call = match.call(), a, b, c)
}

#' @export
test_f64_identity <- function(x) {
    .Call(C_test_f64_identity, .call = match.call(), x)
}

#' @export
test_f64_add_one <- function(x) {
    .Call(C_test_f64_add_one, .call = match.call(), x)
}

#' @export
test_f64_multiply <- function(a, b) {
    .Call(C_test_f64_multiply, .call = match.call(), a, b)
}

#' @export
test_u8_identity <- function(x) {
    .Call(C_test_u8_identity, .call = match.call(), x)
}

#' @export
test_u8_add_one <- function(x) {
    .Call(C_test_u8_add_one, .call = match.call(), x)
}

#' @export
test_logical_identity <- function(x) {
    .Call(C_test_logical_identity, .call = match.call(), x)
}

#' @export
test_logical_not <- function(x) {
    .Call(C_test_logical_not, .call = match.call(), x)
}

#' @export
test_logical_and <- function(a, b) {
    .Call(C_test_logical_and, .call = match.call(), a, b)
}

#' @export
test_i32_to_f64 <- function(x) {
    .Call(C_test_i32_to_f64, .call = match.call(), x)
}

#' @export
test_f64_to_i32 <- function(x) {
    .Call(C_test_f64_to_i32, .call = match.call(), x)
}

#' @export
test_i32_slice_len <- function(x) {
    .Call(C_test_i32_slice_len, .call = match.call(), x)
}

#' @export
test_i32_slice_sum <- function(x) {
    .Call(C_test_i32_slice_sum, .call = match.call(), x)
}

#' @export
test_i32_slice_first <- function(x) {
    .Call(C_test_i32_slice_first, .call = match.call(), x)
}

#' @export
test_i32_slice_last <- function(x) {
    .Call(C_test_i32_slice_last, .call = match.call(), x)
}

#' @export
test_f64_slice_len <- function(x) {
    .Call(C_test_f64_slice_len, .call = match.call(), x)
}

#' @export
test_f64_slice_sum <- function(x) {
    .Call(C_test_f64_slice_sum, .call = match.call(), x)
}

#' @export
test_f64_slice_mean <- function(x) {
    .Call(C_test_f64_slice_mean, .call = match.call(), x)
}

#' @export
test_u8_slice_len <- function(x) {
    .Call(C_test_u8_slice_len, .call = match.call(), x)
}

#' @export
test_u8_slice_sum <- function(x) {
    .Call(C_test_u8_slice_sum, .call = match.call(), x)
}

#' @export
test_logical_slice_len <- function(x) {
    .Call(C_test_logical_slice_len, .call = match.call(), x)
}

#' @export
test_logical_slice_any_true <- function(x) {
    .Call(C_test_logical_slice_any_true, .call = match.call(), x)
}

#' @export
test_logical_slice_all_true <- function(x) {
    .Call(C_test_logical_slice_all_true, .call = match.call(), x)
}

underscore_it_all <- function(private__unused0, private__unused1) {
    invisible(.Call(C_underscore_it_all, .call = match.call(), private__unused0, private__unused1))
}

#' @export
test_coerce_identity <- function(x) {
    .Call(C_test_coerce_identity, .call = match.call(), x)
}

#' @export
test_coerce_widen <- function(x) {
    .Call(C_test_coerce_widen, .call = match.call(), x)
}

#' @export
test_coerce_bool_to_int <- function(x) {
    .Call(C_test_coerce_bool_to_int, .call = match.call(), x)
}

#' @export
test_coerce_via_helper <- function(x) {
    .Call(C_test_coerce_via_helper, .call = match.call(), x)
}

#' @export
test_try_coerce_f64_to_i32 <- function(x) {
    .Call(C_test_try_coerce_f64_to_i32, .call = match.call(), x)
}

#' @export
test_rnative_newtype <- function(id) {
    .Call(C_test_rnative_newtype, .call = match.call(), id)
}

#' @export
test_rnative_named_field <- function(temp) {
    .Call(C_test_rnative_named_field, .call = match.call(), temp)
}

#' @export
test_coerce_attr_u16 <- function(x) {
    .Call(C_test_coerce_attr_u16, .call = match.call(), x)
}

#' @export
test_coerce_attr_i16 <- function(x) {
    .Call(C_test_coerce_attr_i16, .call = match.call(), x)
}

#' @export
test_coerce_attr_vec_u16 <- function(x) {
    .Call(C_test_coerce_attr_vec_u16, .call = match.call(), x)
}

#' @export
test_coerce_attr_f32 <- function(x) {
    .Call(C_test_coerce_attr_f32, .call = match.call(), x)
}

#' @export
test_coerce_attr_with_invisible <- function(x) {
    invisible(.Call(C_test_coerce_attr_with_invisible, .call = match.call(), x))
}

#' @export
test_per_arg_coerce_first <- function(x, y) {
    .Call(C_test_per_arg_coerce_first, .call = match.call(), x, y)
}

#' @export
test_per_arg_coerce_second <- function(x, y) {
    .Call(C_test_per_arg_coerce_second, .call = match.call(), x, y)
}

#' @export
test_per_arg_coerce_both <- function(x, y) {
    .Call(C_test_per_arg_coerce_both, .call = match.call(), x, y)
}

#' @export
test_per_arg_coerce_vec <- function(x, y) {
    .Call(C_test_per_arg_coerce_vec, .call = match.call(), x, y)
}

#' @export
unsafe_rpkg_constant_int <- function() {
    .Call(rpkg_constant_int)
}

#' @export
unsafe_rpkg_constant_real <- function() {
    .Call(rpkg_constant_real)
}

arith_seq <- function(from, to, length_out) {
    .Call(C_arith_seq, .call = match.call(), from, to, length_out)
}

#' @export
lazy_int_seq <- function(from, to, by) {
    .Call(C_lazy_int_seq, .call = match.call(), from, to, by)
}

#' @export
unsafe_rpkg_lazy_int_seq_is_materialized <- function(x) {
    .Call(rpkg_lazy_int_seq_is_materialized, x)
}

constant_logical <- function(value, n) {
    .Call(C_constant_logical, .call = match.call(), value, n)
}

lazy_string <- function(prefix, n) {
    .Call(C_lazy_string, .call = match.call(), prefix, n)
}

repeating_raw <- function(pattern, n) {
    .Call(C_repeating_raw, .call = match.call(), pattern, n)
}

#' @export
unit_circle <- function(n) {
    .Call(C_unit_circle, .call = match.call(), n)
}

extptr_counter_new <- function(initial) {
    .Call(C_extptr_counter_new, .call = match.call(), initial)
}

#' @export
unsafe_C_extptr_counter_get <- function(ptr) {
    .Call(C_extptr_counter_get, ptr)
}

#' @export
unsafe_C_extptr_counter_increment <- function(ptr) {
    .Call(C_extptr_counter_increment, ptr)
}

extptr_point_new <- function(x, y) {
    .Call(C_extptr_point_new, .call = match.call(), x, y)
}

#' @export
unsafe_C_extptr_point_get_x <- function(ptr) {
    .Call(C_extptr_point_get_x, ptr)
}

#' @export
unsafe_C_extptr_point_get_y <- function(ptr) {
    .Call(C_extptr_point_get_y, ptr)
}

#' @export
unsafe_C_extptr_type_mismatch_test <- function(ptr) {
    .Call(C_extptr_type_mismatch_test, ptr)
}

#' @export
unsafe_C_extptr_null_test <- function(ptr) {
    .Call(C_extptr_null_test, ptr)
}

#' @export
unsafe_C_extptr_is_counter <- function(ptr) {
    .Call(C_extptr_is_counter, ptr)
}

#' @export
unsafe_C_extptr_is_point <- function(ptr) {
    .Call(C_extptr_is_point, ptr)
}

#' @export
unsafe_rpkg_simple_vec_int <- function(x) {
    .Call(rpkg_simple_vec_int, x)
}

#' @export
unsafe_rpkg_inferred_vec_real <- function(x) {
    .Call(rpkg_inferred_vec_real, x)
}

#' @export
boxed_ints <- function(n) {
    .Call(C_boxed_ints, .call = match.call(), n)
}

#' @export
static_ints <- function() {
    .Call(C_static_ints, .call = match.call())
}

#' @export
leaked_ints <- function(n) {
    .Call(C_leaked_ints, .call = match.call(), n)
}

#' @export
static_strings <- function() {
    .Call(C_static_strings, .call = match.call())
}

#' @export
unsafe_C_test_r_thread_builder <- function() {
    .Call(C_test_r_thread_builder)
}

#' @export
unsafe_C_test_r_thread_builder_spawn_join <- function() {
    .Call(C_test_r_thread_builder_spawn_join)
}

