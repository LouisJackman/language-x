# Accessibility Levels

* Private, internal, and public.
* Private is the default, no keyword required.
* The `internal` keyword exposes an item to its parent, but only within its
  module.
* The `public` keyword exposes an item to its parent, available from outside the
  module too.
* Classes, interfaces, functions, packages, final package-level bindings,
  methods, fields, enum variants, and constructors can all have accessibility
  modifiers.
* The modifiers go after the item keyword, e.g. `fun` or `class`, but before its
  name. For constructors, they go after the class name but before their parameter
  lists.