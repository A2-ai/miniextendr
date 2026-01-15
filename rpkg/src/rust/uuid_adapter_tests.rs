//! UUID adapter tests
use miniextendr_api::uuid_impl::Uuid;
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn uuid_roundtrip(uuid: Uuid) -> Uuid {
    uuid
}

/// @noRd
#[miniextendr]
pub fn uuid_roundtrip_vec(uuids: Vec<Uuid>) -> Vec<Uuid> {
    uuids
}

/// @noRd
#[miniextendr]
pub fn uuid_new_v4() -> Uuid {
    Uuid::new_v4()
}

/// @noRd
#[miniextendr]
pub fn uuid_nil() -> Uuid {
    Uuid::nil()
}

/// @noRd
#[miniextendr]
pub fn uuid_max() -> Uuid {
    Uuid::max()
}

/// @noRd
#[miniextendr]
pub fn uuid_version(uuid: Uuid) -> i32 {
    uuid.get_version_num() as i32
}

/// @noRd
#[miniextendr]
pub fn uuid_is_nil(uuid: Uuid) -> bool {
    uuid.is_nil()
}

miniextendr_module! {
    mod uuid_adapter_tests;
    fn uuid_roundtrip;
    fn uuid_roundtrip_vec;
    fn uuid_new_v4;
    fn uuid_nil;
    fn uuid_max;
    fn uuid_version;
    fn uuid_is_nil;
}
