// AUTO-GENERATED — DO NOT EDIT.
//
// Produced on host by `miniextendr_write_wasm_registry`. Compiled on
// wasm32-* targets in place of the linkme distributed_slices.
//
// generator-version: 1
// content-hash:      ff4ae0b298c3bdc1

use ::miniextendr_api::abi::mx_tag;
use ::miniextendr_api::ffi::{R_CallMethodDef, SEXP};
use ::miniextendr_api::registry::{AltrepRegistration, TraitDispatchEntry};
use ::core::ffi::c_void;

unsafe extern "C-unwind" {
    pub fn C_validate_class_args(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_validate_strict_args(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_validate_numeric_args(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_validate_attr_optional(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_validate_with_attribute(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_greetings_with_named_dots(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_greetings_last_as_named_dots(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_greetings_with_nameless_dots(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_greetings_last_as_nameless_dots(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_greetings_with_named_and_unused_dots(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_greetings_last_as_named_and_unused_dots(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_vec_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_vec_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_arrow_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_arrow_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_arrow_bool(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_ndarray_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_ndarray_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_arrow_string(_: SEXP) -> SEXP;
    pub fn C_test_lazy_nalgebra_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_nalgebra_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_lazy_arrow_f64_with_nulls(_: SEXP) -> SEXP;
    pub fn C_test_lazy_arrow_string_with_nulls(_: SEXP) -> SEXP;
    pub fn C_do_nothing(_: SEXP) -> SEXP;
    pub fn C_underscore_it_all(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_i8_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_i8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_u8_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_u8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_f32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_f32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_f64_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_f64_ret(_: SEXP) -> SEXP;
    pub fn C_conv_i16_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_i16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_i64_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_i64_ret(_: SEXP) -> SEXP;
    pub fn C_conv_str_ret(_: SEXP) -> SEXP;
    pub fn C_conv_u16_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_u16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_u32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_u32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_u64_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_u64_ret(_: SEXP) -> SEXP;
    pub fn C_conv_char_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_char_ret(_: SEXP) -> SEXP;
    pub fn C_conv_rlog_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_rlog_ret(_: SEXP) -> SEXP;
    pub fn C_conv_sexp_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_sexp_ret(_: SEXP) -> SEXP;
    pub fn C_conv_isize_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_isize_ret(_: SEXP) -> SEXP;
    pub fn C_conv_rbool_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_rbool_ret(_: SEXP) -> SEXP;
    pub fn C_conv_usize_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_usize_ret(_: SEXP) -> SEXP;
    pub fn C_conv_string_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_i8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_i8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_u8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_u8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_ref_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_f32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_f32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_f64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_f64_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_i16_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_i16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_i64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_u16_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_u16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_u32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_u64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_f64_none(_: SEXP) -> SEXP;
    pub fn C_conv_opt_f64_some(_: SEXP) -> SEXP;
    pub fn C_conv_opt_i32_none(_: SEXP) -> SEXP;
    pub fn C_conv_opt_i32_some(_: SEXP) -> SEXP;
    pub fn C_conv_slice_u8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_bool_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_bool_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_rlog_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_rlog_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_bool_none(_: SEXP) -> SEXP;
    pub fn C_conv_opt_bool_some(_: SEXP) -> SEXP;
    pub fn C_conv_result_f64_ok(_: SEXP) -> SEXP;
    pub fn C_conv_result_i32_ok(_: SEXP) -> SEXP;
    pub fn C_conv_slice_f64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_slice_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_isize_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_usize_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashset_i8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashset_u8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashset_u8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_named_list_get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_named_list_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_i8_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_u8_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_result_f64_err(_: SEXP) -> SEXP;
    pub fn C_conv_result_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_result_i32_err(_: SEXP) -> SEXP;
    pub fn C_conv_slice_rlog_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_i8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_u8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_string_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreeset_i8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreeset_u8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_btreeset_u8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashmap_f64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashmap_f64_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashmap_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashmap_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashset_i16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashset_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashset_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashset_u16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_f32_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_f64_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_i16_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_i32_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_i64_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_string_none(_: SEXP) -> SEXP;
    pub fn C_conv_opt_string_some(_: SEXP) -> SEXP;
    pub fn C_conv_opt_u16_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_u32_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_u64_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_vec_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_option_f32_some(_: SEXP) -> SEXP;
    pub fn C_conv_option_i64_none(_: SEXP) -> SEXP;
    pub fn C_conv_option_u32_some(_: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_f64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_f64_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_ref_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_vec_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_vec_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreemap_f64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_btreemap_f64_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreemap_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_btreemap_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreeset_i16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreeset_i32_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_btreeset_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreeset_u16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashmap_rlog_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashmap_rlog_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashset_rlog_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashset_rlog_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_bool_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_rlog_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_result_string_ok(_: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_bool_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_bool_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_rlog_ret(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_vec(_: SEXP) -> SEXP;
    pub fn C_conv_btreemap_rlog_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_btreemap_rlog_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_isize_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_rbool_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_usize_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_result_string_err(_: SEXP) -> SEXP;
    pub fn C_conv_result_vec_i32_ok(_: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_rbool_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_i8_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashmap_string_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashmap_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_hashset_string_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_hashset_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_list_mut_set_first(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_string_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_vec_string_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_result_vec_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_result_vec_i32_err(_: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_string_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_opt_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_f32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_i16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_u16_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_u32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_vec_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_array(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_empty(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_slice(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_f64(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_i32(_: SEXP) -> SEXP;
    pub fn C_conv_btreemap_string_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_btreemap_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_btreeset_string_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_btreeset_string_ret(_: SEXP) -> SEXP;
    pub fn C_conv_named_list_contains(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_hashmap_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_hashset_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_ref_i32_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_option_i64_some_big(_: SEXP) -> SEXP;
    pub fn C_conv_ref_mut_i32_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_hashmap_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_hashmap_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_named_list_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_btreemap_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_ref_i32_none_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_ref_i32_some_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_vec_i32_none_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_vec_i32_some_ret(_: SEXP) -> SEXP;
    pub fn C_conv_slice_mut_u8_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_btreemap_i32_arg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_btreemap_i32_ret(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_array(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_empty(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_slice(_: SEXP) -> SEXP;
    pub fn C_conv_option_i64_some_small(_: SEXP) -> SEXP;
    pub fn C_conv_slice_mut_i32_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_str_keys(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_string(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_i64_ret_big(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_u64_ret_big(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_ext_trait(_: SEXP) -> SEXP;
    pub fn C_conv_opt_vec_string_none_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_vec_string_some_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_slice_i32_total_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_opt_hashmap_i32_none_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_hashmap_i32_some_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_hashset_i32_none_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_hashset_i32_some_ret(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_i64_ret_small(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_i64_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_option_u64_ret_small(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_ext_trait(_: SEXP) -> SEXP;
    pub fn C_conv_opt_btreemap_i32_none_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_btreemap_i32_some_ret(_: SEXP) -> SEXP;
    pub fn C_conv_opt_mut_slice_i32_is_some(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_vec_mut_slice_i32_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_conv_as_named_vector_option_i32(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_isize_ret_small(_: SEXP) -> SEXP;
    pub fn C_conv_vec_option_usize_ret_small(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_heterogeneous(_: SEXP) -> SEXP;
    pub fn C_conv_as_named_list_duplicate_names(_: SEXP) -> SEXP;
    pub fn C_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_add2(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_add3(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_add4(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_error() -> SEXP;
    pub fn C_add_panic(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_add_r_error(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_just_panic() -> SEXP;
    pub fn C_add_left_mut(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_nested_panic(_: SEXP) -> SEXP;
    pub fn C_add_right_mut(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_drop_on_panic(_: SEXP) -> SEXP;
    pub fn C_add_panic_heap(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_add_r_error_heap(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_panic_and_catch() -> SEXP;
    pub fn C_r_error_in_catch() -> SEXP;
    pub fn C_add_left_right_mut(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_error_in_thread() -> SEXP;
    pub fn C_r_print_in_thread() -> SEXP;
    pub fn C_drop_message_on_success(_: SEXP) -> SEXP;
    pub fn C_drop_on_panic_with_move(_: SEXP) -> SEXP;
    pub fn C_take_and_return_nothing(_: SEXP) -> SEXP;
    pub fn C_rayon_par_map(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_par_map2(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_par_map3(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_in_thread(_: SEXP) -> SEXP;
    pub fn C_rayon_with_r_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_num_threads(_: SEXP) -> SEXP;
    pub fn C_rayon_vec_collect(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_parallel_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_parallel_sqrt(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_with_r_matrix(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_parallel_stats(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_with_r_vec_map(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_parallel_sum_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rayon_parallel_filter_positive(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_new_rcrd(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_new_vctr(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_new_list_of_size(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_new_vctr_inherit(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_new_list_of_ptype(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_vctrs_build_error_message(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_widen(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_attr_f32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_attr_i16(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_attr_u16(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_rnative_newtype(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_via_helper(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_bool_to_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_per_arg_coerce_vec(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_attr_vec_u16(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_per_arg_coerce_both(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_rnative_named_field(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_per_arg_coerce_first(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_per_arg_coerce_second(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_try_coerce_f64_to_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_coerce_attr_with_invisible(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_factor_get_color(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_factor_color_levels(_: SEXP) -> SEXP;
    pub fn C_factor_count_colors(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_factor_status_levels(_: SEXP) -> SEXP;
    pub fn C_factor_colors_with_na(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_factor_describe_color(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_factor_get_all_colors(_: SEXP) -> SEXP;
    pub fn C_factor_describe_status(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_factor_priority_levels(_: SEXP) -> SEXP;
    pub fn C_factor_describe_priority(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rarray_matrix_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rarray_vector_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rarray_matrix_dims(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rarray_matrix_column(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_r_thread_builder() -> SEXP;
    pub fn C_test_r_thread_builder_spawn_join() -> SEXP;
    pub fn C_test_worker_simple() -> SEXP;
    pub fn C_worker_drop_on_panic() -> SEXP;
    pub fn C_test_main_thread_r_api(_: SEXP) -> SEXP;
    pub fn C_test_worker_return_f64(_: SEXP) -> SEXP;
    pub fn C_test_worker_return_i32(_: SEXP) -> SEXP;
    pub fn C_test_nested_with_error() -> SEXP;
    pub fn C_test_nested_with_panic() -> SEXP;
    pub fn C_worker_drop_on_success() -> SEXP;
    pub fn C_test_main_thread_r_error(_: SEXP) -> SEXP;
    pub fn C_test_extptr_from_worker() -> SEXP;
    pub fn C_test_wrong_thread_r_api() -> SEXP;
    pub fn C_test_worker_return_string(_: SEXP) -> SEXP;
    pub fn C_test_nested_worker_calls() -> SEXP;
    pub fn C_test_worker_panic_simple() -> SEXP;
    pub fn C_test_nested_with_r_thread() -> SEXP;
    pub fn C_test_worker_with_r_thread() -> SEXP;
    pub fn C_test_nested_multiple_helpers() -> SEXP;
    pub fn C_test_worker_multiple_r_calls() -> SEXP;
    pub fn C_test_worker_panic_with_drops() -> SEXP;
    pub fn C_test_call_worker_fn_from_main() -> SEXP;
    pub fn C_test_worker_panic_in_r_thread() -> SEXP;
    pub fn C_test_nested_helper_from_worker() -> SEXP;
    pub fn C_test_worker_r_calls_then_error() -> SEXP;
    pub fn C_test_worker_r_calls_then_panic() -> SEXP;
    pub fn C_test_worker_r_error_with_drops() -> SEXP;
    pub fn C_test_worker_r_error_in_r_thread() -> SEXP;
    pub fn C_test_deep_with_r_thread_sequence() -> SEXP;
    pub fn C_test_multiple_extptrs_from_worker() -> SEXP;
    pub fn C_test_main_thread_r_error_with_drops(_: SEXP) -> SEXP;
    pub fn C_test_worker_panic_in_r_thread_with_drops() -> SEXP;
    pub fn C_test_collect_empty(_: SEXP) -> SEXP;
    pub fn C_test_collect_range(_: SEXP) -> SEXP;
    pub fn C_test_collect_sines(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_collect_na_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_collect_na_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_collect_squares(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_collect_strings_upper(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_collect_strings_numbered(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_greet(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_with_flag(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_greet_hidden(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_add_with_defaults(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_missing_test_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_missing_test_option(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_missing_test_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_missing_test_present(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__get(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__max(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__min(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__std(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__var(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__last(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__mean(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__ndim(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__first(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__shape(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__get(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__max(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__min(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__std(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__var(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__col(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__max(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__min(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__row(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__std(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__var(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__last(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__mean(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__ndim(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__diag(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__mean(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__ndim(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__product(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__max(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__min(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__std(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__var(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__first(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__shape(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__ncols(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__nrows(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__shape(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__get_many(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__is_empty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__slice_1d(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__mean(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__ndim(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__ones(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__get_2d(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__view_to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__zeros(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__product(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__product(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__from_range(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__get_nd(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__len_nd(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__is_empty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdIntVec__slice_1d(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__is_empty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__flatten(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__product(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__reshape(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__from_rows(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdMatrix__view_to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__is_empty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__shape_nd(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__slice_nd(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__flatten_c(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdVec__is_valid_index(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__axis_slice(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ndarray_roundtrip_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_NdArrayDyn__is_valid_nd(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ndarray_roundtrip_array(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ndarray_roundtrip_matrix(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ndarray_roundtrip_int_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ndarray_roundtrip_int_matrix(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Maps__new(_: SEXP) -> SEXP;
    pub fn C_Maps__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Maps__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_DeepNest__new(_: SEXP) -> SEXP;
    pub fn C_DeepNest__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Rectangle__new(_: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_Rectangle__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithEnums__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Collections__new(_: SEXP) -> SEXP;
    pub fn C_DeepNest__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SerdeRPoint__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_Collections__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Rectangle__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SerdeRPoint__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithEnums__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Collections__empty(_: SEXP) -> SEXP;
    pub fn C_SerdeRPoint3D__new(_: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_Collections__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SerdeRPoint3D__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SerdeRPoint__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithOptionals__to_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithOptionals__mixed(_: SEXP) -> SEXP;
    pub fn C_Rectangle__with_color(_: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_SerdeRPoint3D__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithEnums__new_circle(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithOptionals__from_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithOptionals__all_none(_: SEXP) -> SEXP;
    pub fn C_serde_r_complex_nested(_: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_bool(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithEnums__new_rectangle(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_roundtrip_point(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_tuple(_: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WithOptionals__all_present(_: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_hashmap(_: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_vec_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_vec_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_vec_bool(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_complex(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_vec_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_vec_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_roundtrip_deep_nest(_: SEXP) -> SEXP;
    pub fn C_serde_r_roundtrip_rectangle(_: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_option_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_vec_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_roundtrip_collections(_: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_wrong_type(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_serialize_tuple_struct(_: SEXP) -> SEXP;
    pub fn C_serde_r_roundtrip_optionals_none(_: SEXP) -> SEXP;
    pub fn C_serde_r_deserialize_missing_field(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_serde_r_roundtrip_optionals_present(_: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_compact(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_add_ten(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_bool_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_compact(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_bool_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_uppercase(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_inspect_arrow(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_null_positions(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_null_positions(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_bool_null_positions(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_all_null_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_all_null_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_double_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_double_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_stale_bitmap_demo(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_recordbatch_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_null_positions(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_f64_zero_copy_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_i32_zero_copy_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_all_null_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_recordbatch_null_counts(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_na_string_double_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_demo_error(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_demo_message(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_demo_warning(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_demo_condition(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_demo_error_custom_class(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_demo_warning_custom_class(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_demo_condition_custom_class(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_doc_attr_basic(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_doc_attr_no_params(_: SEXP) -> SEXP;
    pub fn C_assert_utf8_locale_now(_: SEXP) -> SEXP;
    pub fn C_encoding_info_available(_: SEXP) -> SEXP;
    pub fn C_r_backed_rndmat_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndvec_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndvec_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndmat_fill(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndmat_ncol(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndmat_nrow(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdmatrix_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_dot(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndmat_trace(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdmatrix_ncol(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdmatrix_nrow(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_norm(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndvec_double(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdmatrix_scale(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdmatrix_trace(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_scale(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_int_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndmat_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndvec_empty_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndvec_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdmatrix_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rndvec_int_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_int_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_backed_rdvector_empty_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ReceiverCounter__add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ReceiverCounter__inc(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ReceiverCounter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ReceiverCounter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ReceiverCounter__default_counter(_: SEXP) -> SEXP;
    pub fn C_AsCoerceTestData__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceTestData__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceErrorTest__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceTestData__as_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceErrorTest__as_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceTestData__as_integer(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceTestData__as_numeric(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceTestData__as_character(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceErrorTest__as_character(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceTestData__as_data_frame(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AsCoerceErrorTest__as_data_frame(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_backtrace_install_hook(_: SEXP) -> SEXP;
    pub fn C_box_slice_double(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_f64_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_i32_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_raw_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_bool_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_string_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_option_f64_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_option_i32_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_box_slice_option_string_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_condition_ok(_: SEXP) -> SEXP;
    pub fn C_test_condition_chained(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_condition_parse_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ffi_guard_fallback_ok(_: SEXP) -> SEXP;
    pub fn C_ffi_guard_fallback_panic(_: SEXP) -> SEXP;
    pub fn C_ffi_guard_catch_unwind_ok(_: SEXP) -> SEXP;
    pub fn C_test_sexp_equality(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_check_interupt_after() -> SEXP;
    pub fn C_check_interupt_unwind() -> SEXP;
    pub fn C_into_r_as_i64_to_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_defunct_fn(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_superseded_fn(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_also_deprecated(_: SEXP) -> SEXP;
    pub fn C_fully_deprecated(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_LifecycleDemo__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_old_deprecated_fn(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_soft_deprecated_fn(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_experimental_feature(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_LifecycleDemo__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_LifecycleDemo__old_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_LifecycleDemo__legacy_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_LifecycleDemo__experimental_method(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_choices_color(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_choices_mixed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_mixed(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_set_mode(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_choices_correlation(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_choices_multi_color(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_log_level(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_set_status(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_choices_multi_metrics(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_return_mode(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_mode_choices(_: SEXP) -> SEXP;
    pub fn C_match_arg_return_modes(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_set_priority(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_with_default(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_auto_doc_mode(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_auto_doc_modes(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_multi_priority(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_status_choices(_: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode_array(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode_boxed(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode_slice(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_match_arg_priority_choices(_: SEXP) -> SEXP;
    pub fn C_match_arg_mixed__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_match_arg_set_mode__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_match_arg_log_level__match_arg_choices__level(_: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_match_arg_return_mode__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_match_arg_set_status__match_arg_choices__status(_: SEXP) -> SEXP;
    pub fn C_match_arg_with_default__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_match_arg_auto_doc_mode__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_match_arg_return_modes__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_match_arg_auto_doc_modes__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_match_arg_set_priority__match_arg_choices__priority(_: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode_array__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode_boxed__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_match_arg_multi_mode_slice__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_match_arg_multi_priority__match_arg_choices__priorities(_: SEXP) -> SEXP;
    pub fn C_cli_active_progress_bars(_: SEXP) -> SEXP;
    pub fn C_is_widget(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_on_exit_lifo(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_entry_demo(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_create_widget(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_on_exit_short(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_on_exit_no_add(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WrapperDemo__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WrapperDemo__add_by(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_r_post_checks_demo(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WrapperDemo__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_WrapperDemo__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PanickyCounter__try_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_SimpleCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SimpleCounter__trait_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_PanickyCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6TraitCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3TraitCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4TraitCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7TraitCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SimpleCounter__new_counter(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PanickyCounter__new_panicky(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6TraitCounter__new_r6trait(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3TraitCounter__new_s3trait(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4TraitCounter__new_s4trait(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7TraitCounter__new_s7trait(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SimpleCounter__Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PanickyCounter__Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6TraitCounter__Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3TraitCounter__Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4TraitCounter__Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7TraitCounter__Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SimpleCounter__Counter__MAX_VALUE(_: SEXP) -> SEXP;
    pub fn C_SimpleCounter__Counter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PanickyCounter__Counter__MAX_VALUE(_: SEXP) -> SEXP;
    pub fn C_PanickyCounter__Counter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6TraitCounter__Counter__MAX_VALUE(_: SEXP) -> SEXP;
    pub fn C_R6TraitCounter__Counter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3TraitCounter__Counter__MAX_VALUE(_: SEXP) -> SEXP;
    pub fn C_S3TraitCounter__Counter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4TraitCounter__Counter__MAX_VALUE(_: SEXP) -> SEXP;
    pub fn C_S4TraitCounter__Counter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7TraitCounter__Counter__MAX_VALUE(_: SEXP) -> SEXP;
    pub fn C_S7TraitCounter__Counter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SimpleCounter__Counter__checked_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_PanickyCounter__Counter__checked_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6TraitCounter__Counter__checked_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3TraitCounter__Counter__checked_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4TraitCounter__Counter__checked_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7TraitCounter__Counter__checked_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_SimpleCounter__Counter__default_initial(_: SEXP) -> SEXP;
    pub fn C_PanickyCounter__Counter__default_initial(_: SEXP) -> SEXP;
    pub fn C_R6TraitCounter__Counter__default_initial(_: SEXP) -> SEXP;
    pub fn C_S3TraitCounter__Counter__default_initial(_: SEXP) -> SEXP;
    pub fn C_S4TraitCounter__Counter__default_initial(_: SEXP) -> SEXP;
    pub fn C_S7TraitCounter__Counter__default_initial(_: SEXP) -> SEXP;
    pub fn C_zero_copy_cow_f64_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_cow_i32_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_cow_f64_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_cow_str_is_borrowed(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_protected_strvec_unique(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_vec_cow_str_all_borrowed(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_alloc_r_backed(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_sexprec_offset(_: SEXP) -> SEXP;
    pub fn C_zero_copy_vec_f64_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_f64_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_f64_sliced(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_i32_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_u8_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_f64_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_i32_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_f64_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_i32_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_zero_copy_arrow_f64_computed_is_different(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rust_get_stderr(_: SEXP) -> SEXP;
    pub fn C_rust_get_stdout(_: SEXP) -> SEXP;
    pub fn C_rot13_connection(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_cursor_connection(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_memory_connection(_: SEXP) -> SEXP;
    pub fn C_counter_connection(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rust_write_to_null(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rust_write_to_stderr(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_uppercase_connection(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_empty_cursor_connection(_: SEXP) -> SEXP;
    pub fn C_string_input_connection(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rust_get_null_connection(_: SEXP) -> SEXP;
    pub fn C_test_i32_sum(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_strict_echo_i64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_f64_to_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_i32_to_f64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_u8_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_f64_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_i32_add_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_logical_and(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_logical_not(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_u8_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_StrictCounter__add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_StrictCounter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_f64_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_f64_multiply(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_i32_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_u8_slice_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_u8_slice_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_f64_slice_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_f64_slice_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_i32_slice_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_i32_slice_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_strict_echo_vec_i64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_f64_slice_mean(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_i32_slice_last(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_i32_slice_first(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_logical_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_logical_slice_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_StrictCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_strict_echo_vec_option_i64(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_logical_slice_all_true(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_logical_slice_any_true(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_join(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_chain(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_select(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_window(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_columns(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_subquery(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_aggregate(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_sql_query(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_global_agg(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_df_sort_limit(_: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_FallibleImpl__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_error_in_r_panic(_: SEXP) -> SEXP;
    pub fn C_error_in_r_i32_ok(_: SEXP) -> SEXP;
    pub fn C_error_in_r_normal(_: SEXP) -> SEXP;
    pub fn C_error_in_r_i32_err(_: SEXP) -> SEXP;
    pub fn C_ErrorInRCounter__get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRCounter__inc(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRCounter__new(_: SEXP) -> SEXP;
    pub fn C_ErrorInRS7Gauge__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRR6Widget__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_error_in_r_result_ok(_: SEXP) -> SEXP;
    pub fn C_error_in_r_result_err(_: SEXP) -> SEXP;
    pub fn C_error_in_r_option_none(_: SEXP) -> SEXP;
    pub fn C_error_in_r_option_some(_: SEXP) -> SEXP;
    pub fn C_error_in_r_panic_custom(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRR6Widget__get_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRS7Gauge__set_level(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRS7Gauge__read_level(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_FallibleImpl__inherent_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRCounter__panic_method(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRS7Gauge__panic_method(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRR6Widget__panic_method(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRCounter__failing_method(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRS7Gauge__failing_result(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ErrorInRR6Widget__failing_result(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_FallibleImpl__Fallible__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_FallibleImpl__Fallible__will_panic(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_list_set_elt(_: SEXP) -> SEXP;
    pub fn C_test_strvec_set_str(_: SEXP) -> SEXP;
    pub fn C_test_list_builder_set(_: SEXP) -> SEXP;
    pub fn C_test_list_set_elt_with(_: SEXP) -> SEXP;
    pub fn C_test_strvec_builder_set(_: SEXP) -> SEXP;
    pub fn C_test_list_builder_length(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_reprotect_slot_count(_: SEXP) -> SEXP;
    pub fn C_test_strvec_builder_length(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_reprotect_slot_no_growth(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_reprotect_slot_accumulate(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_list_from_pairs_strings_gctorture(_: SEXP) -> SEXP;
    pub fn C_test_list_from_values_strings_gctorture(_: SEXP) -> SEXP;
    pub fn C_impl_return_vec(_: SEXP) -> SEXP;
    pub fn C_impl_return_string(_: SEXP) -> SEXP;
    pub fn C_Calculator__add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_Calculator__get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Calculator__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Calculator__set(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_s4_is_s4(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_s4_has_slot_test(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_s4_class_name_test(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_force_visible_unit(_: SEXP) -> SEXP;
    pub fn C_result_null_on_err(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_result_unwrap_in_r(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_force_invisible_i32(_: SEXP) -> SEXP;
    pub fn C_with_interrupt_check(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_invisibly_return_arrow(_: SEXP) -> SEXP;
    pub fn C_invisibly_return_no_arrow(_: SEXP) -> SEXP;
    pub fn C_invisibly_result_return_ok(_: SEXP) -> SEXP;
    pub fn C_invisibly_option_return_none(_: SEXP) -> SEXP;
    pub fn C_invisibly_option_return_some(_: SEXP) -> SEXP;
    pub fn C_altrep_sexp_check(_: SEXP) -> SEXP;
    pub fn C_altrep_sexp_is_altrep(_: SEXP) -> SEXP;
    pub fn C_altrep_sexp_materialize_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_sexp_materialize_real(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_sexp_materialize_strings(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_ensure_materialized_int(_: SEXP) -> SEXP;
    pub fn C_altrep_ensure_materialized_str(_: SEXP) -> SEXP;
    pub fn C_extptr_point_new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_extptr_is_point(_: SEXP) -> SEXP;
    pub fn C_extptr_null_test(_: SEXP) -> SEXP;
    pub fn C_extptr_counter_new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_extptr_is_counter(_: SEXP) -> SEXP;
    pub fn C_extptr_counter_get(_: SEXP) -> SEXP;
    pub fn C_extptr_point_get_x(_: SEXP) -> SEXP;
    pub fn C_extptr_point_get_y(_: SEXP) -> SEXP;
    pub fn C_extptr_counter_increment(_: SEXP) -> SEXP;
    pub fn C_test_extptr_on_main_thread(_: SEXP) -> SEXP;
    pub fn C_extptr_type_mismatch_test(_: SEXP) -> SEXP;
    pub fn C_test_json_point(_: SEXP) -> SEXP;
    pub fn C_test_json_config(_: SEXP) -> SEXP;
    pub fn C_test_fromjson_bad(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_fromjson_config(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_json_vec_points(_: SEXP) -> SEXP;
    pub fn C_test_json_pretty_point(_: SEXP) -> SEXP;
    pub fn C_test_fromjson_point_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_mx_point_new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_mx_point_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_mx_obs_create(_: SEXP) -> SEXP;
    pub fn C_mx_season_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_mx_derived_ints(_: SEXP) -> SEXP;
    pub fn C_mx_record_create(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_mx_season_summer(_: SEXP) -> SEXP;
    pub fn C_mx_verbosity_check(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedSimpleCounter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AtomicCounter__new_atomic(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedSimpleCounter__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AtomicCounter__SharedCounter__add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_AtomicCounter__SharedCounter__reset(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AtomicCounter__SharedCounter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_AtomicCounter__SharedCounter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedSimpleCounter__SharedCounter__add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedSimpleCounter__SharedCounter__reset(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedSimpleCounter__SharedCounter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedSimpleCounter__SharedCounter__increment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_host(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_path(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_query(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_scheme(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_fragment(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_is_valid(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_roundtrip_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_url_full_components(_: SEXP) -> SEXP;
    pub fn C_url_port_or_default(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_hybrid_as_ptr(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_hybrid_as_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ptr_list_as_ptr(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_attr_prefer_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_hybrid_as_native(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_plain_option_i32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ptr_list_as_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_attr_prefer_native(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_native_list_as_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_native_list_as_native(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_attr_prefer_externalptr(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_attr_prefer_list_option(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_create_events_df(_: SEXP) -> SEXP;
    pub fn C_create_people_df(_: SEXP) -> SEXP;
    pub fn C_create_points_df(_: SEXP) -> SEXP;
    pub fn C_create_shapes_df(_: SEXP) -> SEXP;
    pub fn C_make_signal_factor(_: SEXP) -> SEXP;
    pub fn C_create_events_split(_: SEXP) -> SEXP;
    pub fn C_create_shapes_split(_: SEXP) -> SEXP;
    pub fn C_create_scored_items_df(_: SEXP) -> SEXP;
    pub fn C_create_tuple_sig_split(_: SEXP) -> SEXP;
    pub fn C_create_unit_status_split(_: SEXP) -> SEXP;
    pub fn C_create_expanded_points_df(_: SEXP) -> SEXP;
    pub fn C_create_sensor_readings_df(_: SEXP) -> SEXP;
    pub fn C_create_single_event_split(_: SEXP) -> SEXP;
    pub fn C_SharedData__new(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedData__get_x(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedData__get_y(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_into_sexp_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_SharedData__get_label(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_gc_stress_dataframe_map(_: SEXP) -> SEXP;
    pub fn C_gc_stress_jiff_zoned_vec(_: SEXP) -> SEXP;
    pub fn C_gc_stress_dataframe_struct(_: SEXP) -> SEXP;
    pub fn C_gc_stress_native_sexp_altrep(_: SEXP) -> SEXP;
    pub fn C_gc_stress_vec_option_borrowed(_: SEXP) -> SEXP;
    pub fn C_gc_stress_dataframe_nested_enum(_: SEXP) -> SEXP;
    pub fn C_gc_stress_vec_option_collection(_: SEXP) -> SEXP;
    pub fn C_into_r_error_inner(_: SEXP) -> SEXP;
    pub fn C_into_r_error_length_overflow(_: SEXP) -> SEXP;
    pub fn C_into_r_error_string_too_long(_: SEXP) -> SEXP;
    pub fn C_jiff_date_day(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_span_new(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_time_new(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_date_year(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_span_days(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_time_hour(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_altrep_elt(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_altrep_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_date_month(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_epoch_date(_: SEXP) -> SEXP;
    pub fn C_jiff_span_years(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_zoned_year(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_span_months(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_time_minute(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_time_second(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_zoned_month(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_date_weekday(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_datetime_day(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_datetime_new(_: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_span_is_zero(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_date_tomorrow(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_datetime_hour(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_datetime_year(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_duration_secs(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_zoned_tz_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_zoned_vec_new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_counted_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_date_yesterday(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_datetime_month(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_roundtrip_date(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_span_rcrd_demo(_: SEXP) -> SEXP;
    pub fn C_jiff_zoned_strftime(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_epoch_timestamp(_: SEXP) -> SEXP;
    pub fn C_jiff_roundtrip_zoned(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_zoned_rcrd_demo(_: SEXP) -> SEXP;
    pub fn C_jiff_date_day_of_year(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_option_timestamp(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_span_is_negative(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_altrep_timestamps(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_distant_past_date(_: SEXP) -> SEXP;
    pub fn C_jiff_negative_duration(_: SEXP) -> SEXP;
    pub fn C_jiff_one_hour_duration(_: SEXP) -> SEXP;
    pub fn C_jiff_timestamp_seconds(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_date_last_of_month(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_negative_timestamp(_: SEXP) -> SEXP;
    pub fn C_jiff_roundtrip_date_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_roundtrip_duration(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_timestamp_strftime(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_zoned_start_of_day(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_date_first_of_month(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_roundtrip_timestamp(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_roundtrip_zoned_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_fractional_timestamp(_: SEXP) -> SEXP;
    pub fn C_jiff_roundtrip_timestamp_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_zoned_vec_first_element(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_jiff_counted_altrep_elt_count(_: SEXP) -> SEXP;
    pub fn C_jiff_half_second_before_epoch(_: SEXP) -> SEXP;
    pub fn C_jiff_timestamp_as_millisecond(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_protect_pool_multi(_: SEXP) -> SEXP;
    pub fn C_protect_pool_roundtrip(_: SEXP) -> SEXP;
    pub fn C_sha2_sha256(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_sha2_sha512(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_sha2_sha256_len(_: SEXP) -> SEXP;
    pub fn C_sha2_sha512_len(_: SEXP) -> SEXP;
    pub fn C_sha2_sha256_hello(_: SEXP) -> SEXP;
    pub fn C_sha2_sha256_large(_: SEXP) -> SEXP;
    pub fn C_sha2_sha256_binary_content(_: SEXP) -> SEXP;
    pub fn C_sha2_different_inputs_differ(_: SEXP) -> SEXP;
    pub fn C_time_get_day(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_time_get_year(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_time_get_month(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_time_epoch_date(_: SEXP) -> SEXP;
    pub fn C_time_format_date(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_time_distant_past(_: SEXP) -> SEXP;
    pub fn C_time_epoch_posixct(_: SEXP) -> SEXP;
    pub fn C_time_roundtrip_date(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_time_roundtrip_posixct(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_pretty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_is_table(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_type_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_get_string(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_table_keys(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_array_count(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_mixed_types(_: SEXP) -> SEXP;
    pub fn C_toml_nested_keys(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_decode_config(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_parse_invalid(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_toml_array_of_tables(_: SEXP) -> SEXP;
    pub fn C_uuid_max(_: SEXP) -> SEXP;
    pub fn C_uuid_nil(_: SEXP) -> SEXP;
    pub fn C_uuid_is_nil(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_uuid_new_v4(_: SEXP) -> SEXP;
    pub fn C_uuid_version(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_uuid_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_uuid_roundtrip_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_u8_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_f64_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_f64_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_i32_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_date_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_f64_mean(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_factor_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_posixct_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_arrayref_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_u8_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_f64_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_i32_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_bool_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_date_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_f64_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_i32_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_bool_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_factor_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_recordbatch_ncol(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_recordbatch_nrow(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_string_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_posixct_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_string_null_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_arrayref_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_arrayref_type_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_f64_empty_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_f64_filter_non_null(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_i32_empty_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_recordbatch_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_recordbatch_column_names(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrow_recordbatch_typed_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_borsh_tuple_size(_: SEXP) -> SEXP;
    pub fn C_borsh_invalid_data(_: SEXP) -> SEXP;
    pub fn C_borsh_nested_roundtrip(_: SEXP) -> SEXP;
    pub fn C_borsh_option_roundtrip(_: SEXP) -> SEXP;
    pub fn C_borsh_roundtrip_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_borsh_hashmap_roundtrip(_: SEXP) -> SEXP;
    pub fn C_borsh_roundtrip_doubles(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_borsh_vec_bool_roundtrip(_: SEXP) -> SEXP;
    pub fn C_bytes_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bytes_empty(_: SEXP) -> SEXP;
    pub fn C_bytes_large(_: SEXP) -> SEXP;
    pub fn C_bytes_slice(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bytes_concat(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bytes_empty_len(_: SEXP) -> SEXP;
    pub fn C_bytes_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bytes_all_values(_: SEXP) -> SEXP;
    pub fn C_bytes_mut_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitR6__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS3__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS4__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS7__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitEnv__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitR6__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS3__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS4__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS7__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitEnv__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitEnv__StaticXParam__from_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitR6__MatrixCounter__custom_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitR6__MatrixCounter__custom_get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS3__MatrixCounter__custom_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS3__MatrixCounter__custom_get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS4__MatrixCounter__custom_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS4__MatrixCounter__custom_get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS7__MatrixCounter__custom_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitS7__MatrixCounter__custom_get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitEnv__MatrixCounter__custom_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitEnv__MatrixCounter__custom_get(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_CounterTraitR6__MatrixCounter__default_value(_: SEXP) -> SEXP;
    pub fn C_CounterTraitS3__MatrixCounter__default_value(_: SEXP) -> SEXP;
    pub fn C_CounterTraitS4__MatrixCounter__default_value(_: SEXP) -> SEXP;
    pub fn C_CounterTraitS7__MatrixCounter__default_value(_: SEXP) -> SEXP;
    pub fn C_CounterTraitEnv__MatrixCounter__default_value(_: SEXP) -> SEXP;
    pub fn C_external_slice_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_external_slice_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_docs_demo_three_paras(_: SEXP) -> SEXP;
    pub fn C_SidecarR6__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_SidecarS7__new(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_r6_new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_s3_new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_s4_new(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_s7_new(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_env_new(_: SEXP, _: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_raw_new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarS3_data(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarS3_data(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarEnv_flag(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarEnv_name(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarR6_label(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarR6_value(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarEnv_flag(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarEnv_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarR6_label(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarR6_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_vctrs_new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarEnv_count(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarEnv_score(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarEnv_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarEnv_score(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rdata_sidecar_rawsexp_new(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarS4_slot_int(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarS4_slot_str(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarS7_prop_int(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarS4_slot_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarS4_slot_str(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarS7_prop_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarEnv_raw_slot(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarRaw_byte_val(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarS4_slot_real(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarS7_prop_flag(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarS7_prop_name(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarEnv_raw_slot(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarRaw_byte_val(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarS4_slot_real(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarS7_prop_flag(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarS7_prop_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarVctrs_vec_data(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarVctrs_vec_data(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarRawSexp_env_val(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarRawSexp_int_vec(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarVctrs_vec_label(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarRawSexp_env_val(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarRawSexp_int_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarVctrs_vec_label(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarRawSexp_char_vec(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarRawSexp_func_val(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarRawSexp_list_val(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_get_SidecarRawSexp_real_vec(_: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarRawSexp_char_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarRawSexp_func_val(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarRawSexp_list_val(_: SEXP, _: SEXP) -> SEXP;
    pub fn C__mx_rdata_set_SidecarRawSexp_real_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_find(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_count(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_split(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_captures(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_find_all(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_is_match(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_replace_all(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_regex_replace_first(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_new_percent(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_format_percent(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_proxy_percent(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_restore_percent(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_ptype_abbr_percent(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_cast_double_percent(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_cast_percent_double(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_cast_percent_percent(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_ptype2_double_percent(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_ptype2_percent_double(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_ptype2_percent_percent(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__x(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__y(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_MyFloat__nan(_: SEXP) -> SEXP;
    pub fn C_MyFloat__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_MyFloat__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntVecIter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_GrowableVec__new(_: SEXP) -> SEXP;
    pub fn C_IntSet__contains(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVec__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVec__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__ROrd__cmp(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ChainedError__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_GrowableVec__clear(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RCopy__copy(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RHash__hash(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_GrowableVec__to_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntSet__RToVec__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVec__to_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RClone__clone(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_GrowableVec__from_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RCopy__is_copy(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntSet__RToVec__to_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntSet__RToVec__is_empty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RDebug__debug_str(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RDefault__default(_: SEXP) -> SEXP;
    pub fn C_GrowableVec__RExtend__len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RFromStr__from_str(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntVecIter__RIterator__nth(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntSet__RFromIter__from_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntVecIter__RIterator__next(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntVecIter__RIterator__skip(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ChainedError__without_source(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ExportControlTraitPoint__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntVecIter__RIterator__count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVecIter__collect_all(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RDisplay__as_r_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_GrowableVec__RExtend__is_empty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVecIter__RIterator__nth(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_Point__RDebug__debug_str_pretty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IntVecIter__RIterator__collect_n(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVecIter__RIterator__next(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVecIter__RIterator__skip(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ChainedError__RError__error_chain(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVecIter__RIterator__count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVec__RMakeIter__make_iter(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_MyFloat__RPartialOrd__partial_cmp(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ChainedError__RError__error_message(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_GrowableVec__RExtend__extend_from_vec(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_IterableVecIter__RIterator__collect_n(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_ChainedError__RError__error_chain_length(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ExportControlTraitPoint__RDebug__debug_str(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ExportControlTraitPoint__RDisplay__as_r_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ExportControlTraitPoint__RDebug__debug_str_pretty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bigint_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bigint_mul(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bigint_factorial(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bigint_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bigint_is_positive(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_ones(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_empty(_: SEXP) -> SEXP;
    pub fn C_bitvec_zeros(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_to_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_toggle(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_all_ones(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_from_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_all_zeros(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitvec_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_either_zero(_: SEXP) -> SEXP;
    pub fn C_either_nested(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_either_is_left(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_either_is_right(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_either_make_left(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_either_dbl_or_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_either_int_or_str(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_either_make_right(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_export_control_normal(_: SEXP) -> SEXP;
    pub fn C_export_control_internal(_: SEXP) -> SEXP;
    pub fn C_export_control_noexport(_: SEXP) -> SEXP;
    pub fn C_S3MatchArgPoint__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4MatchArgHolder__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7MatchArgHolder__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7MatchArgHolder__set(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6MatchArgCounter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3MatchArgPoint__label(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvMatchArgCounter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6MatchArgCounter__mode(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_VctrsMatchArgScale__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3MatchArgPoint__relabel(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvMatchArgCounter__count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvMatchArgCounter__reset(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6MatchArgCounter__record(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7MatchArgHolder__current(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4MatchArgHolder__mode_set(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4MatchArgHolder__mode_current(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6MatchArgCounter__describe_level(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4MatchArgHolder__new__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_S7MatchArgHolder__new__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_S7MatchArgHolder__set__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_R6MatchArgCounter__new__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_VctrsMatchArgScale__new__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_EnvMatchArgCounter__new__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_S3MatchArgPoint__relabel__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_R6MatchArgCounter__record__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_EnvMatchArgCounter__reset__match_arg_choices__modes(_: SEXP) -> SEXP;
    pub fn C_S4MatchArgHolder__mode_set__match_arg_choices__mode(_: SEXP) -> SEXP;
    pub fn C_tabled_simple(_: SEXP) -> SEXP;
    pub fn C_tabled_styled(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_tabled_aligned(_: SEXP) -> SEXP;
    pub fn C_tabled_compact(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_tabled_from_vecs(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_tabled_empty_rows(_: SEXP) -> SEXP;
    pub fn C_tabled_single_cell(_: SEXP) -> SEXP;
    pub fn C_tabled_builder_demo(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_tabled_many_columns(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_tabled_struct_table(_: SEXP) -> SEXP;
    pub fn C_tabled_special_chars(_: SEXP) -> SEXP;
    pub fn C_tabled_with_max_width(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_tabled_concat_horizontal(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_unwind_protect_normal() -> SEXP;
    pub fn C_unwind_protect_r_error() -> SEXP;
    pub fn C_unwind_protect_lowlevel_test() -> SEXP;
    pub fn C_new_derived_temp(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_new_derived_point(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_DerivedCurrency__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_new_derived_percent(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_new_derived_rational(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_new_derived_int_lists(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_derived_percent_class_info(_: SEXP) -> SEXP;
    pub fn C_derived_rational_class_info(_: SEXP) -> SEXP;
    pub fn C_DerivedCurrency__format_amounts(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_call_attr_with(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_call_attr_without(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_create_large_par_events(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_create_large_par_points(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_decimal_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_decimal_mul(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_decimal_round(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_decimal_scale(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_decimal_is_zero(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_decimal_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_display_ip(_: SEXP) -> SEXP;
    pub fn C_test_fromstr_ip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_fromstr_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_display_bool(_: SEXP) -> SEXP;
    pub fn C_test_display_number(_: SEXP) -> SEXP;
    pub fn C_test_display_vec_ips(_: SEXP) -> SEXP;
    pub fn C_test_fromstr_vec_ips(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_display_vec_ints(_: SEXP) -> SEXP;
    pub fn C_test_fromstr_vec_ints(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_fromstr_bad_input(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_TypeA__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_TypeB__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_TypeA__get_val(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_TypeB__get_val(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_extptr_any_erased_is(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_extptr_any_into_inner(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_extptr_any_wrong_type_is(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_extptr_any_erased_downcast(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_telemetry_get_count(_: SEXP) -> SEXP;
    pub fn C_telemetry_clear_hook(_: SEXP) -> SEXP;
    pub fn C_telemetry_install_counter(_: SEXP) -> SEXP;
    pub fn C_tinyvec_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_tinyvec_empty(_: SEXP) -> SEXP;
    pub fn C_arrayvec_empty(_: SEXP) -> SEXP;
    pub fn C_tinyvec_at_capacity(_: SEXP) -> SEXP;
    pub fn C_tinyvec_over_capacity(_: SEXP) -> SEXP;
    pub fn C_tinyvec_roundtrip_dbl(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_tinyvec_roundtrip_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrayvec_roundtrip_dbl(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_arrayvec_roundtrip_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_warn_on_elt(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_panic_on_elt(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_message_on_elt(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_panic_at_index(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_condition_on_elt(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_classed_error_on_elt(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_all(_: SEXP) -> SEXP;
    pub fn C_bitflags_xor(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_empty(_: SEXP) -> SEXP;
    pub fn C_bitflags_names(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_union(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_display(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_has_read(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_has_write(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_intersect(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_complement(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_from_strict(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_has_execute(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bitflags_from_truncate(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_columnar_drop(_: SEXP) -> SEXP;
    pub fn C_test_columnar_empty(_: SEXP) -> SEXP;
    pub fn C_test_columnar_nested(_: SEXP) -> SEXP;
    pub fn C_test_columnar_rename(_: SEXP) -> SEXP;
    pub fn C_test_columnar_select(_: SEXP) -> SEXP;
    pub fn C_test_columnar_empty_split(_: SEXP) -> SEXP;
    pub fn C_test_columnar_rename_noop(_: SEXP) -> SEXP;
    pub fn C_test_columnar_tagged_enum(_: SEXP) -> SEXP;
    pub fn C_test_columnar_deep_nesting(_: SEXP) -> SEXP;
    pub fn C_test_columnar_strip_prefix(_: SEXP) -> SEXP;
    pub fn C_test_columnar_serde_flatten(_: SEXP) -> SEXP;
    pub fn C_test_columnar_untagged_enum(_: SEXP) -> SEXP;
    pub fn C_test_columnar_optional_struct(_: SEXP) -> SEXP;
    pub fn C_test_columnar_ext_tagged_split(_: SEXP) -> SEXP;
    pub fn C_test_columnar_int_tagged_split(_: SEXP) -> SEXP;
    pub fn C_test_columnar_with_column_append(_: SEXP) -> SEXP;
    pub fn C_test_columnar_skip_serializing_if(_: SEXP) -> SEXP;
    pub fn C_test_columnar_with_column_replace(_: SEXP) -> SEXP;
    pub fn C_test_columnar_single_variant_split(_: SEXP) -> SEXP;
    pub fn C_PtrSelfTest__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PtrSelfTest__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PtrSelfTest__is_null_ptr(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PtrSelfTest__value_via_ptr(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PtrSelfTest__value_owned_ptr(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PtrSelfTest__set_value_via_ptr(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_indexmap_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_indexmap_keys(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_indexmap_empty(_: SEXP) -> SEXP;
    pub fn C_indexmap_single(_: SEXP) -> SEXP;
    pub fn C_indexmap_duplicate_key(_: SEXP) -> SEXP;
    pub fn C_indexmap_roundtrip_dbl(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_indexmap_roundtrip_int(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_indexmap_roundtrip_str(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_indexmap_order_preserved(_: SEXP) -> SEXP;
    pub fn C_nalgebra_solve(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_from_fn(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_inverse(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_reshape(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_determinant(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dvector_dot(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dvector_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dvector_sum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_eigenvalues(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dvector_norm(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dmatrix_ncols(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dmatrix_nrows(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dmatrix_trace(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_from_row_slice(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dmatrix_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dmatrix_transpose(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_dvector_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_nalgebra_svector3_roundtrip(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_refcount_arena_roundtrip(_: SEXP) -> SEXP;
    pub fn C_streaming_int_range(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_streaming_real_squares(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6SensorReading__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6SensorReading__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6SensorReading__raw_bytes(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_float_ceil(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_float_powi(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_float_sqrt(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_num_is_one(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_signed_abs(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_float_floor(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_num_is_zero(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_float_is_nan(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_signed_signum(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_float_is_finite(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_signed_is_negative(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_signed_is_positive(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_is_null(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_is_array(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_array_len(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_is_number(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_is_object(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_is_string(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_to_pretty(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_type_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_object_keys(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_from_key_values(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_json_serialize_point(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_arith_seq(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_boxed_raw(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_im(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_re(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_conj(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_norm(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_is_finite(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_from_parts(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_complex_roundtrip_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_count(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_replace(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_unicode(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_is_match(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_no_match(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_find_flat(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_overlapping(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_aho_test_replace_empty(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_boxed_ints(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_columnar_enum_all_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_u64_mixed(_: SEXP) -> SEXP;
    pub fn C_test_columnar_flatten_all_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_string_mixed(_: SEXP) -> SEXP;
    pub fn C_test_columnar_bytes_with_values(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_bool_all_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_bytes_and_opt_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_bytes_all_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_string_all_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_enum_some_flips_type(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_hashmap_all_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_schema_upgrade_nested(_: SEXP) -> SEXP;
    pub fn C_test_columnar_schema_upgrade_scalar(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_u64_all_none_multi(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_u64_all_none_single(_: SEXP) -> SEXP;
    pub fn C_test_columnar_opt_user_struct_all_none(_: SEXP) -> SEXP;
    pub fn C_test_columnar_compound_different_shapes(_: SEXP) -> SEXP;
    pub fn C_test_columnar_schema_upgrade_multi_none_first(_: SEXP) -> SEXP;
    pub fn C_ptr_identity(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ptr_pick_larger(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_PtrIdentityTest__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_PtrIdentityTest__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_native_sexp_altrep_new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_OptsTarget__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_OptsTarget__OptionsDemo__with_exit(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_OptsTarget__OptionsDemo__with_entry(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_OptsTarget__OptionsDemo__basic_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_OptsTarget__OptionsDemo__with_checks(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_OptsTarget__OptionsDemo__deprecated_method(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_boxed_reals(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_lazy_string(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_leaked_ints(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_static_ints(_: SEXP) -> SEXP;
    pub fn C_unit_circle(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ordered_float_inf(_: SEXP) -> SEXP;
    pub fn C_ordered_float_sort(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ordered_float_is_nan(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ordered_float_neg_inf(_: SEXP) -> SEXP;
    pub fn C_ordered_float_neg_zero(_: SEXP) -> SEXP;
    pub fn C_ordered_float_is_finite(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ordered_float_roundtrip(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ordered_float_sort_special(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_ordered_float_roundtrip_vec(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_constant_int(_: SEXP) -> SEXP;
    pub fn C_lazy_int_seq(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_lazy_squares(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Raiser__id(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__id(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Raiser__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_id(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_id(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_id(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Raiser__raise_error(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__raise_error(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_condition_error_empty(_: SEXP) -> SEXP;
    pub fn C_R6Raiser__raise_message(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Raiser__raise_warning(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__raise_message(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__raise_warning(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_raise_error(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_raise_error(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_condition_error_unicode(_: SEXP) -> SEXP;
    pub fn C_R6Raiser__raise_condition(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__raise_condition(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_raise_error(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_raise_message(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_raise_warning(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_raise_message(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_raise_warning(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_raise_message(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_raise_warning(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_raise_condition(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_raise_condition(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Raiser__raise_error_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__raise_error_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_condition_error_long_message(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_raise_condition(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Raiser__raise_warning_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__raise_warning_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_raise_error_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_raise_error_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Raiser__raise_condition_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Raiser__raise_condition_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_condition_panic_with_int_payload(_: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_raise_error_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_raise_warning_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_raise_warning_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_raise_warning_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Raiser__s3_raise_condition_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Raiser__s7_raise_condition_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_EnvRaiser__env_raise_condition_classed(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_boxed_complex(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_boxed_strings(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_constant_real(_: SEXP) -> SEXP;
    pub fn C_repeating_raw(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_array_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_array_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_array_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_array_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_array_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_hashmap_align_1v1r(_: SEXP) -> SEXP;
    pub fn C_hashmap_align_1vnr(_: SEXP) -> SEXP;
    pub fn C_hashmap_align_nv1r(_: SEXP) -> SEXP;
    pub fn C_hashmap_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_hashmap_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_hashmap_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_hashmap_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_hashmap_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_hashset_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_hashset_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_hashset_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_hashset_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_hashset_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_btreemap_align_1v1r(_: SEXP) -> SEXP;
    pub fn C_btreemap_align_1vnr(_: SEXP) -> SEXP;
    pub fn C_btreemap_align_nv1r(_: SEXP) -> SEXP;
    pub fn C_btreemap_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_btreemap_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_btreemap_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_btreemap_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_btreemap_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_btreeset_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_btreeset_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_btreeset_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_btreeset_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_btreeset_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_singleton_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_singleton_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_vec_width_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_vec_width_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_vec_width_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_vec_width_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_vec_width_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_vec_expand_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_vec_expand_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_vec_expand_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_vec_expand_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_vec_expand_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_vec_opaque_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_vec_opaque_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_vec_opaque_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_vec_opaque_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_vec_opaque_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_boxed_slice_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_boxed_slice_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_boxed_slice_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_boxed_slice_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_boxed_slice_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_nested_list_align_1v1r(_: SEXP) -> SEXP;
    pub fn C_nested_list_align_1vnr(_: SEXP) -> SEXP;
    pub fn C_nested_list_align_nv1r(_: SEXP) -> SEXP;
    pub fn C_nested_list_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_nested_list_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_nested_list_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_nested_list_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_nested_list_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_struct_list_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_struct_list_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_struct_list_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_struct_list_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_struct_list_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_borrowed_str_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_borrowed_str_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_borrowed_str_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_borrowed_str_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_borrowed_str_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_nested_factor_align_1v1r(_: SEXP) -> SEXP;
    pub fn C_nested_factor_align_1vnr(_: SEXP) -> SEXP;
    pub fn C_nested_factor_align_nv1r(_: SEXP) -> SEXP;
    pub fn C_nested_factor_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_nested_factor_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_nested_factor_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_nested_factor_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_nested_factor_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_borrowed_slice_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_borrowed_slice_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_borrowed_slice_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_borrowed_slice_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_borrowed_slice_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_align_1v1r(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_align_1vnr(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_align_nv1r(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_nested_flatten_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_struct_flatten_align_nvnr(_: SEXP) -> SEXP;
    pub fn C_struct_flatten_split_1v1r(_: SEXP) -> SEXP;
    pub fn C_struct_flatten_split_1vnr(_: SEXP) -> SEXP;
    pub fn C_struct_flatten_split_nv1r(_: SEXP) -> SEXP;
    pub fn C_struct_flatten_split_nvnr(_: SEXP) -> SEXP;
    pub fn C_flat_basic_par(_: SEXP) -> SEXP;
    pub fn C_flat_basic_1row(_: SEXP) -> SEXP;
    pub fn C_flat_basic_nrow(_: SEXP) -> SEXP;
    pub fn C_flat_nested_par(_: SEXP) -> SEXP;
    pub fn C_flat_skip_inner(_: SEXP) -> SEXP;
    pub fn C_flat_as_list_par(_: SEXP) -> SEXP;
    pub fn C_flat_mixed_order(_: SEXP) -> SEXP;
    pub fn C_flat_tuple_struct(_: SEXP) -> SEXP;
    pub fn C_flat_as_list_inner(_: SEXP) -> SEXP;
    pub fn C_flat_nested_struct(_: SEXP) -> SEXP;
    pub fn C_flat_renamed_inner(_: SEXP) -> SEXP;
    pub fn C_qual_located_basic(_: SEXP) -> SEXP;
    pub fn C_flat_basic_zero_rows(_: SEXP) -> SEXP;
    pub fn C_flat_mixed_inner_types(_: SEXP) -> SEXP;
    pub fn C_flat_two_struct_fields(_: SEXP) -> SEXP;
    pub fn C_gc_stress_struct_flatten(_: SEXP) -> SEXP;
    pub fn C_flat_two_struct_fields_par(_: SEXP) -> SEXP;
    pub fn C_gc_stress_struct_flatten_nested(_: SEXP) -> SEXP;
    pub fn C_bench_vec_copy(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_boxed_logicals(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_iter_int_range(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_iter_raw_bytes(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_small_vec_copy(_: SEXP) -> SEXP;
    pub fn C_static_strings(_: SEXP) -> SEXP;
    pub fn C_vec_int_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_from_raw(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_sparse_iter_int(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_sparse_iter_raw(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_real_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_from_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_bench_vec_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_constant_logical(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_large_vec_altrep(_: SEXP) -> SEXP;
    pub fn C_range_i64_altrep(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_range_int_altrep(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_sparse_iter_real(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_standalone_dataframe_roundtrip(_: SEXP) -> SEXP;
    pub fn C_boxed_data_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_iter_int_from_u16(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_iter_real_squares(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_iter_string_items(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_range_real_altrep(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_compact_int(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_iter_real_from_f32(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_vec_complex_altrep(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_from_doubles(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_from_strings(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_sparse_iter_logical(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_from_integers(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_altrep_from_logicals(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_integer_sequence_list(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rpkg_enabled_features(_: SEXP) -> SEXP;
    pub fn C_sparse_iter_int_squares(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_iter_logical_alternating(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_lazy_int_seq_is_materialized(_: SEXP) -> SEXP;
    pub fn C_R6Dog__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Dog__breed(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Dog__fetch(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Animal__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Animal__name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Counter__add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Counter__inc(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Counter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Animal__speak(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Cloneable__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Rectangle__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Rectangle__area(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Accumulator__new(_: SEXP) -> SEXP;
    pub fn C_R6NonPortable__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Temperature__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_r6_standalone_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Accumulator__count(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Accumulator__total(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Accumulator__average(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Cloneable__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Cloneable__set_value(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6GoldenRetriever__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Rectangle__get_width(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Rectangle__perimeter(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Temperature__celsius(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Rectangle__get_height(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6GoldenRetriever__owner(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6NonPortable__get_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Accumulator__accumulate(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Temperature__fahrenheit(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Counter__default_counter(_: SEXP) -> SEXP;
    pub fn C_R6Temperature__set_celsius(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_R6Temperature__set_fahrenheit(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Counter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Counter__s3_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Counter__s3_inc(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Counter__s3_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S3Counter__default_counter(_: SEXP) -> SEXP;
    pub fn C_S4Counter__add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Counter__inc(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Counter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Counter__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S4Counter__default_counter(_: SEXP) -> SEXP;
    pub fn C_S7Dog__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Dog__bark(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Range__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Shape__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Animal__new(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Circle__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Config__new(_: SEXP, _: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Strict__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Animal__legs(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Celsius__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Config__name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Counter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Config__score(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Range__length(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Range__s7_end(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Celsius__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Dog__dog_breed(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7PropInner__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7PropOuter__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Counter__s7_add(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Counter__s7_inc(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Fahrenheit__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Range__s7_start(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7PropInner__label(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Config__set_score(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Counter__s7_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Fahrenheit__value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Shape__shape_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7OverrideShape__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Animal__animal_kind(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Circle__circle_area(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Config__get_version(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Config__old_version(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7OverrideCircle__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Range__get_midpoint(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Range__set_midpoint(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7GoldenRetriever__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Strict__describe_any(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Strict__strict_length(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Fahrenheit__to_celsius(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7GoldenRetriever__color(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7PropOuter__inner_value(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7Counter__default_counter(_: SEXP) -> SEXP;
    pub fn C_S7Fahrenheit__from_celsius(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7OverrideShape__shape_kind(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7GoldenRetriever__retriever_name(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_S7OverrideCircle__override_radius(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_log_info(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_log_warn(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_log_debug(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_log_error(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_test_log_set_level(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_int(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_bool(_: SEXP) -> SEXP;
    pub fn C_rng_range(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_normal(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_uniform(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_RngSampler__new(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_guard_test(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_exponential(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_chi_sq_approx(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_with_rng_test(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_with_interrupt(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_rng_worker_uniform(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_RngSampler__seed_hint(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_RngSampler__sample_normal(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
    pub fn C_RngSampler__static_sample(_: SEXP, _: SEXP) -> SEXP;
    pub fn C_RngSampler__sample_uniform(_: SEXP, _: SEXP, _: SEXP) -> SEXP;
}

unsafe extern "C" {
    pub safe fn __mx_altrep_reg_MxDerivedIntsData();
    pub safe fn __mx_altrep_reg_JiffTimestampVecCounted();
    pub safe fn __mx_altrep_reg_WarnAltrepData();
    pub safe fn __mx_altrep_reg_MessageAltrepData();
    pub safe fn __mx_altrep_reg_ConditionAltrepData();
    pub safe fn __mx_altrep_reg_PanickingAltrepData();
    pub safe fn __mx_altrep_reg_LoopStressAltrepData();
    pub safe fn __mx_altrep_reg_ClassedErrorAltrepData();
    pub safe fn __mx_altrep_reg_StreamingIntRangeData();
    pub safe fn __mx_altrep_reg_StreamingRealSquaresData();
    pub safe fn __mx_altrep_reg_ListData();
    pub safe fn __mx_altrep_reg_ArithSeqData();
    pub safe fn __mx_altrep_reg_BoxedIntsData();
    pub safe fn __mx_altrep_reg_StringVecData();
    pub safe fn __mx_altrep_reg_LazyIntSeqData();
    pub safe fn __mx_altrep_reg_LazyStringData();
    pub safe fn __mx_altrep_reg_LogicalVecData();
    pub safe fn __mx_altrep_reg_StaticIntsData();
    pub safe fn __mx_altrep_reg_UnitCircleData();
    pub safe fn __mx_altrep_reg_ConstantIntData();
    pub safe fn __mx_altrep_reg_ConstantRealData();
    pub safe fn __mx_altrep_reg_RepeatingRawData();
    pub safe fn __mx_altrep_reg_SimpleVecIntData();
    pub safe fn __mx_altrep_reg_SimpleVecRawData();
    pub safe fn __mx_altrep_reg_SparseIntIterData();
    pub safe fn __mx_altrep_reg_SparseRawIterData();
    pub safe fn __mx_altrep_reg_StaticStringsData();
    pub safe fn __mx_altrep_reg_SparseRealIterData();
    pub safe fn __mx_altrep_reg_ConstantLogicalData();
    pub safe fn __mx_altrep_reg_InferredVecRealData();
    pub safe fn __mx_altrep_reg_SparseLogicalIterData();
    pub safe fn __mx_altrep_reg_IntegerSequenceListData();
    pub safe fn __mx_altrep_reg_builtin_Box_u8();
    pub safe fn __mx_altrep_reg_builtin_Cow_u8();
    pub safe fn __mx_altrep_reg_builtin_Vec_u8();
    pub safe fn __mx_altrep_reg_builtin_Box_f64();
    pub safe fn __mx_altrep_reg_builtin_Box_i32();
    pub safe fn __mx_altrep_reg_builtin_Cow_f64();
    pub safe fn __mx_altrep_reg_builtin_Cow_i32();
    pub safe fn __mx_altrep_reg_builtin_Vec_f64();
    pub safe fn __mx_altrep_reg_builtin_Vec_i32();
    pub safe fn __mx_altrep_reg_builtin_Box_bool();
    pub safe fn __mx_altrep_reg_builtin_Vec_bool();
    pub safe fn __mx_altrep_reg_builtin_Range_f64();
    pub safe fn __mx_altrep_reg_builtin_Range_i32();
    pub safe fn __mx_altrep_reg_builtin_Range_i64();
    pub safe fn __mx_altrep_reg_builtin_Box_String();
    pub safe fn __mx_altrep_reg_builtin_Vec_String();
    pub safe fn __mx_altrep_reg_builtin_Vec_Cow_str();
    pub safe fn __mx_altrep_reg_builtin_Box_Rcomplex();
    pub safe fn __mx_altrep_reg_builtin_Cow_Rcomplex();
    pub safe fn __mx_altrep_reg_builtin_Vec_Rcomplex();
    pub safe fn __mx_altrep_reg_builtin_Vec_Option_String();
    pub safe fn __mx_altrep_reg_builtin_Vec_Option_Cow_str();
    pub safe fn __mx_altrep_reg_builtin_arrow_Int32Array();
    pub safe fn __mx_altrep_reg_builtin_arrow_UInt8Array();
    pub safe fn __mx_altrep_reg_builtin_arrow_StringArray();
    pub safe fn __mx_altrep_reg_builtin_arrow_BooleanArray();
    pub safe fn __mx_altrep_reg_builtin_arrow_Float64Array();
    pub safe fn __mx_altrep_reg_JiffZonedVec();
    pub safe fn __mx_altrep_reg_JiffTimestampVec();
    pub static __VTABLE_COUNTER_FOR_SIMPLECOUNTER: u8;
    pub static __VTABLE_COUNTER_FOR_PANICKYCOUNTER: u8;
    pub static __VTABLE_COUNTER_FOR_R6TRAITCOUNTER: u8;
    pub static __VTABLE_COUNTER_FOR_S3TRAITCOUNTER: u8;
    pub static __VTABLE_COUNTER_FOR_S4TRAITCOUNTER: u8;
    pub static __VTABLE_COUNTER_FOR_S7TRAITCOUNTER: u8;
    pub static __VTABLE_FALLIBLE_FOR_FALLIBLEIMPL: u8;
    pub static __VTABLE_SHAREDCOUNTER_FOR_ATOMICCOUNTER: u8;
    pub static __VTABLE_SHAREDCOUNTER_FOR_SHAREDSIMPLECOUNTER: u8;
    pub static __VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITR6: u8;
    pub static __VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS3: u8;
    pub static __VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS4: u8;
    pub static __VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS7: u8;
    pub static __VTABLE_STATICXPARAM_FOR_COUNTERTRAITENV: u8;
    pub static __VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITENV: u8;
    pub static __VTABLE_RORD_FOR_POINT: u8;
    pub static __VTABLE_RCOPY_FOR_POINT: u8;
    pub static __VTABLE_RHASH_FOR_POINT: u8;
    pub static __VTABLE_RCLONE_FOR_POINT: u8;
    pub static __VTABLE_RDEBUG_FOR_POINT: u8;
    pub static __VTABLE_RDEFAULT_FOR_POINT: u8;
    pub static __VTABLE_RDISPLAY_FOR_POINT: u8;
    pub static __VTABLE_RFROMSTR_FOR_POINT: u8;
    pub static __VTABLE_RTOVEC_FOR_INTSET: u8;
    pub static __VTABLE_RFROMITER_FOR_INTSET: u8;
    pub static __VTABLE_RPARTIALORD_FOR_MYFLOAT: u8;
    pub static __VTABLE_REXTEND_FOR_GROWABLEVEC: u8;
    pub static __VTABLE_RITERATOR_FOR_INTVECITER: u8;
    pub static __VTABLE_RERROR_FOR_CHAINEDERROR: u8;
    pub static __VTABLE_RMAKEITER_FOR_ITERABLEVEC: u8;
    pub static __VTABLE_RITERATOR_FOR_ITERABLEVECITER: u8;
    pub static __VTABLE_RDEBUG_FOR_EXPORTCONTROLTRAITPOINT: u8;
    pub static __VTABLE_RDISPLAY_FOR_EXPORTCONTROLTRAITPOINT: u8;
    pub static __VTABLE_OPTIONSDEMO_FOR_OPTSTARGET: u8;
}

pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[
    R_CallMethodDef {
        name: c"C_validate_class_args".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_validate_class_args) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_validate_strict_args".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_validate_strict_args) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_validate_numeric_args".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_validate_numeric_args) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_validate_attr_optional".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_validate_attr_optional) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_validate_with_attribute".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_validate_with_attribute) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_greetings_with_named_dots".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_greetings_with_named_dots) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_greetings_last_as_named_dots".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_greetings_last_as_named_dots) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_greetings_with_nameless_dots".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_greetings_with_nameless_dots) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_greetings_last_as_nameless_dots".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_greetings_last_as_nameless_dots) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_greetings_with_named_and_unused_dots".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_greetings_with_named_and_unused_dots) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_greetings_last_as_named_and_unused_dots".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_greetings_last_as_named_and_unused_dots) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_vec_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_vec_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_vec_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_vec_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_arrow_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_arrow_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_arrow_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_arrow_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_arrow_bool".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_arrow_bool) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_ndarray_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_ndarray_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_ndarray_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_ndarray_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_arrow_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_lazy_arrow_string) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_nalgebra_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_nalgebra_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_nalgebra_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_lazy_nalgebra_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_arrow_f64_with_nulls".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_lazy_arrow_f64_with_nulls) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_lazy_arrow_string_with_nulls".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_lazy_arrow_string_with_nulls) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_do_nothing".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_do_nothing) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_underscore_it_all".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_underscore_it_all) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_conv_i8_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_i8_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_i8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_i8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_u8_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_u8_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_u8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_u8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_f32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_f32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_f32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_f32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_f64_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_f64_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_f64_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_f64_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_i16_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_i16_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_i16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_i16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_i64_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_i64_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_i64_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_i64_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_str_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_str_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_u16_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_u16_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_u16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_u16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_u32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_u32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_u32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_u32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_u64_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_u64_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_u64_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_u64_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_char_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_char_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_char_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_char_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_rlog_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_rlog_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_rlog_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_rlog_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_sexp_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_sexp_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_sexp_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_sexp_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_isize_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_isize_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_isize_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_isize_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_rbool_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_rbool_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_rbool_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_rbool_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_usize_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_usize_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_usize_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_usize_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_string_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_string_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_i8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_i8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_i8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_i8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_u8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_u8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_u8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_u8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_ref_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_ref_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_f32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_f32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_f32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_f32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_f64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_f64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_f64_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_f64_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_i16_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_i16_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_i16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_i16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_i64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_i64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_u16_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_u16_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_u16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_u16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_u32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_u32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_u64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_u64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_f64_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_f64_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_f64_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_f64_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_i32_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_i32_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_i32_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_i32_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_slice_u8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_slice_u8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_bool_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_bool_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_bool_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_bool_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_rlog_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_rlog_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_rlog_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_rlog_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_bool_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_bool_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_bool_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_bool_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_result_f64_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_f64_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_result_i32_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_i32_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_slice_f64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_slice_f64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_slice_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_slice_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_isize_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_isize_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_usize_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_usize_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_i8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashset_i8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_u8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashset_u8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_u8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashset_u8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_named_list_get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_named_list_get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_named_list_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_named_list_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_i8_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_i8_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_u8_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_u8_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_result_f64_err".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_f64_err) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_result_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_result_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_result_i32_err".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_i32_err) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_slice_rlog_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_slice_rlog_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_i8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_opt_i8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_u8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_opt_u8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_string_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_string_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_i8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreeset_i8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_u8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_btreeset_u8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_u8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreeset_u8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_f64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashmap_f64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_f64_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashmap_f64_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashmap_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashmap_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_i16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashset_i16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashset_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashset_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_u16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashset_u16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_f32_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_f32_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_f64_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_f64_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_i16_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_i16_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_i32_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_i32_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_i64_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_i64_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_string_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_string_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_string_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_string_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_u16_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_u16_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_u32_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_u32_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_u64_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_u64_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_vec_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_vec_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_option_f32_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_option_f32_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_option_i64_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_option_i64_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_option_u32_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_option_u32_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_f64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_opt_f64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_f64_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_opt_f64_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_opt_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_opt_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_ref_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_ref_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_vec_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_vec_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_vec_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_vec_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_f64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_btreemap_f64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_f64_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreemap_f64_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_btreemap_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreemap_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_i16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreeset_i16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_i32_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_btreeset_i32_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreeset_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_u16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreeset_u16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_rlog_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashmap_rlog_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_rlog_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashmap_rlog_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_rlog_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashset_rlog_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_rlog_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashset_rlog_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_bool_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_bool_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_rlog_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_rlog_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_result_string_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_string_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_bool_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_opt_bool_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_bool_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_opt_bool_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_rlog_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_opt_rlog_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_vec) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_rlog_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_btreemap_rlog_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_rlog_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreemap_rlog_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_isize_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_isize_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_rbool_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_rbool_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_usize_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_usize_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_result_string_err".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_string_err) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_result_vec_i32_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_vec_i32_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_rbool_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_opt_rbool_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_i8_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_i8_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_string_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashmap_string_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashmap_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashmap_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_string_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_hashset_string_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_hashset_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_hashset_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_list_mut_set_first".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_list_mut_set_first) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_string_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_string_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_vec_string_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_vec_string_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_result_vec_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_result_vec_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_result_vec_i32_err".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_result_vec_i32_err) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_string_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_opt_string_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_opt_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_opt_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_f32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_f32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_i16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_i16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_u16_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_u16_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_u32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_u32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_vec_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_vec_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_array".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_array) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_slice".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_slice) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_f64) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_i32) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_string_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_btreemap_string_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_btreemap_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreemap_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_string_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_btreeset_string_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_btreeset_string_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_btreeset_string_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_named_list_contains".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_named_list_contains) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_hashmap_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_hashmap_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_hashset_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_hashset_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_ref_i32_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_ref_i32_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_option_i64_some_big".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_option_i64_some_big) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_ref_mut_i32_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_ref_mut_i32_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_hashmap_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_hashmap_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_hashmap_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_hashmap_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_named_list_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_named_list_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_btreemap_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_btreemap_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_ref_i32_none_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_ref_i32_none_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_ref_i32_some_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_ref_i32_some_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_vec_i32_none_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_vec_i32_none_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_vec_i32_some_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_vec_i32_some_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_slice_mut_u8_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_slice_mut_u8_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_btreemap_i32_arg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_btreemap_i32_arg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_btreemap_i32_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_btreemap_i32_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_array".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_array) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_slice".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_slice) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_option_i64_some_small".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_option_i64_some_small) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_slice_mut_i32_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_slice_mut_i32_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_str_keys".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_str_keys) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_string) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_i64_ret_big".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_i64_ret_big) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_u64_ret_big".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_u64_ret_big) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_ext_trait".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_ext_trait) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_vec_string_none_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_vec_string_none_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_vec_string_some_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_vec_string_some_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_slice_i32_total_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_slice_i32_total_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_hashmap_i32_none_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_hashmap_i32_none_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_hashmap_i32_some_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_hashmap_i32_some_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_hashset_i32_none_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_hashset_i32_none_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_hashset_i32_some_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_hashset_i32_some_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_i64_ret_small".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_i64_ret_small) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_i64_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_option_i64_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_u64_ret_small".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_u64_ret_small) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_ext_trait".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_ext_trait) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_btreemap_i32_none_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_btreemap_i32_none_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_btreemap_i32_some_ret".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_opt_btreemap_i32_some_ret) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_opt_mut_slice_i32_is_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_opt_mut_slice_i32_is_some) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_mut_slice_i32_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_conv_vec_mut_slice_i32_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_vector_option_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_vector_option_i32) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_isize_ret_small".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_isize_ret_small) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_vec_option_usize_ret_small".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_vec_option_usize_ret_small) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_heterogeneous".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_heterogeneous) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_conv_as_named_list_duplicate_names".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_conv_as_named_list_duplicate_names) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_add2".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_add2) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_add3".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_add3) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_add4".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add4) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_r_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_r_error) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_add_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add_panic) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_add_r_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add_r_error) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_just_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_just_panic) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_add_left_mut".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add_left_mut) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_nested_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_panic) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_add_right_mut".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add_right_mut) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_drop_on_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_drop_on_panic) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_add_panic_heap".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add_panic_heap) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_add_r_error_heap".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add_r_error_heap) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_panic_and_catch".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_panic_and_catch) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_r_error_in_catch".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_r_error_in_catch) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_add_left_right_mut".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_add_left_right_mut) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_r_error_in_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_r_error_in_thread) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_r_print_in_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_r_print_in_thread) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_drop_message_on_success".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_drop_message_on_success) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_drop_on_panic_with_move".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_drop_on_panic_with_move) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_take_and_return_nothing".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_take_and_return_nothing) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_rayon_par_map".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_par_map) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_par_map2".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rayon_par_map2) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_rayon_par_map3".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_rayon_par_map3) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_rayon_in_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rayon_in_thread) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_rayon_with_r_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_with_r_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_num_threads".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rayon_num_threads) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_rayon_vec_collect".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_vec_collect) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_parallel_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_parallel_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_parallel_sqrt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_parallel_sqrt) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_with_r_matrix".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rayon_with_r_matrix) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_rayon_parallel_stats".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_parallel_stats) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_with_r_vec_map".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_with_r_vec_map) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_parallel_sum_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_parallel_sum_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rayon_parallel_filter_positive".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rayon_parallel_filter_positive) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_new_rcrd".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_new_rcrd) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_new_vctr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_new_vctr) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_new_list_of_size".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_test_new_list_of_size) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_test_new_vctr_inherit".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_test_new_vctr_inherit) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_test_new_list_of_ptype".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_test_new_list_of_ptype) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_test_vctrs_build_error_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_vctrs_build_error_message) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_widen".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_widen) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_attr_f32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_attr_f32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_attr_i16".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_attr_i16) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_attr_u16".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_attr_u16) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_rnative_newtype".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_rnative_newtype) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_via_helper".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_via_helper) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_bool_to_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_bool_to_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_per_arg_coerce_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_per_arg_coerce_vec) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_attr_vec_u16".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_attr_vec_u16) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_per_arg_coerce_both".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_per_arg_coerce_both) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_rnative_named_field".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_rnative_named_field) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_per_arg_coerce_first".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_per_arg_coerce_first) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_per_arg_coerce_second".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_per_arg_coerce_second) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_try_coerce_f64_to_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_try_coerce_f64_to_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_coerce_attr_with_invisible".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_coerce_attr_with_invisible) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_factor_get_color".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_factor_get_color) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_factor_color_levels".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_factor_color_levels) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_factor_count_colors".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_factor_count_colors) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_factor_status_levels".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_factor_status_levels) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_factor_colors_with_na".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_factor_colors_with_na) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_factor_describe_color".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_factor_describe_color) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_factor_get_all_colors".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_factor_get_all_colors) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_factor_describe_status".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_factor_describe_status) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_factor_priority_levels".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_factor_priority_levels) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_factor_describe_priority".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_factor_describe_priority) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rarray_matrix_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rarray_matrix_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rarray_vector_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rarray_vector_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rarray_matrix_dims".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rarray_matrix_dims) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rarray_matrix_column".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rarray_matrix_column) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_r_thread_builder".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_r_thread_builder) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_r_thread_builder_spawn_join".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_r_thread_builder_spawn_join) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_simple".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_simple) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_worker_drop_on_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_worker_drop_on_panic) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_main_thread_r_api".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_main_thread_r_api) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_worker_return_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_worker_return_f64) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_worker_return_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_worker_return_i32) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_nested_with_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_nested_with_error) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_nested_with_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_nested_with_panic) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_worker_drop_on_success".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_worker_drop_on_success) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_main_thread_r_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_main_thread_r_error) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_extptr_from_worker".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_extptr_from_worker) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_wrong_thread_r_api".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_wrong_thread_r_api) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_return_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_worker_return_string) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_nested_worker_calls".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_nested_worker_calls) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_panic_simple".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_panic_simple) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_nested_with_r_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_nested_with_r_thread) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_with_r_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_with_r_thread) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_nested_multiple_helpers".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_nested_multiple_helpers) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_multiple_r_calls".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_multiple_r_calls) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_panic_with_drops".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_panic_with_drops) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_call_worker_fn_from_main".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_call_worker_fn_from_main) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_panic_in_r_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_panic_in_r_thread) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_nested_helper_from_worker".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_nested_helper_from_worker) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_r_calls_then_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_r_calls_then_error) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_r_calls_then_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_r_calls_then_panic) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_r_error_with_drops".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_r_error_with_drops) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_worker_r_error_in_r_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_r_error_in_r_thread) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_deep_with_r_thread_sequence".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_deep_with_r_thread_sequence) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_multiple_extptrs_from_worker".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_multiple_extptrs_from_worker) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_main_thread_r_error_with_drops".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_main_thread_r_error_with_drops) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_worker_panic_in_r_thread_with_drops".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_test_worker_panic_in_r_thread_with_drops) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_test_collect_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_collect_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_collect_range".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_collect_range) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_collect_sines".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_collect_sines) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_collect_na_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_collect_na_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_collect_na_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_collect_na_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_collect_squares".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_collect_squares) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_collect_strings_upper".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_collect_strings_upper) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_collect_strings_numbered".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_collect_strings_numbered) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_greet".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_greet) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_with_flag".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_with_flag) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_greet_hidden".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_greet_hidden) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_add_with_defaults".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_add_with_defaults) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_missing_test_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_missing_test_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_missing_test_option".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_missing_test_option) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_missing_test_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_missing_test_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_missing_test_present".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_missing_test_present) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdVec__get) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdVec__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__max".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__max) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__min".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__min) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__std".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__std) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__var".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__var) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__last".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__last) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__mean".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__mean) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__ndim".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__ndim) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__first".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__first) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__shape".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__shape) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdIntVec__get) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__max".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__max) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__min".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__min) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__std".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__std) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__var".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__var) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__col".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdMatrix__col) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__max".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__max) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__min".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__min) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__row".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdMatrix__row) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__std".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__std) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__var".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__var) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__last".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__last) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__mean".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__mean) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__ndim".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__ndim) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__diag".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__diag) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__mean".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__mean) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__ndim".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__ndim) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__product".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__product) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__max".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__max) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__min".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__min) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__std".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__std) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__var".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__var) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__first".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__first) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__shape".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__shape) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__ncols".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__ncols) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__nrows".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__nrows) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__shape".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__shape) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__get_many".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdVec__get_many) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdVec__is_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__is_empty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__slice_1d".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_NdVec__slice_1d) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__mean".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__mean) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__ndim".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__ndim) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__ones".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__ones) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__get_2d".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_NdMatrix__get_2d) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_NdVec__view_to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdVec__view_to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__zeros".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__zeros) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__product".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__product) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__product".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__product) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__from_range".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_NdVec__from_range) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__get_nd".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__get_nd) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__len_nd".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__len_nd) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__is_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdIntVec__is_empty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdIntVec__slice_1d".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_NdIntVec__slice_1d) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__is_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__is_empty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__flatten".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__flatten) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__product".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__product) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__reshape".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__reshape) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__from_rows".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_NdMatrix__from_rows) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_NdMatrix__view_to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdMatrix__view_to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__is_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__is_empty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__shape_nd".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__shape_nd) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__slice_nd".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__slice_nd) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__flatten_c".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__flatten_c) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdVec__is_valid_index".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdVec__is_valid_index) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__axis_slice".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__axis_slice) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_ndarray_roundtrip_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ndarray_roundtrip_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_NdArrayDyn__is_valid_nd".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_NdArrayDyn__is_valid_nd) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ndarray_roundtrip_array".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ndarray_roundtrip_array) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ndarray_roundtrip_matrix".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ndarray_roundtrip_matrix) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ndarray_roundtrip_int_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ndarray_roundtrip_int_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ndarray_roundtrip_int_matrix".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ndarray_roundtrip_int_matrix) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Maps__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_Maps__new) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_Maps__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Maps__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Maps__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Maps__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_DeepNest__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_DeepNest__new) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_DeepNest__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_DeepNest__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Rectangle__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_Rectangle__new) }),
        numArgs: 5,
    },
    R_CallMethodDef {
        name: c"C_Rectangle__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Rectangle__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithEnums__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WithEnums__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Collections__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_Collections__new) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_DeepNest__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_DeepNest__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SerdeRPoint__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_SerdeRPoint__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_Collections__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Collections__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Rectangle__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Rectangle__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SerdeRPoint__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SerdeRPoint__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithEnums__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WithEnums__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Collections__empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_Collections__empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_SerdeRPoint3D__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_SerdeRPoint3D__new) }),
        numArgs: 5,
    },
    R_CallMethodDef {
        name: c"C_Collections__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Collections__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SerdeRPoint3D__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SerdeRPoint3D__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SerdeRPoint__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SerdeRPoint__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithOptionals__to_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WithOptionals__to_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithOptionals__mixed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_WithOptionals__mixed) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_Rectangle__with_color".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_Rectangle__with_color) }),
        numArgs: 6,
    },
    R_CallMethodDef {
        name: c"C_SerdeRPoint3D__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SerdeRPoint3D__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithEnums__new_circle".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WithEnums__new_circle) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithOptionals__from_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WithOptionals__from_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithOptionals__all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_WithOptionals__all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_complex_nested".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_complex_nested) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_bool".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_bool) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithEnums__new_rectangle".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_WithEnums__new_rectangle) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_roundtrip_point".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_serde_r_roundtrip_point) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_tuple".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_serialize_tuple) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WithOptionals__all_present".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_WithOptionals__all_present) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_hashmap".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_serialize_hashmap) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_vec_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_vec_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_vec_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_vec_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_vec_bool".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_vec_bool) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_complex".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_complex) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_vec_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_vec_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_vec_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_vec_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_roundtrip_deep_nest".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_roundtrip_deep_nest) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_roundtrip_rectangle".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_serde_r_roundtrip_rectangle) }),
        numArgs: 5,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_option_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_option_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_vec_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_serialize_vec_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_roundtrip_collections".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_roundtrip_collections) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_wrong_type".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_wrong_type) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_serialize_tuple_struct".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_serialize_tuple_struct) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_roundtrip_optionals_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_roundtrip_optionals_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_serde_r_deserialize_missing_field".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_serde_r_deserialize_missing_field) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_serde_r_roundtrip_optionals_present".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_serde_r_roundtrip_optionals_present) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_compact".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_compact) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_add_ten".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_add_ten) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_bool_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_bool_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_compact".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_compact) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_bool_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_bool_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_uppercase".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_uppercase) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_inspect_arrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_inspect_arrow) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_null_positions".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_null_positions) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_null_positions".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_null_positions) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_bool_null_positions".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_bool_null_positions) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_all_null_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_all_null_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_all_null_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_all_null_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_double_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_double_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_double_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_double_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_stale_bitmap_demo".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_stale_bitmap_demo) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_recordbatch_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_recordbatch_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_null_positions".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_null_positions) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_f64_zero_copy_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_f64_zero_copy_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_i32_zero_copy_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_i32_zero_copy_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_all_null_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_all_null_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_recordbatch_null_counts".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_recordbatch_null_counts) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_na_string_double_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_na_string_double_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_demo_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_demo_error) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_demo_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_demo_message) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_demo_warning".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_demo_warning) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_demo_condition".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_demo_condition) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_demo_error_custom_class".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_demo_error_custom_class) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_demo_warning_custom_class".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_demo_warning_custom_class) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_demo_condition_custom_class".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_demo_condition_custom_class) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_doc_attr_basic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_doc_attr_basic) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_doc_attr_no_params".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_doc_attr_no_params) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_assert_utf8_locale_now".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_assert_utf8_locale_now) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_encoding_info_available".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_encoding_info_available) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndmat_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndmat_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndvec_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndvec_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndvec_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndvec_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndmat_fill".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_r_backed_rndmat_fill) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndmat_ncol".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndmat_ncol) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndmat_nrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndmat_nrow) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdmatrix_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdmatrix_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_dot".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_dot) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndmat_trace".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndmat_trace) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdmatrix_ncol".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdmatrix_ncol) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdmatrix_nrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdmatrix_nrow) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_norm".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_norm) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndvec_double".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndvec_double) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdmatrix_scale".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_r_backed_rdmatrix_scale) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdmatrix_trace".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdmatrix_trace) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_scale".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_scale) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_int_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_int_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndmat_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndmat_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndvec_empty_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndvec_empty_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndvec_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndvec_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdmatrix_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdmatrix_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rndvec_int_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rndvec_int_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_int_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_int_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_backed_rdvector_empty_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_backed_rdvector_empty_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ReceiverCounter__add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_ReceiverCounter__add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ReceiverCounter__inc".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ReceiverCounter__inc) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ReceiverCounter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ReceiverCounter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ReceiverCounter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ReceiverCounter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ReceiverCounter__default_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ReceiverCounter__default_counter) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceTestData__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceTestData__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceTestData__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_AsCoerceTestData__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceErrorTest__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceErrorTest__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceTestData__as_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceTestData__as_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceErrorTest__as_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceErrorTest__as_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceTestData__as_integer".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceTestData__as_integer) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceTestData__as_numeric".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceTestData__as_numeric) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceTestData__as_character".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceTestData__as_character) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceErrorTest__as_character".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceErrorTest__as_character) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceTestData__as_data_frame".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceTestData__as_data_frame) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AsCoerceErrorTest__as_data_frame".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AsCoerceErrorTest__as_data_frame) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_backtrace_install_hook".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_backtrace_install_hook) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_box_slice_double".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_double) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_f64_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_f64_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_i32_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_i32_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_raw_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_raw_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_bool_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_bool_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_string_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_string_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_option_f64_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_option_f64_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_option_i32_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_option_i32_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_box_slice_option_string_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_box_slice_option_string_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_condition_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_condition_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_condition_chained".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_condition_chained) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_condition_parse_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_condition_parse_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ffi_guard_fallback_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ffi_guard_fallback_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ffi_guard_fallback_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ffi_guard_fallback_panic) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ffi_guard_catch_unwind_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ffi_guard_catch_unwind_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_sexp_equality".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_sexp_equality) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_check_interupt_after".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_check_interupt_after) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_check_interupt_unwind".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_check_interupt_unwind) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_into_r_as_i64_to_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_into_r_as_i64_to_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_defunct_fn".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_defunct_fn) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_superseded_fn".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_superseded_fn) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_also_deprecated".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_also_deprecated) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_fully_deprecated".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_fully_deprecated) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_LifecycleDemo__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_LifecycleDemo__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_old_deprecated_fn".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_old_deprecated_fn) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_soft_deprecated_fn".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_soft_deprecated_fn) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_experimental_feature".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_experimental_feature) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_LifecycleDemo__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_LifecycleDemo__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_LifecycleDemo__old_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_LifecycleDemo__old_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_LifecycleDemo__legacy_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_LifecycleDemo__legacy_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_LifecycleDemo__experimental_method".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_LifecycleDemo__experimental_method) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_choices_color".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_choices_color) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_choices_mixed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_choices_mixed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_match_arg_mixed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_match_arg_mixed) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_match_arg_set_mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_set_mode) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_choices_correlation".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_choices_correlation) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_choices_multi_color".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_choices_multi_color) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_log_level".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_log_level) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_multi_mode) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_set_status".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_set_status) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_choices_multi_metrics".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_choices_multi_metrics) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_match_arg_return_mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_return_mode) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_mode_choices".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_mode_choices) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_return_modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_return_modes) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_set_priority".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_set_priority) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_with_default".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_with_default) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_auto_doc_mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_auto_doc_mode) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_auto_doc_modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_auto_doc_modes) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_priority".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_match_arg_multi_priority) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_match_arg_status_choices".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_status_choices) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode_array".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_multi_mode_array) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode_boxed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_multi_mode_boxed) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode_slice".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_match_arg_multi_mode_slice) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_match_arg_priority_choices".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_priority_choices) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_mixed__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_mixed__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_set_mode__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_set_mode__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_log_level__match_arg_choices__level".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_log_level__match_arg_choices__level) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_multi_mode__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_return_mode__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_return_mode__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_set_status__match_arg_choices__status".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_set_status__match_arg_choices__status) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_with_default__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_with_default__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_auto_doc_mode__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_auto_doc_mode__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_return_modes__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_return_modes__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_auto_doc_modes__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_auto_doc_modes__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_set_priority__match_arg_choices__priority".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_set_priority__match_arg_choices__priority) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode_array__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_multi_mode_array__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode_boxed__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_multi_mode_boxed__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_mode_slice__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_multi_mode_slice__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_match_arg_multi_priority__match_arg_choices__priorities".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_match_arg_multi_priority__match_arg_choices__priorities) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_cli_active_progress_bars".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_cli_active_progress_bars) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_is_widget".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_is_widget) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_on_exit_lifo".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_on_exit_lifo) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r_entry_demo".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_entry_demo) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_create_widget".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_create_widget) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_on_exit_short".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_on_exit_short) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_on_exit_no_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_on_exit_no_add) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WrapperDemo__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WrapperDemo__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WrapperDemo__add_by".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_WrapperDemo__add_by) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_r_post_checks_demo".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_r_post_checks_demo) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WrapperDemo__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WrapperDemo__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_WrapperDemo__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_WrapperDemo__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__try_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_PanickyCounter__try_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SimpleCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__trait_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_SimpleCounter__trait_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PanickyCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6TraitCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6TraitCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3TraitCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3TraitCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4TraitCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4TraitCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7TraitCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7TraitCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__new_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SimpleCounter__new_counter) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__new_panicky".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PanickyCounter__new_panicky) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6TraitCounter__new_r6trait".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6TraitCounter__new_r6trait) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3TraitCounter__new_s3trait".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3TraitCounter__new_s3trait) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4TraitCounter__new_s4trait".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4TraitCounter__new_s4trait) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7TraitCounter__new_s7trait".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7TraitCounter__new_s7trait) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SimpleCounter__Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PanickyCounter__Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6TraitCounter__Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6TraitCounter__Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3TraitCounter__Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3TraitCounter__Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4TraitCounter__Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4TraitCounter__Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7TraitCounter__Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7TraitCounter__Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__Counter__MAX_VALUE".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_SimpleCounter__Counter__MAX_VALUE) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__Counter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SimpleCounter__Counter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__Counter__MAX_VALUE".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_PanickyCounter__Counter__MAX_VALUE) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__Counter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PanickyCounter__Counter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6TraitCounter__Counter__MAX_VALUE".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_R6TraitCounter__Counter__MAX_VALUE) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6TraitCounter__Counter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6TraitCounter__Counter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3TraitCounter__Counter__MAX_VALUE".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S3TraitCounter__Counter__MAX_VALUE) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S3TraitCounter__Counter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3TraitCounter__Counter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4TraitCounter__Counter__MAX_VALUE".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S4TraitCounter__Counter__MAX_VALUE) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S4TraitCounter__Counter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4TraitCounter__Counter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7TraitCounter__Counter__MAX_VALUE".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S7TraitCounter__Counter__MAX_VALUE) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S7TraitCounter__Counter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7TraitCounter__Counter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__Counter__checked_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_SimpleCounter__Counter__checked_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__Counter__checked_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_PanickyCounter__Counter__checked_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6TraitCounter__Counter__checked_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6TraitCounter__Counter__checked_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S3TraitCounter__Counter__checked_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S3TraitCounter__Counter__checked_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S4TraitCounter__Counter__checked_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S4TraitCounter__Counter__checked_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7TraitCounter__Counter__checked_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7TraitCounter__Counter__checked_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_SimpleCounter__Counter__default_initial".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_SimpleCounter__Counter__default_initial) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_PanickyCounter__Counter__default_initial".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_PanickyCounter__Counter__default_initial) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6TraitCounter__Counter__default_initial".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_R6TraitCounter__Counter__default_initial) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S3TraitCounter__Counter__default_initial".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S3TraitCounter__Counter__default_initial) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S4TraitCounter__Counter__default_initial".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S4TraitCounter__Counter__default_initial) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S7TraitCounter__Counter__default_initial".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S7TraitCounter__Counter__default_initial) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_cow_f64_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_cow_f64_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_cow_i32_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_cow_i32_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_cow_f64_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_cow_f64_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_cow_str_is_borrowed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_cow_str_is_borrowed) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_protected_strvec_unique".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_protected_strvec_unique) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_vec_cow_str_all_borrowed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_vec_cow_str_all_borrowed) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_alloc_r_backed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_alloc_r_backed) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_sexprec_offset".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_zero_copy_sexprec_offset) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_vec_f64_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_vec_f64_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_f64_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_f64_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_f64_sliced".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_f64_sliced) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_i32_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_i32_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_u8_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_u8_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_f64_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_f64_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_i32_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_i32_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_f64_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_f64_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_i32_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_i32_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_zero_copy_arrow_f64_computed_is_different".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_zero_copy_arrow_f64_computed_is_different) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rust_get_stderr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rust_get_stderr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_rust_get_stdout".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rust_get_stdout) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_rot13_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rot13_connection) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_cursor_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_cursor_connection) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_memory_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_memory_connection) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_counter_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_counter_connection) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_rust_write_to_null".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rust_write_to_null) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rust_write_to_stderr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rust_write_to_stderr) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_uppercase_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_uppercase_connection) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_empty_cursor_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_empty_cursor_connection) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_string_input_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_string_input_connection) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rust_get_null_connection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rust_get_null_connection) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_i32_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_test_i32_sum) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_strict_echo_i64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_strict_echo_i64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_f64_to_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_f64_to_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_i32_to_f64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_i32_to_f64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_u8_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_u8_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_f64_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_f64_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_i32_add_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_i32_add_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_logical_and".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_logical_and) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_logical_not".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_logical_not) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_u8_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_u8_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_StrictCounter__add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_StrictCounter__add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_StrictCounter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_StrictCounter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_f64_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_f64_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_f64_multiply".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_f64_multiply) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_i32_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_i32_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_u8_slice_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_u8_slice_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_u8_slice_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_u8_slice_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_f64_slice_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_f64_slice_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_f64_slice_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_f64_slice_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_i32_slice_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_i32_slice_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_i32_slice_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_i32_slice_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_strict_echo_vec_i64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_strict_echo_vec_i64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_f64_slice_mean".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_f64_slice_mean) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_i32_slice_last".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_i32_slice_last) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_i32_slice_first".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_i32_slice_first) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_logical_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_logical_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_logical_slice_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_logical_slice_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_StrictCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_StrictCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_strict_echo_vec_option_i64".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_strict_echo_vec_option_i64) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_logical_slice_all_true".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_logical_slice_all_true) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_logical_slice_any_true".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_logical_slice_any_true) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_df_join".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_test_df_join) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_test_df_chain".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_df_chain) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_df_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_df_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_df_select".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_df_select) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_df_window".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_df_window) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_df_columns".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_df_columns) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_df_subquery".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_df_subquery) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_df_aggregate".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_df_aggregate) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_df_sql_query".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_test_df_sql_query) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_test_df_global_agg".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_df_global_agg) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_df_sort_limit".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_test_df_sort_limit) }),
        numArgs: 5,
    },
    R_CallMethodDef {
        name: c"C_FallibleImpl__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_FallibleImpl__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_panic) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_i32_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_i32_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_normal".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_normal) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_i32_err".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_i32_err) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRCounter__get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRCounter__get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRCounter__inc".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRCounter__inc) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRCounter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ErrorInRCounter__new) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRS7Gauge__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRS7Gauge__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRR6Widget__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRR6Widget__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_result_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_result_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_result_err".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_result_err) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_option_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_option_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_option_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_error_in_r_option_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_error_in_r_panic_custom".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_error_in_r_panic_custom) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRR6Widget__get_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRR6Widget__get_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRS7Gauge__set_level".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_ErrorInRS7Gauge__set_level) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRS7Gauge__read_level".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRS7Gauge__read_level) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_FallibleImpl__inherent_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_FallibleImpl__inherent_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRCounter__panic_method".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRCounter__panic_method) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRS7Gauge__panic_method".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRS7Gauge__panic_method) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRR6Widget__panic_method".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRR6Widget__panic_method) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRCounter__failing_method".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRCounter__failing_method) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRS7Gauge__failing_result".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRS7Gauge__failing_result) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ErrorInRR6Widget__failing_result".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ErrorInRR6Widget__failing_result) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_FallibleImpl__Fallible__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_FallibleImpl__Fallible__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_FallibleImpl__Fallible__will_panic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_FallibleImpl__Fallible__will_panic) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_list_set_elt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_list_set_elt) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_strvec_set_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_strvec_set_str) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_list_builder_set".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_list_builder_set) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_list_set_elt_with".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_list_set_elt_with) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_strvec_builder_set".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_strvec_builder_set) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_list_builder_length".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_list_builder_length) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_reprotect_slot_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_reprotect_slot_count) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_strvec_builder_length".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_strvec_builder_length) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_reprotect_slot_no_growth".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_reprotect_slot_no_growth) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_reprotect_slot_accumulate".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_reprotect_slot_accumulate) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_list_from_pairs_strings_gctorture".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_list_from_pairs_strings_gctorture) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_list_from_values_strings_gctorture".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_list_from_values_strings_gctorture) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_impl_return_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_impl_return_vec) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_impl_return_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_impl_return_string) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_Calculator__add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_Calculator__add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_Calculator__get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Calculator__get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Calculator__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Calculator__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Calculator__set".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_Calculator__set) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_s4_is_s4".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_s4_is_s4) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_s4_has_slot_test".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_s4_has_slot_test) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_s4_class_name_test".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_s4_class_name_test) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_force_visible_unit".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_force_visible_unit) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_result_null_on_err".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_result_null_on_err) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_result_unwrap_in_r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_result_unwrap_in_r) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_force_invisible_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_force_invisible_i32) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_with_interrupt_check".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_with_interrupt_check) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_invisibly_return_arrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_invisibly_return_arrow) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_invisibly_return_no_arrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_invisibly_return_no_arrow) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_invisibly_result_return_ok".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_invisibly_result_return_ok) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_invisibly_option_return_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_invisibly_option_return_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_invisibly_option_return_some".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_invisibly_option_return_some) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_altrep_sexp_check".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_altrep_sexp_check) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_altrep_sexp_is_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_altrep_sexp_is_altrep) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_altrep_sexp_materialize_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_sexp_materialize_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_sexp_materialize_real".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_sexp_materialize_real) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_sexp_materialize_strings".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_sexp_materialize_strings) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_ensure_materialized_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_altrep_ensure_materialized_int) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_altrep_ensure_materialized_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_altrep_ensure_materialized_str) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_point_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_extptr_point_new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_extptr_is_point".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_is_point) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_null_test".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_null_test) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_counter_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_extptr_counter_new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_extptr_is_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_is_counter) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_counter_get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_counter_get) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_point_get_x".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_point_get_x) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_point_get_y".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_point_get_y) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_counter_increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_counter_increment) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_extptr_on_main_thread".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_extptr_on_main_thread) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_extptr_type_mismatch_test".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_extptr_type_mismatch_test) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_json_point".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_json_point) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_json_config".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_json_config) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_fromjson_bad".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromjson_bad) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_fromjson_config".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromjson_config) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_json_vec_points".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_json_vec_points) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_json_pretty_point".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_json_pretty_point) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_fromjson_point_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromjson_point_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_mx_point_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_mx_point_new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_mx_point_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_mx_point_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_mx_obs_create".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_mx_obs_create) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_mx_season_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_mx_season_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_mx_derived_ints".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_mx_derived_ints) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_mx_record_create".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_mx_record_create) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_mx_season_summer".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_mx_season_summer) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_mx_verbosity_check".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_mx_verbosity_check) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SharedSimpleCounter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedSimpleCounter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AtomicCounter__new_atomic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AtomicCounter__new_atomic) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SharedSimpleCounter__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedSimpleCounter__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AtomicCounter__SharedCounter__add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_AtomicCounter__SharedCounter__add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_AtomicCounter__SharedCounter__reset".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AtomicCounter__SharedCounter__reset) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AtomicCounter__SharedCounter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AtomicCounter__SharedCounter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_AtomicCounter__SharedCounter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_AtomicCounter__SharedCounter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SharedSimpleCounter__SharedCounter__add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_SharedSimpleCounter__SharedCounter__add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_SharedSimpleCounter__SharedCounter__reset".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedSimpleCounter__SharedCounter__reset) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SharedSimpleCounter__SharedCounter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedSimpleCounter__SharedCounter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SharedSimpleCounter__SharedCounter__increment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedSimpleCounter__SharedCounter__increment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_host".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_host) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_path".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_path) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_query".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_query) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_scheme".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_scheme) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_fragment".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_fragment) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_is_valid".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_is_valid) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_roundtrip_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_roundtrip_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_url_full_components".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_url_full_components) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_url_port_or_default".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_url_port_or_default) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_hybrid_as_ptr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_hybrid_as_ptr) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_hybrid_as_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_hybrid_as_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ptr_list_as_ptr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ptr_list_as_ptr) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_attr_prefer_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_attr_prefer_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_hybrid_as_native".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_hybrid_as_native) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_plain_option_i32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_plain_option_i32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ptr_list_as_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ptr_list_as_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_attr_prefer_native".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_attr_prefer_native) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_native_list_as_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_native_list_as_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_native_list_as_native".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_native_list_as_native) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_attr_prefer_externalptr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_attr_prefer_externalptr) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_attr_prefer_list_option".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_attr_prefer_list_option) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_create_events_df".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_events_df) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_people_df".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_people_df) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_points_df".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_points_df) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_shapes_df".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_shapes_df) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_make_signal_factor".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_make_signal_factor) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_events_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_events_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_shapes_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_shapes_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_scored_items_df".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_scored_items_df) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_tuple_sig_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_tuple_sig_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_unit_status_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_unit_status_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_expanded_points_df".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_expanded_points_df) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_sensor_readings_df".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_sensor_readings_df) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_create_single_event_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_create_single_event_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_SharedData__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_SharedData__new) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_SharedData__get_x".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedData__get_x) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SharedData__get_y".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedData__get_y) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_into_sexp_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_into_sexp_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_SharedData__get_label".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_SharedData__get_label) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_dataframe_map".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_dataframe_map) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_jiff_zoned_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_jiff_zoned_vec) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_dataframe_struct".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_dataframe_struct) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_native_sexp_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_native_sexp_altrep) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_vec_option_borrowed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_vec_option_borrowed) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_dataframe_nested_enum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_dataframe_nested_enum) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_vec_option_collection".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_vec_option_collection) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_into_r_error_inner".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_into_r_error_inner) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_into_r_error_length_overflow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_into_r_error_length_overflow) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_into_r_error_string_too_long".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_into_r_error_string_too_long) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_day".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_day) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_span_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_jiff_span_new) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_jiff_time_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_jiff_time_new) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_year".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_year) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_span_days".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_span_days) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_time_hour".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_time_hour) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_altrep_elt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_jiff_altrep_elt) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_jiff_altrep_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_altrep_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_month".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_month) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_epoch_date".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_epoch_date) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_span_years".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_span_years) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_year".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_zoned_year) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_span_months".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_span_months) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_time_minute".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_time_minute) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_time_second".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_time_second) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_month".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_zoned_month) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_weekday".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_weekday) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_datetime_day".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_datetime_day) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_datetime_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_jiff_datetime_new) }),
        numArgs: 7,
    },
    R_CallMethodDef {
        name: c"C_jiff_span_is_zero".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_span_is_zero) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_tomorrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_tomorrow) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_datetime_hour".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_datetime_hour) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_datetime_year".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_datetime_year) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_duration_secs".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_duration_secs) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_tz_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_zoned_tz_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_vec_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_zoned_vec_new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_counted_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_counted_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_yesterday".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_yesterday) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_datetime_month".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_datetime_month) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_roundtrip_date".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_roundtrip_date) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_span_rcrd_demo".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_span_rcrd_demo) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_strftime".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_jiff_zoned_strftime) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_jiff_epoch_timestamp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_epoch_timestamp) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_roundtrip_zoned".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_roundtrip_zoned) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_rcrd_demo".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_zoned_rcrd_demo) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_day_of_year".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_day_of_year) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_option_timestamp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_option_timestamp) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_span_is_negative".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_span_is_negative) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_altrep_timestamps".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_altrep_timestamps) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_distant_past_date".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_distant_past_date) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_negative_duration".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_negative_duration) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_one_hour_duration".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_one_hour_duration) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_timestamp_seconds".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_timestamp_seconds) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_last_of_month".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_last_of_month) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_negative_timestamp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_negative_timestamp) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_roundtrip_date_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_roundtrip_date_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_roundtrip_duration".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_roundtrip_duration) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_timestamp_strftime".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_jiff_timestamp_strftime) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_start_of_day".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_zoned_start_of_day) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_date_first_of_month".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_date_first_of_month) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_roundtrip_timestamp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_roundtrip_timestamp) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_roundtrip_zoned_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_roundtrip_zoned_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_fractional_timestamp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_fractional_timestamp) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_roundtrip_timestamp_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_roundtrip_timestamp_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_zoned_vec_first_element".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_zoned_vec_first_element) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_jiff_counted_altrep_elt_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_counted_altrep_elt_count) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_half_second_before_epoch".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_jiff_half_second_before_epoch) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_jiff_timestamp_as_millisecond".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_jiff_timestamp_as_millisecond) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_protect_pool_multi".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_protect_pool_multi) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_protect_pool_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_protect_pool_roundtrip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_sha2_sha256".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_sha2_sha256) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_sha2_sha512".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_sha2_sha512) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_sha2_sha256_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_sha2_sha256_len) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_sha2_sha512_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_sha2_sha512_len) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_sha2_sha256_hello".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_sha2_sha256_hello) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_sha2_sha256_large".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_sha2_sha256_large) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_sha2_sha256_binary_content".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_sha2_sha256_binary_content) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_sha2_different_inputs_differ".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_sha2_different_inputs_differ) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_time_get_day".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_time_get_day) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_time_get_year".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_time_get_year) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_time_get_month".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_time_get_month) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_time_epoch_date".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_time_epoch_date) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_time_format_date".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_time_format_date) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_time_distant_past".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_time_distant_past) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_time_epoch_posixct".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_time_epoch_posixct) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_time_roundtrip_date".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_time_roundtrip_date) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_time_roundtrip_posixct".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_time_roundtrip_posixct) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_pretty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_pretty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_is_table".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_is_table) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_type_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_type_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_get_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_toml_get_string) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_toml_table_keys".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_table_keys) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_array_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_toml_array_count) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_toml_mixed_types".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_toml_mixed_types) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_toml_nested_keys".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_nested_keys) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_decode_config".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_decode_config) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_parse_invalid".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_toml_parse_invalid) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_toml_array_of_tables".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_toml_array_of_tables) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_uuid_max".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_uuid_max) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_uuid_nil".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_uuid_nil) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_uuid_is_nil".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_uuid_is_nil) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_uuid_new_v4".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_uuid_new_v4) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_uuid_version".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_uuid_version) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_uuid_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_uuid_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_uuid_roundtrip_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_uuid_roundtrip_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_u8_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_u8_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_f64_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_f64_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_f64_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_f64_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_i32_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_i32_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_date_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_date_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_f64_mean".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_f64_mean) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_factor_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_factor_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_posixct_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_posixct_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_arrayref_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_arrayref_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_u8_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_u8_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_f64_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_f64_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_i32_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_i32_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_bool_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_bool_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_date_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_date_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_f64_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_f64_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_i32_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_i32_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_bool_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_bool_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_factor_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_factor_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_recordbatch_ncol".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_recordbatch_ncol) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_recordbatch_nrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_recordbatch_nrow) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_string_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_string_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_posixct_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_posixct_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_string_null_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_string_null_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_arrayref_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_arrayref_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_arrayref_type_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_arrayref_type_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_f64_empty_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_f64_empty_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_f64_filter_non_null".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_f64_filter_non_null) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_i32_empty_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_i32_empty_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_recordbatch_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_recordbatch_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_recordbatch_column_names".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_recordbatch_column_names) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrow_recordbatch_typed_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrow_recordbatch_typed_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_borsh_tuple_size".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borsh_tuple_size) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borsh_invalid_data".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borsh_invalid_data) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borsh_nested_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borsh_nested_roundtrip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borsh_option_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borsh_option_roundtrip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borsh_roundtrip_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_borsh_roundtrip_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_borsh_hashmap_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borsh_hashmap_roundtrip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borsh_roundtrip_doubles".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_borsh_roundtrip_doubles) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_borsh_vec_bool_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borsh_vec_bool_roundtrip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bytes_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bytes_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bytes_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_bytes_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bytes_large".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_bytes_large) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bytes_slice".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_bytes_slice) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_bytes_concat".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_bytes_concat) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_bytes_empty_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_bytes_empty_len) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bytes_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bytes_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bytes_all_values".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_bytes_all_values) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bytes_mut_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bytes_mut_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitR6__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitR6__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS3__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS3__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS4__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS4__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS7__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS7__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitEnv__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitEnv__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitR6__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitR6__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS3__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS3__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS4__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS4__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS7__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS7__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitEnv__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitEnv__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitEnv__StaticXParam__from_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitEnv__StaticXParam__from_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitR6__MatrixCounter__custom_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_CounterTraitR6__MatrixCounter__custom_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitR6__MatrixCounter__custom_get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitR6__MatrixCounter__custom_get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS3__MatrixCounter__custom_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_CounterTraitS3__MatrixCounter__custom_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS3__MatrixCounter__custom_get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS3__MatrixCounter__custom_get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS4__MatrixCounter__custom_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_CounterTraitS4__MatrixCounter__custom_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS4__MatrixCounter__custom_get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS4__MatrixCounter__custom_get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS7__MatrixCounter__custom_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_CounterTraitS7__MatrixCounter__custom_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS7__MatrixCounter__custom_get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitS7__MatrixCounter__custom_get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitEnv__MatrixCounter__custom_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_CounterTraitEnv__MatrixCounter__custom_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitEnv__MatrixCounter__custom_get".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_CounterTraitEnv__MatrixCounter__custom_get) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitR6__MatrixCounter__default_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_CounterTraitR6__MatrixCounter__default_value) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS3__MatrixCounter__default_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_CounterTraitS3__MatrixCounter__default_value) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS4__MatrixCounter__default_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_CounterTraitS4__MatrixCounter__default_value) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitS7__MatrixCounter__default_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_CounterTraitS7__MatrixCounter__default_value) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_CounterTraitEnv__MatrixCounter__default_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_CounterTraitEnv__MatrixCounter__default_value) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_external_slice_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_external_slice_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_external_slice_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_external_slice_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_docs_demo_three_paras".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_docs_demo_three_paras) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_SidecarR6__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_SidecarR6__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_SidecarS7__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_SidecarS7__new) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_r6_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rdata_sidecar_r6_new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_s3_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rdata_sidecar_s3_new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_s4_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_rdata_sidecar_s4_new) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_s7_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_rdata_sidecar_s7_new) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_env_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_rdata_sidecar_env_new) }),
        numArgs: 5,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_raw_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rdata_sidecar_raw_new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarS3_data".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarS3_data) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarS3_data".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarS3_data) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarEnv_flag".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarEnv_flag) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarEnv_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarEnv_name) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarR6_label".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarR6_label) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarR6_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarR6_value) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarEnv_flag".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarEnv_flag) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarEnv_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarEnv_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarR6_label".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarR6_label) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarR6_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarR6_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_vctrs_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rdata_sidecar_vctrs_new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarEnv_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarEnv_count) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarEnv_score".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarEnv_score) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarEnv_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarEnv_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarEnv_score".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarEnv_score) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rdata_sidecar_rawsexp_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rdata_sidecar_rawsexp_new) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarS4_slot_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarS4_slot_int) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarS4_slot_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarS4_slot_str) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarS7_prop_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarS7_prop_int) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarS4_slot_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarS4_slot_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarS4_slot_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarS4_slot_str) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarS7_prop_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarS7_prop_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarEnv_raw_slot".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarEnv_raw_slot) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarRaw_byte_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarRaw_byte_val) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarS4_slot_real".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarS4_slot_real) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarS7_prop_flag".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarS7_prop_flag) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarS7_prop_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarS7_prop_name) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarEnv_raw_slot".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarEnv_raw_slot) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarRaw_byte_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarRaw_byte_val) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarS4_slot_real".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarS4_slot_real) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarS7_prop_flag".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarS7_prop_flag) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarS7_prop_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarS7_prop_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarVctrs_vec_data".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarVctrs_vec_data) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarVctrs_vec_data".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarVctrs_vec_data) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarRawSexp_env_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarRawSexp_env_val) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarRawSexp_int_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarRawSexp_int_vec) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarVctrs_vec_label".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarVctrs_vec_label) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarRawSexp_env_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarRawSexp_env_val) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarRawSexp_int_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarRawSexp_int_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarVctrs_vec_label".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarVctrs_vec_label) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarRawSexp_char_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarRawSexp_char_vec) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarRawSexp_func_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarRawSexp_func_val) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarRawSexp_list_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarRawSexp_list_val) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_get_SidecarRawSexp_real_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C__mx_rdata_get_SidecarRawSexp_real_vec) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarRawSexp_char_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarRawSexp_char_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarRawSexp_func_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarRawSexp_func_val) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarRawSexp_list_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarRawSexp_list_val) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C__mx_rdata_set_SidecarRawSexp_real_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C__mx_rdata_set_SidecarRawSexp_real_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_regex_find".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_find) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_regex_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_count) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_regex_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_split) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_regex_captures".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_captures) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_regex_find_all".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_find_all) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_regex_is_match".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_is_match) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_regex_replace_all".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_replace_all) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_regex_replace_first".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_regex_replace_first) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_new_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_new_percent) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_format_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_format_percent) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_vec_proxy_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_proxy_percent) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_vec_restore_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_restore_percent) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_vec_ptype_abbr_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_ptype_abbr_percent) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_vec_cast_double_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_cast_double_percent) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_vec_cast_percent_double".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_cast_percent_double) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_vec_cast_percent_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_cast_percent_percent) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_vec_ptype2_double_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_ptype2_double_percent) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_vec_ptype2_percent_double".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_ptype2_percent_double) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_vec_ptype2_percent_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_vec_ptype2_percent_percent) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_Point__x".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__x) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__y".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__y) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_Point__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_MyFloat__nan".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_MyFloat__nan) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_MyFloat__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_MyFloat__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_MyFloat__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_MyFloat__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntVecIter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IntVecIter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_GrowableVec__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_GrowableVec__new) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_IntSet__contains".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_IntSet__contains) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_IterableVec__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IterableVec__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IterableVec__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IterableVec__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__ROrd__cmp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_Point__ROrd__cmp) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ChainedError__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_ChainedError__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_GrowableVec__clear".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_GrowableVec__clear) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RCopy__copy".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RCopy__copy) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RHash__hash".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RHash__hash) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_GrowableVec__to_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_GrowableVec__to_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntSet__RToVec__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IntSet__RToVec__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IterableVec__to_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IterableVec__to_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RClone__clone".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RClone__clone) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_GrowableVec__from_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_GrowableVec__from_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RCopy__is_copy".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RCopy__is_copy) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntSet__RToVec__to_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IntSet__RToVec__to_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntSet__RToVec__is_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IntSet__RToVec__is_empty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RDebug__debug_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RDebug__debug_str) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RDefault__default".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_Point__RDefault__default) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_GrowableVec__RExtend__len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_GrowableVec__RExtend__len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RFromStr__from_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RFromStr__from_str) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntVecIter__RIterator__nth".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_IntVecIter__RIterator__nth) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_IntSet__RFromIter__from_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IntSet__RFromIter__from_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntVecIter__RIterator__next".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IntVecIter__RIterator__next) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntVecIter__RIterator__skip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_IntVecIter__RIterator__skip) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ChainedError__without_source".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ChainedError__without_source) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ExportControlTraitPoint__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ExportControlTraitPoint__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntVecIter__RIterator__count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IntVecIter__RIterator__count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IterableVecIter__collect_all".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IterableVecIter__collect_all) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_Point__RDisplay__as_r_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RDisplay__as_r_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_GrowableVec__RExtend__is_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_GrowableVec__RExtend__is_empty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IterableVecIter__RIterator__nth".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_IterableVecIter__RIterator__nth) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_Point__RDebug__debug_str_pretty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_Point__RDebug__debug_str_pretty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IntVecIter__RIterator__collect_n".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_IntVecIter__RIterator__collect_n) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_IterableVecIter__RIterator__next".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IterableVecIter__RIterator__next) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IterableVecIter__RIterator__skip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_IterableVecIter__RIterator__skip) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ChainedError__RError__error_chain".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ChainedError__RError__error_chain) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IterableVecIter__RIterator__count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IterableVecIter__RIterator__count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_IterableVec__RMakeIter__make_iter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_IterableVec__RMakeIter__make_iter) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_MyFloat__RPartialOrd__partial_cmp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_MyFloat__RPartialOrd__partial_cmp) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ChainedError__RError__error_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ChainedError__RError__error_message) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_GrowableVec__RExtend__extend_from_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_GrowableVec__RExtend__extend_from_vec) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_IterableVecIter__RIterator__collect_n".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_IterableVecIter__RIterator__collect_n) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_ChainedError__RError__error_chain_length".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ChainedError__RError__error_chain_length) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ExportControlTraitPoint__RDebug__debug_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ExportControlTraitPoint__RDebug__debug_str) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ExportControlTraitPoint__RDisplay__as_r_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ExportControlTraitPoint__RDisplay__as_r_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ExportControlTraitPoint__RDebug__debug_str_pretty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ExportControlTraitPoint__RDebug__debug_str_pretty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bigint_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_bigint_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_bigint_mul".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_bigint_mul) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_bigint_factorial".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bigint_factorial) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bigint_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bigint_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bigint_is_positive".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bigint_is_positive) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_ones".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_ones) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_bitvec_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bitvec_zeros".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_zeros) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_to_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_to_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_toggle".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_toggle) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_all_ones".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_all_ones) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_from_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_from_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_all_zeros".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_all_zeros) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitvec_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitvec_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_either_zero".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_either_zero) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_either_nested".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_either_nested) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_either_is_left".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_either_is_left) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_either_is_right".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_either_is_right) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_either_make_left".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_either_make_left) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_either_dbl_or_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_either_dbl_or_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_either_int_or_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_either_int_or_str) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_either_make_right".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_either_make_right) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_export_control_normal".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_export_control_normal) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_export_control_internal".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_export_control_internal) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_export_control_noexport".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_export_control_noexport) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S3MatchArgPoint__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3MatchArgPoint__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4MatchArgHolder__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4MatchArgHolder__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7MatchArgHolder__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7MatchArgHolder__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7MatchArgHolder__set".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7MatchArgHolder__set) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6MatchArgCounter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6MatchArgCounter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3MatchArgPoint__label".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3MatchArgPoint__label) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_EnvMatchArgCounter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_EnvMatchArgCounter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6MatchArgCounter__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6MatchArgCounter__mode) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_VctrsMatchArgScale__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_VctrsMatchArgScale__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3MatchArgPoint__relabel".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S3MatchArgPoint__relabel) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_EnvMatchArgCounter__count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_EnvMatchArgCounter__count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_EnvMatchArgCounter__reset".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvMatchArgCounter__reset) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6MatchArgCounter__record".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6MatchArgCounter__record) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7MatchArgHolder__current".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7MatchArgHolder__current) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4MatchArgHolder__mode_set".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S4MatchArgHolder__mode_set) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S4MatchArgHolder__mode_current".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4MatchArgHolder__mode_current) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6MatchArgCounter__describe_level".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6MatchArgCounter__describe_level) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4MatchArgHolder__new__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S4MatchArgHolder__new__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S7MatchArgHolder__new__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S7MatchArgHolder__new__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S7MatchArgHolder__set__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S7MatchArgHolder__set__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6MatchArgCounter__new__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_R6MatchArgCounter__new__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_VctrsMatchArgScale__new__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_VctrsMatchArgScale__new__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_EnvMatchArgCounter__new__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_EnvMatchArgCounter__new__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S3MatchArgPoint__relabel__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S3MatchArgPoint__relabel__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6MatchArgCounter__record__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_R6MatchArgCounter__record__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_EnvMatchArgCounter__reset__match_arg_choices__modes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_EnvMatchArgCounter__reset__match_arg_choices__modes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S4MatchArgHolder__mode_set__match_arg_choices__mode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S4MatchArgHolder__mode_set__match_arg_choices__mode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tabled_simple".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tabled_simple) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tabled_styled".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_tabled_styled) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_tabled_aligned".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tabled_aligned) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tabled_compact".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_tabled_compact) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_tabled_from_vecs".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_tabled_from_vecs) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_tabled_empty_rows".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tabled_empty_rows) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tabled_single_cell".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tabled_single_cell) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tabled_builder_demo".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_tabled_builder_demo) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_tabled_many_columns".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_tabled_many_columns) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_tabled_struct_table".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tabled_struct_table) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tabled_special_chars".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tabled_special_chars) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tabled_with_max_width".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_tabled_with_max_width) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_tabled_concat_horizontal".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_tabled_concat_horizontal) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_unwind_protect_normal".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_unwind_protect_normal) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_unwind_protect_r_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_unwind_protect_r_error) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_unwind_protect_lowlevel_test".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn() -> SEXP, _>(C_unwind_protect_lowlevel_test) }),
        numArgs: 0,
    },
    R_CallMethodDef {
        name: c"C_new_derived_temp".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_new_derived_temp) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_new_derived_point".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_new_derived_point) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_DerivedCurrency__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_DerivedCurrency__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_new_derived_percent".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_new_derived_percent) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_new_derived_rational".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_new_derived_rational) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_new_derived_int_lists".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_new_derived_int_lists) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_derived_percent_class_info".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_derived_percent_class_info) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_derived_rational_class_info".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_derived_rational_class_info) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_DerivedCurrency__format_amounts".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_DerivedCurrency__format_amounts) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_call_attr_with".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_call_attr_with) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_call_attr_without".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_call_attr_without) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_create_large_par_events".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_create_large_par_events) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_create_large_par_points".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_create_large_par_points) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_decimal_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_decimal_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_decimal_mul".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_decimal_mul) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_decimal_round".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_decimal_round) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_decimal_scale".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_decimal_scale) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_decimal_is_zero".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_decimal_is_zero) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_decimal_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_decimal_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_display_ip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_display_ip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_fromstr_ip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromstr_ip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_fromstr_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromstr_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_display_bool".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_display_bool) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_display_number".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_display_number) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_display_vec_ips".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_display_vec_ips) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_fromstr_vec_ips".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromstr_vec_ips) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_display_vec_ints".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_display_vec_ints) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_fromstr_vec_ints".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromstr_vec_ints) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_fromstr_bad_input".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_fromstr_bad_input) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_TypeA__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_TypeA__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_TypeB__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_TypeB__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_TypeA__get_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_TypeA__get_val) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_TypeB__get_val".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_TypeB__get_val) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_extptr_any_erased_is".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_extptr_any_erased_is) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_extptr_any_into_inner".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_extptr_any_into_inner) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_extptr_any_wrong_type_is".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_extptr_any_wrong_type_is) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_extptr_any_erased_downcast".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_extptr_any_erased_downcast) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_telemetry_get_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_telemetry_get_count) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_telemetry_clear_hook".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_telemetry_clear_hook) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_telemetry_install_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_telemetry_install_counter) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tinyvec_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_tinyvec_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_tinyvec_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tinyvec_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_arrayvec_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_arrayvec_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tinyvec_at_capacity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tinyvec_at_capacity) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tinyvec_over_capacity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_tinyvec_over_capacity) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_tinyvec_roundtrip_dbl".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_tinyvec_roundtrip_dbl) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_tinyvec_roundtrip_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_tinyvec_roundtrip_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrayvec_roundtrip_dbl".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrayvec_roundtrip_dbl) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_arrayvec_roundtrip_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_arrayvec_roundtrip_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_warn_on_elt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_altrep_warn_on_elt) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_altrep_panic_on_elt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_altrep_panic_on_elt) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_altrep_message_on_elt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_altrep_message_on_elt) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_altrep_panic_at_index".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_altrep_panic_at_index) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_altrep_condition_on_elt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_altrep_condition_on_elt) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_altrep_classed_error_on_elt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_altrep_classed_error_on_elt) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_bitflags_all".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_bitflags_all) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bitflags_xor".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_bitflags_xor) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_bitflags_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_bitflags_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bitflags_names".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_names) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_union".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_bitflags_union) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_bitflags_display".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_display) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_has_read".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_has_read) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_has_write".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_has_write) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_intersect".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_bitflags_intersect) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_bitflags_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_complement".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_complement) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_from_strict".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_from_strict) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_has_execute".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_has_execute) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bitflags_from_truncate".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bitflags_from_truncate) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_drop".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_drop) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_nested".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_nested) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_rename".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_rename) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_select".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_select) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_empty_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_empty_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_rename_noop".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_rename_noop) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_tagged_enum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_tagged_enum) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_deep_nesting".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_deep_nesting) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_strip_prefix".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_strip_prefix) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_serde_flatten".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_serde_flatten) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_untagged_enum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_untagged_enum) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_optional_struct".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_optional_struct) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_ext_tagged_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_ext_tagged_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_int_tagged_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_int_tagged_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_with_column_append".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_with_column_append) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_skip_serializing_if".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_skip_serializing_if) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_with_column_replace".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_with_column_replace) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_single_variant_split".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_single_variant_split) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_PtrSelfTest__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PtrSelfTest__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PtrSelfTest__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PtrSelfTest__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PtrSelfTest__is_null_ptr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PtrSelfTest__is_null_ptr) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PtrSelfTest__value_via_ptr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PtrSelfTest__value_via_ptr) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PtrSelfTest__value_owned_ptr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PtrSelfTest__value_owned_ptr) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PtrSelfTest__set_value_via_ptr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_PtrSelfTest__set_value_via_ptr) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_indexmap_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_indexmap_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_indexmap_keys".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_indexmap_keys) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_indexmap_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_indexmap_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_indexmap_single".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_indexmap_single) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_indexmap_duplicate_key".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_indexmap_duplicate_key) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_indexmap_roundtrip_dbl".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_indexmap_roundtrip_dbl) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_indexmap_roundtrip_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_indexmap_roundtrip_int) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_indexmap_roundtrip_str".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_indexmap_roundtrip_str) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_indexmap_order_preserved".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_indexmap_order_preserved) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_solve".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_nalgebra_solve) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_from_fn".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_nalgebra_from_fn) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_inverse".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_inverse) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_reshape".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_nalgebra_reshape) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_determinant".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_determinant) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dvector_dot".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_nalgebra_dvector_dot) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dvector_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dvector_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dvector_sum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dvector_sum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_eigenvalues".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_eigenvalues) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dvector_norm".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dvector_norm) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dmatrix_ncols".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dmatrix_ncols) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dmatrix_nrows".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dmatrix_nrows) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dmatrix_trace".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dmatrix_trace) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_from_row_slice".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_nalgebra_from_row_slice) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dmatrix_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dmatrix_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dmatrix_transpose".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dmatrix_transpose) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_dvector_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_nalgebra_dvector_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_nalgebra_svector3_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_nalgebra_svector3_roundtrip) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_refcount_arena_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_refcount_arena_roundtrip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_streaming_int_range".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_streaming_int_range) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_streaming_real_squares".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_streaming_real_squares) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6SensorReading__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6SensorReading__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6SensorReading__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6SensorReading__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6SensorReading__raw_bytes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6SensorReading__raw_bytes) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_float_ceil".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_float_ceil) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_float_powi".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_float_powi) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_float_sqrt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_float_sqrt) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_num_is_one".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_num_is_one) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_signed_abs".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_signed_abs) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_float_floor".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_float_floor) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_num_is_zero".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_num_is_zero) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_float_is_nan".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_float_is_nan) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_signed_signum".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_signed_signum) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_float_is_finite".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_float_is_finite) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_signed_is_negative".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_signed_is_negative) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_signed_is_positive".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_signed_is_positive) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_is_null".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_is_null) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_is_array".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_is_array) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_array_len".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_array_len) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_is_number".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_is_number) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_is_object".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_is_object) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_is_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_is_string) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_to_pretty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_to_pretty) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_type_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_type_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_object_keys".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_json_object_keys) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_json_from_key_values".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_json_from_key_values) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_json_serialize_point".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_json_serialize_point) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_arith_seq".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_arith_seq) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_boxed_raw".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_boxed_raw) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_complex_im".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_complex_im) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_complex_re".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_complex_re) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_complex_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_complex_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_complex_conj".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_complex_conj) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_complex_norm".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_complex_norm) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_complex_is_finite".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_complex_is_finite) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_complex_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_complex_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_complex_from_parts".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_complex_from_parts) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_complex_roundtrip_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_complex_roundtrip_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_aho_test_count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_aho_test_count) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_aho_test_replace".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_aho_test_replace) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_aho_test_unicode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_aho_test_unicode) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_aho_test_is_match".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_aho_test_is_match) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_aho_test_no_match".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_aho_test_no_match) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_aho_test_find_flat".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_aho_test_find_flat) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_aho_test_overlapping".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_aho_test_overlapping) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_aho_test_replace_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_aho_test_replace_empty) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_boxed_ints".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_boxed_ints) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_enum_all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_enum_all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_u64_mixed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_u64_mixed) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_flatten_all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_flatten_all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_string_mixed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_string_mixed) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_bytes_with_values".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_bytes_with_values) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_bool_all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_bool_all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_bytes_and_opt_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_bytes_and_opt_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_bytes_all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_bytes_all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_string_all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_string_all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_enum_some_flips_type".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_enum_some_flips_type) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_hashmap_all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_hashmap_all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_schema_upgrade_nested".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_schema_upgrade_nested) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_schema_upgrade_scalar".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_schema_upgrade_scalar) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_u64_all_none_multi".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_u64_all_none_multi) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_u64_all_none_single".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_u64_all_none_single) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_opt_user_struct_all_none".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_opt_user_struct_all_none) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_compound_different_shapes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_compound_different_shapes) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_test_columnar_schema_upgrade_multi_none_first".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_test_columnar_schema_upgrade_multi_none_first) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ptr_identity".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ptr_identity) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ptr_pick_larger".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_ptr_pick_larger) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_PtrIdentityTest__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PtrIdentityTest__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_PtrIdentityTest__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_PtrIdentityTest__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_native_sexp_altrep_new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_native_sexp_altrep_new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_OptsTarget__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_OptsTarget__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_OptsTarget__OptionsDemo__with_exit".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_OptsTarget__OptionsDemo__with_exit) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_OptsTarget__OptionsDemo__with_entry".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_OptsTarget__OptionsDemo__with_entry) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_OptsTarget__OptionsDemo__basic_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_OptsTarget__OptionsDemo__basic_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_OptsTarget__OptionsDemo__with_checks".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_OptsTarget__OptionsDemo__with_checks) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_OptsTarget__OptionsDemo__deprecated_method".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_OptsTarget__OptionsDemo__deprecated_method) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_boxed_reals".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_boxed_reals) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_lazy_string".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_lazy_string) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_leaked_ints".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_leaked_ints) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_static_ints".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_static_ints) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_unit_circle".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_unit_circle) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_inf".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ordered_float_inf) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_sort".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ordered_float_sort) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_is_nan".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ordered_float_is_nan) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_neg_inf".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ordered_float_neg_inf) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_neg_zero".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_ordered_float_neg_zero) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_is_finite".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ordered_float_is_finite) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ordered_float_roundtrip) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_sort_special".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ordered_float_sort_special) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_ordered_float_roundtrip_vec".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_ordered_float_roundtrip_vec) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_constant_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_constant_int) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_lazy_int_seq".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_lazy_int_seq) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_lazy_squares".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_lazy_squares) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__id".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Raiser__id) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__id".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4Raiser__id) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Raiser__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3Raiser__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4Raiser__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Raiser__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_id".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_id) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_id".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_id) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_id".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_id) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__raise_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Raiser__raise_error) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__raise_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Raiser__raise_error) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_condition_error_empty".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_condition_error_empty) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__raise_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Raiser__raise_message) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__raise_warning".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Raiser__raise_warning) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__raise_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Raiser__raise_message) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__raise_warning".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Raiser__raise_warning) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_raise_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_raise_error) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_raise_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_raise_error) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_condition_error_unicode".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_condition_error_unicode) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__raise_condition".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Raiser__raise_condition) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__raise_condition".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Raiser__raise_condition) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_raise_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_raise_error) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_raise_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_raise_message) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_raise_warning".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_raise_warning) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_raise_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_raise_message) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_raise_warning".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_raise_warning) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_raise_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_raise_message) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_raise_warning".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_raise_warning) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_raise_condition".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_raise_condition) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_raise_condition".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_raise_condition) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__raise_error_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Raiser__raise_error_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__raise_error_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Raiser__raise_error_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_condition_error_long_message".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_condition_error_long_message) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_raise_condition".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_raise_condition) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__raise_warning_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Raiser__raise_warning_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__raise_warning_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Raiser__raise_warning_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_raise_error_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_raise_error_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_raise_error_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_raise_error_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_R6Raiser__raise_condition_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Raiser__raise_condition_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S4Raiser__raise_condition_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Raiser__raise_condition_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_condition_panic_with_int_payload".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_condition_panic_with_int_payload) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_raise_error_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_raise_error_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_raise_warning_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_raise_warning_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_raise_warning_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_raise_warning_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_raise_warning_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_raise_warning_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S3Raiser__s3_raise_condition_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Raiser__s3_raise_condition_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S7Raiser__s7_raise_condition_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Raiser__s7_raise_condition_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_EnvRaiser__env_raise_condition_classed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_EnvRaiser__env_raise_condition_classed) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_boxed_complex".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_boxed_complex) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_boxed_strings".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_boxed_strings) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_constant_real".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_constant_real) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_repeating_raw".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_repeating_raw) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_array_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_array_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_array_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_array_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_array_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_array_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_array_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_array_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_array_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_array_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_align_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_align_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_align_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_align_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_align_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_align_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashmap_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashmap_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashset_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashset_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashset_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashset_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashset_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashset_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashset_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashset_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_hashset_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_hashset_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_align_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_align_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_align_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_align_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_align_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_align_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreemap_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreemap_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreeset_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreeset_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreeset_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreeset_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreeset_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreeset_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreeset_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreeset_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_btreeset_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_btreeset_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_singleton_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_singleton_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_singleton_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_singleton_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_width_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_width_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_width_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_width_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_width_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_width_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_width_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_width_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_width_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_width_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_expand_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_expand_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_expand_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_expand_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_expand_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_expand_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_expand_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_expand_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_expand_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_expand_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_opaque_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_opaque_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_opaque_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_opaque_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_opaque_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_opaque_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_opaque_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_opaque_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_opaque_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_vec_opaque_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_boxed_slice_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_boxed_slice_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_boxed_slice_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_boxed_slice_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_boxed_slice_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_boxed_slice_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_boxed_slice_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_boxed_slice_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_boxed_slice_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_boxed_slice_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_align_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_align_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_align_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_align_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_align_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_align_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_list_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_list_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_list_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_list_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_list_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_list_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_list_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_list_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_list_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_list_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_list_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_list_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_str_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_str_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_str_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_str_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_str_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_str_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_str_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_str_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_str_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_str_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_align_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_align_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_align_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_align_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_align_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_align_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_factor_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_factor_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_slice_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_slice_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_slice_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_slice_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_slice_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_slice_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_slice_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_slice_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_borrowed_slice_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_borrowed_slice_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_align_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_align_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_align_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_align_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_align_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_align_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_nested_flatten_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_nested_flatten_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_flatten_align_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_flatten_align_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_flatten_split_1v1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_flatten_split_1v1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_flatten_split_1vnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_flatten_split_1vnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_flatten_split_nv1r".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_flatten_split_nv1r) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_struct_flatten_split_nvnr".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_struct_flatten_split_nvnr) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_basic_par".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_basic_par) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_basic_1row".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_basic_1row) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_basic_nrow".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_basic_nrow) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_nested_par".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_nested_par) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_skip_inner".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_skip_inner) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_as_list_par".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_as_list_par) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_mixed_order".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_mixed_order) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_tuple_struct".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_tuple_struct) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_as_list_inner".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_as_list_inner) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_nested_struct".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_nested_struct) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_renamed_inner".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_renamed_inner) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_qual_located_basic".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_qual_located_basic) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_basic_zero_rows".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_basic_zero_rows) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_mixed_inner_types".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_mixed_inner_types) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_two_struct_fields".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_two_struct_fields) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_struct_flatten".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_struct_flatten) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_flat_two_struct_fields_par".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_flat_two_struct_fields_par) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_gc_stress_struct_flatten_nested".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_gc_stress_struct_flatten_nested) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_bench_vec_copy".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bench_vec_copy) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_boxed_logicals".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_boxed_logicals) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_iter_int_range".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_iter_int_range) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_iter_raw_bytes".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_iter_raw_bytes) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_small_vec_copy".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_small_vec_copy) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_static_strings".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_static_strings) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_vec_int_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_vec_int_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_from_raw".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_from_raw) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_sparse_iter_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_sparse_iter_int) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_sparse_iter_raw".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_sparse_iter_raw) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_vec_real_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_vec_real_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_from_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_from_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_bench_vec_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_bench_vec_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_constant_logical".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_constant_logical) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_large_vec_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_large_vec_altrep) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_range_i64_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_range_i64_altrep) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_range_int_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_range_int_altrep) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_sparse_iter_real".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_sparse_iter_real) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_standalone_dataframe_roundtrip".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_standalone_dataframe_roundtrip) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_boxed_data_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_boxed_data_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_iter_int_from_u16".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_iter_int_from_u16) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_iter_real_squares".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_iter_real_squares) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_iter_string_items".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_iter_string_items) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_range_real_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_range_real_altrep) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_altrep_compact_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_altrep_compact_int) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_iter_real_from_f32".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_iter_real_from_f32) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_vec_complex_altrep".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_vec_complex_altrep) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_from_doubles".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_from_doubles) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_from_strings".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_from_strings) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_sparse_iter_logical".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_sparse_iter_logical) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_from_integers".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_from_integers) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_altrep_from_logicals".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_altrep_from_logicals) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_integer_sequence_list".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_integer_sequence_list) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rpkg_enabled_features".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rpkg_enabled_features) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_sparse_iter_int_squares".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_sparse_iter_int_squares) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_iter_logical_alternating".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_iter_logical_alternating) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_lazy_int_seq_is_materialized".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_lazy_int_seq_is_materialized) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6Dog__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Dog__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Dog__breed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Dog__breed) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Dog__fetch".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Dog__fetch) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Animal__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Animal__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Animal__name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Animal__name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Counter__add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Counter__add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Counter__inc".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Counter__inc) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Counter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Counter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Animal__speak".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Animal__speak) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Cloneable__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Cloneable__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Rectangle__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Rectangle__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Rectangle__area".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Rectangle__area) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Accumulator__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_R6Accumulator__new) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6NonPortable__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6NonPortable__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Temperature__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Temperature__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_r6_standalone_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_r6_standalone_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Accumulator__count".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Accumulator__count) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Accumulator__total".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Accumulator__total) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Accumulator__average".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Accumulator__average) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Cloneable__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Cloneable__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Cloneable__set_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Cloneable__set_value) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6GoldenRetriever__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6GoldenRetriever__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Rectangle__get_width".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Rectangle__get_width) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Rectangle__perimeter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Rectangle__perimeter) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Temperature__celsius".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Temperature__celsius) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Rectangle__get_height".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Rectangle__get_height) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6GoldenRetriever__owner".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6GoldenRetriever__owner) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6NonPortable__get_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6NonPortable__get_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Accumulator__accumulate".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Accumulator__accumulate) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Temperature__fahrenheit".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_R6Temperature__fahrenheit) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_R6Counter__default_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_R6Counter__default_counter) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_R6Temperature__set_celsius".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Temperature__set_celsius) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_R6Temperature__set_fahrenheit".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_R6Temperature__set_fahrenheit) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S3Counter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3Counter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3Counter__s3_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S3Counter__s3_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S3Counter__s3_inc".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3Counter__s3_inc) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3Counter__s3_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S3Counter__s3_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S3Counter__default_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S3Counter__default_counter) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S4Counter__add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S4Counter__add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S4Counter__inc".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4Counter__inc) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4Counter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4Counter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4Counter__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S4Counter__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S4Counter__default_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S4Counter__default_counter) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S7Dog__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Dog__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Dog__bark".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Dog__bark) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Range__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Range__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Shape__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Shape__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Animal__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Animal__new) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Circle__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Circle__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Config__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Config__new) }),
        numArgs: 4,
    },
    R_CallMethodDef {
        name: c"C_S7Strict__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Strict__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Animal__legs".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Animal__legs) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Celsius__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Celsius__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Config__name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Config__name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Counter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Counter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Config__score".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Config__score) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Range__length".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Range__length) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Range__s7_end".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Range__s7_end) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Celsius__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Celsius__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Dog__dog_breed".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Dog__dog_breed) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7PropInner__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7PropInner__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7PropOuter__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7PropOuter__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Counter__s7_add".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Counter__s7_add) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Counter__s7_inc".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Counter__s7_inc) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Fahrenheit__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Fahrenheit__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Range__s7_start".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Range__s7_start) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7PropInner__label".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7PropInner__label) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Config__set_score".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Config__set_score) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7Counter__s7_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Counter__s7_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Fahrenheit__value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Fahrenheit__value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Shape__shape_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Shape__shape_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7OverrideShape__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7OverrideShape__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Animal__animal_kind".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Animal__animal_kind) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Circle__circle_area".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Circle__circle_area) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Config__get_version".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Config__get_version) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Config__old_version".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Config__old_version) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7OverrideCircle__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7OverrideCircle__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Range__get_midpoint".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Range__get_midpoint) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Range__set_midpoint".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_S7Range__set_midpoint) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_S7GoldenRetriever__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7GoldenRetriever__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Strict__describe_any".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Strict__describe_any) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Strict__strict_length".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Strict__strict_length) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Fahrenheit__to_celsius".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Fahrenheit__to_celsius) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7GoldenRetriever__color".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7GoldenRetriever__color) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7PropOuter__inner_value".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7PropOuter__inner_value) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7Counter__default_counter".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_S7Counter__default_counter) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_S7Fahrenheit__from_celsius".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7Fahrenheit__from_celsius) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7OverrideShape__shape_kind".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7OverrideShape__shape_kind) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7GoldenRetriever__retriever_name".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7GoldenRetriever__retriever_name) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_S7OverrideCircle__override_radius".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_S7OverrideCircle__override_radius) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_log_info".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_log_info) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_log_warn".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_log_warn) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_log_debug".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_log_debug) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_log_error".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_log_error) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_test_log_set_level".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_test_log_set_level) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rng_int".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rng_int) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_rng_bool".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP) -> SEXP, _>(C_rng_bool) }),
        numArgs: 1,
    },
    R_CallMethodDef {
        name: c"C_rng_range".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rng_range) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_rng_normal".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rng_normal) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rng_uniform".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rng_uniform) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_RngSampler__new".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_RngSampler__new) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rng_guard_test".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rng_guard_test) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rng_exponential".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rng_exponential) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rng_chi_sq_approx".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_rng_chi_sq_approx) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_rng_with_rng_test".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rng_with_rng_test) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rng_with_interrupt".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rng_with_interrupt) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_rng_worker_uniform".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_rng_worker_uniform) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_RngSampler__seed_hint".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_RngSampler__seed_hint) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_RngSampler__sample_normal".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_RngSampler__sample_normal) }),
        numArgs: 3,
    },
    R_CallMethodDef {
        name: c"C_RngSampler__static_sample".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP, _>(C_RngSampler__static_sample) }),
        numArgs: 2,
    },
    R_CallMethodDef {
        name: c"C_RngSampler__sample_uniform".as_ptr(),
        fun: Some(unsafe { ::core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(C_RngSampler__sample_uniform) }),
        numArgs: 3,
    },
];

pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &[
    AltrepRegistration {
        register: __mx_altrep_reg_MxDerivedIntsData,
        symbol: "__mx_altrep_reg_MxDerivedIntsData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_JiffTimestampVecCounted,
        symbol: "__mx_altrep_reg_JiffTimestampVecCounted",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_WarnAltrepData,
        symbol: "__mx_altrep_reg_WarnAltrepData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_MessageAltrepData,
        symbol: "__mx_altrep_reg_MessageAltrepData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_ConditionAltrepData,
        symbol: "__mx_altrep_reg_ConditionAltrepData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_PanickingAltrepData,
        symbol: "__mx_altrep_reg_PanickingAltrepData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_LoopStressAltrepData,
        symbol: "__mx_altrep_reg_LoopStressAltrepData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_ClassedErrorAltrepData,
        symbol: "__mx_altrep_reg_ClassedErrorAltrepData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_StreamingIntRangeData,
        symbol: "__mx_altrep_reg_StreamingIntRangeData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_StreamingRealSquaresData,
        symbol: "__mx_altrep_reg_StreamingRealSquaresData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_ListData,
        symbol: "__mx_altrep_reg_ListData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_ArithSeqData,
        symbol: "__mx_altrep_reg_ArithSeqData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_BoxedIntsData,
        symbol: "__mx_altrep_reg_BoxedIntsData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_StringVecData,
        symbol: "__mx_altrep_reg_StringVecData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_LazyIntSeqData,
        symbol: "__mx_altrep_reg_LazyIntSeqData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_LazyStringData,
        symbol: "__mx_altrep_reg_LazyStringData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_LogicalVecData,
        symbol: "__mx_altrep_reg_LogicalVecData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_StaticIntsData,
        symbol: "__mx_altrep_reg_StaticIntsData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_UnitCircleData,
        symbol: "__mx_altrep_reg_UnitCircleData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_ConstantIntData,
        symbol: "__mx_altrep_reg_ConstantIntData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_ConstantRealData,
        symbol: "__mx_altrep_reg_ConstantRealData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_RepeatingRawData,
        symbol: "__mx_altrep_reg_RepeatingRawData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_SimpleVecIntData,
        symbol: "__mx_altrep_reg_SimpleVecIntData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_SimpleVecRawData,
        symbol: "__mx_altrep_reg_SimpleVecRawData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_SparseIntIterData,
        symbol: "__mx_altrep_reg_SparseIntIterData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_SparseRawIterData,
        symbol: "__mx_altrep_reg_SparseRawIterData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_StaticStringsData,
        symbol: "__mx_altrep_reg_StaticStringsData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_SparseRealIterData,
        symbol: "__mx_altrep_reg_SparseRealIterData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_ConstantLogicalData,
        symbol: "__mx_altrep_reg_ConstantLogicalData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_InferredVecRealData,
        symbol: "__mx_altrep_reg_InferredVecRealData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_SparseLogicalIterData,
        symbol: "__mx_altrep_reg_SparseLogicalIterData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_IntegerSequenceListData,
        symbol: "__mx_altrep_reg_IntegerSequenceListData",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Box_u8,
        symbol: "__mx_altrep_reg_builtin_Box_u8",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Cow_u8,
        symbol: "__mx_altrep_reg_builtin_Cow_u8",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_u8,
        symbol: "__mx_altrep_reg_builtin_Vec_u8",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Box_f64,
        symbol: "__mx_altrep_reg_builtin_Box_f64",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Box_i32,
        symbol: "__mx_altrep_reg_builtin_Box_i32",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Cow_f64,
        symbol: "__mx_altrep_reg_builtin_Cow_f64",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Cow_i32,
        symbol: "__mx_altrep_reg_builtin_Cow_i32",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_f64,
        symbol: "__mx_altrep_reg_builtin_Vec_f64",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_i32,
        symbol: "__mx_altrep_reg_builtin_Vec_i32",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Box_bool,
        symbol: "__mx_altrep_reg_builtin_Box_bool",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_bool,
        symbol: "__mx_altrep_reg_builtin_Vec_bool",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Range_f64,
        symbol: "__mx_altrep_reg_builtin_Range_f64",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Range_i32,
        symbol: "__mx_altrep_reg_builtin_Range_i32",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Range_i64,
        symbol: "__mx_altrep_reg_builtin_Range_i64",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Box_String,
        symbol: "__mx_altrep_reg_builtin_Box_String",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_String,
        symbol: "__mx_altrep_reg_builtin_Vec_String",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_Cow_str,
        symbol: "__mx_altrep_reg_builtin_Vec_Cow_str",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Box_Rcomplex,
        symbol: "__mx_altrep_reg_builtin_Box_Rcomplex",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Cow_Rcomplex,
        symbol: "__mx_altrep_reg_builtin_Cow_Rcomplex",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_Rcomplex,
        symbol: "__mx_altrep_reg_builtin_Vec_Rcomplex",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_Option_String,
        symbol: "__mx_altrep_reg_builtin_Vec_Option_String",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_Vec_Option_Cow_str,
        symbol: "__mx_altrep_reg_builtin_Vec_Option_Cow_str",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_Int32Array,
        symbol: "__mx_altrep_reg_builtin_arrow_Int32Array",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_UInt8Array,
        symbol: "__mx_altrep_reg_builtin_arrow_UInt8Array",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_StringArray,
        symbol: "__mx_altrep_reg_builtin_arrow_StringArray",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_BooleanArray,
        symbol: "__mx_altrep_reg_builtin_arrow_BooleanArray",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_Float64Array,
        symbol: "__mx_altrep_reg_builtin_arrow_Float64Array",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_JiffZonedVec,
        symbol: "__mx_altrep_reg_JiffZonedVec",
    },
    AltrepRegistration {
        register: __mx_altrep_reg_JiffTimestampVec,
        symbol: "__mx_altrep_reg_JiffTimestampVec",
    },
];

pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x490c001304960a63, 0x8f8efb39ffc2b5a4),
        trait_tag: mx_tag::new(0x9d1c372fe94004c3, 0x68c30b6166ebf514),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_COUNTER_FOR_SIMPLECOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_COUNTER_FOR_SIMPLECOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x2eec703cac96bf06, 0x9c2698b0e58b43d3),
        trait_tag: mx_tag::new(0x9d1c372fe94004c3, 0x68c30b6166ebf514),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_COUNTER_FOR_PANICKYCOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_COUNTER_FOR_PANICKYCOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x75033f88f21ec60f, 0xbfc2af468b51f4ce),
        trait_tag: mx_tag::new(0x9d1c372fe94004c3, 0x68c30b6166ebf514),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_COUNTER_FOR_R6TRAITCOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_COUNTER_FOR_R6TRAITCOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xc300c9adc4b3553d, 0xc0ee64f588db8de8),
        trait_tag: mx_tag::new(0x9d1c372fe94004c3, 0x68c30b6166ebf514),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_COUNTER_FOR_S3TRAITCOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_COUNTER_FOR_S3TRAITCOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x4697878f57ac6058, 0xfba409c341faae4d),
        trait_tag: mx_tag::new(0x9d1c372fe94004c3, 0x68c30b6166ebf514),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_COUNTER_FOR_S4TRAITCOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_COUNTER_FOR_S4TRAITCOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x75d19c42881ae1f9, 0x0e094336f01dfe6c),
        trait_tag: mx_tag::new(0x9d1c372fe94004c3, 0x68c30b6166ebf514),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_COUNTER_FOR_S7TRAITCOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_COUNTER_FOR_S7TRAITCOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x89f47d45d2e51bd0, 0xc547f68a3562e01b),
        trait_tag: mx_tag::new(0x473a4aecd6e9ba60, 0xd215e4dd02b3c5b1),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_FALLIBLE_FOR_FALLIBLEIMPL).cast::<c_void>() },
        vtable_symbol: "__VTABLE_FALLIBLE_FOR_FALLIBLEIMPL",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xa05cc95a42171c66, 0x5af3154814c4e289),
        trait_tag: mx_tag::new(0x9ac55f373927abe8, 0xe0d982b21e0e317b),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_SHAREDCOUNTER_FOR_ATOMICCOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_SHAREDCOUNTER_FOR_ATOMICCOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xfff6226d2e08381c, 0x06fcf53fb50efa6f),
        trait_tag: mx_tag::new(0x9ac55f373927abe8, 0xe0d982b21e0e317b),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_SHAREDCOUNTER_FOR_SHAREDSIMPLECOUNTER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_SHAREDCOUNTER_FOR_SHAREDSIMPLECOUNTER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x22e453184e296444, 0xb52f0a10b14ee24d),
        trait_tag: mx_tag::new(0x10fdef5a08073710, 0x3c59297cbf422407),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITR6).cast::<c_void>() },
        vtable_symbol: "__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITR6",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x22e7d8184e2c7c1a, 0xb52b7d10b14bbcdf),
        trait_tag: mx_tag::new(0x10fdef5a08073710, 0x3c59297cbf422407),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS3).cast::<c_void>() },
        vtable_symbol: "__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS3",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x22e7d3184e2c739b, 0xb52b7a10b14bb7c6),
        trait_tag: mx_tag::new(0x10fdef5a08073710, 0x3c59297cbf422407),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS4).cast::<c_void>() },
        vtable_symbol: "__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS4",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x22e7d4184e2c754e, 0xb52b7910b14bb613),
        trait_tag: mx_tag::new(0x10fdef5a08073710, 0x3c59297cbf422407),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS7).cast::<c_void>() },
        vtable_symbol: "__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITS7",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xbbde194c67f237cb, 0xaa9bdc5cfe5d4810),
        trait_tag: mx_tag::new(0xf790fc82d0c30db6, 0xde5d4c3ae47995fb),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_STATICXPARAM_FOR_COUNTERTRAITENV).cast::<c_void>() },
        vtable_symbol: "__VTABLE_STATICXPARAM_FOR_COUNTERTRAITENV",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xbbde194c67f237cb, 0xaa9bdc5cfe5d4810),
        trait_tag: mx_tag::new(0x10fdef5a08073710, 0x3c59297cbf422407),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITENV).cast::<c_void>() },
        vtable_symbol: "__VTABLE_MATRIXCOUNTER_FOR_COUNTERTRAITENV",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0xe0e1282c8d577bb0, 0xcd05450514f90a0d),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RORD_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RORD_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0x3273c1953966df54, 0xb417767f828c9cdb),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RCOPY_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RCOPY_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0xab3c7ccbabfc40e1, 0x09bb47cc13ac3d6a),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RHASH_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RHASH_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0xc9e59e96a01eccb2, 0xb827dbc2de9e7823),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RCLONE_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RCLONE_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0x319dbb787099fbe8, 0xb216fd90fb359581),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RDEBUG_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RDEBUG_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0xbad8e6388dbc644e, 0xd7cfad39d8cb7aab),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RDEFAULT_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RDEFAULT_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0xb7ae47a3533c5a85, 0xbebd8fd2714931b4),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RDISPLAY_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RDISPLAY_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x70f07e51546cec87, 0xfb5d55441eba7fd6),
        trait_tag: mx_tag::new(0x9529c227a8d6a130, 0x55f67e5c7a75bea5),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RFROMSTR_FOR_POINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RFROMSTR_FOR_POINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xf28cbab094478cd0, 0x4ee4bef802a03997),
        trait_tag: mx_tag::new(0x87d9f4d6992c04ae, 0x9719deaa90842d2f),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RTOVEC_FOR_INTSET).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RTOVEC_FOR_INTSET",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xf28cbab094478cd0, 0x4ee4bef802a03997),
        trait_tag: mx_tag::new(0xc913a11060b19e47, 0x5d0ff7eeff334934),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RFROMITER_FOR_INTSET).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RFROMITER_FOR_INTSET",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x980c795ce4fd58bd, 0x80df3bd3e08a2258),
        trait_tag: mx_tag::new(0xc43b34517bc16689, 0x0117d19737842f76),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RPARTIALORD_FOR_MYFLOAT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RPARTIALORD_FOR_MYFLOAT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x66062cbe6739c8de, 0x6bde622bd6b2b003),
        trait_tag: mx_tag::new(0xb09038f5eac7dc39, 0x4408694cc07dd3aa),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_REXTEND_FOR_GROWABLEVEC).cast::<c_void>() },
        vtable_symbol: "__VTABLE_REXTEND_FOR_GROWABLEVEC",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x81ddc4926e449e84, 0xdec52a37557ca2bb),
        trait_tag: mx_tag::new(0x060da55bea4ac81f, 0x9c1e0f64680d6a3c),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RITERATOR_FOR_INTVECITER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RITERATOR_FOR_INTVECITER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x7dad2969f08a4261, 0x22ffc44cd7c1f626),
        trait_tag: mx_tag::new(0x10debc956bee9da1, 0xf8d45d9da7990130),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RERROR_FOR_CHAINEDERROR).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RERROR_FOR_CHAINEDERROR",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x79051d3dcdb6d955, 0x3f1e2aaad4245b18),
        trait_tag: mx_tag::new(0x69e3e233e9e043f5, 0xf3e7fafdca94e80a),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RMAKEITER_FOR_ITERABLEVEC).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RMAKEITER_FOR_ITERABLEVEC",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x6201dfbf9c181057, 0x6c949749f821a606),
        trait_tag: mx_tag::new(0x060da55bea4ac81f, 0x9c1e0f64680d6a3c),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RITERATOR_FOR_ITERABLEVECITER).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RITERATOR_FOR_ITERABLEVECITER",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x2f08a2e83404cf5c, 0xb12da404e12a0075),
        trait_tag: mx_tag::new(0x319dbb787099fbe8, 0xb216fd90fb359581),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RDEBUG_FOR_EXPORTCONTROLTRAITPOINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RDEBUG_FOR_EXPORTCONTROLTRAITPOINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0x2f08a2e83404cf5c, 0xb12da404e12a0075),
        trait_tag: mx_tag::new(0xb7ae47a3533c5a85, 0xbebd8fd2714931b4),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_RDISPLAY_FOR_EXPORTCONTROLTRAITPOINT).cast::<c_void>() },
        vtable_symbol: "__VTABLE_RDISPLAY_FOR_EXPORTCONTROLTRAITPOINT",
    },
    TraitDispatchEntry {
        concrete_tag: mx_tag::new(0xfe0be37a4c78070c, 0xe13ad74c42fd73fb),
        trait_tag: mx_tag::new(0x33e634b2c1e407f6, 0xf389e783436b23f1),
        vtable: unsafe { ::core::ptr::from_ref(&__VTABLE_OPTIONSDEMO_FOR_OPTSTARGET).cast::<c_void>() },
        vtable_symbol: "__VTABLE_OPTIONSDEMO_FOR_OPTSTARGET",
    },
];
