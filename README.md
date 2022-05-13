# Lazy Rc

A `LazyRc` is an `Rc` but with slightly different tradeoffs. An `Rc<T>` puts the reference count
next to the referenced object in the same heap allocation, while a `LazyRc<T>` puts them in two
separate allocations (only allocating the reference count when cloned the first time).

1. Use an `Rc` if you have a stack allocated object, or if copying the object is cheep.
2. Use a `LazyRc` if you already have a boxed object, and want to make it shareable without copying anything.
