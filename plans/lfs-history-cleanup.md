# Git LFS History Cleanup

## Goal

Remove all historical copies of `vendor.tar.xz` from git objects across all branches.
Currently 54 versions totaling ~408 MB bloat the repo.

`.gitattributes` already tracks `*.tar.xz` and `*.tar.gz` via LFS (added in PR #23).
This plan rewrites history so past commits also use LFS pointers.

## Current State

- `.gitattributes` tracks `*.tar.xz` and `*.tar.gz` via LFS (PR #23)
- Only file affected: `rpkg/inst/vendor.tar.xz`
- 54 historical versions, ~408 MB total in git objects
- Current version: ~10 MB (xz-compressed vendor deps for CRAN offline builds)

## LFS Setup Details

`.gitattributes` (already in repo):
```
*.tar.xz filter=lfs diff=lfs merge=lfs -text
*.tar.gz filter=lfs diff=lfs merge=lfs -text
```

GitHub LFS quota: 1 GB storage / 1 GB bandwidth (free tier).
At ~10 MB per vendor.tar.xz version, this gives ~100 versions before hitting storage limits.
Bandwidth resets monthly; each clone/fetch downloads only the checked-out version.

## Prerequisites

- PR #23 merged (adds `.gitattributes`)
- All collaborators notified — they will need to re-clone after the force push
- No open PRs with in-flight work (or rebase them after)

## Steps

```bash
# 1. Fresh clone (clean state, no local refs to confuse filter)
git clone https://github.com/A2-ai/miniextendr.git miniextendr-lfs-cleanup
cd miniextendr-lfs-cleanup
git lfs install

# 2. Migrate all tarballs to LFS across all branches and tags
git lfs migrate import --include="*.tar.xz,*.tar.gz" --everything

# 3. Verify
git lfs ls-files --all | wc -l  # should be small (deduplicated)
du -sh .git/lfs/objects/        # LFS objects
du -sh .git/objects/            # should be much smaller now

# 4. Force push all refs
git push --force --all
git push --force --tags

# 5. Server-side GC (GitHub does this automatically, but can take time)
#    Old objects will be garbage collected eventually.

# 6. All collaborators re-clone
#    git clone https://github.com/A2-ai/miniextendr.git
```

## Rollback

If something goes wrong, the old refs are still on GitHub until GC runs.
GitHub support can restore force-pushed refs within ~90 days.
