//! Test: RCondition's field errors point at the token at fault: the name a
//! field gives its `data` entry (its `rename`, else the field), the field's
//! whole type, or the whole `#[condition(..)]` attribute, never the `#` or
//! `pub` a field starts with.

use miniextendr_api::condition::RCondition;

#[derive(Debug, RCondition)]
#[condition(message = "reserved")]
pub struct ReservedRename {
    #[condition(rename = "message")]
    pub detail: String,
}

#[derive(Debug, RCondition)]
#[condition(message = "duplicate")]
pub struct Duplicate {
    pub row: i32,
    #[condition(rename = "row")]
    pub other: i32,
}

#[derive(Debug, RCondition)]
#[condition(message = "tuple")]
pub struct Tuple(#[condition(debug)] pub Vec<i32>);

#[derive(Debug, RCondition)]
#[condition(message = "skip_none")]
pub struct SkipNone {
    #[condition(skip_none)]
    pub values: Vec<i32>,
}

#[derive(Debug, RCondition)]
#[condition(message = "skip")]
pub struct SkipRename {
    #[condition(skip, rename = "x")]
    pub value: i32,
}

fn main() {}
