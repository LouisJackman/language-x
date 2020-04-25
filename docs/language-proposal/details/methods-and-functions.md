# Methods and Functions

* Purposely different from one another; there isn't a UFC mechanism that unifies
  them.
* Both are fully higher-order. Methods carry around their instances with them.
  Lambdas are just functions not bound to a symbol, so they otherwise work in
  the same way.
* Methods, when passed around as values, are just seen as functions that have an
  instance bundled in their closure.
* `::` is used to pick something up without invoking it; by default, with no
  parentheses, Sylan will invoke something with zero-arguments.
* Invocations look like `Class.method::`, `package.function::`, and
  `object.method::`.
* As methods and functions are both higher-order, invoked the same way, and have
  the same type when passed around, there is no real loss of composability from
  being different constructs. They can be composed together easily: `var
  printDouble = Number.double:: # ToString.toString:: # println::`.