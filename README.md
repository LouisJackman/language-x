# The Sylan Programming Language

[![Build Status](https://travis-ci.org/LouisJackman/sylan.svg?branch=master)](https://travis-ci.org/LouisJackman/sylan)
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

    /*
     * If no package is specified, "main" is assumed. A `module-info.sy` file is not
     * required for the top level program as there is an inferred `main` module that
     * contains top-level programs.
     */
    package main

    /*
     * Most of those imports are already in the prelude and thus already imported:
     * `sylan.lang`. They are explicitly imported here to give an idea of the standard
     * package structure.
     */
    import sylan.io.println
    import sylan.util.toString.ToString
    import sylan.util.collections.{HashMap, List}
    import sylan.util.concurrency.Task
    import sylan.util.datetime.ZonedDateTime
    import sylan.util.optional.{Empty, Some}

    void demoIteration() {
        /**
         * SyDocs can be written for code documentation. Unlike JavaDoc but similar
         * to Python, they go inside the item they describe rather than before it.
         */

        // Lambdas are declared with braces and an arrow.
        var highlight = { s -> `>> {s} <<` }

        1.to(5)
            .map(Number::double # ToString::toString # highlight)
            .forEach(println)

        // If a lambda is the only argument to an invocable, the parentheses can
        // be dropped.
        List(1, 2, 3).forEach { n ->

            /*
             * Backquoted strings allow interpolation. Only single symbols can be
             * interpolated; interpolating arbitrary expressions was purposely
             * omitted due to its ability to create cryptic one-liners.
             */
            println(`{n}`)
        }

        // The arrow can be dropped for no-argument lambdas.
        // Furthermore, single-parameter lambdas can use the `it` keyword instead.
        var quadruple = { it.double().double() }

        123456789
            |> quadruple
            |> Object::toString
            |> highlight
            |> println

        var map = HashMap(
            "abc": 123,
            "def": 321,
            "ghi": 987,
        )
        map.forEach { key, value ->
            println(`{key}: {value}`)
        }

        do {
            var mutatingService = using Task {
                for n = 0 {
                    var sender = select Task
                    if n < 5 {
                        sender.send(Some(n))
                        continue n + 1
                    } else {
                        sender.send(Empty)
                        continue n
                    }
                }
            }

            10.times { counterService.send(currentTask) }
            while var Some(n) = select int {
                println(`{n}`)
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
        var innerFactorial = for n = 20, result = 1 {
            if n <= 0 {
                result
            } else {
                continue n - 1, n * result
            }
        }

        OUTER: for n = 10 {
            if 0 < n {
                for {

                    // `continue` normally reiterates the current `for`, but an
                    // explicit label allows going to an outer one instead.
                    OUTER: continue n - 1
                }
            }
        }

        println(`factorial: {innerFactorial}`)
    }

    /*
     * All variables (including functions), packages, classes, getters, fields,
     * and methods are private unless explicitly opened up with the `public`
     * keyword. If the developer wants to expose them openly but only to
     * its own module, they can use the `internal` keyword.
     *
     * As can be seen in this function and the previous, Sylan allows functions to
     * stand alone in packages without being tied to classes or interfaces.
     */
    internal int factorial(int n) {
        switch n {
            0, 1 ->
                1
            n if n < 0 ->
                throw Exception("n cannot be less than 0")
            n ->
                /*
                 * Guaranteed tail call elimination will ensure a stack overflow
                 * does not occur.
                 */
                factorial(n * (n - 1))
        }
    }

    ignorable internal int printFactorial(int n) {
        /*
         * Sylan does not allow callers to throw away non-`void` function or methods
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
         * default. The lack of explicit `return`s mean there is always a single
         * exit point from a function, lambda, or method: the last expression.
         */
        result
    }

    private void twice<N>(N n, N f(N)) where
        N : Number
    {
        f(n) + f(n)
    }

    String double<N>(N n) where
        N : Add & ToString
    {
        (n + n).toString()
    }

    Optional<int> demoContexts() {

        /*
         * Haskell programmers will recognise this as a monadic do notation, which
         * Sylan calls "contexts". Non-Haskell programmers will be glad to know that
         * this feature can make tangled code such as optionality-checking chains
         * and validation code much cleaner.
         */
        with {
            var a <- Some(5)
            doSomething()
            var b <- Empty()
            willNotBeRun()
        }
    }

    package counter {

        enum Message {
            Increment,
            Reset(int),
            Get,
        }

        public void start(Task sender, int n = 0) {
            switch select Message {
                .Increment ->
                    start(sender, n + 1)
                .Reset(n) ->
                    start(sender, n)
                .Get {
                    sender.send(n)
                    start(sender, n)
                }
                timeout 10.seconds ->
                    throw Exception("timed out!")
            }
        }
    }

    interface Concatenate<T, Result = T> {
        public Result concatenate(T y)
    }

    interface Equals<T = Self> {
        public boolean equals(T other)

        /*
         * Sylan's interface methods can have bodies like Java 8's defender methods.
         * Unlike Java 8, private methods with bodies are also allowed with the
         * `default` keyword. Similar to C# but unlike Java, a default method body
         * in an interface can only be overridden if annotated with `virtual`.
         *
         * `virtual` is default for non-default methods since the lack of concrete
         * inheritance in Sylan means there is no other possible use case for them
         * except to be implemented by the implementor.
         */
        public default virtual notEquals(T other) {
            !equals(other)
        }
    }

    class Name {
        /**
         * Getters, hashing, equality checking, and the constructor all just exist
         * automatically. They can still be manually overridden if necessary though.
         * `close` is also automatically implemented for classes that implement
         * `AutoCloseable`, the default implementation just closing all
         * `AutoCloseable` fields. This can also be overridden if the default
         * implementation makes no sense.
         */

        internal String firstName
        internal String lastName
    }

    class Account implements ToString, Concatenate<Account> {
        /**
         * Classes can implement interfaces but cannot extend other classes.
         * Interfaces can extend other interfaces though. Concrete inheritance
         * generally causes more problems that it's worth and type embedding plus a
         * more powerful interface system makes up for the difference.
         */

        /*
         * Embedding a field hoists all of its _accessible_ methods and getters into
         * the class itself. Embedded methods take the lowest priority: methods in
         * the class itself and default methods in implementing interfaces both take
         * priority over embedded methods, although other embedded methods will
         * still call the embedder's method if calling it by name.
         *
         * As getters are also embedded, it can look awfully close to multiple
         * inheritance of state, but is not due to the lack of mutability and the
         * fact that only one getter of each given name and type can emerge from the
         * prioritisation.
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
        embed Name name

        int ageInYears

        ZonedDateTime _expiry = ZonedDateTime.now() + 1.year

        public Account(String firstName, String lastName) {
            println("instantiating an Account...")

            /*
             * Constructors are the only place in Sylan where assignments are
             * allowed outside the context of a binding.
             */
            name.firstName = firstName
            name.lastName = lastName
            ageInYears = 35
        }

        public override String toString() {
            `{firstName} {lastName} is {ageInYears} years old`
        }

        public override Account concatenate(Account a) {
            var firstName = firstName.concat(a.firstName)
            var lastName = lastName.concat(a.lastName)

            Account(
                .firstName,
                .lastName,
                ageInYears = ageInYears + a.ageInYears,
            )
        }

        /*
         * Getters are defined like methods but look like fields. Setters don't
         * exist because Sylan is an immutable language.
         */
        public String get name {
            `{firstName} {lastName}`
        }

        public boolean get locked {
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
    extend class Account implements Concatenate<Account, Result = String> {

        public override String concatenate(This that) {
            `{firstName} {that.firstName}`
        }
    }

    void demoPatternMatching() {
        var account1 = Account(firstName = "Tom", lastName = "Smith")
        var matchingLastName = "Smith"

        if var Account(firstName, lastName = .matchingLastName) as account
                = account1 {
            println(`Matching first name: {firstName}`)
            println(`Matching account: {account}`)
        }

        switch account1 {

            Account(locked = true), Account(firstName = "unlucky") ->
                println("LOCKED!")

            Account(expiry) if expiry < ZonedDateTime.now ->
                println("ALSO LOCKED")

            default {
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
    alias counter2 = counter
    alias Person = Account
    alias Showable = ToString
    alias factorial2 = factorial

    int maxBound = 5

    void demoClosures() {
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

        var f = { a ->
            println(a.toString())
        }

        f(account1)
        f(account2(firstName = "Emma"))

        var g = { a ->
            println("returning an account")
            a
        }

        var z = g(account1)

        var n = twice(3, { x -> x * 2 })
        println(`n == {n}`)
    }

    class NumericLiteralsClass implements ToString {

        // Non-overflowable numbers.
        int a = 5
        uint b = 5u
        decimal c = 10.0

        // Overflowable, machine-width numbers.
        byte d = 5u8
        uint16 e = 11u16
        uint32 f = 12u32
        uint64 g = 13u64
        int8 h = 15s8
        short i = 13s16
        int32 j = 7s
        long k = 7s64
        float l = 12f
        double m = 8f64

        public override String toString() {
            `{a}{b}{c}{d}{e}{f}{g}{h}{i}{j}{k}{l}{m}`
        }
    }

    class AutoCloseableDemo implements AutoCloseable {

        public AutoCloseableDemo() {
            println("Opened")
        }

        public override void close() {
            println("Closed")
        }
    }

    void demoAutoCloseables() {

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
        do {
            var closeable = using AutoCloseableDemo()
            println("Using closeable")
        }
        println("Leaving scope")

        // Prints "Entering scope", "Opened", "Using closeable", "Closed", and finally "Leaving scope".
    }

    void demo() {
        demoIteration()

        printFactorial(42)
        var x = twice(4, double)
        println(`{x}`)

        demoContexts()
        demoPatternMatching()
        demoClosures()

        println(NumericLiteralsDemo().toString())

        demoAutoCloseable()
    }

    /*
     * Top-level code is allowed, but only in the main package within the main
     * module. Code in other packages must be in functions or methods.
     */

    var optionalString = Some("test string")
    if var Some(s) = optionalString {
        println(s)
    }

    var c = Task { counter.start(currentTask) }
    5.times { c.send(counter.Message.Increment()) }

    c.send(counter.Message.Get())
    c.send(counter.Message.Increment())
    c.send(counter.Message.Get())

    // Should print 5 and then 6.
    2.times {
        var n = select int
        println(`{n}`)
    }

    print("""
    3 or more quotes can be used to create a custom string delimiter.
    This allows escaping almost any embedded content with few problems.
    This works with interpolated and raw strings too.
    """)

    println(r"Raw string in which escapes characters are now read literally: \n\r")

    var n = 4

    println(```
    Multiline interpolated strings: {n}
    ```)

    var x = do {
        println("Returning 5 to be bound as x...")
        5
    }
    println(`{x}`)

    /*
      A multiline comment.

      /*
        A nested multiline comment.
      */
    */

    demo()

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
* Grouped expressions with parentheses are not allowed at the top-level of an
  expression, only as subexpressions within larger expressions. This allows
  Sylan to parse separate expressions unambiguously without error prone
  constructs like JavaScript's automatic semicolon insertion. This fixes the
  problem of a grouped expression being ambiguously either an invocation of an
  invocable on the previous line or a new grouped expression.
* Unary operators are the only whitespace sensitive token in Sylan, being
  interpreted as binary with trailing whitespace and unary otherwise. This was
  inspired by the approach taken by Ruby to solve the same problem, a language
  that doesn't have problems with ambiguous statement termination in practice.

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
* While public fields are encouraged, unlike Java Sylan can add getters to
  intercept them at a later date without breaking API compatibility. This makes
  it fine to expose fields publicly. Setters do not exist due to Sylan being a
  fully immutable language.

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
  methods and and getters of a class and hoists them to the top level of the
  class itself. Fields must still be accessed via the field itself, however.
* If multiple embeds embed a getter or a method of the same signature, the
  earliest one takes priority. This is to follow the first-takes-priority system
  characteristics of MRO in inheritance, and to avoid backwards-compatibility
  breakages caused by the addition of new methods or getters to embedded types.
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
* Sylan is more opinionated than most language on how to use type annotations:
  all items should be 100% explicitly typed whereas all declarations scoped
  inside a method or function should be 100% implicitly typed. The language will
  not expose edge cases where type annotations are ever actually necessary in
  method or function bodies (like the "`:: Float` to disambiguate `Show`"
  problem of Haskell). This is why numbers have explicit type suffixes.
* This ensures APIs and program structure is rigid and explicitly typed while
  expressions are concise, without boilerplate, and utterly consistent rather
  than making type annotations a matter of taste.

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
* `Class::method` is a shorthand for `(o, ...args) -> o.method(...args)`, where
  the type of `o` is `Class`.
* As methods and functions are both higher-order, invoked the same way, and have
  the same type when passed around, there is no real loss of composability from
  being different constructs. They can be composed together easily: `var
  printDouble = Number::double # ToString::toString # println`.

### Pattern Matching

* Literals are matched as is.
* Composite types are matched as `Type(field1, field2 = match2, getterValue)`.
* Public fields and getters can be used.
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
* Finish operator overloading proposal.

## Implementation Details

For more documentation on the actual implementation of Sylan, read the Rust
documentation comments of the code. A good starting point is the documentation
of [the main top-level source
file](https://github.com/LouisJackman/sylan/blob/master/src/main.rs).

