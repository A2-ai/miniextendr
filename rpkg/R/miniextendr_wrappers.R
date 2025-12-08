add <- function(left, right) {
    .Call(C_add, .call = match.call(), left, right)
}

add2 <- function(left, right, unused_dummy = NULL) {
    .Call(C_add2, .call = match.call(), left, right, unused_dummy)
}

add3 <- function(left, right, unused_dummy = NULL) {
    .Call(C_add3, .call = match.call(), left, right, unused_dummy)
}

add4 <- function(left, right) {
    .Call(C_add4, .call = match.call(), left, right)
}

add_panic <- function(unused_left, unused_right) {
    .Call(C_add_panic, .call = match.call(), unused_left, unused_right)
}

add_r_error <- function(unused_left, unused_right) {
    .Call(C_add_r_error, .call = match.call(), unused_left, unused_right)
}

add_panic_heap <- function(unused_left, unused_right) {
    .Call(C_add_panic_heap, .call = match.call(), unused_left, unused_right)
}

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

add_left_mut <- function(left, right) {
    .Call(C_add_left_mut, .call = match.call(), left, right)
}

add_right_mut <- function(left, right) {
    .Call(C_add_right_mut, .call = match.call(), left, right)
}

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

drop_message_on_success <- function() {
    .Call(C_drop_message_on_success, .call = match.call())
}

drop_on_panic <- function() {
    invisible(.Call(C_drop_on_panic, .call = match.call()))
}

drop_on_panic_with_move <- function() {
    invisible(.Call(C_drop_on_panic_with_move, .call = match.call()))
}

greetings_with_named_dots <- function(dots = ...) {
    invisible(.Call(C_greetings_with_named_dots, .call = match.call(), list(dots)))
}

greetings_with_named_and_unused_dots <- function(unused_dots = ...) {
    invisible(.Call(C_greetings_with_named_and_unused_dots, .call = match.call(), list(unused_dots)))
}

greetings_with_nameless_dots <- function(...) {
    invisible(.Call(C_greetings_with_nameless_dots, .call = match.call(), list(...)))
}

greetings_last_as_named_dots <- function(unused_exclamations, dots = ...) {
    invisible(.Call(C_greetings_last_as_named_dots, .call = match.call(), unused_exclamations, list(dots)))
}

greetings_last_as_named_and_unused_dots <- function(unused_exclamations, unused_dots = ...) {
    invisible(.Call(C_greetings_last_as_named_and_unused_dots, .call = match.call(), unused_exclamations, list(unused_dots)))
}

greetings_last_as_nameless_dots <- function(unused_exclamations, ...) {
    invisible(.Call(C_greetings_last_as_nameless_dots, .call = match.call(), unused_exclamations, list(...)))
}

invisibly_return_no_arrow <- function() {
    invisible(.Call(C_invisibly_return_no_arrow, .call = match.call()))
}

invisibly_return_arrow <- function() {
    invisible(.Call(C_invisibly_return_arrow, .call = match.call()))
}

invisibly_option_return_none <- function() {
    invisible(.Call(C_invisibly_option_return_none, .call = match.call()))
}

invisibly_option_return_some <- function() {
    invisible(.Call(C_invisibly_option_return_some, .call = match.call()))
}

invisibly_result_return_ok <- function() {
    invisible(.Call(C_invisibly_result_return_ok, .call = match.call()))
}

force_invisible_i32 <- function() {
    invisible(.Call(C_force_invisible_i32, .call = match.call()))
}

force_visible_unit <- function() {
    .Call(C_force_visible_unit, .call = match.call())
}

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

unsafe_C_worker_drop_on_success <- function() {
    .Call(C_worker_drop_on_success)
}

unsafe_C_worker_drop_on_panic <- function() {
    .Call(C_worker_drop_on_panic)
}

unsafe_C_test_worker_simple <- function() {
    .Call(C_test_worker_simple)
}

unsafe_C_test_worker_with_r_thread <- function() {
    .Call(C_test_worker_with_r_thread)
}

unsafe_C_test_worker_multiple_r_calls <- function() {
    .Call(C_test_worker_multiple_r_calls)
}

unsafe_C_test_worker_panic_simple <- function() {
    .Call(C_test_worker_panic_simple)
}

unsafe_C_test_worker_panic_with_drops <- function() {
    .Call(C_test_worker_panic_with_drops)
}

unsafe_C_test_worker_panic_in_r_thread <- function() {
    .Call(C_test_worker_panic_in_r_thread)
}

unsafe_C_test_worker_panic_in_r_thread_with_drops <- function() {
    .Call(C_test_worker_panic_in_r_thread_with_drops)
}

unsafe_C_test_worker_r_error_in_r_thread <- function() {
    .Call(C_test_worker_r_error_in_r_thread)
}

unsafe_C_test_worker_r_error_with_drops <- function() {
    .Call(C_test_worker_r_error_with_drops)
}

unsafe_C_test_worker_r_calls_then_error <- function() {
    .Call(C_test_worker_r_calls_then_error)
}

unsafe_C_test_worker_r_calls_then_panic <- function() {
    .Call(C_test_worker_r_calls_then_panic)
}

test_worker_return_i32 <- function() {
    .Call(C_test_worker_return_i32, .call = match.call())
}

test_worker_return_string <- function() {
    .Call(C_test_worker_return_string, .call = match.call())
}

test_worker_return_f64 <- function() {
    .Call(C_test_worker_return_f64, .call = match.call())
}

unsafe_C_test_extptr_from_worker <- function() {
    .Call(C_test_extptr_from_worker)
}

unsafe_C_test_multiple_extptrs_from_worker <- function() {
    .Call(C_test_multiple_extptrs_from_worker)
}

test_main_thread_r_api <- function() {
    .Call(C_test_main_thread_r_api, .call = match.call())
}

test_main_thread_r_error <- function() {
    .Call(C_test_main_thread_r_error, .call = match.call())
}

test_main_thread_r_error_with_drops <- function() {
    .Call(C_test_main_thread_r_error_with_drops, .call = match.call())
}

unsafe_C_test_wrong_thread_r_api <- function() {
    .Call(C_test_wrong_thread_r_api)
}

unsafe_C_test_nested_helper_from_worker <- function() {
    .Call(C_test_nested_helper_from_worker)
}

unsafe_C_test_nested_multiple_helpers <- function() {
    .Call(C_test_nested_multiple_helpers)
}

unsafe_C_test_nested_with_r_thread <- function() {
    .Call(C_test_nested_with_r_thread)
}

unsafe_C_test_call_worker_fn_from_main <- function() {
    .Call(C_test_call_worker_fn_from_main)
}

unsafe_C_test_nested_worker_calls <- function() {
    .Call(C_test_nested_worker_calls)
}

unsafe_C_test_nested_with_error <- function() {
    .Call(C_test_nested_with_error)
}

unsafe_C_test_nested_with_panic <- function() {
    .Call(C_test_nested_with_panic)
}

unsafe_C_test_deep_with_r_thread_sequence <- function() {
    .Call(C_test_deep_with_r_thread_sequence)
}

test_i32_identity <- function(x) {
    .Call(C_test_i32_identity, .call = match.call(), x)
}

test_i32_add_one <- function(x) {
    .Call(C_test_i32_add_one, .call = match.call(), x)
}

test_i32_sum <- function(a, b, c) {
    .Call(C_test_i32_sum, .call = match.call(), a, b, c)
}

test_f64_identity <- function(x) {
    .Call(C_test_f64_identity, .call = match.call(), x)
}

test_f64_add_one <- function(x) {
    .Call(C_test_f64_add_one, .call = match.call(), x)
}

test_f64_multiply <- function(a, b) {
    .Call(C_test_f64_multiply, .call = match.call(), a, b)
}

test_u8_identity <- function(x) {
    .Call(C_test_u8_identity, .call = match.call(), x)
}

test_u8_add_one <- function(x) {
    .Call(C_test_u8_add_one, .call = match.call(), x)
}

test_logical_identity <- function(x) {
    .Call(C_test_logical_identity, .call = match.call(), x)
}

test_logical_not <- function(x) {
    .Call(C_test_logical_not, .call = match.call(), x)
}

test_logical_and <- function(a, b) {
    .Call(C_test_logical_and, .call = match.call(), a, b)
}

test_i32_to_f64 <- function(x) {
    .Call(C_test_i32_to_f64, .call = match.call(), x)
}

test_f64_to_i32 <- function(x) {
    .Call(C_test_f64_to_i32, .call = match.call(), x)
}

test_i32_slice_len <- function(x) {
    .Call(C_test_i32_slice_len, .call = match.call(), x)
}

test_i32_slice_sum <- function(x) {
    .Call(C_test_i32_slice_sum, .call = match.call(), x)
}

test_i32_slice_first <- function(x) {
    .Call(C_test_i32_slice_first, .call = match.call(), x)
}

test_i32_slice_last <- function(x) {
    .Call(C_test_i32_slice_last, .call = match.call(), x)
}

test_f64_slice_len <- function(x) {
    .Call(C_test_f64_slice_len, .call = match.call(), x)
}

test_f64_slice_sum <- function(x) {
    .Call(C_test_f64_slice_sum, .call = match.call(), x)
}

test_f64_slice_mean <- function(x) {
    .Call(C_test_f64_slice_mean, .call = match.call(), x)
}

test_u8_slice_len <- function(x) {
    .Call(C_test_u8_slice_len, .call = match.call(), x)
}

test_u8_slice_sum <- function(x) {
    .Call(C_test_u8_slice_sum, .call = match.call(), x)
}

test_logical_slice_len <- function(x) {
    .Call(C_test_logical_slice_len, .call = match.call(), x)
}

test_logical_slice_any_true <- function(x) {
    .Call(C_test_logical_slice_any_true, .call = match.call(), x)
}

test_logical_slice_all_true <- function(x) {
    .Call(C_test_logical_slice_all_true, .call = match.call(), x)
}

underscore_it_all <- function(private__unused0, private__unused1) {
    invisible(.Call(C_underscore_it_all, .call = match.call(), private__unused0, private__unused1))
}

unsafe_rpkg_constant_int <- function() {
    .Call(rpkg_constant_int)
}

unsafe_rpkg_constant_real <- function() {
    .Call(rpkg_constant_real)
}

arith_seq <- function(from, to, length_out) {
    .Call(C_arith_seq, .call = match.call(), from, to, length_out)
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

lazy_list <- function(n) {
    .Call(C_lazy_list, .call = match.call(), n)
}

fibonacci <- function(n) {
    .Call(C_fibonacci, .call = match.call(), n)
}

unsafe_rpkg_powers_of_2 <- function() {
    .Call(rpkg_powers_of_2)
}

extptr_counter_new <- function(initial) {
    .Call(C_extptr_counter_new, .call = match.call(), initial)
}

unsafe_C_extptr_counter_get <- function(ptr) {
    .Call(C_extptr_counter_get, ptr)
}

unsafe_C_extptr_counter_increment <- function(ptr) {
    .Call(C_extptr_counter_increment, ptr)
}

extptr_point_new <- function(x, y) {
    .Call(C_extptr_point_new, .call = match.call(), x, y)
}

unsafe_C_extptr_point_get_x <- function(ptr) {
    .Call(C_extptr_point_get_x, ptr)
}

unsafe_C_extptr_point_get_y <- function(ptr) {
    .Call(C_extptr_point_get_y, ptr)
}

unsafe_C_extptr_type_mismatch_test <- function(ptr) {
    .Call(C_extptr_type_mismatch_test, ptr)
}

unsafe_C_extptr_null_test <- function(ptr) {
    .Call(C_extptr_null_test, ptr)
}

unsafe_C_extptr_is_counter <- function(ptr) {
    .Call(C_extptr_is_counter, ptr)
}

unsafe_C_extptr_is_point <- function(ptr) {
    .Call(C_extptr_is_point, ptr)
}

unsafe_rpkg_simple_vec_int <- function(x) {
    .Call(rpkg_simple_vec_int, x)
}

unsafe_rpkg_vec_int_altrep <- function(x) {
    .Call(rpkg_vec_int_altrep, x)
}

