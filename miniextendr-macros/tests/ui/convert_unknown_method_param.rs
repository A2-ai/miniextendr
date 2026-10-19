use miniextendr_api::{ExternalPtr, miniextendr};
#[derive(ExternalPtr)]
pub struct Host;
#[miniextendr(env)]
impl Host {
    pub fn new() -> Self { Self }
    #[miniextendr(convert(typo))]
    pub fn read(&self, value: &str) -> String { value.to_owned() }
}
fn main() {}
