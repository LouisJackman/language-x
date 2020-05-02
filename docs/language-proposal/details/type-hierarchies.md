# Type Hierarchies

* Final classes and typeclass-like interfaces. No concrete inheritance or
  abstract classes. Discourages "type taxonomy". No dynamic dispatch; acheive
  runtime polymorphism via Sylan enums, a.k.a. "sum types" or "discriminated
  unions".
* `super` is to disambiguate which interface's method to delegate to, to
  delegate to the method that the current one overrides, or to fallback to the
  auto-generated constructor or other auto-generated method like `hashcode` or
  `equals` in user-defined constructors and methods. It does not deal with
  concrete class inheritance.
* Interfaces can provide default method implementations, which can be overridden
  in implementing classes.
* All implementations of interface methods must state `override`, whether they
  override a default implementation or not. This is to allow additions of new
  defaults while remaining backwards compatible with implementor's classes.
* There is no method resolution order such as Dylan- and Python 3-inspired MRO.
  Dynamic dispatch does not exist, since all runtime polymorphism should be done
  via enums instead. Interfaces are more like Haskell typeclasses than Java's
  interfaces in this regard. The ability to statically extend types with new
  interface implementations in different packages covers a lot of
  dynamic-dispatches's use cases.