# Contexts

* A block yielding an applicative type can be prefixed with `with`.
* That block is a _context_.
* Suffixing an expression within that with block, even expressions
  within lambdas and other blocks, invokes `flatMap` and unfolds
  the rest of the block into it.
* It isn't yet clear whether the first expression in the block must
  yield that type or whether it can be inferred. This restriction
  will likely be instated in a later iteration.
