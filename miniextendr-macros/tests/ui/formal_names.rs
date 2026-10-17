use miniextendr_api::miniextendr;

#[miniextendr]
fn duplicate(#[miniextendr(formal = "x")] renamed: i32, x: i32) {}

#[miniextendr]
fn reserved(#[miniextendr(formal = ".call")] value: i32) {}

#[miniextendr]
fn syntax(#[miniextendr(formal = "not a name")] value: i32) {}

#[miniextendr]
fn no_call_formal(#[miniextendr(formal = "call.expr")] value: miniextendr_api::Call) {}

#[miniextendr]
fn no_count_formal(#[miniextendr(formal = "arg.count")] value: miniextendr_api::NArgs) {}

fn main() {}
