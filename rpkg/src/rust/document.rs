use juice::R_WRAPPERS_DEPS_rpkg as DEPS;
use juice::R_WRAPPERS_PARTS_rpkg as PARTS;

fn main() {
    let roots = &[PARTS, &DEPS.concat()];
    let mut r_wrapper_output_file = std::fs::OpenOptions::new()
        .append(false)
        .read(false)
        .write(true)
        .truncate(true)
        .create(true)
        .open("miniextendr_wrappers.R")
        .unwrap();
    let mut seen = std::collections::HashSet::new();
    for &group in roots {
        for &s in group {
            if seen.insert(s) {
                use std::io::Write;
                r_wrapper_output_file.write_all(s.as_bytes()).unwrap();
                r_wrapper_output_file.write_all(b"\n\n").unwrap();
            }
        }
    }
}
