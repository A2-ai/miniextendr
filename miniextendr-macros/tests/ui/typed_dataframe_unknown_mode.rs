use miniextendr_macros::typed_dataframe;

typed_dataframe! {
    @strict;
    pub Df { x: i32 }
}

fn main() {}
