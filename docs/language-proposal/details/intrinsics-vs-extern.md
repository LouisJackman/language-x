# Intrinsics vs Extern

## Intrinsics

Intrinsics are either _language intrinsics_ or _IL intrinsics_.

Language intrinsics are implemented by the language pipeline before IL is
emitted. A Sylan platform handling IL needn't concern themselves with them.

IL intrinsics are handled by Sylan platforms. Compilation doesn't worry about
them, emitting IL and treating it like a blackbox.

IL intrinsics are always implementations of IL instructions, with one exception:
the intrinsic to obtain the extern task, which has the special behaviour of
only providing itself to the main task.

Intrinsics are wired straight into Sylan or solve bootstrapping problems
with it. They cannot be changed by users without forking Sylan itself, or
one of its platforms.

## Extern

Extern items get translated to messages sent to the extern task. This is the
gateway to the outside world. It can be removed from tasks by simply not
passing it to those tasks via lexical scope; the outside world can be sealed off
entirely from subtasks, or only opened up selectively via APIs.

The lack of runtime reflection and the refusal of the Sylan IL to represent
tasks as arbritary data in memory should make such APIs secure, assuming the
users don't make mistakes.

See [Capability-based Security](capability-based-security.md) for more details.
