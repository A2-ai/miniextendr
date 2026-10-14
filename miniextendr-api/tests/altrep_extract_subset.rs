//! A class that declines an optimized subset gets R's default subsetting.
mod r_test_utils;

use miniextendr_api::IntoR;
use miniextendr_api::altrep_traits::AltVec;
use miniextendr_api::gc_protect::OwnedProtect;

#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, subset, class = "DecliningSubset")]
pub struct DecliningSubset(Vec<i32>);

impl miniextendr_api::altrep_data::AltrepLen for DecliningSubset {
    fn altrep_len(&self) -> usize {
        self.0.len()
    }
}
impl miniextendr_api::altrep_data::AltIntegerData for DecliningSubset {
    fn elt(&self, i: usize) -> i32 {
        self.0[i]
    }
}
impl miniextendr_api::altrep_data::AltrepExtractSubset for DecliningSubset {
    fn extract_subset(&self, _: &[i32]) -> Option<miniextendr_api::SEXP> {
        None
    }
}

#[test]
fn declined_subset_uses_r_default_subsetting() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe { OwnedProtect::new(DecliningSubset(vec![7, 11]).into_sexp()) };
        let indices = unsafe { OwnedProtect::new(vec![2i32].into_sexp()) };
        let result = unsafe {
            OwnedProtect::new(
                miniextendr_api::expression::RCall::new("[")
                    .arg(x.get())
                    .arg(indices.get())
                    .eval_base()
                    .expect("R subsetting succeeds"),
            )
        };
        let actual = <Vec<i32> as miniextendr_api::TryFromSexp>::try_from_sexp(result.get())
            .expect("declining the optimized subset still returns an integer vector");
        assert_eq!(actual, vec![11]);
        assert!(
            <DecliningSubset as AltVec>::extract_subset(
                x.get(),
                indices.get(),
                miniextendr_api::SEXP::nil()
            )
            .is_null_ptr()
        );
    });
}
