//! #2020: consumer blanket traits must not intercept generated conversions.
#![allow(dead_code)]

pub struct ConsumerMethodCalled;

trait ConsumerConversions: Sized {
    fn into_sexp(self) -> ConsumerMethodCalled { ConsumerMethodCalled }
    fn into_sexp_unchecked(self) -> ConsumerMethodCalled { ConsumerMethodCalled }
    fn try_into_sexp(self) -> ConsumerMethodCalled { ConsumerMethodCalled }
    fn try_into_sexp_unchecked(self) -> ConsumerMethodCalled { ConsumerMethodCalled }
    fn into(self) -> ConsumerMethodCalled { ConsumerMethodCalled }
    fn as_ref(&self) -> &ConsumerMethodCalled { &ConsumerMethodCalled }
    fn to_choice(self) -> ConsumerMethodCalled { ConsumerMethodCalled }
    fn elt(&self, _index: usize) -> ConsumerMethodCalled { ConsumerMethodCalled }
}
impl<T> ConsumerConversions for T {}

include!("common/every_macro.rs");

#[derive(miniextendr_api::AltrepReal)]
#[altrep(len = "len", elt_delegate = "data")]
pub struct DelegatedReal {
    len: usize,
    data: Reals,
}

#[derive(miniextendr_api::AltrepLogical)]
#[altrep(len = "len", elt_delegate = "data")]
pub struct DelegatedLogical {
    len: usize,
    data: Logicals,
}

#[derive(miniextendr_api::AltrepRaw)]
#[altrep(len = "len", elt_delegate = "data")]
pub struct DelegatedRaw {
    len: usize,
    data: Raws,
}

#[derive(miniextendr_api::AltrepString)]
#[altrep(len = "len", elt_delegate = "data")]
pub struct DelegatedString {
    len: usize,
    data: Strings,
}

#[derive(miniextendr_api::AltrepComplex)]
#[altrep(len = "len", elt_delegate = "data")]
pub struct DelegatedComplex {
    len: usize,
    data: Complexes,
}

fn main() {}
