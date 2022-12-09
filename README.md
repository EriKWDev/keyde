> WARNING: Currently under development and might be broken at any moment 

## Keyde - A Tiny and Speedy Kd-Tree
I tiny Kd-Tree implementation that is pretty fast.
It only stores a pointer to your items and returns indices into your list
instead of cloning and allocating a bunch.

The main function that needs optimizing is `nearest_within` since it does some
unpredictable allocations as it grows a vec. Still faster than many other implementations though!

I would like to make that into an iterator instead if possible..

Key things that differ keyde from others and make it fast:
  - No recursion, only iterative implementations
  - No sorting at each split, instead only sorting by each dimension once at start of construction
  - No cloning of your data, everything is refered to by indices into your data
