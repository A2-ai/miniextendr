//! Read-only `#[r_data]` fields (#2268): `#[r_data(read_only)]` per field
//! under S3, R6 and S7, `Computed` fields under R6 and S7 (#1888), and the
//! read-only field syntax `r_data_accessors = "read_only"` under S3, S4 and
//! env. Read-only applies to R only: the methods below write the fields
//! through the Rust side.

use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar};
use miniextendr_api::miniextendr;
use miniextendr_api::{ExternalPtrMut, ExternalPtrRef};

// region: S3, read-only fields next to a writable one

/// An S3 class whose `id` and `keys` R reads and can't write, whose
/// `doubled` is computed, and whose `note` R writes. A refused write
/// carries the package's classes and hint (`field_error`).
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(
    s3,
    field_error(
        class = ["readonly_error_field", "readonly_error"],
        read_only_hint = "Build a new object to change `%s`."
    )
)]
pub struct ReadOnlyS3 {
    #[r_data]
    _r: RSidecar,

    /// The id, read-only from R.
    #[r_data(read_only)]
    pub id: i32,

    /// Keys kept in the pointer's protection list; `keys` in R, read-only
    /// there, written by `add_key()` through the Rust accessors, which
    /// `ref` / `mut` make public.
    #[r_data(ref, mut, read_only, name = "keys")]
    pub r_keys: Sidecar<Vec<i32>>,

    /// `id * 2`, computed from the Rust value.
    #[r_data(get = "Self::doubled")]
    pub doubled: Computed,

    /// A note R writes.
    #[r_data]
    pub note: String,
}

impl ReadOnlyS3 {
    fn doubled(&self) -> i32 {
        self.id * 2
    }
}

/// S3 class registration for ReadOnlyS3: `$` / `[[` read every field,
/// `$<-` / `[[<-` write `note` and refuse the others.
#[miniextendr(s3(r_data_accessors))]
impl ReadOnlyS3 {
    /// Create a ReadOnlyS3 with no keys.
    /// @param id The id.
    /// @param note The note.
    pub fn new(id: i32, note: String) -> Self {
        ReadOnlyS3 {
            _r: RSidecar,
            id,
            r_keys: Sidecar::new(vec![]),
            doubled: Computed,
            note,
        }
    }

    /// Appends `key` to `keys` through the handle: Rust writes a field R
    /// can't.
    /// @param key The key to append.
    pub fn add_key(mut self: ExternalPtrMut<'_, Self>, key: i32) {
        let mut keys = Self::r_keys(&self);
        keys.push(key);
        Self::set_r_keys(&mut self, keys);
    }
}
// endregion

// region: S3, every field read-only (`r_data_accessors = "read_only"`)

/// An S3 class whose fields R reads and can't write: `$<-` / `[[<-` refuse
/// every field with the package's classes and hint.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(
    s3,
    field_error(
        class = "frozen_error",
        read_only_hint = "A FrozenS3 is fixed once built."
    )
)]
pub struct FrozenS3 {
    #[r_data]
    _r: RSidecar,

    /// The size, a struct field.
    #[r_data]
    pub size: i32,

    /// Items kept in the pointer's protection list; `items` in R, written
    /// by `grow()`.
    #[r_data(name = "items")]
    pub r_items: Sidecar<Vec<String>>,
}

/// S3 class registration for FrozenS3: `$` / `[[` read the fields and
/// `$<-` / `[[<-` refuse them.
#[miniextendr(s3(r_data_accessors = "read_only"))]
impl FrozenS3 {
    /// Create a FrozenS3 with no items.
    /// @param size The size.
    pub fn new(size: i32) -> Self {
        FrozenS3 {
            _r: RSidecar,
            size,
            r_items: Sidecar::new(vec![]),
        }
    }

    /// Appends `item` to `items` through the handle.
    /// @param item The item to append.
    pub fn grow(mut self: ExternalPtrMut<'_, Self>, item: String) {
        let mut items = Self::r_items(&self);
        items.push(item);
        Self::set_r_items(&mut self, items);
    }

    /// Number of `items`, read through the handle.
    pub fn item_count(self: ExternalPtrRef<'_, Self>) -> i32 {
        i32::try_from(Self::r_items(&self).len()).expect("item count exceeds i32")
    }
}
// endregion

// region: S4, every field read-only

/// An S4 class whose fields `o$depth` / `o$label` read and `o$depth <-
/// value` refuses.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s4)]
pub struct FrozenS4 {
    #[r_data]
    _r: RSidecar,

    /// The depth, a struct field.
    #[r_data]
    pub depth: i32,

    /// The label, a struct field.
    #[r_data]
    pub label: String,
}

/// S4 class registration for FrozenS4: `$` reads the fields and `$<-`
/// refuses them.
#[miniextendr(s4(r_data_accessors = "read_only"))]
impl FrozenS4 {
    /// Create a FrozenS4.
    /// @param depth The depth.
    /// @param label The label.
    pub fn new(depth: i32, label: String) -> Self {
        FrozenS4 {
            _r: RSidecar,
            depth,
            label,
        }
    }
}
// endregion

// region: env, every field read-only

/// An env class whose `count` `x$count` reads, `x$count <- value` refuses
/// and `x$bump()` increments.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct FrozenEnv {
    #[r_data]
    _r: RSidecar,

    /// The count, a struct field.
    #[r_data]
    pub count: i32,
}

/// The env class FrozenEnv: `$` reads the field, then a method, and `$<-` /
/// `[[<-` refuse the field.
#[miniextendr(env(r_data_accessors = "read_only"))]
impl FrozenEnv {
    /// Create a FrozenEnv.
    /// @param count The count.
    pub fn new(count: i32) -> Self {
        FrozenEnv {
            _r: RSidecar,
            count,
        }
    }

    /// Increments `count`.
    pub fn bump(&mut self) {
        self.count += 1;
    }
}
// endregion

// region: R6, read-only and computed active bindings next to a writable one

/// An R6 class whose `id` binding refuses a value, whose `doubled` binding
/// is computed (and refuses a value), and whose `level` binding R writes.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(
    r6,
    field_error(
        class = "readonly_r6_error",
        read_only_hint = "Build a new ReadOnlyR6 to change `%s`."
    )
)]
pub struct ReadOnlyR6 {
    #[r_data]
    _r: RSidecar,

    #[r_data(read_only)]
    pub id: i32,

    #[r_data(get = "Self::doubled")]
    pub doubled: Computed,

    #[r_data]
    pub level: i32,
}

impl ReadOnlyR6 {
    fn doubled(&self) -> i32 {
        self.id * 2
    }
}

/// R6 class registration for ReadOnlyR6.
/// @field id The id (read-only active binding).
/// @field doubled Twice `id` (read-only active binding, computed from the Rust value).
/// @field level The level (active binding).
#[miniextendr(r6(r_data_accessors))]
impl ReadOnlyR6 {
    /// Create a new ReadOnlyR6.
    /// @param id The id.
    /// @param level The level.
    pub fn new(id: i32, level: i32) -> Self {
        ReadOnlyR6 {
            _r: RSidecar,
            id,
            doubled: Computed,
            level,
        }
    }

    /// Increments `id`: Rust writes a field R can't.
    pub fn next_id(&mut self) {
        self.id += 1;
    }
}
// endregion

// region: S7, read-only and computed properties next to a writable one

/// An S7 class whose `id` property has no setter, whose `doubled` property
/// is computed (no setter either), and whose `level` property R writes.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s7)]
pub struct ReadOnlyS7 {
    #[r_data]
    _r: RSidecar,

    #[r_data(read_only, prop_doc = "The id, read-only.")]
    pub id: i32,

    #[r_data(
        get = "Self::doubled",
        prop_doc = "Twice `id`, computed from the Rust value (read-only)."
    )]
    pub doubled: Computed,

    #[r_data(prop_doc = "The level.")]
    pub level: i32,
}

impl ReadOnlyS7 {
    fn doubled(&self) -> i32 {
        self.id * 2
    }
}

/// S7 class registration for ReadOnlyS7 with property-based sidecar accessors.
#[miniextendr(s7(r_data_accessors))]
impl ReadOnlyS7 {
    /// Create a new ReadOnlyS7.
    /// @param id The id.
    /// @param level The level.
    pub fn new(id: i32, level: i32) -> Self {
        ReadOnlyS7 {
            _r: RSidecar,
            id,
            doubled: Computed,
            level,
        }
    }
}
// endregion
