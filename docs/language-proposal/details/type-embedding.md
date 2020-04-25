# Type Embedding

* The `embed` keyword allows classes to embed other classes. This just takes all
  methods of a class and hoists them to the top level of the class itself.
* If multiple embeds embed a method of the same signature, the
  earliest one takes priority. This is to avoid backwards-compatibility
  breakages caused by the addition of new methods to embedded types.
* There is no sub-typing relationship established by embedding whatsoever.
  Sylan does not support subtyping.