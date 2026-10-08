//! A `Sidecar<T>` type that reads saves by another version of the package
//! through a revive hook (#1854).
//!
//! `SidecarRevive` is an R6 class with `#[externalptr(r6, revive =
//! Self::revive)]`. After `readRDS()` a pointer has no address; the first
//! access of a hooked type runs the hook on the stored `prot` list and
//! installs the rebuilt value in the same pointer. The hook follows the
//! rules a downstream package needs:
//!
//! 1. a slot this version added (`added`) is missing from an older save:
//!    the field keeps its default, nothing is signalled;
//! 2. a slot this version dropped is ignored;
//! 3. a stored value of another shape (`keys` saved as strings) is rebuilt
//!    from the recipe and a classed warning says so;
//! 4. a save by another version is read and a classed warning names it;
//! 5. a save from before the layout record is read positionally.
//!
//! The fixtures build the `EXTPTRSXP` `readRDS()` would give for such a
//! save, so the R tests need no second version of the package.

use std::fmt;

use miniextendr_api::condition::RConditionError;
use miniextendr_api::externalptr::{
    ErasedExternalPtr, ExternalPtr, RSidecar, Sidecar, SlotError, StoredSidecars,
    stored_pointer_for_tests,
};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{OwnedProtect, SEXP, SexpExt};
use miniextendr_api::{SEXPTYPE, TryFromSexp, defer_warning, miniextendr};

// region: The hooked type

/// What the revive hook refuses.
#[derive(Debug, RConditionError)]
#[condition(class = "sidecar_revive_error")]
pub enum ReviveError {
    /// The recipe in the user slot says the object must not be rebuilt.
    #[condition(
        class = "sidecar_revive_refused",
        message = "refused to rebuild: {reason}"
    )]
    Refused { reason: String },
}

/// An R6 class whose `Sidecar` fields come back from a save by any version
/// of the package.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(r6, revive = Self::revive)]
pub struct SidecarRevive {
    #[r_data]
    _r: RSidecar,

    /// Integer keys, `1:n` from the constructor.
    #[r_data(ref, mut)]
    pub keys: Sidecar<Vec<i32>>,

    /// A label, `"fresh"` from the constructor.
    #[r_data(ref, mut)]
    pub label: Sidecar<String>,

    /// A slot this version added: an older save has none, and the hook
    /// keeps `"absent"`.
    #[r_data(ref)]
    pub added: Sidecar<String>,

    /// Number of keys: a struct field, lost by `saveRDS()` and rebuilt by
    /// the hook.
    #[r_data]
    pub n: i32,

    /// Where the value came from: `"new"` from the constructor, `"revived"`
    /// plus what the hook saw after a reload.
    #[r_data]
    pub source: String,
}

impl SidecarRevive {
    /// The revive hook, `#[externalptr(revive = Self::revive)]`.
    ///
    /// The user slot is the recipe: the string `"refuse"` refuses the
    /// rebuild; an integer `n` gives `keys` `1:n` when the stored `keys` are
    /// missing or of another shape (no keys otherwise).
    fn revive(stored: StoredSidecars<'_>) -> Result<Self, ReviveError> {
        let recipe = stored.user_slot();
        if recipe.type_of() == SEXPTYPE::STRSXP
            && matches!(String::try_from_sexp(recipe).as_deref(), Ok("refuse"))
        {
            return Err(ReviveError::Refused {
                reason: String::from("the recipe says so"),
            });
        }
        let recipe_keys = || -> Vec<i32> {
            if recipe.type_of() == SEXPTYPE::INTSXP {
                i32::try_from_sexp(recipe)
                    .map(|n| (1..=n).collect())
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        };
        let mut notes = Vec::new();

        // Rule 4: a save by another version is read, and named.
        let current = env!("CARGO_PKG_VERSION");
        match stored.version() {
            Some(version) if version == current => {}
            Some(version) => {
                defer_warning!(
                    class = "sidecar_revive_version",
                    "rebuilt from a save by version {} (this is {})",
                    version,
                    current
                );
                notes.push(format!("version {version}"));
            }
            None => notes.push(String::from("no version")),
        }

        let (keys, label, added) = match stored.field_names() {
            // Rule 5: a save from before the layout record: `keys` then
            // `label`, by position, after the type ID and the user slot.
            None => {
                notes.push(String::from("no record"));
                let prot = stored.prot();
                let slot = |index: isize| -> SEXP {
                    let len = isize::try_from(prot.len()).expect("prot length");
                    if index < len {
                        prot.vector_elt(index)
                    } else {
                        SEXP::nil()
                    }
                };
                let keys = Vec::<i32>::try_from_sexp(slot(2)).unwrap_or_else(|_| recipe_keys());
                let label =
                    String::try_from_sexp(slot(3)).unwrap_or_else(|_| String::from("unlabelled"));
                (keys, label, String::from("absent"))
            }
            Some(_) => {
                let keys = match stored.slot::<Vec<i32>>("keys") {
                    Ok(keys) => keys,
                    Err(SlotError::Missing { name }) => {
                        notes.push(format!("{name} missing"));
                        recipe_keys()
                    }
                    // Rule 3: another shape; rebuild and say so.
                    Err(SlotError::Conversion { name, error }) => {
                        defer_warning!(
                            class = "sidecar_revive_changed",
                            "`{}` was rebuilt from the recipe: {}",
                            name,
                            error
                        );
                        notes.push(format!("{name} rebuilt"));
                        recipe_keys()
                    }
                };
                let label = match stored.slot::<String>("label") {
                    Ok(label) => label,
                    Err(error) => {
                        notes.push(format!("{} {}", error.name(), slot_outcome(&error)));
                        String::from("unlabelled")
                    }
                };
                // Rule 1: a slot this version added keeps its default.
                let added = match stored.slot::<String>("added") {
                    Ok(added) => added,
                    Err(error) => {
                        notes.push(format!("{} {}", error.name(), slot_outcome(&error)));
                        String::from("absent")
                    }
                };
                // Rule 2: a stored slot nobody asks for is ignored.
                (keys, label, added)
            }
        };

        let n = i32::try_from(keys.len()).expect("key count fits i32");
        let source = if notes.is_empty() {
            String::from("revived")
        } else {
            format!("revived ({})", notes.join(", "))
        };
        Ok(SidecarRevive {
            _r: RSidecar,
            keys: Sidecar::new(keys),
            label: Sidecar::new(label),
            added: Sidecar::new(added),
            n,
            source,
        })
    }
}

/// One word for a `SlotError`, for the `source` notes.
fn slot_outcome<E>(error: &SlotError<E>) -> &'static str {
    if error.is_missing() {
        "missing"
    } else {
        "rebuilt"
    }
}

impl fmt::Display for SidecarRevive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "keys={:?} label={} added={}",
            self.keys(),
            self.label(),
            self.added()
        )
    }
}

/// R6 class with a revive hook: its `Sidecar` fields come back from a save
/// by any version of the package.
/// @field keys Integer keys `1:n`, built by the constructor (active binding).
/// @field label A label, `"fresh"` from the constructor (active binding).
/// @field added A slot this version added; `"present"` from the constructor,
///   `"absent"` when the hook found none (active binding).
/// @field n Number of keys; a struct field the hook rebuilds (active binding).
/// @field source `"new"` from the constructor, `"revived"` plus what the hook
///   saw after a reload (active binding).
#[miniextendr(r6(r_data_accessors))]
impl SidecarRevive {
    /// Create a SidecarRevive whose `keys` are `1:n`.
    /// @param n Number of keys.
    pub fn new(n: i32) -> Self {
        SidecarRevive {
            _r: RSidecar,
            keys: Sidecar::new((1..=n).collect()),
            label: Sidecar::new(String::from("fresh")),
            added: Sidecar::new(String::from("present")),
            n,
            source: String::from("new"),
        }
    }

    /// Length of `keys`, read through `&self`.
    pub fn key_count(&self) -> i32 {
        i32::try_from(self.keys().len()).expect("keys length exceeds i32")
    }

    /// Appends `key` to `keys` through `&mut self`.
    /// @param key The key to append.
    pub fn push_key(&mut self, key: i32) {
        let mut keys = self.keys();
        keys.push(key);
        self.set_keys(keys);
        self.n += 1;
    }

    /// Rebuilds the value by value (`self -> Self`) with a new label.
    /// @param label The new label.
    pub fn relabel(self, label: String) -> Self {
        Self {
            label: Sidecar::new(label),
            ..self
        }
    }
}

/// RDisplay trait ABI registration for SidecarRevive: `as_r_string()` reads
/// the sidecar fields through the trait-impl receiver path.
#[miniextendr(r6)]
impl miniextendr_api::adapter_traits::RDisplay for SidecarRevive {}

// endregion

// region: Fixtures over the stored pointer

/// The `EXTPTRSXP` `readRDS()` gives for a save of a `SidecarRevive` by
/// another version: no address, `type_id` as the stored type ID, `user` in
/// the user slot, the elements of `slots` as the slot values and `names` as
/// the layout record (`NULL` for a save from before the record).
/// @param type_id The stored type ID, `crate@version::module::Type`.
/// @param user The user slot.
/// @param slots A list, one element per slot.
/// @param names The slot names, or `NULL`.
#[miniextendr(noexport)]
#[doc(hidden)]
pub fn sidecar_revive_stored(
    type_id: String,
    user: SEXP,
    slots: SEXP,
    names: Option<Vec<String>>,
) -> SEXP {
    assert_eq!(slots.type_of(), SEXPTYPE::VECSXP, "`slots` is a list");
    let slots: Vec<SEXP> = (0..slots.len())
        .map(|i| slots.vector_elt(isize::try_from(i).expect("list length")))
        .collect();
    let names: Option<Vec<&str>> = names
        .as_ref()
        .map(|names| names.iter().map(String::as_str).collect());
    // SAFETY: on R's main thread (a `#[miniextendr]` fn); `user` and the
    // slots are this call's arguments, which R roots.
    unsafe { stored_pointer_for_tests(&type_id, user, &slots, names.as_deref()) }
}

/// The type ID a `SidecarRevive` built by this version carries.
#[miniextendr(noexport)]
pub fn sidecar_revive_type_id() -> String {
    let fresh = ExternalPtr::new(SidecarRevive::new(0));
    ExternalPtr::<SidecarRevive>::stored_type_id(fresh.as_sexp())
        .expect("a fresh pointer carries its type ID")
}

/// Whether the external pointer `x` has an address, read without reviving.
/// @param x An external pointer.
#[miniextendr(noexport)]
pub fn sidecar_revive_is_live(x: SEXP) -> bool {
    assert_eq!(
        x.type_of(),
        SEXPTYPE::EXTPTRSXP,
        "`x` is an external pointer"
    );
    // SAFETY: `x` is an EXTPTRSXP, on R's main thread.
    !unsafe { miniextendr_api::sys::R_ExternalPtrAddr(x) }.is_null()
}

/// The stored type ID of `x`, an external pointer or an R6 object, live or
/// not; `NA` for anything else.
/// @param x An external pointer or an R6 object.
#[miniextendr(noexport)]
pub fn sidecar_stored_type_id(x: SEXP) -> Option<String> {
    ExternalPtr::<SidecarRevive>::stored_type_id(x)
}

/// A `SidecarRevive` with `n` keys whose user slot holds `recipe` from the
/// start (`ExternalPtr::new_with_protected`).
/// @param n Number of keys.
/// @param recipe Any R value.
#[miniextendr(noexport)]
pub fn sidecar_revive_with_recipe(n: i32, recipe: SEXP) -> ExternalPtr<SidecarRevive> {
    ExternalPtr::new_with_protected(SidecarRevive::new(n), recipe)
}

/// The user slot of a `SidecarRevive` pointer, revived first when needed.
/// @param ptr A `SidecarRevive` pointer.
#[miniextendr(noexport)]
pub fn sidecar_revive_recipe(ptr: ExternalPtr<SidecarRevive>) -> SEXP {
    ptr.protected()
}

/// The `keys` of a pointer, through the `ExternalPtr<T>` argument
/// conversion.
/// @param ptr A `SidecarRevive` pointer.
#[miniextendr(noexport)]
pub fn sidecar_revive_keys_via_ptr(ptr: ExternalPtr<SidecarRevive>) -> Vec<i32> {
    ptr.keys()
}

// endregion

// region: GC stress

/// Allocate garbage so a GC runs between the steps.
fn churn() {
    for i in 0..64 {
        let _garbage = vec![f64::from(i); 64].into_sexp();
    }
}

/// `type_id` with its version replaced by `version`.
fn with_version(type_id: &str, version: &str) -> String {
    let (krate, rest) = type_id.split_once('@').expect("a type ID has a version");
    let (_, path) = rest.split_once("::").expect("a type ID has a path");
    format!("{krate}@{version}::{path}")
}

/// Revive saves by another version under GC churn: the value, its `prot`
/// and the rebuilt sidecar values are rooted by the pointer alone; the
/// warnings the hook defers are signalled when this returns.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep
/// (#430).
#[miniextendr(noexport)]
pub fn gc_stress_sidecar_revive() {
    let current_id = sidecar_revive_type_id();
    let other_id = with_version(&current_id, "0.0.1");

    // SAFETY: on R's main thread (a `#[miniextendr]` fn); every value is
    // protected until the guard drops.
    let keys = unsafe { OwnedProtect::new((1..=5).collect::<Vec<i32>>().into_sexp()) };
    let label = unsafe { OwnedProtect::new(String::from("old").into_sexp()) };
    let legacy = unsafe { OwnedProtect::new(1.5.into_sexp()) };
    let strings =
        unsafe { OwnedProtect::new(vec![String::from("a"), String::from("b")].into_sexp()) };
    let recipe = unsafe { OwnedProtect::new(3.into_sexp()) };

    // A save by another version with a dropped slot: the hook reads `keys`
    // and `label` by name, keeps `added`'s default and ignores `legacy`.
    let stored = unsafe {
        OwnedProtect::new(stored_pointer_for_tests(
            &other_id,
            SEXP::nil(),
            &[keys.get(), label.get(), legacy.get()],
            Some(&["keys", "label", "legacy"]),
        ))
    };
    churn();
    let mut revived = unsafe { ExternalPtr::<SidecarRevive>::wrap_sexp(stored.get()) }
        .expect("a save of the same type revives");
    churn();
    assert_eq!(revived.keys(), vec![1, 2, 3, 4, 5]);
    assert_eq!(revived.label(), "old");
    assert_eq!(revived.added(), "absent");
    assert_eq!(revived.n, 5);
    assert_eq!(revived.source, "revived (version 0.0.1, added missing)");
    assert_eq!(
        ExternalPtr::<SidecarRevive>::stored_type_id(stored.get()).as_deref(),
        Some(current_id.as_str())
    );
    // The rebuilt value is live: a write through `&mut self` lands in the
    // new slots.
    revived.push_key(9);
    churn();
    assert_eq!(revived.keys(), vec![1, 2, 3, 4, 5, 9]);
    assert_eq!(revived.n, 6);
    drop(revived);

    // A recipe set at construction survives churn and a reload: the stored
    // `keys` are strings, so the hook rebuilds `1:3` from it.
    let built = ExternalPtr::new_with_protected(SidecarRevive::new(2), recipe.get());
    churn();
    assert_eq!(
        i32::try_from_sexp(built.protected()).expect("the recipe is an integer"),
        3
    );
    drop(built);
    let stored = unsafe {
        OwnedProtect::new(stored_pointer_for_tests(
            &current_id,
            recipe.get(),
            &[strings.get(), label.get(), label.get()],
            Some(&["keys", "label", "added"]),
        ))
    };
    churn();
    let erased = unsafe { ErasedExternalPtr::from_sexp(stored.get()) };
    let value = erased
        .downcast_ref::<SidecarRevive>()
        .expect("the erased downcast revives");
    churn();
    assert_eq!(value.keys(), vec![1, 2, 3]);
    assert_eq!(value.label(), "old");
    assert_eq!(value.added(), "old");
    assert_eq!(value.source, "revived (keys rebuilt)");
    drop(erased);
    // The recipe is in the rewritten `prot` too.
    let live = unsafe { ExternalPtr::<SidecarRevive>::wrap_sexp(stored.get()) }.expect("live");
    assert_eq!(
        i32::try_from_sexp(live.protected()).expect("the recipe is an integer"),
        3
    );
    drop(live);

    // A save from before the layout record: read by position.
    let stored = unsafe {
        OwnedProtect::new(stored_pointer_for_tests(
            &current_id,
            SEXP::nil(),
            &[keys.get(), label.get()],
            None,
        ))
    };
    churn();
    let revived = unsafe { ExternalPtr::<SidecarRevive>::wrap_sexp(stored.get()) }
        .expect("a record-less save revives");
    churn();
    assert_eq!(revived.keys(), vec![1, 2, 3, 4, 5]);
    assert_eq!(revived.label(), "old");
    assert_eq!(revived.source, "revived (no record)");
    drop(revived);

    // A save of another type is not revived as a `SidecarRevive`.
    let stored = unsafe {
        OwnedProtect::new(stored_pointer_for_tests(
            "miniextendr@0.1.0::miniextendr::other::Other",
            SEXP::nil(),
            &[],
            Some(&[]),
        ))
    };
    churn();
    assert!(unsafe { ExternalPtr::<SidecarRevive>::wrap_sexp(stored.get()) }.is_none());
}

// endregion
