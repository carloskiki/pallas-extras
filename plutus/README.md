## Important Things

The crate should evolve with the plutus core version. So the `builtins`, `primitives`,
and `terms` should all be accepted by default, and at runtime it might be possible to
limit the set of builtins/primitives/terms to a specific version.



## BigInt

- malachite: brings in a stupid amount of dependencies for nothing
- num-bigint: allegedly slow, but widely used and maintained.
- rug: looks good

## Fixed point arithmetic

- bigdecimal: uses arbitrarily large ints :/
- fixed: Brings in weird dependencies, and looks poorly made (still maintained).
- fastnum: wtf?? very early but interesting.
