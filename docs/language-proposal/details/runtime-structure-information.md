# Runtime Structure Information

* No reflection.
* No runtime annotations.
* Use compile-time programming instead.
* Reduces magic, as compile-time metaprogramming cannot happen at random points
  during a running application unless `eval` exists.
* Improves performance as metaprogramming is done at compile-time.