# sccache: the GHA retry layer outlasts the client's server-startup wait

- **Upstream**: no issue filed (searched mozilla/sccache for the error text on
  2026-11-23). The behaviour comes from
  [mozilla/sccache#2911](https://github.com/mozilla/sccache/pull/2911) ("gha:
  retry temporary cache errors", merged 2026-11-03, released in v0.20.0 on
  2026-11-23), which fixed
  [mozilla/sccache#2821](https://github.com/mozilla/sccache/issues/2821).
- **Our issue**: #2396. We have not commented upstream.

## Impact

With the GitHub Actions cache backend, the server checks the cache at
startup by reading and writing `.sccache_check` (`check()` in
`src/cache/storage.rs`). Under GitHub cache rate limiting (HTTP 429), v0.20.0
retries those requests up to six times, up to 8 s apart, before the check's
own tolerance code sees the error. The client gives up after 10 s ("Timed out
waiting for server startup. Maybe the remote service is unreachable?"), and
a failed start fails every compile in the job.

Observed 2026-11-23 on the CI runs of #2393, #2382 and #2378: 11 jobs failed at
cargo's first `rustc -vV`. The server log (`SCCACHE_ERROR_LOG`) showed 429s
with `retry-after` 18–23 s on `CreateCacheEntry` / `GetCacheEntryDownloadURL`
and the retry layer's `will retry ... after 1.85s / 2.50s / 5.50s` lines.
v0.19.0 has no retry layer: a rate-limited read is tolerated and a failed
write leaves the server read-only, so it starts at once.

Users who set `RUSTC_WRAPPER=sccache` locally are not affected; this needs
the GHA backend (`SCCACHE_GHA_ENABLED`).

## Workaround

- CI pins `version: "v0.19.0"` on every `sccache-action` step (#2397).
- On v0.20.x, `SCCACHE_SKIP_CACHE_CHECK=true` skips the startup check and
  starts the server in its configured read-write mode.

## Closes when

CI runs a release after v0.19.0 without the startup timeout, with
`SCCACHE_SKIP_CACHE_CHECK=true` or on a release whose startup check does not
outlast the client's wait, and #2396 closes.

## Last checked

2026-11-23: v0.20.0 is the latest release. No upstream issue about the
startup check and the retry layer; the newest issue is #3053 (unrelated).
