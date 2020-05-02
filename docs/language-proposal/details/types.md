# Types

* Externs and user-defined.
* No difference between them from the user's perspective except for some of them
  having literals baked into the language and externs being predefined by the
  compiler. No Java-like primitive vs object distinction. This is different to
  bindings and functions, which can be `extern` and can _not_ defined by Sylan
  itself but instead by an external artefact.
* Constructors are special; this is done to allow invocable-style instantiations
  while avoiding things like statics, needing to detach constructors from class
  definitions, or having a more complicated initialisation syntax.
* `Void` is an actual type, like `()` in Haskell. Defining a function or method
  as returning `void` is a special-case that discards the result of final
  non-void expression and returns the void value instead. Every function,
  method, and lambda returning a value, rather than having "procedures" without
  return values, avoids special-cases when composing invocables in various ways.
* Generics like Rust, as in monomorphic type erasure.
* Support higher-kinded types, but keep an eye on projects like Dotty to see
  what type-soundness issues they encounter. Perhaps implement a more restricted
  version of it.
* A "type" is really a value-parameterised package; every field is actually a
  getter function that has been parameterised hard-coded return values.
  Interpreted another way, a package is just a singleton type.
* Setters do not exist due to Sylan being a immutable language.
* Multiversal equality rather than universal equality.
* Non-destructive updates by prefixing objects with a `..` and then invoking
  them like a class constructor. Missing fields get filled in from the old
  object.
* Prefixing an enum variant with `ignorable` allows pattern matching to skip it,
  at the cost of throwing runtime errors if calling code doesn't handle them.
  This is a slight tradeoff of robustness for improved backwards compatibility
  when adding new variants to enums.
