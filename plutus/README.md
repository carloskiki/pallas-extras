- [github pages doc](https://www.reddit.com/r/rust/comments/195ao81/publishing_documentation_as_github_page/)

## TODOs for failing tests

DropList: If the expected budget is i64::MAX, then we should accept failing evaluation if it fails with exceeded budget.
ReplicateByte: Incorrect cost model, it needs to be `NumBytesCostedAsNumWords` like for `IntegerToByteStringMemory`.

## Version support

- All plutus versions are supported. This currently means `1.0.0` and `1.1.0`.
- All builtins are always available.

- Application of the different ledger versions is currently out of scope.

## Cost Accounting

Cost accounting deviates slightly from the `IntersectMBO/plutus` implementation. Builtin evaluation
errors if a cost model parameter is missing and needed. The reference instead uses prohibitive
costs when the cost model is missing a parameter. This results in the same behavior as prohibitive
costs immediately lead to failure of script execution. Another difference is that we error
immediately if the cost model is missing a cek machine step cost, whereas the reference
only errors when the step is actually needed. We consider this situation degenerate enough to
not support partial cost models for cek machine steps.
