// Minimal stub so R's build system produces a shared library.
// All entry points (R_init_*) are defined in Rust via miniextendr_init!().
//
// miniextendr_force_link is a package-independent symbol emitted by
// miniextendr_init!() that forces the linker to pull in the user crate's
// archive member (containing all linkme distributed_slice entries).
extern const char miniextendr_force_link;
const void *miniextendr_anchor = &miniextendr_force_link;
