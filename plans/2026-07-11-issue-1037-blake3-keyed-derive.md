# Plan: #1037 — BLAKE3 keyed-hash + derive-key modes

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `feat/1037-blake3-keyed-derive`.

Decision baked in (ledger + issue): one-shots ONLY — `keyed_hash` and
`derive_key`. The streaming `Hasher` (ExternalPtr handle) is explicitly
deferred: file a follow-up issue (`gh issue create`, title "blake3: streaming
Hasher via ExternalPtr handle", body = the issue's streaming paragraph +
reference to #1037/this PR) and link it in the PR body.

## Surface (mirror the existing one-shots at
`miniextendr-api/src/optionals/blake3_impl.rs:50-74`, same doc style, same
`blake3` feature gate)

1. `pub fn blake3_keyed_str(key: &[u8], s: &str) -> String` — hex digest of
   `blake3::keyed_hash(key32, s.as_bytes())`.
2. `pub fn blake3_keyed_bytes(key: &[u8], data: &[u8]) -> Vec<u8>` — raw
   32-byte digest.
3. `pub fn blake3_derive_key(context: &str, key_material: &[u8]) -> Vec<u8>`
   — the 32-byte derived key; rustdoc must carry the domain-separation
   guidance (context is a hardcoded application constant, NOT a secret/salt —
   quote blake3's own docs).
4. Key validation for 1+2: `key.len() == 32` else
   `panic!("blake3 keyed hash requires a 32-byte key, got {} bytes", len)`
   (framework converts to R error; matches the crate's other adapter
   validation style — grep a sibling optionals impl for the idiom).

## rpkg fixtures + tests

5. Extend `rpkg/src/rust/blake3_adapter_tests.rs` (current fns at `:7-21`;
   copy their shape): `blake3_keyed_str(key: Vec<u8>, s: String)`,
   `blake3_keyed_bytes(key: Vec<u8>, data: Vec<u8>)`,
   `blake3_derive_key(context: String, key_material: Vec<u8>)` — thin
   delegations to the api fns. New exports → ×2 install rule.
6. testthat (find the existing blake3 test file via
   `grep -rl blake3 rpkg/tests/testthat/`): add rows using the OFFICIAL
   BLAKE3 test vectors (from the blake3 repo's `test_vectors.json`; the
   vendored crate under the workspace's cargo registry cache or the docs.rs
   page carry them — use the standard vector set: key =
   `"whats the Elvish word for friend"` (32 bytes), context =
   `"BLAKE3 2019-12-27 16:29:52 test vectors context"`, input = the 0..251
   byte pattern truncated per case; pin at least the empty-input and
   1024-byte cases for keyed_hash and derive_key). Also: a 31-byte key
   raises an R error matching `32-byte key`.
7. No SEXP storage across allocations (pure compute) → no gc-stress fixture
   required (#430 rule not triggered).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-api --features blake3 2>&1 > /tmp/1037-api.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
just devtools-test 2>&1 > /tmp/1037-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1037-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Commit regenerated `NAMESPACE`/`man` with the fixtures.

## Must NOT touch

- The existing one-shot fns and their vector helpers (additive only).
- No `Hasher`/ExternalPtr surface (deferred issue).
- No new cargo dependencies (blake3 already carries keyed/derive in its
  default API).

## Done criteria

- Both modes callable from R with official-vector-pinned digests; bad key
  length errors clearly; follow-up streaming issue filed and linked;
  suites + three clippy legs green; `Fixes #1037`.

## Escalation rule

If reality diverges from this plan — the pinned test vectors don't match
(wrong vector provenance), the api fn signatures can't mirror the existing
style — **stop, commit nothing further, and report back. Do not improvise.**
