// use crate::ffi::Rprintf;
// use std::{
//     cell::RefCell,
//     ffi::{CStr, CString},
// };

// work-in-progress: Use common buffer for the *const char APIs..
// thread_local! {
//     /// Buffer using in `rprintln`/`rprint`/`rerror`
//     pub static R_MESSAGE_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(256));
// }
