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

# Update rpkg/DESCRIPTION
DESCRIPTION="$ROOT_DIR/rpkg/DESCRIPTION"
if [ -f "$DESCRIPTION" ]; then
    sed -i.bak -E 's/^(Version: )[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?/\1'"$VERSION"'/' "$DESCRIPTION"
    rm -f "$DESCRIPTION.bak"
    echo "  Updated: $DESCRIPTION"
else
    echo "  Warning: $DESCRIPTION not found"
fi

DESCRIPTION="$ROOT_DIR/minirextendr"
if [ -f "$DESCRIPTION" ]; then
    sed -i.bak -E 's/^(Version: )[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?/\1'"$VERSION"'/' "$DESCRIPTION"
    rm -f "$DESCRIPTION.bak"
    echo "  Updated: $DESCRIPTION"
else
    echo "  Warning: $DESCRIPTION not found"
fi

echo ""
echo "Done! Verify changes with:"
echo "  git diff Cargo.toml rpkg/DESCRIPTION minirextendr/DESCRIPTION"
