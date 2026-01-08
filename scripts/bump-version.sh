#!/usr/bin/env bash
#
# Bump version across all version files in the repository.
#
# Usage: ./scripts/bump-version.sh <version>
#   e.g., ./scripts/bump-version.sh 0.2.0
#
# This updates:
#   - Cargo.toml (workspace.package.version)
#   - minirextendr/DESCRIPTION (Version:)
#   - rpkg/DESCRIPTION (Version:)
#   - tests/cross-package/producer.pkg/DESCRIPTION (Version:)
#   - tests/cross-package/consumer.pkg/DESCRIPTION (Version:)
#   - tests/cross-package/shared-traits/Cargo.toml (version)
#
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "  e.g., $0 0.2.0"
    exit 1
fi

VERSION="$1"

# Validate version format (semver: X.Y.Z or X.Y.Z.9000 for R dev versions)
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?$'; then
    echo "Error: Version must be in format X.Y.Z or X.Y.Z.W (e.g., 0.2.0 or 0.2.0.9000)"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Bumping version to $VERSION"

# Update Cargo.toml workspace version
CARGO_TOML="$ROOT_DIR/Cargo.toml"
if [ -f "$CARGO_TOML" ]; then
    # Match: version = "X.Y.Z" under [workspace.package]
    sed -i.bak -E 's/^(version = ")[0-9]+\.[0-9]+\.[0-9]+(")/\1'"$VERSION"'\2/' "$CARGO_TOML"
    rm -f "$CARGO_TOML.bak"
    echo "  Updated: $CARGO_TOML"
else
    echo "  Warning: $CARGO_TOML not found"
fi

# Function to update DESCRIPTION file
update_description() {
    local desc_file="$1"
    if [ -f "$desc_file" ]; then
        sed -i.bak -E 's/^(Version: )[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?/\1'"$VERSION"'/' "$desc_file"
        rm -f "$desc_file.bak"
        echo "  Updated: $desc_file"
    else
        echo "  Warning: $desc_file not found"
    fi
}

# Function to update Cargo.toml version (for non-workspace crates)
update_cargo_toml() {
    local cargo_file="$1"
    if [ -f "$cargo_file" ]; then
        sed -i.bak -E 's/^(version = ")[0-9]+\.[0-9]+\.[0-9]+(")/\1'"$VERSION"'\2/' "$cargo_file"
        rm -f "$cargo_file.bak"
        echo "  Updated: $cargo_file"
    else
        echo "  Warning: $cargo_file not found"
    fi
}

# Update all R package DESCRIPTION files
update_description "$ROOT_DIR/rpkg/DESCRIPTION"
update_description "$ROOT_DIR/minirextendr/DESCRIPTION"
update_description "$ROOT_DIR/tests/cross-package/producer.pkg/DESCRIPTION"
update_description "$ROOT_DIR/tests/cross-package/consumer.pkg/DESCRIPTION"

# Update standalone Cargo.toml files (not part of workspace)
update_cargo_toml "$ROOT_DIR/tests/cross-package/shared-traits/Cargo.toml"

echo ""
echo "Done! Verify changes with:"
echo "  git diff Cargo.toml rpkg/DESCRIPTION minirextendr/DESCRIPTION tests/cross-package/*/DESCRIPTION tests/cross-package/shared-traits/Cargo.toml"
