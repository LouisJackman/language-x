# Type Inference

* Only nominal types, no structural types. While it would be nice to mandate
  that all type inference should be derivable from inner expressions outwards
  without going the other way, that isn't possible due to constructs like
  immediately-invoked lambdas or lambdas with parameters being passed as a
  parameter typed as a lambda that takes compatible but more precise types.
* Once we allow that, there's no reason not to allow LHS type inference to flow
  from the RHS as long as it only flows to declarations within a function,
  lambda, or method body and does not escape it.
* Sylan is more opinionated than some statically-typed functional languages on
  type inference; all function-local variables _must_ infer types, whereas
  declared items in classes and packages must always be explicitly typed. This
  includes package functions, class methods, constructors, and class fields.
  There's no choice a developer must make about whether or not to use type
  inference. Also, all `var`s must infer types whereas `final`s must spell
  them out explicitly.
* This ensures APIs and program structure is rigid and explicitly typed while
  expressions are concise and with little boilerplate.
* This philosophy extends to lambdas; as lambdas are used locally and not for
  definitions (which use declared functions and methods), lambda expressions
  also only allow inferred types (which is why type parameter syntax is not
  supported).
