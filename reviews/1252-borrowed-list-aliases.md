# #1252: borrowed list elements bypassed the alias guard

Attempted: reject shared R vector storage when a mutable slice is bound through
a list argument and the same vector is also bound through another argument.

Failure: code-generation regressions fail because Vec parameters have no alias
guard, and the existing direct-slice guard disappears in release builds.

Root cause: classification recognizes only direct slice references and Option;
it compares top-level parameter SEXPs, so a list and its element look distinct.

Fix: classify borrowed slice leaves with their native element type and Vec
nesting depth. Check raw leaf identities before conversion, including duplicate
leaves inside a mutable list. Skip NULL, empty vectors, wrong-type leaves, and
shared/shared pairs; collect all conflicting parameter pairs in one diagnostic.
Keep current list elements protected while visiting another list so allocating
ALTREP access cannot collect a live leaf. Protection depth follows type nesting.

The first fixture compile exposed a missing feature gate: explicit worker
fixtures in an otherwise unconditional module must be gated on worker-thread
or worker-default. The R test mirrors that gate, while the detected native
release build exercises the worker fixture.

Runtime validation caught two test-harness mistakes: expect_s3_class has no
info argument, and R-side vector preconditions can reject malformed inputs
before the C wrapper. Keep those normal diagnostics and separately exercise
wrong native types through the raw C wrapper. Disable gctorture before testthat
assertions so only the intended fixture allocations are stressed.

Review added an allocating integer_sequence_list to the GC fixture, with an
invalid owned first argument stopping conversion before any temporary elements
become Rust references. Nonempty worker preflight runs inside
with_r_unwind_protect because ALTREP access can raise R errors; broader worker
conversion unwinding remains tracked by #1302. The separate native scalar
reference alias gap is tracked in audited follow-up #1502.

Final review caught an adjacent GC hazard in RNG wrappers: PutRNGstate can
allocate when .Random.seed is shared, after a successful value or tagged error
has been returned by unwind protection. Root that SEXP across RNG cleanup in
both wrapper strategies. The regression keeps a shared seed while exercising
main-thread and worker alias errors under gctorture.

The #1502 follow-up also covers boxed borrowed-element containers, whose
conversion delegates to Vec. Documentation names native-element owned
containers explicitly rather than implying that an outer Box owns borrowed
element storage.
