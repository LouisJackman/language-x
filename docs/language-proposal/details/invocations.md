# Invocations

* Methods, lambdas, functions, classes, and objects can be invoked with `()`,
  except nullary invocables which are invoked by just referring to them
  directly.
* Any of those are referred to "invocables" when used like that.
* Passing a nullary invocable without invoking it is done with the invocable
  handle posfix operator: `::`.
* Invoking a method, lambda, or a function does as one would expect; invoking a
  class constructs an instance; invoking an object allows non-destructive
  updates, which is why just referring to an object directly works, as there's
  no distinction between a no-change update and the identity of an immutable
  object.
* Arguments can have default values.
* Arguments have labels, like Swift. Like Swift, the label is the same as the
  parameter if omitted, and the `_` label allows callers to drop it.
* Two special parameter types exist: variadic and variadic entry. The former
  allows an variable amount of arguments, whereas the latter allows a variable
  amount of `sylan.lang.Entry` arguments with syntactical sugar.  The latter is
  primarily for providing a good syntax for constructing map types that
  user-defined types can use.
* The compiler knows whether an argument is positional or labelled based on the
  `_` label; one parameter can't be both. This means the compiler, if seeing a
  unexpected positional argument with a variable name matching a keyword
  argument, can automatically convert it into a keyword argument.
* Passing `_` for one or more arguments partially applies the invocation,
  returning a new function value with the non-underscore arguments evaluated and
  partially applied to the result. This allows, for example, partially-applying
  a non-destructive object update, partially applying a function, or
  partially-applying the instantiation of a class.
* Passing `_` to a labelled argument transforms it into a partially applied
 . positional argument, to assist with functional operations.
