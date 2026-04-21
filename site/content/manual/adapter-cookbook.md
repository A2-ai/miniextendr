+++
title = "Adapter Trait Cookbook"
weight = 14
description = "Practical recipes for exposing external Rust traits to R using the adapter pattern."
+++

Practical recipes for exposing external Rust traits to R using the adapter pattern.

## Recipe 1: Expose a Custom Iterator to R

**Goal:** Let R code iterate through a Rust iterator one element at a time.

```rust
use miniextendr_api::prelude::*;

// 1. Define the adapter trait
#[miniextendr]
pub trait RIterator {
    fn next_item(&mut self) -> Option<f64>;
    fn size_hint(&self) -> Vec<i32>;
    fn collect_rest(&mut self) -> Vec<f64>;
}

// 2. Wrap your iterator in ExternalPtr
#[derive(ExternalPtr)]
pub struct FloatIter {
    inner: Box<dyn Iterator<Item = f64> + Send>,
}

impl FloatIter {
    pub fn new<I>(iter: I) -> Self
    where
        I: Iterator<Item = f64> + Send + 'static,
    {
        Self { inner: Box::new(iter) }
    }
}

// 3. Implement the adapter
#[miniextendr]
impl RIterator for FloatIter {
    fn next_item(&mut self) -> Option<f64> {
        self.inner.next()
    }

    fn size_hint(&self) -> Vec<i32> {
        let (low, high) = self.inner.size_hint();
        vec![low as i32, high.map(|h| h as i32).unwrap_or(-1)]
    }

    fn collect_rest(&mut self) -> Vec<f64> {
        self.inner.by_ref().collect()
    }
}

// 4. Factory function to create iterator from R
#[miniextendr]
fn range_iter(start: f64, end: f64, step: f64) -> FloatIter {
    let iter = std::iter::successors(Some(start), move |&x| {
        let next = x + step;
        if next < end { Some(next) } else { None }
    });
    FloatIter::new(iter)
}

// 5. Registration is automatic via #[miniextendr] and linkme distributed slices.
```

**Usage in R:**
```r
it <- range_iter(0, 10, 0.5)
it$next_item()   # 0
it$next_item()   # 0.5
it$collect_rest() # c(1, 1.5, 2, ..., 9.5)
```

---

## Recipe 2: Serialize/Deserialize Custom Types with Serde

**Goal:** Convert Rust structs to/from JSON for R interop.

```rust
use miniextendr_api::prelude::*;
use serde::{Serialize, Deserialize};

// 1. Define your data structure
#[derive(Serialize, Deserialize, Clone, ExternalPtr)]
pub struct Config {
    pub name: String,
    pub values: Vec<f64>,
    pub enabled: bool,
}

// 2. Define the serde adapter trait
#[miniextendr]
pub trait RSerializable {
    fn to_json(&self) -> Result<String, String>;
    fn to_json_pretty(&self) -> Result<String, String>;
}

// 3. Implement for Config (or use blanket impl)
#[miniextendr]
impl RSerializable for Config {
    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

// 4. Factory function that parses JSON
#[miniextendr]
fn config_from_json(json: &str) -> Result<Config, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

// 5. Direct field accessors (alternative to JSON)
#[miniextendr]
impl Config {
    fn new(name: &str, values: Vec<f64>, enabled: bool) -> Self {
        Config { name: name.to_string(), values, enabled }
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn values(&self) -> Vec<f64> {
        self.values.clone()
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

// Registration is automatic via #[miniextendr].
```

**Usage in R:**
```r
# Create from JSON
cfg <- config_from_json('{"name": "test", "values": [1,2,3], "enabled": true}')

# Access fields
cfg$name()      # "test"
cfg$values()    # c(1, 2, 3)

# Serialize back
cfg$to_json()        # compact JSON
cfg$to_json_pretty() # formatted JSON

# Or create directly
cfg2 <- Config$new("other", c(4,5,6), FALSE)
```

---

## Recipe 3: Use Rust IO with R Connections

**Goal:** Wrap Rust readers/writers for R to consume.

```rust
use miniextendr_api::prelude::*;
use std::io::{Read, Write, BufRead, BufReader, Cursor};

// 1. Define IO adapter traits
#[miniextendr]
pub trait RReader {
    fn read_bytes(&mut self, n: i32) -> Result<Vec<u8>, String>;
    fn read_all(&mut self) -> Result<Vec<u8>, String>;
    fn read_string(&mut self) -> Result<String, String>;
}

#[miniextendr]
pub trait RLineReader {
    fn read_line(&mut self) -> Result<Option<String>, String>;
    fn read_lines(&mut self, n: i32) -> Result<Vec<String>, String>;
}

// 2. Wrap a BufReader
#[derive(ExternalPtr)]
pub struct TextReader {
    inner: BufReader<Cursor<Vec<u8>>>,
}

impl TextReader {
    pub fn from_string(s: &str) -> Self {
        Self {
            inner: BufReader::new(Cursor::new(s.as_bytes().to_vec())),
        }
    }
}

// 3. Implement the adapters
#[miniextendr]
impl RReader for TextReader {
    fn read_bytes(&mut self, n: i32) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; n as usize];
        let read = self.inner.read(&mut buf).map_err(|e| e.to_string())?;
        buf.truncate(read);
        Ok(buf)
    }

    fn read_all(&mut self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        self.inner.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    fn read_string(&mut self) -> Result<String, String> {
        let mut s = String::new();
        self.inner.read_to_string(&mut s).map_err(|e| e.to_string())?;
        Ok(s)
    }
}

#[miniextendr]
impl RLineReader for TextReader {
    fn read_line(&mut self) -> Result<Option<String>, String> {
        let mut line = String::new();
        match self.inner.read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_) => {
                // Trim trailing newline
                while line.ends_with('\n') || line.ends_with('\r') {
                    line.pop();
                }
                Ok(Some(line))
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn read_lines(&mut self, n: i32) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();
        for _ in 0..n {
            match self.read_line()? {
                Some(line) => lines.push(line),
                None => break,
            }
        }
        Ok(lines)
    }
}

// 4. Factory function
#[miniextendr]
fn text_reader(content: &str) -> TextReader {
    TextReader::from_string(content)
}

// Registration is automatic via #[miniextendr].
```

**Usage in R:**
```r
reader <- text_reader("line1\nline2\nline3\n")
reader$read_line()   # "line1"
reader$read_lines(2) # c("line2", "line3")

# Or read raw bytes
reader2 <- text_reader("hello world")
reader2$read_bytes(5)  # raw vector for "hello"
reader2$read_string()  # " world"
```

---

## Recipe 4: Wrap Comparison for R Sorting

**Goal:** Let R sort custom Rust objects using their natural ordering.

```rust
use miniextendr_api::prelude::*;
use std::cmp::Ordering;

// 1. Your comparable type
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ExternalPtr)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

// 2. Comparison adapter
#[miniextendr]
pub trait RComparable {
    fn cmp_to(&self, other: &Self) -> i32;
    fn eq_to(&self, other: &Self) -> bool;
    fn lt(&self, other: &Self) -> bool;
    fn le(&self, other: &Self) -> bool;
}

#[miniextendr]
impl RComparable for Version {
    fn cmp_to(&self, other: &Self) -> i32 {
        match self.cmp(other) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    fn eq_to(&self, other: &Self) -> bool {
        self == other
    }

    fn lt(&self, other: &Self) -> bool {
        self < other
    }

    fn le(&self, other: &Self) -> bool {
        self <= other
    }
}

// 3. Display for printing
#[miniextendr]
impl Version {
    fn new(major: i32, minor: i32, patch: i32) -> Self {
        Version {
            major: major as u32,
            minor: minor as u32,
            patch: patch as u32,
        }
    }

    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// Registration is automatic via #[miniextendr].
```

**Usage in R:**
```r
v1 <- Version$new(1, 2, 3)
v2 <- Version$new(1, 2, 4)
v3 <- Version$new(2, 0, 0)

v1$cmp_to(v2)  # -1 (v1 < v2)
v1$lt(v3)      # TRUE
v2$to_string() # "1.2.4"

# Sort a list using cmp_to
versions <- list(v3, v1, v2)
order_idx <- sapply(seq_along(versions), function(i) {
  sum(sapply(versions, function(v) versions[[i]]$cmp_to(v) > 0))
})
sorted <- versions[order(order_idx)]
```

---

## Recipe 5: Expose Hash for Deduplication

**Goal:** Use Rust's hashing for fast deduplication in R.

```rust
use miniextendr_api::prelude::*;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[miniextendr]
pub trait RHashable {
    fn hash_code(&self) -> i64;
}

// Blanket impl for any Hash type
impl<T: Hash> RHashable for T {
    fn hash_code(&self) -> i64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as i64
    }
}

// Your type that implements Hash
#[derive(Hash, Clone, ExternalPtr)]
pub struct Record {
    id: String,
    value: i64,
}

#[miniextendr]
impl RHashable for Record {}

#[miniextendr]
impl Record {
    fn new(id: &str, value: i64) -> Self {
        Record { id: id.to_string(), value }
    }
}

// Registration is automatic via #[miniextendr].
```

**Usage in R:**
```r
r1 <- Record$new("a", 1)
r2 <- Record$new("a", 1)
r3 <- Record$new("b", 2)

r1$hash_code() == r2$hash_code()  # TRUE (same content)
r1$hash_code() == r3$hash_code()  # FALSE (different content)

# Deduplicate using hash
records <- list(r1, r2, r3)
hashes <- sapply(records, function(r) r$hash_code())
unique_records <- records[!duplicated(hashes)]
```

---

## Tips

1. **Keep adapters focused** - One trait per capability (iteration, serialization, comparison)
2. **Use `Result<T, String>`** - R handles string errors gracefully
3. **Prefer owned types** - `String` over `&str`, `Vec<T>` over `&[T]` for return values
4. **Document the mapping** - Users need to know what R types become what Rust types
5. **Consider caching** - Wrap expensive operations in `ExternalPtr` for reuse

## See Also

- [ADAPTER_TRAITS.md](../adapter-traits/) - Pattern explanation and constraints
- [SAFETY.md](../safety/) - Thread safety for trait objects
