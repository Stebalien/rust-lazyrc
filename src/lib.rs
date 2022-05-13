use std::fmt::{Debug, Display};
use std::{cell::Cell, ops::Deref, ptr::NonNull};

/// A lazy ref-cell that acts like a box until cloned.
///
/// Use when you have pre-boxed data that's rarely shared
pub struct LazyRc<T: ?Sized> {
    data: NonNull<T>,
    share_count: Cell<*const Cell<usize>>,
}

impl<T: ?Sized> Default for LazyRc<T>
where
    Box<T>: Default,
{
    fn default() -> Self {
        let boxed: Box<T> = Default::default();
        Self::new(boxed)
    }
}

impl<T: ?Sized> Debug for LazyRc<T>
where
    T: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> Display for LazyRc<T>
where
    T: Display,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: ?Sized> Deref for LazyRc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized> LazyRc<T> {
    #[inline]
    pub fn new(inner: Box<T>) -> Self {
        unsafe {
            LazyRc {
                // Box always returns a non-null pointer.
                data: NonNull::new_unchecked(Box::into_raw(inner)),
                share_count: Cell::new(std::ptr::null()),
            }
        }
    }
}

impl<T: ?Sized> From<Box<T>> for LazyRc<T> {
    fn from(value: Box<T>) -> Self {
        Self::new(value)
    }
}

impl<T> From<Vec<T>> for LazyRc<[T]> {
    fn from(value: Vec<T>) -> Self {
        Self::new(value.into_boxed_slice())
    }
}

impl From<String> for LazyRc<str> {
    fn from(value: String) -> Self {
        Self::new(value.into_boxed_str())
    }
}

impl<T: ?Sized> Clone for LazyRc<T> {
    fn clone(&self) -> Self {
        unsafe {
            if let Some(counter) = self.share_count.get().as_ref() {
                counter.set(counter.get() + 1);
            } else {
                self.share_count
                    .set(Box::into_raw(Box::new(Cell::new(2))) as *const _);
            }

            Self {
                data: self.data,
                share_count: self.share_count.clone(),
            }
        }
    }
}

impl<T: ?Sized> Drop for LazyRc<T> {
    fn drop(&mut self) {
        unsafe {
            let counter = self.share_count.get();
            if !counter.is_null() {
                {
                    let counter_ref = &*counter;
                    let count = counter_ref.get();
                    if count > 1 {
                        counter_ref.set(count - 1);
                        // Nothing to deallocate.
                        return;
                    }
                }
                // And drop the counter.
                Box::from_raw(counter as *mut Cell<usize>);
            }
            Box::from_raw(self.data.as_ptr());
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    static DROP_COUNT: AtomicU32 = AtomicU32::new(0);

    struct DropTest {
        dropped: bool,
    }

    impl DropTest {
        fn new() -> Self {
            Self { dropped: false }
        }
    }

    impl Drop for DropTest {
        fn drop(&mut self) {
            assert!(!self.dropped);
            self.dropped = true;
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_owned() {
        drop(LazyRc::new(Box::new((1, DropTest::new()))));
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
        let thing = LazyRc::new(Box::new((Cell::new(2), DropTest::new())));
        let thing2 = thing.clone();
        let thing3 = thing.clone();
        assert_eq!(thing2.0.get(), 2);
        assert_eq!(thing3.0.get(), 2);
        assert_eq!(thing.0.get(), 2);

        thing2.0.set(5);

        assert_eq!(thing2.0.get(), 5);
        assert_eq!(thing3.0.get(), 5);
        assert_eq!(thing.0.get(), 5);

        drop(thing);
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
        drop(thing3);
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
        drop(thing2);
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 2);
    }
}
