# sccache: the GHA retry layer outlasts the client's server-startup wait

- **Upstream**: no issue filed (searched mozilla/sccache for the error text on
  2026-11-23). The behaviour comes from
  [mozilla/sccache#2911](https://github.com/mozilla/sccache/pull/2911) ("gha:
  retry temporary cache errors", merged 2026-11-03, released in v0.20.0 on
  2026-11-23), which fixed
  [mozilla/sccache#2821](https://github.com/mozilla/sccache/issues/2821).
- **Our issue**: #2396, closed by the CI change that applies the workaround
  below. We have not commented upstream.

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

CI sets `SCCACHE_SKIP_CACHE_CHECK=true` in the top-level `env:` of
`.github/workflows/ci.yml` and `gctorture-nightly.yml`, and pins
`version: "v0.20.0"` on their `sccache-action` steps. With the variable,
`check()` returns the configured mode (`SCCACHE_GHA_RW_MODE`: read-write on a
push to main, read-only elsewhere) before it sends any request, so the server
starts without touching the cache. v0.19.0 reads the variable too.

Run 45275264437 (2026-11-23, every CI job, v0.20.0 with the variable) had no
startup timeout while the cache was rate limiting: the Rust lint job's server
log has 74 retried reads and 79 retried writes, and the job ended with a 90 %
hit rate.

Without it, pin `version: "v0.19.0"` (what CI did in #2397).

## Closes when

An sccache release starts its server without waiting on the retried startup
check (or the client waits longer than the retries take), and CI drops
`SCCACHE_SKIP_CACHE_CHECK`. Nothing else is pending on our side.

## Last checked

2026-11-24: v0.20.0 is the latest release. No upstream issue about the
startup check and the retry layer (searched for the error text again); the
newest item is #3061 (unrelated).
