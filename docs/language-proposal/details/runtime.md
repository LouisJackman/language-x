# Runtime

-   It will probably be heavily BEAM-inspired.
-   Must do tail call elimination with a "shadow stack" to aid debugging, inspired
    by Safari's JavaScript runtime's implementation of ES6 tail call elimination.
-   No mutable data except the execution of tasks over time.
-   Lightweight processes. Immutability means changes can only be sent via
    messages or tracked via services external to the Sylan program.
-   Initial toy implementation to use threads. Real implementation can use
    userland scheduler with remote process support.
-   To handle remote processes, Tasks need node IDs.
-   ...and nodes need network addresses or localhost.
-   Per-task GC in theory; probably global GC in first implementation for
    simplicity. (Perhaps that's OK if only a single task's world gets stopped at
    any time.)
-   Persistent data structures, which should be definable as a library rather than
    baked into the language itself.
-   Async IO; use a library like Tokio. OK to block in the prototype, but don't
    add any language features that break compatibility with async in the proper
    version.
