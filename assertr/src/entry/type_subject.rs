use core::{any::type_name, marker::PhantomData, mem::needs_drop};

use crate::{AssertThat, actual::Actual, mode::Panic};

/// A zero-sized subject standing for the type `T` itself.
///
/// [`assert_that_type`] creates it. It represents properties of `T` rather than a value, and its
/// accessors feed further facts about `T` into
/// [`AssertThat::satisfies_owned`].
pub struct Type<T> {
    phantom: PhantomData<T>,
}

impl<T> Type<T> {
    /// Creates a subject representing the type `T`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    /// The name of `T`, as reported by [`core::any::type_name`].
    #[must_use]
    pub fn get_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    /// Returns [`core::mem::needs_drop`] for `T`.
    ///
    /// A `false` result guarantees that dropping `T` has no side effects. A `true` result is
    /// conservative and does not guarantee that dropping it runs code.
    #[must_use]
    pub fn needs_drop(&self) -> bool {
        needs_drop::<T>()
    }

    /// The size of a `T` in bytes, as reported by [`core::mem::size_of`].
    #[must_use]
    pub fn size(&self) -> usize {
        size_of::<T>()
    }
}

impl<T> Default for Type<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts an assertion about the type `T` itself rather than about a value.
///
/// The subject is a [`Type<T>`](Type). With the `std` feature, `needs_drop()` asserts that
/// [`core::mem::needs_drop`] returns `true`. Other properties are available through [`Type`]'s
/// accessors and [`AssertThat::satisfies_owned`]:
///
/// ```
/// use assertr::prelude::*;
///
/// # #[cfg(feature = "std")] {
/// assert_that_type::<String>().needs_drop();
/// # }
///
/// assert_that_type::<[u8; 4]>().satisfies_owned(|it| it.size(), |size| {
///     size.is_equal_to(4);
/// });
/// ```
#[must_use]
pub fn assert_that_type<T>() -> AssertThat<'static, Type<T>, Panic> {
    AssertThat::new_panicking(Actual::Owned(Type::<T>::new())).with_expression(type_name::<T>())
}
