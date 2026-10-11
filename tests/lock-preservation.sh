#!/usr/bin/env bash
# Exact lock preservation, including the real just recipe interpreter wiring.
# Fake cargo/R commands exercise success/failure without compiling or installing.
set -euo pipefail

mx_root="$(cd "$(dirname "$0")/.." && pwd)"
mx_tmp="$(mktemp -d)"
mx_cleanup() {
    local status=$?
    if command -v trash >/dev/null 2>&1; then
        trash "$mx_tmp"
    elif command -v gio >/dev/null 2>&1; then
        gio trash "$mx_tmp"
    else
        echo "No trash utility; fixtures retained at $mx_tmp" >&2
    fi
    exit "$status"
}
trap mx_cleanup EXIT
mkdir -p "$mx_tmp/repo/scripts" "$mx_tmp/repo/rpkg/src/rust" "$mx_tmp/bin" "$mx_tmp/tmp"
cp "$mx_root/scripts/with-rpkg-lock.sh" "$mx_tmp/repo/scripts/"
cp "$mx_root/justfile" "$mx_tmp/repo/"
export TMPDIR="$mx_tmp/tmp"
mx_guard="$mx_tmp/repo/scripts/with-rpkg-lock.sh"
mx_lock="$mx_tmp/repo/rpkg/src/rust/Cargo.lock"
export MX_TEST_LOCK="$mx_lock"
mx_just="$(command -v just)"

# Three distinct versions: index, starting worktree, command-generated.
# There need not be a HEAD commit: the interpreter never reads Git state.
git init -q "$mx_tmp/repo"
printf 'index version\n' > "$mx_lock"
git -C "$mx_tmp/repo" add rpkg/src/rust/Cargo.lock
git -C "$mx_tmp/repo" show :rpkg/src/rust/Cargo.lock > "$mx_tmp/index-before"
cp "$mx_lock" "$mx_tmp/expected"
bash "$mx_guard" bash -c 'printf "generated\n" > "$MX_TEST_LOCK"'
cmp "$mx_tmp/expected" "$mx_lock"
printf 'unstaged dependency edit\n' > "$mx_lock"
cp "$mx_lock" "$mx_tmp/expected"

mx_assert_preserved() {
    cmp "$mx_tmp/expected" "$mx_lock"
    git -C "$mx_tmp/repo" show :rpkg/src/rust/Cargo.lock > "$mx_tmp/index-after"
    cmp "$mx_tmp/index-before" "$mx_tmp/index-after"
}
mx_run() {
    local expected_status=$1 status=0
    shift
    "$@" || status=$?
    [[ $status -eq $expected_status ]] || {
        echo "expected exit $expected_status, got $status" >&2
        exit 1
    }
    mx_assert_preserved
}

mx_run 0 bash "$mx_guard" bash -c 'printf "generated\n" > "$MX_TEST_LOCK"'
mx_run 42 bash "$mx_guard" bash -c 'printf "generated\n" > "$MX_TEST_LOCK"; exit 42'
# Each nested invocation restores its own entry state, then the outer snapshot.
cat > "$mx_tmp/nested.sh" <<'NESTED'
set -euo pipefail
printf 'outer edit\n' > "$MX_TEST_LOCK"
bash "$1" bash -c 'printf "inner edit\n" > "$MX_TEST_LOCK"'
[[ "$(cat "$MX_TEST_LOCK")" == 'outer edit' ]]
NESTED
mx_run 0 bash "$mx_guard" bash "$mx_tmp/nested.sh" "$mx_guard"

# Preserve an initially absent lockfile without permanent deletion.
mv "$mx_lock" "$mx_tmp/saved-lock"
bash "$mx_guard" bash -c 'printf "generated\n" > "$MX_TEST_LOCK"'
[[ ! -e "$mx_lock" ]]
mv "$mx_tmp/saved-lock" "$mx_lock"
mx_assert_preserved

# Fake cargo ignores manifests but rewrites the lock, like a path override.
cat > "$mx_tmp/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
printf 'cargo generated\n' > "$MX_TEST_LOCK"
exit "${MX_TEST_STATUS:-0}"
CARGO
chmod +x "$mx_tmp/bin/cargo"
export PATH="$mx_tmp/bin:$PATH"
cd "$mx_tmp/repo"
mx_run 0 "$mx_just" check
MX_TEST_STATUS=23 mx_run 23 "$mx_just" check
mx_run 0 "$mx_just" test-rpkg
MX_TEST_STATUS=24 mx_run 24 "$mx_just" expand-rpkg

# Stub only nested just calls, keeping the actual tarball recipe/interpreter.
# The command must see vendor's lock, while the caller gets its pre-vendor file.
cat > "$mx_tmp/bin/just" <<'JUST'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    r-cmd-install) printf 'installed\n' > "$MX_TEST_LOCK" ;;
    vendor) printf 'vendored\n' > "$MX_TEST_LOCK" ;;
    *) echo "unexpected nested recipe: $1" >&2; exit 1 ;;
esac
JUST
cat > "$mx_tmp/bin/R" <<'R'
#!/usr/bin/env bash
[[ "$(cat "$MX_TEST_LOCK")" == vendored ]] || exit 1
exit "${MX_TEST_STATUS:-0}"
R
chmod +x "$mx_tmp/bin/just" "$mx_tmp/bin/R"
mx_run 0 "$mx_just" r-cmd-build
MX_TEST_STATUS=25 mx_run 25 "$mx_just" r-cmd-build

echo 'lock preservation: all cases passed (no cargo/R builds)'
