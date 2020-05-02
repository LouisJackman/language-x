# Sylan IL: the Sylan Intermediate Language

Sylan IL is based heavily on WebAssembly and BEAM.

A Sylan platform is two components: an Sylan IL processor, and a
runtime. The IL pushes or pops values off an implicit stack, or
manipulates tasks. Each function has its own stack (although most
implementations will likely use a single stack for multiple functions in a
single task in their implementation). Parameters are on the stack when a
function starts, and the function must end with zero or multiple elements left
on the stack depending on whether it declares a return value. All code is in a
function.

Arrays and indices arguments to task-related IL instructions _copy_ the array;
arrays never leak from tasks. Therefore, tasks never share memory
_conceptually_. When it says _send_, it always means _copy across_.

Arrays can store numeric data, functions, or tasks, but they cannot be mixed.
Being able to return multiple values, including multiple array types, is how
Sylan can return data that "mixes" numeric data, tasks, and functions.

Sylan IL platforms must not allow data from non-numeric arrays to be treated
like numeric data; if a process can bruteforce a numeric range until it finds a
valid task reference, Sylan's capability-based security goes out of the window.

## Data Types

Sylan IL has these data types, split into three categories plus indices. Indices
are the only type with an undefined size, taking the size of a target platform's
default memory index type.

-   Index

### Numeric

-   Int32
-   Int64
-   UInt32
-   UInt64
-   Float
-   Double
-   NumericArray
-   InitializerMutableNumericArray

### Functions

-   Fun
-   FunArray
-   InitializerMutableFunArray

### Tasks

-   Task
-   TaskArray
-   InitializerMutableTaskArray

Items appear on the stack as the function parameter list defines. For example,
a function `fun f(Double, Float, Task, Int32)` already has a stack in that
order when the function starts, with an `Int32` being on the top of the stack.
Therefore, starting that function with an operation that does not take an Int32
will cause a validation error. Swapping parameters around for the right
operation can be done with `peek`.

The types can not be mixed in IL, otherwise a validation error occurs. A
function or task reference are opaque and cannot, say, be incremented.
Numerics have a range of instructions that act on them.

Arrays are immutable, homogenous sequences of data.

InitializerMutableArrays are mutable, homogenous sequences of data that only
live for as long as the function that received them. The moment the function
ends, they are converted to immutable arrays and given back to the caller.

"Constructors" at the level of Sylan IL lose all of their type meaning. They
are just functions that receive a InitializerMutableArray as their first
argument. If constructors return basic data, they might not even bother with
that, just making a function that returns a single value. The final value of
that mutable array is used to implicitly create an immutable array that is
returned to the caller of `new`.

This means IL introduces two forms of mutability not present in Kernel Sylan;
local callstack mutability for the current function, and temporary mutability
of zeroed arrays given to constructors.

Allocation is implicit. Once a constructor is called with `new`, zeroed memory
appears out of nowhere. The runtime must clean it up when no longer referenced,
presumably via a garbage collector.

An array doesn't care about whether it's on the stack or the heap, so long as
immutability and reachability are upheld. As the caller invokes `new` to get
the array back, an implementation could do escape analysis to work out whether
to even bother putting in on the heap.

## Instructions

If you know WebAssembly, skip the numeric, bitwise, and comparison sections,
which were basically taken from WebAssembly unaltered.

Assume that all types above are `Type`, that `Int32` to `Int64` are `Int`,
`Float` and `Double` are `Floating`, `Int32` to `Double` are `Numeric`, `Array`
are all array types, and `ZeroedArray` are all zeroed array types. The
Sylan-like syntax below represents the stack before and after invocation. The
IL instruction set is as follows.

### Structure

mod $mod-name
pkg $pkg-name
fun \$fun-name(Type...) Type

### Control Flow

```
call     = ($pkg-name, $fun-name, ...) ...
callind  = (Fun, ...                 ) ...
begin    = (                         )
end      = (                         )
cont     = (                         )
br       = (                         )
brif     = (Int                      )
```

### Numeric Maths

```
[Int & Float].add     = (Numeric, Numeric) Numeric
[Int & Float].sub     = (Numeric, Numeric) Numeric
[Int & Float].mul     = (Numeric, Numeric) Numeric
[Int & Float].div     = (Numeric, Numeric) Numeric
[Int & Float].divu    = (Numeric, Numeric) Numeric

[Int].rem             = (Numeric, Numeric) Numeric
[Int].remu            = (Numeric, Numeric) Numeric
[Int].convert.[Float] = (Int             ) Float

[Float].min           = (Float, Float) Float
[Float].max           = (Float, Float) Float
[Float].copysign      = (Float, Float) Float
[Float].abs           = (Float       ) Float
[Float].neg           = (Float       ) Float
[Float].sqrt          = (Float       ) Float
[Float].ceil          = (Float       ) Float
[Float].floor         = (Float       ) Float
[Float].trunc.[Int]   = (Float       ) Int
[Float].nearest.[Int] = (Float       ) Int
```

### Numeric Bitwise

```
[Int].not     = (Int     ) Int
[Int].and     = (Int, Int) Int
[Int].xor     = (Int, Int) Int
[Int].or      = (Int, Int) Int
[Int].shl     = (Int, Int) Int
[Int].shr     = (Int, Int) Int
[Int].shru    = (Int, Int) Int
[Int].clz     = (Int     ) Int
[Int].ctz     = (Int     ) Int
[Int].popcnt  = (Int     ) Int

```

### Comparison

```
[Int].ieqz  = (Int     ) Int
[Int].ieq   = (Int, Int) Int
[Int].ine   = (Int, Int) Int
[Int].ilt   = (Int, Int) Int
[Int].iltu  = (Int, Int) Int
[Int].igt   = (Int, Int) Int
[Int].igtu  = (Int, Int) Int
[Int].ile   = (Int, Int) Int
[Int].ileu  = (Int, Int) Int
[Int].ige   = (Int, Int) Int
[Int].igeu  = (Int, Int) Int

[Float].ieq   = (Float, Float) Float
[Float].ine   = (Float, Float) Float
[Float].ilt   = (Float, Float) Float
[Float].igt   = (Float, Float) Float
[Float].ile   = (Float, Float) Float
[Float].ige   = (Float, Float) Float
```

### Stack Manipulation

[Type].peek = (Offset) Type

### Task-local Storage

```
[Type].init  = (Offset, $fun-name, ...) Array
[Array].load = (Array, Offset         ) Type

[InitializerArray].load.[Type]  = (ZereodArray, Offset      ) Type
[InitializerArray].store.[Type] = (ZereodArray, Offset, Type)
```

### Tasks

```
spawn   = (Fun                                    ) Task
current = (                                       ) Task
send    = (Task, Memory, Offset, TaskTable, Offset)
receive = (Memory, Offset, UInt64                 )
kill    = (Task                                   )
```

## Behaviour

Spawning a task automatically links it to the spawner process. Linking just
means one thing: if the child dies, its exception is sent to the parent with
the failing task reference. To "unlink" a task, a parent task need only ignore
anything the child sends back, or even just kill the child task.

So many intrinsics are dedicated to tasks because runtimes can implement them
however they like, but Sylan must be able to depend on some fundamentals.
Killing tasks will likely be the hardest feature to implement: obvious
implementations like most OS threading systems won't work here, although OS
processes and green-threading might.

All intrinsics implemented by a platform have a direct connection to an omitted
Sylan IL instruction, except one: how to get the extern task. On start up Sylan
sends the extern task reference to the main task, the sole starting item in its
inbox. If the main task wants to give more powers to subtasks, it must pass the
that value as a message to a subtask, which presumably will set that same
dynamically-scoped variable to the value.

The `extern` task is defined by the runtime. This task handles all interaction
with the outside world. Sylan reserves messages whose allocated message leads
with the bit `1` according to interpretation as a `UInt64`; a platform must
interpret all of them. Any other sort of message, i.e. an allocation which leads
with the bit `0` is entirely up to the runtime.

So far, the only Sylan message to be handled has leading bits `10` and
interprets the second `UInt64` and onwards in the message as a UTF-8-encoded
error message to be meaningfully conveyed in the platform. This is for
catastrophic failures in Sylan core runtime that cannot be handled with
normal exceptions and linked tasks.
