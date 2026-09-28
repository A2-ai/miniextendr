//! `#[miniextendr]` trait methods that take (and return) the reading markers,
//! driven by `tests/testthat/test-trait-markers.R`.
//!
//! The trait's View converts each argument with `IntoR` and the vtable shim
//! reads it back with `TryFromSexp`, so a marker parameter crosses as its
//! inner value (`AsFromStr` as its text). From R, the implementing method's
//! wrapper reads the argument the way the marker does, with its R guard and,
//! under `no_na`, the check of the converted value. The parameter is `value`
//! because `x` is the receiver of an env-class trait method.

use std::net::IpAddr;

use miniextendr_api::{
    AsCharacter, AsCharacterVec, AsFromStr, AsFromStrVec, AsNumeric, AsNumericVec, Missing,
    miniextendr,
};

/// Reading-marker parameters on trait methods.
#[miniextendr]
pub trait MarkerArgs {
    /// The number read, returned as the marker: `NA` when it reads as missing.
    fn marker_number(&self, value: AsNumeric) -> AsNumeric;
    /// How many values read as missing.
    fn marker_missing(&self, value: AsNumericVec) -> i32;
    /// The label read, `"<NA>"` for `NA`.
    fn marker_label(&self, value: AsCharacter) -> String;
    /// `"null"` for `NULL`, else the labels joined by commas (`"<NA>"` for `NA`).
    fn marker_labels(&self, value: Option<AsCharacterVec>) -> String;
    /// `"absent"` when omitted, else the number read.
    fn marker_maybe(&self, value: Missing<AsNumeric>) -> String;
    /// The address parsed, formatted back.
    fn marker_ip(&self, value: AsFromStr<IpAddr>) -> String;
    /// How many addresses were parsed.
    fn marker_ips(&self, value: AsFromStrVec<IpAddr>) -> i32;
    /// The numbers read, returned as the marker.
    fn marker_echo(&self, value: AsNumericVec) -> AsNumericVec;
    /// The number read; the implementation refuses `NA` with `no_na`.
    fn marker_dose(&self, value: AsNumeric) -> f64;
}

/// Env class implementing `MarkerArgs`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MarkerReader;

/// Env class whose `MarkerArgs` trait methods take the reading markers.
#[miniextendr(env)]
impl MarkerReader {
    /// Create a reader.
    pub fn new() -> Self {
        MarkerReader
    }
}

/// A number the way the fixture reports it: `NA` for `None`.
fn number_text(x: Option<f64>) -> String {
    x.map_or_else(|| "NA".to_string(), |v| v.to_string())
}

/// A label the way the fixture reports it: `<NA>` for `None`.
fn label_text(x: Option<String>) -> String {
    x.unwrap_or_else(|| "<NA>".to_string())
}

fn count(n: usize) -> i32 {
    i32::try_from(n).expect("fewer than 2^31 values")
}

#[miniextendr(env)]
impl MarkerArgs for MarkerReader {
    fn marker_number(&self, value: AsNumeric) -> AsNumeric {
        value
    }

    fn marker_missing(&self, value: AsNumericVec) -> i32 {
        count(value.0.iter().filter(|v| v.is_none()).count())
    }

    fn marker_label(&self, value: AsCharacter) -> String {
        label_text(value.0)
    }

    fn marker_labels(&self, value: Option<AsCharacterVec>) -> String {
        match value {
            None => "null".to_string(),
            Some(labels) => labels
                .0
                .into_iter()
                .map(label_text)
                .collect::<Vec<_>>()
                .join(","),
        }
    }

    fn marker_maybe(&self, value: Missing<AsNumeric>) -> String {
        match value {
            Missing::Absent => "absent".to_string(),
            Missing::Present(value) => number_text(value.0),
        }
    }

    fn marker_ip(&self, value: AsFromStr<IpAddr>) -> String {
        value.0.to_string()
    }

    fn marker_ips(&self, value: AsFromStrVec<IpAddr>) -> i32 {
        count(value.0.len())
    }

    fn marker_echo(&self, value: AsNumericVec) -> AsNumericVec {
        value
    }

    #[miniextendr(no_na(value))]
    fn marker_dose(&self, value: AsNumeric) -> f64 {
        value.0.expect("no_na refuses NA")
    }
}

/// Calls the `MarkerArgs` methods through the trait's View, the path another
/// package takes: each marker argument crosses as its inner value and is read
/// back unchanged (`None`, `NaN`, the text `"NA"`, `NULL`, an omitted
/// argument), and a returned marker crosses back the same way.
#[miniextendr(no_worker)]
pub fn marker_args_through_view() -> Vec<String> {
    let localhost: IpAddr = "127.0.0.1".parse().expect("an IPv4 address");
    let loopback6: IpAddr = "::1".parse().expect("an IPv6 address");
    unsafe {
        let erased = __mx_wrap_markerreader(MarkerReader);
        let sexp = miniextendr_api::gc_protect::OwnedProtect::new(
            miniextendr_api::trait_abi::ccall::mx_wrap(erased),
        );
        let view = MarkerArgsView::from_sexp(sexp.get());
        let echoed = view.marker_echo(AsNumericVec(vec![Some(1.0), None, Some(f64::NAN)]));
        vec![
            number_text(view.marker_number(AsNumeric(Some(2.5))).0),
            number_text(view.marker_number(AsNumeric(None)).0),
            number_text(view.marker_number(AsNumeric(Some(f64::NAN))).0),
            view.marker_missing(AsNumericVec(vec![Some(1.0), None, None]))
                .to_string(),
            view.marker_label(AsCharacter(Some("NA".to_string()))),
            view.marker_label(AsCharacter(None)),
            view.marker_labels(None),
            view.marker_labels(Some(AsCharacterVec(vec![Some("a".to_string()), None]))),
            view.marker_maybe(Missing::Absent),
            view.marker_maybe(Missing::Present(AsNumeric(Some(1.0)))),
            view.marker_ip(AsFromStr(localhost)),
            view.marker_ips(AsFromStrVec(vec![localhost, loopback6]))
                .to_string(),
            echoed
                .0
                .into_iter()
                .map(number_text)
                .collect::<Vec<_>>()
                .join(","),
        ]
    }
}
