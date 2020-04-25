# Pattern Matching

* Literals are matched as is.
* Composite types are matched as `Type(getter1, getter2: match2, getter3)`.
* "Getters" (i.e. nullary methods) can be used.
* An identifier by itself is a shorthand for `identifier = identifier`.
* There are no positional matches, only matches by specific names.
* For types not overriding the built-in constructor, destructuring should be
  symmetrical with constructing.
* `..` can be used to omit the rest of the match.
* `as` can bind part of a match to a name, e.g.  `Account(firstName) as
  account`.
* Prefixing an identifier with a dot matches its value rather than binding
  against the identifier, e.g: `Account(firstName: .enteredFirstName, ...)`.
  This only works against single identifiers, not lookups with multiple
  identifiers separated with dots. To use a more complex identifier, or even
  a full expression, either use a guard clause in a switch or select case, or
  put the more complex expression into a temporarly local variable and use
  that.