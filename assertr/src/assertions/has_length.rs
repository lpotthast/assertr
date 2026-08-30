use ::alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, LinkedList, VecDeque},
    string::String,
    vec::Vec,
};
use ::core::ops::{Range, RangeInclusive};
#[cfg(feature = "std")]
use ::std::hash::BuildHasher;

/// A value whose finite length can be inspected by
/// [`LengthAssertions`](crate::assertions::core::length::LengthAssertions).
///
/// Implement it to make `is_empty`, `is_not_empty`, and `has_length` available on a custom type.
/// Built-in implementations cover strings, collection families, and integer ranges.
///
/// Integer ranges use their mathematical element count converted to `usize`. Asking for the
/// length of a range whose count cannot be represented by `usize` panics with an explicit
/// `range length exceeds usize::MAX` message.
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

impl HasLength for &str {
    fn length(&self) -> usize {
        str::len(self)
    }

    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }
}

impl HasLength for String {
    fn length(&self) -> usize {
        String::len(self)
    }

    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
}

impl HasLength for &String {
    fn length(&self) -> usize {
        String::len(self)
    }

    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
}

impl HasLength for Box<str> {
    fn length(&self) -> usize {
        str::len(self)
    }

    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }
}

impl HasLength for Cow<'_, str> {
    fn length(&self) -> usize {
        str::len(self)
    }

    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }
}

impl<T> HasLength for &[T] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, const S: usize> HasLength for [T; S] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, const S: usize> HasLength for &[T; S] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for Vec<T> {
    fn length(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

impl<T> HasLength for &Vec<T> {
    fn length(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

impl<T> HasLength for VecDeque<T> {
    fn length(&self) -> usize {
        VecDeque::len(self)
    }

    fn is_empty(&self) -> bool {
        VecDeque::is_empty(self)
    }
}

impl<K, V> HasLength for BTreeMap<K, V> {
    fn length(&self) -> usize {
        BTreeMap::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeMap::is_empty(self)
    }
}

impl<K, V> HasLength for &BTreeMap<K, V> {
    fn length(&self) -> usize {
        BTreeMap::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeMap::is_empty(self)
    }
}

impl<T> HasLength for BTreeSet<T> {
    fn length(&self) -> usize {
        BTreeSet::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeSet::is_empty(self)
    }
}

impl<T> HasLength for &BTreeSet<T> {
    fn length(&self) -> usize {
        BTreeSet::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeSet::is_empty(self)
    }
}

impl<T> HasLength for LinkedList<T> {
    fn length(&self) -> usize {
        LinkedList::len(self)
    }

    fn is_empty(&self) -> bool {
        LinkedList::is_empty(self)
    }
}

impl<T> HasLength for &LinkedList<T> {
    fn length(&self) -> usize {
        LinkedList::len(self)
    }

    fn is_empty(&self) -> bool {
        LinkedList::is_empty(self)
    }
}

impl<T> HasLength for &VecDeque<T> {
    fn length(&self) -> usize {
        VecDeque::len(self)
    }

    fn is_empty(&self) -> bool {
        VecDeque::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<K, V, S: BuildHasher> HasLength for ::std::collections::HashMap<K, V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<K, V, S: BuildHasher> HasLength for &::std::collections::HashMap<K, V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<V, S: BuildHasher> HasLength for ::std::collections::HashSet<V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashSet::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashSet::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<V, S: BuildHasher> HasLength for &::std::collections::HashSet<V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashSet::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashSet::is_empty(self)
    }
}

/// Converts a range's mathematical length to `usize`, panicking when it does not fit.
fn range_length<D>(difference: D) -> usize
where
    D: TryInto<usize>,
{
    difference
        .try_into()
        .unwrap_or_else(|_| panic!("range length exceeds usize::MAX"))
}

/// Length of a non-empty inclusive range, `difference + 1`, panicking when it does not fit.
fn inclusive_range_length<D>(difference: D) -> usize
where
    D: TryInto<usize>,
{
    match range_length(difference).checked_add(1) {
        Some(length) => length,
        None => panic!("range length exceeds usize::MAX"),
    }
}

macro_rules! impl_has_length_for_unsigned_ranges {
    ($($type:ty),+ $(,)?) => {
        $(
            impl HasLength for Range<$type> {
                fn length(&self) -> usize {
                    if self.start < self.end {
                        range_length(self.end - self.start)
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
                        inclusive_range_length(*self.end() - *self.start())
                    }
                }
            }
        )+
    };
}

macro_rules! impl_has_length_for_signed_ranges {
    ($($type:ty),+ $(,)?) => {
        $(
            impl HasLength for Range<$type> {
                fn length(&self) -> usize {
                    if self.start < self.end {
                        range_length(self.end.abs_diff(self.start))
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
                        inclusive_range_length(self.end().abs_diff(*self.start()))
                    }
                }
            }
        )+
    };
}

impl_has_length_for_unsigned_ranges!(usize, u8, u16, u32, u64);
impl_has_length_for_signed_ranges!(i8, i16, i32, i64);

#[cfg(test)]
mod tests {
    mod has_length_for_array_references {
        use crate::prelude::*;

        #[test]
        fn supports_length_assertions() {
            let empty: [i32; 0] = [];
            assert_that_owned!(&empty).is_empty().has_length(0);

            let values = [1, 2, 3];
            assert_that_owned!(&values).is_not_empty().has_length(3);
        }
    }

    #[allow(clippy::reversed_empty_ranges)]
    mod has_length {
        #[cfg(feature = "fluent")]
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            (1_usize..9_usize).must().have_length(8);
        }

        mod on_usize_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_usize..9_usize).has_length(8);
                assert_that!(1_usize..=9_usize).has_length(9);
                assert_that!(5_usize..=5_usize).has_length(1);

                // inverted range
                assert_that!(9_usize..1_usize).has_length(0);
                assert_that!(9_usize..=1_usize).has_length(0);
            }

            #[test]
            fn exhausted_inclusive_range_has_length_zero() {
                let mut range = 5_usize..=5_usize;
                assert_eq!(range.next(), Some(5));
                assert!(range.is_empty());

                assert_that!(range).has_length(0);
            }
        }

        mod on_u8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u8..9_u8).has_length(8);
                assert_that!(1_u8..=9_u8).has_length(9);
                assert_that!(u8::MIN..=u8::MAX).has_length(256);

                // inverted range
                assert_that!(9_u8..1_u8).has_length(0);
                assert_that!(9_u8..=1_u8).has_length(0);
            }
        }

        mod on_u16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u16..9_u16).has_length(8);
                assert_that!(1_u16..=9_u16).has_length(9);

                // inverted range
                assert_that!(9_u16..1_u16).has_length(0);
                assert_that!(9_u16..=1_u16).has_length(0);
            }
        }

        mod on_u32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u32..9_u32).has_length(8);
                assert_that!(1_u32..=9_u32).has_length(9);

                // inverted range
                assert_that!(9_u32..1_u32).has_length(0);
                assert_that!(9_u32..=1_u32).has_length(0);
            }
        }

        mod on_u64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u64..9_u64).has_length(8);
                assert_that!(1_u64..=9_u64).has_length(9);

                // inverted range
                assert_that!(9_u64..1_u64).has_length(0);
                assert_that!(9_u64..=1_u64).has_length(0);
            }
        }

        mod on_i8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i8..9_i8).has_length(8);
                assert_that!(1_i8..=9_i8).has_length(9);

                // inverted range
                assert_that!(9_i8..1_i8).has_length(0);
                assert_that!(9_i8..=1_i8).has_length(0);

                // negative range
                assert_that!(-9_i8..-1_i8).has_length(8);
                assert_that!(-9_i8..=-1_i8).has_length(9);

                // across zero
                assert_that!(-4_i8..4_i8).has_length(8);
                assert_that!(-4_i8..=4_i8).has_length(9);

                // full domain without intermediate signed overflow
                assert_that!(i8::MIN..i8::MAX).has_length(255);
                assert_that!(i8::MIN..=i8::MAX).has_length(256);
            }

            #[test]
            fn exhausted_inclusive_range_has_length_zero() {
                let mut range = 5_i8..=5_i8;
                assert_eq!(range.next(), Some(5));
                assert!(range.is_empty());

                assert_that!(range).has_length(0);
            }
        }

        mod on_i16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i16..9_i16).has_length(8);
                assert_that!(1_i16..=9_i16).has_length(9);

                // inverted range
                assert_that!(9_i16..1_i16).has_length(0);
                assert_that!(9_i16..=1_i16).has_length(0);

                // negative range
                assert_that!(-9_i16..-1_i16).has_length(8);
                assert_that!(-9_i16..=-1_i16).has_length(9);

                // across zero
                assert_that!(-4_i16..4_i16).has_length(8);
                assert_that!(-4_i16..=4_i16).has_length(9);
            }
        }

        mod on_i32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i32..9_i32).has_length(8);
                assert_that!(1_i32..=9_i32).has_length(9);

                // inverted range
                assert_that!(9_i32..1_i32).has_length(0);
                assert_that!(9_i32..=1_i32).has_length(0);

                // negative range
                assert_that!(-9_i32..-1_i32).has_length(8);
                assert_that!(-9_i32..=-1_i32).has_length(9);

                // across zero
                assert_that!(-4_i32..4_i32).has_length(8);
                assert_that!(-4_i32..=4_i32).has_length(9);
            }
        }

        mod on_i64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i64..9_i64).has_length(8);
                assert_that!(1_i64..=9_i64).has_length(9);

                // inverted range
                assert_that!(9_i64..1_i64).has_length(0);
                assert_that!(9_i64..=1_i64).has_length(0);

                // negative range
                assert_that!(-9_i64..-1_i64).has_length(8);
                assert_that!(-9_i64..=-1_i64).has_length(9);

                // across zero
                assert_that!(-4_i64..4_i64).has_length(8);
                assert_that!(-4_i64..=4_i64).has_length(9);
            }
        }
    }
}
