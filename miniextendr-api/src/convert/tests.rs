use super::*;

/// A trait's View converts each argument with `IntoR`, so every reading
/// marker (and the `Option` of those that read `NULL`) is `IntoR`. The
/// round trip through a real R is tested in `tests/as_numeric.rs` and
/// `tests/as_character.rs` (`markers_round_trip`), and through a trait's
/// View in rpkg (`marker_args_through_view`).
#[test]
fn reading_markers_are_into_r() {
    fn into_r<T: IntoR>() {}
    into_r::<AsNumeric>();
    into_r::<AsNumericVec>();
    into_r::<Option<AsNumeric>>();
    into_r::<Option<AsNumericVec>>();
    into_r::<AsCharacter>();
    into_r::<AsCharacterVec>();
    into_r::<Option<AsCharacter>>();
    into_r::<Option<AsCharacterVec>>();
    into_r::<AsFromStr<std::net::IpAddr>>();
    into_r::<AsFromStrVec<std::net::IpAddr>>();
}
