# The Sylan Programming Language

[![CircleCI](https://circleci.com/gh/LouisJackman/sylan.svg?style=svg)](https://circleci.com/gh/LouisJackman/sylan)
[![codecov](https://codecov.io/gh/LouisJackman/sylan/branch/master/graph/badge.svg)](https://codecov.io/gh/LouisJackman/sylan)
![](https://img.shields.io/github/license/LouisJackman/sylan.svg)

**Warning: this project is still early stage and is a long way from completion.
See the "Done so Far" section for more details.**

Sylan is a Java-like, statically typed, functional, object-oriented, generic,
meta-programmable, and natively-compiled programming language with lightweight
concurrency and parallelism, transparently non-blocking IO, and extensive compile-time
programming.

It is an application language designed for web applications, network services,
command line programs, developer tooling, and scripting.

It is intended to scale from small scripts to large codebases, remove footguns,
and prioritise large-scale software engineering concerns over interesting
computer science concepts. It takes versioning and backwards compatibility
seriously and eases distribution by producing single, statically-linked
executables with no required runtimes.

## Contents

* [Installation](#installation)
* [Done so Far](#done-so-far)
* [Example](#example)
* [Overview](#overview)
* [Goals](#goals)
* [Details](#details)
  - [Modules and Packages](#modules-and-packages)
  - [Accessibility Levels](#accessibility-levels)
  - [Types](#types)
  - [Type Hierarchy](#type-hierarchies)
  - [Type Embedding](#type-embedding)
  - [Type Inference](#type-inference)
  - [Items and Expressions](#items-and-expressions)
  - [Pattern Matching](#pattern-matching)
  - [Matching in Switch and Select](#matching-in-switch-and-select)
  - [Invocations](#invocations)
  - [Compile-time Metaprogramming](#compile-time-metaprogramming)
  - [Runtime Structure Information](#runtime-structure-information)
  - [Resource Management](#resource-management)
  - [The Runtime](#the-runtime)
  - [Interoperability](#interoperability)
  - [Standard Library](#standard-library)
  - [To Consider](#to-consider)
* [Implementation Details](#implementation-details)

## Installation

Run `make` to build a production release Sylan, or `make build-dev` for a
development release. You can then find the resulting `sylan` executable in the
`target` directory; run Sylan programs via commands like `./sylan main.sy`.

If you'd rather run the build and the resulting Sylan executable in a container,
you can create a Docker image with `docker build -t sylan .` and then run Sylan
with invocations like `docker run --rm sylan main.sy`.

Sylan, being written in Rust, uses standard Rust tooling such as cargo for
development.  If you're familiar with it, use it as you would with any other
Rust project.

If not and you'd rather use `make`, run `make help` to see the available tasks
that can be run.

## Done so Far

- [x] Create Sylan "sourcing".
- [x] Lex.
- [ ] Parse.
- [ ] Simplify into "Kernel Sylan".
- [ ] Emit "Sylan IL".
- [ ] Implement the runtime.
- [ ] Interpret without any checks.
- [ ] Add checks such as types.
- [ ] Add optimisations like persistent data structures.
- [ ] Compile with bundled runtime.

## Example

    #!/usr/bin/env sylan

    // If no package is specified, "main" is assumed. A `module-info.sy` file is not
    // required for the top level program as there is an inferred `main` module that
    // contains top-level programs.
    package main

    // Most of those imports are already in the prelude and thus already imported:
    // `sylan.lang`. They are explicitly imported here to give an idea of the standard
    // package structure.
    import sylan.io.println
    import sylan.util.toString.ToString
    import sylan.util.collections.{HashMap, List}
    import sylan.util.concurrency.Task
    import sylan.util.datetime.ZonedDateTime
    import sylan.util.optional.{Empty, Some}

    func fizzBuzz(n Int) Int {

        // Sylan supports blocks, syntax for passing a lambda as a final argument to
        // an invocable. Note the use of `it` as a shorthand for one-argument
        // blocks.
        1.upTo(n).forEach -> {
            println(
                switch {
                    0 == (it % 15) { "FizzBuzz" }
                    0 == (it % 5) { "Fizz" }
                    0 == (it % 3) { "Buzz" }
                    _ { it }
                }
            )
        }
    }

    fizzBuzz(100)

    /**
     * SyDocs can be written for code documentation.
     */
    func demoIteration() {

        var highlight = -> s { $">> {s} <<" }

        // The # operator composes invocables together.
        1.upTo(5)
            .map(::Number.doubled # ::ToString.toString # ::highlight)
            .forEach(::println)

        // Sylan supports field-looking getters and transparently upgrading an
        // expression to a lazy computation, both of which mandate no parentheses
        // for no-argument invocables. Use the :: operator without a prefix to pick
        // up these invocables as first-class values. Combine it with the dot
        // operator to refer to a method of a type.

        // Lambdas can be passed outside of calling parentheses when they are the
        // final argument.
        List(1, 2, 3).forEach -> n {

            /*
             * Backquoted strings allow interpolation. Only single symbols can be
             * interpolated; interpolating arbitrary expressions was purposely
             * omitted due to its ability to create cryptic one-liners.
             */
            println($"{n}")
        }

        var quadruple = -> { it.doubled.doubled }

        123456789
            |> ::quadruple
            |> ::Object.toString
            |> ::highlight
            |> ::println

        var map = HashMap(
            "abc": 123,
            "def": 321,
            "ghi": 987,
        )
        map.forEach -> key, value {
            println($"{key}: {value}")
        }

        do -> {
            var counterService = using Task -> {
                for n = 0 {
                    var sender = select Task {
                        _ { it }
                    }
                    if n < 5 {
                        sender.send(Some(n))
                        continue(n + 1)
                    } else {
                        sender.send(Empty)
                        continue(n)
                    }
                }
            }

            5.times -> {
                counterService.send(currentTask)
            }
            while var Some(n) = select int { _ { it } } {
                println($"{n}")
            }
        }

        /*
         * Sylan does not allow scope shadowing. As functions are just
         * variables of a "function type", `factorial` would clash with the
         * top-level `factorial` function defined later on, hence the disambiguating
         * name `innerFactorial`.
         *
         * `for` is a cross between Java's enhanced-for loop and Scheme's named-let.
         */
        var innerFactorial = for var n = 20, var result = 1 {
            if n <= 0 {
                result
            } else {

                // Continue, like `it`, is a keyword representing an implicit
                // variable. In this case it stands for the inner-most
                // unlabelled for loop.
                continue(n - 1, n * result)
            }
        }

        for outer var n = 10 {
            if 0 < n {
                for {

                    // This for loop is labelled as "outer", so that is available
                    // to call as a function if iteration is desired. That allows
                    // it to skip the inner `for` here; `continue` in this context
                    // would cause an infinite loop.
                    outer(n - 1)
                }
            }
        }

        println($"factorial: {innerFactorial}")
    }

    /*
     * All variables (including functions), packages, classes
     * and methods are private unless explicitly opened up with the `public`
     * keyword. If the developer wants to expose them openly but only to
     * its own module, they can use the `internal` keyword.
     *
     * As can be seen in this function and the previous, Sylan allows functions to
     * stand alone in packages without being tied to classes or interfaces.
     */
    func internal factorial(n Int) Int {
        switch n {
            0, 1 {
                1
            }
            n if n < 0 {
                throw Exception("n cannot be less than 0")
            }
            n {
                /*
                 * Guaranteed tail call elimination will ensure a stack overflow
                 * does not occur.
                 */
                factorial(n * (n - 1))
            }
        }
    }

    func internal ignorable printFactorial(n Int) Int {
        /*
         * Sylan does not allow callers to throw away non-`Void` function or methods
         * results unless they're declared with the `ignorable` modifier.
         *
         * This makes sense when getting a return value is potentially useful but
         * not the entire reason behind invoking it.  Such a function or method can
         * be assumed to have side effects, with the exception of constant-time
         * operation NOPs or the like.
         */

        var result = factorial(n)
        println(result)

        /*
         * Sylan is an expression-oriented language, so everything returns a value
         * including loops and `if`s. Therefore, the last value is returned by
         * default. The lack of explicit `return`s means there is always a single
         * exit point from a function or method: the last expression.
         */
        result
    }

    func twice[N Number = int](n N, f (N) N) N {
        f(n) + f(n)
    }

    func doubled[N Add & ToString](n N) String {
        (n + n).toString()
    }

    /**
     * Sylan uses concrete implementations of interfaces by default when they're
     * used as typed directly. To get dynamic dispatch, use the virtual keyword
     * like so:
     *
     *     func printNumber(n virtual Number) Number {
     */
    func printNumber(n Number) {
        println($"{n}")
    }

    func demoContexts() Optional[Int] {

        /*
         * Haskell programmers will recognise this as a monadic do notation, which
         * Sylan calls "contexts". Non-Haskell programmers will be glad to know that
         * this feature can make tangled code such as optionality-checking chains
         * and validation code much cleaner.
         */
        with {
            <-Some(5)
            println("Doing something.")

            // All invocables can drop parentheses if calling with no arguments.
            // This is actually how getters work behind the scenes and why Sylan
            // needn't allow public fields as a feature.
            var _ = <-Empty

            println("Will not be run.")

            Empty
        }
    }

    package counter {

        enum Message {
            Increment,
            Reset(Int),
            Get,
        }

        func public start(sender Task, n Int = 0) {
            select Message {
                .Increment {
                    start(sender, n + 1)
                }
                .Reset(n) {
                    start(sender, n)
                }
                .Get {
                    sender.send(n)
                    start(sender, n)
                }
                timeout 10.seconds {
                    throw Exception("timed out!")
                }
            }
        }
    }

    /**
     * Backquotes can be used to allow any character in an identifier name. This is
     * designed for two cases:
     *
     *  * Allowing fluent sentence-like test cases like below.
     *  * Easing interoperability with foreign APIs that use different naming
     *    contentions.
     *
     * Like normal strings, they also support raw mode and custom delimiters.
     * Interpolation is obviously not supported, however.
     */
    func `test that addition works`() {
        if (2 + 2) != 4 {
            throw Exception("OK, I give up...")
        }
    }


    interface Concatenate[T] {
        func public concatenate(T) T
    }

    interface Equals[T] {
        func public equals(T) Boolean

        /*
         * Sylan's interface methods can have bodies like Java 8's defender methods.
         * Unlike Java 8, private methods with bodies are also allowed with the.
         * Similar to C# but unlike Java, a default method body in an interface
         * can only be overridden if annotated with `virtual`.
         *
         * `virtual` is default for non-default methods since the lack of concrete
         * inheritance in Sylan means there is no other possible use case for them
         * except to be implemented by the implementor.
         */
        func public virtual notEquals(other T) boolean {
            !equals(other)
        }
    }

    /**
     * Hashing, equality checking, and the constructor all just exist
     * automatically. They can still be manually overridden if necessary though.
     * `close` is also automatically implemented for classes that implement
     * `AutoCloseable`, the default implementation just closing all
     * `AutoCloseable` fields. This can also be overridden if the default
     * implementation makes no sense.
     */
    class Name {
        var internal firstName String
        var internal lastName String
    }

    /**
     * Classes can implement interfaces but cannot extend other classes.
     * Interfaces can extend other interfaces though. Concrete inheritance
     * generally causes more problems that it's worth and type embedding plus a
     * more powerful interface system makes up for the difference.
     */
    class Account implements ToString, Concatenate[This] {

        /*
         * Embedding a field hoists all of its _accessible_ methods into
         * the class itself. Embedded methods take the lowest priority: methods in
         * the class itself and default methods in implementing interfaces both take
         * priority over embedded methods, although other embedded methods will
         * still call the embedder's method if calling it by name.
         *
         * Embedded classes can implement methods for implementing interfaces on the
         * embedder's behalf, following the rules above.
         *
         * Note that no relationship whatsoever is established between the classes
         * after the embedding; there is no concrete subtyping in Sylan. It's solely
         * for DRY, a.k.a. Don't Repeat Yourself.
         *
         * The earlier embeddings take priority over the later ones, to be
         * consistent with the MRO of inheritance. Therefore, changing the order of
         * embeds can change behaviour.
         */
        var embed name Name

        var internal ageInYears Int

        var expiry ZonedDateTime = ZonedDateTime.now() + 1.year

        constructor(firstName String, lastName String) {
            println("instantiating an Account...")

            /*
             * Constructors are the only place in Sylan where assignments are
             * allowed outside the context of a binding. Each field must be set
             * once, and exactly once.
             *
             * Note that this class elsewhere uses the auto-generated getters for
             * these fields but assignments in constructor must set the underlying
             * fields via `this` directly.
             */
            this.name.firstName = firstName
            this.name.lastName = lastName
            this.ageInYears = 35
        }

        func public override toString() String {
            $"{firstName} {lastName} is {ageInYears} years old"
        }

        func public override concatenate(other This) This {
            var firstName = firstName.concat(other.firstName)
            var lastName = lastName.concat(other.lastName)

            Account(
                .firstName,
                .lastName,
                ageInYears = ageInYears + other.ageInYears,
            )
        }

        /*
         * Methods without any arguments can be called without parentheses,
         * effectively making them readonly properties or getters with syntactical
         * sugar. An important difference between Java getters and Sylan "getters"
         * is that Sylan's can replace fields transparently, allowing fields to be
         * "upgraded" to getters transparently without breaking API compatibility.
         *
         * Getters are automatically generated for fields with the same
         * accessibility.
         *
         * Fields must always be accessed via `this`, but as even private fields
         * autogenerate getter methods of the same accessibility, methods can use
         * the getter in practice e.g. `name` rather than `this.name`.
         *
         * `this` is intended to only be used explicitly in explicit getter
         * implementations or when assigning in constructors.
         */

        func public name() String {
            $"{firstName} {lastName}"
        }

        func public locked() Boolean {
            expiry < ZonedDateTime.now
        }
    }

    /*
     * Type extensions allow modules to add their own features to an existing class
     * when imported. Note that this implements `Concatenate` again just with
     * different type parameters passed to `Concatenate`; Sylan considers these to
     * be two completely different implementations. This is a more disciplined
     * replacement for method overloading.
     */
    extend class Account implements Concatenate[This, Result = String] {

        func public override concatenate(that This) String {
            $"{firstName} {that.firstName}"
        }
    }

    func demoPatternMatching() {
        var account1 = Account(firstName = "Tom", lastName = "Smith")
        var matchingLastName = "Smith"

        if var Account(firstName, lastName = .matchingLastName) as account
                = account1 {
            println($"Matching first name: {firstName}")
            println($"Matching account: {account}")
        }

        switch account1 {

            Account(locked = true), Account(firstName = "unlucky") {
                println("LOCKED!")
            }

            Account(expiry) if expiry < ZonedDateTime.now {
                println("ALSO LOCKED")
            }

            _ {
                println("NOT LOCKED: ")
                println(account1)
            }

            /*
             * Switches are expressions, support multiple cases, can do pattern
             * matching, and have support for conditional "guards".
             */
        }
    }

    /*
     * Aliasing of various constructs to allow easy code repair during large-scale
     * refactoring. Any item in any module can be aliased, which can ease module
     * versioning where new major module versions are effectively different modules.
     */
    var counter2 = counter
    var Person = Account
    var Showable = ToString
    var factorial2 = factorial

    var maxBound Int = 5

    func demoClosures() {
        var x = 5

        var account1 = Account(
            firstName = "Tom",
            lastName = "Smith",
            ageInYears = 15,
        )

        var firstName = "Tom"
        var lastName = "Smith"
        var age = 25
        var account2 = Account(.firstName, .lastName, ageInYears = age)

        var f = -> { println(it.toString) }

        f(account1)
        f(account2(firstName = "Emma"))

        var g = -> {
            println("returning a value")
            it
        }

        var z = g(account1)

        var n = twice(3) -> { it * 2 }
        println($"n == {n}")
    }

    class NumericLiteralsClass implements ToString {

        // Non-overflowable numbers.
        var a Int = 5
        var b UInt = 5u
        var c Decimal = 10.0

        // Overflowable, machine-width numbers.
        var d Byte = 5u8
        var e UInt16 = 11u16
        var f UInt32 = 12u32
        var g UInt64 = 13u64
        var h Int8 = 15s8
        var i Short = 13s16
        var j Int32 = 7s
        var k Long = 7s64
        var l Float = 12f
        var m Double = 8f64

        func public override toString() String {
            $"{a}{b}{c}{d}{e}{f}{g}{h}{i}{j}{k}{l}{m}"
        }
    }

    class AutoCloseableDemo implements AutoCloseable {

        constructor() {
            println("Opened")
        }

        func public override close() {
            println("Closed")
        }
    }

    func demoAutoCloseables() {

        /*
         * Sylan, like Erlang and Go, strongly discourages catching exceptional
         * problems on a routine basis - if something is truly "exceptional" then
         * the developer wouldn't have thought of a meaningful response to it.
         * Instead it encourages assertive programming through a "happy path" and
         * using supervisor tasks to allow the system to heal from unexpected
         * failures.
         *
         * Sylan does not offer `try`/`catch` for this reason, and also to ensure
         * that exception-driven control flow isn't possible.
         *
         * However, there still needs to be a strategy regarding closing resources
         * in tasks that have had unexpected failures. Sylan offers `AutoClosable`
         * types and the `using` keyword for this, which are a sort of middle ground
         * between C#'s `IDisposable` and its `using` keyword, and Go's `defer`
         * keyword.
         *
         * If an `AutoCloseable` type is prefixed with the `using` keyword, its
         * `close` method is invoked at the end of the scope _even if the
         * code in the scope completely fails unexpectedly_. The closing is done in
         * the reverse order that they are set up. If any `defer`d `close` methods
         * themselves fail, all other deferred calls are given a chance to run too,
         * and all errors are accumulated into one and rethrown once they're all
         * finished.
         */

        println("Entering scope")
        do -> {
            var closeable = using AutoCloseableDemo
            println("Using closeable")
        }
        println("Leaving scope")

        // Prints "Entering scope", "Opened", "Using closeable", "Closed", and finally "Leaving scope".
    }

    func demo() {
        // As mentioned above, all invocables can omit parentheses if they have
        // zero parameters. This is primarily for getters, but should also be used
        // for Pascal-procedure-style invocations just for consistency. That
        // includes constructors, methods, lambdas, and functions.

        demoIteration

        printFactorial(42)
        var x = twice(4, ::doubled)
        println($"{x}")

        demoContexts
        demoPatternMatching
        demoClosures

        println(NumericLiteralsDemo.toString)

        demoAutoCloseable
    }

    /*
     * Top-level code is allowed, but only in the main package within the main
     * module. Code in other packages must be in functions or methods.
     *
     * Top-level declarations must usually be explicitly typed, but type inference
     * with `var` is allowed for the main package, in which case they are just
     * type-inferred local variables for the top-level program which are private
     * to the `main` package.
     */

    var optionalString = Some("test string")
    if var Some(s) = optionalString {
        println(s)
    }

    var c = Task -> { counter.start(currentTask) }
    5.times -> { c.send(counter.Message.Increment) }

    c.send(counter.Message.Get)
    c.send(counter.Message.Increment)
    c.send(counter.Message.Get)

    // Should print 5 and then 6.
    2.times -> {
        var n = select Int {
            _ { it }
        }
        println($"{n}")
    }

    print("""
    3 or more quotes can be used to create a custom string delimiter.
    This allows escaping almost any embedded content with few problems.
    This works with interpolated and raw strings too.
    """)

    println(r"Raw string in which escapes characters are now read literally: \n\r")

    var n = 4

    println($"""
    Multiline interpolated strings: {n}
    """)

    var x = do -> {
        println("Returning 5 to be bound as x...")
        5
    }
    println($"{x}")

    /*
      A multiline comment.
    
      /*
        A nested multiline comment.
      */
    */

    demo

## Overview

Java and C# moved C++ application programmers away from direct memory management
onto managed abstract machines with concurrency, networking, serialisation, and
portability built-in.

What about the next language and runtime after Java and .NET? Better distributed
and concurrent programming support would be a good starting point. Null-free
types, ubiquitous immutability, a lack of concrete inheritance, built-in
supervisor trees, and transparently non-blocking IO would acknowledge this era
of computing.

Hardware is no longer speeding up exponentially year-on-year and is becoming
increasingly parallel. Performance cannot be discarded out of hand like the rise
of dynamic languages throughout the previous decade.

Large semantic changes will likely be rejected by developers if not presented in
a syntax similar to existing languages; like Java and C# took C++'s syntax, the
next language should take Java's and C#'s.

A gap exists between scripting languages and application languages; would enough
type inference and support for features like top-level executable code and
shebangs help bridge the gap to allow both to be done in one language?

Strong isolation boundaries at modules and explicitly declared dependent modules
and their full versions are required to evolve large systems over time without
breaking compatibility. Major version changes, according to semantic versioning,
are breaking changes, and should therefore be considered completely different
modules. Non-pinned dependency versions or disallowing the import of multiple
major versions of the same module does not scale in the modern world of software
development.

A runtime will be needed for preemptive concurrency and light weight tasks.  Go
shows that compromises regarding the tasks' preemptiveness must be made in the
case of native compilation.

## Goals

* Look as syntactically similar to Java and C# as possible.
* Support mixed-ability teams by not adding footguns or abstractions that do not
  scale; powerful features should have very little action-at-a-distance.
  Computer science concepts are trumped by real world, large-scale software
  engineering concerns.
* Use null-free static types and increase type-system expressiveness over Java
  and C#.
* Make tool and IDE integration as easy as possible. Perhaps an FFI into the
  final parser and compiler and an initial Language Server Protocol
  implementation.
* Focus on AOT compilation for ease of distribution, Go- and Rust-style.
  Support interpretation for embedding, quick debugging, and interactive
  development.
* Use ubiquitous immutability to reduce unnecessary side-effects and coupling;
  invalid states should be unrepresentable.
* Allow distributed programming with message-passing.
* Transparently handle asynchronous IO.
* Make tasks cheap, preemptive, and killable; tasks should be a useful
  abstraction, not like threads which are a low-level OS-provided feature with
  endless edge cases.
* Remove or fix error-prone features from Java and C#, like
  assignments-as-expressions, pre and post decrement and increment, nulls,
  concrete inheritance, pervasive mutability, type erasure, statics, primitives
  and autoboxing, default memory sharing across tasks, and in-task catchable
  exceptions.
* Non-overflowable arithmetic should be default; machine-width arithmetic as an
  opt-in for performance.
* Encourage compile-time metaprogramming over runtime annotation and reflection;
  design it to scale in the large without becoming cryptic.
* Be mostly expression-based and with decent pattern matching.
* Guarantee tail-call elimination with shadow stacks to aid debugging.

## Details

### Modules and Packages

* Packages live within modules.
* Modules are the most coarse encapsulation boundary developers have.
* Package are for laying out the structure of modules.
* There is an implicit `main` module and `main` package.  The implicitness is to
  reduce boilerplate for single-line programs and scripts.
* Programs must be in a `main` module and a `main` package, but libraries must
  have their own unique modules names.
* Modules, unlike packages, are versioned.
* They declare which packages they depend on.
* Only items that are public and all of whose parents are also public are
  exposed.
* The `internal` accessibility modifier allows making something public to other
  packages, but only within the module.
* A different major version of a module is effectively considered a different
  module in practice. In fact, multiple major versions of the same module can be
  required at once. Cross-module aliases can be used to gradually evolve major
  package versions over time without breaking compatibility.
* Leaving a version number out assumes 0.x. Major version 0 is the only version
  for which breaking changes across minor versions is acceptable.
* The use of private-by-default accessibility combined with no reflection or
  runtime metadata should give some decent oppertunities for dead-code
  elimination and inter-procedural optimisations within a module.

### Accessibility Levels

* Private, internal, and public.
* Private is the default, no keyword required.
* The `internal` keyword exposes an item to its parent, but only within its
  module.
* The `public` keyword exposes an item to its parent, available from outside the
  module too.

### Items and Expressions

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
* Both unary and infix operators are supported, but one operator cannot be both.
  This is to avoid parsing ambiguities and to avoid disambiguating with
  whitespace. Unary operators are fixed whereas infix operators are arbitary
  identifiers, which can be defined by developers.

### Types

* Built-ins and user-defined.
* No difference between them from the user's perspective except for some of them
  having literals baked into the language  and built-ins being predefined by the
  compiler and runtime. No Java-like primitive vs object distinction.
* Constructors are special; this is done to allow invocable-style instantiations
  while avoiding things like statics, needing to detach constructors from class
  definitions, or having a more complicated initialisation syntax.
* `void` is an actual type, like `()` in Haskell.  Defining a function or method
  as returning `void` is a special-case that discards the result of final
  non-void expression and returns the void value instead. Every function,
  method, and lambda returning a value, rather than having "procedures" without
  return values, avoids special-cases when composing invocables in various ways.
* Generics like C#, as in no type erasure.
* Support higher-kinded types, but keep an eye on projects like Dotty to see
  what type-soundness issues they encounter. Perhaps implement a more restricted
  version of it.
* Public fields don't exist, but Sylan will autogenerate getter methods for
  "public" fields that can later be redefined. This means, unlike Java, Sylan
  can provide a manually-implemented getter at a later date without breaking API
  compatibility with the existing public field. Setters do not exist due to
  Sylan being a immutable language.

### Bindings

* Types, variables, functions are all in the same namespace. Although types and
  values exist in two different phases, compile-time and runtime respectively,
  Sylan still won't allow identifiers between the two to clash.
* Types should start with capital letters, and values with lowercase letters.
  This is just a convention though, one which Sylan's prelude package breaks
  frequently to make the language look more Java-like, e.g. `boolean` rather
  than `Boolean`.
* Methods are namespaced to their types, although just deeper in the namespacing
  hierarchy rather than in a completely different standalone global namespace.
* Shadowing is not allowed except for keyword identifiers.
* There are three keyword identifiers: `_`, `continue`, and `it`. `continue` and
  `it` are _almost_ dynamically scoped, changing implicitly throughout scopes
  based on the context. `continue` binds to the innermost non-labelled `for`
  iteration function, `it` is the innermost syntactically-zero-parameter
  lambda's sole parameter, and `_` is an ignored value in a binding and a
  partial-application notation for the innermost invocation.
* Types and variables can both be thought of as "bindings", just one at
  compile-time and another at runtime. Never the twain shall meet, at least
  until Sylan designs how they should interoperate if at all. This will depend
  on how compile-time metaprogramming is implemented and whether Sylan decides
  to implement any form of dependent typing.
* Fields are always accessed via the `this` keyword, as fields don't belong to
  the class but instead via a special scope only their own methods can access.
  Classes ultimately can only expose methods and nothing else. As field
  declarations always autogenerate getters of the same accessibility, methods
  can access even private fields without using `this.` as it can use the getter
  instead, i.e. `name` rather than `this.name`.

### Type Hierarchies

* Final classes and trait-like interfaces. No concrete inheritance or abstract
  classes. Discourages "type taxonomy".
* `super` is to either disambiguate which interface's method to delegate to, or
  to fallback to the auto-generated constructor or other auto-generated method
  like `hashcode` or `equals` in user-defined constructors and methods. It does
  not deal with concrete class inheritance.
* Interfaces can provide default method implementations, which can be overridden
  in implementing classes if they are `virtual`. `override` on a method in a
  implementing class documents that it must override rather than just shadow.
  Method shadowing by default, while problematic in some areas, allows
  interfaces to add new virtual default methods over time without breaking
  backwards compatibility with implementors that already define their own
  methods of the same name. "Shadowing" is when a method occupies that name in a
  class' own other methods but does not override calls to that method name in
  the interfaces' default methods.
* Method resolution is done via Dylan- and Python 3-inspired MRO rather than
  forcing explicit disambiguation like Java 8's defender methods. This is done
  to ensure that default method additions do not break backwards compatibility
  with implementing types that potentially implement another interface that has
  a method compatible with the newly added method, causing a diamond inheritance
  clash. See [C3
  linearization](https://en.wikipedia.org/wiki/C3_linearization").

### Type Embedding

* The `embed` keyword allows classes to embed other classes. This just takes all
  methods of a class and hoists them to the top level of the class itself.
* If multiple embeds embed a method of the same signature, the
  earliest one takes priority. This is to follow the first-takes-priority system
  characteristics of MRO in inheritance, and to avoid backwards-compatibility
  breakages caused by the addition of new methods to embedded types.
* There is no sub-typing relationship established by embedding whatsoever.
  Sylan does not support subtyping.

### Type Inference

* Only nominal types, no structural types. While it would be nice to mandate
  that all type inference should be derivable from inner expressions outwards
  without going the other way, that isn't possible due to constructs like
  immediately-invoked lambdas or lambdas with parameters being passed as a
  parameter typed as a lambda that takes compatible but more precise types.
* Once we allow that, there's no reason not to allow LHS type inference to flow
  from the RHS as long as it only flows to declarations within a function,
  lambda, or method body and does not escape it.
* Sylan is more opinionated than some statically-typed functional languages on
  type inference; while all function-local variables can infer types, declared
  items in classes and packages must always be explicitly typed. This includes
  package functions, class methods, constructors, and class fields.
* This ensures APIs and program structure is rigid and explicitly typed while
  expressions are concise and with little boilerplate.
* This philosophy extends to lambdas; as lambdas are used locally and not for
  definitions (which use declared functions and methods), lambda expressions
  also only allow inferred types (which is why type parameter syntax is not
  supported).

### Language Versioning

* Keyword `v` can start a file to denote the source version.
* Has a version after it, e.g. `v1.0`
* If not present, assume to be the latest minor version of the earliest major
  release of the language, i.e. 1.x.
* Three things are versioned: the source, the tokens, and the AST.
* A source version pins its tokens version; a tokens version pins its AST
  version. So the versioning between the three isn't necessarily in lockstep,
  but it is fixed.

### Methods and Functions

* Purposely different from one another; there isn't a UFC mechanism that unifies
  them.
* Both are fully higher-order. Methods carry around their instances with them.
  Lambdas are just functions not bound to a symbol, so they otherwise work in
  the same way.
* Methods, when passed around as values, are just seen as functions that have an
  instance bundled in their closure.
* `Class::method` is a shorthand for `fn -> o, ...args { o.method(...args) }`, where
  the type of `o` is `Class`.
* As methods and functions are both higher-order, invoked the same way, and have
  the same type when passed around, there is no real loss of composability from
  being different constructs. They can be composed together easily: `var
  printDouble = Number::double # ToString::toString # println`.

### Pattern Matching

* Literals are matched as is.
* Composite types are matched as `Type(getter1, getter2 = match2, getter3
* "Getters" (i.e. nullary methods) can be used, which are autogenerated for
  all "public" fields automatically.
* An identifier by itself is a shorthand for `identifier = identifier`.
* There are no positional matches, only matches by specific names.
* For types not overriding the built-in constructor, destructuring should be
  symmetrical with constructing.
* `...` can be used to omit the rest of the match.
* `as` can bind part of a match to a name, e.g.  `Account(firstName) as
  account`.
* Prefixing an identifier with a dot matches its value rather than binding
  against the identifier, e.g: `Account(firstName = .enteredFirstName, ...)`.

### Matching in Switch and Select

* They both have cases which each match one or more patterns separated by
  commas.
* Both have `default` clauses as a fallback "match all" clause.
* `switch` is exhaustive: a compiler error happens if not all cases are covered.
* `select` is non-exhaustive, sending non-matching messages to the `noReceiver`
  task's mailbox (whose behaviour can be manually overridden).
* `select` blocks the current task until someone sends the process a message of
  the specified type with a match. `timeout` clauses are available.

### Invocations

* Methods, lambdas, functions, classes, and objects can be invoked with `()`.
* Any of those are referred to "invocables" when used like that.
* Invoking a method, lambda, or a function does as one would expect; invoking a
  class constructs an instance; invoking an object allows non-destructive
  updates.
* Arguments can have default values.
* Any argument can be invoked as either positional or keyword; it's up to the
  caller.
* Two special parameter types exist: variadic and variadic entry. The former
  allows an variable amount of arguments, whereas the latter allows a variable
  amount of `sylan.lang.Entry` arguments with syntactical sugar.  The latter is
  primarily for providing a good syntax for constructing map types that
  user-defined types can use.
* Prefixing an argument with a dot is a shortcut for assigning a keyword
  argument from a binding of the same name, e.g.  `Account(.firstName)` is
  `Account(firstName = firstName)`.
* Passing `_` for one or more arguments partially applies the invocation,
  returning a new function value with the non-underscore arguments evaluated and
  partially applied to the result. This allows, for example, partially-applying
  a non-destructive object update, partially applying a function, or
  partially-applying the instantiation of a class.

### Compile-time Metaprogramming

* No `constexpr`, templating, or `static if`. Should be the same language as
  runtime.
* Derive from Lisp and Jai but reduce foot guns like Common Lisp automatic
  variable captures.
* Do not copy D or C++.
* Will eliminate the need for reflection.
* What are the security implications of running arbitrary code from the
  compiler? Surely we should at least ban system side-effects?
* CL's `defmacro` is too low-level; a Java-like annotation syntax could be used
  for a more controlled subset, perhaps hygienic macro system a la Scheme.
* Macros can only be invoked against a specific tokens version or an AST
  version. The compiler just won't compile macros designed for a v2 AST when
  invoked against a v3 source.

### Runtime Structure Information

* No reflection.
* No runtime annotations.
* Use compile-time programming instead.
* Reduces magic, as compile-time metaprogramming cannot happen at random points
  during a running application unless `eval` exists.
* Improves performance as metaprogramming is done at compile-time.

### Resource Management

* Closable resources like file handles are managed consistently via the
  `AutoCloseable` interface.
* The `try` keyword can prefix any `AutoCloseable` value, in which case it is
  guaranteed to close at the end of the scope even if the current task fails.
* All `AutoCloseable` prefixed with `try` are guaranteed to have a chance to run
  their close method, even if one of them fails before all are complete.
* They are closed in the order they were set up, reversed.

### The Runtime

* It will probably be heavily BEAM-inspired.
* Must do tail call elimination with a "shadow stack" to aid debugging, inspired
  by Safari's JavaScript runtime's implementation of ES6 tail call elimination.
* No mutable data except the execution of tasks over time.
* Lightweight processes. Immutability means changes can only be sent via
  messages or tracked via services external to the Sylan program.
* Initial toy implementation to use threads. Real implementation can use
  userland scheduler with remote process support.
* To handle remote processes, Tasks need node IDs.
* ...and nodes need network addresses or localhost.
* Per-task GC in theory; probably global GC in first implementation for
  simplicity. (Perhaps that's OK if only a single task's world gets stopped at
  any time.)
* Persistent data structures, which should be definable as a library rather than
  baked into the language itself.
* Async IO; use a library like Tokio. OK to block in the prototype, but don't
  add any language features that break compatibility with async in the proper
  version.

### The Build System

* Go-style; just derive information from the source files rather than using
  separate configurations.
* Strong compile-time metaprogramming can assist here, similar to how Jai
  replaces Makefiles with compile-time profiles.

### Interoperability

* `extern` allows calling functions that are either statically linked in or via
  a named dynamically linked library.
* Public exposed symbols in Sylan are accessible by either statically linking
  the result into another executable or by creating a dynamically linked library
  when building the Sylan program and then referring to it dynamically from
  other executables.
* As Sylan does not support ad hoc overloading, symbol demangling is
  straightforward. One underscore denotes a package change while two indicate a
  method belonging to a type. E.g.: `sylan.util.collections.HashMap#put` becomes
  `sylan_util_collections_HashMap__put`. How type parameters work with symbol
  mangling still needs to be worked out.
* Lightweight tasks will be awkward with POSIX/WinNT threads for native
  interoperability; see Erlang and Go's issues here. Not sure of a better
  alternative if we're using userland stacks and scheduling. Entering and
  exiting Sylan from non-Sylan code will probably require allocating threads
  allocated solely to avoid blocking in foreign code blocking the Sylan runtime.

### Standard Library

* Standard lib should be modular, like Java 9's JRE.

## To Consider

* Parameterisable packages, perhaps a less powerful version of ML functors.
* Matrix operations to implement for user types, even if builtins do not use
  them. See Python 3 for an implementation of this.
* Subtyping for built in types, like subranges across integers, Ada-style?
* Multiversal equality rather than universal equality.
* Reject changes that break backwards compatibility if the major version isn't
  bumped, similar to Elm.
* Replace auto-generated methods with a more consistent `derives`/`deriving`
  approach, a la Rust and Haskell.

## Implementation Details

For more documentation on the actual implementation of Sylan, read the Rust
documentation comments of the code. A good starting point is the documentation
of [the main top-level source
file](https://github.com/LouisJackman/sylan/blob/master/src/main.rs).

