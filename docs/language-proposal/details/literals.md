# Literals

* Numeric types, variable-width by default.
* Machine-width ones with checked overflows with suffixes on literals.
* Boolean literals don't exist; `True` and `False` are just `Boolean` enum
  variants.
* Lambdas.
* Characters with `'`.
* Strings and tainted strings exist, the latter being buildable at runtime and
  the former being only constructable at compile-time.
* Strings with `"` for escaped, non-interpolated strings.
* Strings with `r"` for non-escaped, non-interpolated strings.
* Strings with `$""` for compile-time interpolated strings.
* Combined `r` and `$` for strings that interpolate but do not escape.
* `$package1.function1"string"` passes the string to
  `package1.function1` for interpolation. It doesn't need to return a string
  type. This can be used to implement wrapper string types immune to
  injection, e.g. SQL strings.
* Interpolation with custom function are called _template strings_. They take
  a list of string fragments and a list of values that were interleaved with
  them.
* Templating functions that are final can return strings or tainted strings;
  non-final functions can only return tainted strings.
* Strings with three or more delimiters for multiline strings, regardless of
  string type.
* Example: `$sql" select * from table where item = {item}"`, which is impossible
  to inject if it returns a wrapper type whose constructor is private and
  ensures all interpolated values are escaped. It returns a non-tainted string,
  meaning it can't be dynamically interpolated into other tainted strings.
* To interpolate strings with runtime values with no further processing, use the
  `sylan.lang.tainted` function, e.g.:
  `$tainted"a string with {runtimeValues} in it"`.
* Tainted strings cannot be passed back into interpolation templates that only
  take non-tainted strings;
