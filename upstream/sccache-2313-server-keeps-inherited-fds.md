# sccache: the auto-started server keeps inherited file descriptors

- **Upstream**: [mozilla/sccache#2313](https://github.com/mozilla/sccache/issues/2313),
  open, filed 2025-01-08. Same class:
  [mozilla/sccache#2145](https://github.com/mozilla/sccache/issues/2145) (GNU
  make's FIFO jobserver hangs when sccache has to start its server).
- **Our issue**: #1583 (troubleshooting docs and a confirmed workaround).
  We have not commented upstream.

## Impact

When cargo's `rustc-wrapper` is `sccache` (usually set in the user's own
`~/.cargo/config.toml`) and no sccache server is running yet, the first
compile starts one. The server keeps file descriptors it inherited from the
build other than stdin, stdout and stderr. A frontend that waits for the
build's pipes to close then waits forever.

Observed 2026-09-23 while testing PR #1582: `pak::pkg_install()` of a
miniextendr package hung indefinitely. pak runs builds through processx and
waits for EOF. The same installs completed with `RUSTC_WRAPPER=""`. It only
happens when the build has to start the server, so it looks intermittent.

Not yet confirmed: which descriptor stays open (the build's output pipe or
processx's extra poll pipe).

## Workaround

- Start the server before installing: `sccache --start-server` (the
  workaround given in mozilla/sccache#2145).
- Or disable the wrapper for the install: `RUSTC_WRAPPER=""`.

## Closes when

Upstream closes descriptors beyond stdio when daemonizing (#2313), and our
docs name the first fixed sccache release.

## Last checked

2026-09-23: #2313 open, two comments. One reply pointed at earlier related
work (mozilla/sccache#2296); the reporter answered that it still reproduces
on top of tree. No fix linked.
