# Resource Management

-   Closable resources like file handles are managed consistently via the
    `AutoCloseable` interface.
-   The `using` keyword can prefix any `AutoCloseable` value, in which case it is
    guaranteed to close at the end of the scope even if the current task fails.
-   All `AutoCloseable` prefixed with `try` are guaranteed to have a chance to run
    their close method, even if one of them fails before all are complete.
-   They are closed in the order they were set up, reversed.
-   Using is scoped to the scope of the current top expression. For example, this
    applies to the whole `do` block and not just the argument evaluation inside of
    `f`: `do -> { g(f(using x)) } |> println`
