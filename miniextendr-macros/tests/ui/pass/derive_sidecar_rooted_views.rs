use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::{DataFrame, ExternalPtr, List};

#[derive(ExternalPtr)]
struct RootedViews {
    #[r_data]
    pub list: Sidecar<List>,
    #[r_data]
    pub optional: Sidecar<Option<List>>,
    #[r_data]
    pub frame: Sidecar<DataFrame>,
}

fn main() {}
