# Explicit return class wrapping (#1521)

The initial mutable container walk held a borrow across its exit, so inspecting
or replacing the leaf failed Rust borrow checking. The resolver now validates
through shared borrows, then replaces the validated payload slot in its clone.
Both marker and attribute spellings still produce the same wrapping plan.
