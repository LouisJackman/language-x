# Intrinsics vs Extern

## Intrinsics

Intrinsics are always implementations of IL instructions, with one exception:
the intrinsic to obtain the extern task, which has the special behaviour of
only providing itself to the main task.

Intrinsics are wired straight into Kernel Sylan or solve bootstrapping problems
with it. They cannot be changed by users without forking Sylan itself.

## Extern

Extern items get translated to messages sent to the extern task. This is the
gateway to the outside world. It can be removed from tasks by simply not
passing it to those tasks via lexical scope; the outside world can be sealed off
entirely from subtasks, or only opened up selectively via APIs.

The lack of runtime reflection and the refusal of the Sylan IL to represent
tasks as arbritary data in memory should make such APIs secure, assuming the
users don't make mistakes.
