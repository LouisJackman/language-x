# Language Versioning

* Keyword `v` can start a file to denote the source version.
* Has a version after it, e.g. `v1.0`
* If not present, assume to be the latest minor version of the earliest major
  release of the language, i.e. 1.x.
* Three things are versioned: the source, the tokens, and the AST.
* A source version pins its tokens version; a tokens version pins its AST
  version. So the versioning between the three isn't necessarily in lockstep,
  but it is fixed.