# Repository check exposed a conversion-fixture warning

## What was attempted

Run the repository-wide `just check` gate after the documentation and rustdoc
updates.

## What went wrong

The check passed but warned that `StructPreferNative`'s tuple field was never
read.

## Root cause

The fixture exists to verify layout-based `RNativeType` return conversion. The
derive-generated conversion consumes the field through its native layout, so
Rust's dead-code analysis does not see a normal field read.

## Fix

Add a narrowly scoped `#[allow(dead_code)]` to that fixture field with a comment
explaining why the lint is a false positive. No production type or lint level
was weakened.
