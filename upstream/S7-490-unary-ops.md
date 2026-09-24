# S7: unary operators on S7 objects fail inside `Ops.S7_object`

- **Upstream**: [RConsortium/S7#490](https://github.com/RConsortium/S7/pull/490)
  ("fix unary Ops methods: `-`, `+`, `!`"), a pull request, open and approved,
  opened 2024-11-06, last updated 2026-07-27. The bug report
  [RConsortium/S7#531](https://github.com/RConsortium/S7/issues/531) was
  closed on 2025-03-28 as a duplicate of #490.
- **Our issue**: none yet (not filed from the PR that added this note).
  We have not commented upstream, and no upstream post is needed.

## Impact

`-x` (or `+x`, `!x`) on an S7 object errors in S7 0.2.2, whatever method the
package registers: `Ops.S7_object(e1, e2)` forwards `e2` to the S7 operator
generic, and the missing `e2` is evaluated. A method registered with
`class_missing` (the form in #531) and a `class_any` method that checks
`missing(e2)` both fail. PR #1598 documents this as an S7 limitation for
miniextendr's generated `Ops` methods.

Reproduction on R 4.6.1 with S7 0.2.2:

```r
library(S7)
Money <- new_class("Money", properties = list(value = class_double))
method(`-`, list(Money, class_missing)) <- function(e1, e2) Money(value = -e1@value)
m <- Money(value = 5)
print(tryCatch((-m)@value, error = function(e) conditionMessage(e)))
Cash <- new_class("Cash", properties = list(value = class_double))
method(`-`, list(Cash, class_any)) <- function(e1, e2) if (missing(e2)) "unary via class_any" else "binary"
print(tryCatch(-Cash(value = 1), error = function(e) conditionMessage(e)))
```

```
[1] "argument \"e2\" is missing, with no default"
[1] "argument \"e2\" is missing, with no default"
```

Without `tryCatch()` the error reads
`Error in Ops.S7_object(m) : argument "e2" is missing, with no default`.
Binary use (`m - 2`) works.

With S7 main at `bee740f` (0.2.2.9000) the same script prints `[1] -5` and
`[1] "unary via class_any"`, although #490 is still open. `Ops.S7_object()` is
unchanged there; the likely fix is
[RConsortium/S7#621](https://github.com/RConsortium/S7/pull/621) (merged
2026-05-26), which made dispatch work on forwarded missing arguments.

## Workaround

Define the operator the S3 way, as suggested on #490: a method named
`` `-.mypkg::Money` `` registered with `S3method()` in `NAMESPACE`. It takes
precedence over the S7 method for that class (on S7 0.2.2, `NextMethod()` from
it reaches `-.default`, not the S7 method), so it has to handle the binary case
as well: `if (missing(e2)) <unary> else <binary>`. miniextendr does not
generate such methods.

## Closes when

S7 releases a version after 0.2.2 in which unary operators dispatch, and
miniextendr's S7 floor and docs are updated to match.

## Last checked

2026-09-24: #490 open and approved, no new comments since 2026-07-27. The
latest release is still 0.2.2.
