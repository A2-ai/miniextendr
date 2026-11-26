# An abstract S7 class written in R: the parent of the Rust class
# `MxS7HandSquare` (`s7(parent = "MxS7HandShape")` in src/rust/s7_tests.rs,
# #2387). R collates a package without a `Collate` field by file name, so this
# file is sourced before miniextendr-wrappers.R, which defines the subclass.
#
# The subclass's object starts from `S7::S7_object()`, so S7 sets none of the
# parent's stored properties; a dynamic one (a getter) works.
MxS7HandShape <- S7::new_class("MxS7HandShape",
  abstract = TRUE,
  properties = list(
    family = S7::new_property(S7::class_character, getter = function(self) "shape")
  )
)

# A generic with a method on the parent, which the Rust subclass inherits.
mx_hand_shape_kind <- S7::new_generic("mx_hand_shape_kind", "x")
S7::method(mx_hand_shape_kind, MxS7HandShape) <- function(x) "hand-written shape"
