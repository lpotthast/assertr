use ::alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
    string::String,
    vec::Vec,
};
use ::core::ops::{Range, RangeInclusive};

/// A finite length, the capability behind the length assertions.
///
/// Implementing `HasLength` for your own type makes [`LengthAssertions`] available on it:
/// `is_empty`, `is_not_empty`, and `has_length`, plus the [`HasLengthOf`], [`IsEmpty`], and
/// [`IsNotEmpty`] matchers. Implement [`length`](Self::length). Override
/// [`is_empty`](Self::is_empty) only when your type can answer it more cheaply than by counting.
///
/// ```
/// use assertr::{assertions::HasLength, prelude::*};
///
/// #[derive(Debug)]
/// struct Playlist {
///     tracks: Vec<&'static str>,
/// }
///
/// impl HasLength for Playlist {
///     fn length(&self) -> usize {
///         self.tracks.len()
///     }
/// }
///
/// let playlist = Playlist { tracks: vec!["Intro", "Outro"] };
/// assert_that!(playlist).is_not_empty().has_length(2);
///
/// let failures = assert_that!(Playlist { tracks: Vec::new() })
///     .with_location(false)
///     .capture(|it| it.has_length(1));
/// assert_that!(failures[0].to_string()).contains("Actual length: 0");
/// ```
///
/// Failure reports render the subject and its length, so the length assertions need
/// [`ValueRenderer`](crate::renderer::ValueRenderer) support for your type and for `usize`. A
/// derived `Debug` provides both with the default renderer. `HasLength` is not part of the
/// prelude, so import it from [`assertr::assertions`](crate::assertions) to implement it.
///
/// Built-in implementations cover strings (counting bytes), the standard collections, arrays,
/// slices, and integer ranges. Integer ranges use their mathematical element count converted to
/// `usize`. Asking for the length of a range whose count cannot be represented by `usize` panics
/// with an explicit `range length exceeds usize::MAX` message.
///
/// [`LengthAssertions`]: crate::assertions::LengthAssertions
/// [`HasLengthOf`]: crate::matchers::HasLengthOf
/// [`IsEmpty`]: crate::matchers::IsEmpty
/// [`IsNotEmpty`]: crate::matchers::IsNotEmpty
pub trait HasLength {
    /// Returns the finite number of elements or bytes according to the type's native length.
    fn length(&self) -> usize;

    /// Returns whether [`HasLength::length`] is zero.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for str {
    fn length(&self) -> usize {
        str::len(self)
    }
}

impl HasLength for String {
    fn length(&self) -> usize {
        String::len(self)
    }
}

impl HasLength for Box<str> {
    fn length(&self) -> usize {
        str::len(self)
    }
}

impl HasLength for Cow<'_, str> {
    fn length(&self) -> usize {
        str::len(self)
    }
}

impl<T> HasLength for [T] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, const S: usize> HasLength for [T; S] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for Vec<T> {
    fn length(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> HasLength for VecDeque<T> {
    fn length(&self) -> usize {
        VecDeque::len(self)
    }
}

impl<K, V> HasLength for BTreeMap<K, V> {
    fn length(&self) -> usize {
        BTreeMap::len(self)
    }
}

impl<T> HasLength for BTreeSet<T> {
    fn length(&self) -> usize {
        BTreeSet::len(self)
    }
}

impl<T> HasLength for LinkedList<T> {
    fn length(&self) -> usize {
        LinkedList::len(self)
    }
}

impl<T> HasLength for BinaryHeap<T> {
    fn length(&self) -> usize {
        BinaryHeap::len(self)
    }
}

#[cfg(feature = "std")]
impl<K, V, S> HasLength for std::collections::HashMap<K, V, S> {
    fn length(&self) -> usize {
        std::collections::HashMap::len(self)
    }
}

#[cfg(feature = "std")]
impl<V, S> HasLength for std::collections::HashSet<V, S> {
    fn length(&self) -> usize {
        std::collections::HashSet::len(self)
    }
}

impl<T> HasLength for &T
where
    T: HasLength + ?Sized,
{
    fn length(&self) -> usize {
        T::length(self)
    }

    fn is_empty(&self) -> bool {
        T::is_empty(self)
    }
}

impl<T> HasLength for &mut T
where
    T: HasLength + ?Sized,
{
    fn length(&self) -> usize {
        T::length(self)
    }

    fn is_empty(&self) -> bool {
        T::is_empty(self)
    }
}

macro_rules! impl_has_length_for_integer_ranges {
    ($($type:ty),+ $(,)?) => {$(
        impl HasLength for Range<$type> {
            fn length(&self) -> usize {
                if self.start < self.end {
                    range_length(self.end.abs_diff(self.start), 0)
                } else {
                    0
                }
            }
        }

        impl HasLength for RangeInclusive<$type> {
            fn length(&self) -> usize {
                if self.is_empty() {
                    0
                } else {
                    range_length(self.end().abs_diff(*self.start()), 1)
                }
            }
        }
    )+};
}

/// Converts a range's mathematical length, `difference + extra`, to `usize`, panicking when it
/// does not fit.
fn range_length(difference: impl TryInto<usize>, extra: usize) -> usize {
    difference
        .try_into()
        .ok()
        .and_then(|difference| difference.checked_add(extra))
        .unwrap_or_else(|| panic!("range length exceeds usize::MAX"))
}

impl_has_length_for_integer_ranges!(
    usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128
);

#[cfg(test)]
mod tests {
    use super::HasLength;
    use crate::prelude::*;

    #[test]
    fn has_length_remains_usable_as_a_trait_object() {
        let values = [1, 2, 3];
        let value: &dyn HasLength = &values;

        assert_that!(value.length()).is_equal_to(3);
        assert_that!(value.is_empty()).is_false();
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn integer_ranges_use_their_mathematical_element_count() {
        macro_rules! check {
            ($($type:ty),+) => {$(
                let (one, five, nine): ($type, $type, $type) = (1, 5, 9);
                assert_that!(one..nine).has_length(8);
                assert_that!(one..=nine).has_length(9);
                assert_that!(five..=five).has_length(1);
                assert_that!(nine..one).has_length(0);
                assert_that!(nine..=one).has_length(0);
                let mut exhausted = five..=five;
                exhausted.next();
                assert_that!(exhausted).has_length(0);
            )+};
        }
        check!(
            usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128
        );

        macro_rules! check_signed {
            ($($type:ty),+) => {$(
                let (low, below_zero, centered): ($type, $type, $type) = (-9, -1, 4);
                assert_that!(low..below_zero).has_length(8);
                assert_that!(low..=below_zero).has_length(9);
                assert_that!(-centered..centered).has_length(8);
                assert_that!(-centered..=centered).has_length(9);
            )+};
        }
        check_signed!(isize, i8, i16, i32, i64, i128);

        // Full domains do not overflow the intermediate difference.
        assert_that!(u8::MIN..=u8::MAX).has_length(256);
        assert_that!(i8::MIN..i8::MAX).has_length(255);
        assert_that!(i8::MIN..=i8::MAX).has_length(256);
    }

    #[test]
    #[cfg(feature = "std")]
    fn unrepresentable_range_lengths_panic() {
        assert_that!(|| (0..u128::MAX).length())
            .panics()
            .has_type::<&str>()
            .is_equal_to("range length exceeds usize::MAX");
    }
}
