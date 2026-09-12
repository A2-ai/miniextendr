# Serialize return attribute (#1521)

The attribute must change both the value conversion and return-shape analysis.
Otherwise a serialized Self or class return would still receive an R constructor
wrapper, and Result/Option would keep their ordinary boundary handling instead of
matching AsSerialize<Result/Option>. The shared helpers model the converted type
and prepare the call result after unwrapping any outer visibility marker.

The first compile identified a missed TraitMethod initializer in the adapter path;
that initializer and the unit-test helper now explicitly retain serialize = false.

The class-system unit probe initially used an instance receiver for vctrs, which
correctly rejects instance methods because its R values are base vectors. The
probe now uses static methods for that system, while retaining instance methods
for the other five.
