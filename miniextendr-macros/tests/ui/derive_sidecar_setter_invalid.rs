use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::RSidecar;

#[derive(ExternalPtr)]
struct Unknown {
    #[r_data(setter = "quiet")]
    pub value: i32,
}

#[derive(ExternalPtr)]
struct Duplicate {
    #[r_data(setter = "visible")]
    #[r_data(setter = "invisible")]
    pub value: i32,
}

#[derive(ExternalPtr)]
struct Private {
    #[r_data(setter = "visible")]
    value: i32,
}

#[derive(ExternalPtr)]
struct Selector {
    #[r_data(setter = "visible")]
    pub sidecar: RSidecar,
}

fn main() {}
