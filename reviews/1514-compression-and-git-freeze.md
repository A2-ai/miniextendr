# #1514: compression levels and honest freeze diagnostics

The request was to expose a faster xz preset without changing the default
archive behavior, and stop warning about the designed Git source-replacement
mode. The actual implementation calls `tar -cJf`; macOS BSD tar links liblzma,
so `XZ_OPT` does not control it. BSD tar requires its `xz:compression-level`
option; GNU tar accepts `--use-compress-program=xz -N`. The new 0–9 option is
explicit, requires `--compress`, and leaves the no-option command unchanged.
An explicit level recompresses a cached vendor tree without another vendor pass.

The first real regression used a pinned local Git repository, froze and
compressed the dependency, then checked the extracted project offline with an
empty Cargo home. It failed: the generated replacement retained the URL but
lost `rev`. A warm Git cache hid that missing source mapping. The fix retains
Cargo vendor's emitted source replacements (including selectors and transitive
or synced sources), while updating the staging directory to the final vendor
path and keeping the framework's local-patch mappings. The regression now
passes with default, 0, 1, and 9 presets; extracted file bytes are identical,
Git dependencies have an informational message, and strict freeze still fails.
It also tests recompression on a cache hit to a new archive path.

The test was tightened to point source replacement at the extracted vendor
copy, rather than the original directory. During the cache-hit refactor an
ambiguous text insertion placed a helper inside the CLI struct, then exposed a
missing Path qualification. Both compile errors were corrected before the
successful formatting and test run.

`--strip-all` was considered separately from compression. Existing regressions
cover source-referenced include files, but trimming changes the shipped file
set. Bootstrap retains its existing defaults; the README explains the explicit
stripping option alongside the compression example.
