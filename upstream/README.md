# Upstream issues we track

Problems whose cause lives in another project but that affect miniextendr or
packages built with it. Tracking them here keeps the upstream state, our
workaround and our own issue in one place, without commenting upstream by
default.

One file per upstream issue, named `<project>-<number>-<slug>.md`, with:

- **Upstream**: link, state, and when it was filed.
- **Our issue**: the miniextendr issue that tracks our side (docs, workaround, version bump).
- **Impact**: what breaks for us or our users, and when.
- **Workaround**: what to do until upstream fixes it.
- **Closes when**: what has to happen upstream and on our side.
- **Last checked**: date and what the upstream issue looked like then. Update it when you re-check.

Remove the file once upstream has fixed it and our side is done.

When the upstream fix is merged but not released, name the file after the
merged pull request, and say in **Closes when** which release we wait for.

## Index

| File | Project | Upstream | State (last checked) |
| --- | --- | --- | --- |
| [`sccache-2313-server-keeps-inherited-fds.md`](sccache-2313-server-keeps-inherited-fds.md) | mozilla/sccache | #2313 | open (2026-09-23) |
| [`S7-721-external-generic-s3-registration.md`](S7-721-external-generic-s3-registration.md) | RConsortium/S7 | #721 | fixed on main, unreleased (2026-09-24) |
| [`S7-490-unary-ops.md`](S7-490-unary-ops.md) | RConsortium/S7 | #490 | PR open; works on main, unreleased (2026-09-24) |
