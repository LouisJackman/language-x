# The Sylan Programming Language

[![pipeline status](https://gitlab.com/sylan-language/sylan/badges/master/pipeline.svg)](https://gitlab.com/sylan-language/sylan/-/commits/master) [![coverage report](https://gitlab.com/sylan-language/sylan/badges/master/coverage.svg)](https://gitlab.com/sylan-language/sylan/-/commits/master)

**Warning: this project is still early stage and is a long way from completion.
See the "Done so Far" section for more details.**

## Secure yet Expressive Computation

Sylan is a work-in-progress programming language aiming to mix
transparently-distributed programming, compile-time meta-programming, easy
distribution, and a powerful type system with an approachable syntax.

Many language features prone to security holes in other languages have been
removed or reconsidered in Sylan. Sylan seeks to avoid the common problem of
increased security crippling developer's productivity.

It aims to be an application language designed for web applications, network
services, command line programs, developer tooling, and scripting.

It is intended to scale from small scripts to large codebases, remove footguns,
and prioritise large-scale software engineering concerns over interesting
computer science concepts. It should take versioning and backwards compatibility
seriously and ease distribution by producing single, statically-linked
executables with no required external runtimes.

This repository is hosted [on
GitLab.com](https://gitlab.com/sylan-language/sylan) in the [Sylan
group](https://gitlab.com/sylan-language). If you're seeing this on GitHub,
you're on my personal GitHub mirror. [Go to
GitLab](https://gitlab.com/sylan-language/sylan) to contribute.

## Example

$EXAMPLE_SOURCE

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

* Remove or reconsider features in other programming languages that have
  historically caused security bugs in software.
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

See more details in the [language proposal
documentation](docs/language-proposal/details).

## Implementation Details

For more documentation on the actual implementation of Sylan, read the Rust
documentation comments of the code. A good starting point is the documentation
of [the main top-level source
file](https://gitlab.com/sylan-language/sylan/blob/master/src/main.rs).

