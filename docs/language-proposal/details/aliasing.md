# Aliasing

Classes, interfaces, functions, and methods can be aliased with a `=`.

If an alias puts `public` or `internal` after their name, they become
part of the out context's API, becoming _published_.

Overloaded operators can be aliased too, similar to other methods.

Aliases must point to _realised_ items. For example, pointing to an object's
method is fine, but pointing to a whole type's method is not. This is
consistent with how symbols are resolved in Sylan generally.

```
class C1 {
    fun public m() Int { throw Unimplemented() }
    fun public operator + (that This) This { throw Unimplemented() }
    fun public fun - (that This) This { throw Unimplemented() }
}

class C2 {
    var c1 C1 = C1()

    fun public m1 = c1.m
    fun public m2 = c1.operator +
    fun public operator + = c1.operator +

    fun public mul(that This) This
    fun public operator * = mul
}
```

### Non-published Aliases & Type Aliases

Non-published aliases to types, i.e. interfaces and classes, are less restricted
than other forms of alias.

A published alias must follow the rule of any other published item an an API in
Sylan; namely that interfaces, functions, and classes cannot each nest within
themselves.

A non-published type alias can appear in a few more places, such as inside the
bodies of classes, interfaces, functions, and methods.

```
fun f(message String) {
    println(message)
}

// Part of the package's API surface.
fun public g = f

// _Not_ part of the package's API surface.
fun h = g

fun p[S ToString](_ s S) {
    println(s)
}

fun h(message String) {

    // Will not compile:
    // fun f2 = f

    // Will compile:
    class FooBar = String

    var s = "abc"
    p[FooBar](s)
}
```
