use core::{
    any::type_name,
    fmt,
    ops::{Deref, DerefMut},
};

/// Wraps a value that cannot be rendered, so diagnostics show its type name instead of its
/// contents.
///
/// Some assertions render a value even though it is not what the test is about. `is_err` and
/// `err` show an unexpected `Ok` value, so they need a renderer for the `Ok` type. When that type
/// has no `Debug` implementation, wrap it:
///
/// ```
/// use assertr::{prelude::*, renderer::Opaque};
///
/// struct Client; // No `Debug`.
///
/// fn connect(address: &str) -> Result<Client, String> {
///     Err(format!("cannot resolve {address}"))
/// }
///
/// assert_that_owned!(connect("nowhere").map(Opaque::new))
///     .err()
///     .is_equal_to("cannot resolve nowhere");
/// ```
///
/// A failure then shows the wrapped value as its type name in angle brackets, such as
/// `<my_crate::Client>`. The default renderer formats it through `Debug`. A custom renderer needs
/// its own `ValueRenderer<Opaque<T>>` implementation to show it.
///
/// `Opaque` dereferences to the wrapped value and forwards equality and ordering to it, so it
/// can be compared and inspected like the value itself.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opaque<T>(pub T);

impl<T> Opaque<T> {
    /// Wraps `value`.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Returns the wrapped value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Opaque<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Opaque<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Opaque<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// Shows only the wrapped type's name, never its contents.
impl<T> fmt::Debug for Opaque<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", type_name::<T>())
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use indoc::formatdoc;

    use super::Opaque;
    use crate::prelude::*;

    struct Client;

    #[test]
    fn renders_the_type_name_instead_of_the_contents() {
        assert_that!(format!("{:?}", Opaque::new(Client)))
            .is_equal_to(format!("<{}>", core::any::type_name::<Client>()));
    }

    #[test]
    fn makes_results_with_unrenderable_ok_types_checkable() {
        let result: Result<Client, String> = Err(String::from("refused"));
        assert_that_owned!(result.map(Opaque::new))
            .is_err()
            .err()
            .is_equal_to("refused");
    }

    #[test]
    fn reports_an_unexpected_ok_value_by_its_type() {
        let result: Result<Opaque<Client>, String> = Ok(Opaque::new(Client));
        assert_that!(|| {
            assert_that!(result).with_location(false).is_err();
        })
        .panics()
        .message()
        .is_equal_to(formatdoc! {"
            -------- assertr --------
            Expression: `result`

            Actual: Ok(
                <{client}>,
            )

            is not the expected variant

            Expected: Result::Err
            -------- assertr --------
        ", client = core::any::type_name::<Client>()});
    }

    #[test]
    fn forwards_access_and_comparisons_to_the_wrapped_value() {
        let mut wrapped = Opaque::from(1);
        *wrapped += 1;
        assert_that!(*wrapped).is_equal_to(2);
        assert_that!(Opaque::new(1) < Opaque::new(2)).is_true();
        assert_that!(wrapped.into_inner()).is_equal_to(2);
    }
}
