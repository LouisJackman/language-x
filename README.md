# The Sylan Programming Language

[![CircleCI](https://circleci.com/gh/LouisJackman/sylan.svg?style=svg)](https://circleci.com/gh/LouisJackman/sylan)
[![codecov](https://codecov.io/gh/LouisJackman/sylan/branch/master/graph/badge.svg)](https://codecov.io/gh/LouisJackman/sylan)
![](https://img.shields.io/github/license/LouisJackman/sylan.svg)

**Warning: this project is still early stage and is a long way from completion.
See the "Done so Far" section for more details.**

Sylan is a work-in-progress programming language aiming to mix
transparently-distributed programming, compile-time meta-programming, easy
distribution, and a powerful type system with an approachable syntax.

It aims to be an application language designed for web applications, network
services, command line programs, developer tooling, and scripting.

It is intended to scale from small scripts to large codebases, remove footguns,
and prioritise large-scale software engineering concerns over interesting
computer science concepts. It should take versioning and backwards compatibility
seriously and ease distribution by producing single, statically-linked
executables with no required external runtimes.

## Example

```sylan
#!/usr/bin/env sylan

fun fizzBuzz
    /** The (in)famous FizzBuzz. */
(n Int) Int {
    1.up(to: n).each -> {
        switch {
            0 == (it % 15) { "FizzBuzz" }
            0 == (it % 5) { "Fizz" }
            0 == (it % 3) { "Buzz" }
            _ { it }
        } |> println
    }
}

enum List[of Element]
    /** Time for some data structures. Note: real pattern matching. */
(
    Node(of element Element, next This),
    Nil,
) {
    fun each(do (element Element)) {
        switch this {
            Node(element, next) {
                do(element)
                next.each(do)
            }
            Nil {
                Void
            }
        }
    }
}

// Now, Erlang-style concurrency and nested packages.

package counter {
    enum Message(
        Increment,
        Reset(with n Int),
        Get,
    )

    fun public start(task Task, n Int: 0) {
        select Message {
            .Increment {
                start(task, n: n + 1)
            }
            .Reset(n) {
                start(task, n)
            }
            .Get {
                send(n, to: task)
                start(task, n)
            }
            timeout 10.seconds {
                throw Exception("timed out!")
            }
        }
    }
}

var c = do -> {
    var parent = currentTask
    Task -> {
        counter.start(task: parent)
    }
}
5.times -> {
    send(counter.Message.Increment, to: c)
}
send(counter.Message.Get, to: c)
send(counter.Message.Increment, to: c)
send(counter.Message.Get, to: c)
2.times -> {
    var n = select Int
    println($"{n}")
}
send(counter.Message.Reset(to: 0), to: c)

class Name
    /** Easy data definition with keyword arguments and defaults. */
(
    var public first String: "James",
    var public last String: "Bond",
) implements ToString {
    fun public override toString String {
        $"The name is {lastName}, {firstName} {lastName}."
    }
}

fun demoContexts Optional[Int]
    /** Finally, the esoteric for Haskell fans: applicative notation. */
{
    with {
        var x = Some(5)?
        Empty?
        println("Will not be run.")
        Empty
    }
}

// There's much more! See `examples/exhaustive_example.sy` for a demo of all
// language features, with explanatory comments for each.
```

## Contents

* [Example](#example)
* [Installation](#installation)
* [Done so Far](#done-so-far)
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
with invocations like `docker run -it --rm -v "$PWD":/home/user sylan
examples/main.sy`.

Sylan, being written in Rust, uses standard Rust tooling such as Cargo for
development.  If you're familiar with it, use it as you would with any other
Rust project. Cargo can be installed by [installing Rust
itself](https://www.rust-lang.org/tools/install), and its available commands
can be enumerated by running `cargo help`.

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

A runtime will be needed for preemptive concurrency and light weight tasks. Go
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
* Guarantee tail-call elimination with Safari-like shadow stacks to aid debugging.

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
* They declare which modules they depend on.
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
* Infix operators are supported, but only a limited set of known operators
  exist. This is to avoid parsing ambiguities and to avoid disambiguating with
  whitespace. There are no prefix operators, but there are two non-overriddable
  postfix operators. Despite being a limited set, each can be overridden by
  user libraries.

### Types

* Externs and user-defined.
* No difference between them from the user's perspective except for some of them
  having literals baked into the language  and externs being predefined by the
  compiler and runtime. No Java-like primitive vs object distinction. This is
  different to bindings and functions, which can be `extern` and can _not_
  defined by Sylan itself but instead by an external artefact.
* Constructors are special; this is done to allow invocable-style instantiations
  while avoiding things like statics, needing to detach constructors from class
  definitions, or having a more complicated initialisation syntax.
* `Void` is an actual type, like `()` in Haskell.  Defining a function or method
  as returning `void` is a special-case that discards the result of final
  non-void expression and returns the void value instead. Every function,
  method, and lambda returning a value, rather than having "procedures" without
  return values, avoids special-cases when composing invocables in various ways.
* Generics like Rust, as in monomorphic type erasure.
* Support higher-kinded types, but keep an eye on projects like Dotty to see
  what type-soundness issues they encounter. Perhaps implement a more restricted
  version of it.
* A "type" is really a value-parameterised package; every field is actually a
  getter function that has been parameterised hard-coded return values.
  Interpreted another way, a package is just a singleton type.
* Setters do not exist due to Sylan being a immutable language.
* Multiversal equality rather than universal equality.

### Bindings

* Types, variables, functions are all in the same namespace. Although types and
  values exist in two different phases, compile-time and runtime respectively,
  Sylan still won't allow identifiers between the two to clash.
* Types should start with capital letters, and values with lowercase letters.
* Methods are namespaced to their types, although just deeper in the namespacing
  hierarchy rather than in a completely different standalone global namespace.
* Shadowing is not allowed in the same block except for pseudoidentifiers, which
  use keyphrases. Shadowing is allowed between packages and their subpackages
  and classes in the same file though; a particular example is methods being
  able to shadow package-wide functions of the same name. Explicitly specifying
  the package lets subpackages and classes disambiguate which identifier they
  mean, which the `this.package` pseudoidentifier can help with.
* There are nine psuedoidentifiers: `...`, `_`, `continue`, `it`, `this`, `This`
  `this.module`, `this.package`, and `super`. `continue` and `it` are _almost_
  dynamically scoped, changing implicitly throughout scopes based on the
  context. `continue` binds to the innermost non-labelled `for` iteration
  function, `it` is the innermost syntactically-zero-parameter lambda's sole
  parameter, and `_` is an ignored value in a binding and a partial-application
  notation for the innermost invocation. `this` refers to the current object,
  `This` to the current type, `this.module` to the current module, and
  `this.package` to the current package.
* Types and variables can both be thought of as "bindings", just one at
  compile-time and another at runtime. Never the twain shall meet, at least
  until Sylan designs how they should interoperate if at all. This will depend
  on how compile-time metaprogramming is implemented and whether Sylan decides
  to implement any form of dependent typing.
* Classes ultimately can only expose methods and nothing else. As field
  declarations are always autogenerated getters.

### Type Hierarchies

* Final classes and typeclass-like interfaces. No concrete inheritance or
  abstract classes. Discourages "type taxonomy". No dynamic dispatch; acheive
  runtime polymorphism via Sylan enums, a.k.a. "sum types" or "discriminated
  unions".
* `super` is to either disambiguate which interface's method to delegate to, or
  to fallback to the auto-generated constructor or other auto-generated method
  like `hashcode` or `equals` in user-defined constructors and methods. It does
  not deal with concrete class inheritance.
* Interfaces can provide default method implementations, which can be overridden
  in implementing classes. `override` on a method makes such overrides clear.
  Method shadowing does not exist; if it has the same name, it must override,
  and the developer must explicitly state that with `override`.
* There is no method resolution order such as Dylan- and Python 3-inspired MRO.
  Dynamic dispatch does not exist, since all runtime polymorphism should be done
  via enums instead. Interfaces are more like Haskell typeclasses than Java's
  interfaces in this regard. The ability to statically extend types with new
  interface implementations in different packages covers a lot of
  dynamic-dispatches's use cases.

### Type Embedding

* The `embed` keyword allows classes to embed other classes. This just takes all
  methods of a class and hoists them to the top level of the class itself.
* If multiple embeds embed a method of the same signature, the
  earliest one takes priority. This is to avoid backwards-compatibility
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
  type inference; all function-local variables _must_ infer types, whereas
  declared items in classes and packages must always be explicitly typed. This
  includes package functions, class methods, constructors, and class fields.
  There's no choice a developer must make about whether or not to use type
  inference. Also, all `var`s must infer types whereas `final`s must spell
  them out explicitly.
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
* `::` is used to pick something up without invoking it; by default, with no
  parentheses, Sylan will invoke something with zero-arguments.
* Invocations look like `Class.method::`, `package.function::`, and
  `object.method::`.
* As methods and functions are both higher-order, invoked the same way, and have
  the same type when passed around, there is no real loss of composability from
  being different constructs. They can be composed together easily: `var
  printDouble = Number.double:: # ToString.toString:: # println::`.

### Pattern Matching

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

### Matching in Switch and Select

* They both have cases which each match one or more patterns separated by
  commas.
* Both have `default` clauses as a fallback "match all" clause.
* `switch` is exhaustive: a compiler error happens if not all cases are covered.
* `select` is exhaustive for the selected type, but sends logs of non-matching types to the `noReceiver`
  task's mailbox (whose behaviour can be manually overridden). When `select`
  happens, all messages with non-matching types are logged to the noReceiver mailbox
  until a message with a matching type is encountered. If the mailbox is empty, it waits
  until messages are sent, repeating the same behaviour until a matching message
  is encountered.
* `select` blocks the current task until someone sends the process a message of
  the specified type with a match. `timeout` clauses are available.

### Invocations

* Methods, lambdas, functions, classes, and objects can be invoked with `()`,
  except nullary invocables which are invoked by just referring to them
  directly.
* Any of those are referred to "invocables" when used like that.
* Passing a nullary invocable without invoking it is done with the invocable
  handle posfix operator: `::`.
* Invoking a method, lambda, or a function does as one would expect; invoking a
  class constructs an instance; invoking an object allows non-destructive
  updates, which is why just referring to an object directly works, as there's
  no distinction between a no-change update and the identity of an immutable
  object.
* Arguments can have default values.
* Arguments have labels, like Swift. Like Swift, the label is the same as the
  parameter if omitted, and the `_` label allows callers to drop it.
* Two special parameter types exist: variadic and variadic entry. The former
  allows an variable amount of arguments, whereas the latter allows a variable
  amount of `sylan.lang.Entry` arguments with syntactical sugar.  The latter is
  primarily for providing a good syntax for constructing map types that
  user-defined types can use.
* The compiler knows whether an argument is positional or labelled based on the
  `_` label; one parameter can't be both. This means the compiler, if seeing a
  unexpected positional argument with a variable name matching a keyword
  argument, can automatically convert it into a keyword argument.
* Passing `_` for one or more arguments partially applies the invocation,
  returning a new function value with the non-underscore arguments evaluated and
  partially applied to the result. This allows, for example, partially-applying
  a non-destructive object update, partially applying a function, or
  partially-applying the instantiation of a class.
* Passing `_` to a labelled argument transforms it into a partially applied
 . positional argument, to assist with functional operations.

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
* The `using` keyword can prefix any `AutoCloseable` value, in which case it is
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
  a named dynamically linked library. `extern` types are defined by Sylan itself. `extern` finals refer to extern variables in other compiled artefacts, but won't assume the other artefact actually keeps it constant, thereby employing memory fences on access.
* Public exposed symbols in Sylan are accessible by either statically linking
  the result into another executable or by creating a dynamically linked library
  when building the Sylan program and then referring to it dynamically from
  other executables.
* As Sylan does not support ad hoc overloading or defining new, arbitrary
  operators, symbol demangling is straightforward. One underscore denotes a
  package change while two indicate a method belonging to a type. E.g.:
  `sylan.util.collections.HashMap#put` becomes
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
* Subtyping for built in types, like subranges across integers, Ada-style?
* Reject changes that break backwards compatibility if the major version isn't
  bumped, similar to Elm.
* Replace auto-generated methods with a more consistent `derives`/`deriving`
  approach, a la Rust and Haskell.

## Implementation Details

For more documentation on the actual implementation of Sylan, read the Rust
documentation comments of the code. A good starting point is the documentation
of [the main top-level source
file](https://github.com/LouisJackman/sylan/blob/master/src/main.rs).

