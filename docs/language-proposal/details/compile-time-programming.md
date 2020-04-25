# Compile-time Programming

* `final` bindings must run the whole expression at compile-time, or fail
  compilation. As they are allowed in non-main packages but not `var`, it means
  loading a compiled module at runtime should incur a fixed, known performance
  overhead.
* `final` bindings are allowed inside function bodies too. In this context,
  their requirement for explicit types is dropped.
* Compile-time expressions shouldn't be limited compared to runtime ones; steal
  this from Jai. Use an interpreter in the compiler if necessary.
* `final` functions will implicitly run at compile-time and fail to compile if
  callsites can't provide their arguments at compile time. This allows a
  runtime function to be changed to a compile-time function and vice-versa
  without breaking API compatibilty.
* Constructors can also be marked as `final` by putting `final` after the class
  name but before the constructor parameter list. This means a type can only be
  constructed at compile-time.
* Calling a function with `!`, e.g. `f!()`, forces it to run at compile-time
  regardless of whether it's final. If it can't run at compile-time,
  compilation fails. It isn't seen as redundent on a `final` function, because
  the function's finality isn't part of its published API and can be changed as
  mentioned in the previous point.
* Normal functions or parts of them can run at compile-time too, as an
  optimisation, but they and their callers just can't depend on the behaviour.
* The standard `do` function that takes a block basically makes it possible to
  run any code block at compile time with `!` via a unified mechanism.