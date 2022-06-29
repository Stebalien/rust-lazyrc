# Lazy Rc

A `LazyRc` is an `Rc` but with slightly different tradeoffs. An `Rc<T>` puts the reference count
next to the referenced object in the same heap allocation, while a `LazyRc<T>` puts them in two
separate allocations (only allocating the reference count when cloned the first time).

1. Use an `Rc` if you have a stack allocated object or if copying the object is cheep.
2. Use a `LazyRc` if you already have a boxed object, are unlikely to share it, but want to be able
   to share it without copying it.

For example, if you have an API that returns a potentially large buffer in a `Vec<u8>` or a
`Box<[u8]>`, you may not want to convert this to an `Rc<[u8]>` as that will copy the entire buffer.
Without this library, you'd have to have to use an `Rc<Box<[u8]>>` however:

1. That requires additional pointer chasing to dereference.
2. That will always allocate even if the `Rc` is never shared.

_This_ library will allow you to convert that `Box<[u8]>` into a `LazyRc<[u8]>` with no allocations
until it's cloned the first time and no copying of the underlying buffer (ever).
