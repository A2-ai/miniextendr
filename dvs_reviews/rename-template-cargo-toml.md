# Rename template Cargo.toml files to prevent cargo parsing errors

## Problem

When using miniextendr as a git dependency:

```toml
[dependencies]
miniextendr-api = { git = "https://github.com/CGMossa/miniextendr" }
```

Cargo fails with:

```
error: invalid character `{` in package name: `{{crate_name}}`
 --> minirextendr/inst/templates/monorepo/my-crate/Cargo.toml:2:8
```

This happens because cargo parses **all** `Cargo.toml` files in a git repo when resolving dependencies, and the template files contain mustache-style placeholders (`{{crate_name}}`) that are invalid TOML.

The workspace `exclude` directive doesn't help because it only controls workspace membership, not which files cargo parses.

## Solution

Rename template `Cargo.toml` files to `Cargo.toml.tmpl`:

- `minirextendr/inst/templates/monorepo/Cargo.toml` → `Cargo.toml.tmpl`
- `minirextendr/inst/templates/monorepo/my-crate/Cargo.toml` → `Cargo.toml.tmpl`

## Required follow-up

Update the R template scaffolding code in minirextendr to rename `Cargo.toml.tmpl` → `Cargo.toml` when creating new projects.

## Files changed

```
minirextendr/inst/templates/monorepo/Cargo.toml.tmpl      (renamed from Cargo.toml)
minirextendr/inst/templates/monorepo/my-crate/Cargo.toml.tmpl  (renamed from Cargo.toml)
```
