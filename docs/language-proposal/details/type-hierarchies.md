# Type Hierarchies

* Final classes and typeclass-like interfaces. No concrete inheritance or
  abstract classes. Discourages "type taxonomy". No dynamic dispatch; acheive
  runtime polymorphism via Sylan enums, a.k.a. "sum types" or "discriminated
  unions".
* `super` is to either disambiguate which interface's method to delegate to, or
  to fallback to the auto-generated constructor or other auto-generated method
  like `hashcode` or `equals` in user-defined constructors and methods. It does
  not deal with concrete class inheritance.
* Interfaces can provide default method implementations, which can be overridden
  in implementing classes. `override` on a method makes such overrides clear.
  Method shadowing does not exist; if it has the same name, it must override,
  and the developer must explicitly state that with `override`.
* There is no method resolution order such as Dylan- and Python 3-inspired MRO.
  Dynamic dispatch does not exist, since all runtime polymorphism should be done
  via enums instead. Interfaces are more like Haskell typeclasses than Java's
  interfaces in this regard. The ability to statically extend types with new
  interface implementations in different packages covers a lot of
  dynamic-dispatches's use cases.