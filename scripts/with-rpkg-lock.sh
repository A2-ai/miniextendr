#!/usr/bin/env bash
# Run a maintainer command with an exact snapshot of its starting rpkg lockfile.
# Used as a just script interpreter, so cleanup also covers failed recipe lines.
# --lock-file supports other lock paths and the non-compiling regression harness.
set -euo pipefail

mx_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mx_lock="$mx_root/rpkg/src/rust/Cargo.lock"
if [[ "${1:-}" == --lock-file ]]; then
    mx_lock="${2:?--lock-file needs a path}"
    shift 2
fi
if [[ $# -eq 0 ]]; then
    echo "usage: with-rpkg-lock.sh [--lock-file PATH] COMMAND [ARG ...]" >&2
    exit 2
fi

mx_backup_dir="$(mktemp -d "${TMPDIR:-/tmp}/miniextendr-lock.XXXXXX")"
mx_had_lock=false
if [[ -e "$mx_lock" || -L "$mx_lock" ]]; then
    cp -pP "$mx_lock" "$mx_backup_dir/Cargo.lock"
    mx_had_lock=true
fi

mx_restore_lock() {
    local status=$?
    trap - EXIT
    if [[ "$mx_had_lock" == true ]]; then
        if ! mv -f "$mx_backup_dir/Cargo.lock" "$mx_lock"; then
            echo "lock restore failed; original retained in $mx_backup_dir" >&2
            [[ $status -ne 0 ]] || status=1
        fi
    elif [[ -e "$mx_lock" || -L "$mx_lock" ]]; then
        # Restore absence without deleting the generated file.
        if ! mv "$mx_lock" "$mx_backup_dir/generated-Cargo.lock"; then
            echo "cannot restore absent lockfile: $mx_lock" >&2
            [[ $status -ne 0 ]] || status=1
        fi
    fi
    # Empty snapshot directories are left in the OS temp area, like the
    # neutral cargo working directories in the justfile. No permanent deletion.
    exit "$status"
}
trap mx_restore_lock EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

"$@"
