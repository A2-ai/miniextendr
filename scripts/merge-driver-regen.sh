#!/usr/bin/env bash
# Git merge driver "mx-regen" for regenerated files.
#
# Git calls this when both sides of a merge, rebase or cherry-pick changed a
# file whose `merge` attribute is `mx-regen`:
#
#   bash scripts/merge-driver-regen.sh %O %A %B %L %P %S %X %Y
#
# (base, current, other, conflict-marker size, path, and the three conflict
# labels). Git runs merge drivers from the top of the work tree.
#
# Generated files are never merged by hand. For every path except Cargo.lock
# the driver keeps the current side (%A, which during a rebase is the branch
# being rebased onto), reports a clean merge, and records the path in
# `$(git rev-parse --git-dir)/mx-regenerate`, a per-worktree list that
# `just regenerate-merged` reads to rebuild the files from the merged sources.
#
# Cargo.lock is merged as text. If that conflicts only because both sides
# re-stamped the `source = "git+...#<sha>"` pins of the framework crates, the
# conflicting hunks take the current side's pins and the path is recorded for
# re-stamping. Any other conflict is left in the file with the usual markers
# and reported as a conflict, so the rebase stops as it would without the
# driver.
#
# Installed by `just merge-drivers-install`. Without it, git ignores the
# unknown driver name and merges these files with its default text merge.
set -euo pipefail

base=$1 current=$2 other=$3 marker=$4 path=$5
label_base=${6:-base} label_current=${7:-current} label_other=${8:-other}

flag_for_regeneration() {
  local list
  list="$(git rev-parse --git-dir)/mx-regenerate"
  if ! grep -qxF -- "$path" "$list" 2>/dev/null; then
    printf '%s\n' "$path" >> "$list"
  fi
}

# Replace the 40-hex commit pin of every git source line with a placeholder.
normalize_pins() {
  sed -E 's/^(source = "git\+[^"#]*#)[0-9a-f]{40}"/\1<pin>"/' "$1" > "$2"
}

merge_cargo_lock() {
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  cp "$current" "$tmp/current"

  # git merge-file writes the result into its first argument and exits with
  # the number of conflicts.
  if git merge-file --marker-size="$marker" \
      -L "$label_current" -L "$label_base" -L "$label_other" \
      "$current" "$base" "$other"; then
    exit 0
  fi

  normalize_pins "$tmp/current" "$tmp/pinless-current"
  normalize_pins "$base" "$tmp/pinless-base"
  normalize_pins "$other" "$tmp/pinless-other"
  if git merge-file -q "$tmp/pinless-current" "$tmp/pinless-base" "$tmp/pinless-other"; then
    # Only pins conflicted: redo the merge, resolving conflicts to our side.
    cp "$tmp/current" "$current"
    git merge-file --ours "$current" "$base" "$other"
    flag_for_regeneration
    exit 0
  fi

  # A real conflict: leave the markers from the first merge in place.
  exit 1
}

case "$path" in
  Cargo.lock | */Cargo.lock) merge_cargo_lock ;;
  *) flag_for_regeneration ;;
esac
