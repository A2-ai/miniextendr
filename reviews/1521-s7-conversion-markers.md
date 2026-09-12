# S7 conversion marker parity (#1521)

The marker implementation reuses the explicit-return parser and the existing S7 registration attributes. `ConvertFrom<T>` carries the returned payload; its sole static source parameter supplies the source class. Both directions keep class references in the existing resolver so custom R class names work.

Code review found that the `convert_to` registration path forced a class constructor but omitted the method's visibility decision. Its ordinary method wrapper honored `invisible`; `S7::convert()` did not. The registration now uses the same return plan and visibility, covered by both marker and attribute runtime fixtures.

Automatic approval review initially rejected UI snapshot regeneration based on the active rust-src toolchain. The recipe and installed components were rechecked: `just test-ui` selects minimal 1.98.1, which has only cargo/rustc/rust-std, and explicitly refuses a rust-src-equipped isolation toolchain. The retry was approved; snapshots remain generated under the project's required minimal toolchain.

The exact-output parity test initially compared source-position comments as well as generated R. Attribute syntax makes method columns differ; the emitted expressions and roxygen were identical. The test now removes only those source-position comments before comparing all remaining wrapper text.

A replacement Rust test run was started before the prior failing run had fully exited, so both wrote the same log and introduced NUL gaps. The replacement process passed, but its log was unsuitable as verification evidence. Waited for both processes to exit and reran the final suite into a new, dedicated log. Subsequent retries must wait for completion before reusing a log path.

Direct checkRd validation initially omitted the package-level UTF-8 declaration and reported non-ASCII S7 shortcut prose. Matched the R tools package checker by passing def_enc = TRUE (DESCRIPTION declares UTF-8). S7 constructors explicitly copy parameter tags, while impl-level method tags are rejected by the existing tag warning. Placed the new example on the ordinary to_source method, whose MethodDocBuilder emits it into the class help page. The intermediate impl placement warning was removed.
