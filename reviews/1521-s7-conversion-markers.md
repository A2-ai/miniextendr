# S7 conversion marker parity (#1521)

The marker implementation reuses the explicit-return parser and the existing S7 registration attributes. `ConvertFrom<T>` carries the returned payload; its sole static source parameter supplies the source class. Both directions keep class references in the existing resolver so custom R class names work.

Code review found that the `convert_to` registration path forced a class constructor but omitted the method's visibility decision. Its ordinary method wrapper honored `invisible`; `S7::convert()` did not. The registration now uses the same return plan and visibility, covered by both marker and attribute runtime fixtures.

Automatic approval review initially rejected UI snapshot regeneration based on the active rust-src toolchain. The recipe and installed components were rechecked: `just test-ui` selects minimal 1.98.1, which has only cargo/rustc/rust-std, and explicitly refuses a rust-src-equipped isolation toolchain. The retry was approved; snapshots remain generated under the project's required minimal toolchain.
