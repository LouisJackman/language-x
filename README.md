# Sylan

An experimental programming language project to investigate what a spiritual
successor to Java and C# might look like.


## Overview

Java and C# helped to move C++ application programmers away from direct memory
management onto managed abstract machines with concurrency, networking,
serialisation, and portability built-in.

What will be the next language and runtime after Java and .NET? Better
distributed and concurrent programming support would be a good starting point.
No nulls, ubiquitous immutability, no concrete inheritance, supervisor trees in
the standard library, and transparent asynchronous IO would acknowledge this era
of computing.

Hardware is no longer speeding up exponentially year-on-year and is becoming
increasingly parallel. Performance cannot be discarded out of hand like the rise
of dynamic languages throughout the previous decade.

Large semantic changes will likely be rejected by developers if not presented in
a syntax similar to existing languages; like Java took C++'s syntax, the next
language should take Java's.

A gap exists between scripting languages and application languages; would enough
type inference and support for features like top-level executable code and
shebangs help bridge the gap?


## TODO

A VM will be needed for preemptive concurrency and light weight tasks; direct
compilation to native code will not be possible for such a language, although
JIT compilation is viable.

* [X] Lex.
* [ ] Parse.
* [ ] Interpret without any checks.
* [ ] Add checks such as types.
* [ ] Add optimisations like persistent data structures.


## The Language Proposal so Far

```
#!/usr/bin/env sylan

// If no package is specified, "main" is assumed.
public package main

// A single line comment.

/*
  A multiline comment.

  /*
    A nested multiline comment.
  */
*/

import io.{println, print}

interface ToString {
    public String toString()
}

interface Concatenate<T, Result = Concatenate> {
    public Result concatenate(T y)
}

class Account: ToString, Concatenate<Account> {
    public String firstName
    public String lastName
    public int ageInYears

    public Account(String firstName, String lastName) {
        println("instantiating an Account...")
        super(.firstName, .lastName, ageInYears = 35)
    }

    public override String toString() {
        `{firstName} {lastName} is {ageInYears} years old`
    }

    public override Account concatenate(Account a) {
        val firstName = firstName.concat(a.firstName)
        val lastName = lastName.concat(a.lastName)

        Account(
            .firstName,
            .lastName,
            ageInYears = ageInYears + a.ageInYears,
        )
    }

    public String get name() {
        `{firstName} {lastName}`
    }
}

extends class Account: Concatenate<Account, Result = String> {
    public override String concatenate(This that) {
        `{firstName} {that.firstName}`
    }
}

class Person = Account
interface Showable = ToString

int maxBound = 5

int factorial(int n) {
    switch n {
        case 0, 1:
            1
        default:
            if n < 0 {
                throw Exception("n cannot be less than 0")
            }
            factorial(n * (n - 1))
    }
}

package counter {
    public enum Message {
        Increment,
        Reset(int),
        Get,
    }

    public void start(Task sender, int n = 0) {
        select Message {
            case Increment:
                start(sender, n + 1)
            case Reset(n):
                start(sender, n)
            case Get:
                sender.send(n)
            timeout 10.seconds:
                throw Exception("timed out!")
        }
    }
}

void closureDemo() {
    val x = 5

    val account1 = Account(
        firstName = "Tom",
        lastName = "Smith",
        ageInYears = 15,
    )

    val firstName = "Tom"
    val lastName = "Smith"
    val age = 25
    val account2 = Account(.firstName, .lastName, ageInYears = age)

    val f = a -> {
        println(a.toString())
    }

    f(account1)
    f(account2(firstName = "Emma"))

    val g = a -> {
        println("returning an account")
        a
    }

    val z = g(account1)
}

void demoNumericLiterals() {
    int a = 5
    uint b = 5
    decimal c = 10.0

    byte d = 5u8
    uint16 e = 11u16
    uint32 f = 12u32
    uint64 g = 13u64
    int8 h = 15s8
    short i = 13s16
    int32 j = 7s32
    long k = 7s64
    float l = 12f16
    double m = 8f32
}

String double<N>(N n) if N: Add & ToString {
    (n + n).toString()
}

void demoIteration() {
    List(1, 2, 3).forEach(n -> {
        println(`{n}`)
    })

    val highlight = s -> `>> {s} <<`

    1.to(5)
        .map(double :: #toString :: highlight)
        .forEach(println)

    val quadruple = n -> n.double().double()

    123456789
        |> quadruple
        |> #toString
        |> highlight
        |> println

    val map = HashMap(
        "abc": 123,
        "def": 321,
        "ghi": 987,
    )
    map.forEach(Entry(key, value) -> {
        println(`{key}: {value}`)
    })

    val fact = for n = 20, result = 1 {
        if n <= 0 {
            result
        } else  {
            continue(n - 1, n * result)
        }
    }
    println(`factorial: {fact}`)
}

Optional<int> demoContexts() {
    do {
        val a <- Some(5)
        doSomething()
        val b <- Empty()
        willNotBeRun()
    }
}

// Top-level code is allowed, but only in the main package. Code in other packages must be in
// functions or methods.

Optional<String> optionalString = Some("test string")
if val Some(s) = optionalString {
    println(s)
}

val c = Task(-> counter.start(Task.self))
5.times(-> c.send(counter.Message.Increment()))

c.send(counter.Message.Get())
c.send(counter.Message.Increment())
c.send(counter.Message.Get())

// Should print 5 and then 6.
2.times(->
    select int n {
        println(`{n}`)
    }
)

print("""
Multiline
strings
""")

val x = {
    println("Returning 5 to be bound as x...")
    5
}
print(`{x}`)
```

See `src/parsing/ast_node_overview.lisp` for a visualisation of the abstract syntax tree generated
by that code.


## Goals

* Look as syntactically similar to Java and C# as possible.
* Support mixed-ability teams by not adding footguns or abstractions that do not
  scale; powerful features should have very little action-at-a-distance.
* Use null-free static types and increase type-system expressiveness over Java
  and C#.
* Make interpreter and other components easy to work with; make tool and IDE
  integration as easy as possible. Perhaps an FFI into the final parser and
  compiler and an initial Language Server Protocol implementation.
* Python/Perl style distribution; expect an interpreter on the OS and avoid
  bureaucratic requirements to run small programs.
* Use ubiquitous immutability to reduce unnecessary side-effects and coupling;
  invalid states should be unrepresentable.
* Allow distributed programming with message-passing.
* Transparently handle asynchronous IO.
* Make tasks cheap, preemptive, and killable; tasks should be a useful
  abstraction, not like threads which are a low-level OS-provided feature with
  endless edge cases.
* Remove or fix error-prone features from Java and C#, like
  assignments-as-expressions, pre and post decrement and increment, nulls,
  concrete inheritance, pervasive mutability, type erasure, statics,
  primitives and autoboxing, default memory sharing across tasks, and in-task
  catchable exceptions.
* Non-overflowable arithmetic should be default; machine-width arithmetic as an
  opt-in for performance.
* Encourage compile-time metaprogramming over runtime annotation and reflection;
  design it to scale in the large without becoming cryptic.
* Be mostly expression-based and with decent pattern matching.
* Guarantee tail-call elimination.


## Detailed Proposals

Accessibility levels:
- Public, internal, and private; only public and internal have keywords.
- Private level is default.

Types:
* Built-ins and user-defined.
* No difference between them from the user's perspective except for literal
  support and built-ins being predefined by the compiler and runtime.
* Final classes and trait-like interfaces. No concrete inheritance or
  abstract classes.
* Constructors are special; this is done to allow function-style
  instantiations while avoiding things like statics, needing to detach
  constructors from class definitions, or having a more complicated
  initialisation syntax.
* `void` is an actual type, like `()` in Haskell. Defining a method as
  returning `void` is a special-case that discards the result of final
  non-void expression and returns the void value instead. Every function
  returning a value, rather than having "procedures" without return values,
  avoids special-cases when composing functions in various ways.
* `super` is to either disambiguate which interface's method to delegate to,
  or to fallback to the auto-generated constructor in user-defined
  constructors. It does not deal with concrete class inheritance.
* Generics like C#, as in no type erasure.
* Support higher-kinded types, but keep an eye on projects like Dotty to
  see what type-soundness issues they encounter. Perhaps implement a more
  restricted version of it.

Methods and functions:
* Purposely different from one another; there isn't a UFC mechanism that
  unifies them.
* Both are fully higher-order. Methods carry around their instances with them.
* Methods, when passed around as values, are just seen as functions that have
  an instance bundled in their closure.
* `#method` is a shorthand for `(o, ...args) -> o.method(...args)`, where the
  type of `o` is inferred from the context.
* As methods and functions are both higher-order, invoked the same way, and
  have the same type when passed around, there is no real loss of
  composibility from being different constructs. They can be composed
  together easily:
  `val printDouble = Number.double :: #toString :: println`.

Pattern matching:
* Literals are matched as is.
* Composite types are matched as `Type(field1, field2 = match2, getterValue)`.
* Public fields and getters can be used.
* An identifier by itself is a shorthand for `identifier = identifier`.
* There are no positional matches, only matches by specific names.
* For types not overriding the built-in constructor, destructuring should be
  symmetrical with constructing.
* `...` can be used to omit the rest of the match.
* `val` can be used to bind any pattern to an identifier, e.g.:
  `case val account = Account(firstName, lastName = last, ...):`.
* Prefixing an identifier with a dot matches its value rather than binding
  against the identifier, e.g: `Account(firstName = .enteredFirstName, ...)`.

Matching in switch and select:
* They both have cases which each match one or more patterns seperated by commas.
* Both have `default` clauses as a fallback "match all" clause.
* `switch` is exhaustive: a compiler error happens if not all cases are covered.
* `select` is non-exhaustive, silently throwing away messages that don't match
  either the type or the matching patterns.
* `select` blocks the current task until someone sends the process a message of
  the specified type with a match. `timeout` clauses are available.

Invocations:
* Methods, functions, classes, and objects can be invoked with `()`.
* Invoking a method or a function does as one would expect; invoking a class
  constructs an instance; invoking a object allows non-destructive updates.
* Arguments can have default values.
* Any argument can be invoked as either positional or keyword; it's up to the
  caller.
* Two special parameter types exist: variadic and variadic entry. The former
  allows an variable amount of arguments, whereas the latter allows a variable
  amount of `sylan.lang.Entry` arguments with syntactical sugar. The latter is
  primarily for providing a good syntax for constructing map types that
  user-defined types can use.
* Prefixing an argument with a dot is a shortcut for assigning a keyword
  argument from a binding of the same name, e.g. `Account(.firstName)` is
  `Account(firstName = firstName)`.
* Passing `_` for one or more arguments partially applies the invocation,
  returning a new function with the non-underscore arguments evaluated and
  partially applied to the result. This allows, for example,
  partially-applying a non-destructive object update, partially applying a
  function, or partially-applying the instantiation of a class.

Language versioning:
* Keyword `v` can start a file.
* Has a version after it, e.g. `v1.0`
* If not present, assume to be the earliest stable release of the language.

Compile-time metaprogramming:
* No `constexpr`, templating, or `static if`. Should be the same language
  as runtime.
* Derive from Lisp and Jai but reduce footguns like Common Lisp automatic
  variable captures.
* Do not copy D or C++.
* Will eliminate the need for reflection.
* What are the security implications of running arbitrary code from the
  compiler?
* CL's `defmacro` is too low-level; a Java-like annotation syntax could be
  used for a more controlled subset, perhaps hygienic macro system a la
  Scheme.

Runtime structure information:
* No reflection.
* No runtime annotations.
* Use compile-time programming instead.
* Reduces magic, as compile-time metaprogramming cannot happen at random
  points during a running application unless `eval` exists.
* Improves performance as metaprogramming is done at compile-time.

The VM:
* It will probably be heavily BEAM-inspired.
* Must do tail call elimination.
* No mutable data except the execution of tasks over time.
* Lightweight processes. Immutability means changes can only be sent via
  messages.
* Initial toy implementation to use threads. Real implementation can use
  userland scheduler with remote process support.
* To handle remote processes, Tasks need node IDs.
* ...and nodes need network addresses or localhost.
* Per-task GC in theory; probably global GC in first implementation for
  simplicity. (Perhaps that's OK if only a single task's world gets stopped
  at any time.)
* Look at leveraging existing GCs via native interop, like Boehm-GC. However,
  they might be unsuitable for many lightweight tasks collecting
  concurrently.
* Persistent data structures.
* Async IO; use a library like Tokio. OK to block in the prototype, but don't
  add any language features that break compatibility with async in the proper
  version.

The build system:
* Go-style; just derive information from the source files rather than using
  separate configurations.
* If we must have config files, consider TOML.

Interop:
* Lightweight tasks will be awkward with POSIX/WinNT threads for native
  interop; see Erlang and Go's issues here. Not sure of a better alternative
  if we're using userland stacks and scheduling.

Standard lib:
* Standard lib should be modular, like Java 9's JRE. Implementations can
  opt-in to each, similar to C11 features like VLAs.

To consider later on:
* What happens if a task does multiple selections with different types? Do
  messages of the wrong type get saved to be selected later on, or are
  messages always thrown away if the current blocking select does not
  support that type?
* Parameterisable packages, perhaps a less powerful version of ML functors.
* Matrix operations to implement for user types, even if builtins do not use
  them. See Python 3 for an implementation of this.
