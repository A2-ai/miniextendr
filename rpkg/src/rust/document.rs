// NOTICE: Any changes to this file, must also be applied to configure.ac's embedded version of this file!
use juice::R_WRAPPERS_DEPS_rpkg as DEPS;
use juice::R_WRAPPERS_PARTS_rpkg as PARTS;

fn main() {
    let dep_flat: Vec<&'static str> = DEPS.concat();
    let roots: [&[&'static str]; 2] = [PARTS, &dep_flat];

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("miniextendr_wrappers.R")
        .unwrap();

    let mut seen = std::collections::HashSet::new();
    for group in roots {
        for &s in group {
            if seen.insert(s) {
                use std::io::Write;
                f.write_all(s.as_bytes()).unwrap();
                f.write_all(b"\n\n").unwrap();
            }
        }
    }
}
