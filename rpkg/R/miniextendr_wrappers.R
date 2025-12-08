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

#' @source Generated by miniextendr from Rust fn `C_extptr_counter_get`
#' @export
unsafe_C_extptr_counter_get <- function(ptr) {
    .Call(C_extptr_counter_get, ptr)
}

#' @source Generated by miniextendr from Rust fn `C_extptr_counter_increment`
#' @export
unsafe_C_extptr_counter_increment <- function(ptr) {
    .Call(C_extptr_counter_increment, ptr)
}

#' @source Generated by miniextendr from Rust fn `extptr_point_new`
#' @export
extptr_point_new <- function(x, y) {
    .Call(C_extptr_point_new, .call = match.call(), x, y)
}

#' @source Generated by miniextendr from Rust fn `C_extptr_point_get_x`
#' @export
unsafe_C_extptr_point_get_x <- function(ptr) {
    .Call(C_extptr_point_get_x, ptr)
}

#' @source Generated by miniextendr from Rust fn `C_extptr_point_get_y`
#' @export
unsafe_C_extptr_point_get_y <- function(ptr) {
    .Call(C_extptr_point_get_y, ptr)
}

#' @source Generated by miniextendr from Rust fn `C_extptr_type_mismatch_test`
#' @export
unsafe_C_extptr_type_mismatch_test <- function(ptr) {
    .Call(C_extptr_type_mismatch_test, ptr)
}

#' @source Generated by miniextendr from Rust fn `C_extptr_null_test`
#' @export
unsafe_C_extptr_null_test <- function(ptr) {
    .Call(C_extptr_null_test, ptr)
}

#' @source Generated by miniextendr from Rust fn `C_extptr_is_counter`
#' @export
unsafe_C_extptr_is_counter <- function(ptr) {
    .Call(C_extptr_is_counter, ptr)
}

#' @source Generated by miniextendr from Rust fn `C_extptr_is_point`
#' @export
unsafe_C_extptr_is_point <- function(ptr) {
    .Call(C_extptr_is_point, ptr)
}

#' @source Generated by miniextendr from Rust fn `r6_standalone_add`
#' @export
r6_standalone_add <- function(a, b) {
    .Call(C_r6_standalone_add, .call = match.call(), a, b)
}

#' @source Generated by miniextendr from Rust fn `C_worker_drop_on_success`
#' @export
unsafe_C_worker_drop_on_success <- function() {
    .Call(C_worker_drop_on_success)
}

#' @source Generated by miniextendr from Rust fn `C_worker_drop_on_panic`
#' @export
unsafe_C_worker_drop_on_panic <- function() {
    .Call(C_worker_drop_on_panic)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_simple`
#' @export
unsafe_C_test_worker_simple <- function() {
    .Call(C_test_worker_simple)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_with_r_thread`
#' @export
unsafe_C_test_worker_with_r_thread <- function() {
    .Call(C_test_worker_with_r_thread)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_multiple_r_calls`
#' @export
unsafe_C_test_worker_multiple_r_calls <- function() {
    .Call(C_test_worker_multiple_r_calls)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_panic_simple`
#' @export
unsafe_C_test_worker_panic_simple <- function() {
    .Call(C_test_worker_panic_simple)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_panic_with_drops`
#' @export
unsafe_C_test_worker_panic_with_drops <- function() {
    .Call(C_test_worker_panic_with_drops)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_panic_in_r_thread`
#' @export
unsafe_C_test_worker_panic_in_r_thread <- function() {
    .Call(C_test_worker_panic_in_r_thread)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_panic_in_r_thread_with_drops`
#' @export
unsafe_C_test_worker_panic_in_r_thread_with_drops <- function() {
    .Call(C_test_worker_panic_in_r_thread_with_drops)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_r_error_in_r_thread`
#' @export
unsafe_C_test_worker_r_error_in_r_thread <- function() {
    .Call(C_test_worker_r_error_in_r_thread)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_r_error_with_drops`
#' @export
unsafe_C_test_worker_r_error_with_drops <- function() {
    .Call(C_test_worker_r_error_with_drops)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_r_calls_then_error`
#' @export
unsafe_C_test_worker_r_calls_then_error <- function() {
    .Call(C_test_worker_r_calls_then_error)
}

#' @source Generated by miniextendr from Rust fn `C_test_worker_r_calls_then_panic`
#' @export
unsafe_C_test_worker_r_calls_then_panic <- function() {
    .Call(C_test_worker_r_calls_then_panic)
}

#' @title Worker Thread Tests
#' @name rpkg_worker_tests
#' @keywords internal
#' @description Worker-thread and main-thread helpers
#' @examples
#' test_worker_return_i32()
#' test_worker_return_string()
#' test_worker_return_f64()
#' try(test_main_thread_r_error())
#' \dontrun{
#' unsafe_C_test_worker_simple()
#' }
#' @aliases test_worker_return_i32 test_worker_return_string test_worker_return_f64
#' @aliases test_main_thread_r_api test_main_thread_r_error test_main_thread_r_error_with_drops
#' @aliases unsafe_C_worker_drop_on_success unsafe_C_worker_drop_on_panic
#' @aliases unsafe_C_test_worker_simple unsafe_C_test_worker_with_r_thread
#' @aliases unsafe_C_test_worker_multiple_r_calls unsafe_C_test_worker_panic_simple
#' @aliases unsafe_C_test_worker_panic_with_drops unsafe_C_test_worker_panic_in_r_thread
#' @aliases unsafe_C_test_worker_panic_in_r_thread_with_drops
#' @aliases unsafe_C_test_worker_r_error_in_r_thread unsafe_C_test_worker_r_error_with_drops
#' @aliases unsafe_C_test_worker_r_calls_then_error unsafe_C_test_worker_r_calls_then_panic
#' @aliases unsafe_C_test_extptr_from_worker unsafe_C_test_multiple_extptrs_from_worker
#' @aliases unsafe_C_test_wrong_thread_r_api unsafe_C_test_call_worker_fn_from_main
#' @aliases unsafe_C_test_nested_helper_from_worker unsafe_C_test_nested_multiple_helpers
#' @aliases unsafe_C_test_nested_with_r_thread unsafe_C_test_nested_worker_calls
#' @aliases unsafe_C_test_nested_with_error unsafe_C_test_nested_with_panic
#' @aliases unsafe_C_test_deep_with_r_thread_sequence
#' @source Generated by miniextendr from Rust fn `test_worker_return_i32`
#' @export
test_worker_return_i32 <- function() {
    .Call(C_test_worker_return_i32, .call = match.call())
}

#' @source Generated by miniextendr from Rust fn `test_worker_return_string`
#' @export
test_worker_return_string <- function() {
    .Call(C_test_worker_return_string, .call = match.call())
}

#' @source Generated by miniextendr from Rust fn `test_worker_return_f64`
#' @export
test_worker_return_f64 <- function() {
    .Call(C_test_worker_return_f64, .call = match.call())
}

#' @source Generated by miniextendr from Rust fn `C_test_extptr_from_worker`
#' @export
unsafe_C_test_extptr_from_worker <- function() {
    .Call(C_test_extptr_from_worker)
}

#' @source Generated by miniextendr from Rust fn `C_test_multiple_extptrs_from_worker`
#' @export
unsafe_C_test_multiple_extptrs_from_worker <- function() {
    .Call(C_test_multiple_extptrs_from_worker)
}

#' @source Generated by miniextendr from Rust fn `test_main_thread_r_api`
#' @export
test_main_thread_r_api <- function() {
    .Call(C_test_main_thread_r_api, .call = match.call())
}

#' @source Generated by miniextendr from Rust fn `test_main_thread_r_error`
#' @export
test_main_thread_r_error <- function() {
    .Call(C_test_main_thread_r_error, .call = match.call())
}

#' @source Generated by miniextendr from Rust fn `test_main_thread_r_error_with_drops`
#' @export
test_main_thread_r_error_with_drops <- function() {
    .Call(C_test_main_thread_r_error_with_drops, .call = match.call())
}

#' @source Generated by miniextendr from Rust fn `C_test_wrong_thread_r_api`
#' @export
unsafe_C_test_wrong_thread_r_api <- function() {
    .Call(C_test_wrong_thread_r_api)
}

#' @source Generated by miniextendr from Rust fn `C_test_nested_helper_from_worker`
#' @export
unsafe_C_test_nested_helper_from_worker <- function() {
    .Call(C_test_nested_helper_from_worker)
}

#' @source Generated by miniextendr from Rust fn `C_test_nested_multiple_helpers`
#' @export
unsafe_C_test_nested_multiple_helpers <- function() {
    .Call(C_test_nested_multiple_helpers)
}

#' @source Generated by miniextendr from Rust fn `C_test_nested_with_r_thread`
#' @export
unsafe_C_test_nested_with_r_thread <- function() {
    .Call(C_test_nested_with_r_thread)
}

#' @source Generated by miniextendr from Rust fn `C_test_call_worker_fn_from_main`
#' @export
unsafe_C_test_call_worker_fn_from_main <- function() {
    .Call(C_test_call_worker_fn_from_main)
}

#' @source Generated by miniextendr from Rust fn `C_test_nested_worker_calls`
#' @export
unsafe_C_test_nested_worker_calls <- function() {
    .Call(C_test_nested_worker_calls)
}

#' @source Generated by miniextendr from Rust fn `C_test_nested_with_error`
#' @export
unsafe_C_test_nested_with_error <- function() {
    .Call(C_test_nested_with_error)
}

#' @source Generated by miniextendr from Rust fn `C_test_nested_with_panic`
#' @export
unsafe_C_test_nested_with_panic <- function() {
    .Call(C_test_nested_with_panic)
}

#' @source Generated by miniextendr from Rust fn `C_test_deep_with_r_thread_sequence`
#' @export
unsafe_C_test_deep_with_r_thread_sequence <- function() {
    .Call(C_test_deep_with_r_thread_sequence)
}

#' @title Coercion Tests
#' @name rpkg_coercion_tests
#' @keywords internal
#' @description Coercion and RNativeType tests
#' @examples
#' test_coerce_identity(1L)
#' test_coerce_widen(1L)
#' test_try_coerce_f64_to_i32(1.2)
#' test_coerce_attr_u16(10L)
#' test_per_arg_coerce_first(10L, 5L)
#' @aliases test_coerce_identity test_coerce_widen test_coerce_bool_to_int
#' @aliases test_coerce_via_helper test_try_coerce_f64_to_i32
#' @aliases test_rnative_newtype test_rnative_named_field
#' @aliases test_coerce_attr_u16 test_coerce_attr_i16 test_coerce_attr_vec_u16
#' @aliases test_coerce_attr_f32 test_coerce_attr_with_invisible
#' @aliases test_per_arg_coerce_first test_per_arg_coerce_second
#' @aliases test_per_arg_coerce_both test_per_arg_coerce_vec
#' @source Generated by miniextendr from Rust fn `test_coerce_identity`
#' @export
test_coerce_identity <- function(x) {
    .Call(C_test_coerce_identity, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_widen`
#' @export
test_coerce_widen <- function(x) {
    .Call(C_test_coerce_widen, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_bool_to_int`
#' @export
test_coerce_bool_to_int <- function(x) {
    .Call(C_test_coerce_bool_to_int, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_via_helper`
#' @export
test_coerce_via_helper <- function(x) {
    .Call(C_test_coerce_via_helper, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_try_coerce_f64_to_i32`
#' @export
test_try_coerce_f64_to_i32 <- function(x) {
    .Call(C_test_try_coerce_f64_to_i32, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_rnative_newtype`
#' @export
test_rnative_newtype <- function(id) {
    .Call(C_test_rnative_newtype, .call = match.call(), id)
}

#' @source Generated by miniextendr from Rust fn `test_rnative_named_field`
#' @export
test_rnative_named_field <- function(temp) {
    .Call(C_test_rnative_named_field, .call = match.call(), temp)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_attr_u16`
#' @export
test_coerce_attr_u16 <- function(x) {
    .Call(C_test_coerce_attr_u16, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_attr_i16`
#' @export
test_coerce_attr_i16 <- function(x) {
    .Call(C_test_coerce_attr_i16, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_attr_vec_u16`
#' @export
test_coerce_attr_vec_u16 <- function(x) {
    .Call(C_test_coerce_attr_vec_u16, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_attr_f32`
#' @export
test_coerce_attr_f32 <- function(x) {
    .Call(C_test_coerce_attr_f32, .call = match.call(), x)
}

#' @source Generated by miniextendr from Rust fn `test_coerce_attr_with_invisible`
#' @export
test_coerce_attr_with_invisible <- function(x) {
    invisible(.Call(C_test_coerce_attr_with_invisible, .call = match.call(), x))
}

#' @source Generated by miniextendr from Rust fn `test_per_arg_coerce_first`
#' @export
test_per_arg_coerce_first <- function(x, y) {
    .Call(C_test_per_arg_coerce_first, .call = match.call(), x, y)
}

#' @source Generated by miniextendr from Rust fn `test_per_arg_coerce_second`
#' @export
test_per_arg_coerce_second <- function(x, y) {
    .Call(C_test_per_arg_coerce_second, .call = match.call(), x, y)
}

#' @source Generated by miniextendr from Rust fn `test_per_arg_coerce_both`
#' @export
test_per_arg_coerce_both <- function(x, y) {
    .Call(C_test_per_arg_coerce_both, .call = match.call(), x, y)
}

#' @source Generated by miniextendr from Rust fn `test_per_arg_coerce_vec`
#' @export
test_per_arg_coerce_vec <- function(x, y) {
    .Call(C_test_per_arg_coerce_vec, .call = match.call(), x, y)
}

#' @title Visibility and Interrupt Tests
#' @name rpkg_visibility_interrupts
#' @keywords internal
#' @description Visibility and interrupt checks
#' @examples
#' invisibly_return_no_arrow()
#' force_invisible_i32()
#' with_interrupt_check(2L)
#' try(invisibly_option_return_none())
#' \dontrun{
#' unsafe_C_check_interupt_after()
#' }
#' @aliases invisibly_return_no_arrow invisibly_return_arrow
#' @aliases invisibly_option_return_none invisibly_option_return_some
#' @aliases invisibly_result_return_ok force_invisible_i32 force_visible_unit
#' @aliases with_interrupt_check unsafe_C_check_interupt_after unsafe_C_check_interupt_unwind
#' @source Generated by miniextendr from Rust fn `invisibly_return_no_arrow`
#' @export
invisibly_return_no_arrow <- function() {
    invisible(.Call(C_invisibly_return_no_arrow, .call = match.call()))
}

#' @source Generated by miniextendr from Rust fn `invisibly_return_arrow`
#' @export
invisibly_return_arrow <- function() {
    invisible(.Call(C_invisibly_return_arrow, .call = match.call()))
}

#' @source Generated by miniextendr from Rust fn `invisibly_option_return_none`
#' @export
invisibly_option_return_none <- function() {
    invisible(.Call(C_invisibly_option_return_none, .call = match.call()))
}

#' @source Generated by miniextendr from Rust fn `invisibly_option_return_some`
#' @export
invisibly_option_return_some <- function() {
    invisible(.Call(C_invisibly_option_return_some, .call = match.call()))
}

#' @source Generated by miniextendr from Rust fn `invisibly_result_return_ok`
#' @export
invisibly_result_return_ok <- function() {
    invisible(.Call(C_invisibly_result_return_ok, .call = match.call()))
}

#' @source Generated by miniextendr from Rust fn `force_invisible_i32`
#' @export
force_invisible_i32 <- function() {
    invisible(.Call(C_force_invisible_i32, .call = match.call()))
}

#' @source Generated by miniextendr from Rust fn `force_visible_unit`
#' @export
force_visible_unit <- function() {
    .Call(C_force_visible_unit, .call = match.call())
}

#' @source Generated by miniextendr from Rust fn `with_interrupt_check`
#' @export
with_interrupt_check <- function(x) {
    .Call(C_with_interrupt_check, .call = match.call(), x)
}

#' @title Thread Builder Tests
#' @name rpkg_thread_builder
#' @keywords internal
#' @description Thread builder and lean-stack tests
#' @examples
#' \dontrun{
#' unsafe_C_test_r_thread_builder()
#' unsafe_C_test_r_thread_builder_spawn_join()
#' unsafe_C_test_spawn_with_r_lean_stack()
#' unsafe_C_test_stack_check_guard_lean()
#' }
#' @aliases unsafe_C_test_r_thread_builder unsafe_C_test_r_thread_builder_spawn_join
#' @aliases unsafe_C_test_spawn_with_r_lean_stack unsafe_C_test_stack_check_guard_lean
#' @source Generated by miniextendr from Rust fn `C_test_r_thread_builder`
#' @export
unsafe_C_test_r_thread_builder <- function() {
    .Call(C_test_r_thread_builder)
}

#' @source Generated by miniextendr from Rust fn `C_test_r_thread_builder_spawn_join`
#' @export
unsafe_C_test_r_thread_builder_spawn_join <- function() {
    .Call(C_test_r_thread_builder_spawn_join)
}

#' @title Miscellaneous Tests
#' @name rpkg_misc
#' @keywords internal
#' @description Miscellaneous test helpers
#' @examples
#' underscore_it_all(1L, 2)
#' r6_standalone_add(1L, 2L)
#' @aliases underscore_it_all r6_standalone_add
#' @source Generated by miniextendr from Rust fn `underscore_it_all`
#' @export
underscore_it_all <- function(private__unused0, private__unused1) {
    invisible(.Call(C_underscore_it_all, .call = match.call(), private__unused0, private__unused1))
}

#' @title ALTREP Helpers
#' @name rpkg_altrep_helpers
#' @keywords internal
#' @description ALTREP convenience wrappers (internal)
#' @examples
#' x <- altrep_compact_int(5L, 1L, 2L)
#' y <- altrep_from_doubles(c(1, 2, 3))
#' z <- altrep_from_strings(c("a", "b"))
#' altrep_lazy_int_seq_is_materialized(lazy_int_seq(1L, 5L, 1L))
#' @aliases altrep_compact_int altrep_from_doubles altrep_from_strings
#' @aliases altrep_from_logicals altrep_from_raw altrep_from_list
#' @aliases altrep_constant_int altrep_lazy_int_seq_is_materialized
#' @source Generated by miniextendr from Rust fn `rpkg_doc_altrep_helpers`
rpkg_doc_altrep_helpers <- function() {
    invisible(.Call(C_rpkg_doc_altrep_helpers, .call = match.call()))
}

#' @source Generated by miniextendr from Rust fn `C_test_spawn_with_r_lean_stack`
#' @export
unsafe_C_test_spawn_with_r_lean_stack <- function() {
    .Call(C_test_spawn_with_r_lean_stack)
}

#' @source Generated by miniextendr from Rust fn `C_test_stack_check_guard_lean`
#' @export
unsafe_C_test_stack_check_guard_lean <- function() {
    .Call(C_test_stack_check_guard_lean)
}

#' @title ReceiverCounter Class
#' @name ReceiverCounter
#' @rdname ReceiverCounter
#' @description Receiver-style counter with `$new()`, `$value()`, `$inc()`, and `$add()` methods.
#' @aliases ReceiverCounter
#' @examples
#' rc <- ReceiverCounter$new(1L)
#' rc$value()
#' rc$inc()
#' rc$add(5L)
#' ReceiverCounter$default_counter()$value()
#' @source Generated by miniextendr from Rust type `ReceiverCounter`
#' @export
ReceiverCounter <- new.env(parent = emptyenv())

ReceiverCounter$new <- function(initial) {
    self <- .Call(C_ReceiverCounter__new, .call = match.call(), initial)
    class(self) <- "ReceiverCounter"
    self
}

ReceiverCounter$value <- function() {
    .Call(C_ReceiverCounter__value, .call = match.call(), self)
}

ReceiverCounter$inc <- function() {
    .Call(C_ReceiverCounter__inc, .call = match.call(), self)
}

ReceiverCounter$add <- function(amount) {
    .Call(C_ReceiverCounter__add, .call = match.call(), self, amount)
}

ReceiverCounter$default_counter <- function() {
    result <- .Call(C_ReceiverCounter__default_counter, .call = match.call())
    class(result) <- "ReceiverCounter"
    result
}

#' @rdname ReceiverCounter
#' @export
`$.ReceiverCounter` <- function(self, name) {
    func <- ReceiverCounter[[name]]
    environment(func) <- environment()
    func
}
#' @rdname ReceiverCounter
#' @export
`[[.ReceiverCounter` <- `$.ReceiverCounter`

#' @title R6 Counter Class
#' @name R6Counter
#' @rdname R6Counter
#' @description R6 counter class that stores a single integer value.
#' @aliases R6Counter
#' @param initial The initial counter value (integer).
#' @param amount The amount to add to the counter (integer).
#' @details
#' **Methods:**
#' - `$new(initial)`: Creates a new counter with the given initial value.
#' - `$value()`: Returns the current value.
#' - `$inc()`: Increments the counter by 1 and returns the new value.
#' - `$add(amount)`: Adds the given amount to the counter and returns the new value.
#' @examples
#' c <- R6Counter$new(1L)
#' c$value()
#' c$inc()
#' c$add(10L)
#' R6Counter$default_counter()$value()
#' @source Generated by miniextendr from Rust type `R6Counter`
#' @importFrom R6 R6Class
#' @param .ptr Internal pointer (used by static methods, not for direct use).
#' @export
R6Counter <- R6::R6Class("R6Counter",
    public = list(
        initialize = function(initial, .ptr = NULL) {
            if (!is.null(.ptr)) {
                private$.ptr <- .ptr
            } else {
                private$.ptr <- .Call(C_R6Counter__new, .call = match.call(), initial)
            }
        },
        value = function() {
            .Call(C_R6Counter__value, .call = match.call(), private$.ptr)
        },
        inc = function() {
            .Call(C_R6Counter__inc, .call = match.call(), private$.ptr)
        },
        add = function(amount) {
            .Call(C_R6Counter__add, .call = match.call(), private$.ptr, amount)
        }
    ),
    private = list(
        .ptr = NULL
    ),
    lock_objects = TRUE,
    lock_class = FALSE,
    cloneable = FALSE
)

#' @name R6Counter$default_counter
#' @rdname R6Counter
R6Counter$default_counter <- function() {
    R6Counter$new(.ptr = .Call(C_R6Counter__default_counter, .call = match.call()))
}

#' @title R6 Accumulator Class
#' @name R6Accumulator
#' @rdname R6Accumulator
#' @description R6 accumulator with running total and count.
#' @aliases R6Accumulator
#' @param value The value to accumulate (numeric).
#' @details
#' **Methods:**
#' - `$new()`: Creates a new accumulator starting at zero.
#' - `$accumulate(value)`: Adds a value and returns the new total.
#' - `$total()`: Returns the current total.
#' - `$count()`: Returns the count of accumulated values.
#' - `$average()`: Returns the average, or NA if no values accumulated.
#' @examples
#' acc <- R6Accumulator$new()
#' acc$accumulate(1.5)
#' acc$total()
#' acc$count()
#' acc$average()
#' @source Generated by miniextendr from Rust type `R6Accumulator`
#' @importFrom R6 R6Class
#' @param .ptr Internal pointer (used by static methods, not for direct use).
#' @export
R6Accumulator <- R6::R6Class("R6Accumulator",
    public = list(
        initialize = function(.ptr = NULL) {
            if (!is.null(.ptr)) {
                private$.ptr <- .ptr
            } else {
                private$.ptr <- .Call(C_R6Accumulator__new, .call = match.call())
            }
        },
        accumulate = function(value) {
            .Call(C_R6Accumulator__accumulate, .call = match.call(), private$.ptr, value)
        },
        total = function() {
            .Call(C_R6Accumulator__total, .call = match.call(), private$.ptr)
        },
        count = function() {
            .Call(C_R6Accumulator__count, .call = match.call(), private$.ptr)
        },
        average = function() {
            .Call(C_R6Accumulator__average, .call = match.call(), private$.ptr)
        }
    ),
    private = list(
        .ptr = NULL
    ),
    lock_objects = TRUE,
    lock_class = FALSE,
    cloneable = FALSE
)

#' @title S3Counter S3 Class
#' @name S3Counter
#' @rdname S3Counter
#' @description S3 counter with `s3_value()`, `s3_inc()`, and `s3_add()` methods.
#' @aliases new_s3counter s3_value s3_inc s3_add s3counter_default_counter
#' @examples
#' x <- new_s3counter(1L)
#' s3_value(x)
#' s3_inc(x)
#' s3_add(x, 5L)
#' s3_value(s3counter_default_counter())
#' @source Generated by miniextendr from `S3Counter::new`
#' @export
new_s3counter <- function(initial) {
    structure(.Call(C_S3Counter__new, .call = match.call(), initial), class = "S3Counter")
}

#' @source Generated by miniextendr from `S3Counter::s3_value`
#' @export
if (!exists("s3_value", mode = "function")) s3_value <- function(x, ...) UseMethod("s3_value")

#' @rdname S3Counter
#' @export
#' @method s3_value S3Counter
s3_value.S3Counter <- function(x, ...) {
    .Call(C_S3Counter__s3_value, .call = match.call(), x)
}

#' @source Generated by miniextendr from `S3Counter::s3_inc`
#' @export
if (!exists("s3_inc", mode = "function")) s3_inc <- function(x, ...) UseMethod("s3_inc")

#' @rdname S3Counter
#' @export
#' @method s3_inc S3Counter
s3_inc.S3Counter <- function(x, ...) {
    .Call(C_S3Counter__s3_inc, .call = match.call(), x)
}

#' @source Generated by miniextendr from `S3Counter::s3_add`
#' @export
if (!exists("s3_add", mode = "function")) s3_add <- function(x, ...) UseMethod("s3_add")

#' @rdname S3Counter
#' @export
#' @method s3_add S3Counter
s3_add.S3Counter <- function(x, amount, ...) {
    .Call(C_S3Counter__s3_add, .call = match.call(), x, amount)
}

#' @rdname S3Counter
#' @source Generated by miniextendr from `S3Counter::default_counter`
#' @export
s3counter_default_counter <- function() {
    structure(.Call(C_S3Counter__default_counter, .call = match.call()), class = "S3Counter")
}

#' @title S7Counter S7 Class
#' @name S7Counter
#' @rdname S7Counter
#' @description S7 counter with `s7_value()`, `s7_inc()`, and `s7_add()` methods.
#' @aliases S7Counter s7_value s7_inc s7_add S7Counter_default_counter
#' @examples
#' x <- S7Counter(1L)
#' s7_value(x)
#' s7_inc(x)
#' s7_add(x, 2L)
#' s7_value(S7Counter_default_counter())
#' @source Generated by miniextendr from Rust type `S7Counter`
#' @importFrom S7 new_class class_any new_object S7_object new_generic method
#' @export
S7Counter <- S7::new_class("S7Counter",
    properties = list(
        .ptr = S7::class_any
    ),
    constructor = function(initial, .ptr = NULL) {
        if (!is.null(.ptr)) {
            S7::new_object(S7::S7_object(), .ptr = .ptr)
        } else {
            S7::new_object(S7::S7_object(), .ptr = .Call(C_S7Counter__new, .call = match.call(), initial))
        }
    }
)

#' @name s7_value
#' @rdname S7Counter
#' @source Generated by miniextendr from `S7Counter::s7_value`
#' @export
if (!exists("s7_value", mode = "function")) s7_value <- S7::new_generic("s7_value", "x", function(x, ...) S7::S7_dispatch())
S7::method(s7_value, S7Counter) <- function(x, ...) .Call(C_S7Counter__s7_value, .call = match.call(), x@.ptr)

#' @name s7_inc
#' @rdname S7Counter
#' @source Generated by miniextendr from `S7Counter::s7_inc`
#' @export
if (!exists("s7_inc", mode = "function")) s7_inc <- S7::new_generic("s7_inc", "x", function(x, ...) S7::S7_dispatch())
S7::method(s7_inc, S7Counter) <- function(x, ...) .Call(C_S7Counter__s7_inc, .call = match.call(), x@.ptr)

#' @name s7_add
#' @rdname S7Counter
#' @source Generated by miniextendr from `S7Counter::s7_add`
#' @export
if (!exists("s7_add", mode = "function")) s7_add <- S7::new_generic("s7_add", "x", function(x, ...) S7::S7_dispatch())
S7::method(s7_add, S7Counter) <- function(x, amount, ...) .Call(C_S7Counter__s7_add, .call = match.call(), x@.ptr, amount)

#' @rdname S7Counter
#' @source Generated by miniextendr from `S7Counter::default_counter`
#' @export
S7Counter_default_counter <- function() {
    S7Counter(.ptr = .Call(C_S7Counter__default_counter, .call = match.call()))
}

#' @title S4Counter S4 Class
#' @name S4Counter
#' @rdname S4Counter
#' @description S4 counter with `s4_value()`, `s4_inc()`, and `s4_add()` methods.
#' @aliases S4Counter s4_value s4_inc s4_add S4Counter_default_counter
#' @examples
#' x <- S4Counter(1L)
#' s4_value(x)
#' s4_inc(x)
#' s4_add(x, 3L)
#' s4_value(S4Counter_default_counter())
#' @source Generated by miniextendr from Rust type `S4Counter`
#' @importFrom methods setClass setGeneric setMethod new isGeneric
#' @slot ptr External pointer to Rust `S4Counter` struct
methods::setClass("S4Counter", slots = c(ptr = "externalptr"))

#' @rdname S4Counter
#' @source Generated by miniextendr from `S4Counter::new`
#' @export
S4Counter <- function(initial) {
    methods::new("S4Counter", ptr = .Call(C_S4Counter__new, .call = match.call(), initial))
}

#' @name s4_value
#' @rdname S4Counter
#' @source Generated by miniextendr from `S4Counter::value`
#' @export
if (!methods::isGeneric("s4_value")) methods::setGeneric("s4_value", function(x, ...) standardGeneric("s4_value"))
#' @exportMethod s4_value
methods::setMethod("s4_value", "S4Counter", function(x, ...) .Call(C_S4Counter__value, .call = match.call(), x@ptr))

#' @name s4_inc
#' @rdname S4Counter
#' @source Generated by miniextendr from `S4Counter::inc`
#' @export
if (!methods::isGeneric("s4_inc")) methods::setGeneric("s4_inc", function(x, ...) standardGeneric("s4_inc"))
#' @exportMethod s4_inc
methods::setMethod("s4_inc", "S4Counter", function(x, ...) .Call(C_S4Counter__inc, .call = match.call(), x@ptr))

#' @name s4_add
#' @rdname S4Counter
#' @source Generated by miniextendr from `S4Counter::add`
#' @export
if (!methods::isGeneric("s4_add")) methods::setGeneric("s4_add", function(x, ...) standardGeneric("s4_add"))
#' @exportMethod s4_add
methods::setMethod("s4_add", "S4Counter", function(x, amount, ...) .Call(C_S4Counter__add, .call = match.call(), x@ptr, amount))

#' @rdname S4Counter
#' @source Generated by miniextendr from `S4Counter::default_counter`
#' @export
S4Counter_default_counter <- function() {
    methods::new("S4Counter", ptr = .Call(C_S4Counter__default_counter, .call = match.call()))
}

