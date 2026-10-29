//! The snapshot encoding round-trips exactly (#2256): each value is encoded,
//! saved with R's `serialize()`, read back with `unserialize()` and decoded,
//! as a snapshot of an `ExternalPtr` is. Every value is checked bare and
//! inside a plain struct, an untagged enum, an internally tagged enum, a
//! `#[serde(flatten)]` struct, a `Vec` and a map value: the last four decode
//! through serde's buffered `Content`, which sees only what `deserialize_any`
//! visits. The general `to_r` / `from_r` mapping is unchanged.
#![cfg(feature = "serde")]

mod r_test_utils;

use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use miniextendr_api::expression::RCall;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::serde::{__from_r_exact, __to_r_exact, from_r, to_r};
use miniextendr_api::{OwnedProtect, SEXP, SEXPTYPE, SexpExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// region: Harness

/// `value` encoded, saved and read back with R, and decoded.
fn try_reload<T: Serialize + DeserializeOwned>(value: &T) -> Result<T, String> {
    // SAFETY: on R's main thread; each value is rooted before the next call
    // allocates, and the guards drop in reverse order.
    unsafe {
        let encoded =
            OwnedProtect::new(__to_r_exact(value).map_err(|e| format!("encoding failed: {e}"))?);
        let bytes = OwnedProtect::new(
            RCall::new("serialize")
                .arg(encoded.get())
                .arg(SEXP::nil())
                .eval_base()
                .expect("serialize"),
        );
        let back = OwnedProtect::new(
            RCall::new("unserialize")
                .arg(bytes.get())
                .eval_base()
                .expect("unserialize"),
        );
        __from_r_exact(back.get()).map_err(|e| format!("decoding failed: {e}"))
    }
}

fn reload<T: Serialize + DeserializeOwned>(value: &T) -> T {
    try_reload(value).unwrap_or_else(|e| panic!("{}: {e}", std::any::type_name::<T>()))
}

/// A struct no value matches (a derived struct also reads a sequence of its
/// fields): the first variant of `InUntagged`, so serde tries it first and
/// has to fall back.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Decoy {
    never: Never,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum Never {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct InPlain<T> {
    value: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
enum InUntagged<T> {
    Decoy(Decoy),
    Value(T),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind")]
enum InTagged<T> {
    Holder { value: T },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct InFlatten<T> {
    label: String,
    #[serde(flatten)]
    inner: InPlain<T>,
}

/// `value` comes back unchanged bare, in a plain struct, an untagged enum, an
/// internally tagged enum, a flattened struct, a `Vec` and a map value.
fn check_everywhere<T>(value: T)
where
    T: Serialize + DeserializeOwned + Clone + Debug + PartialEq,
{
    let context = |place: &str| format!("{place}: {value:?}");
    assert_eq!(reload(&value), value, "{}", context("bare"));
    assert_eq!(
        reload(&InPlain {
            value: value.clone()
        }),
        InPlain {
            value: value.clone()
        },
        "{}",
        context("plain struct")
    );
    assert_eq!(
        reload(&InUntagged::Value(value.clone())),
        InUntagged::Value(value.clone()),
        "{}",
        context("untagged enum")
    );
    assert_eq!(
        reload(&InTagged::Holder {
            value: value.clone()
        }),
        InTagged::Holder {
            value: value.clone()
        },
        "{}",
        context("internally tagged enum")
    );
    let flat = InFlatten {
        label: "flat".to_owned(),
        inner: InPlain {
            value: value.clone(),
        },
    };
    assert_eq!(reload(&flat), flat, "{}", context("flattened struct"));
    assert_eq!(
        reload(&vec![value.clone()]),
        vec![value.clone()],
        "{}",
        context("Vec")
    );
    let map: BTreeMap<String, T> = [("key".to_owned(), value.clone())].into();
    assert_eq!(reload(&map), map, "{}", context("map value"));
}

/// An `f64` compared by its bits, so NaNs and `-0.0` compare exactly.
#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(transparent)]
struct Bits(f64);

impl PartialEq for Bits {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Debug for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#018x}", self.0.to_bits())
    }
}

// endregion

// region: The shapes a downstream package found

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Plain {
    values: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
enum Values {
    Many(Vec<String>),
    Flag(bool),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Selector {
    name: String,
    values: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
enum Scope {
    Selectors(Vec<Selector>),
    Any { any: Vec<Scope> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Notice {
    Dropped { profiles: Vec<String> },
    Counted { n: usize },
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|&s| s.to_owned()).collect()
}

fn selector(values: &[&str]) -> Selector {
    Selector {
        name: "ID".to_owned(),
        values: strings(values),
    }
}

/// `NA_real_`: R's NA payload, 1954, in a quiet NaN.
const NA_REAL_BITS: u64 = 0x7ff0_0000_0000_07a2;

#[test]
fn serde_exact_suite() {
    let cases: [(&str, fn()); 9] = [
        (
            "one-element sequences in buffered enums",
            one_element_sequences_in_buffered_enums,
        ),
        ("floats keep their bits", floats_keep_their_bits),
        ("integers come back unchanged", integers_come_back_unchanged),
        (
            "maps with keys that are not strings",
            maps_with_keys_that_are_not_strings,
        ),
        ("nested options", nested_options),
        ("strings holding a NUL byte", strings_holding_a_nul_byte),
        (
            "a million floats stay one double vector",
            a_million_floats_stay_one_double_vector,
        ),
        (
            "values outside the encoding are refused",
            values_outside_the_encoding_are_refused,
        ),
        (
            "the general mapping is unchanged",
            the_general_mapping_is_unchanged,
        ),
    ];
    // R's panic hook keeps the message off the test output: report it here.
    let failures = r_test_utils::with_r_thread(move || {
        cases
            .into_iter()
            .filter_map(|(name, case)| {
                std::panic::catch_unwind(case).err().map(|payload| {
                    let message = payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                        .unwrap_or_default();
                    format!("{name}: {message}")
                })
            })
            .collect::<Vec<_>>()
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn one_element_sequences_in_buffered_enums() {
    check_everywhere(Plain {
        values: strings(&["1"]),
    });
    check_everywhere(Values::Many(strings(&["1"])));
    check_everywhere(Values::Many(strings(&["1", "2"])));
    check_everywhere(Values::Many(vec![]));
    check_everywhere(Values::Flag(true));
    check_everywhere(Scope::Selectors(vec![selector(&["1"])]));
    check_everywhere(Scope::Selectors(vec![selector(&["1", "2"])]));
    check_everywhere(Scope::Any {
        any: vec![Scope::Selectors(vec![selector(&["1"])])],
    });
    check_everywhere(Notice::Dropped {
        profiles: strings(&["ID=1"]),
    });
    check_everywhere(Notice::Dropped {
        profiles: strings(&["ID=1", "ID=2"]),
    });
    check_everywhere(Notice::Dropped { profiles: vec![] });
    check_everywhere(Notice::Counted { n: 3 });
    // Untagged enums nested in each context.
    check_everywhere(InUntagged::Value(Values::Many(strings(&["1"]))));
    check_everywhere(InTagged::Holder {
        value: Values::Many(strings(&["1"])),
    });
    check_everywhere(vec![1.5]);
    check_everywhere(vec![true]);
    check_everywhere(vec![7i32]);
    check_everywhere(vec![vec![1.0], vec![]]);
    check_everywhere((1u8, "one".to_owned()));
    check_everywhere(vec![Some(1.0), None]);
}

fn floats_keep_their_bits() {
    let bits = [
        NA_REAL_BITS,
        f64::NAN.to_bits(),
        // A NaN with another payload, and a signaling one.
        0x7ff8_0000_0000_0123,
        0x7ff0_0000_0000_0001,
        (-0.0f64).to_bits(),
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
        f64::MIN_POSITIVE.to_bits(),
        1.5f64.to_bits(),
    ];
    for bits in bits {
        let x = Bits(f64::from_bits(bits));
        check_everywhere(x);
        // `Some(NA_real_)` stays `Some`: only `NULL` is `None`.
        check_everywhere(Some(x));
        check_everywhere(vec![x, Bits(2.0)]);
        check_everywhere(vec![Some(x), None]);
    }
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct Real {
        x: Bits,
    }
    check_everywhere(Real {
        x: Bits(f64::from_bits(NA_REAL_BITS)),
    });
    check_everywhere(1.25f32);
}

fn integers_come_back_unchanged() {
    check_everywhere(i32::MIN);
    check_everywhere(i32::MAX);
    check_everywhere(u32::MAX);
    check_everywhere(9_007_199_254_740_993u64);
    check_everywhere(u64::MAX);
    check_everywhere(i64::MIN);
    check_everywhere(i64::from(i32::MIN) - 1);
    check_everywhere(vec![1u64, 9_007_199_254_740_993, u64::MAX]);
    check_everywhere(vec![i64::MIN, 0, i64::MAX]);
    check_everywhere(Some(i32::MIN));

    // 128-bit integers are exact where serde reads them directly.
    for value in [i128::MIN, i128::from(i64::MIN), -1, i128::MAX] {
        assert_eq!(reload(&value), value);
        assert_eq!(reload(&InPlain { value }), InPlain { value });
        assert_eq!(reload(&vec![value, 1]), vec![value, 1]);
    }
    for value in [u128::MAX, u128::from(u64::MAX), 0] {
        assert_eq!(reload(&value), value);
        assert_eq!(reload(&vec![Some(value), None]), vec![Some(value), None]);
    }
    // serde's `Content` reads no 128-bit integer at all, so an untagged
    // enum can't hold one, however small (#2257, serde-rs/serde#1682).
    let err = try_reload(&InUntagged::Value(1i128)).expect_err("Content has no i128");
    assert!(err.contains("decoding failed"), "{err}");
}

fn maps_with_keys_that_are_not_strings() {
    let profiles: HashMap<Vec<(String, String)>, usize> = [
        (vec![("ID".to_owned(), "1".to_owned())], 0),
        (
            vec![
                ("ID".to_owned(), "2".to_owned()),
                ("DOSE".to_owned(), "10".to_owned()),
            ],
            1,
        ),
    ]
    .into();
    check_everywhere(profiles);
    let by_id: HashMap<u32, f64> = [(1, 0.5), (u32::MAX, 2.5)].into();
    check_everywhere(by_id);
    let by_flag: BTreeMap<bool, Vec<u64>> = [(false, vec![]), (true, vec![u64::MAX])].into();
    check_everywhere(by_flag);
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Color {
        Red,
        Blue,
    }
    let by_color: BTreeMap<Color, u8> = [(Color::Red, 1), (Color::Blue, 2)].into();
    check_everywhere(by_color);
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Key(String);
    let by_key: BTreeMap<Key, u8> = [(Key("a".to_owned()), 1)].into();
    check_everywhere(by_key);
    check_everywhere(BTreeMap::<String, u8>::new());
    check_everywhere(BTreeMap::<u32, u8>::new());
}

fn nested_options() {
    check_everywhere(None::<Option<Bits>>);
    check_everywhere(Some(None::<Bits>));
    check_everywhere(Some(Some(Bits(f64::from_bits(NA_REAL_BITS)))));
    check_everywhere(Some(Some(None::<u8>)));
    check_everywhere(Some(()));
    check_everywhere(vec![Some(None::<u8>), None, Some(Some(1))]);
}

fn strings_holding_a_nul_byte() {
    check_everywhere("a\0b".to_owned());
    check_everywhere('\0');
    check_everywhere(strings(&["x", "a\0b"]));
    let keys: BTreeMap<String, u8> = [("a\0b".to_owned(), 1), ("c".to_owned(), 2)].into();
    check_everywhere(keys);
    check_everywhere(String::new());
    check_everywhere("NA".to_owned());
}

fn a_million_floats_stay_one_double_vector() {
    let values: Vec<f64> = (0..1_000_000).map(f64::from).collect();
    // SAFETY: on R's main thread; rooted while inspected.
    let encoded = unsafe { OwnedProtect::new(__to_r_exact(&values).expect("encodes")) };
    assert_eq!(encoded.get().type_of(), SEXPTYPE::REALSXP);
    assert_eq!(encoded.get().len(), 1_000_000);
    assert_eq!(
        __from_r_exact::<Vec<f64>>(encoded.get()).expect("decodes"),
        values
    );
    drop(encoded);
    assert_eq!(reload(&values), values);
}

fn values_outside_the_encoding_are_refused() {
    // An atomic vector of two elements without the `serde` attribute: the
    // encoding writes none, and won't guess.
    // SAFETY: on R's main thread; rooted while decoded.
    let unmarked = unsafe { OwnedProtect::new(vec![1.0, 2.0].into_sexp()) };
    let err = __from_r_exact::<Vec<f64>>(unmarked.get()).expect_err("not the encoding");
    assert!(
        err.to_string()
            .contains("not a value of the snapshot encoding: numeric of length 2"),
        "{err}"
    );
}

fn the_general_mapping_is_unchanged() {
    // `to_r` still writes a one-element `Vec<String>` as `character(1)`,
    // without an attribute.
    // SAFETY: on R's main thread; rooted while inspected.
    let general = unsafe { OwnedProtect::new(to_r(&strings(&["1"])).expect("encodes")) };
    assert_eq!(general.get().type_of(), SEXPTYPE::STRSXP);
    assert_eq!(general.get().len(), 1);
    assert!(!general.get().has_attributes());
    assert_eq!(
        from_r::<Vec<String>>(general.get()).expect("decodes"),
        strings(&["1"])
    );
    drop(general);
    // And still fails inside an untagged enum (#1417 and #1419 stay open for
    // the general mapping).
    let general =
        unsafe { OwnedProtect::new(to_r(&Values::Many(strings(&["1"]))).expect("encodes")) };
    assert!(from_r::<Values>(general.get()).is_err());
}

// endregion
