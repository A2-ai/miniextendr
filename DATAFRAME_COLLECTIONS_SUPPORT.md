# DataFrameRow Collection Type Support

## Overview

The `DataFrameRow` derive macro now supports structs with fields containing various Rust collection types. Each collection type in a row field becomes a `Vec<Collection<T>>` in the DataFrame.

## Supported Collection Types

### 1. Vec<T>
Standard Rust vectors work directly:
```rust
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct WithVec {
    pub ids: Vec<i32>,
    pub names: Vec<String>,
}
```

### 2. Box<[T]> - Boxed Slices
Boxed slices are supported for efficient heap-allocated arrays:
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBoxedSlice {
    pub data: Box<[f64]>,
}
```

### 3. [T; N] - Fixed-Size Arrays
Compile-time sized arrays work seamlessly:
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithArray {
    pub coords: [f64; 3],
}
```

### 4. HashSet<T> - Unordered Sets
Hash sets convert to R lists of vectors:
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithHashSet {
    pub tags: HashSet<String>,
}
```

### 5. BTreeSet<T> - Ordered Sets
Tree sets maintain order when converted:
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBTreeSet {
    pub categories: BTreeSet<i32>,
}
```

### 6. Mixed Collections
Different collection types can be mixed in the same struct:
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct MixedCollections {
    pub vec_field: Vec<i32>,
    pub array_field: [f64; 2],
    pub boxed_field: Box<[String]>,
}
```

## How It Works

### DataFrame Structure
When you derive `DataFrameRow` on a struct with collection fields, the generated DataFrame wraps each field type in a `Vec`:

**Row Type:**
```rust
struct MyRow {
    tags: HashSet<String>,
    coords: [f64; 3],
}
```

**Generated DataFrame:**
```rust
struct MyRowDataFrame {
    tags: Vec<HashSet<String>>,  // Vec of sets
    coords: Vec<[f64; 3]>,       // Vec of arrays
}
```

### IntoR Implementations
To support conversion to R, we implemented `IntoR` for these Vec<Collection<T>> types:
- `Vec<Box<[T]>>` → R list of vectors
- `Vec<[T; N]>` → R list of vectors
- `Vec<HashSet<T>>` → R list of vectors (unordered)
- `Vec<BTreeSet<T>>` → R list of vectors (ordered)

Each collection becomes an element in an R list, and the inner elements become R vectors.

## Element Type Requirements

The element type `T` must satisfy:
- **For numeric types (i32, f64, etc.)**: Must implement `RNativeType`
- **For String**: Special implementations provided
- **For other types**: Must implement `IntoR`

## IntoList Requirement

Since rows need to be convertible to R lists, structs with collection fields that don't automatically implement `IntoR` need manual `IntoList` implementations:

```rust
impl ::miniextendr_api::list::IntoList for WithArray {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        ::miniextendr_api::List::from_raw_pairs(vec![
            ("coords", self.coords.to_vec().into_sexp()),
        ])
    }
}
```

## Round-Trip Support

All collection types support round-trip conversion:
```rust
// Create rows
let rows = vec![
    MyRow { tags: my_set, coords: [1.0, 2.0, 3.0] },
    // ...
];

// Convert to DataFrame
let df = MyRow::to_dataframe(rows);

// Convert back to rows
let recovered: Vec<MyRow> = MyRow::from_dataframe(df);
```

## Implementation Details

### Files Modified
1. **miniextendr-api/src/into_r.rs**
   - Added `impl IntoR for Vec<Box<[T]>>`
   - Added `impl IntoR for Vec<[T; N]>`
   - Added `impl IntoR for Vec<HashSet<T>>`
   - Added `impl IntoR for Vec<BTreeSet<T>>`
   - Added specific implementations for String variants

2. **miniextendr-macros/src/dataframe_derive.rs**
   - No changes needed! The macro already supports arbitrary field types by wrapping them in Vec

### Test Coverage
Comprehensive tests in `rpkg/src/rust/dataframe_collections_test.rs`:
- ✅ Vec fields
- ✅ Boxed slice fields
- ✅ Array fields
- ✅ HashSet fields
- ✅ BTreeSet fields
- ✅ Mixed collection types
- ✅ Round-trip conversions

## Examples

### Simple Vec Example
```rust
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct Data {
    pub values: Vec<f64>,
}

let rows = vec![
    Data { values: vec![1.0, 2.0] },
    Data { values: vec![3.0, 4.0, 5.0] },
];

let df = Data::to_dataframe(rows);
// df.values is Vec<Vec<f64>> with 2 elements
```

### Array Example
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct Point3D {
    pub coords: [f64; 3],
}

let rows = vec![
    Point3D { coords: [1.0, 2.0, 3.0] },
    Point3D { coords: [4.0, 5.0, 6.0] },
];

let df = Point3D::to_dataframe(rows);
// df.coords is Vec<[f64; 3]> with 2 elements
```

### Set Example
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct Tagged {
    pub tags: HashSet<String>,
}

let mut tags1 = HashSet::new();
tags1.insert("rust".into());
tags1.insert("r".into());

let rows = vec![Tagged { tags: tags1 }];
let df = Tagged::to_dataframe(rows);
// df.tags is Vec<HashSet<String>> with 1 element
```

## Future Extensions

Potential future additions:
- Slice references (&[T]) with lifetime management
- HashMap/BTreeMap support
- Custom collection types via trait
- Nested collections (Vec<Vec<Vec<T>>>)

## Performance Considerations

- **Cloning**: The `From<Vec<Row>>` impl clones field values. For large collections, consider using `Box<[T]>` or moving data.
- **Hash ordering**: `HashSet` elements may appear in different orders in R.
- **Memory**: Each collection is stored separately, which may use more memory than a flat structure.

## Compatibility

- **Rust version**: Requires Rust 1.93.0 or later (const generics for array support)
- **R version**: Compatible with R 4.0+
- **Features**: No additional Cargo features required

## Related
- [DATAFRAME_HETEROGENEOUS_BUG.md](./DATAFRAME_HETEROGENEOUS_BUG.md) - Original heterogeneous type issue and resolution
- [SESSION_CHANGES.md](./SESSION_CHANGES.md) - Complete session change log
