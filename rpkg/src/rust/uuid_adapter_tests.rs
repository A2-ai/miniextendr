//! UUID adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::uuid_impl::Uuid;

/// Test UUID roundtrip through R string.
/// @param uuid UUID parsed from R string.
#[miniextendr]
pub fn uuid_roundtrip(uuid: Uuid) -> Uuid {
    uuid
}

/// Test Vec<Uuid> roundtrip through R character vector.
/// @param uuids Character vector of UUIDs from R.
#[miniextendr]
pub fn uuid_roundtrip_vec(uuids: Vec<Uuid>) -> Vec<Uuid> {
    uuids
}

/// Test generating a new random UUID v4.
#[miniextendr]
pub fn uuid_new_v4() -> Uuid {
    Uuid::new_v4()
}

/// Test creating a nil (all-zeros) UUID.
#[miniextendr]
pub fn uuid_nil() -> Uuid {
    Uuid::nil()
}

/// Test creating a max (all-ones) UUID.
#[miniextendr]
pub fn uuid_max() -> Uuid {
    Uuid::max()
}

/// Test extracting the version number from a UUID.
/// @param uuid UUID parsed from R string.
#[miniextendr]
pub fn uuid_version(uuid: Uuid) -> i32 {
    uuid.get_version_num() as i32
}

/// Test whether a UUID is the nil UUID.
/// @param uuid UUID parsed from R string.
#[miniextendr]
pub fn uuid_is_nil(uuid: Uuid) -> bool {
    uuid.is_nil()
}
