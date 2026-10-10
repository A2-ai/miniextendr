#![allow(dead_code)]
use miniextendr_api::altrep_data::{AltIntegerData, AltListData, AltrepLen};

#[derive(miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt = "value")]
struct PrivateInteger { value: i32, len: usize }

#[derive(miniextendr_api::AltrepList)]
#[altrep(manual)]
pub(crate) struct CrateList;
impl AltrepLen for CrateList { fn len(&self) -> usize { 0 } }
impl AltListData for CrateList {
    fn elt(&self, _: usize) -> miniextendr_api::SEXP { unreachable!() }
}

#[derive(miniextendr_api::Altrep)]
struct PrivateRegistration;
impl AltrepLen for PrivateRegistration { fn len(&self) -> usize { 0 } }
impl AltIntegerData for PrivateRegistration { fn elt(&self, _: usize) -> i32 { 0 } }
miniextendr_api::impl_altinteger_from_data!(PrivateRegistration);

fn main() {}
