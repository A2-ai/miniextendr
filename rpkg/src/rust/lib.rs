#[non_exhaustive]
#[repr(transparent)]
#[derive(Debug)]
pub struct SEXPREC(std::ffi::c_void);
pub type SEXP = *mut SEXPREC;

unsafe extern "C" {
    static R_NilValue: SEXP;
}

#[unsafe(no_mangle)]
pub extern "C" fn C_add(_left: SEXP, _right: SEXP) -> SEXP {
    unsafe { R_NilValue }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_unwind_add(_left: SEXP, _right: SEXP) -> SEXP {
    unsafe { R_NilValue }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
