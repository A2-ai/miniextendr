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
