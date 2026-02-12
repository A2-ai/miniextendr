//! Generated R wrapper benchmarks.
//!
//! Measures overhead of calling generated R wrappers vs direct `.Call`,
//! argument coercion costs, and class-system method dispatch.
//!
//! Implemented groups:
//! - `wrapper_call_overhead`: noop and realvec wrapper vs direct .Call
//! - `direct_call_overhead`: direct .Call baseline (noop, realvec, eval_sum)
//! - `argument_coercion`: as.integer/as.double/as.character scalar + vec256
//! - `class_methods`: Env/R6/S3/S4/S7 dispatch + plain function baseline
