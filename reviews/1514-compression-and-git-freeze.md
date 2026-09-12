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

The complete network-enabled integration run exposed an obsolete assertion
that expected Cargo.lock registry checksums to be stripped, although the
current implementation deliberately preserves them for directory-source
verification. The assertion and test name now reflect that contract.

A 28-crate clap/serde_json fixture measured compression alone on macOS arm64,
three samples per mode: BSD tar median 5.22 s default versus 0.56 s at level 1,
2,891,852 versus 3,432,220 bytes (about 19% larger). GNU tar on the same host
measured 4.53 s versus 0.10 s, 2,919,172 versus 3,473,312 bytes. These timings
isolate compression on one unchanged vendor tree; they do not conflate a full
vendor pass with the new cache-hit recompression path.

The full suite also caught source_root_wins_over_patch_config: a configured
Git patch makes Cargo metadata report a local path (source=None), which skipped
the explicit source-root override applied to Git sources. Local dependencies
now honor the same pre-prioritized overrides with version checking. The existing
fixture distinguishes source-root and config copies by their actual source bytes.

The external-only integration then exposed bootstrap stubs left in its output:
the old cleanup removed only non-dependency stubs despite promising external
sources only. Bootstrap now returns the exact newly seeded members; this pass
rewrites and removes only those entries, preserving any existing local crate
directories. This avoids broad deletion of caller-visible local-pass outputs.

Inspection also clarified that regenerate_lockfile normally copies the already
vendored lock; it does not always resolve the graph again. Documentation and
comments therefore attribute offline verification to the empty-Cargo-home
regression, not that lock-copy step.

Final validation: just revendor-test-all passes 185 tests with none ignored.
The regular suite (123 tests) also passes with GNU tar selected through PATH,
including every compression preset/offline extraction case. just fmt and all
six sequential Clippy gates pass: the repository recipe, three root CI feature
configurations, full-feature rpkg, and standalone cargo-revendor, all with
-D warnings. Final CLI validation rejects presets in verify/local-only modes,
which do not perform full-pass compression; its regular suite and standalone
Clippy were rerun successfully after that validation change.
