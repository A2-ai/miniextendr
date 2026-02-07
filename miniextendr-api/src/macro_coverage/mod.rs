//! Macro expansion coverage module.
//!
//! Instantiates every macro path so `cargo expand` can be used as a living
//! catalog of what gets generated. Feature-gated behind `macro-coverage`.
//!
//! ## Submodules
//!
//! | Module | Coverage |
//! |--------|----------|
//! | `fn_matrix` | Every `MiniextendrFnAttrs` option + combinations |
//! | `impl_matrix` | Class systems: env, r6, s3, s4, s7 |
//! | `trait_abi_matrix` | Trait definition + trait impl ABI |
//! | `derive_matrix` | Every derive entrypoint |
//! | `helper_macro_matrix` | `r_ffi_checked`, `list!`, `typed_list!` |
//! | `nested` | `use submodule;` module variant |

pub mod derive_matrix;
pub mod fn_matrix;
pub mod helper_macro_matrix;
pub mod impl_matrix;
mod nested;
pub mod trait_abi_matrix;

use crate::miniextendr_module;

miniextendr_module! {
    mod macro_coverage;
    use fn_matrix;
    use impl_matrix;
    use trait_abi_matrix;
    use helper_macro_matrix;
    use nested;
}
