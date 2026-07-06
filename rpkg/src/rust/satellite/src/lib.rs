//! A miniextendr-free crate. Only `serde` is in scope here — there is no
//! `use miniextendr_api::...` anywhere, and there never should be. The whole
//! point is to model a real third-party crate that happens to derive serde and
//! gets R interop "for free" through rpkg's bridge layer.

use serde::{Deserialize, Serialize};

/// Plain record: scalars + an optional (NA-carrying) field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reading {
    pub sensor: String,
    pub value: f64,
    pub ok: bool,
    pub note: Option<String>,
}

/// Nested struct — exercises the columnar flatten path (`site_lat`, `site_lon`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: i32,
    pub site: Site,
    pub readings_taken: i64,
}

/// Enum with data variants — exercises the tagged-list / split path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Started { at: i64 },
    Measured { sensor: String, value: f64 },
    Failed { code: i32 },
}

/// A canned batch of readings, so the R side can call a no-arg function.
pub fn sample_readings() -> Vec<Reading> {
    vec![
        Reading {
            sensor: "temp".into(),
            value: 21.5,
            ok: true,
            note: Some("nominal".into()),
        },
        Reading {
            sensor: "humidity".into(),
            value: 48.0,
            ok: true,
            note: None,
        },
        Reading {
            sensor: "pressure".into(),
            value: 1013.2,
            ok: false,
            note: Some("drift".into()),
        },
    ]
}

pub fn sample_stations() -> Vec<Station> {
    vec![
        Station {
            id: 1,
            site: Site {
                lat: 51.5,
                lon: -0.12,
            },
            readings_taken: 8_000_000_000,
        },
        Station {
            id: 2,
            site: Site {
                lat: 40.7,
                lon: -74.0,
            },
            readings_taken: 12,
        },
    ]
}

pub fn sample_events() -> Vec<Event> {
    vec![
        Event::Started { at: 1_700_000_000 },
        Event::Measured {
            sensor: "temp".into(),
            value: 21.5,
        },
        Event::Failed { code: 7 },
    ]
}
