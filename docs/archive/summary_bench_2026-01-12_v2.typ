#let horizontalrule = line(start: (25%,0%), end: (75%,0%))

#show terms: it => {
  it.children
    .map(child => [
      #strong[#child.term]
      #block(inset: (left: 1.5em, top: -0.4em))[#child.description]
      ])
    .join()
}

#set table(
  inset: 6pt,
  stroke: none
)

#show figure.where(
  kind: table
): set figure.caption(position: top)

#show figure.where(
  kind: image
): set figure.caption(position: bottom)

#let content-to-string(content) = {
  if content.has("text") {
    content.text
  } else if content.has("children") {
    content.children.map(content-to-string).join("")
  } else if content.has("body") {
    content-to-string(content.body)
  } else if content == [ ] {
    " "
  }
}
#let conf(
  title: none,
  subtitle: none,
  authors: (),
  keywords: (),
  date: none,
  abstract-title: none,
  abstract: none,
  thanks: none,
  cols: 1,
  margin: (top: 2em, left: 1em, right: 2em, bottom: 2.2em),
  // margin: (x: 1.25in, y: 1.25in),
  // paper: "us-letter",
  paper: "a5",
  lang: "en",
  region: "US",
  font: none,
  fontsize: 11pt,
  mathfont: none,
  codefont: none,
  linestretch: 1,
  sectionnumbering: none,
  linkcolor: none,
  citecolor: none,
  filecolor: none,
  pagenumbering: "1 of 1",
  doc,
) = {
  set document(
    title: title,
    keywords: keywords,
  )
  set document(
      author: authors.map(author => content-to-string(author.name)).join(", ", last: " & "),
  ) if authors != none and authors != ()
  set page(
    paper: paper,
    margin: margin,
    numbering: pagenumbering,
    columns: cols
  )

  set par(
    justify: true,
    leading: linestretch * 0.65em
  )
  set text(lang: lang,
           region: region,
           size: fontsize)

  set text(font: font) if font != none
  show math.equation: set text(font: mathfont) if mathfont != none
  show raw: set text(font: codefont) if codefont != none

  set heading(numbering: sectionnumbering)

  show link: set text(fill: rgb(content-to-string(linkcolor))) if linkcolor != none
  show ref: set text(fill: rgb(content-to-string(citecolor))) if citecolor != none
  show link: this => {
    if filecolor != none and type(this.dest) == label {
      text(this, fill: rgb(content-to-string(filecolor)))
    } else {
      text(this)
    }
  }

  block(below: 1em, width: 100%)[
    #if title != none {
      align(center, block[
          #text(weight: "bold", size: 1.5em)[#title #if thanks != none {
              footnote(thanks, numbering: "*")
              counter(footnote).update(n => n - 1)
            }]
          #(
            if subtitle != none {
              parbreak()
              text(weight: "bold", size: 1.25em)[#subtitle]
            }
           )])
    }

    #if authors != none and authors != [] {
      let count = authors.len()
      let ncols = calc.min(count, 3)
      grid(
        columns: (1fr,) * ncols,
        row-gutter: 1.5em,
        ..authors.map(author => align(center)[
          #author.name \
          #author.affiliation \
          #author.email
        ])
      )
    }

    #if date != none {
      align(center)[#block(inset: 1em)[
          #date
        ]]
    }

    #if abstract != none {
      block(inset: 2em)[
        #text(weight: "semibold")[#abstract-title] #h(1em) #abstract
      ]
    }
  ]

  doc
}
#show: doc => conf(
  abstract-title: [Abstract],
  pagenumbering: "1",
  cols: 1,
  doc,
)


= Benchmark Summary - 2026-01-12 (v2)
<benchmark-summary---2026-01-12-v2>
This document summarizes benchmark results for miniextendr after recent
bug fixes.

== Table of Contents
<table-of-contents>
- #link(<gc-protection-mechanisms>)[GC Protection Mechanisms]
- #link(<ppsize-analysis>)[PPSize Analysis]
- #link(<refcountedarena-vs-raw-preserve>)[RefCountedArena vs Raw R\_PreserveObject]
- #link(<ffi-and-r-interop>)[FFI and R Interop]
- #link(<type-conversions>)[Type Conversions]
- #link(<memory-and-allocation>)[Memory and Allocation]
- #link(<altrep>)[ALTREP]
- #link(<externalptr>)[ExternalPtr]
- #link(<string-operations>)[String Operations]
- #link(<worker-thread>)[Worker Thread]
- #link(<miscellaneous>)[Miscellaneous]

#horizontalrule

== GC Protection Mechanisms
<gc-protection-mechanisms>
=== Key Findings
<key-findings>
R's protect stack (`--max-ppsize`) has these limits:

- #strong[Minimum]: 10,000
- #strong[Default]: 50,000
- #strong[Maximum]: 500,000

#strong[Critical Discovery]: R uses \~30-40 protect slots at
initialization, leaving \~49,960 available in the default configuration.

=== ProtectScope vs Arena Implementations
<protectscope-vs-arena-implementations>
#figure(
  align(center)[#table(
    columns: (28.57%, 8.93%, 8.93%, 10.71%, 10.71%, 10.71%, 10.71%, 10.71%),
    align: (auto,auto,auto,auto,auto,auto,auto,auto,),
    table.header([Implementation], [10k], [50k], [100k], [200k], [300k], [400k], [500k],),
    table.hline(),
    [ProtectScope], [125µs], [N/A], [N/A], [N/A], [N/A], [N/A], [N/A],
    [RefCountedArena
    (BTreeMap)], [791µs], [4.3ms], [8.6ms], [17.8ms], [29.6ms], [#strong[38.9ms]], [75.5ms],
    [HashMapArena], [786µs], [3.7ms], [7.9ms], [19.6ms], [36ms], [45.7ms], [90.2ms],
    [ThreadLocalArena
    (BTreeMap)], [748µs], [4.0ms], [8.6ms], [#strong[19ms]], [#strong[28.8ms]], [#strong[38.8ms]], [#strong[52.6ms]],
    [ThreadLocalHashArena], [#strong[433µs]], [#strong[2.5ms]], [#strong[6.7ms]], [17.6ms], [33.2ms], [46.9ms], [76.2ms],
  )]
  , kind: table
  )

=== Crossover Analysis
<crossover-analysis>
- #strong[\< 150k protections]: ThreadLocalHashArena wins (HashMap O(1)
  operations)
- #strong[\~150-200k]: Crossover point - BTreeMap and HashMap roughly
  equivalent
- #strong[\> 200k protections]: ThreadLocalArena (BTreeMap) wins due to
  better cache locality
- #strong[At 500k]: ThreadLocalArena is 31% faster than
  ThreadLocalHashArena (52.6ms vs 76.2ms)

=== Recommendations
<recommendations>
+ #strong[For \< 50k protections]: Use `ProtectScope` when possible
  (fastest, but limited by ppsize)
+ #strong[For 10k-150k protections]: Use `ThreadLocalHashArena` (fastest
  arena implementation)
+ #strong[For \> 150k protections]: Use `ThreadLocalArena` (BTreeMap) -
  wins at scale due to better cache locality

=== Detailed Protection Benchmarks
<detailed-protection-benchmarks>
==== Single Protection Operations
<single-protection-operations>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [ProtectScope single], [14.9-17.8 ns],
    [RefCountedArena single], [66.8-92.8 ns],
    [ThreadLocal single], [38.8-41.5 ns],
    [ThreadLocalHash single], [149-160 ns],
    [Raw protect/unprotect], [14.2-15.8 ns],
  )]
  , kind: table
  )

==== GC Protection Update (2026-01-12)
<gc-protection-update-2026-01-12>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Median Time],),
    table.hline(),
    [Raw protect/unprotect], [14.2 ns],
    [ProtectScope single], [14.86 ns],
    [OwnedProtect], [15.02 ns],
    [Preserve insert/release], [56.36 ns],
    [Preserve insert/release (unchecked)], [29.51 ns],
    [Raw expensive reference], [46.6 ns],
  )]
  , kind: table
  )

These medians come from `cargo bench --bench gc_protect` on 2026-01-12.

==== Reference Counting (same value)
<reference-counting-same-value>
#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Count], [ProtectScope], [RefCountedArena], [ThreadLocalArena],),
    table.hline(),
    [10], [47.6-50.4 ns], [71-125 ns], [97-100 ns],
    [100], [396-422 ns], [228-277 ns], [465-477 ns],
    [1000], [3.9-3.9 µs], [1.8-1.8 µs], [4.1-4.2 µs],
  )]
  , kind: table
  )

#strong[Note]: RefCountedArena wins for repeated protections of the same
value due to efficient reference counting.

#horizontalrule

== PPSize Analysis
<ppsize-analysis>
=== ProtectScope at ppsize Boundaries
<protectscope-at-ppsize-boundaries>
R's protect stack is shared across all operations. When running after
other benchmarks, available slots decrease.

#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Protections Attempted], [Median Time], [Notes],),
    table.hline(),
    [10,000], [125µs], [Stable],
    [20,000], [248µs], [Stable],
    [30,000], [359µs], [Stable],
    [40,000], [449µs], [Stable],
    [49,000], [584µs], [Near limit],
    [49,500], [573µs], [Near limit],
    [49,900], [594µs], [At limit (\~40 slots used by R init)],
  )]
  , kind: table
  )

#strong[Formula]:
`max_available = 50000 - ~40 (R init) = ~49,960 protections`

=== Arena Performance Across ppsize Range
<arena-performance-across-ppsize-range>
Full comparison at extended scale (median times):

#figure(
  align(center)[#table(
    columns: (14.13%, 18.48%, 15.22%, 19.57%, 23.91%, 8.7%),
    align: (auto,auto,auto,auto,auto,auto,),
    table.header([Protections], [RefCountedArena], [HashMapArena], [ThreadLocalArena], [ThreadLocalHashArena], [Winner],),
    table.hline(),
    [10k], [791µs], [786µs], [748µs], [#strong[433µs]], [TL-Hash],
    [50k], [4.3ms], [3.7ms], [4.0ms], [#strong[2.5ms]], [TL-Hash],
    [100k], [8.6ms], [7.9ms], [8.6ms], [#strong[6.7ms]], [TL-Hash],
    [200k], [17.8ms], [19.6ms], [#strong[19ms]], [17.6ms], [TL-Hash],
    [300k], [29.6ms], [36ms], [#strong[28.8ms]], [33.2ms], [TL-BTree],
    [400k], [#strong[38.9ms]], [45.7ms], [#strong[38.8ms]], [46.9ms], [TL-BTree],
    [500k], [75.5ms], [90.2ms], [#strong[52.6ms]], [76.2ms], [TL-BTree],
  )]
  , kind: table
  )

#strong[Key insight]: HashMap's O(1) operations are faster for small
counts, but at scale (\>200k), BTreeMap's predictable memory layout and
better cache locality overcome the theoretical O(log n) disadvantage.

#horizontalrule

== RefCountedArena vs Raw R\_PreserveObject/R\_ReleaseObject
<refcountedarena-vs-raw-preserve>

This section compares the hash-table based `RefCountedArena` against R's native `R_PreserveObject`/`R_ReleaseObject` API.

=== Critical Finding: R\_ReleaseObject is O(n)
<critical-finding-release-is-on>

R's `R_ReleaseObject` must #strong[scan the entire precious list] to find and remove an object. This makes:
- Single preserve+release: O(1) - fast
- N preserve+release cycles: #strong[O(n²)] total - catastrophically slow at scale

=== Performance Comparison (Protect + Unprotect Cycles)
<performance-comparison-preserve>

#figure(
  align(center)[#table(
    columns: 5,
    align: (auto,auto,auto,auto,auto,),
    table.header([Implementation], [10 objects], [100 objects], [1000 objects], [Scaling],),
    table.hline(),
    [Raw R\_PreserveObject/R\_ReleaseObject], [250 ns], [8.2 µs], [#strong[644 µs]], [O(n²)],
    [ThreadLocal (BTree)], [500 ns], [7.2 µs], [#strong[92 µs]], [O(n log n)],
    [RefCountedArena (BTree)], [552 ns], [8.7 µs], [100 µs], [O(n log n)],
    [HashMapArena], [635 ns], [9.2 µs], [95 µs], [O(n)],
    [ThreadLocalHash], [1.1 µs], [10.8 µs], [113 µs], [O(n)],
  )]
  , kind: table
  )

#strong[At 1000 objects, arena implementations are 6-7x faster than raw R\_PreserveObject/R\_ReleaseObject!]

=== Single Operation Performance
<single-operation-preserve>

#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Operation], [Time], [Notes],),
    table.hline(),
    [`protect_scope_single`], [#strong[13.6 ns]], [Fastest - direct stack push],
    [`raw_preserve_release_unchecked`], [15.7 ns], [Fast for single objects],
    [`raw_preserve_release_single`], [20.5 ns], [Checked variant],
    [`thread_local_single` (BTree)], [35.6 ns], [ThreadLocal + BTreeMap],
    [`btreemap_single`], [100 ns], [RefCell + BTreeMap],
    [`thread_local_hash_single`], [107 ns], [ThreadLocal + HashMap],
    [`refcount_arena_single`], [111 ns], [RefCell + BTreeMap (same as btreemap)],
    [`hashmap_single`], [156 ns], [RefCell + HashMap],
  )]
  , kind: table
  )

=== Why RefCountedArena Wins at Scale
<why-refcountedarena-wins>

+ #strong[Architecture]: RefCountedArena stores SEXPs in a single `VECSXP` that's preserved once via `R_PreserveObject`. Individual SEXPs go into slots.
+ #strong[O(1) Lookup]: A hash table (or BTreeMap) maps SEXP addresses to slot indices. Finding an object to release is O(1) or O(log n), not O(n).
+ #strong[Reference Counting]: Protecting the same SEXP multiple times just increments a counter, no additional storage needed.
+ #strong[Slot Reuse]: When objects are unprotected, their slots are added to a free list for reuse without resizing.

=== R's Precious List Architecture
<r-precious-list-arch>

R's `R_PreserveObject`/`R_ReleaseObject` uses a simple linked list:
- `R_PreserveObject`: O(1) - prepend to list
- `R_ReleaseObject`: #strong[O(n)] - must scan list to find object

This is fine for a handful of long-lived objects, but becomes a bottleneck when protecting many temporary objects.

=== Preserve List (Doubly-Linked List) Performance
<preserve-list-performance>

The `preserve` module uses a doubly-linked list with O(1) insert and O(1) release:

#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Operation], [Time], [Notes],),
    table.hline(),
    [`preserve_count`], [4 ns], [Check current count],
    [`preserve_insert_release_unchecked`], [39 ns], [Single insert+release],
    [`preserve_multiple` (1000)], [34 µs], [Insert 1000, release LIFO],
    [`preserve_release_arbitrary_order` (1000)], [30 µs], [Insert 1000, release any order],
  )]
  , kind: table
  )

#strong[Key insight]: Arbitrary-order release is the same speed as LIFO release - this confirms the O(1) removal of the doubly-linked list.

=== Large Scale Comparison (10k - 500k objects)
<large-scale-comparison>

Full comparison of all arena implementations at scale (median times from 2026-01-12 run):

#figure(
  align(center)[#table(
    columns: 6,
    align: (auto,auto,auto,auto,auto,auto,),
    table.header([Objects], [RefCountedArena], [HashMapArena], [ThreadLocalArena], [ThreadLocalHashArena], [Winner],),
    table.hline(),
    [10k], [806 µs], [738 µs], [729 µs], [#strong[425 µs]], [TL-Hash],
    [20k], [1.65 ms], [1.47 ms], [1.52 ms], [#strong[869 µs]], [TL-Hash],
    [50k], [4.26 ms], [3.67 ms], [3.88 ms], [#strong[2.44 ms]], [TL-Hash],
    [100k], [8.45 ms], [7.80 ms], [8.37 ms], [#strong[5.36 ms]], [TL-Hash],
    [200k], [17.6 ms], [16.8 ms], [17.8 ms], [#strong[15.0 ms]], [TL-Hash],
    [300k], [31.3 ms], [32.6 ms], [#strong[28.3 ms]], [32.0 ms], [TL-BTree],
    [400k], [38.2 ms], [39.0 ms], [#strong[37.1 ms]], [42.1 ms], [TL-BTree],
    [500k], [75.2 ms], [83.4 ms], [#strong[49.5 ms]], [70.2 ms], [TL-BTree],
  )]
  , kind: table
  )

#strong[Key observations]:

+ #strong[ThreadLocalHashArena wins \< 300k]: HashMap O(1) lookup beats BTreeMap O(log n)
+ #strong[ThreadLocalArena (BTreeMap) wins \> 300k]: Cache locality overcomes theoretical disadvantage
+ #strong[Crossover point \~250-300k]: Where BTreeMap's contiguous memory layout starts winning
+ #strong[At 500k]: ThreadLocalArena is 30% faster than ThreadLocalHashArena (49.5ms vs 70.2ms)

#strong[Why BTreeMap wins at extreme scale]:
- BTreeMap nodes are contiguous in memory, better cache utilization
- HashMap has random memory access patterns, more cache misses at scale
- The O(log n) vs O(1) difference is dwarfed by memory access patterns

=== Reference Counting: Same Value Protected N Times
<reference-counting-same-value>

When the same SEXP is protected multiple times, reference counting avoids duplicate storage:

#figure(
  align(center)[#table(
    columns: 5,
    align: (auto,auto,auto,auto,auto,),
    table.header([Implementation], [N=10], [N=100], [N=1000], [Notes],),
    table.hline(),
    [ThreadLocal (BTree)], [97 ns], [442 ns], [#strong[3.87 µs]], [Best for repeated protects],
    [RefCountedArena (BTree)], [143 ns], [308 ns], [1.84 µs], [Slightly faster ref counting],
    [ProtectScope], [51 ns], [411 ns], [3.92 µs], [No dedup, wastes stack slots],
    [HashMapArena], [255 ns], [1.05 µs], [9.25 µs], [HashMap lookup overhead],
    [ThreadLocalHash], [33.7 µs], [34.6 µs], [38.7 µs], [#strong[20x slower than BTree!]],
  )]
  , kind: table
  )

#strong[Key insight]: BTreeMap-based arenas are dramatically faster for reference counting the same value. ThreadLocalHashArena has ~20x overhead due to hash computation on every protect call.

=== Scale Test Results (vs Raw R\_PreserveObject)
<scale-test-results>

At larger scales, the O(n²) cost of raw preserve/release becomes prohibitive:

#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Objects], [Raw R\_PreserveObject], [BTreeMap Arena], [Speedup],),
    table.hline(),
    [100], [8.2 µs], [5.6 µs], [1.5x],
    [500], [167 µs], [\~30 µs], [#strong[5.6x]],
    [1000], [644 µs], [60 µs], [#strong[10.7x]],
    [2000], [3.82 ms], [\~120 µs], [#strong[32x]],
  )]
  , kind: table
  )

#strong[O(n²) confirmed]: Doubling N roughly quadruples time for raw R\_PreserveObject/R\_ReleaseObject.

#horizontalrule

== FFI and R Interop
<ffi-and-r-interop>
=== Basic R FFI Calls
<basic-r-ffi-calls>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [Scalar integer], [9.3-12.7 ns],
    [Scalar logical], [3.6-4.2 ns],
    [Scalar real], [9.6-12.4 ns],
    [xlength], [6.9-7.4 ns],
    [INTEGER\_ELT], [7.4-8.5 ns],
    [REAL\_ELT], [7.5-9.0 ns],
    [INTEGER\_PTR], [7.5-8.7 ns],
  )]
  , kind: table
  )

=== Vector Allocation
<vector-allocation>
#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Type], [Size], [Time],),
    table.hline(),
    [INTSXP], [1], [5.9-14.0 ns],
    [INTSXP], [256], [82-118 ns],
    [INTSXP], [4096], [677-931 ns],
    [REALSXP], [1], [6.2-10.3 ns],
    [REALSXP], [4096], [729ns-1.2µs],
    [RAWSXP], [1], [7.1-9.4 ns],
    [RAWSXP], [4096], [226-288 ns],
  )]
  , kind: table
  )

=== Checked vs Unchecked FFI
<checked-vs-unchecked-ffi>
#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Operation], [Checked], [Unchecked],),
    table.hline(),
    [Scalar integer], [7.3-8.9 ns], [6.0-9.7 ns],
    [xlength], [9.0-9.2 ns], [6.6-6.9 ns],
    [Alloc vector (256)], [75-97 ns], [41-136 ns],
  )]
  , kind: table
  )

#strong[Conclusion]: Checked FFI adds \~1-2 ns overhead per call,
negligible for most use cases.

#horizontalrule

== Type Conversions
<type-conversions>
=== Rust to R (into\_r)
<rust-to-r-into_r>
#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Type], [Size=1], [Size=256], [Size=65536],),
    table.hline(),
    [`i32`], [11.9-14.1 ns], [69-116 ns], [9.5-16µs],
    [`f64`], [7.5-15.4 ns], [172-225 ns], [6.5-36µs],
    [`u8`], [11.7-13.7 ns], [34-45 ns], [0.9-5.9µs],
    [`bool`], [3.5-3.7 ns], [-], [-],
    [`String`], [90-105 ns], [11.7-12.4µs], [3.5-4.2ms],
    [`&str`], [46-63 ns], [4.7-4.9µs], [1.2-1.3ms],
    [`Option<i32>` (no NA)], [34-36 ns], [164-239 ns], [26.6-38µs],
    [`Option<i32>` (50% NA)], [35.5-36.8 ns], [170-173
    ns], [27.0-27.1µs],
  )]
  , kind: table
  )

=== R to Rust (from\_r)
<r-to-rust-from_r>
#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Type], [Size=1], [Size=256], [Size=65536],),
    table.hline(),
    [`i32` slice], [20.6-21.1 ns], [20.7-20.9 ns], [20.7-22.1 ns],
    [`f64` slice], [20.6-20.9 ns], [20.6-21.1 ns], [20.6-20.9 ns],
    [`u8` slice], [20.6-21.6 ns], [20.7-21.6 ns], [20.7-21.2 ns],
    [Scalar `i32`], [24.8-33.0 ns], [-], [-],
    [Scalar `f64`], [24.1-30.7 ns], [-], [-],
    [`String` (UTF-8)], [40.4-41.9 ns], [-], [-],
    [`String` (Latin-1)], [250ns-13µs], [-], [-],
  )]
  , kind: table
  )

#strong[Key Insight]: Slice access is essentially zero-copy (\~21 ns
regardless of size).

=== Type Coercion
<type-coercion>
#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Conversion], [R coerce], [Rust coerce],),
    table.hline(),
    [int -\> real (scalar)], [30-40 ns], [23.2-23.5 ns],
    [real -\> int (scalar)], [31-35.5 ns], [23.2-23.4 ns],
    [int -\> real (64k)], [35.7-74µs], [6.0-8.3µs],
    [real -\> int (64k)], [35.7-46.7µs], [53.6-54.2µs],
    [raw -\> int (64k)], [18.0-18.9µs], [4.6-4.9µs],
  )]
  , kind: table
  )

#strong[Recommendation]: Use Rust coercion for numeric conversions -
3-10x faster than R.

#horizontalrule

== Memory and Allocation
<memory-and-allocation>
=== R Allocator vs System Allocator
<r-allocator-vs-system-allocator>
#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Size], [R Allocator], [System Allocator], [Ratio],),
    table.hline(),
    [8 bytes], [62-87 ns], [11-14 ns], [5-6x],
    [64 bytes], [65-96 ns], [13-17 ns], [4-6x],
    [1024 bytes], [144-207 ns], [22-24 ns], [6-9x],
    [8192 bytes], [406-682 ns], [21-21 ns], [19-32x],
    [65536 bytes], [667-1232 ns], [515-525 ns], [1.3x],
  )]
  , kind: table
  )

#strong[Conclusion]: System allocator is significantly faster for small
allocations. R allocator overhead is due to GC tracking.

=== Zeroed Allocation
<zeroed-allocation>
#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Size], [R Allocator], [System Allocator],),
    table.hline(),
    [8 bytes], [62-78 ns], [11-11 ns],
    [1024 bytes], [122-187 ns], [22-24 ns],
    [65536 bytes], [3.7-5.0µs], [807-821 ns],
  )]
  , kind: table
  )

#horizontalrule

== ALTREP
<altrep>
=== ALTREP vs Plain Vectors
<altrep-vs-plain-vectors>
#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Operation], [Plain], [ALTREP (no expand)], [ALTREP
      (expanded)],),
    table.hline(),
    [INTEGER\_ELT], [9.0-10.3 ns], [187-298 ns], [14.6-18.8µs],
    [DATAPTR], [9.0-10.7 ns], [125-283 ns], [14.2-17.6µs],
    [REAL\_ELT], [9.0-10.2 ns], [166-380 ns], [30.4-38.3µs],
  )]
  , kind: table
  )

#strong[Key Insight]: ALTREP has \~20-30x overhead for element access.
When materialized (expanded), overhead increases to \~1500x due to full
vector creation.

=== ALTREP Iteration
<altrep-iteration>
#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Operation], [No expansion], [After 2
      expansions], [After 4 expansions],),
    table.hline(),
    [INTEGER\_ELT iteration], [375-776 ns], [403-571 ns], [547-738 ns],
    [xlength], [260-298 ns], [299-355 ns], [422-523 ns],
  )]
  , kind: table
  )

#horizontalrule

== ExternalPtr
<externalptr>
=== Creation and Access
<creation-and-access>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [Create (small payload)], [169-203 ns],
    [Create (medium payload)], [221-271 ns],
    [Create (large payload)], [208-766 ns],
    [Access as ref], [3.75-3.83 ns],
    [Access as ptr], [3.73-4.14 ns],
    [Deref], [3.55-3.88 ns],
    [as\_sexp], [0.002-0.015 ns],
    [get\_tag], [3.65-4.02 ns],
    [set\_protected], [15.4-17.5 ns],
  )]
  , kind: table
  )

=== Type-Erased Operations
<type-erased-operations>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [erased\_is (hit)], [117.5-118.3 ns],
    [erased\_is (miss)], [118.2-119.6 ns],
    [erased\_downcast\_ref (hit)], [123.4-124.8 ns],
    [erased\_downcast\_mut (hit)], [121.4-123.7 ns],
  )]
  , kind: table
  )

=== Baseline Comparisons
<baseline-comparisons>
#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Operation], [ExternalPtr], [Box (Rust)],),
    table.hline(),
    [Small payload], [169-203 ns], [13.0-13.7 ns],
    [Medium payload], [221-271 ns], [24.1-24.4 ns],
    [Large payload], [208-766 ns], [1.75-1.93µs],
  )]
  , kind: table
  )

#strong[Note]: ExternalPtr creation is \~10-15x slower than Box for
small payloads due to R object creation overhead.

#horizontalrule

== String Operations
<string-operations>
=== String Conversion Performance
<string-conversion-performance>
#figure(
  align(center)[#table(
    columns: 4,
    align: (auto,auto,auto,auto,),
    table.header([Operation], [Short (1 char)], [Medium (256
      char)], [Long (65536 char)],),
    table.hline(),
    [mkCharLen], [8.3-8.8 ns], [226-244 ns], [61-66µs],
    [from\_r CStr], [15.3-16.2 ns], [101-102 ns], [1.2-1.2µs],
    [Translate (UTF-8)], [7.3-7.6 ns], [-], [-],
    [Translate (Latin-1)], [208-389 ns], [-], [-],
  )]
  , kind: table
  )

=== String Interning
<string-interning>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [Empty string (R\_BlankString)], [0.003-0.015 ns],
    [Empty string (mkCharLen)], [41-194 ns],
  )]
  , kind: table
  )

#strong[Recommendation]: Use `R_BlankString` for empty strings -
effectively free.

#horizontalrule

== Worker Thread
<worker-thread>
=== Thread Dispatch Overhead
<thread-dispatch-overhead>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [run\_on\_worker (no R)], [3.2-6.3µs],
    [run\_on\_worker (with R thread)], [6.0-9.9µs],
    [with\_r\_thread (main)], [11.5-16.0 ns],
  )]
  , kind: table
  )

#strong[Key Insight]: Worker thread dispatch adds \~3-6µs overhead. Main
thread R access is essentially free.

#horizontalrule

== Miscellaneous
<miscellaneous>
=== List Operations
<list-operations>
#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Operation], [Size], [Time],),
    table.hline(),
    [Derive into\_list (named)], [-], [125-278 ns],
    [Derive into\_list (tuple)], [-], [64-93 ns],
    [Derive try\_from\_list (named)], [-], [175-179 ns],
    [Derive try\_from\_list (tuple)], [-], [53-56 ns],
    [List get by index], [-], [33-34 ns],
    [List get by name (first)], [-], [52-54 ns],
    [List get by name (last, 65536 elements)], [-], [52-56µs],
  )]
  , kind: table
  )

=== Factor Operations
<factor-operations>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [Single factor (cached)], [45-68 ns],
    [Single factor (uncached)], [364-435 ns],
    [100 factors (cached)], [4.3-5.9µs],
    [100 factors (uncached)], [37-41µs],
  )]
  , kind: table
  )

=== Unwind Protect
<unwind-protect>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [Direct noop], [0.005-0.049 ns],
    [R\_UnwindProtect noop], [33.3-40.2 ns],
  )]
  , kind: table
  )

#strong[Key Insight]: `R_UnwindProtect` adds \~33-40 ns overhead per
call.

=== Trait ABI (Cross-Package)
<trait-abi-cross-package>
#figure(
  align(center)[#table(
    columns: 2,
    align: (auto,auto,),
    table.header([Operation], [Time],),
    table.hline(),
    [mx\_query\_vtable], [0.82-0.85 ns],
    [query\_view\_value], [25.1-32.1 ns],
    [view\_value\_only], [24.5-31.1 ns],
    [baseline\_direct], [0.006-0.019 ns],
  )]
  , kind: table
  )

#horizontalrule

== Changes from Previous Run
<changes-from-previous-run>
The benchmarks show consistent results with the previous run. Minor
variations are expected due to system load and micro-benchmark
variability:

- #strong[GC Protection]: ThreadLocalArena (BTreeMap) remains the best
  choice for large-scale operations (\>200k protections)
- #strong[ExternalPtr]: Performance unchanged, access operations remain
  at \~4ns
- #strong[Type Conversions]: Slice access remains zero-copy (\~21ns
  regardless of size)
- #strong[No regressions detected] from recent bug fixes

== Benchmark Coverage Assessment
<benchmark-coverage-assessment>
The current benchmark suite is #strong[comprehensive] and covers:

+ #strong[GC Protection]: All arena implementations, ProtectScope,
  scaling tests
+ #strong[FFI]: All basic operations, checked vs unchecked
+ #strong[Type Conversions]: All primitive types, vectors, options,
  strings
+ #strong[Memory]: Both allocators, all common sizes
+ #strong[ALTREP]: Element access, iteration, expansion
+ #strong[ExternalPtr]: Creation, access, type-erased operations
+ #strong[Strings]: Interning, conversion, encoding
+ #strong[Worker Thread]: Dispatch overhead
+ #strong[Trait ABI]: Cross-package vtable operations

=== Potential Future Benchmarks
<potential-future-benchmarks>
- `#[r_data]` sidecar field access (new feature)
- Derive macro code generation overhead
- R6/S3/S4 class wrapper overhead comparison

#horizontalrule

== Summary Recommendations
<summary-recommendations>

=== GC Protection (choose based on scale and use case)
<gc-protection-recommendations>

#figure(
  align(center)[#table(
    columns: 3,
    align: (auto,auto,auto,),
    table.header([Scale], [Best Choice], [Notes],),
    table.hline(),
    [\< 50k, LIFO release], [`ProtectScope`], [Fastest (13.6 ns), limited by ppsize],
    [\< 300k, any release order], [`ThreadLocalHashArena`], [O(1) lookup, wins at medium scale],
    [\> 300k], [`ThreadLocalArena` (BTreeMap)], [Best cache locality at extreme scale],
    [Same value repeated], [`ThreadLocalArena` (BTreeMap)], [20x faster ref counting than Hash],
    [Single long-lived object], [Raw `R_PreserveObject`], [Simple, 16ns overhead],
  )]
  , kind: table
  )

#strong[Never use raw `R_PreserveObject`/`R_ReleaseObject` for many objects] - O(n²) scaling!

#strong[ThreadLocal vs RefCell variants]: ThreadLocal versions avoid RefCell borrow overhead, ~2x faster for single operations.
+ #strong[Type Conversions]:
  - Prefer Rust coercion over R's `Rf_coerceVector` (3-10x faster)
  - Use slice views instead of copying when possible (zero-copy)
+ #strong[Strings]:
  - Use `R_BlankString` for empty strings
  - UTF-8 strings are \~30x faster than Latin-1 (no translation needed)
+ #strong[Memory]:
  - System allocator is 5-20x faster for small allocations
  - Consider using Rust vectors and converting at boundaries
+ #strong[Worker Thread]:
  - Batch operations to amortize thread dispatch overhead (\~3-6µs per
    call)
  - Main thread R access is essentially free (\~12 ns)
+ #strong[ALTREP]:
  - Avoid materializing ALTREP vectors unless necessary
  - Element access has \~20-30x overhead vs plain vectors
