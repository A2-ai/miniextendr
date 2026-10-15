use miniextendr_api::miniextendr;
type Text = str;
#[miniextendr]
pub fn escape(#[miniextendr(convert)] value: &'static Text) -> &'static str { value }
fn main() {}
