# Capabilities

The main task gets the extern task sent to its inbox on startup. No other
task gets this by default. If the main task wants to delegate ambient
authority to spawned tasks, it must also sent it to them.

`spawnTrusted` does this by default, making it a good choice for small
scripts and programs that don't want to worry about it too much.

`spawn` does not. Therefore, items created with `spawn` are effectively
sandboxed. When invoking, say, IO operations, they'll fail because they'll
fail to receive an extern task to delegate system calls to.

Refs can be sealed off, making it impossible to pull their value out again.
Refs are just tasks underneath. Tasks can also perform other security-related
tasks like wrapping values in an envelope and giving away the only key that
can unlock it.

This works because Sylan does not share memory across tasks and does not
support runtime reflection or mutability.

There is nothing special about `spawnTrusted`; it's just Sylan code that
receives the current ambient authority, sends one back to itself for other
`spawnTrusted` calls, and sends the ambient authority into the new task via
lexical scoping.

This means a program can drop unneeded rights by receiving the ambient authority
and just throwing it away, finishing running the rest of the program with fewer
capabilities passed in. A convenience function `dropRights` does this.

There's an interesting capability-based security consequence: tasks are always
opaque in the runtime. If a task doesn't receive another task, it simply cannot
communicate with it. Only the main task starts with access to `extern`; if it
doesn't pass it to a spawned process, it's effectively sandboxed.

See [Sylan Intermediate Language](sylan-il.md) to see how tasks remain
unforgeable in the VM.
