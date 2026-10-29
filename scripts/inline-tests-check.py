#!/usr/bin/env python3
"""Fail on unit tests written inline instead of in a `tests.rs` file.

Usage: python3 scripts/inline-tests-check.py    (or `just inline-tests-check`)

A unit test module is declared `#[cfg(test)] mod tests;` and lives in
`foo/tests.rs` beside `foo.rs` (in `tests.rs` beside a crate root); a second
test module goes under `foo/tests/`, declared from `foo/tests.rs`. Path tools
then tell tests from library code: `tokei -e '**/tests.rs'`,
`rg -g '!**/tests.rs'`, a review diff. rustc's tidy has the same rule.

Flagged in every git-tracked `.rs` file outside `tests.rs` files and `tests/`
directories:

- an inline `#[cfg(test)] mod name { .. }`;
- a `#[cfg(test)] mod name;` not named `tests`;
- a `#[test]` function.

`#[cfg(test)]` functions, statics and `use`s that library code swaps in for
test builds stay where they are. The scaffolding templates are end-user code
and are not checked.
"""

import re
import subprocess
import sys
from pathlib import Path

TEST_CFG = re.compile(r"\s*#\[cfg\((?:test|all\((?:[^\]]*,\s*)?test\s*(?:,[^\]]*)?\))\)\]\s*")
ATTR_OR_COMMENT = re.compile(r"\s*(?:#\[.*\]|//.*)")
MOD = re.compile(r"\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*([{;])")
TEST_FN = re.compile(r"\s*#\[test\]\s*")


def checked(path):
    parts = Path(path).parts
    return not (
        path.startswith("minirextendr/inst/templates/")
        or parts[-1] == "tests.rs"
        or "tests" in parts[1:-1]
    )


def tests_file(path):
    p = Path(path)
    return p.parent / "tests.rs" if p.name in ("lib.rs", "main.rs", "mod.rs") else p.parent / p.stem / "tests.rs"


def problems(path, text):
    lines = text.split("\n")
    out = []
    for i, line in enumerate(lines):
        if TEST_FN.fullmatch(line):
            out.append(f"{path}:{i + 1}: #[test] function; move it to {tests_file(path)}")
            continue
        if not TEST_CFG.fullmatch(line):
            continue
        j = i + 1
        while j < len(lines) and ATTR_OR_COMMENT.fullmatch(lines[j]):
            j += 1
        m = MOD.match(lines[j]) if j < len(lines) else None
        if m is None:
            continue
        name, end = m.groups()
        if end == "{":
            out.append(f"{path}:{j + 1}: inline test module `{name}`; move it to {tests_file(path)}")
        elif name != "tests":
            out.append(f"{path}:{j + 1}: test module `{name}` is not named `tests`")
    return out


def main():
    root = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())
    files = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "*.rs"], cwd=root, text=True
    ).split()
    found = []
    for path in filter(checked, files):
        if (root / path).is_file():
            found += problems(path, (root / path).read_text())
    if found:
        print("inline-tests-check: unit tests belong in tests.rs files:", file=sys.stderr)
        for p in found:
            print(f"  {p}", file=sys.stderr)
        print("Declare `#[cfg(test)] mod tests;` and put the module body in that file; "
              "a second test module goes under the tests/ directory beside it.", file=sys.stderr)
        sys.exit(1)
    print("inline-tests-check: OK")


if __name__ == "__main__":
    main()
