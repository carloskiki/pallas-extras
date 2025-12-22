## Important Things

The crate should evolve with the plutus core version. So the `builtins`, `primitives`,
and `terms` should all be accepted by default, and at runtime it might be possible to
limit the set of builtins/primitives/terms to a specific version.

- [github pages doc](https://www.reddit.com/r/rust/comments/195ao81/publishing_documentation_as_github_page/)

## Fixed point arithmetic

- bigdecimal: uses arbitrarily large ints :/
- fixed: Brings in weird dependencies, and looks poorly made (still maintained).
- fastnum: wtf?? very early but interesting.
