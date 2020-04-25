# Items and Expressions

* Items declare rigid, static program structure.
* Expressions express computations and are obviously Turing complete.
* Items can contain expressions, i.e. a method item heading a method body that
  contains expressions.
* Expressions cannot contain items, with one exception: let bindings within
  lambdas expressions.
* Grouped expressions with parentheses and lambda literals are not allowed at
  the top-level of an expression, only as subexpressions within larger
  expressions. This allows Sylan to parse separate expressions unambiguously
  without error prone constructs like JavaScript's automatic semicolon
  insertion. This fixes the problem of a grouped expression being ambiguously
  either an invocation of an invocable on the previous line or a new grouped
  expression.
* Infix operators are supported, but only a limited set of known operators
  exist. This is to avoid parsing ambiguities and to avoid disambiguating with
  whitespace. There are no prefix operators, but there are two non-overriddable
  postfix operators. Despite being a limited set, each can be overridden by
  user libraries.
