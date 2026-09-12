use miniextendr_api::miniextendr;

#[miniextendr]
pub fn free() -> miniextendr_api::ConvertTo<i32> { unimplemented!() }

pub struct WrongSystem;
#[miniextendr(env)]
impl WrongSystem {
    pub fn convert(&self) -> miniextendr_api::ConvertTo<Other> { unimplemented!() }
}

pub struct StaticTo;
#[miniextendr(s7)]
impl StaticTo {
    pub fn convert() -> miniextendr_api::ConvertTo<Other> { unimplemented!() }
}

pub struct InstanceFrom;
#[miniextendr(s7)]
impl InstanceFrom {
    pub fn convert(&self) -> miniextendr_api::ConvertFrom<Self> { unimplemented!() }
}

pub struct WrongPayload;
#[miniextendr(s7)]
impl WrongPayload {
    pub fn convert(source: &Other) -> miniextendr_api::ConvertFrom<Other> { unimplemented!() }
}

pub struct Conflict;
#[miniextendr(s7)]
impl Conflict {
    #[miniextendr(s7(convert_to = "Different"))]
    pub fn convert(&self) -> miniextendr_api::ConvertTo<Other> { unimplemented!() }
}

pub struct Vector;
#[miniextendr(s7)]
impl Vector {
    pub fn convert(&self) -> Vec<miniextendr_api::ConvertTo<Other>> { unimplemented!() }
}

#[miniextendr]
pub trait TraitConversion {
    fn convert(&self) -> miniextendr_api::ConvertTo<Other>;
}

pub struct Other;
fn main() {}
