use miniextendr_api::ExternalPtr;
use miniextendr_api::SEXP;
use miniextendr_api::externalptr::RSidecar;

#[derive(ExternalPtr)]
struct Engine {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub keys: SEXP,
}

fn main() {}
