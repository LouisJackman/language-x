# Modules and Packages

* Packages live within modules.
* Modules are the most coarse encapsulation boundary developers have.
* Modules declare which version of Sylan they use; all versions of Sylan should
  be able to import modules from older versions of Sylan.
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