use ::alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
    string::String,
    vec::Vec,
};
use ::core::ops::{Range, RangeInclusive};

/// A value whose finite length can be inspected by
/// [`LengthAssertions`](crate::assertions::core::length::LengthAssertions).
///
/// Implement it to make `is_empty`, `is_not_empty`, and `has_length` available on a custom type.
/// Built-in implementations cover strings, collection families, and integer ranges.
///
/// Integer ranges use their mathematical element count converted to `usize`. Asking for the length
/// of a range whose count cannot be represented by `usize` panics with an explicit `range length
/// exceeds usize::MAX` message.
pub trait HasLength {
    /// Returns the finite number of elements or bytes according to the type's native length.
    fn length(&self) -> usize;

    /// Returns whether [`HasLength::length`] is zero.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Returns whether [`HasLength::length`] is nonzero.
    #[must_use]
    fn is_not_empty(&self) -> bool {
        !self.is_empty()
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

impl_has_length_for_integer_ranges!(usize, u8, u16, u32, u64, i8, i16, i32, i64);

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn has_length_remains_usable_as_a_trait_object() {
        let values = [1, 2, 3];
        let value: &dyn HasLength = &values;

        assert_that!(value.length()).is_equal_to(3);
        assert_that!(value.is_empty()).is_false();
        assert_that!(value.is_not_empty()).is_true();
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
        check!(usize, u8, u16, u32, u64, i8, i16, i32, i64);

        macro_rules! check_signed {
            ($($type:ty),+) => {$(
                let (low, below_zero, centered): ($type, $type, $type) = (-9, -1, 4);
                assert_that!(low..below_zero).has_length(8);
                assert_that!(low..=below_zero).has_length(9);
                assert_that!(-centered..centered).has_length(8);
                assert_that!(-centered..=centered).has_length(9);
            )+};
        }
        check_signed!(i8, i16, i32, i64);

        // Full domains do not overflow the intermediate difference.
        assert_that!(u8::MIN..=u8::MAX).has_length(256);
        assert_that!(i8::MIN..i8::MAX).has_length(255);
        assert_that!(i8::MIN..=i8::MAX).has_length(256);
    }
}
