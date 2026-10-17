use miniextendr_macros::AllEqual;

#[derive(AllEqual)]
#[all_equal(tolerence = 0)]
struct Unknown;

#[derive(AllEqual)]
struct Skip {
    #[all_equal(skip, label = "ignored")]
    value: i32,
}

#[derive(AllEqual)]
#[all_equal(tolerance = 0, tolerance = 1)]
struct Duplicate;

fn main() {}
