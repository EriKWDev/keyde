
# Keyde - Small and Speedy spacial queries
Keyde aims to provide minimal yet fast implementations of spacial query structures.

Currently, keyde provides a:
  - Kd-tree

## "Points"
Keyde provides a `Point` trait that is implemented for arrays of sizes 1 to 4,
tuples of 2 and 3 dimension as well as for all the basic 1D types (u8, i8, isize..)

By enabling optional features such as `glam`, you can get an implementation for that crate's
default Vec3, Vec4, Vec2 and Vec3A types.

Keyde wants to suppoort more linear algebra crates, so feel free to make a PR and add your favorite one.
See `src/point_implementations.rs` for inspiration.

## Kd-tree
Key things that differ keyde's kd-tree implementation from others:
  - No recursion, only iterative implementations
  - No cloning of your data, everything is refered to by indices into your data
  - Provides `KdTreeStrategy` to choose sorting strategy which might help you find a
    creation/querying-strategy that is more optimal for your particular data layout
