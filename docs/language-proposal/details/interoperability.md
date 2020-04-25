# Interoperability

* `extern` allows calling functions that are either statically linked in or via
  a named dynamically linked library. `extern` types are defined by Sylan
  itself. `extern` finals refer to extern variables in other compiled artefacts,
  but won't assume the other artefact actually keeps it constant, thereby
  employing memory fences on access.
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