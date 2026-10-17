use miniextendr_api::miniextendr;

struct Host;
#[miniextendr(env)]
impl Host {
    unsafe fn unchecked(&self, value: i32) -> i32 {
        value
    }
}

fn main() {}
